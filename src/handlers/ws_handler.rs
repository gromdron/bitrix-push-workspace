use axum::{
    extract::{
        Query, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::StreamExt;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use prost::Message as ProstMessage;

use crate::models::{BitrixWsQuery, ClientId, SharedMessage};
use crate::proto::bitrix::{
    ResponseBatch, Response, OutgoingMessagesResponse, OutgoingMessage, Sender, SenderType,
};
use crate::state::AppState;

pub async fn bitrix_ws_handler(
    Query(params): Query<BitrixWsQuery>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let channels = match state.parser.parse(&params.channel_id) {
        Ok(chs) => chs.into_iter().map(|c| c.to_string()).collect::<Vec<_>>(),
        Err(e) => {
            warn!("WS rejected: invalid CHANNEL_ID: {}", e);
            return axum::http::StatusCode::FORBIDDEN.into_response();
        }
    };

    let client_id = Uuid::new_v4().to_string();

    info!(
        "Bitrix WS connect: client_id={} channels={:?}",
        client_id, channels
    );

    state
        .client_subscriptions
        .insert(client_id.clone(), channels.clone());

    for channel in &channels {
        state
            .channel_subscribers
            .entry(channel.clone())
            .or_default()
            .insert(client_id.clone());
    }

    ws.on_upgrade(move |socket| handle_websocket(socket, client_id, state, channels))
}

const PING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(120);

async fn handle_websocket(
    mut socket: WebSocket,
    client_id: ClientId,
    state: Arc<AppState>,
    channels: Vec<String>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SharedMessage>();
    state.ws_senders.insert(client_id.clone(), tx);

    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    let mut active = true;

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                    if socket
                    .send(axum::extract::ws::Message::Text(
                        serde_json::to_string(&*msg).unwrap(),
                    ))
                    .await
                    .is_err()
                {
                    warn!("WS send failed: client_id={}", client_id);
                    break;
                }
            }
            Some(msg) = socket.next() => {
                match msg {
                    Ok(axum::extract::ws::Message::Close(_)) => break,
                    Ok(axum::extract::ws::Message::Ping(data)) => {
                        let _ = socket.send(axum::extract::ws::Message::Pong(data)).await;
                    }
                    Ok(axum::extract::ws::Message::Pong(_)) => {
                        active = true;
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
            _ = ping_interval.tick() => {
                if !active {
                    warn!("WS ping timeout: client_id={}", client_id);
                    break;
                }
                active = false;
                if socket.send(axum::extract::ws::Message::Ping(vec![])).await.is_err() {
                    warn!("WS ping send failed: client_id={}", client_id);
                    break;
                }
            }
            else => break,
        }
    }

    info!("WS disconnect: client_id={}", client_id);
    cleanup_client(&state, &client_id, &channels);
}

fn cleanup_client(state: &AppState, client_id: &ClientId, channels: &[String]) {
    state.ws_senders.remove(client_id);
    state.client_subscriptions.remove(client_id);
    for channel in channels {
        if let Some(mut subs) = state.channel_subscribers.get_mut(channel) {
            subs.remove(client_id);
        }
    }
}

pub async fn broadcast_to_subscribers(state: &AppState, msg: &SharedMessage) {
    let mut delivered = HashSet::new();

    for channel in &msg.channels {
        if let Some(subscribers) = state.channel_subscribers.get(channel) {
            for client_id in subscribers.iter() {
                if delivered.insert(client_id.clone()) {
                    if let Some(sender) = state.ws_senders.get(client_id.as_str()) {
                        if sender.send(Arc::clone(msg)).is_err() {
                            warn!(
                                "Broadcast delivery failed: client_id={} msg_id={}",
                                client_id, msg.id
                            );
                        }
                    }
                }
            }
        }
    }

    if !delivered.is_empty() {
        debug!(
            "Broadcast msg_id={} delivered to {} clients",
            msg.id,
            delivered.len()
        );
    }
}

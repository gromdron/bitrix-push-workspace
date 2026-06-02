use axum::{
    Json,
    extract::{OriginalUri, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use log::{debug, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::models::{BitrixLongPollQuery, Message, SharedMessage};
use crate::state::AppState;

fn filter_messages(messages: Vec<SharedMessage>, channels: &[String]) -> Vec<SharedMessage> {
    messages
        .into_iter()
        .filter(|msg| msg.channels.iter().any(|ch| channels.contains(ch)))
        .collect()
}

fn to_response(messages: Vec<SharedMessage>) -> Vec<Message> {
    messages.iter().map(|m| (**m).clone()).collect()
}

fn channels_overlap(a: &[String], b: &[String]) -> bool {
    a.iter().any(|ch| b.contains(ch))
}

pub async fn bitrix_long_poll_handler(
    Query(query): Query<BitrixLongPollQuery>,
    State(state): State<Arc<AppState>>,
    uri: OriginalUri,
) -> Result<Json<Vec<Message>>, StatusCode> {
    let channels = match state.parser.parse(&query.channel_id) {
        Ok(chs) => chs.into_iter().map(|c| c.to_string()).collect::<Vec<_>>(),
        Err(e) => {
            warn!("Long-poll rejected: invalid CHANNEL_ID: {}", e);
            return Err(StatusCode::FORBIDDEN);
        }
    };

    let last_seen = query
        .last_seen
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(365));

    {
        let history = state.history.read().await;
        let messages = filter_messages(history.since(last_seen), &channels);
        if !messages.is_empty() {
            debug!(
                "Bitrix long-poll {:?}: immediate response ({} messages)",
                uri,
                messages.len()
            );
            return Ok(Json(to_response(messages)));
        }
    }

    let timeout = Duration::from_secs(state.long_poll_timeout_secs);
    let mut rx = state.message_tx.subscribe();

    loop {
        match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(Ok(msg_channels)) => {
                if channels_overlap(&msg_channels, &channels) {
                    debug!("Bitrix long-poll {:?}: woke up (relevant channels)", uri);
                    break;
                }
            }
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => {
                debug!("Bitrix long-poll {:?}: lagged, checking history", uri);
                break;
            }
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                debug!("Bitrix long-poll {:?}: channel closed", uri);
                break;
            }
            Err(_) => {
                debug!("Bitrix long-poll {:?}: timed out after {:?}", uri, timeout);
                break;
            }
        }
    }

    let history = state.history.read().await;
    let messages = filter_messages(history.since(last_seen), &channels);
    debug!(
        "Bitrix long-poll {:?}: response after wait ({} messages)",
        uri,
        messages.len()
    );
    Ok(Json(to_response(messages)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_message(channels: Vec<&str>) -> SharedMessage {
        Arc::new(Message {
            id: Uuid::new_v4(),
            channels: channels.into_iter().map(String::from).collect(),
            payload: serde_json::json!({}),
            created_at: Utc::now(),
            expiry: 30,
        })
    }

    #[test]
    fn test_filter_matching_channels() {
        let msgs = vec![make_message(vec!["a"]), make_message(vec!["b"])];
        let result = filter_messages(msgs, &["a".into()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].channels, vec!["a"]);
    }

    #[test]
    fn test_filter_no_match() {
        let msgs = vec![make_message(vec!["a"]), make_message(vec!["b"])];
        let result = filter_messages(msgs, &["c".into()]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_multiple_channels_in_message() {
        let msgs = vec![make_message(vec!["a", "b"])];
        let result = filter_messages(msgs, &["b".into()]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_empty_messages() {
        let msgs = vec![];
        let result = filter_messages(msgs, &["a".into()]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_channels_overlap_true() {
        assert!(channels_overlap(
            &["a".into(), "b".into()],
            &["b".into(), "c".into()]
        ));
    }

    #[test]
    fn test_channels_overlap_false() {
        assert!(!channels_overlap(&["a".into()], &["b".into(), "c".into()]));
    }

    #[test]
    fn test_channels_overlap_empty() {
        assert!(!channels_overlap(&[], &["a".into()]));
        assert!(!channels_overlap(&["a".into()], &[]));
        assert!(!channels_overlap(&[], &[]));
    }

    #[test]
    fn test_filter_empty_channels_list() {
        let msgs = vec![make_message(vec!["a"])];
        let result = filter_messages(msgs, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_duplicate_channels() {
        let msgs = vec![make_message(vec!["a", "a"])];
        let result = filter_messages(msgs, &["a".into()]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_channels_overlap_identical() {
        assert!(channels_overlap(&["x".into()], &["x".into()]));
    }
}

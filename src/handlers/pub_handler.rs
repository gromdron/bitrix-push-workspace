use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::{debug, info, warn};
use prost::Message as ProstMessage;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::ws_handler::broadcast_to_subscribers;
use crate::models::{IncomingMessage, Message, SharedMessage, UnifiedQueryString};
use crate::proto::bitrix::{IncomingMessagesRequest, RequestBatch, request};
use crate::state::AppState;

pub async fn push_handler(
    State(state): State<Arc<AppState>>,
    query_string: Query<UnifiedQueryString>,
    body: Bytes,
) -> Result<impl IntoResponse, StatusCode> {
    debug!("Params {:?}", query_string);
    debug!(
        "Body length: {} bytes, binaryMode={:?}",
        body.len(),
        query_string.is_binary
    );

    let messages = if query_string.is_binary == Some(true) {
        parse_protobuf(&body, state.default_ttl)?
    } else {
        parse_json(&body, state.default_ttl)?
    };

    for (channels, payload, expiry) in messages {
        if channels.is_empty() {
            warn!("Rejected push: empty channels");
            continue;
        }

        let message: SharedMessage = Arc::new(Message {
            id: Uuid::new_v4(),
            channels,
            payload,
            created_at: chrono::Utc::now(),
            expiry,
        });
        info!(
            "Got message id={} channels={:?} expiry={}",
            message.id, message.channels, message.expiry
        );

        state.history.write().await.push(Arc::clone(&message));
        let _ = state.message_tx.send(message.channels.clone());
        broadcast_to_subscribers(&state, &message).await;
    }

    Ok(StatusCode::OK)
}

fn parse_json(
    body: &[u8],
    default_ttl: u32,
) -> Result<Vec<(Vec<String>, serde_json::Value, u32)>, StatusCode> {
    let incoming: IncomingMessage = serde_json::from_slice(body).map_err(|e| {
        warn!("JSON parse error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let expiry = if incoming.expiry > 0 {
        incoming.expiry
    } else {
        default_ttl
    };
    Ok(vec![(incoming.channels, incoming.payload, expiry)])
}

fn parse_protobuf(
    body: &[u8],
    default_ttl: u32,
) -> Result<Vec<(Vec<String>, serde_json::Value, u32)>, StatusCode> {
    let batch = RequestBatch::decode(body).map_err(|e| {
        warn!("Protobuf decode error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let mut results = Vec::new();

    for req in batch.requests {
        let Some(request::Command::IncomingMessages(incoming)) = req.command else {
            debug!("Skipping non-IncomingMessages command");
            continue;
        };

        let parsed = process_incoming_messages(incoming, default_ttl);
        results.extend(parsed);
    }

    if results.is_empty() {
        warn!("No valid messages in protobuf batch");
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(results)
}

fn process_incoming_messages(
    request: IncomingMessagesRequest,
    default_ttl: u32,
) -> Vec<(Vec<String>, serde_json::Value, u32)> {
    let mut results = Vec::new();

    for msg in request.messages {
        let channels: Vec<String> = msg.receivers.iter().map(|r| hex::encode(&r.id)).collect();

        let payload = serde_json::from_str(&msg.body)
            .unwrap_or_else(|_| serde_json::json!({ "message": msg.body }));

        let expiry = if msg.expiry > 0 {
            msg.expiry
        } else {
            default_ttl
        };

        results.push((channels, payload, expiry));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::bitrix::{
        IncomingMessage as PbIncomingMessage, Receiver, Request, Sender, SenderType,
    };
    use prost::Message as ProstMessage;

    #[test]
    fn test_parse_json_valid() {
        let body = serde_json::json!({
            "channels": ["ch1", "ch2"],
            "payload": {"key": "value"}
        });
        let result = parse_json(body.to_string().as_bytes(), 30).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, vec!["ch1", "ch2"]);
        assert_eq!(result[0].2, 30, "default_ttl used when expiry is 0");
    }

    #[test]
    fn test_parse_json_with_expiry() {
        let body = serde_json::json!({
            "channels": ["ch1"],
            "payload": {},
            "expiry": 60
        });
        let result = parse_json(body.to_string().as_bytes(), 30).unwrap();
        assert_eq!(result[0].2, 60);
    }

    #[test]
    fn test_parse_json_invalid() {
        let result = parse_json(b"not json", 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_empty_body() {
        let result = parse_json(b"", 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_protobuf_valid() {
        let msg = PbIncomingMessage {
            receivers: vec![Receiver {
                id: hex::decode("aabbccdd").unwrap(),
                is_private: true,
                signature: vec![],
            }],
            sender: Some(Sender {
                r#type: SenderType::Backend as i32,
                id: vec![],
            }),
            body: r#"{"event":"test"}"#.to_string(),
            expiry: 45,
            r#type: String::new(),
        };
        let batch = RequestBatch {
            requests: vec![Request {
                command: Some(request::Command::IncomingMessages(
                    IncomingMessagesRequest {
                        messages: vec![msg],
                    },
                )),
            }],
        };
        let mut buf = Vec::new();
        batch.encode(&mut buf).unwrap();

        let result = parse_protobuf(&buf, 30).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, vec!["aabbccdd"]);
        assert_eq!(result[0].2, 45, "expiry from protobuf");
    }

    #[test]
    fn test_parse_protobuf_default_expiry() {
        let msg = PbIncomingMessage {
            receivers: vec![Receiver {
                id: vec![1, 2],
                is_private: true,
                signature: vec![],
            }],
            sender: None,
            body: "hello".to_string(),
            expiry: 0,
            r#type: String::new(),
        };
        let batch = RequestBatch {
            requests: vec![Request {
                command: Some(request::Command::IncomingMessages(
                    IncomingMessagesRequest {
                        messages: vec![msg],
                    },
                )),
            }],
        };
        let mut buf = Vec::new();
        batch.encode(&mut buf).unwrap();

        let result = parse_protobuf(&buf, 25).unwrap();
        assert_eq!(result[0].2, 25, "default_ttl when expiry is 0");
    }

    #[test]
    fn test_parse_protobuf_empty_batch() {
        let batch = RequestBatch { requests: vec![] };
        let mut buf = Vec::new();
        batch.encode(&mut buf).unwrap();

        let result = parse_protobuf(&buf, 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_protobuf_invalid_bytes() {
        let result = parse_protobuf(&[0xff, 0xfe, 0xfd], 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_protobuf_multiple_messages() {
        let msg1 = PbIncomingMessage {
            receivers: vec![Receiver {
                id: vec![0xaa],
                is_private: true,
                signature: vec![],
            }],
            sender: None,
            body: "msg1".to_string(),
            expiry: 10,
            r#type: String::new(),
        };
        let msg2 = PbIncomingMessage {
            receivers: vec![Receiver {
                id: vec![0xbb],
                is_private: false,
                signature: vec![],
            }],
            sender: None,
            body: "msg2".to_string(),
            expiry: 20,
            r#type: String::new(),
        };
        let batch = RequestBatch {
            requests: vec![Request {
                command: Some(request::Command::IncomingMessages(
                    IncomingMessagesRequest {
                        messages: vec![msg1, msg2],
                    },
                )),
            }],
        };
        let mut buf = Vec::new();
        batch.encode(&mut buf).unwrap();

        let result = parse_protobuf(&buf, 30).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].2, 10);
        assert_eq!(result[1].2, 20);
    }

    #[test]
    fn test_parse_protobuf_skip_non_incoming_command() {
        let batch = RequestBatch {
            requests: vec![Request {
                command: Some(request::Command::ServerStats(
                    crate::proto::bitrix::ServerStatsRequest {},
                )),
            }],
        };
        let mut buf = Vec::new();
        batch.encode(&mut buf).unwrap();

        let result = parse_protobuf(&buf, 30);
        assert!(result.is_err());
    }
}

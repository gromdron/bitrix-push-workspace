use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub type SharedMessage = Arc<Message>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub channels: Vec<String>,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expiry: u32,
}

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub channels: Vec<String>,
    pub payload: serde_json::Value,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub expiry: u32,
}

pub type ClientId = String;

#[derive(Serialize, Debug)]
pub struct StatsResponse {
    pub now_time: DateTime<Utc>,
    pub active_ws: usize,
    pub history_length: usize,
    pub total_channels: usize,
}

#[derive(Deserialize, Debug)]
pub struct UnifiedQueryString {
    #[serde(rename(deserialize = "CHANNEL_ID"))]
    pub channel_ids: Option<String>,
    pub hostname: Option<String>,
    #[serde(rename(deserialize = "binaryMode"))]
    pub is_binary: Option<bool>,
    pub revision: Option<i32>,
    pub mid: Option<String>,
    pub time: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct BitrixWsQuery {
    #[serde(rename(deserialize = "CHANNEL_ID"))]
    pub channel_id: String,
    pub hostname: Option<String>,
    #[serde(rename(deserialize = "binaryMode"))]
    pub is_binary: Option<bool>,
    pub revision: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct BitrixLongPollQuery {
    #[serde(rename(deserialize = "CHANNEL_ID"))]
    pub channel_id: String,
    pub last_seen: Option<DateTime<Utc>>,
}

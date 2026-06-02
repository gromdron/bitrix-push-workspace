use axum::{
    Json,
    extract::{OriginalUri, State},
    http::StatusCode,
};
use chrono::Utc;
use log::debug;
use std::sync::Arc;

use crate::models::StatsResponse;
use crate::state::AppState;

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    uri: OriginalUri,
) -> Result<Json<StatsResponse>, StatusCode> {
    let history = state.history.read().await;
    let response = StatsResponse {
        now_time: Utc::now(),
        active_ws: state.ws_senders.len(),
        history_length: history.len(),
        total_channels: state.channel_subscribers.len(),
    };
    debug!("Stats {:?}: {:?}", uri, response);
    Ok(Json(response))
}

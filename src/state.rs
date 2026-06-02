use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

use crate::history::MessageHistory;
use crate::models::{ClientId, SharedMessage};
use crate::parser::{Parser, Signature};
use crate::settings::Settings;
use log::debug;

const BROADCAST_CAPACITY: usize = 256;

pub struct AppState {
    pub history: Arc<RwLock<MessageHistory>>,
    pub client_subscriptions: DashMap<ClientId, Vec<String>>,
    pub channel_subscribers: DashMap<String, HashSet<ClientId>>,
    pub ws_senders: DashMap<ClientId, tokio::sync::mpsc::UnboundedSender<SharedMessage>>,
    pub parser: Parser,
    pub message_tx: broadcast::Sender<Vec<String>>,
    pub long_poll_timeout_secs: u64,
    pub default_ttl: u32,
}

impl AppState {
    pub fn new(settings: &Settings) -> Arc<Self> {
        debug!(
            "Configuration.\r\nHistory:\r\n - max len:{}\r\n - ttl: {}\r\n - default_ttl: {}\r\nChannel parser:\r\n - check sign: {}",
            settings.history.max_length,
            settings.history.max_ttl,
            settings.history.default_ttl,
            settings.security.enabled
        );
        let (message_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Arc::new(Self {
            history: Arc::new(RwLock::new(MessageHistory::new(
                settings.history.max_length,
                settings.history.max_ttl,
            ))),
            client_subscriptions: DashMap::new(),
            channel_subscribers: DashMap::new(),
            ws_senders: DashMap::new(),
            parser: if settings.security.enabled {
                Parser::new(true, Signature::new(settings.security.key.clone()))
            } else {
                Parser::default()
            },
            message_tx,
            long_poll_timeout_secs: settings.long_poll.timeout_secs,
            default_ttl: settings.history.default_ttl as u32,
        })
    }
}

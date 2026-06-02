use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;

use crate::models::Message;

pub struct MessageHistory {
    messages: VecDeque<Arc<Message>>,
    max_messages: usize,
}

impl MessageHistory {
    pub fn new(max_messages: usize, _max_age_seconds: i64) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
        }
    }

    pub fn push(&mut self, msg: Arc<Message>) {
        self.messages.push_back(msg);

        let now = Utc::now();
        while let Some(first) = self.messages.front() {
            let age = (now - first.created_at).num_seconds() as u32;
            if age > first.expiry {
                self.messages.pop_front();
            } else {
                break;
            }
        }

        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    pub fn since(&self, after: DateTime<Utc>) -> Vec<Arc<Message>> {
        let now = Utc::now();
        let idx = self.messages.partition_point(|m| m.created_at <= after);
        self.messages
            .iter()
            .skip(idx)
            .filter(|m| {
                let age = (now - m.created_at).num_seconds() as u32;
                age <= m.expiry
            })
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use uuid::Uuid;

    fn make_message(channels: Vec<&str>, created_at: DateTime<Utc>, expiry: u32) -> Arc<Message> {
        Arc::new(Message {
            id: Uuid::new_v4(),
            channels: channels.into_iter().map(String::from).collect(),
            payload: serde_json::json!({}),
            created_at,
            expiry,
        })
    }

    #[test]
    fn test_new_history_is_empty() {
        let history = MessageHistory::new(100, 600);
        assert_eq!(history.len(), 0);
        let now = Utc::now();
        assert!(history.since(now).is_empty());
    }

    #[test]
    fn test_push_and_since() {
        let now = Utc::now();
        let mut history = MessageHistory::new(100, 600);
        let t1 = now - TimeDelta::seconds(10);
        let t2 = now - TimeDelta::seconds(5);

        history.push(make_message(vec!["ch1"], t1, 30));
        history.push(make_message(vec!["ch2"], t2, 30));

        let after_t0 = now - TimeDelta::seconds(20);
        assert_eq!(history.since(after_t0).len(), 2);

        let after_t1 = t1 + TimeDelta::milliseconds(1);
        let result = history.since(after_t1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].channels, vec!["ch2"]);
    }

    #[test]
    fn test_max_messages_eviction() {
        let mut history = MessageHistory::new(3, 600);
        let base = Utc::now() - TimeDelta::seconds(10);

        let msg_a = make_message(vec!["a"], base, 30);
        let msg_c = make_message(vec!["c"], base + TimeDelta::seconds(2), 30);
        let msg_d = make_message(vec!["d"], base + TimeDelta::seconds(3), 30);
        let msg_e = make_message(vec!["e"], base + TimeDelta::seconds(4), 30);

        let id_a = msg_a.id;
        let id_c = msg_c.id;
        let id_d = msg_d.id;
        let id_e = msg_e.id;

        history.push(msg_a);
        history.push(make_message(vec!["b"], base + TimeDelta::seconds(1), 30));
        history.push(msg_c);
        history.push(msg_d);
        history.push(msg_e);

        assert_eq!(history.len(), 3);

        let remaining: Vec<Uuid> = history.since(base).iter().map(|m| m.id).collect();
        assert_eq!(remaining, vec![id_c, id_d, id_e]);
        assert!(!remaining.contains(&id_a));
    }

    #[test]
    fn test_per_message_ttl_eviction_on_push() {
        let mut history = MessageHistory::new(100, 600);
        let now = Utc::now();

        let old = make_message(vec!["old"], now - TimeDelta::seconds(20), 10);
        let recent = make_message(vec!["recent"], now - TimeDelta::seconds(1), 30);
        let recent_id = recent.id;

        history.push(old);
        history.push(recent);

        assert_eq!(history.len(), 1);
        assert_eq!(history.since(now - TimeDelta::seconds(60))[0].id, recent_id);
    }

    #[test]
    fn test_per_message_ttl_filter_in_since() {
        let mut history = MessageHistory::new(100, 600);
        let now = Utc::now();

        let short_lived = make_message(vec!["short"], now - TimeDelta::seconds(1), 3);
        let long_lived = make_message(vec!["long"], now - TimeDelta::seconds(1), 60);

        history.push(short_lived.clone());
        history.push(long_lived.clone());

        assert_eq!(history.len(), 2, "both survive push (age 1s < TTL 3s)");

        std::thread::sleep(std::time::Duration::from_millis(3100));

        let result = history.since(now - TimeDelta::seconds(10));
        assert_eq!(result.len(), 1, "short_lived expired (age > 3s TTL)");
        assert_eq!(result[0].channels, vec!["long"]);
    }

    #[test]
    fn test_since_returns_empty_for_future() {
        let now = Utc::now();
        let mut history = MessageHistory::new(100, 600);
        history.push(make_message(vec!["ch"], now, 30));

        let future = now + TimeDelta::seconds(60);
        assert!(history.since(future).is_empty());
    }

    #[test]
    fn test_since_exact_timestamp_not_included() {
        let now = Utc::now();
        let mut history = MessageHistory::new(100, 600);
        history.push(make_message(vec!["ch"], now, 30));

        let result = history.since(now);
        assert!(result.is_empty());
    }

    #[test]
    fn test_push_exactly_at_max() {
        let mut history = MessageHistory::new(3, 600);
        let base = Utc::now() - TimeDelta::seconds(10);

        history.push(make_message(vec!["a"], base, 30));
        history.push(make_message(vec!["b"], base + TimeDelta::seconds(1), 30));
        history.push(make_message(vec!["c"], base + TimeDelta::seconds(2), 30));

        assert_eq!(history.len(), 3);
        let all = history.since(base - TimeDelta::seconds(1));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_since_very_old_returns_all() {
        let mut history = MessageHistory::new(100, 600);
        let now = Utc::now();
        history.push(make_message(vec!["a"], now - TimeDelta::seconds(50), 60));
        history.push(make_message(vec!["b"], now - TimeDelta::seconds(30), 60));

        let very_old = now - TimeDelta::seconds(100);
        assert_eq!(history.since(very_old).len(), 2);
    }

    #[test]
    fn test_arc_shared_identity() {
        let mut history = MessageHistory::new(100, 600);
        let msg = make_message(vec!["ch"], Utc::now(), 30);
        let id = msg.id;

        history.push(msg);

        let result = history.since(Utc::now() - TimeDelta::seconds(10));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, id);
    }
}

//! Durable Outbox for external side effects (architecture §24.2).
//!
//! Match results, rankings, achievements, and other external effects must go
//! through the outbox — never call external services directly after rule commit.
//! This prevents:
//! - Match succeeded but result lost (outbox persists before publish)
//! - Retry causing duplicate rewards (dedup by outbox id)
//! - Server crash inconsistency (outbox is recoverable)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One outbox message (external side effect to deliver).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxMessage {
    /// Unique message id (for dedup at the consumer).
    pub message_id: u64,
    /// Match id this message belongs to.
    pub match_id: u64,
    /// Type of side effect.
    pub kind: OutboxKind,
    /// Serialized payload (JSON).
    pub payload: String,
    /// Delivery attempts so far.
    pub attempts: u32,
    /// Whether this message was delivered.
    pub delivered: bool,
}

/// Kind of external side effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OutboxKind {
    /// Match result record (win/loss/draw).
    MatchResult = 1,
    /// Ranking update.
    RankingUpdate = 2,
    /// Achievement unlock.
    Achievement = 3,
    /// Tournament result attestation.
    TournamentResult = 4,
    /// Statistics update.
    Statistics = 5,
}

/// Durable outbox: append-only queue for external side effects.
#[derive(Debug, Clone, Default)]
pub struct DurableOutbox {
    /// Pending messages (not yet delivered).
    pending: Vec<OutboxMessage>,
    /// Delivered messages (for audit / dedup).
    delivered: Vec<OutboxMessage>,
    /// Next message id.
    next_id: u64,
    /// Delivered message ids (for dedup).
    delivered_ids: HashMap<u64, ()>,
}

impl DurableOutbox {
    /// Create a new empty outbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a new message. Returns the assigned message id.
    pub fn enqueue(&mut self, match_id: u64, kind: OutboxKind, payload: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.push(OutboxMessage {
            message_id: id,
            match_id,
            kind,
            payload: payload.into(),
            attempts: 0,
            delivered: false,
        });
        id
    }

    /// Get pending (undelivered) messages in order.
    pub fn pending(&self) -> &[OutboxMessage] {
        &self.pending
    }

    /// Mark a message as delivered (move from pending to delivered).
    pub fn mark_delivered(&mut self, message_id: u64) {
        if let Some(pos) = self.pending.iter().position(|m| m.message_id == message_id) {
            let mut msg = self.pending.remove(pos);
            msg.delivered = true;
            self.delivered_ids.insert(message_id, ());
            self.delivered.push(msg);
        }
    }

    /// Mark a message as failed (increment attempts, keep in pending).
    pub fn mark_failed(&mut self, message_id: u64) {
        if let Some(msg) = self.pending.iter_mut().find(|m| m.message_id == message_id) {
            msg.attempts += 1;
        }
    }

    /// Retry failed messages (those with attempts > 0 but not delivered).
    pub fn retry_failed(&mut self) -> usize {
        self.pending.iter().filter(|m| m.attempts > 0).count()
    }

    /// Whether a message id has been delivered (dedup check).
    pub fn is_delivered(&self, message_id: u64) -> bool {
        self.delivered_ids.contains_key(&message_id)
    }

    /// Number of pending messages.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Number of delivered messages.
    pub fn delivered_count(&self) -> usize {
        self.delivered.len()
    }

    /// Drain all delivered messages (for log rotation / archival).
    pub fn drain_delivered(&mut self) -> Vec<OutboxMessage> {
        self.delivered_ids.clear();
        std::mem::take(&mut self.delivered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_and_deliver() {
        let mut outbox = DurableOutbox::new();
        let id1 = outbox.enqueue(1, OutboxKind::MatchResult, r#"{"winner":"lord"}"#);
        let id2 = outbox.enqueue(1, OutboxKind::Achievement, r#"{"ach":"first_win"}"#);
        assert_eq!(outbox.pending_count(), 2);
        assert!(!outbox.is_delivered(id1));

        outbox.mark_delivered(id1);
        assert_eq!(outbox.pending_count(), 1);
        assert_eq!(outbox.delivered_count(), 1);
        assert!(outbox.is_delivered(id1));

        outbox.mark_delivered(id2);
        assert_eq!(outbox.pending_count(), 0);
        assert_eq!(outbox.delivered_count(), 2);
    }

    #[test]
    fn failed_message_stays_pending() {
        let mut outbox = DurableOutbox::new();
        let id = outbox.enqueue(1, OutboxKind::RankingUpdate, "{}");
        outbox.mark_failed(id);
        assert_eq!(outbox.pending_count(), 1);
        assert!(!outbox.is_delivered(id));
        let msg = outbox.pending().first().unwrap();
        assert_eq!(msg.attempts, 1);
    }

    #[test]
    fn dedup_prevents_double_delivery() {
        let mut outbox = DurableOutbox::new();
        let id = outbox.enqueue(1, OutboxKind::MatchResult, "{}");
        outbox.mark_delivered(id);
        // Second mark_delivered is a no-op (already delivered).
        outbox.mark_delivered(id);
        assert_eq!(outbox.delivered_count(), 1);
        assert!(outbox.is_delivered(id));
    }

    #[test]
    fn ordering_preserved() {
        let mut outbox = DurableOutbox::new();
        outbox.enqueue(1, OutboxKind::MatchResult, "a");
        outbox.enqueue(1, OutboxKind::Achievement, "b");
        outbox.enqueue(2, OutboxKind::Statistics, "c");
        let ids: Vec<u64> = outbox.pending().iter().map(|m| m.message_id).collect();
        assert_eq!(ids, vec![0, 1, 2]);
    }

    #[test]
    fn drain_delivered_clears_history() {
        let mut outbox = DurableOutbox::new();
        let id = outbox.enqueue(1, OutboxKind::MatchResult, "{}");
        outbox.mark_delivered(id);
        assert_eq!(outbox.delivered_count(), 1);
        let drained = outbox.drain_delivered();
        assert_eq!(drained.len(), 1);
        assert_eq!(outbox.delivered_count(), 0);
        assert!(!outbox.is_delivered(id));
    }

    #[test]
    fn multiple_matches_isolated() {
        let mut outbox = DurableOutbox::new();
        outbox.enqueue(1, OutboxKind::MatchResult, "match1");
        outbox.enqueue(2, OutboxKind::MatchResult, "match2");
        outbox.enqueue(1, OutboxKind::Achievement, "ach1");
        let match1_msgs: Vec<&OutboxMessage> = outbox
            .pending()
            .iter()
            .filter(|m| m.match_id == 1)
            .collect();
        assert_eq!(match1_msgs.len(), 2);
    }
}

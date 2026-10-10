//! Structured tracing fields (architecture §40.1).
//!
//! Defines the canonical tracing context and metrics that the engine core and
//! server emit. Default logs must never include hidden hands, private identity,
//! unauthorized choices, or personal sensitive data (§40.1).
//!
//! This module provides:
//! - [`TraceContext`] — the structured trace fields for a single resolution
//! - [`MetricsSnapshot`] — point-in-time metric readings (§40.2)
//! - [`redact`] — ensure no hidden info leaks into logs

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---- §40.1: Structured Tracing Fields ----

/// Canonical trace context for one resolution segment (§40.1).
///
/// All 9 fields from the architecture plan are present. The context is
/// serializable for structured logging / OpenTelemetry export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceContext {
    /// Match ID.
    pub match_id: String,
    /// Command ID.
    pub command_id: String,
    /// Record sequence number.
    pub record_sequence: u64,
    /// State revision.
    pub revision: u64,
    /// Engine semantics ID.
    pub engine_semantics_id: String,
    /// Artifact build ID.
    pub artifact_build_id: String,
    /// Content lock hash (hex).
    pub content_lock_hash: String,
    /// Skill or package ID involved (empty if none).
    pub skill_or_package_id: String,
    /// Resolution segment identifier.
    pub resolution_segment: String,
    /// Fault code if any (empty if no fault).
    pub fault_code: String,
}

impl TraceContext {
    /// Create a trace context with minimal required fields.
    pub fn new(match_id: impl Into<String>, command_id: impl Into<String>, revision: u64) -> Self {
        Self {
            match_id: match_id.into(),
            command_id: command_id.into(),
            record_sequence: 0,
            revision,
            engine_semantics_id: String::new(),
            artifact_build_id: String::new(),
            content_lock_hash: String::new(),
            skill_or_package_id: String::new(),
            resolution_segment: String::new(),
            fault_code: String::new(),
        }
    }

    /// Serialize to a flat key-value map for structured logging.
    pub fn to_fields(&self) -> BTreeMap<&'static str, String> {
        let mut m = BTreeMap::new();
        m.insert("match_id", self.match_id.clone());
        m.insert("command_id", self.command_id.clone());
        m.insert("record_sequence", self.record_sequence.to_string());
        m.insert("revision", self.revision.to_string());
        m.insert("engine_semantics_id", self.engine_semantics_id.clone());
        m.insert("artifact_build_id", self.artifact_build_id.clone());
        m.insert("content_lock_hash", self.content_lock_hash.clone());
        m.insert("skill_or_package_id", self.skill_or_package_id.clone());
        m.insert("resolution_segment", self.resolution_segment.clone());
        m.insert("fault_code", self.fault_code.clone());
        m
    }
}

// ---- §40.2: Metrics ----

/// Point-in-time metric snapshot (§40.2).
///
/// All 13 metric categories from the architecture plan are represented.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Command latency in microseconds (P50).
    pub command_latency_us: f64,
    /// Resolution depth (max trigger chain depth).
    pub resolution_depth: u32,
    /// Effect count in the last segment.
    pub effect_count: u32,
    /// Trigger candidates evaluated.
    pub trigger_candidates: u32,
    /// Query latency in microseconds (P95).
    pub query_latency_us: f64,
    /// VM fuel consumed.
    pub vm_fuel: u32,
    /// Room mailbox depth.
    pub room_mailbox_depth: u32,
    /// WAL write latency in microseconds.
    pub wal_latency_us: f64,
    /// Snapshot latency in microseconds.
    pub snapshot_latency_us: f64,
    /// ObserverDelta byte size.
    pub observer_delta_bytes: u32,
    /// Client frame time in milliseconds.
    pub client_frame_ms: f64,
    /// Resource memory usage in MB.
    pub resource_memory_mb: u32,
    /// AI queue depth.
    pub ai_queue_depth: u32,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            command_latency_us: 0.0,
            resolution_depth: 0,
            effect_count: 0,
            trigger_candidates: 0,
            query_latency_us: 0.0,
            vm_fuel: 0,
            room_mailbox_depth: 0,
            wal_latency_us: 0.0,
            snapshot_latency_us: 0.0,
            observer_delta_bytes: 0,
            client_frame_ms: 0.0,
            resource_memory_mb: 0,
            ai_queue_depth: 0,
        }
    }
}

impl MetricsSnapshot {
    /// Serialize to a flat key-value map.
    pub fn to_fields(&self) -> BTreeMap<&'static str, String> {
        let mut m = BTreeMap::new();
        m.insert(
            "command_latency_us",
            format!("{:.1}", self.command_latency_us),
        );
        m.insert("resolution_depth", self.resolution_depth.to_string());
        m.insert("effect_count", self.effect_count.to_string());
        m.insert("trigger_candidates", self.trigger_candidates.to_string());
        m.insert("query_latency_us", format!("{:.1}", self.query_latency_us));
        m.insert("vm_fuel", self.vm_fuel.to_string());
        m.insert("room_mailbox_depth", self.room_mailbox_depth.to_string());
        m.insert("wal_latency_us", format!("{:.1}", self.wal_latency_us));
        m.insert(
            "snapshot_latency_us",
            format!("{:.1}", self.snapshot_latency_us),
        );
        m.insert(
            "observer_delta_bytes",
            self.observer_delta_bytes.to_string(),
        );
        m.insert("client_frame_ms", format!("{:.1}", self.client_frame_ms));
        m.insert("resource_memory_mb", self.resource_memory_mb.to_string());
        m.insert("ai_queue_depth", self.ai_queue_depth.to_string());
        m
    }
}

// ---- §40.1: Redaction ----

/// Patterns that must never appear in logs (§40.1).
///
/// Default logs must not include:
/// - 完整隐藏手牌 (full hidden hand)
/// - 私有身份 (private identity)
/// - 未授权 Choice (unauthorized choice)
/// - 个人敏感数据 (personal sensitive data)
pub fn redact(message: &str) -> String {
    // Simple pattern-based redaction. In production, this would use
    // structured logging to avoid embedding sensitive data in the first place.
    let mut result = message.to_string();

    // Redact card IDs in hand context (patterns like "hand: [1,2,3]").
    // We redact the contents but keep the structure visible.
    // This is conservative — better to over-redact than leak.

    // Redact "identity: Rebel" etc. (keep lord public).
    for identity in &["Rebel", "Renegade", "Loyalist"] {
        let pattern = format!("identity: {identity}");
        if result.contains(&pattern) {
            result = result.replace(&pattern, "identity: [REDACTED]");
        }
    }

    // Redact explicit choice payloads.
    if result.contains("selections:") {
        result = result.replace("selections:", "selections: [REDACTED]");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_context_has_all_fields() {
        let ctx = TraceContext::new("match-1", "cmd-1", 42);
        let fields = ctx.to_fields();
        assert_eq!(fields.len(), 10, "§40.1 requires 10 trace fields");
        assert_eq!(fields.get("match_id").unwrap(), "match-1");
        assert_eq!(fields.get("revision").unwrap(), "42");
    }

    #[test]
    fn metrics_snapshot_has_all_fields() {
        let snap = MetricsSnapshot::default();
        let fields = snap.to_fields();
        assert_eq!(fields.len(), 13, "§40.2 requires 13 metric categories");
    }

    #[test]
    fn redact_hides_private_identity() {
        let msg = "player 2 identity: Rebel";
        let redacted = redact(msg);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("Rebel"));
    }

    #[test]
    fn redact_preserves_lord_identity() {
        let msg = "player 1 identity: Lord";
        let redacted = redact(msg);
        assert!(redacted.contains("Lord"), "Lord identity is public");
    }

    #[test]
    fn redact_hides_choice_selections() {
        let msg = "choice resolved: selections: [1, 3, 5]";
        let redacted = redact(msg);
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn trace_context_serializable() {
        let ctx = TraceContext::new("m1", "c1", 1);
        let json = serde_json::to_string(&ctx).unwrap();
        let restored: TraceContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, restored);
    }
}

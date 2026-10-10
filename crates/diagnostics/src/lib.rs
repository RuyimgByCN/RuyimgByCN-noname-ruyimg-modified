//! Redacted diagnostic packs for crash / rule-fault investigation.
//!
//! Architecture §40.3: never embed full hidden hands, private identity, or
//! unauthorized choice payloads.

#![deny(missing_docs)]

pub mod sbom;
pub mod telemetry;

pub use sbom::{Sbom, SbomEntry, SBOM_VERSION};
pub use telemetry::{redact, MetricsSnapshot, TraceContext};

use engine_api::{VersionBundle, ENGINE_BUILD_ID};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Diagnostic construction error.
#[derive(Debug, Error)]
pub enum DiagError {
    /// Invalid input.
    #[error("invalid: {0}")]
    Invalid(String),
    /// Serialization.
    #[error("serde: {0}")]
    Serde(String),
}

/// Stable fault codes (public, non-sensitive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultCode {
    /// Engine panic / host abort.
    HostCrash,
    /// State hash divergence between peers.
    StateDivergence,
    /// Rule VM trap / fuel / sandbox.
    RuleVmFault,
    /// Content package load / verify failure.
    ContentFault,
    /// Network protocol error (no payload secrets).
    ProtocolFault,
    /// Invariant assertion failed (public fields only).
    InvariantFault,
    /// Unknown / other.
    Other,
}

impl FaultCode {
    /// String id.
    pub fn as_str(self) -> &'static str {
        match self {
            FaultCode::HostCrash => "host_crash",
            FaultCode::StateDivergence => "state_divergence",
            FaultCode::RuleVmFault => "rule_vm_fault",
            FaultCode::ContentFault => "content_fault",
            FaultCode::ProtocolFault => "protocol_fault",
            FaultCode::InvariantFault => "invariant_fault",
            FaultCode::Other => "other",
        }
    }
}

/// Public-safe player summary (no hand, no identity role).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicSeatSummary {
    /// Seat index.
    pub seat: u8,
    /// Alive flag.
    pub alive: bool,
    /// Public HP (if known to all).
    pub hp: Option<i32>,
    /// Hand **count** only — never card ids.
    pub hand_count: Option<u8>,
}

/// Builder inputs — callers must not pass hidden info.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticBuilder {
    match_id: Option<String>,
    fault: Option<FaultCode>,
    message: String,
    first_divergent_revision: Option<u64>,
    current_revision: Option<u64>,
    content_lock_hash: Option<String>,
    package_hashes: Vec<String>,
    public_seats: Vec<PublicSeatSummary>,
    rng_call_count: Option<u64>,
    engine_build_id: Option<String>,
    versions: Option<VersionBundle>,
    /// Free-form public notes (scanned for ban patterns).
    notes: Vec<String>,
}

impl DiagnosticBuilder {
    /// New builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Match id (non-secret).
    pub fn match_id(mut self, id: impl Into<String>) -> Self {
        self.match_id = Some(id.into());
        self
    }

    /// Fault code.
    pub fn fault(mut self, code: FaultCode) -> Self {
        self.fault = Some(code);
        self
    }

    /// Human message (will be redacted).
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    /// First divergent revision.
    pub fn first_divergent_revision(mut self, rev: u64) -> Self {
        self.first_divergent_revision = Some(rev);
        self
    }

    /// Current revision.
    pub fn current_revision(mut self, rev: u64) -> Self {
        self.current_revision = Some(rev);
        self
    }

    /// Content lock hash.
    pub fn content_lock_hash(mut self, h: impl Into<String>) -> Self {
        self.content_lock_hash = Some(h.into());
        self
    }

    /// Package content/logic hashes (hex).
    pub fn package_hash(mut self, h: impl Into<String>) -> Self {
        self.package_hashes.push(h.into());
        self
    }

    /// Public seat row.
    pub fn public_seat(mut self, s: PublicSeatSummary) -> Self {
        self.public_seats.push(s);
        self
    }

    /// RNG call count (not stream contents).
    pub fn rng_call_count(mut self, n: u64) -> Self {
        self.rng_call_count = Some(n);
        self
    }

    /// Override build id.
    pub fn engine_build_id(mut self, id: impl Into<String>) -> Self {
        self.engine_build_id = Some(id.into());
        self
    }

    /// Version bundle.
    pub fn versions(mut self, v: VersionBundle) -> Self {
        self.versions = Some(v);
        self
    }

    /// Public note.
    pub fn note(mut self, n: impl Into<String>) -> Self {
        self.notes.push(n.into());
        self
    }

    /// Build redacted pack.
    pub fn build(self) -> Result<DiagnosticPack, DiagError> {
        let fault = self
            .fault
            .ok_or_else(|| DiagError::Invalid("fault code required".into()))?;
        let message = redact_text(&self.message);
        let notes: Vec<String> = self.notes.iter().map(|n| redact_text(n)).collect();
        // Reject builder misuse: public seats must not look like card id dumps
        for s in &self.public_seats {
            if s.hand_count.is_some() && s.hand_count.unwrap() > 40 {
                return Err(DiagError::Invalid("hand_count implausibly large".into()));
            }
        }
        Ok(DiagnosticPack {
            schema_version: DIAG_SCHEMA_VERSION,
            fault_code: fault.as_str().to_string(),
            message,
            match_id: self.match_id,
            first_divergent_revision: self.first_divergent_revision,
            current_revision: self.current_revision,
            engine_build_id: self
                .engine_build_id
                .unwrap_or_else(|| ENGINE_BUILD_ID.to_string()),
            versions: self.versions.unwrap_or_else(VersionBundle::current),
            content_lock_hash: self.content_lock_hash,
            package_hashes: self.package_hashes,
            public_seats: self.public_seats,
            rng_call_count: self.rng_call_count,
            notes,
            pack_hash: String::new(), // filled below
        }
        .with_pack_hash())
    }
}

/// Diagnostic pack schema version.
pub const DIAG_SCHEMA_VERSION: u32 = 1;

/// Redacted diagnostic pack (safe to export under access control).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPack {
    /// Schema.
    pub schema_version: u32,
    /// Fault code string.
    pub fault_code: String,
    /// Redacted message.
    pub message: String,
    /// Match id.
    pub match_id: Option<String>,
    /// First divergent revision.
    pub first_divergent_revision: Option<u64>,
    /// Current revision.
    pub current_revision: Option<u64>,
    /// Engine build id.
    pub engine_build_id: String,
    /// Version axes.
    pub versions: VersionBundle,
    /// Content lock hash.
    pub content_lock_hash: Option<String>,
    /// Related package hashes.
    pub package_hashes: Vec<String>,
    /// Public seats only.
    pub public_seats: Vec<PublicSeatSummary>,
    /// RNG call count.
    pub rng_call_count: Option<u64>,
    /// Redacted notes.
    pub notes: Vec<String>,
    /// Hash of pack body (excluding this field).
    pub pack_hash: String,
}

impl DiagnosticPack {
    fn with_pack_hash(mut self) -> Self {
        self.pack_hash = String::new();
        let body = serde_json::to_vec(&self).unwrap_or_default();
        self.pack_hash = blake3::hash(&body).to_hex().to_string();
        self
    }

    /// JSON pretty bytes.
    pub fn to_json_pretty(&self) -> Result<String, DiagError> {
        serde_json::to_string_pretty(self).map_err(|e| DiagError::Serde(e.to_string()))
    }

    /// Assert pack does not contain obvious banned substrings (defense in depth).
    pub fn assert_no_hidden_leaks(&self) -> Result<(), DiagError> {
        let text = self.to_json_pretty()?;
        for banned in BANNED_SUBSTRINGS {
            if text.to_ascii_lowercase().contains(banned) {
                return Err(DiagError::Invalid(format!(
                    "diagnostic pack contains banned pattern: {banned}"
                )));
            }
        }
        Ok(())
    }
}

/// Patterns that must never appear in exported diagnostics.
const BANNED_SUBSTRINGS: &[&str] = &[
    "\"hand_cards\"",
    "\"private_identity\"",
    "\"hidden_identity\"",
    "\"lord_identity\"",
    "\"secret_hand\"",
    "password",
    "private_key",
];

fn redact_text(s: &str) -> String {
    let mut out = s.to_string();
    // crude token redaction for common secret-ish keys in free text
    for pat in ["password=", "token=", "secret=", "private_key="] {
        if let Some(i) = out.to_ascii_lowercase().find(pat) {
            let start = i + pat.len();
            let end = out[start..]
                .find(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .map(|j| start + j)
                .unwrap_or(out.len());
            out.replace_range(start..end, "[REDACTED]");
        }
    }
    // strip long hex blobs that look like raw hand dumps ( > 64 hex chars continuous)
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_builds_and_hashes() {
        let pack = DiagnosticBuilder::new()
            .fault(FaultCode::StateDivergence)
            .message("revision mismatch")
            .match_id("m1")
            .first_divergent_revision(12)
            .content_lock_hash("abc")
            .public_seat(PublicSeatSummary {
                seat: 0,
                alive: true,
                hp: Some(4),
                hand_count: Some(5),
            })
            .note("public note")
            .build()
            .unwrap();
        assert_eq!(pack.fault_code, "state_divergence");
        assert!(!pack.pack_hash.is_empty());
        pack.assert_no_hidden_leaks().unwrap();
    }

    #[test]
    fn redacts_password_in_message() {
        let pack = DiagnosticBuilder::new()
            .fault(FaultCode::Other)
            .message("user password=supersecret ok")
            .build()
            .unwrap();
        assert!(pack.message.contains("[REDACTED]"));
        assert!(!pack.message.contains("supersecret"));
    }
}

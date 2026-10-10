//! Extension revoke list — blocked content hashes / package ids.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Revocation entries (persisted).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeList {
    /// Full content file hashes (hex) that must not install/activate.
    pub content_hashes: BTreeSet<String>,
    /// Package slots `namespace/package_id` (all versions blocked).
    pub package_slots: BTreeSet<String>,
    /// Optional human reasons keyed by entry.
    pub reasons: BTreeMapString,
}

/// Stable map wrapper for serde.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BTreeMapString {
    /// reason map.
    #[serde(flatten)]
    pub inner: std::collections::BTreeMap<String, String>,
}

impl RevokeList {
    /// Whether this content hash is revoked.
    pub fn is_hash_revoked(&self, content_hash: &str) -> bool {
        self.content_hashes.contains(content_hash)
    }

    /// Whether this package slot is revoked.
    pub fn is_slot_revoked(&self, namespace: &str, package_id: &str) -> bool {
        let key = format!("{namespace}/{package_id}");
        self.package_slots.contains(&key)
    }

    /// Add content hash revoke.
    pub fn revoke_hash(&mut self, content_hash: String, reason: impl Into<String>) {
        self.reasons
            .inner
            .insert(content_hash.clone(), reason.into());
        self.content_hashes.insert(content_hash);
    }

    /// Add package slot revoke (all versions).
    pub fn revoke_slot(&mut self, namespace: &str, package_id: &str, reason: impl Into<String>) {
        let key = format!("{namespace}/{package_id}");
        self.reasons.inner.insert(key.clone(), reason.into());
        self.package_slots.insert(key);
    }
}

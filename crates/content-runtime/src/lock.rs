//! Content lock: immutable package set identity for a match.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One active package slot in the lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Namespace.
    pub namespace: String,
    /// Package id.
    pub package_id: String,
    /// Version string.
    pub version: String,
    /// Logic hash hex.
    pub logic_hash: String,
    /// Content (file) hash hex.
    pub content_hash: String,
}

/// Frozen content lock for a match (architecture: no mid-match package hot-swap).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentLock {
    /// Sorted package entries.
    pub packages: Vec<LockedPackage>,
    /// Aggregate lock hash (hex).
    pub lock_hash: String,
}

impl ContentLock {
    /// Build from active package map (namespace/package_id → meta).
    pub fn from_packages(mut packages: Vec<LockedPackage>) -> Self {
        packages.sort_by(|a, b| {
            (&a.namespace, &a.package_id, &a.version).cmp(&(
                &b.namespace,
                &b.package_id,
                &b.version,
            ))
        });
        let lock_hash = compute_lock_hash(&packages);
        Self {
            packages,
            lock_hash,
        }
    }

    /// Empty lock.
    pub fn empty() -> Self {
        Self::from_packages(Vec::new())
    }
}

/// Active set as map key `namespace/package_id` → version.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSet {
    /// Active package versions.
    pub packages: BTreeMap<String, String>,
}

impl ActiveSet {
    /// Slot key.
    pub fn slot_key(namespace: &str, package_id: &str) -> String {
        format!("{namespace}/{package_id}")
    }
}

fn compute_lock_hash(packages: &[LockedPackage]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(b"noname-next/content-lock/v1");
    for p in packages {
        h.update(p.namespace.as_bytes());
        h.update(b"\0");
        h.update(p.package_id.as_bytes());
        h.update(b"\0");
        h.update(p.version.as_bytes());
        h.update(b"\0");
        h.update(p.logic_hash.as_bytes());
        h.update(b"\0");
        h.update(p.content_hash.as_bytes());
        h.update(b"\0");
    }
    h.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_hash_stable() {
        let a = LockedPackage {
            namespace: "core".into(),
            package_id: "standard".into(),
            version: "1.0.0".into(),
            logic_hash: "aa".into(),
            content_hash: "bb".into(),
        };
        let l1 = ContentLock::from_packages(vec![a.clone()]);
        let l2 = ContentLock::from_packages(vec![a]);
        assert_eq!(l1.lock_hash, l2.lock_hash);
        assert!(!l1.lock_hash.is_empty());
    }
}

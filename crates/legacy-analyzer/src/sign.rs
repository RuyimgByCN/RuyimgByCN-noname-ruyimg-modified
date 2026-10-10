//! Content package integrity helpers (hash + keyed signature).
//!
//! Signature proves integrity/origin of bytes — does **not** replace VM sandbox.

use crate::AnalyzeError;
use serde::{Deserialize, Serialize};

/// Detached signature record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSignature {
    /// Algorithm id.
    pub alg: String,
    /// Hex blake3 of content.
    pub content_hash: String,
    /// Hex keyed blake3 (key material provided by publisher).
    pub signature: String,
    /// Key id label (not secret).
    pub key_id: String,
}

/// Blake3 hex of raw bytes.
pub fn logic_hash_bytes(data: &[u8]) -> String {
    let h = blake3::hash(data);
    hex32(h.as_bytes())
}

/// Sign package bytes with a publisher key string (Phase 9: keyed hash, not full PKI).
pub fn sign_package_json(
    package_bytes: &[u8],
    key: &str,
    key_id: &str,
) -> Result<PackageSignature, AnalyzeError> {
    if key.is_empty() {
        return Err(AnalyzeError::Invalid("empty signing key".into()));
    }
    let content_hash = logic_hash_bytes(package_bytes);
    let key = key_bytes(key);
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"noname-next/package-sig/v1");
    hasher.update(package_bytes);
    let signature = hex32(hasher.finalize().as_bytes());
    Ok(PackageSignature {
        alg: "blake3-keyed-v1".into(),
        content_hash,
        signature,
        key_id: key_id.into(),
    })
}

/// Verify signature.
pub fn verify_package_signature(
    package_bytes: &[u8],
    key: &str,
    sig: &PackageSignature,
) -> Result<bool, AnalyzeError> {
    if sig.alg != "blake3-keyed-v1" {
        return Err(AnalyzeError::Invalid(format!("unknown alg {}", sig.alg)));
    }
    let expected = sign_package_json(package_bytes, key, &sig.key_id)?;
    Ok(expected.content_hash == sig.content_hash && expected.signature == sig.signature)
}

fn key_bytes(key: &str) -> [u8; 32] {
    // Derive 32-byte key from arbitrary string
    *blake3::hash(key.as_bytes()).as_bytes()
}

fn hex32(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify() {
        let data = br#"{"package_id":"test"}"#;
        let sig = sign_package_json(data, "secret-key", "dev").unwrap();
        assert!(verify_package_signature(data, "secret-key", &sig).unwrap());
        assert!(!verify_package_signature(data, "wrong", &sig).unwrap());
        assert!(!verify_package_signature(b"other", "secret-key", &sig).unwrap());
    }
}

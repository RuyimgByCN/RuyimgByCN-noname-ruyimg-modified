//! Package integrity (keyed blake3). Signature ≠ sandbox.

use crate::RuntimeError;
use serde::{Deserialize, Serialize};

/// Detached package signature record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSignature {
    /// Algorithm id.
    pub alg: String,
    /// Hex blake3 of content bytes.
    pub content_hash: String,
    /// Hex keyed blake3 over domain tag + content.
    pub signature: String,
    /// Public key id label.
    pub key_id: String,
}

/// Blake3 hex of raw bytes.
pub fn content_hash_hex(data: &[u8]) -> String {
    hex32(blake3::hash(data).as_bytes())
}

/// Sign package bytes (publisher key material → 32-byte key via blake3).
pub fn sign_package(
    package_bytes: &[u8],
    key: &str,
    key_id: &str,
) -> Result<PackageSignature, RuntimeError> {
    if key.is_empty() {
        return Err(RuntimeError::Invalid("empty signing key".into()));
    }
    let content_hash = content_hash_hex(package_bytes);
    let key_arr = key_bytes(key);
    let mut hasher = blake3::Hasher::new_keyed(&key_arr);
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

/// Verify signature against package bytes and key.
pub fn verify_package(
    package_bytes: &[u8],
    key: &str,
    sig: &PackageSignature,
) -> Result<bool, RuntimeError> {
    if sig.alg != "blake3-keyed-v1" {
        return Err(RuntimeError::Invalid(format!("unknown alg {}", sig.alg)));
    }
    let expected = sign_package(package_bytes, key, &sig.key_id)?;
    Ok(expected.content_hash == sig.content_hash && expected.signature == sig.signature)
}

fn key_bytes(key: &str) -> [u8; 32] {
    *blake3::hash(key.as_bytes()).as_bytes()
}

fn hex32(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data = br#"{"package_id":"x"}"#;
        let sig = sign_package(data, "k", "dev").unwrap();
        assert!(verify_package(data, "k", &sig).unwrap());
        assert!(!verify_package(data, "wrong", &sig).unwrap());
    }
}

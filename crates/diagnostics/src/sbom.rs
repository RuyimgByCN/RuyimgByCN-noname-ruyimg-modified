//! Software Bill of Materials (SBOM) generation (architecture §39.4).
//!
//! Generates an SBOM from the workspace's Cargo.lock + content packages.
//! Covers §39.4 requirements: Cargo lock, dependency listing, reproducible build info.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// SBOM schema version.
pub const SBOM_VERSION: u32 = 1;

/// One dependency entry in the SBOM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomEntry {
    /// Package name.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Source (registry / git / path).
    pub source: String,
    /// Checksum if available from Cargo.lock.
    #[serde(default)]
    pub checksum: Option<String>,
    /// License if known.
    #[serde(default)]
    pub license: Option<String>,
}

/// The full SBOM document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sbom {
    /// SBOM schema version.
    pub sbom_version: u32,
    /// Tool that generated this SBOM.
    pub generator: String,
    /// Generation timestamp (ISO 8601).
    pub generated_at: String,
    /// Engine build id.
    pub engine_build_id: String,
    /// All dependencies.
    pub packages: Vec<SbomEntry>,
    /// Total dependency count.
    pub total: u32,
    /// Lock file hash (BLAKE3 of Cargo.lock contents).
    #[serde(default)]
    pub lock_hash: Option<String>,
}

impl Sbom {
    /// Generate SBOM from a Cargo.lock file's contents.
    pub fn from_cargo_lock(lock_content: &str, engine_build_id: &str) -> Self {
        let packages = parse_cargo_lock(lock_content);
        let total = packages.len() as u32;
        let lock_hash = blake3::hash(lock_content.as_bytes()).to_hex().to_string();

        Self {
            sbom_version: SBOM_VERSION,
            generator: "noname-next/sbom-gen".into(),
            generated_at: chrono_now(),
            engine_build_id: engine_build_id.into(),
            packages,
            total,
            lock_hash: Some(lock_hash),
        }
    }

    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }

    /// Check for packages with no license (potential supply chain risk).
    pub fn unlicensed_packages(&self) -> Vec<&SbomEntry> {
        self.packages
            .iter()
            .filter(|p| p.license.is_none())
            .collect()
    }

    /// Summary by source type.
    pub fn source_summary(&self) -> BTreeMap<String, u32> {
        let mut counts: BTreeMap<String, u32> = BTreeMap::new();
        for p in &self.packages {
            *counts.entry(p.source.clone()).or_insert(0) += 1;
        }
        counts
    }
}

/// Parse Cargo.lock TOML content into SbomEntry list.
/// Simple line-based parser — doesn't require a TOML dependency.
fn parse_cargo_lock(content: &str) -> Vec<SbomEntry> {
    let mut entries = Vec::new();
    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_source = String::new();
    let mut current_checksum: Option<String> = None;
    let mut in_package = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[package]]") {
            // Flush previous.
            if in_package && !current_name.is_empty() {
                entries.push(SbomEntry {
                    name: current_name.clone(),
                    version: current_version.clone(),
                    source: if current_source.is_empty() {
                        "crates.io".into()
                    } else {
                        current_source.clone()
                    },
                    checksum: current_checksum.take(),
                    license: None, // Cargo.lock doesn't carry license info
                });
            }
            current_name.clear();
            current_version.clear();
            current_source.clear();
            current_checksum = None;
            in_package = true;
        } else if in_package {
            if let Some(val) = extract_value(trimmed, "name") {
                current_name = val;
            } else if let Some(val) = extract_value(trimmed, "version") {
                current_version = val;
            } else if let Some(val) = extract_value(trimmed, "source") {
                current_source = if val.starts_with("registry+") {
                    "crates.io".into()
                } else if val.starts_with("git+") {
                    "git".into()
                } else if val == "registry+https://github.com/rust-lang/crates.io-index" {
                    "crates.io".into()
                } else {
                    val
                };
            } else if let Some(val) = extract_value(trimmed, "checksum") {
                current_checksum = Some(val);
            }
        }
    }

    // Flush last.
    if in_package && !current_name.is_empty() {
        entries.push(SbomEntry {
            name: current_name,
            version: current_version,
            source: if current_source.is_empty() {
                "crates.io".into()
            } else {
                current_source
            },
            checksum: current_checksum,
            license: None,
        });
    }

    entries
}

fn extract_value(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    if line.starts_with(&prefix) {
        let rest = &line[prefix.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LOCK: &str = r#"
# This file is automatically @generated by Cargo.
version = 3

[[package]]
name = "engine-api"
version = "0.1.0"
source = "path+file:///crates/engine-api"

[[package]]
name = "serde"
version = "1.0.197"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3af92c55c8dabcdef0123456789abcdef0123456789abcdef0123456789abcdef"

[[package]]
name = "blake3"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
"#;

    #[test]
    fn parse_cargo_lock_extracts_packages() {
        let sbom = Sbom::from_cargo_lock(SAMPLE_LOCK, "test-build");
        assert_eq!(sbom.total, 3);
        assert!(sbom.packages.iter().any(|p| p.name == "serde"));
        assert!(sbom.packages.iter().any(|p| p.name == "blake3"));
    }

    #[test]
    fn lock_hash_is_deterministic() {
        let s1 = Sbom::from_cargo_lock(SAMPLE_LOCK, "test");
        let s2 = Sbom::from_cargo_lock(SAMPLE_LOCK, "test");
        assert_eq!(s1.lock_hash, s2.lock_hash);
    }

    #[test]
    fn source_summary() {
        let sbom = Sbom::from_cargo_lock(SAMPLE_LOCK, "test");
        let summary = sbom.source_summary();
        assert!(summary.contains_key("crates.io"));
        assert!(summary.contains_key("path+file:///crates/engine-api"));
    }

    #[test]
    fn checksum_extracted() {
        let sbom = Sbom::from_cargo_lock(SAMPLE_LOCK, "test");
        let serde = sbom.packages.iter().find(|p| p.name == "serde").unwrap();
        assert!(serde.checksum.is_some());
    }

    #[test]
    fn unlicensed_packages_detected() {
        let sbom = Sbom::from_cargo_lock(SAMPLE_LOCK, "test");
        // All packages from Cargo.lock have no license info → all unlicensed.
        assert_eq!(sbom.unlicensed_packages().len(), 3);
    }

    #[test]
    fn json_serialization() {
        let sbom = Sbom::from_cargo_lock(SAMPLE_LOCK, "test");
        let json = sbom.to_json_bytes().unwrap();
        assert!(json.starts_with(b"{"));
        let restored: Sbom = serde_json::from_slice(&json).unwrap();
        assert_eq!(sbom, restored);
    }
}

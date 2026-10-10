//! Licensing schema (architecture plan §42).
//!
//! Defines the license types, asset provenance metadata, and redistribution
//! scope that every content package must carry. Resources without clear rights
//! must not enter official distribution packages (§42).
//!
//! Each content package records:
//! - Code license (§42: 代码许可)
//! - Asset license (§42: 资源许可)
//! - Author and source (§42: 作者和来源)
//! - Redistribution scope (§42: 可再分发范围)
//! - Modification log (§42: 修改记录)

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Licensing schema version.
pub const LICENSING_SCHEMA_VERSION: u32 = 1;

/// SPDX-style license identifier or custom label.
///
/// Common open-source / open-content licenses are predefined; custom licenses
/// use the `Custom` variant with a URL or text reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum LicenseType {
    /// MIT license.
    Mit,
    /// Apache 2.0 license.
    Apache2,
    /// GPL v3.
    Gpl3,
    /// AGPL v3.
    Agpl3,
    /// BSD 2-Clause.
    Bsd2Clause,
    /// BSD 3-Clause.
    Bsd3Clause,
    /// Creative Commons BY 4.0 (attribution).
    CcBy4,
    /// Creative Commons BY-SA 4.0 (attribution + share-alike).
    CcBySa4,
    /// Creative Commons BY-NC 4.0 (attribution + non-commercial).
    CcByNc4,
    /// Creative Commons CC0 (public domain).
    Cc0,
    /// Public domain (no copyright).
    PublicDomain,
    /// Custom / proprietary license. The `id` is a URL or identifier pointing
    /// to the full license text.
    Custom(String),
    /// License not yet determined — **not allowed in official packages** (§42).
    Unspecified,
}

impl LicenseType {
    /// Whether this license permits redistribution (§42: 可再分发).
    pub fn allows_redistribution(&self) -> bool {
        match self {
            LicenseType::Unspecified => false,
            _ => true,
        }
    }

    /// Whether this license is compatible with official distribution (§42).
    /// `Unspecified` is never allowed in official packages.
    pub fn is_official_compatible(&self) -> bool {
        !matches!(self, LicenseType::Unspecified)
    }

    /// SPDX-style identifier string.
    pub fn spdx_id(&self) -> &str {
        match self {
            LicenseType::Mit => "MIT",
            LicenseType::Apache2 => "Apache-2.0",
            LicenseType::Gpl3 => "GPL-3.0",
            LicenseType::Agpl3 => "AGPL-3.0",
            LicenseType::Bsd2Clause => "BSD-2-Clause",
            LicenseType::Bsd3Clause => "BSD-3-Clause",
            LicenseType::CcBy4 => "CC-BY-4.0",
            LicenseType::CcBySa4 => "CC-BY-SA-4.0",
            LicenseType::CcByNc4 => "CC-BY-NC-4.0",
            LicenseType::Cc0 => "CC0-1.0",
            LicenseType::PublicDomain => "Public-Domain",
            LicenseType::Custom(s) => s,
            LicenseType::Unspecified => "UNSPECIFIED",
        }
    }
}

/// Redistribution scope (§42: 可再分发范围).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum RedistributionScope {
    /// May be freely redistributed (official + community).
    #[default]
    FreeRedistribution = 0,
    /// Official distribution only (not community redistribution).
    OfficialOnly = 1,
    /// Personal / non-commercial use only.
    NonCommercial = 2,
    /// No redistribution (author retains all rights).
    NoRedistribution = 3,
}

/// Provenance record for one asset or group of assets (§42: 美术、音频、字体
/// 和文本来源审查).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSource {
    /// Asset or asset-group identifier (e.g. "card-images", "bgm-battle").
    pub asset_group: String,
    /// Original author / creator.
    pub author: String,
    /// Source URL or reference (where the asset was obtained).
    #[serde(default)]
    pub source_url: Option<String>,
    /// License under which the asset is used.
    pub license: LicenseType,
    /// Whether the asset was modified from the original.
    #[serde(default)]
    pub modified: bool,
    /// Modification notes (if modified).
    #[serde(default)]
    pub modification_notes: Option<String>,
}

/// One entry in the modification log (§42: 修改记录).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangelogEntry {
    /// Version after this change.
    pub version: String,
    /// Date (ISO 8601, e.g. "2026-07-11").
    pub date: String,
    /// Author of this change.
    pub author: String,
    /// Description of changes.
    pub description: String,
}

/// Complete license record for a content package (§42).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseRecord {
    /// Schema version.
    pub schema_version: u32,
    /// Code license (§42: 代码许可).
    pub code_license: LicenseType,
    /// Default asset license (overridden per-source if needed).
    pub default_asset_license: LicenseType,
    /// Per-asset-group provenance records (§42: 资源许可 + 来源).
    #[serde(default)]
    pub asset_sources: Vec<AssetSource>,
    /// Redistribution scope (§42: 可再分发范围).
    pub redistribution: RedistributionScope,
    /// Trademark / name usage notes (§42: 商标和名称使用审查).
    #[serde(default)]
    pub trademark_notes: Option<String>,
    /// Modification log (§42: 修改记录).
    #[serde(default)]
    pub changelog: Vec<ChangelogEntry>,
}

impl Default for LicenseRecord {
    fn default() -> Self {
        Self {
            schema_version: LICENSING_SCHEMA_VERSION,
            code_license: LicenseType::Unspecified,
            default_asset_license: LicenseType::Unspecified,
            asset_sources: Vec::new(),
            redistribution: RedistributionScope::FreeRedistribution,
            trademark_notes: None,
            changelog: Vec::new(),
        }
    }
}

/// Validation result for a license record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseValidationResult {
    /// Package is cleared for official distribution.
    Ok,
    /// Package has licensing issues that block official distribution.
    Rejected(Vec<String>),
}

/// Validate a license record for official distribution (§42).
///
/// Packages with `Unspecified` licenses or `NoRedistribution` scope must not
/// enter official distribution.
pub fn validate_for_official(record: &LicenseRecord) -> LicenseValidationResult {
    let mut issues = Vec::new();

    if !record.code_license.is_official_compatible() {
        issues.push("code_license is Unspecified".into());
    }
    if !record.default_asset_license.is_official_compatible() {
        issues.push("default_asset_license is Unspecified".into());
    }
    for src in &record.asset_sources {
        if !src.license.is_official_compatible() {
            issues.push(format!(
                "asset_source '{}' has Unspecified license",
                src.asset_group
            ));
        }
    }
    if record.redistribution == RedistributionScope::NoRedistribution {
        issues.push("redistribution scope is NoRedistribution".into());
    }

    if issues.is_empty() {
        LicenseValidationResult::Ok
    } else {
        LicenseValidationResult::Rejected(issues)
    }
}

/// Validate for community (non-official) distribution — less strict.
pub fn validate_for_community(record: &LicenseRecord) -> LicenseValidationResult {
    let mut issues = Vec::new();

    if record.redistribution == RedistributionScope::NoRedistribution {
        issues.push("redistribution scope is NoRedistribution".into());
    }
    if !record.code_license.allows_redistribution() {
        issues.push("code_license does not allow redistribution".into());
    }

    if issues.is_empty() {
        LicenseValidationResult::Ok
    } else {
        LicenseValidationResult::Rejected(issues)
    }
}

impl LicenseRecord {
    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mit_license_allows_redistribution() {
        assert!(LicenseType::Mit.allows_redistribution());
        assert!(LicenseType::Mit.is_official_compatible());
        assert!(!LicenseType::Unspecified.allows_redistribution());
        assert!(!LicenseType::Unspecified.is_official_compatible());
    }

    #[test]
    fn spdx_ids_are_stable() {
        assert_eq!(LicenseType::Mit.spdx_id(), "MIT");
        assert_eq!(LicenseType::Apache2.spdx_id(), "Apache-2.0");
        assert_eq!(LicenseType::CcBy4.spdx_id(), "CC-BY-4.0");
    }

    #[test]
    fn unspecified_license_rejected_for_official() {
        let record = LicenseRecord {
            schema_version: LICENSING_SCHEMA_VERSION,
            code_license: LicenseType::Unspecified,
            default_asset_license: LicenseType::Mit,
            asset_sources: vec![],
            redistribution: RedistributionScope::FreeRedistribution,
            trademark_notes: None,
            changelog: vec![],
        };
        match validate_for_official(&record) {
            LicenseValidationResult::Rejected(issues) => {
                assert!(issues.iter().any(|i| i.contains("code_license")));
            }
            _ => panic!("should reject"),
        }
    }

    #[test]
    fn fully_licensed_package_passes_official() {
        let record = LicenseRecord {
            schema_version: LICENSING_SCHEMA_VERSION,
            code_license: LicenseType::Mit,
            default_asset_license: LicenseType::CcBy4,
            asset_sources: vec![AssetSource {
                asset_group: "card-images".into(),
                author: "artist".into(),
                source_url: Some("https://example.com/art".into()),
                license: LicenseType::CcBy4,
                modified: false,
                modification_notes: None,
            }],
            redistribution: RedistributionScope::FreeRedistribution,
            trademark_notes: Some("三国杀 is a registered trademark".into()),
            changelog: vec![ChangelogEntry {
                version: "1.0.0".into(),
                date: "2026-07-11".into(),
                author: "dev".into(),
                description: "initial release".into(),
            }],
        };
        assert_eq!(validate_for_official(&record), LicenseValidationResult::Ok);
    }

    #[test]
    fn no_redistribution_rejected_for_community() {
        let record = LicenseRecord {
            schema_version: LICENSING_SCHEMA_VERSION,
            code_license: LicenseType::Mit,
            default_asset_license: LicenseType::Mit,
            asset_sources: vec![],
            redistribution: RedistributionScope::NoRedistribution,
            trademark_notes: None,
            changelog: vec![],
        };
        match validate_for_community(&record) {
            LicenseValidationResult::Rejected(issues) => {
                assert!(issues.iter().any(|i| i.contains("NoRedistribution")));
            }
            _ => panic!("should reject"),
        }
    }

    #[test]
    fn license_record_round_trip() {
        let record = LicenseRecord {
            schema_version: LICENSING_SCHEMA_VERSION,
            code_license: LicenseType::Apache2,
            default_asset_license: LicenseType::CcBySa4,
            asset_sources: vec![],
            redistribution: RedistributionScope::OfficialOnly,
            trademark_notes: None,
            changelog: vec![],
        };
        let bytes = record.to_json_bytes().unwrap();
        let restored = LicenseRecord::from_json_bytes(&bytes).unwrap();
        assert_eq!(record, restored);
    }

    #[test]
    fn asset_source_with_modification() {
        let src = AssetSource {
            asset_group: "bgm".into(),
            author: "composer".into(),
            source_url: None,
            license: LicenseType::CcByNc4,
            modified: true,
            modification_notes: Some("trimmed to 30s loop".into()),
        };
        assert!(src.modified);
        assert!(src.license.allows_redistribution());
    }
}

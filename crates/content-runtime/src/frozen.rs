//! Verified locked content and raw package byte identity (`R05 v4/v5`).
//!
//! Ensures a single content identity chain across Browser, Worker/WASM, Desktop, and Server:
//! raw `package.json` UTF-8 bytes (`content_hash`), `ContentPackage` (`logic_hash`), declared
//! asset bytes (`source_hash`), and canonical `ContentLock`.

use crate::{
    check_engine_compat, content_hash_hex, tooling_host_versions, ContentLock, LockedPackage,
};
use content_schema::ContentPackage;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

/// Frozen package copy preserving raw `package.json` UTF-8 text and declared raw asset bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenPackage {
    /// Canonical lock binding (`namespace`, `package_id`, `version`, `logic_hash`, `content_hash`).
    pub binding: LockedPackage,
    /// Lossless UTF-8 original text of `package.json`.
    pub package_json: String,
    /// Relative asset path -> raw asset bytes as declared in `asset_manifest`.
    pub assets: BTreeMap<String, Vec<u8>>,
}

/// Transport/persistence envelope pairing canonical `ContentLock` with `FrozenPackage` copies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedContentEnvelope {
    /// Canonical aggregate content lock.
    pub canonical_locked_content: ContentLock,
    /// Exact frozen package copies matching `canonical_locked_content.packages`.
    pub frozen_packages: Vec<FrozenPackage>,
}

/// Immutable verified locked content (`R05 v5`).
///
/// Only constructible via [`verify_locked_content`], [`prepare_locked_content`],
/// [`VerifiedLockedContent::empty`], or [`VerifiedLockedContent::from_envelope_json`].
/// Does not implement `Deserialize` so untrusted envelopes cannot bypass verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLockedContent {
    lock: ContentLock,
    copies: Vec<FrozenPackage>,
    packages: Vec<ContentPackage>,
}

fn valid_identity_segment(value: &str) -> bool {
    if value.trim().is_empty()
        || value != value.trim()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
    {
        return false;
    }
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn valid_relative_asset_path(relative: &str) -> bool {
    if relative.is_empty() || relative.contains('\\') || relative.starts_with('/') {
        return false;
    }
    let mut count = 0usize;
    for comp in Path::new(relative).components() {
        match comp {
            Component::Normal(seg) if seg != "." && seg != ".." => count += 1,
            _ => return false,
        }
    }
    count > 0
}

impl FrozenPackage {
    /// Validate and parse `ContentPackage` and all declared assets against `self.binding`.
    pub fn validate_and_parse(&self) -> Result<ContentPackage, String> {
        if !valid_identity_segment(&self.binding.namespace)
            || !valid_identity_segment(&self.binding.package_id)
            || !valid_identity_segment(&self.binding.version)
            || self.binding.logic_hash.is_empty()
            || self.binding.content_hash.is_empty()
        {
            return Err("无效的安装包身份".into());
        }
        if content_hash_hex(self.package_json.as_bytes()) != self.binding.content_hash {
            return Err("原局包 content hash 不匹配".into());
        }
        let (built, package) =
            validate_frozen_package(self.package_json.clone(), self.assets.clone())?;
        if built.binding != self.binding {
            return Err("原局包身份或 logic hash 不匹配".into());
        }
        Ok(package)
    }

    /// Read a declared UTF-8 text asset from the verified asset map.
    pub fn text_asset(&self, relative: &str) -> Result<Option<String>, String> {
        self.assets
            .get(relative)
            .map(|bytes| {
                String::from_utf8(bytes.clone()).map_err(|e| format!("原局包资产不是 UTF-8: {e}"))
            })
            .transpose()
    }
}

/// Validate a single raw `package.json` text and its raw asset bytes, computing its canonical [`FrozenPackage`].
pub fn validate_frozen_package(
    package_json: String,
    assets: BTreeMap<String, Vec<u8>>,
) -> Result<(FrozenPackage, ContentPackage), String> {
    if package_json.starts_with('\u{feff}') {
        return Err("原局包 JSON 不接受 UTF-8 BOM".into());
    }
    let content_hash = content_hash_hex(package_json.as_bytes());
    let package = ContentPackage::from_json_bytes(package_json.as_bytes())?;
    package.validate()?;
    check_engine_compat(&package.manifest, &tooling_host_versions()).map_err(|e| e.to_string())?;
    let manifest = &package.manifest;
    if !valid_identity_segment(&manifest.namespace)
        || !valid_identity_segment(&manifest.package_id)
        || !valid_identity_segment(&manifest.version)
    {
        return Err("无效的安装包身份".into());
    }
    let mut declared = BTreeMap::new();
    if let Some(asset_manifest) = &package.asset_manifest {
        for entry in &asset_manifest.entries {
            let relative = package
                .asset_paths
                .get(&entry.asset_id)
                .ok_or_else(|| "资产路径缺失".to_string())?;
            if !valid_relative_asset_path(relative) {
                return Err(format!("原局包资产路径越界: {relative}"));
            }
            if entry.source_hash == [0u8; 32] {
                return Err(format!("原局包资产 hash 为零: {relative}"));
            }
            if let Some((hash, kind)) = declared.insert(relative, (entry.source_hash, entry.kind)) {
                if hash != entry.source_hash || kind != entry.kind {
                    return Err(format!("原局包资产路径冲突: {relative}"));
                }
            }
            let bytes = assets
                .get(relative)
                .ok_or_else(|| format!("原局包资产缺失: {relative}"))?;
            if blake3::hash(bytes).as_bytes() != &entry.source_hash {
                return Err(format!("原局包资产 hash 不匹配: {relative}"));
            }
        }
    }
    for key in assets.keys() {
        if !valid_relative_asset_path(key) {
            return Err(format!("原局包资产路径越界: {key}"));
        }
    }
    if declared.len() != assets.len() {
        return Err("原局包含未声明资产".into());
    }
    let logic_hash = blake3::Hash::from(manifest.logic_hash).to_hex().to_string();
    let binding = LockedPackage {
        namespace: manifest.namespace.clone(),
        package_id: manifest.package_id.clone(),
        version: manifest.version.clone(),
        logic_hash,
        content_hash,
    };
    Ok((
        FrozenPackage {
            binding,
            package_json,
            assets,
        },
        package,
    ))
}

/// Shared canonical verification point (`R05 v4 §2` / `v5 §2`).
///
/// Verifies:
/// 1. Slot uniqueness and valid identity fields on `lock` and `copies`.
/// 2. `ContentLock::from_packages(lock.packages.clone()) == *lock` (canonical order + `lock_hash`).
/// 3. Exact 1-to-1 binding equality between `lock.packages` and `copies`.
/// 4. Raw `package_json` byte hash (`content_hash`), `ContentPackage::validate()`, ABI compat, and `logic_hash`.
/// 5. Full dependency closure across the verified package set.
/// 6. Declared asset path safety, completeness, and `source_hash` equality.
pub fn verify_locked_content(
    lock: &ContentLock,
    copies: &[FrozenPackage],
) -> Result<VerifiedLockedContent, String> {
    if lock.lock_hash.is_empty() {
        return Err("内容锁 hash 不能为空".into());
    }
    let mut lock_slots = BTreeSet::new();
    for pkg in &lock.packages {
        if !valid_identity_segment(&pkg.namespace)
            || !valid_identity_segment(&pkg.package_id)
            || !valid_identity_segment(&pkg.version)
            || pkg.logic_hash.is_empty()
            || pkg.content_hash.is_empty()
        {
            return Err("内容锁包含无效的包身份或空 hash".into());
        }
        if !lock_slots.insert((pkg.namespace.clone(), pkg.package_id.clone())) {
            return Err(format!(
                "内容锁槽位重复: {}/{}",
                pkg.namespace, pkg.package_id
            ));
        }
    }
    if &ContentLock::from_packages(lock.packages.clone()) != lock {
        return Err("内容锁不满足规范排序或聚合 lock_hash 不匹配".into());
    }
    if copies.len() != lock.packages.len() {
        return Err("内容锁与原局包副本数量不一致".into());
    }
    let mut copy_slots = BTreeSet::new();
    for copy in copies {
        if !copy_slots.insert((
            copy.binding.namespace.clone(),
            copy.binding.package_id.clone(),
        )) {
            return Err(format!(
                "原局包槽位重复: {}/{}",
                copy.binding.namespace, copy.binding.package_id
            ));
        }
    }
    for (locked_pkg, copy) in lock.packages.iter().zip(copies.iter()) {
        if locked_pkg != &copy.binding {
            return Err(format!(
                "内容锁与原局包 binding 或规范顺序不匹配: {}/{}@{}",
                copy.binding.namespace, copy.binding.package_id, copy.binding.version
            ));
        }
    }
    let mut parsed_by_slot = BTreeMap::new();
    let mut sorted_packages = Vec::with_capacity(copies.len());
    for copy in copies {
        let pkg = copy.validate_and_parse()?;
        parsed_by_slot.insert(
            (
                pkg.manifest.namespace.clone(),
                pkg.manifest.package_id.clone(),
            ),
            (pkg.manifest.version.clone(), pkg.manifest.logic_hash),
        );
        sorted_packages.push(pkg);
    }
    for pkg in &sorted_packages {
        let own = (
            pkg.manifest.namespace.clone(),
            pkg.manifest.package_id.clone(),
        );
        let mut seen_deps = BTreeSet::new();
        for dep in &pkg.manifest.dependencies {
            if !valid_identity_segment(&dep.namespace)
                || !valid_identity_segment(&dep.package_id)
                || !valid_identity_segment(&dep.version)
            {
                return Err(format!(
                    "原局包依赖身份字段无效: {}/{}",
                    own.0, own.1
                ));
            }
            let dep_key = (dep.namespace.clone(), dep.package_id.clone());
            if dep_key == own || !seen_deps.insert(dep_key.clone()) {
                return Err(format!(
                    "原局包依赖重复或指向自身: {}/{}",
                    dep_key.0, dep_key.1
                ));
            }
            let Some((target_ver, target_hash)) = parsed_by_slot.get(&dep_key) else {
                return Err(format!(
                    "原局包缺少依赖: {}/{}@{}",
                    dep_key.0, dep_key.1, dep.version
                ));
            };
            if target_ver != &dep.version
                || (dep.logic_hash != [0u8; 32] && target_hash != &dep.logic_hash)
            {
                return Err(format!(
                    "原局包依赖版本或 logic_hash 不匹配: {}/{}@{}",
                    dep_key.0, dep_key.1, dep.version
                ));
            }
        }
    }
    Ok(VerifiedLockedContent {
        lock: lock.clone(),
        copies: copies.to_vec(),
        packages: sorted_packages,
    })
}

/// Build and verify canonical locked content from selected [`FrozenPackage`] copies (`R05 v4 §2`).
pub fn prepare_locked_content(mut copies: Vec<FrozenPackage>) -> Result<VerifiedLockedContent, String> {
    copies.sort_by(|a, b| {
        (
            &a.binding.namespace,
            &a.binding.package_id,
            &a.binding.version,
        )
            .cmp(&(
                &b.binding.namespace,
                &b.binding.package_id,
                &b.binding.version,
            ))
    });
    let bindings = copies.iter().map(|c| c.binding.clone()).collect();
    let lock = ContentLock::from_packages(bindings);
    verify_locked_content(&lock, &copies)
}

impl VerifiedLockedContent {
    /// Verified empty content lock (`ContentLock::empty()`, `frozen_packages: []`) for standard matches.
    pub fn empty() -> Self {
        verify_locked_content(&ContentLock::empty(), &[]).expect("empty ContentLock is valid")
    }

    /// True if no extension packages are present (`ContentLock::empty()`).
    pub fn is_empty(&self) -> bool {
        self.copies.is_empty()
    }

    /// Read-only reference to the canonical [`ContentLock`].
    pub fn lock(&self) -> &ContentLock {
        &self.lock
    }

    /// Read-only slice of verified [`FrozenPackage`] copies in canonical slot order.
    pub fn copies(&self) -> &[FrozenPackage] {
        &self.copies
    }

    /// Read-only slice of verified [`ContentPackage`]s in canonical slot order.
    pub fn packages(&self) -> &[ContentPackage] {
        &self.packages
    }

    /// Export as a serializable [`LockedContentEnvelope`].
    pub fn to_envelope(&self) -> LockedContentEnvelope {
        LockedContentEnvelope {
            canonical_locked_content: self.lock.clone(),
            frozen_packages: self.copies.clone(),
        }
    }

    /// Serialize the verified content envelope to JSON.
    pub fn to_envelope_json(&self) -> String {
        serde_json::to_string(&self.to_envelope()).expect("LockedContentEnvelope serializes")
    }

    /// Strictly parse and verify a `lockedContentJson` envelope (`R05 v4 §4`).
    ///
    /// Rejects empty strings, `"null"`, `"{}"`, missing fields, null fields, or unverified hashes.
    pub fn from_envelope_json(json: &str) -> Result<Self, String> {
        if json.trim().is_empty() {
            return Err("lockedContentJson 不能为空".into());
        }
        let val: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("lockedContentJson 非法 JSON: {e}"))?;
        let Some(obj) = val.as_object() else {
            return Err("lockedContentJson 必须是包含 canonical_locked_content 与 frozen_packages 的对象".into());
        };
        if !obj.contains_key("canonical_locked_content") || !obj.contains_key("frozen_packages") {
            return Err("lockedContentJson 缺少 canonical_locked_content 或 frozen_packages 字段".into());
        }
        let env: LockedContentEnvelope = serde_json::from_value(val)
            .map_err(|e| format!("lockedContentJson 字段类型无效: {e}"))?;
        verify_locked_content(&env.canonical_locked_content, &env.frozen_packages)
    }
}

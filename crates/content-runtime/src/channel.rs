//! Content update channel: catalog + planned atomic apply.
//!
//! A channel describes *available* package versions (local directory or mirrored
//! remote cache). Apply never mutates live match content — host must not freeze.

use crate::sign::PackageSignature;
use crate::store::{InstallOptions, PackageStore};
use crate::RuntimeError;
use content_schema::PackageDependency;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// One package entry in a channel catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelPackage {
    /// Namespace.
    pub namespace: String,
    /// Package id.
    pub package_id: String,
    /// Version.
    pub version: String,
    /// Relative path to package.json within channel root (or absolute).
    pub path: String,
    /// Optional relative path to signature.json.
    pub signature_path: Option<String>,
    /// Content hash hex (integrity tip).
    pub content_hash: String,
    /// Min engine API.
    pub min_engine_api: u32,
    /// Max engine API.
    pub max_engine_api: u32,
}

/// Channel catalog (immutable artifact when published).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelManifest {
    /// Channel id (e.g. official-stable).
    pub channel_id: String,
    /// Catalog schema version.
    pub schema_version: u32,
    /// Human channel version / generation.
    pub generation: String,
    /// Packages offered.
    pub packages: Vec<ChannelPackage>,
    /// Optional catalog signature over canonical body (not required for local).
    pub catalog_hash: String,
}

impl ChannelManifest {
    /// Schema constant.
    pub const SCHEMA: u32 = 1;

    /// Build catalog hash from packages (semantic).
    pub fn compute_catalog_hash(&self) -> String {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/channel-catalog/v1");
        h.update(self.channel_id.as_bytes());
        h.update(self.generation.as_bytes());
        for p in &self.packages {
            h.update(p.namespace.as_bytes());
            h.update(p.package_id.as_bytes());
            h.update(p.version.as_bytes());
            h.update(p.content_hash.as_bytes());
            h.update(p.path.as_bytes());
        }
        h.finalize().to_hex().to_string()
    }

    /// Recompute and set catalog_hash.
    pub fn seal(&mut self) {
        self.catalog_hash = self.compute_catalog_hash();
    }

    /// Load from JSON file.
    pub fn load(path: &Path) -> Result<Self, RuntimeError> {
        let data = fs::read_to_string(path)?;
        let m: Self = serde_json::from_str(&data)?;
        if m.schema_version != Self::SCHEMA {
            return Err(RuntimeError::Invalid(format!(
                "channel schema {} unsupported",
                m.schema_version
            )));
        }
        Ok(m)
    }

    /// Save pretty JSON.
    pub fn save(&self, path: &Path) -> Result<(), RuntimeError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

/// Planned install action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateAction {
    /// Slot key namespace/package_id.
    pub slot: String,
    /// From version if any.
    pub from_version: Option<String>,
    /// To version.
    pub to_version: String,
    /// Content hash of target.
    pub content_hash: String,
}

/// Diff between active set and channel desired versions (latest per slot in channel).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePlan {
    /// Channel id.
    pub channel_id: String,
    /// Actions to apply.
    pub actions: Vec<UpdateAction>,
    /// Already up to date slots.
    pub up_to_date: Vec<String>,
}

/// Resolve plan: for each package_id in channel, pick **last** listed version
/// (catalog author orders; typically newest last).
pub fn plan_updates(
    store: &PackageStore,
    channel: &ChannelManifest,
) -> Result<UpdatePlan, RuntimeError> {
    let active = store.read_active()?;
    // latest per slot as last occurrence
    let mut latest: BTreeMap<String, &ChannelPackage> = BTreeMap::new();
    for p in &channel.packages {
        let slot = format!("{}/{}", p.namespace, p.package_id);
        latest.insert(slot, p);
    }
    let mut actions = Vec::new();
    let mut up_to_date = Vec::new();
    for (slot, pkg) in latest {
        let current = active.packages.get(&slot).cloned();
        if current.as_deref() == Some(pkg.version.as_str()) {
            up_to_date.push(slot);
        } else {
            actions.push(UpdateAction {
                slot: slot.clone(),
                from_version: current,
                to_version: pkg.version.clone(),
                content_hash: pkg.content_hash.clone(),
            });
        }
    }
    Ok(UpdatePlan {
        channel_id: channel.channel_id.clone(),
        actions,
        up_to_date,
    })
}

/// Apply update plan from channel root (reads package files, installs+activates).
pub fn apply_updates(
    store: &PackageStore,
    channel_root: &Path,
    channel: &ChannelManifest,
    plan: &UpdatePlan,
    opts: &InstallOptions,
) -> Result<Vec<String>, RuntimeError> {
    if plan.channel_id != channel.channel_id {
        return Err(RuntimeError::Invalid("plan/channel id mismatch".into()));
    }
    let mut installed = Vec::new();
    let mut selections = Vec::new();
    for action in &plan.actions {
        let pkg_entry = channel
            .packages
            .iter()
            .find(|p| {
                format!("{}/{}", p.namespace, p.package_id) == action.slot
                    && p.version == action.to_version
            })
            .ok_or_else(|| RuntimeError::NotFound(action.slot.clone()))?;

        let pkg_path = resolve_path(channel_root, &pkg_entry.path);
        let bytes = fs::read(&pkg_path)?;
        let hash = crate::sign::content_hash_hex(&bytes);
        if hash != pkg_entry.content_hash {
            return Err(RuntimeError::Invalid(format!(
                "content hash mismatch for {}: got {hash} want {}",
                pkg_entry.path, pkg_entry.content_hash
            )));
        }
        let signature = match &pkg_entry.signature_path {
            Some(sp) => {
                let spath = resolve_path(channel_root, sp);
                let t = fs::read_to_string(spath)?;
                Some(serde_json::from_str::<PackageSignature>(&t)?)
            }
            None => None,
        };
        let mut install_opts = opts.clone();
        install_opts.activate = false;
        let source_root = pkg_path
            .parent()
            .ok_or_else(|| RuntimeError::Invalid("package path has no parent".into()))?;
        store.install_with_assets(&bytes, signature.as_ref(), &install_opts, source_root)?;
        selections.push(PackageDependency {
            namespace: pkg_entry.namespace.clone(),
            package_id: pkg_entry.package_id.clone(),
            version: pkg_entry.version.clone(),
            logic_hash: [0u8; 32],
        });
        installed.push(format!(
            "{}/{}@{}",
            pkg_entry.namespace, pkg_entry.package_id, pkg_entry.version
        ));
    }
    if opts.activate && !selections.is_empty() {
        store.activate_packages(&selections)?;
    }
    Ok(installed)
}

/// Build a channel entry from on-disk package bytes.
pub fn entry_from_package_file(
    namespace: &str,
    package_id: &str,
    version: &str,
    rel_path: &str,
    package_bytes: &[u8],
    signature_rel: Option<&str>,
    min_engine_api: u32,
    max_engine_api: u32,
) -> ChannelPackage {
    ChannelPackage {
        namespace: namespace.into(),
        package_id: package_id.into(),
        version: version.into(),
        path: rel_path.into(),
        signature_path: signature_rel.map(|s| s.to_string()),
        content_hash: crate::sign::content_hash_hex(package_bytes),
        min_engine_api,
        max_engine_api,
    }
}

fn resolve_path(root: &Path, p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sign::sign_package;
    use crate::InstallOptions;
    use content_schema::{ContentPackage, PackageManifest};
    use rule_vm::BytecodeModule;

    fn pkg_bytes(version: &str) -> Vec<u8> {
        let mut pkg = ContentPackage {
            manifest: PackageManifest::new("core", "std", version),
            modules: vec![BytecodeModule {
                name: "m".into(),
                code: vec![1, version.as_bytes()[0]],
                local_count: 0,
                max_stack: 8,
                entry_pc: 0,
                logic_hash: [0u8; 32],
                caps_mask: 0,
            }],
            skill_index: vec![],
            character_index: vec![],
            skin_index: Default::default(),
            card_index: vec![],
            trigger_index: vec![],
            modifier_index: vec![],
            active_index: vec![],
            view_as_index: vec![],
            translations: Default::default(),
            asset_manifest: None,
            asset_paths: Default::default(),
            compiler_version: "t".into(),
        };
        pkg.modules[0].logic_hash = *blake3::hash(&pkg.modules[0].code).as_bytes();
        pkg.manifest.logic_hash = pkg.compute_logic_hash();
        pkg.to_json_bytes().unwrap()
    }

    #[test]
    fn plan_and_apply_channel() {
        let dir = tempfile::tempdir().unwrap();
        let channel_root = dir.path().join("channel");
        fs::create_dir_all(channel_root.join("pkgs")).unwrap();
        let store = PackageStore::open(dir.path().join("store")).unwrap();

        let b1 = pkg_bytes("1.0.0");
        let b2 = pkg_bytes("1.1.0");
        fs::write(channel_root.join("pkgs/std-1.0.0.json"), &b1).unwrap();
        fs::write(channel_root.join("pkgs/std-1.1.0.json"), &b2).unwrap();
        let s1 = sign_package(&b1, "k", "dev").unwrap();
        let s2 = sign_package(&b2, "k", "dev").unwrap();
        fs::write(
            channel_root.join("pkgs/std-1.0.0.sig.json"),
            serde_json::to_string(&s1).unwrap(),
        )
        .unwrap();
        fs::write(
            channel_root.join("pkgs/std-1.1.0.sig.json"),
            serde_json::to_string(&s2).unwrap(),
        )
        .unwrap();

        let mut channel = ChannelManifest {
            channel_id: "official".into(),
            schema_version: ChannelManifest::SCHEMA,
            generation: "2026.1".into(),
            packages: vec![
                entry_from_package_file(
                    "core",
                    "std",
                    "1.0.0",
                    "pkgs/std-1.0.0.json",
                    &b1,
                    Some("pkgs/std-1.0.0.sig.json"),
                    1,
                    1,
                ),
                entry_from_package_file(
                    "core",
                    "std",
                    "1.1.0",
                    "pkgs/std-1.1.0.json",
                    &b2,
                    Some("pkgs/std-1.1.0.sig.json"),
                    1,
                    1,
                ),
            ],
            catalog_hash: String::new(),
        };
        channel.seal();
        channel.save(&channel_root.join("channel.json")).unwrap();

        let opts = InstallOptions {
            activate: true,
            allow_unsigned: false,
            verify_key: Some("k".into()),
            host: crate::tooling_host_versions(),
        };

        // first apply both planned as latest only (1.1.0)
        let plan = plan_updates(&store, &channel).unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].to_version, "1.1.0");
        apply_updates(&store, &channel_root, &channel, &plan, &opts).unwrap();
        let active = store.read_active().unwrap();
        assert_eq!(
            active.packages.get("core/std").map(String::as_str),
            Some("1.1.0")
        );

        let plan2 = plan_updates(&store, &channel).unwrap();
        assert!(plan2.actions.is_empty());
        assert_eq!(plan2.up_to_date, vec!["core/std".to_string()]);
    }
}

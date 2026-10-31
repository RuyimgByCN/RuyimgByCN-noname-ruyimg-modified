//! On-disk package store with atomic active-set updates.

use crate::atomic_fs::{atomic_write_json, remove_dir_all_if_exists};
use crate::catalog::ActiveCatalog;
use crate::compat::{check_engine_compat, tooling_host_versions};
use crate::extension::{checked_child, validate_relative};
use crate::lock::{ActiveSet, ContentLock, LockedPackage};
use crate::revoke::RevokeList;
use crate::sign::{content_hash_hex, verify_package, PackageSignature};
use crate::RuntimeError;
use content_schema::{ContentPackage, PackageDependency};
use engine_api::VersionBundle;
use resource_schema::AssetKind;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
mod ordering;
use ordering::*;

const MAX_HISTORY: usize = 16;

/// Installed package metadata beside the bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledMeta {
    /// Namespace.
    pub namespace: String,
    /// Package id.
    pub package_id: String,
    /// Version.
    pub version: String,
    /// Logic hash hex.
    pub logic_hash: String,
    /// Content file hash hex.
    pub content_hash: String,
    /// Whether signature was verified at install.
    pub signed: bool,
    /// Signature key id if signed.
    pub key_id: Option<String>,
    /// Canonical source extension directory for package-owned assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_root: Option<String>,
}

/// Options for installing a package.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    /// Activate after install (update active set).
    pub activate: bool,
    /// Allow packages without signature (dev only; marked unsigned).
    pub allow_unsigned: bool,
    /// Publisher key for verification (required unless allow_unsigned).
    pub verify_key: Option<String>,
    /// Host version axes for compat check.
    pub host: VersionBundle,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            activate: true,
            allow_unsigned: false,
            verify_key: None,
            host: tooling_host_versions(),
        }
    }
}

/// Result of a successful install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallResult {
    /// Meta of installed package.
    pub meta: InstalledMeta,
    /// Path to package.json in store.
    pub store_path: String,
    /// Content lock after install (if activated).
    pub content_lock: Option<ContentLock>,
}

/// Immutable store location and declared metadata for one active package asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveAssetInfo {
    /// Verified path inside the installed package version directory.
    pub path: PathBuf,
    /// BLAKE3 hash verified while the asset was staged.
    pub source_hash: [u8; 32],
    /// Declared asset kind.
    pub kind: AssetKind,
}

/// Content package store root.
#[derive(Debug, Clone)]
pub struct PackageStore {
    root: PathBuf,
}

impl PackageStore {
    /// Open or create a store at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, RuntimeError> {
        let root = root.into();
        fs::create_dir_all(root.join("packages"))?;
        fs::create_dir_all(root.join("staging"))?;
        fs::create_dir_all(root.join("state"))?;
        let store = Self { root };
        if !store.active_path().exists() {
            store.write_active(&ActiveSet::default())?;
        }
        if !store.history_path().exists() {
            store.write_history(&Vec::new())?;
        }
        if !store.revoke_path().exists() {
            store.write_revoke(&RevokeList::default())?;
        }
        Ok(store)
    }

    /// Store root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn packages_dir(&self) -> PathBuf {
        self.root.join("packages")
    }
    fn staging_dir(&self) -> PathBuf {
        self.root.join("staging")
    }
    fn state_dir(&self) -> PathBuf {
        self.root.join("state")
    }
    fn active_path(&self) -> PathBuf {
        self.state_dir().join("active.json")
    }
    fn history_path(&self) -> PathBuf {
        self.state_dir().join("history.json")
    }
    fn revoke_path(&self) -> PathBuf {
        self.state_dir().join("revoke.json")
    }

    fn package_dir(&self, namespace: &str, package_id: &str, version: &str) -> PathBuf {
        self.packages_dir()
            .join(format!("{namespace}__{package_id}"))
            .join(version)
    }

    /// Read active set.
    pub fn read_active(&self) -> Result<ActiveSet, RuntimeError> {
        let data = fs::read_to_string(self.active_path())?;
        Ok(serde_json::from_str(&data)?)
    }

    fn write_active(&self, active: &ActiveSet) -> Result<(), RuntimeError> {
        atomic_write_json(&self.active_path(), active)?;
        Ok(())
    }

    fn read_history(&self) -> Result<Vec<ActiveSet>, RuntimeError> {
        let data = fs::read_to_string(self.history_path())?;
        Ok(serde_json::from_str(&data)?)
    }

    fn write_history(&self, history: &Vec<ActiveSet>) -> Result<(), RuntimeError> {
        atomic_write_json(&self.history_path(), history)?;
        Ok(())
    }

    /// Read revoke list.
    pub fn read_revoke(&self) -> Result<RevokeList, RuntimeError> {
        let data = fs::read_to_string(self.revoke_path())?;
        Ok(serde_json::from_str(&data)?)
    }

    fn write_revoke(&self, list: &RevokeList) -> Result<(), RuntimeError> {
        atomic_write_json(&self.revoke_path(), list)?;
        Ok(())
    }

    /// Install package bytes (+ optional signature JSON).
    ///
    /// Uses a staging directory; only commits to `packages/` after full verification.
    /// Active set is updated atomically (tmp + rename). Interrupted installs leave only
    /// staging garbage (ignored) or a complete version dir never referenced by active.
    pub fn install(
        &self,
        package_bytes: &[u8],
        signature: Option<&PackageSignature>,
        opts: &InstallOptions,
    ) -> Result<InstallResult, RuntimeError> {
        self.install_inner(package_bytes, signature, opts, None)
    }

    /// Install package bytes and package-owned assets from `source_root`.
    pub fn install_with_assets(
        &self,
        package_bytes: &[u8],
        signature: Option<&PackageSignature>,
        opts: &InstallOptions,
        source_root: &Path,
    ) -> Result<InstallResult, RuntimeError> {
        let source_root = source_root.canonicalize()?;
        if !source_root.is_dir() {
            return Err(RuntimeError::Invalid(format!(
                "asset source root is not a directory: {}",
                source_root.display()
            )));
        }
        self.install_inner(package_bytes, signature, opts, Some(&source_root))
    }

    fn install_inner(
        &self,
        package_bytes: &[u8],
        signature: Option<&PackageSignature>,
        opts: &InstallOptions,
        source_root: Option<&Path>,
    ) -> Result<InstallResult, RuntimeError> {
        let pkg = ContentPackage::from_json_bytes(package_bytes).map_err(RuntimeError::Package)?;
        pkg.validate().map_err(RuntimeError::Package)?;
        let m = &pkg.manifest;
        check_engine_compat(m, &opts.host)?;

        let content_hash = content_hash_hex(package_bytes);
        let revoke = self.read_revoke()?;
        if revoke.is_hash_revoked(&content_hash) {
            return Err(RuntimeError::Revoked(format!(
                "content_hash {content_hash}"
            )));
        }
        if revoke.is_slot_revoked(&m.namespace, &m.package_id) {
            return Err(RuntimeError::Revoked(format!(
                "{}/{}",
                m.namespace, m.package_id
            )));
        }

        let mut signed = false;
        let mut key_id = None;
        match (signature, &opts.verify_key) {
            (Some(sig), Some(key)) => {
                let ok = verify_package(package_bytes, key, sig)?;
                if !ok {
                    return Err(RuntimeError::Signature("verification failed".into()));
                }
                if sig.content_hash != content_hash {
                    return Err(RuntimeError::Signature("content_hash mismatch".into()));
                }
                signed = true;
                key_id = Some(sig.key_id.clone());
            }
            (Some(_), None) => {
                return Err(RuntimeError::Invalid(
                    "signature provided but no verify_key".into(),
                ));
            }
            (None, _) => {
                if !opts.allow_unsigned {
                    return Err(RuntimeError::UnsignedNotAllowed);
                }
            }
        }

        let logic_hash = hex_bytes(&pkg.compute_logic_hash());
        let mut meta = InstalledMeta {
            namespace: m.namespace.clone(),
            package_id: m.package_id.clone(),
            version: m.version.clone(),
            logic_hash,
            content_hash: content_hash.clone(),
            signed,
            key_id,
            extension_root: source_root.map(|root| root.display().to_string()),
        };

        let stage_id = format!(
            "{}-{}",
            std::process::id(),
            &content_hash[..16.min(content_hash.len())]
        );
        let stage = self.staging_dir().join(&stage_id);
        remove_dir_all_if_exists(&stage)?;
        fs::create_dir_all(&stage)?;
        fs::write(stage.join("package.json"), package_bytes)?;
        if let Some(sig) = signature {
            atomic_write_json(&stage.join("signature.json"), sig)?;
        }
        atomic_write_json(&stage.join("meta.json"), &meta)?;
        if let Err(error) = self.stage_assets(&pkg, source_root, &stage) {
            remove_dir_all_if_exists(&stage)?;
            return Err(error);
        }

        let dest = self.package_dir(&m.namespace, &m.package_id, &m.version);
        if dest.exists() {
            let existing = fs::read(dest.join("package.json"))?;
            if existing.as_slice() != package_bytes {
                remove_dir_all_if_exists(&stage)?;
                return Err(RuntimeError::Invalid(format!(
                    "version {} already installed with different content",
                    m.version
                )));
            }
            let existing_meta: InstalledMeta =
                serde_json::from_str(&fs::read_to_string(dest.join("meta.json"))?)?;
            if source_root.is_some() {
                if let Err(error) = self.verify_installed_asset_files(&pkg, &dest) {
                    remove_dir_all_if_exists(&stage)?;
                    return Err(error);
                }
            }
            if meta.extension_root.is_some() && meta.extension_root != existing_meta.extension_root
            {
                remove_dir_all_if_exists(&stage)?;
                return Err(RuntimeError::Invalid(format!(
                    "version {} already installed from a different extension root",
                    m.version
                )));
            }
            meta = existing_meta;
        } else {
            fs::create_dir_all(dest.parent().unwrap())?;
            fs::rename(&stage, &dest)?;
        }
        remove_dir_all_if_exists(&stage)?;

        let store_path = dest.join("package.json").display().to_string();
        let content_lock = if opts.activate {
            self.activate_version(&m.namespace, &m.package_id, &m.version)?;
            Some(self.current_lock()?)
        } else {
            None
        };

        Ok(InstallResult {
            meta,
            store_path,
            content_lock,
        })
    }

    fn stage_assets(
        &self,
        package: &ContentPackage,
        source_root: Option<&Path>,
        stage: &Path,
    ) -> Result<(), RuntimeError> {
        let files = declared_asset_files(package)?;
        if files.is_empty() {
            return Ok(());
        }
        let source_root = source_root.ok_or_else(|| {
            RuntimeError::Invalid(
                "package declares assets but no asset source root was provided".into(),
            )
        })?;
        for (relative, (source_hash, _)) in files {
            let source = checked_child(source_root, &relative)?;
            let target = stage.join(&relative);
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(source, &target)?;
            let actual = hash_file(&target)?;
            if actual != source_hash {
                return Err(RuntimeError::Invalid(format!(
                    "asset hash mismatch {relative}"
                )));
            }
        }
        Ok(())
    }

    fn verify_installed_asset_files(
        &self,
        package: &ContentPackage,
        dest: &Path,
    ) -> Result<(), RuntimeError> {
        for relative in declared_asset_files(package)?.keys() {
            if !dest.join(relative).is_file() {
                return Err(RuntimeError::Invalid(format!(
                    "installed version is missing asset {relative}; reinstall with a new version"
                )));
            }
        }
        Ok(())
    }

    /// Activate a specific installed version (atomic active + history push).
    pub fn activate_version(
        &self,
        namespace: &str,
        package_id: &str,
        version: &str,
    ) -> Result<(), RuntimeError> {
        self.activate_packages(&[PackageDependency {
            namespace: namespace.into(),
            package_id: package_id.into(),
            version: version.into(),
            logic_hash: [0u8; 32],
        }])
    }

    /// Atomically activate package versions after validating the complete dependency graph.
    pub fn activate_packages(&self, selections: &[PackageDependency]) -> Result<(), RuntimeError> {
        let active = self.read_active()?;
        let mut next = active.clone();
        self.apply_selections(&mut next, selections)?;
        self.packages_for_active(&next)?;
        if next == active {
            return Ok(());
        }
        let mut history = self.read_history()?;
        history.push(active);
        if history.len() > MAX_HISTORY {
            let drop_n = history.len() - MAX_HISTORY;
            history.drain(0..drop_n);
        }
        self.write_history(&history)?;
        self.write_active(&next)?;
        Ok(())
    }

    pub(crate) fn reconcile_extension_set(
        &self,
        extension_parent: &Path,
        selections: &[PackageDependency],
    ) -> Result<(), RuntimeError> {
        let active = self.read_active()?;
        let mut next = active.clone();
        for (slot, version) in &active.packages {
            let (namespace, package_id) = split_slot(slot)?;
            let (_, meta) = self.load_installed(namespace, package_id, version)?;
            let managed = meta
                .extension_root
                .as_deref()
                .is_some_and(|root| Path::new(root).parent() == Some(extension_parent));
            if managed {
                next.packages.remove(slot);
            }
        }
        self.apply_selections(&mut next, selections)?;
        self.prune_broken_dependents(&mut next)?;
        self.packages_for_active(&next)?;
        if next == active {
            return Ok(());
        }
        let mut history = self.read_history()?;
        history.push(active);
        if history.len() > MAX_HISTORY {
            let drop_n = history.len() - MAX_HISTORY;
            history.drain(0..drop_n);
        }
        self.write_history(&history)?;
        self.write_active(&next)?;
        Ok(())
    }

    /// Roll back active set to previous history entry.
    pub fn rollback(&self) -> Result<ActiveSet, RuntimeError> {
        let mut history = self.read_history()?;
        let prev = history
            .pop()
            .ok_or_else(|| RuntimeError::NotFound("no history to rollback".into()))?;
        self.packages_for_active(&prev)?;
        self.write_history(&history)?;
        self.write_active(&prev)?;
        Ok(prev)
    }

    /// Deactivate a package slot without deleting installed versions.
    ///
    /// Used for disable/enable: later `activate_version` can re-enable it. Fails
    /// before mutation when another active package still depends on the slot.
    pub fn deactivate_slot(&self, namespace: &str, package_id: &str) -> Result<(), RuntimeError> {
        let active = self.read_active()?;
        let key = ActiveSet::slot_key(namespace, package_id);
        if !active.packages.contains_key(&key) {
            return Ok(());
        }
        let mut next = active.clone();
        next.packages.remove(&key);
        self.packages_for_active(&next)?;
        let mut history = self.read_history()?;
        history.push(active);
        if history.len() > MAX_HISTORY {
            history.remove(0);
        }
        self.write_active(&next)?;
        self.write_history(&history)?;
        Ok(())
    }

    /// Physically uninstall every installed version of a package slot.
    ///
    /// Active references are removed first (and rollback history is cleared so a
    /// later rollback cannot resurrect a deleted directory). Dependents that still
    /// require the slot make this fail before any mutation. Removing the on-disk
    /// versions happens after the active set is durable; a crash between the two
    /// leaves only harmless orphan directories.
    pub fn uninstall_slot(&self, namespace: &str, package_id: &str) -> Result<(), RuntimeError> {
        let mut active = self.read_active()?;
        let key = ActiveSet::slot_key(namespace, package_id);
        let was_active = active.packages.remove(&key).is_some();
        if was_active {
            self.packages_for_active(&active)?;
            self.write_active(&active)?;
            self.write_history(&Vec::new())?;
        }
        let dir = self
            .packages_dir()
            .join(format!("{namespace}__{package_id}"));
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    /// Revoke by content hash and optionally deactivate matching actives.
    pub fn revoke_content_hash(
        &self,
        content_hash: &str,
        reason: &str,
        deactivate: bool,
    ) -> Result<(), RuntimeError> {
        let mut list = self.read_revoke()?;
        list.revoke_hash(content_hash.to_string(), reason);
        self.write_revoke(&list)?;
        if deactivate {
            self.deactivate_matching_hash(content_hash)?;
        }
        Ok(())
    }

    /// Revoke package slot (all versions).
    pub fn revoke_slot(
        &self,
        namespace: &str,
        package_id: &str,
        reason: &str,
        deactivate: bool,
    ) -> Result<(), RuntimeError> {
        let mut list = self.read_revoke()?;
        list.revoke_slot(namespace, package_id, reason);
        self.write_revoke(&list)?;
        if deactivate {
            let active = self.read_active()?;
            let key = ActiveSet::slot_key(namespace, package_id);
            if active.packages.contains_key(&key) {
                let mut history = self.read_history()?;
                history.push(active.clone());
                if history.len() > MAX_HISTORY {
                    let drop_n = history.len() - MAX_HISTORY;
                    history.drain(0..drop_n);
                }
                let mut next = active;
                next.packages.remove(&key);
                self.prune_broken_dependents(&mut next)?;
                self.packages_for_active(&next)?;
                self.write_history(&history)?;
                self.write_active(&next)?;
            }
        }
        Ok(())
    }

    fn deactivate_matching_hash(&self, content_hash: &str) -> Result<(), RuntimeError> {
        let active = self.read_active()?;
        let mut changed = false;
        let mut next = active.clone();
        for (slot, version) in &active.packages {
            let (ns, id) = split_slot(slot)?;
            let dest = self.package_dir(ns, id, version);
            if dest.join("meta.json").exists() {
                let meta: InstalledMeta =
                    serde_json::from_str(&fs::read_to_string(dest.join("meta.json"))?)?;
                if meta.content_hash == content_hash {
                    next.packages.remove(slot);
                    changed = true;
                }
            }
        }
        if changed {
            self.prune_broken_dependents(&mut next)?;
            self.packages_for_active(&next)?;
            let mut history = self.read_history()?;
            history.push(active);
            if history.len() > MAX_HISTORY {
                let drop_n = history.len() - MAX_HISTORY;
                history.drain(0..drop_n);
            }
            self.write_history(&history)?;
            self.write_active(&next)?;
        }
        Ok(())
    }

    /// Build content lock from current active set.
    pub fn current_lock(&self) -> Result<ContentLock, RuntimeError> {
        let active = self.read_active()?;
        self.packages_for_active(&active)?;
        let mut packages = Vec::new();
        for (slot, version) in &active.packages {
            let (ns, id) = split_slot(slot)?;
            let dest = self.package_dir(ns, id, version);
            let meta: InstalledMeta =
                serde_json::from_str(&fs::read_to_string(dest.join("meta.json"))?)?;
            packages.push(LockedPackage {
                namespace: meta.namespace,
                package_id: meta.package_id,
                version: meta.version,
                logic_hash: meta.logic_hash,
                content_hash: meta.content_hash,
            });
        }
        Ok(ContentLock::from_packages(packages))
    }

    /// List installed versions under packages/.
    pub fn list_installed(&self) -> Result<Vec<InstalledMeta>, RuntimeError> {
        let mut out = Vec::new();
        let pkgs = self.packages_dir();
        if !pkgs.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(pkgs)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            for ver in fs::read_dir(entry.path())? {
                let ver = ver?;
                let meta_path = ver.path().join("meta.json");
                if meta_path.exists() {
                    let meta: InstalledMeta =
                        serde_json::from_str(&fs::read_to_string(meta_path)?)?;
                    out.push(meta);
                }
            }
        }
        out.sort_by(|a, b| {
            (&a.namespace, &a.package_id, &a.version).cmp(&(
                &b.namespace,
                &b.package_id,
                &b.version,
            ))
        });
        Ok(out)
    }

    /// Sweep incomplete staging directories (safe after crash).
    pub fn gc_staging(&self) -> Result<usize, RuntimeError> {
        let mut n = 0;
        let stage = self.staging_dir();
        if !stage.exists() {
            return Ok(0);
        }
        for entry in fs::read_dir(stage)? {
            let entry = entry?;
            remove_dir_all_if_exists(&entry.path())?;
            n += 1;
        }
        Ok(n)
    }

    /// Load active package JSON bytes for a slot.
    pub fn load_active_package_bytes(
        &self,
        namespace: &str,
        package_id: &str,
    ) -> Result<Vec<u8>, RuntimeError> {
        let active = self.read_active()?;
        let key = ActiveSet::slot_key(namespace, package_id);
        let version = active
            .packages
            .get(&key)
            .ok_or_else(|| RuntimeError::NotFound(key.clone()))?;
        let path = self
            .package_dir(namespace, package_id, version)
            .join("package.json");
        Ok(fs::read(path)?)
    }

    /// Load and dependency-order every package pinned by a content lock.
    pub fn load_locked_packages(
        &self,
        lock: &ContentLock,
    ) -> Result<Vec<ContentPackage>, RuntimeError> {
        if &ContentLock::from_packages(lock.packages.clone()) != lock {
            return Err(RuntimeError::Invalid(
                "content lock is not canonical".into(),
            ));
        }
        let mut packages = Vec::with_capacity(lock.packages.len());
        for locked in &lock.packages {
            let (package, meta) =
                self.load_installed(&locked.namespace, &locked.package_id, &locked.version)?;
            if meta.logic_hash != locked.logic_hash || meta.content_hash != locked.content_hash {
                return Err(RuntimeError::Invalid(format!(
                    "content lock mismatch for {}/{}@{}",
                    locked.namespace, locked.package_id, locked.version
                )));
            }
            packages.push(package);
        }
        order_packages(packages)
    }

    /// Load all currently active packages using the canonical content lock.
    pub fn load_active_packages(&self) -> Result<(ContentLock, Vec<ContentPackage>), RuntimeError> {
        let lock = self.current_lock()?;
        let packages = self.load_locked_packages(&lock)?;
        Ok((lock, packages))
    }

    /// Load the active lock and aggregate its dependency-ordered presentation catalog.
    pub fn load_active_catalog(&self) -> Result<(ContentLock, ActiveCatalog), RuntimeError> {
        let (lock, packages) = self.load_active_packages()?;
        let catalog = ActiveCatalog::from_dependency_ordered_packages(&packages)?;
        Ok((lock, catalog))
    }

    fn packages_for_active(&self, active: &ActiveSet) -> Result<Vec<ContentPackage>, RuntimeError> {
        let mut packages = Vec::with_capacity(active.packages.len());
        for (slot, version) in &active.packages {
            let (namespace, package_id) = split_slot(slot)?;
            packages.push(self.load_installed(namespace, package_id, version)?.0);
        }
        let packages = order_packages(packages)?;
        ActiveCatalog::from_dependency_ordered_packages(&packages)?;
        Ok(packages)
    }

    fn apply_selections(
        &self,
        active: &mut ActiveSet,
        selections: &[PackageDependency],
    ) -> Result<(), RuntimeError> {
        let revoke = self.read_revoke()?;
        let mut selected_slots = BTreeSet::new();
        for selection in selections {
            let slot = ActiveSet::slot_key(&selection.namespace, &selection.package_id);
            if !selected_slots.insert(slot.clone()) {
                return Err(RuntimeError::Invalid(format!(
                    "duplicate activation selection {slot}"
                )));
            }
            let (_, meta) = self.load_installed(
                &selection.namespace,
                &selection.package_id,
                &selection.version,
            )?;
            if revoke.is_hash_revoked(&meta.content_hash)
                || revoke.is_slot_revoked(&selection.namespace, &selection.package_id)
            {
                return Err(RuntimeError::Revoked(format!(
                    "{}/{}@{}",
                    selection.namespace, selection.package_id, selection.version
                )));
            }
            if selection.logic_hash != [0u8; 32]
                && selection.logic_hash != hex_to_array(&meta.logic_hash)?
            {
                return Err(RuntimeError::Invalid(format!(
                    "logic hash mismatch for {}/{}@{}",
                    selection.namespace, selection.package_id, selection.version
                )));
            }
            active.packages.insert(slot, selection.version.clone());
        }
        Ok(())
    }

    fn prune_broken_dependents(&self, active: &mut ActiveSet) -> Result<(), RuntimeError> {
        loop {
            let snapshot = active.clone();
            let mut removed = false;
            for (slot, version) in &snapshot.packages {
                let (namespace, package_id) = split_slot(slot)?;
                let (package, _) = self.load_installed(namespace, package_id, version)?;
                let satisfied = package.manifest.dependencies.iter().all(|dependency| {
                    active
                        .packages
                        .get(&ActiveSet::slot_key(
                            &dependency.namespace,
                            &dependency.package_id,
                        ))
                        .is_some_and(|active_version| active_version == &dependency.version)
                });
                if !satisfied {
                    active.packages.remove(slot);
                    removed = true;
                }
            }
            if !removed {
                return Ok(());
            }
        }
    }

    fn load_installed(
        &self,
        namespace: &str,
        package_id: &str,
        version: &str,
    ) -> Result<(ContentPackage, InstalledMeta), RuntimeError> {
        let dest = self.package_dir(namespace, package_id, version);
        let bytes = fs::read(dest.join("package.json"))
            .map_err(|_| RuntimeError::NotFound(format!("{namespace}/{package_id}@{version}")))?;
        let package = ContentPackage::from_json_bytes(&bytes).map_err(RuntimeError::Package)?;
        package.validate().map_err(RuntimeError::Package)?;
        if package.manifest.namespace != namespace
            || package.manifest.package_id != package_id
            || package.manifest.version != version
        {
            return Err(RuntimeError::Invalid(format!(
                "installed package identity mismatch for {namespace}/{package_id}@{version}"
            )));
        }
        let meta: InstalledMeta =
            serde_json::from_str(&fs::read_to_string(dest.join("meta.json"))?)?;
        if meta.namespace != namespace
            || meta.package_id != package_id
            || meta.version != version
            || meta.logic_hash != hex_bytes(&package.manifest.logic_hash)
            || meta.content_hash != content_hash_hex(&bytes)
        {
            return Err(RuntimeError::Invalid(format!(
                "installed metadata mismatch for {namespace}/{package_id}@{version}"
            )));
        }
        Ok((package, meta))
    }

    /// Resolve one active package asset to its immutable store path.
    pub fn resolve_active_asset(
        &self,
        namespace: &str,
        package_id: &str,
        asset_id: &str,
    ) -> Result<PathBuf, RuntimeError> {
        Ok(self
            .resolve_active_asset_info(namespace, package_id, asset_id)?
            .path)
    }

    /// Resolve one active package asset and its install-time verified metadata.
    pub fn resolve_active_asset_info(
        &self,
        namespace: &str,
        package_id: &str,
        asset_id: &str,
    ) -> Result<ActiveAssetInfo, RuntimeError> {
        let active = self.read_active()?;
        let slot = ActiveSet::slot_key(namespace, package_id);
        let version = active
            .packages
            .get(&slot)
            .ok_or_else(|| RuntimeError::NotFound(slot.clone()))?;
        let (package, _) = self.load_installed(namespace, package_id, version)?;
        let root = self
            .package_dir(namespace, package_id, version)
            .canonicalize()?;
        installed_asset_info(&root, &package, asset_id)
    }

    /// Load every active package asset into a cacheable immutable-store index.
    pub fn load_active_asset_index(
        &self,
    ) -> Result<BTreeMap<(String, String, String), ActiveAssetInfo>, RuntimeError> {
        let active = self.read_active()?;
        let packages = self.packages_for_active(&active)?;
        let mut index = BTreeMap::new();
        for package in packages {
            let manifest = &package.manifest;
            let root = self
                .package_dir(&manifest.namespace, &manifest.package_id, &manifest.version)
                .canonicalize()?;
            if let Some(assets) = &package.asset_manifest {
                for entry in &assets.entries {
                    index.insert(
                        (
                            manifest.namespace.clone(),
                            manifest.package_id.clone(),
                            entry.asset_id.clone(),
                        ),
                        installed_asset_info(&root, &package, &entry.asset_id)?,
                    );
                }
            }
        }
        Ok(index)
    }
}

fn installed_asset_info(
    root: &Path,
    package: &ContentPackage,
    asset_id: &str,
) -> Result<ActiveAssetInfo, RuntimeError> {
    let relative = package
        .asset_paths
        .get(asset_id)
        .ok_or_else(|| RuntimeError::NotFound(asset_id.into()))?;
    validate_relative(relative)?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root) || !path.is_file() {
        return Err(RuntimeError::Invalid(format!(
            "asset escapes installed package: {relative}"
        )));
    }
    let entry = package
        .asset_manifest
        .as_ref()
        .and_then(|manifest| manifest.get(asset_id))
        .ok_or_else(|| RuntimeError::Invalid(format!("missing asset manifest {asset_id}")))?;
    if entry.source_hash == [0u8; 32] {
        return Err(RuntimeError::Invalid(format!(
            "active asset has zero source hash {asset_id}"
        )));
    }
    Ok(ActiveAssetInfo {
        path,
        source_hash: entry.source_hash,
        kind: entry.kind,
    })
}

fn declared_asset_files(
    package: &ContentPackage,
) -> Result<BTreeMap<String, ([u8; 32], AssetKind)>, RuntimeError> {
    let Some(manifest) = &package.asset_manifest else {
        return Ok(BTreeMap::new());
    };
    let mut files = BTreeMap::new();
    for entry in &manifest.entries {
        let relative = package.asset_paths.get(&entry.asset_id).ok_or_else(|| {
            RuntimeError::Invalid(format!("missing asset path {}", entry.asset_id))
        })?;
        validate_relative(relative)?;
        if ["package.json", "meta.json", "signature.json"]
            .iter()
            .any(|reserved| relative.eq_ignore_ascii_case(reserved))
        {
            return Err(RuntimeError::Invalid(format!(
                "asset path is reserved: {relative}"
            )));
        }
        if entry.source_hash == [0u8; 32] {
            return Err(RuntimeError::Invalid(format!(
                "asset source hash is zero: {}",
                entry.asset_id
            )));
        }
        if let Some((source_hash, kind)) = files.get(relative) {
            if source_hash != &entry.source_hash || kind != &entry.kind {
                return Err(RuntimeError::Invalid(format!(
                    "asset path {relative} has conflicting hash or kind"
                )));
            }
        } else {
            files.insert(relative.clone(), (entry.source_hash, entry.kind));
        }
    }
    Ok(files)
}

fn hash_file(path: &Path) -> Result<[u8; 32], RuntimeError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            return Ok(*hasher.finalize().as_bytes());
        }
        hasher.update(&buffer[..read]);
    }
}

fn split_slot(slot: &str) -> Result<(&str, &str), RuntimeError> {
    let (ns, id) = slot
        .split_once('/')
        .ok_or_else(|| RuntimeError::Invalid(format!("bad slot {slot}")))?;
    Ok((ns, id))
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_to_array(value: &str) -> Result<[u8; 32], RuntimeError> {
    if value.len() != 64 {
        return Err(RuntimeError::Invalid("invalid logic hash hex".into()));
    }
    let mut out = [0u8; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| RuntimeError::Invalid("invalid logic hash hex".into()))?;
    }
    Ok(out)
}

//! Content package runtime: install store, atomic activate/rollback, revoke, content lock.
//!
//! Architecture Phase 10 — does not execute package code; hosts load bytecode via engine.

#![deny(missing_docs)]

mod artifact;
mod atomic_fs;
mod catalog;
mod channel;
mod compat;
mod error;
mod extension;
mod lock;
mod revoke;
mod sign;
mod store;

pub use artifact::{ArtifactRecord, ArtifactRegistry};
pub use catalog::{
    ActiveAssetRef, ActiveCatalog, ActiveCatalogPage, ActiveCharacterEntry, ActiveSkinAliasEntry,
    ActiveSkinEntry, ActiveSkinPresentation, DEFAULT_CATALOG_PAGE_LIMIT, MAX_CATALOG_PAGE_LIMIT,
};
pub use channel::{
    apply_updates, entry_from_package_file, plan_updates, ChannelManifest, ChannelPackage,
    UpdateAction, UpdatePlan,
};
pub use compat::{check_engine_compat, tooling_host_versions};
pub use error::RuntimeError;
pub use extension::{
    discover_extensions, import_extensions, DiscoveredExtension, ExtensionDirectoryManifest,
    ExtensionImportReport,
};
pub use lock::{ActiveSet, ContentLock, LockedPackage};
pub use revoke::RevokeList;
pub use sign::{content_hash_hex, sign_package, verify_package, PackageSignature};
pub use store::{ActiveAssetInfo, InstallOptions, InstallResult, InstalledMeta, PackageStore};

/// Persist artifact registry under store `state/artifacts.json`.
pub fn save_artifacts(store: &PackageStore, reg: &ArtifactRegistry) -> Result<(), RuntimeError> {
    let path = store.root().join("state").join("artifacts.json");
    atomic_fs::atomic_write_json(&path, reg)?;
    Ok(())
}

/// Load artifact registry (empty if missing).
pub fn load_artifacts(store: &PackageStore) -> Result<ArtifactRegistry, RuntimeError> {
    let path = store.root().join("state").join("artifacts.json");
    if !path.exists() {
        return Ok(ArtifactRegistry::default());
    }
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

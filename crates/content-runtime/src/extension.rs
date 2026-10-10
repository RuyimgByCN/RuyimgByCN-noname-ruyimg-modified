//! Safe discovery and import of drop-in extension directories.

use crate::{
    ContentLock, InstallOptions, InstalledMeta, PackageSignature, PackageStore, RuntimeError,
};
use content_schema::PackageDependency;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Directory descriptor stored as `manifest.json` beside `package.npack.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDirectoryManifest {
    /// Descriptor schema version.
    #[serde(default = "manifest_schema")]
    pub schema_version: u32,
    /// Whether startup import should activate this extension.
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    /// Relative package JSON path.
    #[serde(default = "default_package_path")]
    pub package: String,
    /// Optional relative package signature JSON path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Default for ExtensionDirectoryManifest {
    fn default() -> Self {
        Self {
            schema_version: manifest_schema(),
            enabled: enabled_by_default(),
            package: default_package_path(),
            signature: None,
        }
    }
}

/// One discovered extension directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredExtension {
    /// Directory name.
    pub name: String,
    /// Canonical directory path.
    pub root: PathBuf,
    /// Parsed directory descriptor.
    pub manifest: ExtensionDirectoryManifest,
}

/// Result of importing every enabled extension in a directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionImportReport {
    /// Installed/updated package metadata.
    pub installed: Vec<InstalledMeta>,
    /// Disabled directory names that were intentionally skipped.
    pub skipped: Vec<String>,
    /// Canonical active content lock after the atomic activation.
    pub content_lock: ContentLock,
}

/// Discover direct child directories containing `manifest.json` or `package.npack.json`.
pub fn discover_extensions(root: &Path) -> Result<Vec<DiscoveredExtension>, RuntimeError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let canonical_root = root.canonicalize()?;
    let mut directories = Vec::new();
    for entry in fs::read_dir(&canonical_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let extension_root = entry.path().canonicalize()?;
        if !extension_root.starts_with(&canonical_root) {
            return Err(RuntimeError::Invalid(
                "extension directory escapes root".into(),
            ));
        }
        let manifest_path = extension_root.join("manifest.json");
        let package_path = extension_root.join(default_package_path());
        if !manifest_path.is_file() && !package_path.is_file() {
            continue;
        }
        let manifest = if manifest_path.is_file() {
            serde_json::from_str::<ExtensionDirectoryManifest>(&fs::read_to_string(manifest_path)?)?
        } else {
            ExtensionDirectoryManifest::default()
        };
        if manifest.schema_version != manifest_schema() {
            return Err(RuntimeError::Invalid(format!(
                "unsupported extension manifest schema {}",
                manifest.schema_version
            )));
        }
        validate_relative(&manifest.package)?;
        if let Some(signature) = &manifest.signature {
            validate_relative(signature)?;
        }
        directories.push(DiscoveredExtension {
            name: entry.file_name().to_string_lossy().into_owned(),
            root: extension_root,
            manifest,
        });
    }
    directories.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(directories)
}

/// Install all discovered packages, then activate them atomically in dependency order.
pub fn import_extensions(
    store: &PackageStore,
    root: &Path,
    opts: &InstallOptions,
) -> Result<ExtensionImportReport, RuntimeError> {
    let extension_parent = canonical_extension_parent(root)?;
    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    let mut selections = Vec::new();
    for extension in discover_extensions(root)? {
        if !extension.manifest.enabled {
            skipped.push(extension.name);
            continue;
        }
        let package_path = checked_child(&extension.root, &extension.manifest.package)?;
        let bytes = fs::read(&package_path)?;

        let signature_path = extension
            .manifest
            .signature
            .as_deref()
            .map(|path| checked_child(&extension.root, path))
            .transpose()?
            .or_else(|| {
                let candidate = package_path.with_extension("sig.json");
                candidate.is_file().then_some(candidate)
            });
        let signature = signature_path
            .map(fs::read_to_string)
            .transpose()?
            .map(|json| serde_json::from_str::<PackageSignature>(&json))
            .transpose()?;
        let mut install_options = opts.clone();
        install_options.activate = false;
        let result = store.install_with_assets(
            &bytes,
            signature.as_ref(),
            &install_options,
            &extension.root,
        )?;
        selections.push(PackageDependency {
            namespace: result.meta.namespace.clone(),
            package_id: result.meta.package_id.clone(),
            version: result.meta.version.clone(),
            logic_hash: [0u8; 32],
        });
        installed.push(result.meta);
    }
    store.reconcile_extension_set(&extension_parent, &selections)?;
    Ok(ExtensionImportReport {
        installed,
        skipped,
        content_lock: store.current_lock()?,
    })
}

fn canonical_extension_parent(root: &Path) -> Result<PathBuf, RuntimeError> {
    fs::create_dir_all(root)?;
    Ok(root.canonicalize()?)
}

pub(crate) fn checked_child(root: &Path, relative: &str) -> Result<PathBuf, RuntimeError> {
    validate_relative(relative)?;
    let root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(RuntimeError::Invalid(format!(
            "extension file escapes root: {relative}"
        )));
    }
    Ok(path)
}

pub(crate) fn validate_relative(value: &str) -> Result<(), RuntimeError> {
    if value.is_empty()
        || value.contains('\\')
        || Path::new(value).is_absolute()
        || !Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(RuntimeError::Invalid(format!(
            "unsafe extension path {value:?}"
        )));
    }
    Ok(())
}

const fn manifest_schema() -> u32 {
    1
}

const fn enabled_by_default() -> bool {
    true
}

fn default_package_path() -> String {
    "package.npack.json".into()
}

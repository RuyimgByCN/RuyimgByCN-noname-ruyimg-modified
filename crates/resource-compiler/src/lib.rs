//! Build-time resource compiler (architecture plan §28.1).
//!
//! Takes source asset descriptors and produces an [`AssetManifest`] with:
//! - Content-addressed hashes (BLAKE3)
//! - Multi-size image variants
//! - Web-friendly format selection per platform
//! - Dependency resolution (topological validation)
//! - Preload priority assignment
//! - Platform capability tagging

#![deny(missing_docs)]

use resource_schema::{
    AssetEntry, AssetKind, AssetManifest, AudioFormat, AudioVariant, ImageFormat, ImageVariant,
    PlatformTag, PreloadPriority,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A source asset descriptor fed into the compiler.
#[derive(Debug, Clone)]
pub struct SourceAsset {
    /// Logical asset id.
    pub asset_id: String,
    /// Kind.
    pub kind: AssetKind,
    /// Raw source bytes.
    pub bytes: Vec<u8>,
    /// Original width (images only; 0 for non-images).
    pub width: u32,
    /// Original height (images only).
    pub height: u32,
    /// Audio duration in seconds (audio only; 0 for non-audio).
    pub duration_secs: f64,
    /// Sample rate (audio only).
    pub sample_rate: u32,
    /// Channels (audio only).
    pub channels: u8,
    /// Preload priority.
    pub priority: PreloadPriority,
    /// Dependencies.
    pub depends_on: Vec<String>,
    /// Target platforms (empty = all).
    pub platforms: Vec<PlatformTag>,
    /// Size variants to generate (max widths for images; empty = original only).
    pub size_variants: Vec<u32>,
}

/// Manifest plus package-relative paths generated from one asset directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledAssetDirectory {
    /// Compiled asset metadata.
    pub manifest: AssetManifest,
    /// Stable asset id to package-relative source path.
    pub asset_paths: BTreeMap<String, String>,
}

/// Compiler errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Dependency cycle detected.
    DependencyCycle(String),
    /// Duplicate asset id.
    DuplicateId(String),
    /// Dependency references unknown asset.
    UnknownDependency(String),
    /// Empty asset bytes.
    EmptyBytes(String),
    /// Asset directory could not be read.
    Io(String),
    /// Asset path cannot be represented safely in a package.
    InvalidPath(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::DependencyCycle(id) => {
                write!(f, "dependency cycle involving {id}")
            }
            CompileError::DuplicateId(id) => write!(f, "duplicate asset id: {id}"),
            CompileError::UnknownDependency(id) => {
                write!(f, "unknown dependency: {id}")
            }
            CompileError::EmptyBytes(id) => write!(f, "empty bytes for asset {id}"),
            CompileError::Io(message) => write!(f, "asset io: {message}"),
            CompileError::InvalidPath(path) => write!(f, "invalid asset path: {path}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// Content-addressed hash of raw bytes (BLAKE3).
pub fn content_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

/// Recursively compile files under `root` using URL-safe ids derived from relative paths.
pub fn compile_asset_directory(
    package_id: &str,
    root: &Path,
) -> Result<CompiledAssetDirectory, CompileError> {
    if !root.is_dir() {
        return Err(CompileError::InvalidPath(root.display().to_string()));
    }
    let mut files = Vec::new();
    list_files(root, &mut files)?;
    files.sort();
    let mut manifest = AssetManifest::new(package_id);
    let mut asset_paths = BTreeMap::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .map_err(|_| CompileError::InvalidPath(file.display().to_string()))?;
        let relative = package_relative_path(relative)?;
        let asset_id = url_safe_asset_id(&relative);
        if asset_paths
            .insert(asset_id.clone(), format!("assets/{relative}"))
            .is_some()
        {
            return Err(CompileError::DuplicateId(asset_id));
        }
        let bytes = fs::read(&file)
            .map_err(|error| CompileError::Io(format!("{}: {error}", file.display())))?;
        let source = SourceAsset {
            asset_id,
            kind: kind_for_path(&file),
            bytes,
            width: 0,
            height: 0,
            duration_secs: 0.0,
            sample_rate: 0,
            channels: 0,
            priority: PreloadPriority::Lazy,
            depends_on: vec![],
            platforms: vec![],
            size_variants: vec![],
        };
        let entry = compile_assets(package_id, &[source])?
            .entries
            .into_iter()
            .next()
            .expect("one source produces one entry");
        manifest.add_entry(entry);
    }
    Ok(CompiledAssetDirectory {
        manifest,
        asset_paths,
    })
}

/// Copy a scanned asset directory to a self-contained package `assets/` directory.
pub fn copy_asset_directory(source: &Path, target: &Path) -> Result<(), CompileError> {
    let source = source
        .canonicalize()
        .map_err(|error| CompileError::Io(format!("{}: {error}", source.display())))?;
    let target_parent = target
        .parent()
        .ok_or_else(|| CompileError::InvalidPath(target.display().to_string()))?;
    fs::create_dir_all(target_parent)
        .map_err(|error| CompileError::Io(format!("{}: {error}", target_parent.display())))?;
    let target_name = target
        .file_name()
        .ok_or_else(|| CompileError::InvalidPath(target.display().to_string()))?;
    let target = target_parent
        .canonicalize()
        .map_err(|error| CompileError::Io(format!("{}: {error}", target_parent.display())))?
        .join(target_name);
    if target.exists() && target.canonicalize().ok().as_deref() == Some(source.as_path()) {
        return Ok(());
    }
    if target.starts_with(&source) || source.starts_with(&target) {
        return Err(CompileError::InvalidPath(target.display().to_string()));
    }
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|error| CompileError::Io(format!("{}: {error}", target.display())))?;
    }
    let mut files = Vec::new();
    list_files(&source, &mut files)?;
    for file in files {
        let relative = file
            .strip_prefix(&source)
            .map_err(|_| CompileError::InvalidPath(file.display().to_string()))?;
        let destination = target.join(relative);
        fs::create_dir_all(destination.parent().unwrap())
            .map_err(|error| CompileError::Io(format!("{}: {error}", destination.display())))?;
        fs::copy(&file, &destination)
            .map_err(|error| CompileError::Io(format!("{}: {error}", destination.display())))?;
    }
    Ok(())
}

fn list_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), CompileError> {
    let entries = fs::read_dir(directory)
        .map_err(|error| CompileError::Io(format!("{}: {error}", directory.display())))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| CompileError::Io(format!("{}: {error}", directory.display())))?;
        let file_type = entry
            .file_type()
            .map_err(|error| CompileError::Io(format!("{}: {error}", entry.path().display())))?;
        if file_type.is_dir() {
            list_files(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        } else {
            return Err(CompileError::InvalidPath(
                entry.path().display().to_string(),
            ));
        }
    }
    Ok(())
}

fn package_relative_path(path: &Path) -> Result<String, CompileError> {
    let mut parts = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(CompileError::InvalidPath(path.display().to_string()));
        };
        parts.push(
            part.to_str()
                .ok_or_else(|| CompileError::InvalidPath(path.display().to_string()))?,
        );
    }
    if parts.is_empty() {
        return Err(CompileError::InvalidPath(path.display().to_string()));
    }
    Ok(parts.join("/"))
}

fn url_safe_asset_id(relative: &str) -> String {
    relative
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || "._-".contains(character) {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn kind_for_path(path: &Path) -> AssetKind {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "gif" | "bmp" => AssetKind::Image,
        "atlas" => AssetKind::TextureAtlas,
        "mp3" | "ogg" | "opus" | "wav" | "m4a" | "aac" | "flac" => AssetKind::Audio,
        "ttf" | "otf" | "woff" | "woff2" => AssetKind::Font,
        _ => AssetKind::Data,
    }
}

/// Compile a list of source assets into an [`AssetManifest`].
///
/// Validates dependency graph (no cycles), generates content hashes, and
/// selects appropriate format variants per platform.
pub fn compile_assets(
    package_id: &str,
    sources: &[SourceAsset],
) -> Result<AssetManifest, CompileError> {
    // Validate: no duplicates, no empty bytes.
    let mut seen_ids = std::collections::HashSet::new();
    for s in sources {
        if !seen_ids.insert(&s.asset_id) {
            return Err(CompileError::DuplicateId(s.asset_id.clone()));
        }
        if s.bytes.is_empty() {
            return Err(CompileError::EmptyBytes(s.asset_id.clone()));
        }
    }

    // Validate: all dependencies must reference known assets.
    let id_set: std::collections::HashSet<&str> =
        sources.iter().map(|s| s.asset_id.as_str()).collect();
    for s in sources {
        for dep in &s.depends_on {
            if !id_set.contains(dep.as_str()) {
                return Err(CompileError::UnknownDependency(dep.clone()));
            }
        }
    }

    // Validate: no dependency cycles (topological check).
    check_no_cycles(sources)?;

    // Build manifest.
    let mut manifest = AssetManifest::new(package_id);
    for s in sources {
        let entry = match s.kind {
            AssetKind::Image | AssetKind::TextureAtlas => compile_image_entry(s),
            AssetKind::Audio => compile_audio_entry(s),
            _ => compile_data_entry(s),
        };
        manifest.add_entry(entry);
    }
    Ok(manifest)
}

/// Compile an image source into an asset entry with multi-size variants.
fn compile_image_entry(s: &SourceAsset) -> AssetEntry {
    let mut variants = Vec::new();
    // Generate requested size variants + original.
    let mut sizes = s.size_variants.clone();
    if sizes.is_empty() {
        sizes.push(s.width);
    }
    sizes.sort_unstable();
    sizes.dedup();

    for &max_w in &sizes {
        let (vw, vh) = if max_w == 0 || max_w >= s.width {
            (s.width, s.height)
        } else {
            let scale = max_w as f64 / s.width.max(1) as f64;
            (max_w, (s.height as f64 * scale).round() as u32)
        };
        // In a real compiler, we'd resize the image here. For the schema-level
        // compiler, we hash the original bytes and estimate decoded size from
        // the target dimensions. The build tool would produce actual encoded
        // variant bytes separately.
        let hash = content_hash(&s.bytes);
        let decoded = (vw as u64) * (vh as u64) * 4; // RGBA
        let compressed = (s.bytes.len() as f64 * (vw as f64 / s.width.max(1) as f64)) as u64;
        variants.push(ImageVariant {
            format: ImageFormat::WebP,
            width: vw,
            height: vh,
            platforms: if s.platforms.is_empty() {
                vec![]
            } else {
                s.platforms.clone()
            },
            content_hash: hash,
            compressed_size: compressed.max(100),
            decoded_size: decoded,
        });
    }

    AssetEntry {
        asset_id: s.asset_id.clone(),
        source_hash: content_hash(&s.bytes),
        kind: s.kind,
        image_variants: variants,
        audio_variants: vec![],
        priority: s.priority,
        depends_on: s.depends_on.clone(),
        platforms: s.platforms.clone(),
    }
}

/// Compile an audio source.
fn compile_audio_entry(s: &SourceAsset) -> AssetEntry {
    let hash = content_hash(&s.bytes);
    let decoded = (s.duration_secs * s.sample_rate as f64 * s.channels as f64) as u64;
    let variant = AudioVariant {
        format: AudioFormat::Opus,
        platforms: s.platforms.clone(),
        content_hash: hash,
        compressed_size: s.bytes.len() as u64,
        decoded_size: decoded,
    };
    AssetEntry {
        asset_id: s.asset_id.clone(),
        source_hash: content_hash(&s.bytes),
        kind: s.kind,
        image_variants: vec![],
        audio_variants: vec![variant],
        priority: s.priority,
        depends_on: s.depends_on.clone(),
        platforms: s.platforms.clone(),
    }
}

/// Compile a generic data asset (font, json, etc.).
fn compile_data_entry(s: &SourceAsset) -> AssetEntry {
    AssetEntry {
        asset_id: s.asset_id.clone(),
        source_hash: content_hash(&s.bytes),
        kind: s.kind,
        image_variants: vec![],
        audio_variants: vec![],
        priority: s.priority,
        depends_on: s.depends_on.clone(),
        platforms: s.platforms.clone(),
    }
}

/// Check for dependency cycles via DFS topological sort.
fn check_no_cycles(sources: &[SourceAsset]) -> Result<(), CompileError> {
    // Build adjacency: id → deps
    let graph: std::collections::HashMap<&str, Vec<String>> = sources
        .iter()
        .map(|s| (s.asset_id.as_str(), s.depends_on.clone()))
        .collect();
    let mut visited: std::collections::HashMap<String, u8> =
        sources.iter().map(|s| (s.asset_id.clone(), 0u8)).collect();
    for s in sources {
        if visited[&s.asset_id] == 0 {
            dfs_visit(&s.asset_id.clone(), &graph, &mut visited)?;
        }
    }
    Ok(())
}

fn dfs_visit(
    node: &str,
    graph: &std::collections::HashMap<&str, Vec<String>>,
    visited: &mut std::collections::HashMap<String, u8>,
) -> Result<(), CompileError> {
    visited.insert(node.into(), 1);
    let deps = graph.get(node).cloned().unwrap_or_default();
    for dep in &deps {
        match visited.get(dep).copied() {
            Some(1) => return Err(CompileError::DependencyCycle(dep.clone())),
            Some(0) => dfs_visit(dep, graph, visited)?,
            _ => {}
        }
    }
    visited.insert(node.into(), 2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_basic_image_manifest() {
        let sources = vec![SourceAsset {
            asset_id: "card.sha".into(),
            kind: AssetKind::Image,
            bytes: vec![0xAB; 5000],
            width: 512,
            height: 712,
            duration_secs: 0.0,
            sample_rate: 0,
            channels: 0,
            priority: PreloadPriority::High,
            depends_on: vec![],
            platforms: vec![PlatformTag::Browser],
            size_variants: vec![256, 512],
        }];
        let manifest = compile_assets("standard", &sources).unwrap();
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].image_variants.len(), 2);
        assert!(manifest.total_compressed_size > 0);
        assert!(manifest.total_decoded_size > 0);
        // Smaller variant has less decoded memory.
        assert!(
            manifest.entries[0].image_variants[0].decoded_size
                <= manifest.entries[0].image_variants[1].decoded_size
        );
    }

    #[test]
    fn dependency_cycle_detected() {
        let sources = vec![
            SourceAsset {
                asset_id: "a".into(),
                kind: AssetKind::Data,
                bytes: vec![1],
                width: 0,
                height: 0,
                duration_secs: 0.0,
                sample_rate: 0,
                channels: 0,
                priority: PreloadPriority::Normal,
                depends_on: vec!["b".into()],
                platforms: vec![],
                size_variants: vec![],
            },
            SourceAsset {
                asset_id: "b".into(),
                kind: AssetKind::Data,
                bytes: vec![2],
                width: 0,
                height: 0,
                duration_secs: 0.0,
                sample_rate: 0,
                channels: 0,
                priority: PreloadPriority::Normal,
                depends_on: vec!["a".into()],
                platforms: vec![],
                size_variants: vec![],
            },
        ];
        let err = compile_assets("standard", &sources).unwrap_err();
        assert!(matches!(err, CompileError::DependencyCycle(_)));
    }

    #[test]
    fn unknown_dependency_detected() {
        let sources = vec![SourceAsset {
            asset_id: "a".into(),
            kind: AssetKind::Data,
            bytes: vec![1],
            width: 0,
            height: 0,
            duration_secs: 0.0,
            sample_rate: 0,
            channels: 0,
            priority: PreloadPriority::Normal,
            depends_on: vec!["nonexistent".into()],
            platforms: vec![],
            size_variants: vec![],
        }];
        let err = compile_assets("standard", &sources).unwrap_err();
        assert!(matches!(err, CompileError::UnknownDependency(_)));
    }

    #[test]
    fn duplicate_id_detected() {
        let sources = vec![
            SourceAsset {
                asset_id: "a".into(),
                kind: AssetKind::Data,
                bytes: vec![1],
                width: 0,
                height: 0,
                duration_secs: 0.0,
                sample_rate: 0,
                channels: 0,
                priority: PreloadPriority::Normal,
                depends_on: vec![],
                platforms: vec![],
                size_variants: vec![],
            },
            SourceAsset {
                asset_id: "a".into(),
                kind: AssetKind::Data,
                bytes: vec![2],
                width: 0,
                height: 0,
                duration_secs: 0.0,
                sample_rate: 0,
                channels: 0,
                priority: PreloadPriority::Normal,
                depends_on: vec![],
                platforms: vec![],
                size_variants: vec![],
            },
        ];
        let err = compile_assets("standard", &sources).unwrap_err();
        assert!(matches!(err, CompileError::DuplicateId(_)));
    }

    #[test]
    fn audio_compiled_with_buffer_estimate() {
        let sources = vec![SourceAsset {
            asset_id: "bgm.battle".into(),
            kind: AssetKind::Audio,
            bytes: vec![0xCD; 100_000],
            width: 0,
            height: 0,
            duration_secs: 30.0,
            sample_rate: 44_100,
            channels: 2,
            priority: PreloadPriority::Normal,
            depends_on: vec![],
            platforms: vec![],
            size_variants: vec![],
        }];
        let manifest = compile_assets("standard", &sources).unwrap();
        let audio = &manifest.entries[0].audio_variants[0];
        // 30s × 44100 × 2 channels = 2,646,000 samples
        assert_eq!(audio.decoded_size, 2_646_000);
        assert_eq!(audio.format, AudioFormat::Opus);
    }

    #[test]
    fn directory_scan_is_stable_url_safe_and_detects_collisions() {
        let root = std::env::temp_dir().join(format!(
            "noname-resource-compiler-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::write(root.join("nested/hero.png"), b"image").unwrap();
        std::fs::write(root.join("voice.ogg"), b"audio").unwrap();
        let compiled = compile_asset_directory("test", &root).unwrap();
        assert_eq!(
            compiled.asset_paths["nested_hero.png"],
            "assets/nested/hero.png"
        );
        assert_eq!(compiled.asset_paths["voice.ogg"], "assets/voice.ogg");
        assert_eq!(compiled.manifest.entries[0].kind, AssetKind::Image);
        assert_eq!(
            compiled.manifest.entries[0].source_hash,
            content_hash(b"image")
        );

        std::fs::write(root.join("nested_hero.png"), b"collision").unwrap();
        assert!(matches!(
            compile_asset_directory("test", &root),
            Err(CompileError::DuplicateId(id)) if id == "nested_hero.png"
        ));
        std::fs::remove_dir_all(root).unwrap();
    }
}

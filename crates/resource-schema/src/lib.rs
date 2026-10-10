//! Resource pipeline schema (architecture plan §28).
//!
//! Defines the build-time asset manifest and runtime budget model. The asset
//! compiler produces an [`AssetManifest`] from source assets; the runtime
//! Resource Manager consumes it for preload, budget enforcement, and fallback.

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Asset manifest schema version.
pub const ASSET_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Kind of asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AssetKind {
    /// Character / card / UI image.
    Image = 1,
    /// Texture atlas (multiple sprites packed).
    TextureAtlas = 2,
    /// Audio clip (sfx / bgm).
    Audio = 3,
    /// Font file.
    Font = 4,
    /// Data file (json, binary table).
    Data = 5,
}

/// Image format variant (§28.1: Web-friendly formats).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ImageFormat {
    /// WebP (modern browsers).
    WebP = 1,
    /// AVIF (next-gen).
    Avif = 2,
    /// PNG (lossless fallback).
    Png = 3,
    /// JPEG (photo).
    Jpeg = 4,
}

/// Audio format variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AudioFormat {
    /// Opus (modern, low-bitrate).
    Opus = 1,
    /// AAC (broad compatibility).
    Aac = 2,
    /// Vorbis (legacy).
    Vorbis = 3,
}

/// Platform capability tag (§28.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlatformTag {
    /// Desktop native (full GPU).
    Desktop = 1,
    /// High-end mobile.
    MobileHigh = 2,
    /// Low-end mobile (WebView, limited memory).
    MobileLow = 3,
    /// Browser (WASM + WebGL/WebGPU).
    Browser = 4,
}

/// Preload priority (§28.1). Higher = load earlier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum PreloadPriority {
    /// Lazy load (only on demand).
    Lazy = -1,
    /// Normal.
    Normal = 0,
    /// High (preload at scene enter).
    High = 1,
    /// Critical (preload at app start).
    Critical = 2,
}

/// One size variant of an image asset (§28.1: multi-size variants).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageVariant {
    /// Format.
    pub format: ImageFormat,
    /// Width in pixels (0 = original).
    pub width: u32,
    /// Height in pixels (0 = original).
    pub height: u32,
    /// Platform tags this variant targets (empty = all).
    pub platforms: Vec<PlatformTag>,
    /// Content-addressed hash (BLAKE3) of the encoded bytes.
    pub content_hash: [u8; 32],
    /// Compressed byte size on disk.
    pub compressed_size: u64,
    /// Estimated decoded byte size in memory (width × height × 4 for RGBA).
    pub decoded_size: u64,
}

/// One format variant of an audio asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioVariant {
    /// Format.
    pub format: AudioFormat,
    /// Platform tags.
    pub platforms: Vec<PlatformTag>,
    /// Content-addressed hash.
    pub content_hash: [u8; 32],
    /// Compressed byte size.
    pub compressed_size: u64,
    /// Estimated decoded buffer size (duration × sample_rate × channels).
    pub decoded_size: u64,
}

/// A single asset entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetEntry {
    /// Logical asset id (e.g. `standard:card.sha`).
    pub asset_id: String,
    /// Hash of the source file referenced by a content package.
    #[serde(default, skip_serializing_if = "is_zero_hash")]
    pub source_hash: [u8; 32],
    /// Asset kind.
    pub kind: AssetKind,
    /// Image variants (present if kind == Image or TextureAtlas).
    #[serde(default)]
    pub image_variants: Vec<ImageVariant>,
    /// Audio variants (present if kind == Audio).
    #[serde(default)]
    pub audio_variants: Vec<AudioVariant>,
    /// Preload priority.
    pub priority: PreloadPriority,
    /// Dependencies — other asset ids that must load before this one.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Platform tags this asset is relevant to (empty = all platforms).
    #[serde(default)]
    pub platforms: Vec<PlatformTag>,
}

fn is_zero_hash(value: &[u8; 32]) -> bool {
    *value == [0u8; 32]
}

/// The full asset manifest produced by the build-time compiler (§28.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Schema version.
    pub schema_version: u32,
    /// Package id this manifest belongs to.
    pub package_id: String,
    /// Total compressed size (sum of all variant compressed_size).
    pub total_compressed_size: u64,
    /// Total estimated decoded size (sum of all variant decoded_size).
    pub total_decoded_size: u64,
    /// Asset entries.
    pub entries: Vec<AssetEntry>,
}

impl AssetManifest {
    /// Create an empty manifest for a package.
    pub fn new(package_id: &str) -> Self {
        Self {
            schema_version: ASSET_MANIFEST_SCHEMA_VERSION,
            package_id: package_id.into(),
            total_compressed_size: 0,
            total_decoded_size: 0,
            entries: Vec::new(),
        }
    }

    /// Add an entry and update totals.
    pub fn add_entry(&mut self, entry: AssetEntry) {
        for v in &entry.image_variants {
            self.total_compressed_size += v.compressed_size;
            self.total_decoded_size += v.decoded_size;
        }
        for v in &entry.audio_variants {
            self.total_compressed_size += v.compressed_size;
            self.total_decoded_size += v.decoded_size;
        }
        self.entries.push(entry);
    }

    /// Look up an asset by id.
    pub fn get(&self, asset_id: &str) -> Option<&AssetEntry> {
        self.entries.iter().find(|e| e.asset_id == asset_id)
    }

    /// Assets at or above a priority threshold, in priority order.
    pub fn preload_order(&self, min_priority: PreloadPriority) -> Vec<&AssetEntry> {
        let mut filtered: Vec<&AssetEntry> = self
            .entries
            .iter()
            .filter(|e| e.priority >= min_priority)
            .collect();
        filtered.sort_by(|a, b| b.priority.cmp(&a.priority));
        filtered
    }

    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

// ---- §28.3 Budget model ----

/// Runtime memory budget per category (architecture §28.3).
///
/// Each category must be tracked independently — "WebP is small on disk" does
/// not imply "small in memory after decode."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryBudget {
    /// Compressed asset bytes on disk / in package.
    pub compressed_bytes: u64,
    /// Decoded image pixel buffer memory.
    pub decoded_image_bytes: u64,
    /// GPU texture memory (may differ from decoded due to format / mipmaps).
    pub gpu_texture_bytes: u64,
    /// Audio decode buffer memory.
    pub audio_buffer_bytes: u64,
    /// DOM node count budget.
    pub dom_nodes: u32,
    /// Canvas / Pixi object count budget.
    pub canvas_objects: u32,
    /// Worker / WASM linear memory bytes.
    pub worker_wasm_bytes: u64,
}

impl Default for MemoryBudget {
    fn default() -> Self {
        // Default budgets for a mid-range device.
        Self {
            compressed_bytes: 256 * 1024 * 1024,    // 256 MB
            decoded_image_bytes: 512 * 1024 * 1024, // 512 MB
            gpu_texture_bytes: 512 * 1024 * 1024,   // 512 MB
            audio_buffer_bytes: 64 * 1024 * 1024,   // 64 MB
            dom_nodes: 50_000,
            canvas_objects: 5_000,
            worker_wasm_bytes: 128 * 1024 * 1024, // 128 MB
        }
    }
}

/// Low-memory mode budget (reduced limits for low-end mobile).
impl MemoryBudget {
    /// Budget for a low-end mobile device.
    pub fn low_memory() -> Self {
        Self {
            compressed_bytes: 64 * 1024 * 1024,
            decoded_image_bytes: 128 * 1024 * 1024,
            gpu_texture_bytes: 128 * 1024 * 1024,
            audio_buffer_bytes: 16 * 1024 * 1024,
            dom_nodes: 10_000,
            canvas_objects: 1_000,
            worker_wasm_bytes: 64 * 1024 * 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trip() {
        let mut m = AssetManifest::new("standard");
        m.add_entry(AssetEntry {
            asset_id: "standard:card.sha".into(),
            source_hash: [0u8; 32],
            kind: AssetKind::Image,
            image_variants: vec![ImageVariant {
                format: ImageFormat::WebP,
                width: 256,
                height: 356,
                platforms: vec![PlatformTag::Browser],
                content_hash: [1u8; 32],
                compressed_size: 12_000,
                decoded_size: 256 * 356 * 4,
            }],
            audio_variants: vec![],
            priority: PreloadPriority::High,
            depends_on: vec![],
            platforms: vec![],
        });
        assert!(m.total_compressed_size > 0);
        assert!(m.total_decoded_size > 0);

        let bytes = m.to_json_bytes().unwrap();
        let restored = AssetManifest::from_json_bytes(&bytes).unwrap();
        assert_eq!(m, restored);
    }

    #[test]
    fn preload_order_by_priority() {
        let mut m = AssetManifest::new("standard");
        m.add_entry(AssetEntry {
            asset_id: "a".into(),
            source_hash: [0u8; 32],
            kind: AssetKind::Data,
            image_variants: vec![],
            audio_variants: vec![],
            priority: PreloadPriority::Normal,
            depends_on: vec![],
            platforms: vec![],
        });
        m.add_entry(AssetEntry {
            asset_id: "b".into(),
            source_hash: [0u8; 32],
            kind: AssetKind::Data,
            image_variants: vec![],
            audio_variants: vec![],
            priority: PreloadPriority::Critical,
            depends_on: vec![],
            platforms: vec![],
        });
        m.add_entry(AssetEntry {
            asset_id: "c".into(),
            source_hash: [0u8; 32],
            kind: AssetKind::Data,
            image_variants: vec![],
            audio_variants: vec![],
            priority: PreloadPriority::Lazy,
            depends_on: vec![],
            platforms: vec![],
        });
        let order: Vec<&str> = m
            .preload_order(PreloadPriority::Normal)
            .iter()
            .map(|e| e.asset_id.as_str())
            .collect();
        // Critical first, then Normal. Lazy excluded.
        assert_eq!(order, vec!["b", "a"]);
    }

    #[test]
    fn low_memory_budget_is_smaller() {
        let normal = MemoryBudget::default();
        let low = MemoryBudget::low_memory();
        assert!(low.decoded_image_bytes < normal.decoded_image_bytes);
        assert!(low.gpu_texture_bytes < normal.gpu_texture_bytes);
    }
}

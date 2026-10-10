//! Runtime Resource Manager (architecture plan §28.2).
//!
//! Consumes an [`AssetManifest`] and manages in-memory asset lifecycle:
//! - Scene-based preload (by priority)
//! - Decode concurrency limit
//! - Image/GPU/audio memory budget enforcement (§28.3)
//! - LRU eviction with generation tracking
//! - Reference counting (asset only freed when refs = 0)
//! - Low-memory mode (reduced budgets)
//! - Missing-resource fallback (placeholder)

#![deny(missing_docs)]

use resource_schema::{AssetManifest, MemoryBudget};
use std::collections::{HashMap, VecDeque};

/// Default max concurrent decode operations.
pub const DEFAULT_DECODE_CONCURRENCY: u32 = 4;

/// A loaded asset in the cache.
#[derive(Debug, Clone)]
struct CachedAsset {
    /// Decoded size in bytes (for budget tracking).
    decoded_size: u64,
    /// Current reference count.
    ref_count: u32,
    /// LRU generation (incremented on access).
    last_access_gen: u64,
}

/// Resource Manager errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// Asset not found in manifest.
    NotFound(String),
    /// Budget exceeded — cannot load without evicting.
    BudgetExceeded {
        /// Category.
        category: &'static str,
        /// Needed bytes.
        needed: u64,
        /// Available bytes.
        available: u64,
    },
    /// Decode concurrency limit reached.
    ConcurrencyLimit,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::NotFound(id) => write!(f, "asset not found: {id}"),
            ResourceError::BudgetExceeded {
                category,
                needed,
                available,
            } => write!(
                f,
                "{category} budget exceeded: need {needed}, have {available}"
            ),
            ResourceError::ConcurrencyLimit => write!(f, "decode concurrency limit reached"),
        }
    }
}

impl std::error::Error for ResourceError {}

/// Runtime Resource Manager (§28.2).
pub struct ResourceManager {
    /// Asset manifest (build-time produced).
    manifest: AssetManifest,
    /// Memory budget.
    budget: MemoryBudget,
    /// Whether low-memory mode is active.
    low_memory: bool,
    /// Max concurrent decodes.
    decode_concurrency: u32,
    /// Current in-flight decodes.
    in_flight_decodes: u32,
    /// Cached assets: id → cache entry.
    cache: HashMap<String, CachedAsset>,
    /// LRU eviction queue (least recently used at front).
    lru_queue: VecDeque<String>,
    /// Current decoded image memory usage.
    decoded_image_used: u64,
    /// Current audio buffer usage.
    audio_used: u64,
    /// Global access generation counter.
    access_gen: u64,
    /// Whether a fallback placeholder is used for missing assets.
    fallback_enabled: bool,
}

impl ResourceManager {
    /// Create a new manager from a manifest + default budget.
    pub fn new(manifest: AssetManifest) -> Self {
        Self {
            manifest,
            budget: MemoryBudget::default(),
            low_memory: false,
            decode_concurrency: DEFAULT_DECODE_CONCURRENCY,
            in_flight_decodes: 0,
            cache: HashMap::new(),
            lru_queue: VecDeque::new(),
            decoded_image_used: 0,
            audio_used: 0,
            access_gen: 0,
            fallback_enabled: true,
        }
    }

    /// Enable / disable low-memory mode (§28.2: 低内存模式).
    pub fn set_low_memory(&mut self, enabled: bool) {
        self.low_memory = enabled;
        self.budget = if enabled {
            MemoryBudget::low_memory()
        } else {
            MemoryBudget::default()
        };
    }

    /// Whether low-memory mode is active.
    pub fn is_low_memory(&self) -> bool {
        self.low_memory
    }

    /// Set max decode concurrency.
    pub fn set_decode_concurrency(&mut self, limit: u32) {
        self.decode_concurrency = limit.max(1);
    }

    /// Current decoded image memory usage.
    pub fn decoded_image_used(&self) -> u64 {
        self.decoded_image_used
    }

    /// Current audio buffer usage.
    pub fn audio_used(&self) -> u64 {
        self.audio_used
    }

    /// Number of cached assets.
    pub fn cache_count(&self) -> usize {
        self.cache.len()
    }

    /// Whether an asset is currently cached (loaded).
    pub fn is_loaded(&self, asset_id: &str) -> bool {
        self.cache.contains_key(asset_id)
    }

    /// Request to load an asset (§28.2: 按场景预加载).
    ///
    /// Checks budget, evicts LRU if needed, increments ref count. Returns
    /// `Err(NotFound)` if the asset is unknown and fallback is disabled.
    pub fn load(&mut self, asset_id: &str) -> Result<(), ResourceError> {
        let entry = match self.manifest.get(asset_id) {
            Some(e) => e,
            None => {
                if self.fallback_enabled {
                    return Ok(()); // Fallback: pretend loaded with placeholder
                }
                return Err(ResourceError::NotFound(asset_id.into()));
            }
        };

        // If already cached, just bump ref + access.
        if let Some(cached) = self.cache.get_mut(asset_id) {
            cached.ref_count += 1;
            self.access_gen += 1;
            cached.last_access_gen = self.access_gen;
            return Ok(());
        }

        // Determine decoded size for this asset.
        let decoded_size = self.estimate_decoded_size(entry);

        // Check decode concurrency.
        if self.in_flight_decodes >= self.decode_concurrency {
            return Err(ResourceError::ConcurrencyLimit);
        }

        // Check budget; evict LRU if needed.
        let category = self.category_for(entry);
        let budget_limit = self.budget_for(category);
        let current_used = self.used_for(category);
        if current_used + decoded_size > budget_limit {
            // Try evicting unreferenced LRU entries.
            self.evict_for(category, decoded_size, budget_limit)?;
        }

        // Load into cache.
        self.in_flight_decodes += 1;
        self.access_gen += 1;
        self.cache.insert(
            asset_id.into(),
            CachedAsset {
                decoded_size,
                ref_count: 1,
                last_access_gen: self.access_gen,
            },
        );
        self.lru_queue.push_back(asset_id.into());
        self.add_usage(category, decoded_size);

        // Decode completes immediately in this model (sync).
        self.in_flight_decodes -= 1;

        Ok(())
    }

    /// Release a reference to an asset (§28.2: 引用计数).
    /// When ref count reaches 0, the asset becomes eligible for LRU eviction.
    pub fn release(&mut self, asset_id: &str) {
        if let Some(cached) = self.cache.get_mut(asset_id) {
            if cached.ref_count > 0 {
                cached.ref_count -= 1;
            }
        }
    }

    /// Preload all assets at or above a priority (§28.2: 按场景预加载).
    pub fn preload_scene(
        &mut self,
        min_priority: resource_schema::PreloadPriority,
    ) -> Vec<ResourceError> {
        let ids: Vec<String> = self
            .manifest
            .preload_order(min_priority)
            .into_iter()
            .map(|e| e.asset_id.clone())
            .collect();
        let mut errors = Vec::new();
        for id in ids {
            if let Err(e) = self.load(&id) {
                errors.push(e);
            }
        }
        errors
    }

    /// Evict all unreferenced assets from the cache (§28.2: 断开页面后的释放).
    pub fn flush_unreferenced(&mut self) {
        let to_remove: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, c)| c.ref_count == 0)
            .map(|(k, _)| k.clone())
            .collect();
        for id in to_remove {
            self.remove_asset(&id);
        }
    }

    /// Total memory in use across all categories.
    pub fn total_memory_used(&self) -> u64 {
        self.decoded_image_used + self.audio_used
    }

    // ---- internal helpers ----

    fn estimate_decoded_size(&self, entry: &resource_schema::AssetEntry) -> u64 {
        // Use the first (smallest) variant's decoded size.
        if let Some(v) = entry.image_variants.first() {
            v.decoded_size
        } else if let Some(v) = entry.audio_variants.first() {
            v.decoded_size
        } else {
            0
        }
    }

    fn category_for(&self, entry: &resource_schema::AssetEntry) -> &'static str {
        match entry.kind {
            resource_schema::AssetKind::Image | resource_schema::AssetKind::TextureAtlas => {
                "decoded_image"
            }
            resource_schema::AssetKind::Audio => "audio",
            _ => "data",
        }
    }

    fn budget_for(&self, category: &str) -> u64 {
        match category {
            "decoded_image" => self.budget.decoded_image_bytes,
            "audio" => self.budget.audio_buffer_bytes,
            _ => u64::MAX, // data has no specific budget
        }
    }

    fn used_for(&self, category: &str) -> u64 {
        match category {
            "decoded_image" => self.decoded_image_used,
            "audio" => self.audio_used,
            _ => 0,
        }
    }

    fn add_usage(&mut self, category: &str, size: u64) {
        match category {
            "decoded_image" => self.decoded_image_used += size,
            "audio" => self.audio_used += size,
            _ => {}
        }
    }

    fn sub_usage(&mut self, category: &str, size: u64) {
        match category {
            "decoded_image" => {
                self.decoded_image_used = self.decoded_image_used.saturating_sub(size)
            }
            "audio" => self.audio_used = self.audio_used.saturating_sub(size),
            _ => {}
        }
    }

    /// Evict unreferenced LRU entries until enough space is available.
    fn evict_for(
        &mut self,
        category: &'static str,
        needed: u64,
        budget_limit: u64,
    ) -> Result<(), ResourceError> {
        let current = self.used_for(category);
        // Collect eviction candidates: assets in this category with ref_count=0.
        let mut candidates: Vec<(String, u64, u64)> = self
            .cache
            .iter()
            .filter(|(_, c)| c.ref_count == 0)
            .map(|(k, c)| (k.clone(), c.last_access_gen, c.decoded_size))
            .collect();
        // Sort by access gen ascending (oldest first = LRU).
        candidates.sort_by_key(|(_, gen, _)| *gen);

        let mut freed = 0u64;
        for (id, _, size) in &candidates {
            if current + needed - freed <= budget_limit {
                break;
            }
            self.remove_asset(id);
            freed += size;
        }

        if self.used_for(category) + needed > budget_limit {
            return Err(ResourceError::BudgetExceeded {
                category,
                needed,
                available: budget_limit.saturating_sub(self.used_for(category)),
            });
        }
        Ok(())
    }

    fn remove_asset(&mut self, id: &str) {
        if let Some(cached) = self.cache.remove(id) {
            // Determine category from manifest.
            if let Some(entry) = self.manifest.get(id) {
                let cat = self.category_for(entry);
                self.sub_usage(cat, cached.decoded_size);
            }
        }
        self.lru_queue.retain(|x| x != id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use resource_schema::{AssetEntry, AssetKind, ImageFormat, ImageVariant, PreloadPriority};

    fn make_manifest() -> AssetManifest {
        let mut m = AssetManifest::new("test");
        for i in 0..5 {
            m.add_entry(AssetEntry {
                asset_id: format!("img{i}"),
                source_hash: [i as u8; 32],
                kind: AssetKind::Image,
                image_variants: vec![ImageVariant {
                    format: ImageFormat::WebP,
                    width: 256,
                    height: 256,
                    platforms: vec![],
                    content_hash: [i as u8; 32],
                    compressed_size: 10_000,
                    decoded_size: 256 * 256 * 4, // 256KB
                }],
                audio_variants: vec![],
                priority: if i < 2 {
                    PreloadPriority::Critical
                } else {
                    PreloadPriority::Normal
                },
                depends_on: vec![],
                platforms: vec![],
            });
        }
        m
    }

    #[test]
    fn load_and_release_refcount() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        assert!(!rm.is_loaded("img0"));
        rm.load("img0").unwrap();
        assert!(rm.is_loaded("img0"));
        assert_eq!(rm.cache_count(), 1);
        // Load again bumps ref.
        rm.load("img0").unwrap();
        // Release twice.
        rm.release("img0");
        rm.release("img0");
        // Still cached but ref_count = 0.
        assert!(rm.is_loaded("img0"));
    }

    #[test]
    fn preload_scene_loads_by_priority() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        let errors = rm.preload_scene(PreloadPriority::Critical);
        assert!(errors.is_empty());
        // Only 2 critical assets.
        assert_eq!(rm.cache_count(), 2);
    }

    #[test]
    fn budget_exceeded_triggers_eviction() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        // Set a tiny image budget (only 1 image fits).
        rm.budget.decoded_image_bytes = 256 * 256 * 4 + 1;
        rm.load("img0").unwrap(); // fills budget
                                  // img1 needs same size; img0 is referenced. Should fail.
        let result = rm.load("img1");
        // img0 is referenced (ref_count=1), so cannot evict → error.
        assert!(result.is_err());
        // Release img0, then img1 can load (img0 evicted).
        rm.release("img0");
        rm.load("img1").unwrap();
    }

    #[test]
    fn flush_unreferenced_clears_cache() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        rm.load("img0").unwrap();
        rm.load("img1").unwrap();
        rm.release("img0");
        rm.release("img1");
        assert_eq!(rm.cache_count(), 2);
        rm.flush_unreferenced();
        assert_eq!(rm.cache_count(), 0);
    }

    #[test]
    fn missing_asset_fallback() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        // Nonexistent asset with fallback enabled → Ok (placeholder).
        assert!(rm.load("nonexistent").is_ok());
        rm.fallback_enabled = false;
        assert!(rm.load("nonexistent").is_err());
    }

    #[test]
    fn low_memory_mode_reduces_budget() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        let normal_budget = rm.budget.decoded_image_bytes;
        rm.set_low_memory(true);
        assert!(rm.budget.decoded_image_bytes < normal_budget);
        assert!(rm.is_low_memory());
    }

    #[test]
    fn decode_concurrency_limit() {
        let manifest = make_manifest();
        let mut rm = ResourceManager::new(manifest);
        rm.set_decode_concurrency(1);
        rm.in_flight_decodes = 1; // Simulate one in-flight decode.
        let result = rm.load("img0");
        assert!(matches!(result, Err(ResourceError::ConcurrencyLimit)));
    }
}

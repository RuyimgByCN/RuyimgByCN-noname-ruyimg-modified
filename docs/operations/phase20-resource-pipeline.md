# 阶段 20：资源流水线（§28）

> 对应架构计划 v2.0 §28（资源流水线：构建期编译器 + 运行期 Resource Manager
> + 预算模型）。资源性能是项目的一级架构问题。

## 目标

建立完整的资源生命周期管理，从构建期到运行期：

1. **构建期**（§28.1）：资源编译器生成 asset manifest
2. **运行期**（§28.2）：Resource Manager 管理内存生命周期
3. **预算**（§28.3）：分类独立统计，不用「磁盘小」推断「内存小」

## 新增 crate

| crate | 路径 | 职责 |
|---|---|---|
| `resource-schema` | `crates/resource-schema` | AssetManifest / AssetEntry / ImageVariant / AudioVariant / MemoryBudget |
| `resource-compiler` | `crates/resource-compiler` | 构建期编译器：多尺寸变体、内容哈希、依赖图验证、优先级 |
| `resource-runtime` | `packages/resource-runtime` | 运行期 Resource Manager |

## §28.1 构建期

```rust
use resource_compiler::{compile_assets, SourceAsset};

let manifest = compile_assets("standard", &[
    SourceAsset {
        asset_id: "card.sha".into(),
        kind: AssetKind::Image,
        bytes: image_bytes,
        width: 512,
        height: 712,
        size_variants: vec![128, 256, 512], // 多尺寸变体
        priority: PreloadPriority::High,
        platforms: vec![PlatformTag::Browser, PlatformTag::MobileLow],
        ..Default::default()
    },
])?;
```

编译器产出：
- 内容寻址哈希（BLAKE3）
- 多尺寸图片变体（§28.1）
- Web 友好格式选择（WebP/AVIF/PNG）
- 音频变体（Opus/AAC/Vorbis）+ 解码缓冲估算
- 依赖关系（拓扑验证 + 环检测）
- 预加载优先级（Critical > High > Normal > Lazy）
- 平台能力标签（Desktop / MobileHigh / MobileLow / Browser）

编译器验证：
- ID 唯一性
- 依赖引用已知资产
- 依赖图无环（DFS 拓扑排序）
- 资源字节非空

## §28.2 运行期

```rust
use resource_runtime::ResourceManager;

let mut rm = ResourceManager::new(manifest);
rm.set_low_memory(true);                    // 低内存模式
rm.set_decode_concurrency(2);               // 解码并发限制
rm.preload_scene(PreloadPriority::Critical); // 按场景预加载
rm.load("card.sha")?;                       // 引用计数 +1
rm.release("card.sha");                     // 引用计数 -1
rm.flush_unreferenced();                    // 释放未引用资产
```

| 能力 | 方法 |
|---|---|
| 按场景预加载 | `preload_scene(min_priority)` |
| 解码并发限制 | `set_decode_concurrency(limit)` |
| 内存预算强制 | `load()` 检查预算，超限触发 LRU 驱逐 |
| LRU 分代回收 | `evict_for()` 按 access generation 排序驱逐 |
| 引用计数 | `load()` +1，`release()` -1，ref=0 才可驱逐 |
| 低内存模式 | `set_low_memory(true)` 切换到缩减预算 |
| 缺失资源回退 | `fallback_enabled` → 占位符；禁用则 `NotFound` |
| 断开页面释放 | `flush_unreferenced()` 清除所有 ref=0 资产 |

## §28.3 预算

```rust
pub struct MemoryBudget {
    pub compressed_bytes: u64,      // 压缩包大小
    pub decoded_image_bytes: u64,   // 解码后图片内存
    pub gpu_texture_bytes: u64,     // GPU 纹理内存
    pub audio_buffer_bytes: u64,    // 音频缓冲
    pub dom_nodes: u32,             // DOM 节点
    pub canvas_objects: u32,        // Canvas / Pixi 对象
    pub worker_wasm_bytes: u64,     // Worker / WASM 内存
}
```

`MemoryBudget::default()` = 中端设备；`MemoryBudget::low_memory()` = 低端移动。
每类独立统计——不能用「磁盘上的 WebP 很小」推断运行内存很小。

## 测试

```bash
cargo test -p resource-schema      # 3 tests
cargo test -p resource-compiler    # 5 tests
cargo test -p resource-runtime     # 7 tests
```

| 领域 | 测试 |
|---|---|
| schema | manifest 往返、preload 优先级排序、低内存预算更小 |
| compiler | 多尺寸变体编译、依赖环检测、未知依赖检测、重复 ID 检测、音频缓冲估算 |
| runtime | load/release 引用计数、preload_scene 优先级、预算超限 LRU 驱逐、flush 清理、缺失回退、低内存模式、解码并发限制 |

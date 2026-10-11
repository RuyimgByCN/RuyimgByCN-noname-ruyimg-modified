# 阶段 27：配置驱动的牌堆（§48）

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

> 将引擎牌堆从硬编码改为从 `content/core-rules/base-deck.json` 读取卡牌定义，
> 使 §48 的「标准卡牌」验收从配置描述变为引擎实际使用的牌堆。

## 改动

### `content/core-rules/base-deck.json`

从描述性 JSON（"count: 53"）改为逐张卡牌定义，共 119 张：

```json
{
  "cards": [
    {"def": 11, "kind": "sha", "suit": "spade", "rank": 1},
    {"def": 12, "kind": "sha", "suit": "spade", "rank": 2},
    ...
    {"def": 100, "kind": "weapon", "suit": "spade", "rank": 6, "weapon_range": 2},
    {"def": 110, "kind": "armor", "suit": "spade", "rank": 2},
    ...
    {"def": 263, "kind": "trick_wuxie", "suit": "diamond", "rank": 12}
  ]
}
```

每张卡牌：`def`（定义 ID）、`kind`（卡牌类型）、`suit`（花色）、`rank`（点数）、
`weapon_range`（武器攻击范围，仅武器牌）。

### `crates/engine-core/src/content/deck.rs`

新增：
- `build_deck_from_config()` — 从嵌入的 JSON 解析牌堆定义（编译时 `include_str!`，无文件系统依赖）
- `build_deck_specs()` — 首选 JSON 配置，回退到硬编码 `build_standard_deck_specs()`
- `parse_kind()` / `parse_suit()` / `equip_slot_for()` — JSON 字符串到枚举的映射

### `bootstrap_match` 改用配置驱动

```rust
// Before (hardcoded):
let specs = build_standard_deck_specs();

// After (config-driven with fallback):
let specs = crate::content::build_deck_specs();
```

## 设计约束

- **无文件系统依赖**：JSON 通过 `include_str!` 在编译时嵌入，引擎核心不读文件（§2.1 不变量）
- **黄金向量稳定**：config deck 的 def/type/suit/rank 与硬编码完全一致，`golden_start` 和
  `conformance` 向量哈希不变
- **硬编码保留**：如果 JSON 解析失败或为空，回退到 `build_standard_deck_specs()`

## 验证

```bash
# 配置解析测试
cargo test -p engine-core --lib config_deck

# 黄金向量不变
cargo test -p engine-core --test golden_regression  # ✓
cargo run -p conformance                            # ✓
cargo run -p engine-cli -- hash 4 42                # 536ff710... (不变)
```

## 测试

| 测试 | 断言 |
|---|---|
| `config_deck_loads_successfully` | 嵌入 JSON 解析成功，包含 Sha/Shan/Tao/Weapon/TrickWuxie |
| `config_deck_matches_hardcoded_count` | 配置和硬编码牌堆卡牌数一致（119） |
| `config_deck_type_distribution_matches_hardcoded` | 每种卡牌类型数量一致 |
| `build_deck_specs_prefers_config` | `build_deck_specs()` 使用配置而非硬编码 |

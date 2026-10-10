# ADR 0594：拆分 `primitives.rs` 与 `resolve_choice.rs` 巨石文件

## 背景

`engine-core` 的两个文件长到了阻碍并行开发的程度：

| 文件 | 行数 | 内容 |
|---|---|---|
| `src/effect/primitives.rs` | 78 094 | 5 209 行 `apply_effect` 调度表 + 1 388 个原语项 |
| `src/command/resolve_choice.rs` | 41 159 | 4 998 行 `handle_resolve_choice` 调度 + 430 个续体处理器 |

两者合计占 `engine-core` 源码（187k 行）的 **64%**。当前有 4 个 agent 分段并行改引擎
（见 `ABI-COORDINATION.md`），每个新技能都要往这两个文件里加函数，冲突面几乎是整个文件；
会话记录里反复出现「中间态编译红」「括号被破坏」「旧桩臂抢先匹配」正是这个后果。

## 决策

把两个文件各自改为模块目录，**调度表留在 `mod.rs`，实现下沉到子模块**：

```
effect/primitives.rs        →  effect/primitives/mod.rs          （前导 use + apply_effect + glob 重导出）
                               effect/primitives/pack_*.rs       （24 个，各约 3 000 行）
command/resolve_choice.rs   →  command/resolve_choice/mod.rs     （前导 use + handle_resolve_choice + 重导出）
                               command/resolve_choice/group_*.rs （12 个，各约 3 000 行）
```

切分**按文件内位置连续切块**，不按包重排。原因：`primitives.rs` 里 19 345 行无法由 DSL
引用关系归属（多为 `resolve_choice` 调用的 `*_choice` 续体），另有 5 120 行被多包共享，
按包硬分会引入大范围可见性改动。而位置本身已经对齐包——函数按移植时间聚集成簇，
文件后半段呈现清晰的 bingshi / xianding / ddd / tw / shiji 连续区。子模块名取该块的
主导内容包（`pack_shiji`、`pack_ddd_2`…），仅作导航提示，不构成归属约束。

连续切分的收益是**没有任何项相对邻居移动过**，因此没有引用跨越了新边界。

## 机械改动清单

除移动文本外只有三类改动，全部是可见性收窄的直接后果：

1. **947 + 424 个默认私有项提升为 `pub(crate)`**。父模块的调度表看不见子模块的私有项。
   `pub`、`pub(crate)`、`pub(super)` 原样保留，所以 crate 现有的
   `pub use primitives::{...}` 重导出可见性不变。
2. **`DcsbmengmouSequence` 的 4 个字段提升为 `pub(crate)`**——它被跨块读取。
3. **`super::` 路径改写**。子模块深了一层，`super::play::` / `super::skill::` /
   `super::judge::` / `super::turn::` 指向的是 `command` 下的兄弟模块，改为
   `crate::command::*`；`super::continue_luanwu_after_hp_loss` 改为
   `crate::effect::continue_luanwu_after_hp_loss`。子模块内 `super::run_effects`
   保持不变，因为它解析到 `primitives/mod.rs` 的前导 `use`（后代可读祖先的私有导入）。

子模块统一 `use super::*;`，`mod.rs` 对每个子模块 glob 重导出——只含 `pub(crate)` 项的
块用 `pub(crate) use` 以免触发 `unused_imports` 警告。

## 验证

- `cargo test --workspace`：66 套件、**862 passed / 1 failed**。
- 唯一失败 `original_clan_draw_to::committed_package_matches_source_and_draw_to_semantics`
  经回滚原文件对照确认**拆分前即存在**（clan 包 ABI 336 过时，B/E 段领域，未动）。
- 警告 **59 个，与拆分前逐类相同**（38 unused variable / 8 unreachable pattern /
  6 needless mut / 5 dead function / 1 unused import / 1 dead const），零新增。
- `cargo test -p conformance`：3 向量全过。切分不触碰内容，`logic_hash` 不变，golden 不漂移。

## 后果

- 单文件最大值从 78 094 行降到 5 285 行（`primitives/mod.rs` 的调度表）。
- 并行会话的冲突面从「整个文件」缩到「单个 3 000 行的包簇」；改 shiji 技能与改 ddd 技能
  不再互相阻塞编译。
- 调度表独立成文件后，`apply_effect` 与 `handle_resolve_choice` 的规模（5 209 / 4 998 行）
  本身成为可见的技术债信号——它们的膨胀速度就是「每技能一个 `EffectRequest` 变体」这一
  模式的直接计量，见 ADR 0595。

## 注意

后续新增原语请加到语义相符的 `pack_*` / `group_*` 子模块，不要在 `mod.rs` 里堆函数——
那里只放调度表。子模块内新增项默认可见性即可（`pub(crate)`），glob 重导出会自动带出。

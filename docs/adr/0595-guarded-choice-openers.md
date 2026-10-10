# ADR 0595：`GuardedChoice` —— 折叠「守卫 + 开一个选择」样板

## 背景

架构计划 §47.3 写明：*「历史包出现 M0 未覆盖的模式时，先扩通用原语再写技能，
**禁止技能级硬编码旁路**。」*

实测这条已经全面失守：`rule-api` 的 `EffectRequest` 有 **682 个变体，其中 344 个是
`Resolve*`；这 344 个里 257 个的名字直接就是某个技能名**（`ResolveYeyan`、
`ResolveTwfeiyi`、`ResolveZhugeruoxue`…）。`content/registry.rs` 的 804 个 filter 里
109 个内嵌技能名。DSL 侧只剩一行 `effect resolve_xxx(...)` 的调度桩，技能实现在引擎里。

代价可量化：**新增一个带选择的技能要同步改 8 个文件**——`rule-api` 变体、
`rule-vm/bytecode.rs`、`rule-vm/vm.rs`、`engine-core/vm_host.rs` 的 ctx 填充臂、
`hash.rs` 的穷举臂、原语实现、`resolve_choice.rs` 的 resume 臂、
`content-compiler` 的 typecheck + codegen。漏改任一处的后果是编译错误，或更糟——
headless conformance 对局卡在选择上死循环。ABI 号已到 640 且单调递增，退化成了技能计数器。

## 结构分析

把 391 个 `resolve_*` 实现按操作序列指纹（`EffectRequest::*` / `open_choice` /
`ChoiceKind` / `ContinuationKind`）聚类：

- **13 组操作序列完全相同**，含 87 个函数；
- 最大三组共 56 个：`open_choice+SelectCards+Custom`(22)、
  `open_choice+Custom+Custom`(17)、`open_choice+SelectTargets+Custom`(17)。

读代码确认这些是同一个三段式骨架：**守卫谓词 → 候选集计算 → 单个 `open_choice`
挂续体**。重复度有数据支撑：

- 307/391 个实现的守卫都以 `originating_skill_is_enabled` 起头，后接高度重复的谓词：
  `turn.phase` 97 次、`turn.current` 89 次、`players.get(x).alive` 51 次；
- `open_choice` 的 `(min, max, optional)` 实参只有 4 种覆盖 90%：
  `(1,1,false)` 259 次、`(1,1,true)` 94 次、`(0,1,true)` 46 次、`(1,max,false)` 10 次。

## 决策

新增 `effect/primitives/guarded_choice.rs`，把守卫与 `open_choice` 样板收进一个
builder。候选集**仍由调用方计算**——它是唯一真正编码技能语义的部分。

```rust
GuardedChoice::new(skill, Participants::owner(player), ContinuationKind::Custom, payload)
    .during(TurnContext::OwnersPhase(Phase::Play))
    .shape(ChoiceShape::one_optional())
    .open_targets(tx, candidates)     // 守卫不过 or 候选集为空 → Ok(vec![])，静默 no-op
```

三个维度各自建模：

- `Participants`：`owner`（查存活）/ `owner_any_state`（不查，濒死技用）/ `pair` /
  `distinct_pair` / `distinct_other_only`（只查对方，拥有者可能已阵亡）。
  **刻意不做成「总是查存活」**——`resolve_potqingshi`、`resolve_jianrou`、
  `resolve_zhuiyi` 的原守卫只有 `originating_skill_is_enabled`，无条件加存活检查会
  静默改变语义。
- `TurnContext`：`Any` / `OwnersTurn` / `OwnersPhase(phase)`。
- `ChoiceShape`：`one` / `one_optional` / `exactly(n)` / `range(min, max, optional)`。

**这不是新的 `EffectRequest` 变体**，是普通函数。所以折叠一个技能是**移除**接线，
不是新增——不动 ABI，不动 DSL，不动 `logic_hash`。

## 落地范围

本次折叠 **21 处**，覆盖 6 个包簇：

| 包簇 | 折叠数 | 技能 |
|---|---|---|
| `pack_shenhua_1` | 11 | 雷击、挑衅、征荣、清俭、烈刃、清识、放权、缔盟、巧变、黄天、享乐 |
| `pack_tw_3` | 4 | 新名士、隐据、曲迪、谋溃 |
| `pack_shenhua_2` | 2 | 荐相、掣政 |
| `pack_old` | 2 | 剑柔、刚烈（三人局分支） |
| `pack_refresh` | 1 | 追忆 |
| `pack_wandianlunsha` | 1 | 率言 |

分类器初筛出 65 个「纯开场桩」候选，但逐个读发现其中 37 个含真实逻辑而非样板——
`resolve_twshanghe` 有轮询 `while` 循环、`resolve_gzbuqu_recover` 有点数重复优先排序、
`resolve_potjugu` / `resolve_zaoli` / `resolve_jinqu` 有摸牌副作用、
`resolve_zhijian` / `resolve_potjiejie` 是多段状态机。这些**不折叠**：套上 builder 只会
把真实逻辑藏进一层间接。`resolve_xiangle` 折叠了守卫但保留原结构，因为它的空候选集分支
不是 no-op（要令该【杀】无效）。

行数不是这次的目标：28 个候选共 1 536 行里样板只占约 465 行（30%），其余是各异的候选集
表达式。**收益在接线税**——被折叠的技能不再需要独立的守卫与 `open_choice` 样板，
后续同形技能可直接复用 builder 而不必新开 `Resolve*` 变体。

## 验证

- `cargo test --workspace`：66 套件、**862 passed / 1 failed**，与折叠前基线逐项一致
  （唯一失败是既有的 `original_clan_draw_to`，见 ADR 0594）。
- `cargo test -p conformance`：3 向量全过，**golden 零漂移**——折叠不碰内容与 DSL。

## 后续

这 21 处是第一梯队。第二梯队是 126 个带 `match stage` 状态机的实现，折叠前需要先给
continuation payload 定一套通用的 stage 布局约定（当前每个技能自己约定
`[MAGIC, stage, ...]` 的字段顺序）。真正要压下 344 个 `Resolve*` 变体，需要的是
把「选目标 → 选牌 → 拼点 → 分支」这类反复出现的多段状态机做成参数化 continuation；
那是一次会动 ABI 与 golden 的改动，应单独立 ADR。

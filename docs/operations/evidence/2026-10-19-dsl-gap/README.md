# **DSL 原语缺口清单**（从 5 个真规则候选反推，2026-10-19）

## 1 为什么做这件事

第 194 行把「待迁移量」收窄到 **5 个真规则候选**（全在 L11）。它们的 `content` 只含 `effect`，
**看起来**是 DSL 能表达的。本轮逐个比对**DSL 现有词汇表**，判定「能不能写成 DSL」。

### 1.1 DSL 现有词汇（从补全表与编译器关键字读出）

| 类别 | 可用 |
|---|---|
| **trigger 事件** | `damage`、`hp_lost`、`phase_prepare`、**`phase_begin`**、`phase_end`、`phase_draw_end`、`phase_play_end`、`phase_any_end`、`phase_draw_skipped`、`death` … |
| **effect** | **`recover_hp`**、**`draw_cards`**、**`lose_hp`**、**`damage`** |
| **query** | **`hp`**、**`max_hp`**、**`hand_count`**、**`cards_used_this_round`** |
| **结构** | `trigger <event> <filter> { … }`、**`modifier <target> <mode> <n>;`** |

## 2 逐个候选的缺口

| 技能 | 需要什么 | DSL 有/缺 | 缺口数 |
|---|---|---|---:|
| **`dz_gs_yinfu`**(300B) | `player.recover` ⇒ `recover_hp` | **有** | **3** |
| | `getDamagedHp()` ⇒ `max_hp - hp` 组合 | **有** | |
| | **`game.roundNumber`** ⇒ 无 `round` 查询 | **缺** | |
| | **`removeSkill`** ⇒ 无 `remove_skill` | **缺** | |
| | **`getAllHistory(...).length>=3`** ⇒ 无计数 | **缺** | |
| **`boss_danshu`**(368B) | `player.recover` | 有 | **1** |
| | **`player.judge()`** ⇒ 无 `judge` | **缺** | |
| **`boss_xixing`**(396B) | **`chooseTarget`** ⇒ 无 `choose_target` | **缺** | **2** |
| | **`isLinked`** ⇒ 无 `is_chained` | **缺** | |
| **`polu_damage`**(440B) | `player.draw` ⇒ `draw_cards` | 有 | **2** |
| | **`trigger.num`**（伤害点数）⇒ 无 | **缺** | |
| | **`get.cardPile2(...)`** ⇒ 无牌堆检索 | **缺** | |
| **`boss_tianyun`**(728B) | **`chooseTarget`** | **缺** | **2** |
| | **`forceDie`** ⇒ 无 `force_die` | **缺** | |

## 3 缺口汇总（**去重后 9 个**）

| # | 缺的原语 | 用途 | 出现于 |
|---:|---|---|---|
| 1 | **`round`**（当前轮数查询） | 条件 | `dz_gs_yinfu` |
| 2 | **`remove_skill`**（移除自身技能） | 次数上限 | `dz_gs_yinfu` |
| 3 | **`history_count`**（按事件/技能计数） | 次数上限 | `dz_gs_yinfu` |
| 4 | **`judge`**（判定 + 读花色） | 判定 | `boss_danshu` |
| 5 | **`choose_target`**（选目标） | 交互 | `boss_xixing`、`boss_tianyun` |
| 6 | **`is_chained`**（链状态查询） | 条件 | `boss_xixing` |
| 7 | **`trigger.num`**（事件数值参数） | 结算 | `polu_damage` |
| 8 | **`card_pile_search`**（按条件检索牌堆） | 结算 | `polu_damage` |
| 9 | **`force_die`**（强制死亡） | 结算 | `boss_tianyun` |

## 4 判读（**如实**）

1. **5 个候选全部至少缺 1 个原语** ⇒ **今天一个也写不成纯 DSL** ❗；
2. **缺的不是「复杂的宿主逻辑」**，而是 **9 个基础原语**（判定、选目标、事件参数、牌堆检索、
   计数、当前轮数）—— **其中 6 个在同类引擎里都是常规能力**；
3. **这改变了「阻塞」的性质**：此前记「需改 `kernel/**`（未跟踪）⇒ 阻塞」；
   **更准确的表述是「DSL 缺 9 个原语，补在 `content-compiler`/`rule-vm`（已跟踪）即可推进」** ✅
   —— **这两个 crate 是 tracked 的**，所以**这部分工作本代理本来就能做**！
4. **本轮未实现任何原语**（只做缺口分析）。

## 5 下一步（**可执行**）

1. **先补最通用的两个**：`trigger.num`（事件参数读取）与 `round`（当前轮数）——
   它们**只需 query 层扩展**，不改内核；
2. 再补 **`judge`**（判定）与 **`choose_target`**（选目标）—— 这两个需要**询问通道**，工作量较大；
3. 补完一个就**回头迁一个**（`boss_danshu` 只需 `judge` 一个原语 ⇒ **最短路径**）。

## 6 口径

- 词汇表来自**补全表与编译器关键字**（`lsp/completions/`、`content-compiler/src`），**未穷举**；
- 「缺」的判定基于**关键词搜索**，**不排除**存在未列入补全的隐藏原语；
- 未实际编译任何 DSL（本轮只比对词汇）。

---

## 7 ⚠ **更正**（同日）：DSL 原语路径**同样被未跟踪代码阻塞**

§4 第 3 条我写：「阻塞性质不是『需改 `kernel/**`』，而是『DSL 缺 9 个原语，补在
`content-compiler`/`rule-vm`（均已跟踪）即可推进』⇒ **这部分本代理本来就能做**」。
**这个判断是错的** ❌ —— 我只看了**编译端**是否 tracked，没看**求值端**。

### 7.1 实际调用链

```
DSL 关键字
  → content-compiler/codegen.rs       (tracked ✓)
  → rule-vm/bytecode.rs  query_tag::* (tracked ✓)
  → rule-vm/vm.rs  →  QueryRequest::*  (tracked ✓, crates/rule-api/src/query.rs 定义 **46 个变体**)
  → **host 侧 match QueryRequest**  ← 这里断掉
```

### 7.2 断点在哪里（**实测**）

| 文件 | `QueryRequest::` 臂数 | **`_ =>` catch-all** | tracked? |
|---|---:|---:|---|
| `crates/engine-native/src/rvm_ops.rs` | 42 | **7** | ✅ tracked |
| **`crates/engine-core/src/query/cache.rs`** | **41** | **0** ❗ | ❌ **未跟踪** |
| **`crates/engine-core/src/query/mod.rs`** | **16** | **0** ❗ | ❌ **未跟踪** |

⇒ `engine-core` 的这两处 match **没有 catch-all**，且**覆盖全部 46 个变体**（41 臂 + 多变体合并臂）。
⇒ **新增任何一个 `QueryRequest` 变体，都必然要让这两个未跟踪文件编译通过** ❌
⇒ **按仓库纪律不得修改未跟踪文件** ⇒ **DSL 原语扩展路径被阻塞**。

### 7.3 更正后的准确表述

| 说法 | 正误 |
|---|---|
| 「补原语只动 `content-compiler`/`rule-vm`（tracked）」 | ❌ **错**：求值端在 `engine-core`（未跟踪），且 **match 无 catch-all** |
| 「DSL 原语扩展被未跟踪代码阻塞」 | ✅ **对**：具体是 `engine-core/src/query/{cache.rs,mod.rs}` 的**穷举 match** |
| 「所以完全无法推进」 | ❌ **也不对**：**不需要新增变体**的缺口仍可能用**已有变体**组合表达（**未逐一核对**）|

### 7.4 下一步（**修正后**）

1. 对 9 个缺口**逐个**核对：能否用**已有 46 个变体**组合表达？
   - 例如 `is_chained`/`force_die` 可能已存在（**未查**）；`round` 可能无对应；
2. 只有「**必须新增变体**」的那些，才是**真正的未跟踪阻塞**；
3. 若确实需要新增变体，则**唯一出路**是把 `engine-core` 纳入可修改范围（**需人工决策**）。

---

## 8 逐项核对：9 个缺口 ↔ **已有 46 个 `QueryRequest` 变体**

`crates/rule-api/src/query.rs` 的 46 个变体（全文）：

```
Distance AttackRange CardUsable TargetLegal HandLimit SkillUsesRemaining CardAttribute
Relation Hp HandCount EquippedCount EmptyEquipSlots MaxHp Zone CardZone MarkCount
AliveCount CardActionCount AliveGroupCount UniqueMaxCurrentHp UniqueMaxCurrentHpAt
LifetimeDamageDealt PublicCardKindCount **RoundNumberCapped** FieldEquipmentKindCount
AliveWoundedOrGuimingWuCount HandEquipSuitCount CurrentPhasePlayer HandSuitCount
HandColorCount SkillContentFlag SkillPileCount CardsDrawnThisTurn DamageDealtThisTurn
HasSkill CardNameLength InPlayPhase CardHasTag StorageGet StorageHas CardsUsedThisRound
Int Bool Relation Error SkillPileCount
```

### 8.1 结果：**9 个缺口里 1 个已有对应**

| 缺口 | 已有变体？ | 说明 |
|---|---|---|
| **`round`**（当前轮数）| ✅ **`RoundNumberCapped`** | **但语义不同**：它是「**轮数被内容提供的上限截断**」⇒ 需传一个大上限才能近似「当前轮数」|
| `history_count` | ❌ 无 | |
| `is_chained`（链状态）| ❌ 无 | |
| `trigger.num`（事件数值参数）| ❌ 无 | |
| `remove_skill` | ❌ 无（**且这不是 query，是 effect**）| |
| `judge` | ❌ 无（**effect**）| |
| `choose_target` | ❌ 无（**effect**）| |
| `card_pile_search` | ❌ 无（**effect**）| |
| `force_die` | ❌ 无（**effect**）| |

### 8.2 更要紧的：**effect 派发也在未跟踪区**

| 文件 | tracked? |
|---|---|
| `crates/engine-core/src/effect/primitives/mod.rs`（派发表）| ✅ **tracked** |
| `crates/engine-core/src/effect/primitives/dispatch_*.rs`（**每个效果一个文件**）| ❌ **UNTRACKED** |

⇒ 9 个缺口里 **5 个是 effect**（`remove_skill`/`judge`/`choose_target`/`card_pile_search`/`force_die`），
它们要**新增效果派发** ⇒ 必然落在 **`dispatch_*.rs`（未跟踪）** ⇒ **同样被阻塞** ❌。

### 8.3 更正后的结论

| 缺口类型 | 数量 | 状态 |
|---|---:|---|
| **已存在**（`round` ⇒ `RoundNumberCapped`，语义近似）| **1** | ✅ 可用（需大上限）|
| **query 类，需新增变体** | **3**（`history_count`/`is_chained`/`trigger.num`）| ❌ 阻塞（`engine-core` 穷举 match）|
| **effect 类，需新增派发文件** | **5** | ❌ 阻塞（`dispatch_*.rs` 未跟踪）|
| **可纯 DSL 完成的** | **0** | — |

⇒ **5 个真规则候选里，`dz_gs_yinfu` 只差 `remove_skill`+`history_count`，`boss_danshu` 只差 `judge`**
—— **无一能在当前可修改范围内落地** ❌。**这不是分析不足，是边界** ✅。

### 8.4 仍可能的例外（**未验证**）

- 若某个 effect **已有派发**、只是 **DSL 关键字没接**（如 `damage`/`draw_cards` 已有 ✓），
  那么**只在 `content-compiler` 加关键字**即可 ⇒ **这类才是可做的**；
- 本轮**未逐一核对**「5 个 effect 是否已有派发」⇒ **下一步**：查 `dispatch_*.rs` 里是否已有 judge/choose_target 的实现。

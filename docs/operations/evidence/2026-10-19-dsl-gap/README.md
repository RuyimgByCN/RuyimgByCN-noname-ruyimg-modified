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
| `crates/engine-native-kernel/src/rvm_ops.rs` | 42 | **7** | ✅ tracked |
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

---

## 9 追踪：effect 缺口的**接线路径**（比 query 类宽）

### 9.1 已确认的事实

| # | 事实 | 依据 |
|---|---|---|
| 1 | **`rule-api` 里没有 effect 种类枚举** —— `effect.rs` 只有 6 个**区域选择**变体（`Equip`/`HandEquip`/`HandEquipJudge`/`Judge`/`EquipJudge`/`Hand`），**不是效果种类** | `crates/rule-api/src/effect.rs` |
| 2 | VM 的效果指令是**通用的 `Op::HostEffect(u16)`** —— 效果由**宿主按 tag 编号**处理，不是固定枚举 | `rule-vm/src/bytecode.rs:227` |
| 3 | tag 常量在 **`rule-vm`**（**tracked** ✅）| `effect_tag`（45 个常量）|
| 4 | DSL 效果接线在 **`content-compiler/src/codegen/effect_*.rs`** —— **混合跟踪**（`effect_resolve_wdb_yangyi.rs`、`effect_ban_active_until_turn_end.rs` 等**未跟踪** ❌）| `git ls-files` |
| 5 | query 类的 host 端 match **穷举且无 catch-all** ⇒ **新增 query 变体必然改未跟踪文件** ❌ | `engine-core/src/query/{cache.rs,mod.rs}` |
| 6 | effect 的 **host 处理位置未确认**（可能在 `engine-native/src/rvm_ops.rs` ✅ 或 `engine-core` ❌）| **未查** |

### 9.2 因此

| 缺口类型 | 能否在可修改范围落地 | 理由 |
|---|---|---|
| **query（3 个）**：`history_count`/`is_chained`/`trigger.num` | ❌ **不能** | 需新增 `QueryRequest` 变体 ⇒ 撞 §7.2 的穷举 match |
| **effect（5 个）**：`remove_skill`/`judge`/`choose_target`/`card_pile_search`/`force_die` | ⚠️ **待定** | tag 与 DSL 接线层**部分 tracked**；**host 处理位置未确认** ⇒ 若在 `rvm_ops.rs`（tracked）则**可做** |

### 9.3 本轮**没有**回答的问题

1. `judge`/`choose_target`/`remove_skill`/`card_pile_search`/`force_die` 在 **`engine-core` 里是否已有实现**（**部分可能已有** —— §8.4 的例外）；
2. `Op::HostEffect(tag)` 的 **host 实现在哪个 crate**（决定 5 个 effect 缺口是否可做）；
3. `effect_tag` 的 **45 个常量**里有没有 map 到 judge/choose_target 的（**未逐个读注释**）。

⇒ **下一步最省力的一步**：查 `Op::HostEffect` 的 host 实现落在哪个文件，以及 `effect_tag` 里是否已有 judge/choose_target 的 tag。

---

## 10 ⚠ 再更正：上一节的「arms/ca」测量**方法有误**

§9 我按 `grep -c 'EffectRequest::' 文件` 数「臂数」、按 `grep -cE '^\s+_ =>'` 数「catch-all」，
据此判断 `crates/rule-api/src/effect/choose_other_give_then_recover.rs`（**arms=3 / ca=0**）是「**穷举 match**」
⇒ 推出「新增 `EffectRequest` 变体会破坏未跟踪文件」。

**打开文件后发现**：它根本不是 matcher，而是**宏定义的输入片段**：

```
([$($tail:ident),*] [$($variants:tt)*]) => { … }     // macro_rules! 规则
/// Engine-owned prepared form of [`EffectRequest::DealDamageBanActiveIfDying`].   // 文档注释
```

⇒ 那 3 个「`EffectRequest::`」**全是文档注释里的链接** ❌ ⇒ **arms/ca 的统计是假的** ❌。

### 10.1 真实情况

| 项 | §9 的说法 | **实际** |
|---|---|---|
| `EffectRequest` 定义 | —— | **宏生成**（`rule-api/src/effect.rs:220`，**tracked** ✅）|
| 判定「哪些文件会因新增变体而编译失败」 | 「至少一个未跟踪文件无 catch-all」 | ❌ **未验证，且测量方法错误** |
| 真实 matcher 的 catch-all | —— | **未统计**（`resolve_qixing.rs` 等 100+ 臂文件确有 `ca=1`，但同文件可能还有别处 match）|

⇒ **「effect 5 个缺口是否可做」重新回到「未知」** ⚠️。

### 10.2 唯一可信的判定办法（**下一步**）

**做一次真实实验**（不改仓库永久状态）：

1. 在 `rule-api/src/effect.rs` 的变体列表里**加一个** `EffectRequest::__Probe`（或复用现成变体改一个字段）；
2. `cargo build --offline -p engine-core` —— **能编过 ⇒ effect 缺口可做**；**编不过 ⇒ 看报错文件是否未跟踪**；
3. **立即 `git checkout` 还原**（或只在临时分支上做）。

这是**唯一**能把「未知」变成结论的办法；继续用 grep 统计只会制造第二个假结论。

### 10.3 本轮方法论教训（**第三次同类**）

| 次数 | 假结论 | 原因 |
|---:|---|---|
| 1 | 「组合矩阵 130 个缺陷」 | 判据不适用 |
| 2 | 「179 个待迁移技能」 | 三个不同口径相加 |
| 3 | **「effect 缺口被未跟踪文件阻塞」** | **`grep` 把文档注释当成 match 臂** |

⇒ 共同点：**用文本统计代替结构判断**。**凡是「数 match 臂」这类问题，应直接看 AST/编译结果**。

---

## 11 ✅ **决定性实验**：新增 `EffectRequest` 变体 ⇒ **`engine-core` 编译失败**（2 处穷举 match）

不再用 grep 猜，直接做实验（**已完全还原**）。

### 11.1 做法

1. 在 `crates/rule-api/src/effect.rs` 的 `pub enum EffectRequest {` 后插入一个**探针变体** `__ProbeVariant`；
2. `cargo build --offline -p engine-core`；
3. **立即 `git checkout -- crates/rule-api/src/effect.rs` 还原**（`git status` 已确认为空）。

### 11.2 结果（原始日志：[probe-build.log](probe-build.log)）

```
exit=101
error[E0004]: non-exhaustive patterns: `rule_api::EffectRequest::__ProbeVariant` not covered
error[E0004]: non-exhaustive patterns: `&rule_api::EffectRequest::__ProbeVariant` not covered
error: could not compile `engine-core` (lib) due to 2 previous errors; 94 warnings emitted
```

报错指向 **`crates/engine-core/src/command/resolve_choice/…`**（`group_post_3`/`group_post_4`/`group_post_5`/`group_post_6`
下的多个文件；**均为未跟踪** ❌）。

### 11.3 结论（**确定**，不再是「未知」）

| 缺口类型 | 数量 | 能否在可修改范围落地 | **证据** |
|---|---:|---|---|
| **query 类**（`history_count`/`is_chained`/`trigger.num`）| 3 | ❌ **不能** | `engine-core/src/query/{cache.rs,mod.rs}` 穷举 match（41/16 臂、**0 catch-all**）|
| **effect 类**（`remove_skill`/`judge`/`choose_target`/`card_pile_search`/`force_die`）| 5 | ❌ **不能** | **本实验**：新增变体 ⇒ **2 处 `E0004`**，均在**未跟踪**的 `resolve_choice/**` |
| `round` | 1 | ✅ 可用近似 | `RoundNumberCapped`（需传大上限）|

⇒ **5 个真规则候选（`dz_gs_yinfu`/`boss_danshu`/`boss_xixing`/`polu_damage`/`boss_tianyun`）
在当前可修改范围内一个也无法迁移** ❌ —— 这是**边界**，不是分析不足 ✅。

### 11.4 顺带的方法论收获

- 本实验**一次就终结了三轮猜测**（§9 猜「无 catch-all」、§10 推翻、§11 定案）；
- **失败过的第一次尝试**：把锚点 `DealDamageBanActiveIfDying`（不存在）当作插入点 ⇒ 插到了文件首行 ⇒ **语法错误** ⇒
  **这次失败不是答案**（我差点把它当成「编译失败=阻塞」的证据 ✗）—— **必须区分「实验机制失败」与「实验对象失败」** ✅。

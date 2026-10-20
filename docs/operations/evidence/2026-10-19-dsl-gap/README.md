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

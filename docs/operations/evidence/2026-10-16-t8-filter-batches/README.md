# T8 具名组合 filter 分批清单（2026-10-16）

只整理可执行分批证据；**未改** `crates/**`、`native-progress.md`、台账正文。

关联：`T8-FILTER-COMPOSITE`（[todo-register](../2026-10-16-todo-register/README.md)）；分类账来源 [增量 17](../2026-10-14-t8-increment17-tied-max-filters/README.md)；机器可读 [`batches.json`](batches.json)。

## 1. 核实数字与来源

| 口径 | 数字 | 来源（2026-10-16 实读） |
|---|---|---|
| 词表总量 | **642** | `crates/content-compiler/src/typecheck.rs` · `FILTERS`（唯一、无重复） |
| 增量 17 扫描（历史快照） | **18 已承载 + 624 待办** | [增量 17 README](../2026-10-14-t8-increment17-tied-max-filters/README.md)；进度主稿 T8 行、todo-register 仍引用此快照 |
| **当前已承载（词表∩装载器精确映射）** | **23** | `content-native-loader::bridge_vm_skill_decls` 精确名臂 ∩ `FILTERS` |
| **当前待办** | **619** | 642 − 23；= 624 − 5（增量 17 当日落地的并列最大族 + `owner_targeted_by_other_sha`） |
| `VmFilterPolicy` 变体 | **28** | `engine-native/src/rvm_bridge.rs`（含参数化 `OwnerHpBelow`/`OwnerPlayCountEq` 等；参数化模板名**不在** 642 词表内） |
| 装载器有、词表无 | 2 | `owner_is_target_hand_empty`、`owner_source_play_card_count_eq_hp`（历史登记：`hand_empty` 不在编译器 FILTERS） |

**漂移说明**：进度主稿 / todo-register 的「约 18 / 624」是增量 17 **同日扫描、+5 映射写入前**的快照。映射落地后同词表应变为 **23 / 619**。本证据以当前仓库为准；族细分启发式与增量 17 公布数有少量漂移（见下表），族标签仍沿用增量 17 分类账。

### 当前已承载 23 名（FILTERS ∩ 装载器）

`always` · `owner_is_target` · `owner_source` · `owner_is_target_wounded` · `owner_is_target_linked` · `owner_is_target_hand_nonempty` · `owner_is_target_no_turn_damage` · `owner_is_target_turn_damage` · `owner_is_target_hand_below_max_hp` · `owner_is_target_hand_above_hp` · `owner_is_target_other_hand_nonempty` · `owner_is_target_turn_damage_at_least_three` · `owner_is_target_hand_at_least_hp_other_alive` · `owner_is_target_play_phase_card_category_count_positive` · `owner_alive_play_phase_card_category_count_positive` · `owner_is_target_card_used_at_least_hp` · `owner_target_hand_empty_elemental_or_trick_damage_before` · `owner_is_target_source_movable_field_card` · `owner_is_target_tied_max_hand` · `owner_is_target_tied_max_hp` · `owner_is_target_tied_max_equip_nonempty` · `owner_is_target_not_unique_max_current_hp` · `owner_targeted_by_other_sha`

相关测试面：`crates/engine-native/tests/t8_rvm_bridge.rs`（装载器锁面、双闸正负例、反例实证惯例）。

## 2. 待办按族（619）

| 族 | 增量 17 快照（624） | 本核实启发式（619） | 是否须先补内核原语 |
|---|---:|---:|---|
| 复合查询 | 461 | **447** | 多数否；含技能跟踪面/特定技名的子集可能要实例状态 |
| once 实例旗标 | 52 | **55** | **是** — `PRIM-ONCE` |
| 移动批次上下文 | 51 | **57** | **是** — `PRIM-MOVE-BATCH` |
| pot 族 | 24 | **25** | **是** — `PRIM-POT`（可复用 mark/pile） |
| 丢失历史 | 16 | **15** | **是** — `PRIM-LOST-HISTORY` |
| 技能牌堆 | 9 | **9** | **是** — `PRIM-SKILL-PILE` |
| 标记 | 9 | **9** | **是** — `PRIM-MARK` |
| 转化杀溯源 | 1 | **1** | **是** — `PRIM-CONVERT-SHA` |
| 实例记忆 | 1 | **1** | **是** — `PRIM-MEMORY` |

另：**参数化主动技计数** `owner_is_target_active_used_at_least_{N}_this_play_phase` 不在 642 具名表内，增量 15 已跳过登记 → 门闩 `PRIM-ACTIVE-USED`（属效果/查询变体与 filter 交界，勿与复合查询批混做）。

启发式规则（可复现）：`once`/`twice_per` → once；名含 `pot` → pot；`skill_pile` → 牌堆；`mark`/`falu`/`marked_` → 标记；`remember` → 记忆；`transformed_sha` → 转化杀；丢失/移动关键字表见生成脚本逻辑（与 `batches.json` 同源）。复合查询为剩余兜底；与增量 17 公布数的差额来自 +5 已承载消化 + 关键字边界（once/move 互斥优先级）。

## 3. 建议分批（摘要）

共 **60** 批，每批 **5–15** 项（末批可不足）；条目 ID 形如 `T8F-<dsl_name>`。细节与全表见 [`batches.json`](batches.json)。

| 批号段 | 族 | 优先级 | 新原语 | TurnState/快照/哈希 | 验收策略 |
|---|---|---|---|---|---|
| **B01–B04** | 复合查询（首批精选，47 项） | P0 | 否 | **unlikely** | `t8_rvm_bridge` 双闸 + 装载器锁面 + Gate 1b；预期无版本/golden 变化 |
| B05–B38 | 复合查询（其余 400） | P1 | 多数否；技能跟踪面个案 | case_by_case | 同上；触及回合计数才补 TurnState 断言；**新字段**才升存档/哈希 |
| B39–B44 | once（55） | P1 | **PRIM-ONCE 先** | likely | 旗标置位/作用域清除/耗尽负例 + 快照往返 |
| B45–B50 | 移动批次（57） | P1 | **PRIM-MOVE-BATCH 先** | 多为事件上下文 | 移动事件正负例、空批边界 |
| B51–B54 | pot（25） | P2 | **PRIM-POT 先** | likely 技能状态 | 按技能样本包；禁止无原语空映射 |
| B55–B56 | 丢失历史（15） | P1 | **PRIM-LOST-HISTORY 先** | likely TurnState+哈希 | 累加/清零 + 若入状态则存档往返 |
| B57 | 技能牌堆（9） | P2 | **PRIM-SKILL-PILE 先** | likely 区域快照 | 置入/取用/空堆 |
| B58 | 标记（9） | P2 | **PRIM-MARK 先** | likely 标记快照 | 阈值门 + 往返 |
| B59 | 转化杀（1） | P2 | **PRIM-CONVERT-SHA 先** | likely | 转化置位/非转化负例/跨回合 |
| B60 | 记忆（1） | P2 | **PRIM-MEMORY 先** | likely | 记忆匹配/恢复后仍匹配 |

以 `batches.json` 的 `family` / `item_ids` 为准。

### 必须先补内核原语才能做的批

- **B39–B60**：once / 移动批次 / pot / 丢失历史 / 牌堆 / 标记 / 转化杀 / 记忆 — 对应 `primitive_gates`（`PRIM-*`）。
- **复合查询批内**：名含 `tracked_` / `*_pending` / 强技能绑定、或语义依赖「主动技发动次数」者，不得在无原语时硬映射；应 defer 或先立项原语。
- **禁止**：未补原语时把上述族从 `deferred_filters`「假映射」成恒 true/false（违反增量 6 严格面与双闸惯例）。

### 首批建议（B01，13 项）

可立即开工、**无需新原语**、预期**不动** TurnState/快照/哈希：

1. `owner_alive`
2. `owner_is_lord`
3. `owner_is_target_current_turn`
4. `owner_alive_current_turn`
5. `owner_is_target_other_alive`
6. `owner_is_target_source_alive`
7. `owner_is_target_other_source_alive`
8. `owner_is_target_judge_nonempty`
9. `owner_phase_begin_hand_empty`
10. `owner_is_target_face_down`
11. `owner_turned_over`
12. `owner_is_target_hp_le_one`
13. `owner_is_target_hp_le_two`

验收：每策略双闸（正例过闸 / 负例不过闸）+ 装载器逐名锁面（不落 `deferred_filters`）+ `cargo test --offline -p engine-native --test t8_rvm_bridge` + Gate 1b。语义以 `content-compiler` lsp/`FILTERS` 权威描述为准。

## 4. 与 T8 效果/查询变体的边界

`T8-EFFECT-QUERY-VARIANTS`（todo-register）**不在本清单条目内**，但依赖本清单收敛：`deferred_filters` 下降后，开放规则验收面才谈得上「不重编译装新技能」。效果变体中需新原语的项（含 `PRIM-ACTIVE-USED`）单独立项，勿塞进 B01–B04。

## 5. 生成与复算

本目录数字由 2026-10-16 对照 `FILTERS` + `bridge_vm_skill_decls` 实算；族启发式与增量 17 公布数可能差数项，**工单以 `batches.json` 条目 ID 为准**，不以族计数为合同。若词表增删，重跑同样交集后再改批。

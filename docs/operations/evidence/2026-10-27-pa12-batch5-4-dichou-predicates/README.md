# PA12 批⑤-4：R01 涤仇谓词面原生化 + card_targeted 模块体 event_player 差异实测登记（2026-10-27）

- 执行时间：2026-10-27 **17:15:45 → 17:55 +08:00**（`HEAD = e0af1a73`；裁决问询未获答复按默认路线代理开工 R01；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) 批⑤ R01（UP37-R01 台帐行）；依赖面核实：`RESOLVE_DCDICHOU`(592) 为带选择状态机（§2 J 族），R01 的**谓词面**可独立小步先行——本批范围

**一句话结论**：R01 涤仇（xianding 包 `dcdichou.skill`）两个谓词触发段落原生：`owner_is_target_other_source_alive`（精确策略 `OwnerTargetOtherSourceAlive`，旧线 `OwnerIsEventPlayerOtherSourceAlive` 语义核对）与 **参数化族** `owner_source_target_mark_positive_{mark}`（`strip_prefix` 同旧线 map_filter 口径，`OwnerSourceTargetMarkPositive(u32)`）；两谓词均为 card_targeted→CardUsed+targets（增量 17 映射）做了 targets 集感知。真实 .skill 整局验收：他人对持有者用牌→来源挂仇（vm_aux.marks 落账）✓；持有者对带仇者用牌→谓词门命中+摸 1 ✓。**实测登记新差异（增量 17 扩展）**：模块体内 `event_player` 在该映射下=**用牌者**（旧线=被指定目标）——`remove_mark(event_player)` 移除的是用牌者的仇而非目标；逐目标帧型（per-target frame）随 §2 J 选择链批次评估。

## 1 谓词语义（旧线 dispatch 核对 → 原生）

| 滤名 | 旧线 | 原生（本批） |
|---|---|---|
| owner_is_target_other_source_alive | `owner_is_player && source_alive && !owner_is_source`（card_targeted 逐目标 ctx：player=target） | `owner∈{event_player ∨ targets(k=CardUsed)} ∧ 来源存活 ∧ 来源≠owner` |
| owner_source_target_mark_positive_{mark}（strip_prefix 参数化） | `owner_is_source && ctx.player 的 mark>0`（player=target） | `source==owner ∧ (event_player ∨ targets 任一) 的 mark>0`（vm_aux.marks） |

## 2 验收（真实 dcdichou.skill + xianding npack）

| 断言 | 结果 |
|---|---|
| 两 card_targeted 段（`dcdichou_card_targeted_0/_2`）按名过滤装载桥接 | 2/2 decls |
| 整包桥接（四段滤镜全承载） | 4/4 decls——装载器只核事件/滤镜；`resolve_dcdichou` 为效果层，运行期由 `accept_effect` 拒绝（VmFault 回滚），登记 §2 J |
| ① 0 号对 1 号用杀 → `owner_is_target_other_source_alive` 命中 → `add_mark(event_source)` | marks[(0, 3304378096)]=1 ✓ |
| ② 1 号对 0 号（带仇）用杀 → 参数化谓词命中 → 摸 1 | 手牌不减少（杀 −1 + 摸 1）✓ |
| ② 登记差异 | marks[(0,…)]=1 仍留——模块体 `event_player`=用牌者（下文 §3） |

## 3 新登记差异：card_targeted 模块体 event_player（增量 17 扩展）

- 旧线：card_targeted 逐目标发 TriggerContext（player=**目标**、source=用牌者）；DSL 模块内 `event_player`=目标。
- 原生：无独立 targeted 帧（增量 17），映射为单 CardUsed 事件（event_player=**用牌者**、targets 集）——模块体 `HOST_EVENT_PLAYER` 读到用牌者。谓词门经 targets 集补正（本批），但**模块体效果面**（如 `remove_mark(event_player)`、`draw_cards(event_player)`）仍指向用牌者。
- 处置：登记（计划 §2 增量 17 扩展条目）；彻底解法=per-target 帧型或 HostContext 逐目标注入——属 §2 J 选择链同批（多目标卡须逐目标窗）。R01 单目标杀的挂仇段（event_source=用牌者）不受影响。

## 4 变更与回归

- `rvm_bridge.rs`：两策略 + strip_prefix 参数化 + targets 集求值。
- 回归（[b54 汇总](../2026-10-26-pa12-dsl-native-prototype/b54-regressions.txt)）：五 crate 全量 + workspace `--all-targets` + 前端 + 三门禁（见提交信息）。

## 5 R01 剩余

`resolve_dcdichou`（phase_begin/phase_end 段，选择状态机 + 回合移除历史 `turn.removed_marks`）＝§2 J 族；`dcdichou_eff` 修正器（attack_range/sha_use_limit 按 owner_skill_mark）＝modifier 桥待接（R03 的 max_hp_gt 同族已接 content-schema，VM modifier 面待登记）。

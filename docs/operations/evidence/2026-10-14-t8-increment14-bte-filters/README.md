# T8 增量 14：出牌阶段三类计数 + `*_play_phase_card_category_count_positive` 双策略（2026-10-14）

## 任务来源

[T8 余项](../../../architecture/native-progress.md)「具名组合 filter 深化」第二梯队：词表中
`owner_is_target_play_phase_card_category_count_positive`（ABI 42）与
`owner_alive_play_phase_card_category_count_positive`（ABI 293）的权威语义
（lsp）为「出牌阶段结束时，该阶段内已使用 ≥1 张 logical basic/trick/equipment 牌」。
引擎侧无现成承载（`cards_used_play_phase` 为任意牌总数、不含类别面）——需新增
回合级计数字段，构成**字段批**（存档+哈希+golden 重录）。

## 交付

- **`CardKind::is_basic_trick_or_equip`**（catalog.rs）：三类判定——基本（is_basic_kind）
  ∨ 装备（equip_slot().is_some()）∨ 其余经出牌提交的种（反演排除地 10 种+机关牌）；
  扩展种未逐一归类按三类近似（保守集登记差异，同 is_basic_kind 先例）。
- **`TurnState.bte_used_play_phase: u32`**（state.rs）：`commit_card_used` 尾部按
  **结算牌种**（view-as 转化后，同上游 get.type）计数；Play 阶段开始清零（与
  cards_used_play_phase 同点）；装备牌走 Equipped 帧不经 commit_card_used，与
  cards_used_play_phase 同集（登记差异：上游 countUsed 是否含装备未逐项对照）。
- **哈希/存档**：字段入规范哈希（engine.rs push_u32）+ NGSA 编解码
  （**存档 v64→v65**）。
- **`VmFilterPolicy` 扩至 14 策略**：`OwnerPlayPhaseBteUsed`（ABI 42）/
  `OwnerAlivePlayPhaseBteUsed`（ABI 293）——合取=事件主玩家为持有者（ABI 293
  的持有者归因由「仅当前回合玩家可用牌」化归）∧ 事件为 Play 结束 ∧ 计数>0。
  `VmTriggerCtx` 增 `phase` 捕获（Phase 帧载荷）。
- **装载器映射** 2 名（content-native-loader）。
- **规则版本 277→278**。

## 字段批探针（golden 重录前置）

版本回退 277（保留字段）复跑 14 场景：**三值互异**（旧 277 无字段 / 277+字段
[probe-v277-field.log] / 278+字段 [engine-native-tests-golden-shift.log]）——
位移=版本字段+新增字段双源、无行为面扰动。标准场景不装载扩展 VM 技能，双策略
不在可达路径，逐场景命令与断言不变；14 哈希重录 + 补录注记
（replay_scenarios.rs）。

## 定向回归（t8_rvm_bridge +3 测，21/21）

- `t8_bte_play_phase_filter_dual_gate_and_phase_kind`：双策略（① 玩家 0 出杀
  bte=1 → 其 Play 结束事件主≠持有者不过闸；② 持有者出杀 → Play 结束触发摸 1；
  ③ 阶段种负例——弃牌阶段结束时计数仍 1 但事件非 Play 结束不再触发）。
  结算链时序实测：触发摸牌落队首、于弃牌之后结算（4+2−1−1+1=5）。
- `t8_bte_zero_count_negative`：不出牌直接结束（计数 0）不触发。
- `t8_bte_kind_classification`：三类判定单测——杀/酒/武器/防具/万箭/乐计入；
  地 10 种+机关牌不计。
- 装载器锁面扩至 12 名（deferred_filters 空 + 逐名策略映射断言）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t8_rvm_bridge **21/21** |
| engine-native 全量 | **721/0/0**（重录后）——[日志](engine-native-tests.log)；重录前 720/0/1（唯一失败=14 golden 预期位移）——[日志](engine-native-tests-golden-shift.log) |
| 装载/包管理/服务端 | content-runtime + content-native-loader + game-server **84/0** |
| Gate 1b | exit 0（独立 CARGO_TARGET_DIR）——[日志](gate1b-all-targets.log)（拦截 t8 测试文件未用导入已修） |

## 登记边界

- 装备牌不计入 bte 计数（Equipped 帧不经出牌提交）——与 `cards_used_play_phase`
  同集；上游 countUsed 对装备的口径未逐项对照，登记差异。
- 三类判定对未归类扩展种取「非异类即三类」保守近似（机关/地显式排除）。
- ABI 42/293 的 once 旗标面（若有）未建模——词表语义仅计数面。

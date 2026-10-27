# PA12 迁移批④-1：宿主原语 `turn_over` 接入原生（2026-10-27）

- 执行时间：2026-10-27 **12:17:02 → 12:50 +08:00**（`HEAD = 365fa0e1`；裁决问询未获答复，按计划顺序小步代理开工——批④「宿主原语」首个可独立验收原语；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) §3 批④；技能级重盘（本批）修正了文件级近似的口径：**宿主解析原语 Top-13 全部映射到不在原生允许面的 `EffectRequest` 变体**（`discard_hand_equip_count` 80 技能、`choose_other_damage` 48、`prevent/modify_current_damage` 79、`choose_use_virtual_sha` 33、`turn_over` 25、`discard_selected` 23、`storage_set` 22…）；`SkipCurrentPhase`/`UseCard` 无内核落点（rvm_ops 头注登记）→ 选**有现成内核提交路径**的 `turn_over` 作批④首项。

**一句话结论**：DSL `turn_over(player)`（tag `TURN_OVER` → `EffectRequest::TurnOver`，旧线「武将牌翻面：背面跳过下个回合后翻回」）接入原生：`accept_effect` 允许面 + `commit_effect` 以 `TurnOverBefore` 子事件帧走**内核翻面提交**（与 `DealDamage` 同法）——神裔「不能翻至背面」守卫、急冻冰封解除、`TurnOvered` 事件全部同口径，无新内核代码。真实技能全链：雪堕（diy，`damage owner_is_source_or_target optional` → 事件主玩家摸 2 + 翻面）——持有者受伤翻至背面、再受伤翻回、第三次翻背面后其下个回合到达时已被 E03 跳过路径翻回（`commit_phase`：PhaseBegin(Judge) ∧ face_down → 翻回 + 直达回合收束）。

## 1 验收

| 测试（`pa12_batch4_1_host_primitives.rs` 1/1） | 断言 |
|---|---|
| 雪堕全链 | 三次受伤：face 正→背→正→背；确认门（optional）逐次结算；第三次翻背面后推进到 1 号回合首次出现时 `face_down==false`（跳过路径已执行翻回） |

**驱动注记（登记）**：单元驱动在 E03 跳过窗口内注入 `NextPhase` 会与待提交的 `TurnEnded` 子帧竞争（实测窗口内可见 phase=Judge 且 face 已翻回、随后驱动注入会推进 Judge→Draw 打断整回合跳过）——故以「回合到达时已翻回」为跳过路径执行判据；**整回合无阶段帧**的全量验证属内核 E03 既有测试面，不在本批重复。

## 2 变更与回归

- `crvm_ops.rs`：允许面 `TurnOver`（座位校验）+ `commit_effect` 臂（`ctx.emit(TurnOverBefore{player, cancelled:false})`，头注更新）。
- 盘点脚本：`turn_over` 移入已承载集（近似 1,098 不变——`turn_over` 技能多有其它阻塞原语；本批价值在原语通道打通与真实技能验收）。[技能级重盘](skill-level-survey.txt)：宿主原语按技能数 Top-13 及包分布（sp 144/mobile 107/shahai 88/yijiang 88/xianding 86…）。
- 回归（[b41 汇总](../2026-10-26-pa12-dsl-native-prototype/b41-regressions.txt)，12:44–12:48）：engine-native **1577/0/71**、loader 32/0、wasm 52/0、game-server 358/0、engine-core 3841/0/1；其余门禁见提交信息。

## 3 后续（批④余项排序依据）

| 原语（技能数） | 落点难度 | 备注 |
|---|---|---|
| prevent/modify_current_damage（79） | 中 | DamageBefore 窗口改写已有内核机制（v187 先例） |
| discard_hand_equip_count（80） | 低 | `DiscardHandEquipCount` 变体已在 rule-api；补允许面+确定性弃牌臂 |
| choose_other_damage（48）/choose_use_virtual_sha（33） | 高 | 选择挂起链（HOST_CHOICE 后置） |
| discard_selected（23） | 中 | 需机器 selected 输入面 |
| storage_set（22）/remove_skill_by_def（22） | 低-中 | vm_aux.flags / 目录 def 查删 |
| SkipCurrentPhase/UseCard | 高 | 无内核落点（登记） |

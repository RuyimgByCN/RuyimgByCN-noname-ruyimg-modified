# PA12 迁移批④-2：宿主原语 `discard_hand_equip_count`（N>0 字面路径）接入原生（2026-10-27）

- 执行时间：2026-10-27 **13:03:44 → 13:30 +08:00**（`HEAD = d95d0b55`；裁决问询未获答复，按计划顺序小步代理开工；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) §3 批④第二原语

**一句话结论**：DSL `discard_hand_equip_count(player, N)`（tag 375 → `EffectRequest::DiscardHandEquipCount`，旧线语义：**N 为字面正数时确定性弃 N 张（手牌优先、装备补足、不足尽弃）；N==0 是「VM 首发——打开选牌窗口」**）。本批接入 N>0 路径：`accept_effect` 允许（座位校验 + **N=0 显式拒绝**——选牌挂起链未接，不静默）+ `commit_effect` 与既有 `DiscardFromHandEquip` 同口径落账（`discard_some`：弃牌堆 + `CardDiscarded` 日志 + `CardsDiscarded` 事件）；**N=0 选牌首发留批④选择链接入**。盘点口径注记：文件级近似 1,098 不变（`discard_hand_equip_count` 技能大多伴其它未承载原语；技能级重盘见[批④-1 附表](../2026-10-27-pa12-batch4-1-turn-over/skill-level-survey.txt)：该原语阻塞 80 技能，多为组合阻塞）。

## 1 验收（真实 ported .skill 全链 3/3 含批④-1 回归）

| 测试（`pa12_batch4_1_host_primitives.rs`） | 断言 |
|---|---|
| `pa12_batch4_2_discard_hand_equip_count`（消面，collab，`card_used owner_alive optional` 摸 1 后弃 1） | 持有者自己出杀 → optional 确认门 → 摸 1 → 弃 1（手牌净 −1：−1 出 +1 摸 −1 弃） |
| `pa12_batch4_2_accept_rejects_zero_count` | `DiscardHandEquipCount{count:0}` → `accept_effect` Err（N=0 选牌首发拒绝、不静默空成功）；`count:1` → Ok |
| `pa12_batch4_1_turn_over_toggles_face`（回归） | 批④-1 雪堕翻面三连 + E03 跳过路径不受影响 |

**测试侧缺陷（本批顺手修，登记）**：测试助手 `settle` 此前对非确认类询问只答首选项——`discard_to_limit`（SelectCards min=2）逐次 `InvalidChoice`（accepts 长度校验）导致驱动死循环；已改为按 min 取 Card 选项（与批② `settle_plain` 同法）。后续子批沿用。

## 2 变更与回归

- `rvm_ops.rs`：允许面（N=0 拒绝）+ commit 臂（复用 `discard_some`，与 `DiscardFromHandEquip` 完全同口径）；头注更新。
- 回归（[b42 汇总](../2026-10-26-pa12-dsl-native-prototype/b42-regressions.txt)，13:24–13:28）：engine-native **1579/0/71**（+2）、loader 32/0、wasm 52/0、game-server 358/0、engine-core 3841/0/1。

## 3 语义勘误（2026-10-27 14:30 复核旧线实现后登记，批④-4 勘察产物）

复核 engine-core `discard_hand_equip_count`（pack_xianding_2/resolve_stdgaobian.rs:596 + resolve_choice 续行）后修正本批对旧线语义的描述：

- **N>0 的旧线语义是「打开选恰好 N 张手/装备牌的选择窗（min=max=N），玩家选牌，续行按所选弃置」**——不是确定性首 N。原生批④-2 落的是**确定性首 N**（与 DiscardFromHandEquip 同口径）：当且仅当玩家总选「前 N 张」时两线一致；这是**登记差异（同迁移计划 §2 H 类）**，不是缺陷——原生 VM 桥的两层提交协议（VM Halt 后统一落账）没有「效果落地时开窗挂起、答后续跑」的挂起点。
- **N==0 在旧线是空操作**（`if count == 0 … return Ok(Vec::new())`），并非 rule-api 文档注释所说「VM 首发（打开选牌）」——该注释描述的是另一层（选择链重发），对本 DSL 原语不适用。原生批④-2 对 N=0 显式拒绝（比旧线空操作更严格），保持。
- **「玩家选牌」语义要在原生表达需要「效果级选择窗挂起」基建**（不同于 VM 内 choose() 的 token 选择）：登记为批④选择链基建项（连带 ChooseDiscardEquipmentPreventCurrentDamage / choose_other_damage / choose_use_virtual_sha 全族）。在此基建落地前，涉及「玩家选哪几张」的技能在原生按确定性首 N 近似运行，逐技能登记差异。

## 4 后续

- N=0 选牌首发（VM 交互选牌链）与 `prevent/modify_current_damage`（79 技能，DamageBefore 窗口改写）为批④下一优先；编译器对 count 字面 0 的态度（拒绝/放行）随选牌链一并验收。

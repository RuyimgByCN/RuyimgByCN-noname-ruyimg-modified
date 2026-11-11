# `L06 wuxing`（五行）全源 3/3 项独立复核报告（2026-11-12）

- **复核人**：`l06-verifier`（独立复核 agent；未编写或修改任何 `L06` 代码或测试，严格遵守裁决 40 第三节第 4 条“实现者不得参与复核”）
- **复核日期**：`2026-11-12`
- **上游来源**：`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js`（共 **3** 项实体：1 张卡牌 `wuxingpan` + 2 项技能 `_shengke`、`wuxingpan_skill`）
- **原生实现**：[`packages/extensions/wuxing-upstream-liuli/assets/cards.json`](../../../../packages/extensions/wuxing-upstream-liuli/assets/cards.json)、[`crates/content-native-loader/src/decision34_pack9.rs`](../../../../crates/content-native-loader/src/decision34_pack9.rs)、[`crates/content-native-loader/src/host_skill_bindings.rs`](../../../../crates/content-native-loader/src/host_skill_bindings.rs)、[`crates/content-native-loader/src/shahai_attach.rs`](../../../../crates/content-native-loader/src/shahai_attach.rs)、[`crates/engine-native-content/src/content/host/filtering/common.rs`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs)、[`crates/engine-native-content/src/content/host/activation/common.rs`](../../../../crates/engine-native-content/src/content/host/activation/common.rs)、[`crates/engine-native-content/src/content/host/asks.rs`](../../../../crates/engine-native-content/src/content/host/asks.rs)、[`crates/engine-native-content/src/content/host/step/skills_status.rs`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs)、[`crates/engine-native-content/src/content/host/step/skills_card_use.rs`](../../../../crates/engine-native-content/src/content/host/step/skills_card_use.rs)、[`crates/engine-native-kernel/src/archive/resolution.rs`](../../../../crates/engine-native-kernel/src/archive/resolution.rs)、[`crates/engine-native/src/ai/extension_choices.rs`](../../../../crates/engine-native/src/ai/extension_choices.rs)
- **真实对局测试**：[`crates/engine-native/tests/l06_wuxing_eight.rs`](../../../../crates/engine-native/tests/l06_wuxing_eight.rs)、[`crates/engine-native/tests/g1_wuxing.rs`](../../../../crates/engine-native/tests/g1_wuxing.rs)、[`crates/engine-native/tests/nc08_upstream_shengke.rs`](../../../../crates/engine-native/tests/nc08_upstream_shengke.rs)
- **版本门禁**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`（零升号，复用既有 `HostBehavior + HostStage + ChoiceKind` 选择通道，14 场 golden 哈希零变化）
- **复核结论**：**3/3 项 100% 通过独立复核（`已复核等价` 3 项，`简化近似` 0 项，`缺失` 0 项）**；全仓 [`docs/migration/g0/inventory_status.json`](../../../migration/g0/inventory_status.json) 累计 `已复核等价` 由 `50` 增至 **`53`**，`简化近似` 由 `4,374` 降至 **`4,371`**。

---

## 1. 逐项上游源码 vs 原生实现对照表（3/3 项 100% 覆盖）

| 序号 | 清单键 (`inventory_status.json`) | 类别 | 实体 ID | 上游源码 `file:line` 与核心语义（时机 / 目标与卡牌过滤 / 选择窗口 / 效果 / AI 公开态） | 原生实现位置 (`file:line`) | 六维真实对局测试与确定性种子 (`seed`) | 独立复核结论 |
|---:|---|---|---|---|---|---|---|
| 1 | `L06:cards:wuxingpan@archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:196` | `cards` | `wuxingpan` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:7, 196-201`：五行盘（`['spade', 5, 'wuxingpan']`，`type: 'equip', subtype: 'equip5'` 宝物栏装备，附带装备技 `skills: ['wuxingpan_skill']`）；装备入宝物栏后授予持有者 `wuxingpan_skill`，离场/弃置时自动撤销 `wuxingpan_skill` | [`cards.json:7`](../../../../packages/extensions/wuxing-upstream-liuli/assets/cards.json#L7) · [`decision34_pack9.rs:252,258-259,493-494,585`](../../../../crates/content-native-loader/src/decision34_pack9.rs#L252) · [`shahai_attach.rs:90-91`](../../../../crates/content-native-loader/src/shahai_attach.rs#L90-L91) (`CardKind::WuxingPan`, `EquipSlot::Treasure`, `Suit::Spade, 5`, `equip_skills: vec!["wuxingpan_skill"]`) | [`l06_wuxing_eight.rs:439-692`](../../../../crates/engine-native/tests/l06_wuxing_eight.rs#L439-L692) (`l06_wuxingpan_and_skill_six_dimension_real_match_equivalence`, `seed = 6200`) + [`g1_wuxing.rs:255-278`](../../../../crates/engine-native/tests/g1_wuxing.rs#L255-L278) (`wuxingpan_card_equips_via_treasure_slot`, `seed = 3303`) | **已复核等价**（无语义差异） |
| 2 | `L06:skills:wuxingpan_skill@archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:173` | `skills` | `wuxingpan_skill` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:173-193`：五行（`enable: 'phaseUse'`, `usable: 1`, `filterCard: true` 默认手牌 `'h'` 且 `selectCard: 1`, `lose: false` 不弃置所选牌）；Stage 0 选择 1 张手牌；Stage 1 `player.chooseControl('metal', 'wood', 'water', 'fire', 'soil')` 选择五种五行属性之一，将该手牌的 `wunature` 永久更新为所选属性 | [`host_skill_bindings.rs:325`](../../../../crates/content-native-loader/src/host_skill_bindings.rs#L325) · [`activation/common.rs:270-284`](../../../../crates/engine-native-content/src/content/host/activation/common.rs#L270-L284) · [`asks.rs:217-237`](../../../../crates/engine-native-content/src/content/host/asks.rs#L217-L237) · [`skills_card_use.rs:6-62`](../../../../crates/engine-native-content/src/content/host/step/skills_card_use.rs#L6-L62) (`HostBehavior::ChangeCardNature` + `HostStage::{SelectCard,SelectNature,Act}`, `WUXING_SELECT_CARD` + `WUXING_SELECT_NATURE` `Tag(0..=4)`) | [`l06_wuxing_eight.rs:439-692`](../../../../crates/engine-native/tests/l06_wuxing_eight.rs#L439-L692) (`l06_wuxingpan_and_skill_six_dimension_real_match_equivalence`, `seed = 6200`) + [`g1_wuxing.rs:203-251`](../../../../crates/engine-native/tests/g1_wuxing.rs#L203-L251) (`wuxingpan_skill_change_nature_with_restore`, `seed = 3302`) | **已复核等价**（无语义差异） |
| 3 | `L06:skills:_shengke@archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62` | `skills` | `_shengke` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-172`：全局锁定技（`_` 前缀，`trigger: { target: 'useCardToBegin' }`, `forced: true`, `filter: event.card.wunature && player.wunature`）；当成为带五行属性卡牌的目标时：① **相克**（金克木 `0->1`、木克土 `1->4`、水克火 `2->3`、火克金 `3->0`、土克水 `4->2`）：若 `player.countCards('he') > 0` 则发起强制弃牌 `player.chooseToDiscard(true, 'he')`（`ai = get.disvalue`，覆盖手牌与装备区，弃装备同步撤销装备技）；② **相生**（金生水 `0->2`、木生火 `1->3`、水生木 `2->1`、火生土 `3->4`、土生金 `4->0`）：目标摸 1 张牌（`player.draw()`）；③ 无生克关系或任一方无属性：无效果；AI 仅凭公开/己方可见牌估值弃最低价值牌 | [`host_skill_bindings.rs:324`](../../../../crates/content-native-loader/src/host_skill_bindings.rs#L324) · [`filtering/common.rs:557-561`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L557-L561) · [`asks.rs:274-280`](../../../../crates/engine-native-content/src/content/host/asks.rs#L274-L280) · [`skills_status.rs:677-693,1002-1035`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs#L677-L693) (`HostBehavior::ShengkeCheck` + `HostStage::{Act,SelectCard}`, `"shengke_discard"`) · [`resolution.rs:254`](../../../../crates/engine-native-kernel/src/archive/resolution.rs#L254) · [`extension_choices.rs:243-259`](../../../../crates/engine-native/src/ai/extension_choices.rs#L243-L259) | [`l06_wuxing_eight.rs:103-429`](../../../../crates/engine-native/tests/l06_wuxing_eight.rs#L103-L429) (`l06_shengke_six_dimension_real_match_equivalence`, `seeds = 6100..6104, 6110, 6120..6124, 6130`) + [`nc08_upstream_shengke.rs:250-401`](../../../../crates/engine-native/tests/nc08_upstream_shengke.rs#L250-L401) (`seeds = 20262801..20262807`) | **已复核等价**（无语义差异） |

---

## 2. 六维度真实对局事件路径核验摘要

独立复核了 [`crates/engine-native/tests/l06_wuxing_eight.rs`](../../../../crates/engine-native/tests/l06_wuxing_eight.rs) 中的两个六维度测试函数（全程通过 `Engine::submit(PlayCard / ActivateSkill / NextPhase / Answer)` 推进真实阶段与事件，零 `debug_execute_effect` 或伪造触发器调用）：

1. **维度 ①（触发时机与负条件）**：
   - `_shengke`：遍历全部 5 组相生（`0->2, 1->3, 2->1, 3->4, 4->0`）目标摸 1 张牌且无 `trigger_confirm` / `shengke_discard`；遍历全部 5 组相克（`0->1, 1->4, 2->3, 3->0, 4->2`）在卡牌生效前挂起 `shengke_discard`；验证无生克关系（`0->4`）、卡牌无属性、目标角色无属性、以及被克但目标 `he` 区为空等 4 类负例均不弹窗、不摸不弃；
   - `wuxingpan` & `wuxingpan_skill`：装备 `wuxingpan` 入宝物栏授予 `wuxingpan_skill`；空手牌（仅装备区有牌）时发动 `wuxingpan_skill` 被 `Rejected`；同一出牌阶段第 2 次发动被 `Rejected(RejectReason::SkillAlreadyUsed)`（`usable: 1`）。
2. **维度 ②（目标与卡牌过滤及数量限制）**：
   - `_shengke`：`shengke_discard` 候选同时包含目标手牌（`hand_tao`）与装备区牌（`pan`），`min: 1, max: 1, allow_cancel: false`；
   - `wuxingpan_skill`：Stage 1 `wuxing_select_card` 候选仅包含手牌（`card_a`, `card_b`），不含装备区的 `pan`；Stage 2 `wuxing_select_nature` 候选为 `Tag(0..=4)` 五种属性。
3. **维度 ③（玩家选择分支与强制拒空）**：
   - `_shengke`：空选 `vec![]` 与多选 `vec![hand_tao, pan]` 均返回 `Rejected(RejectReason::InvalidChoice)`；选弃手牌保留装备与 `wuxingpan_skill`，选弃装备区 `pan` 则移除装备并同步撤销 `wuxingpan_skill`；
   - `wuxingpan_skill`：Stage 1 选 `card_a` vs `card_b`、Stage 1 取消选牌、Stage 2 遍历全部 `Tag(0..=4)` 五种属性分支，均精确更新对应手牌的 `card_natures` 且不消耗手牌（`lose: false`）。
4. **维度 ④（多步状态变更联动）**：
   - `_shengke` 在 `useCardToBegin`（【杀】伤害结算前）先完成弃牌，再继续结算【杀】造成 1 点伤害；
   - 装备 `wuxingpan` -> 发动 `wuxingpan_skill` 将手牌【杀】改为火属性（`Tag(3)`）-> 对金属性（`0`）角色打出该【杀】-> 真实触发火克金 `shengke_discard` 并继续结算伤害。
5. **维度 ⑤（挂起窗口中点存档恢复）**：
   - 在 `shengke_discard`、`wuxing_select_card`、`wuxing_select_nature` 三个挂起窗口中点分别执行 `encode_archive(&e.save()) -> decode_archive -> Engine::restore_with_extensions` 后提交相同 `Answer`，恢复前后 `state_hash()` 与 `log()` 100% 一致。
6. **维度 ⑥（AI `BoundedPlanner` 仅凭公开/己方视图决策）**：
   - 在 `shengke_discard` 窗口，AI 按 `extension_card_value`（对齐上游 `ai = get.disvalue`）优先弃置低价值装备牌并保留高价值手牌【桃】；在 `wuxing_select_card` 与 `wuxing_select_nature` 窗口均生成合法应答，全程不读取暗置身份或他人暗牌。

---

## 3. 核验命令与实际运行结果

```bash
cargo test --offline -p engine-native --test l_rounds l06_wuxing_eight
# running 2 tests
# test l06_wuxing_eight::l06_wuxingpan_and_skill_six_dimension_real_match_equivalence ... ok
# test l06_wuxing_eight::l06_shengke_six_dimension_real_match_equivalence ... ok
# test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out

cargo test --offline -p engine-native --test g1 g1_wuxing
# running 5 tests
# test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 175 filtered out

cargo test --offline -p engine-native --test nc08_upstream_l_z nc08_upstream_shengke
# running 5 tests
# test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 204 filtered out

python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py
# Ran 2 tests in 0.011s - OK
# Source line limit passed: 9278 files <= 2000 lines; Rust non-test files <= 1000 effective lines
```

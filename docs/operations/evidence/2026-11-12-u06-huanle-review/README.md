# U06 欢乐卡牌（11/11 项）独立复核证据（2026-11-12）

- **核验日期**：2026-11-12
- **独立复核员（Reviewer）**：`u06-verifier`（与实现/收尾 agent `u06-author` / `u06-finisher` 严格职责分离，未编写或修改任何 `U06` 业务或测试代码）
- **来源范围**：`U06 欢乐卡牌`（上游目录：`archive/upstream/noname/apps/core/extension/欢乐卡牌/`，共 **11** 项：6 张卡牌 `monkey`、`mianju`、`shoulijian`、`kuwu`、`xuelunyang`、`jiuwei` + 5 项装备技能 `monkey`、`mianju`、`kuwu`、`xuelunyang`、`jiuwei`）
- **版本门禁**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`（零升号，复用 `HostBehavior + HostStage + ChoiceKind` 与 `MonkeyStealMachine` 交互通道）
- **复核结论**：**11/11 项 100% 通过独立复核（`已复核等价` 11 项，`简化近似` 0 项，`缺失` 0 项）**；全仓 `inventory_status.json` 累计 `已复核等价` 由 `3` 增至 **`14`**，`简化近似` 由 `4,421` 降至 **`4,410`**。

---

## 1. 逐项上游源码 vs 原生实现对照表（11/11 项 100% 覆盖）

| 序号 | 清单键 (`inventory_status.json`) | 类别 | 实体 ID | 上游源码 `file:line` 与核心语义 | 原生实现位置 (`file:line`) | 六维真实对局测试与确定性种子 (`seed`) | 独立复核结论 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `U06:cards:monkey@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:4` | 卡牌 | `monkey` | `card.js:4-15`、`list.js:2`：宝物牌（`type: "equip", subtype: "equip5"`），花色点数为方片 5（`["diamond", "5", "monkey"]`），伴随装备技 `["monkey"]`，`equipValue: 8` | [`cards.json:35-39`](../../../../packages/extensions/huanle-upstream-noname/assets/cards.json#L35-L39)、[`card_skills.rs:131-133`](../../../../crates/engine-native-kernel/src/vocabulary/card_skills.rs#L131-L133)、[`card_skills.rs:373`](../../../../crates/engine-native-kernel/src/vocabulary/card_skills.rs#L373) | [`u06.rs:514-670`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L514-L670) (`u06_monkey_six_dimension_real_match_equivalence`, seeds `3601..3603`)、[`u06.rs:10-162`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L10-L162) (`seed=3402`) | **已复核等价**（`difference: "无"`） |
| 2 | `U06:skills:monkey@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:5` | 技能 | `monkey` | `skill.js:5-31`：`trigger: { global: "useCardToBegin" }`，当其他角色（`event.player !== player`）使用【桃】（`event.name === "tao"`）且装备区有【猴子】时可选发动；`check`: `get.attitude(player, event.player) <= 0`；`content`: 取消【桃】结算（`trigger.untrigger(); trigger.finish()`），弃置装备区【猴子】并获得该【桃】 | [`monkey.rs:8-128`](../../../../crates/engine-native-kernel/src/kernel/monkey.rs#L8-L128)、[`play_card.rs:696-734`](../../../../crates/engine-native/src/engine/play_card.rs#L696-L734)、[`filtering/common.rs:670-678`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L670-L678)、[`skills_status.rs:939-990`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs#L939-L990)、[`ai.rs:710-753`](../../../../crates/engine-native/src/ai.rs#L710-L753) | [`u06.rs:514-670`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L514-L670) (`u06_monkey_six_dimension_real_match_equivalence`, seeds `3601..3603`) | **已复核等价**（`difference: "无"`） |
| 3 | `U06:cards:mianju@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:16` | 卡牌 | `mianju` | `card.js:16-35`、`list.js:8`：防具牌（`type: "equip", subtype: "equip2"`），花色点数为梅花 3（`["club", "3", "mianju"]`），伴随装备技 `["mianju"]`；`equipValue`: 未翻面时 `6`，已翻面（背面朝上）时 `-10` | [`cards.json:46-50`](../../../../packages/extensions/huanle-upstream-noname/assets/cards.json#L46-L50)、[`card_skills.rs:141-143`](../../../../crates/engine-native-kernel/src/vocabulary/card_skills.rs#L141-L143)、[`card_skills.rs:375`](../../../../crates/engine-native-kernel/src/vocabulary/card_skills.rs#L375)、[`ai.rs:761-779`](../../../../crates/engine-native/src/ai.rs#L761-L779) | [`u06.rs:675-739`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L675-L739) (`u06_mianju_six_dimension_real_match_equivalence`, seed `3604`) | **已复核等价**（`difference: "无"`） |
| 4 | `U06:skills:mianju@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:32` | 技能 | `mianju` | `skill.js:32-52`：`trigger: { player: "turnOverBefore" }, forced: true, equipSkill: true`，`content`: `trigger.cancel()`（锁定拦截装备者本人一切翻面：正面不可翻至背面、背面不可翻回正面，不影响其他角色翻面） | [`filtering/common.rs:705-710`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L705-L710)、[`metadata/common.rs:951-958`](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs#L951-L958)、[`skills_status.rs:991-997`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs#L991-L997) | [`u06.rs:675-739`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L675-L739) (`u06_mianju_six_dimension_real_match_equivalence`, seed `3604`) | **已复核等价**（`difference: "无"`） |
| 5 | `U06:cards:shoulijian@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:36` | 卡牌 | `shoulijian` | `card.js:36-80`、`list.js:6-7`：基本牌（方片 4、黑桃 4 共 2 张），`outrange: { global: 2 }`（仅限距离 > 1 的其他角色），`selectTarget: 1`；若目标 `he` 区有装备牌则发起可选弃置 1 张装备牌（`chooseToDiscard("he", { type: "equip" })`），取消或无装备牌则受到使用者造成的 1 点普通伤害 | [`cards.json:23-33`](../../../../packages/extensions/huanle-upstream-noname/assets/cards.json#L23-L33)、[`play_card.rs:761-773`](../../../../crates/engine-native/src/engine/play_card.rs#L761-L773)、[`asks.rs:239-263`](../../../../crates/engine-native-content/src/content/host/asks.rs#L239-L263)、[`support.rs:613-633`](../../../../crates/engine-native-content/src/content/host/support.rs#L613-L633)、[`skills_damage.rs:570-637`](../../../../crates/engine-native-content/src/content/host/step/skills_damage.rs#L570-L637)、[`ai.rs:761-796`](../../../../crates/engine-native/src/ai.rs#L761-L796) | [`u06.rs:743-878`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L743-L878) (`u06_shoulijian_six_dimension_real_match_equivalence`, seeds `3605..3606`) | **已复核等价**（`difference: "无"`） |
| 6 | `U06:cards:kuwu@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:81` | 卡牌 | `kuwu` | `card.js:81-96`、`list.js:5`：武器牌（`type: "equip", subtype: "equip1"`，黑桃 6，`distance: { attackFrom: -1 }` 即攻击范围 2），伴随装备技 `["kuwu"]`，`equipValue: 6` | [`cards.json:12-21`](../../../../packages/extensions/huanle-upstream-noname/assets/cards.json#L12-L21)、[`host_skill_bindings.rs:119`](../../../../crates/content-native-loader/src/host_skill_bindings.rs#L119) | [`u06.rs:883-1012`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L883-L1012) (`u06_kuwu_six_dimension_real_match_equivalence`, seeds `3607..3608`)、[`u06.rs:169-499`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L169-L499) (`seed=3417`) | **已复核等价**（`difference: "无"`） |
| 7 | `U06:skills:kuwu@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:53` | 技能 | `kuwu` | `skill.js:53-69`：`trigger: { source: "damageSource" }, forced: true, equipSkill: true`；`filter`: `event.card.name === "sha" && event.notLink() && event.player.countCards("he") > 0`；`content`: `await trigger.player.chooseToDiscard(true, "he")`（受伤目标强制自选弃置 1 张手牌或装备区牌） | [`filtering/common.rs:543-552`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L543-L552)、[`asks.rs:265-290`](../../../../crates/engine-native-content/src/content/host/asks.rs#L265-L290)、[`skills_status.rs:665-723`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs#L665-L723)、[`resolution.rs:254`](../../../../crates/engine-native-kernel/src/archive/resolution.rs#L254)、[`ai.rs:798-814`](../../../../crates/engine-native/src/ai.rs#L798-L814) | [`u06.rs:883-1012`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L883-L1012) (`u06_kuwu_six_dimension_real_match_equivalence`, seeds `3607..3608`)、[`nc08_upstream_kuwu.rs:212-389`](../../../../crates/engine-native/tests/nc08_upstream_kuwu.rs#L212-L389) (seeds `20262703..20262705`) | **已复核等价**（`difference: "无"`） |
| 8 | `U06:cards:xuelunyang@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:97` | 卡牌 | `xuelunyang` | `card.js:97-108`、`list.js:4`：宝物牌（`type: "equip", subtype: "equip5"`，红桃 2），伴随装备技 `["xuelunyang"]`，`equipValue: 8` | [`cards.json:40-45`](../../../../packages/extensions/huanle-upstream-noname/assets/cards.json#L40-L45)、[`card_skills.rs:136-138`](../../../../crates/engine-native-kernel/src/vocabulary/card_skills.rs#L136-L138)、[`card_skills.rs:374`](../../../../crates/engine-native-kernel/src/vocabulary/card_skills.rs#L374) | [`u06.rs:1017-1212`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L1017-L1212) (`u06_xuelunyang_six_dimension_real_match_equivalence`, seeds `3609..3610`) | **已复核等价**（`difference: "无"`） |
| 9 | `U06:skills:xuelunyang@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:70` | 技能 | `xuelunyang` | `skill.js:70-128`：`trigger: { player: "phaseBegin" }, equipSkill: true`；当存在拥有技能的其他存活角色时，可选 1 名有技能的其他角色（`cost: chooseTarget`，可取消），再从其技能列表中选 1 个技能（`chooseButton`）临时获得至回合结束（`addTempSkills`） | [`filtering/common.rs:681-699`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L681-L699)、[`metadata/common.rs:943-950`](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs#L943-L950)、[`asks.rs:292-333`](../../../../crates/engine-native-content/src/content/host/asks.rs#L292-L333)、[`skills_targets.rs:212-301`](../../../../crates/engine-native-content/src/content/host/step/skills_targets.rs#L212-L301)、[`ai.rs:816-858`](../../../../crates/engine-native/src/ai.rs#L816-L858) | [`u06.rs:1017-1212`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L1017-L1212) (`u06_xuelunyang_six_dimension_real_match_equivalence`, seeds `3609..3610`) | **已复核等价**（`difference: "无"`） |
| 10 | `U06:cards:jiuwei@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:109` | 卡牌 | `jiuwei` | `card.js:109-120`、`list.js:3`：宝物牌（`type: "equip", subtype: "equip5"`，红桃 9），伴随装备技 `["jiuwei"]`，`equipValue: 8` | [`cards.json:52-60`](../../../../packages/extensions/huanle-upstream-noname/assets/cards.json#L52-L60)、[`host_skill_bindings.rs:121`](../../../../crates/content-native-loader/src/host_skill_bindings.rs#L121) | [`u06.rs:1217-1342`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L1217-L1342) (`u06_jiuwei_six_dimension_real_match_equivalence`, seed `3611`) | **已复核等价**（`difference: "无"`） |
| 11 | `U06:skills:jiuwei@archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:129` | 技能 | `jiuwei` | `skill.js:129-143`：`trigger: { player: "phaseEnd" }, frequent: true, equipSkill: true`；回合结束时可选发动（`trigger_confirm`），若已受伤（`hp < max_hp`）则回复 1 点体力，否则摸 1 张牌 | [`filtering/common.rs:553-555`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L553-L555)、[`skills_recovery.rs:287-295`](../../../../crates/engine-native-content/src/content/host/step/skills_recovery.rs#L287-L295)、[`ai.rs:754-757`](../../../../crates/engine-native/src/ai.rs#L754-L757) | [`u06.rs:1217-1342`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L1217-L1342) (`u06_jiuwei_six_dimension_real_match_equivalence`, seed `3611`) | **已复核等价**（`difference: "无"`） |

---

## 2. 六维度真实对局事件路径核验摘要

针对 `U06 欢乐卡牌` 全部 6 张卡牌与 5 项技能，独立核验了 [`crates/engine-native/tests/g1_huanle/u06.rs:514-1342`](../../../../crates/engine-native/tests/g1_huanle/u06.rs#L514-L1342) 中的 6 个六维度测试函数（全程通过 `Engine::submit(PlayCard / NextPhase / Answer)` 推进真实阶段与事件，零 `debug_execute_effect` 调用）：

1. **维度 1（正向触发/使用路径）**：
   - `monkey`：他人使用【桃】时挂起 `huanle_monkey_confirm`，选择 `Confirm` 弃置【猴子】、将【桃】收入手牌并取消【桃】回血结算；
   - `mianju`：装备【旋涡面具】后无论正面朝上或背面朝上，触发 `TurnOverBefore` 均被锁定拦截；
   - `shoulijian`：对距离 > 1 的目标使用【手里剑】挂起 `host_target_discard_equip`；
   - `kuwu`：装备【苦无】（攻击范围 2）使用【杀】造成非连环伤害后挂起 `kuwu_discard`；
   - `xuelunyang`：回合开始挂起 `huanle_xuelunyang_target` -> `huanle_xuelunyang_skill`，获得目标技能至回合结束；
   - `jiuwei`：回合结束挂起 `trigger_confirm`（`source_skill: Some("huanle_jiuwei")`），确认后回血或摸牌。
2. **维度 2（反向过滤路径 / 负例）**：
   - `monkey`：装备者自己使用【桃】不触发（`seed=3601`）；
   - `mianju`：0 号装备【旋涡面具】不拦截 1 号翻面（`seed=3604`）；
   - `shoulijian`：对自身或距离 1 角色使用返回 `Rejected(RejectReason::IllegalTargets)`；对距离 2 且 `he` 区无装备的角色直接造成 1 点伤害不弹窗（`seed=3605`）；
   - `kuwu`：目标 `he` 区为空时不挂起（`seed=3607`）；铁索连环传导伤害（`from_chain: true`）不触发（`seed=20262704`）；
   - `xuelunyang`：场上其他存活角色均无技能时不触发（`seed=3609`）；
   - `jiuwei`：其他角色回合结束时不触发（`seed=3611`）。
3. **维度 3（选择分支 A vs B）**：
   - `monkey`：`Confirm`（夺桃弃猴）vs `Pass`（放行回血留猴）；
   - `shoulijian`：弃手牌装备 `hand_equip` vs 弃装备区装备 `worn_equip` 均免伤；
   - `kuwu`：弃手牌 `hand_tao` vs 弃装备区 `eq_monkey`（同步撤销伴随装备技 `huanle:monkey`）；
   - `xuelunyang`：选目标 1 vs 目标 2，以及选 `Tag(0)`（「马术」）vs `Tag(1)`（「咆哮」）；
   - `jiuwei`：已受伤（`hp=2/4`）回复 1 点体力 vs 满血（`hp=4/4`）摸 1 张牌。
4. **维度 4（可选取消路径 / 强制拒空）**：
   - `monkey`、`shoulijian`、`xuelunyang`（阶段一）、`jiuwei` 支持取消/Pass 并按上游语义分支结算；
   - `kuwu`（`allow_cancel: false`）对空选 `vec![]` 返回 `Rejected(RejectReason::InvalidChoice)`。
5. **维度 5（AI `BoundedPlanner` 仅凭公开视图决策）**：
   - `monkey`：AI 仅凭公开身份态度（主公明置身份）对敌方 `Confirm`、对友方 `Pass`，且翻转未明置第三方的暗身份（忠/内/反）决策完全一致；
   - `mianju` + `shoulijian`：背面朝上且装备【旋涡面具】时 AI 选择 `Pass` 受 1 伤保留面具（对齐 `card.js:28-30, 50-58`）；正面朝上时弃最低价值装备免伤；
   - `kuwu`：AI 保留高价值【桃】、弃置较低价值装备；
   - `xuelunyang`：AI 优先选择技能数最多的目标与首个可窃技能；
   - `jiuwei`：AI 自动确认发动（对齐 `frequent: true`）。
6. **维度 6（挂起中点 `save() -> encode_archive -> decode_archive -> restore() -> Answer` 等价）**：
   - `monkey`、`mianju`+`shoulijian`、`kuwu`、`xuelunyang`（Stage 1 & Stage 2 双挂起占点）、`jiuwei` 在选择挂起中途序列化往返恢复后提交相同 `Answer`，`state_hash` 与 `log()` 事件流 100% 一致。

---

## 3. 门禁与测试执行记录

```text
$ cargo test --offline -p engine-native --test g1 g1_huanle
running 29 tests ... test result: ok. 28 passed; 0 failed; 1 ignored

$ cargo test --offline -p engine-native --test nc08_upstream_a_k upstream_kuwu
running 5 tests ... test result: ok. 5 passed; 0 failed

$ cargo test --offline -p engine-native --test l_rounds l11_taixu_boss_events
running 5 tests ... test result: ok. 5 passed; 0 failed

$ cargo test --offline -p engine-native --test misc decision34_pack9_closure
running 6 tests ... test result: ok. 6 passed; 0 failed

$ cargo test --offline -p engine-native --test misc decision34_rules_and_campaigns
running 7 tests ... test result: ok. 7 passed; 0 failed
```

# `L29 龙血玄黄`（`longxue`）全源 9 项（6/6 规则与配置项 + 3 项纯前端显示面）独立复核报告（2026-11-12）

- **复核人**：`l29-signoff`（独立复核 agent；未编写或修改任何 `L29` 代码或测试，严格遵守裁决 40 第三节第 4 条“实现者不得参与复核”）
- **复核日期**：`2026-11-12`
- **上游来源**：`archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js` 及手杀同源技能定义 `archive/upstream/noname-liuli/noname/character/mobile/skill.js:14399-14847`（共 **9** 项实体：5 项规则技能 `mbqianlong`（含伴生主公技 `mbweitong`）、`mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin` + 1 项扩展配置 `shengjian` + 3 项纯前端显示面 `_CMeffects`、`main1`、`main1.css`）
- **原生实现**：[`packages/extensions/longxue-upstream-liuli/assets/cards.json`](../../../../packages/extensions/longxue-upstream-liuli/assets/cards.json)、[`packages/extensions/longxue-upstream-liuli/assets/config.json`](../../../../packages/extensions/longxue-upstream-liuli/assets/config.json)、[`crates/content-native-loader/src/longxue.rs`](../../../../crates/content-native-loader/src/longxue.rs)、[`crates/content-native-loader/src/host_skill_bindings.rs`](../../../../crates/content-native-loader/src/host_skill_bindings.rs)、[`crates/engine-native-content/src/content/host/activation/common.rs`](../../../../crates/engine-native-content/src/content/host/activation/common.rs)、[`crates/engine-native-content/src/content/host/filtering/common.rs`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs)、[`crates/engine-native-content/src/content/host/step/skills_status.rs`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs)、[`crates/engine-native-content/src/content/host/step/skills_damage.rs`](../../../../crates/engine-native-content/src/content/host/step/skills_damage.rs)、[`crates/engine-native-content/src/content/host/step/skills_targets.rs`](../../../../crates/engine-native-content/src/content/host/step/skills_targets.rs)、[`crates/engine-native-content/src/content/host/support.rs`](../../../../crates/engine-native-content/src/content/host/support.rs)、[`crates/engine-native-kernel/src/kernel/combat_rules.rs`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs)、[`crates/engine-native-kernel/src/kernel/events/turn.rs`](../../../../crates/engine-native-kernel/src/kernel/events/turn.rs)、[`crates/engine-native-kernel/src/kernel/phase.rs`](../../../../crates/engine-native-kernel/src/kernel/phase.rs)、[`crates/engine-native-kernel/src/machine.rs`](../../../../crates/engine-native-kernel/src/machine.rs)、[`crates/engine-native/src/ai/extension_choices.rs`](../../../../crates/engine-native/src/ai/extension_choices.rs)
- **真实对局测试**：[`crates/engine-native/tests/l29_longxue_eight.rs`](../../../../crates/engine-native/tests/l29_longxue_eight.rs)、[`crates/engine-native/tests/d5_longxue_mbqianlong.rs`](../../../../crates/engine-native/tests/d5_longxue_mbqianlong.rs)
- **版本门禁**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`（零升号，14 场 golden 哈希零变化）
- **复核结论**：**6/6 项规则与配置项 100% 通过独立复核（`已复核等价` 6 项），3 项纯前端 UI/CSS 显示面保持 `简化近似`（归属执行队列第 5 项界面阶段），`缺失` 0 项**；全仓 [`docs/migration/g0/inventory_status.json`](../../../migration/g0/inventory_status.json) 累计 `已复核等价` 由 `61` 增至 **`67`**，`简化近似` 由 `4,363` 降至 **`4,357`**。

---

## 1. 逐项上游源码 vs 原生实现对照表（9 项全覆盖：6 项规则与配置 `已复核等价` + 3 项纯前端显示面 `简化近似`）

| 序号 | 清单键 (`inventory_status.json`) | 类别 | 实体 ID | 上游源码 `file:line` 与核心语义（时机 / 过滤与作用域 / 交互与状态效果） | 原生实现位置 (`file:line`) | 六维真实对局测试与确定性种子 (`seed`) | 独立复核结论 |
|---:|---|---|---|---|---|---|---|
| 1 | `L29:skills:mbqianlong@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `skills` | `mbqianlong` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:6-176`：持技者（`mb_caomao`）道心值系统（上限 `99`）：首阶段开始获 `+20` 道心（若场上另有其他存活魏势力角色持主公技 `mbweitong` 则改为 `+60`）；非摸牌阶段获得牌 `+5`；受到伤害 `+10 × 伤害值`；造成伤害 `+15 × 伤害值`；实时按阈值刷新授予 `25 -> mbcmqingzheng`、`50 -> mbcmjiushi`、`75 -> mbcmfangzhu`、`99 -> mbjuejin`（达 `99` 时同步移除自身 `mbweitong`） | [`filtering/common.rs:53-71,462-487`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L53-L71) · [`step/skills_status.rs:195-231,657`](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs#L195-L231) · [`support.rs:416-460`](../../../../crates/engine-native-content/src/content/host/support.rs#L416-L460) (`HostBehavior::MbQianlong`, tag 469; `HostBehavior::MbWeitong`, tag 470; `mbqianlong_refresh`) | [`l29_longxue_eight.rs:155-463`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L155-L463) · [`d5_longxue_mbqianlong.rs:34-215`](../../../../crates/engine-native/tests/d5_longxue_mbqianlong.rs#L34-L215) (`seeds = 96001, 96010`) | **已复核等价**（无语义差异） |
| 2 | `L29:skills:mbcmqingzheng@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `skills` | `mbcmqingzheng` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:177-303`：出牌阶段开始时（手牌非空），经确认门进入三段真实 `Choice` 交互：① `PickTag("mbcmqingzheng_suit")` 选择手牌中的一种花色（不可取消） -> ② `SelectPlayers("mbcmqingzheng_target")` 选择一名有手牌的其他存活角色（可取消） -> ③ `PickTag("mbcmqingzheng_target_suit")` 选择目标手牌中的一种花色（不可取消），双方同时弃置所选花色手牌；若目标弃牌张数严格少于使用者，使用者对目标造成 `1` 点普通伤害 | [`filtering/common.rs:417-419`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L417-L419) · [`step/skills_damage.rs:453-568`](../../../../crates/engine-native-content/src/content/host/step/skills_damage.rs#L453-L568) · [`extension_choices.rs:46-75`](../../../../crates/engine-native/src/ai/extension_choices.rs#L46-L75) (`HostBehavior::MbcmQingzheng`, tag 474) | [`l29_longxue_eight.rs:464-574`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L464-L574) · [`d5_longxue_mbqianlong.rs:218-360`](../../../../crates/engine-native/tests/d5_longxue_mbqianlong.rs#L218-L360) (`seed = 96011`) | **已复核等价**（无语义差异） |
| 3 | `L29:skills:mbcmjiushi@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `skills` | `mbcmjiushi` | `archive/upstream/noname-liuli/noname/character/mobile/skill.js:14399-14558`：① 出牌阶段主动技（正面朝上且本回合未用过【酒】）：将武将牌翻至背面（`face_down = true` 并派发 `TurnOvered`）并视为使用一张【酒】（`jiu_used = true, jiu_active = true`，标记本技能强化的【杀】）；② 背面朝上受到非本技能强化【杀】造成的伤害后，打开 `Confirm("mbcmjiushi_turnback_confirm")` 询问是否翻回正面（`face_down = false` 并派发 `TurnOvered`）；③ 每当武将牌翻面（`TurnOvered`）时，打开 `Confirm("mbcmjiushi_gain_confirm")` 询问是否从牌堆或弃牌堆获得一张锦囊牌 | [`activation/common.rs:511-525`](../../../../crates/engine-native-content/src/content/host/activation/common.rs#L511-L525) · [`filtering/common.rs:421-459`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L421-L459) · [`step/skills_targets.rs:439-535`](../../../../crates/engine-native-content/src/content/host/step/skills_targets.rs#L439-L535) (`HostBehavior::MbcmJiushi`, tag 546) | [`l29_longxue_eight.rs:577-683`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L577-L683) (`seed = 96020`) | **已复核等价**（无语义差异） |
| 4 | `L29:skills:mbcmfangzhu@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `skills` | `mbcmfangzhu` | `archive/upstream/noname-liuli/noname/character/mobile/skill.js:14559-14750`：出牌阶段限一次主动技：第一段 `SelectPlayers("mbcmfangzhu_target")` 选择一名除自身及上次放逐目标（`mbcmfangzhu_last_target`）外的其他存活角色（取消则回滚 `once_used`；若出牌阶段未发动则在阶段结束清除上次目标记录） -> 第二段 `PickTag("mbcmfangzhu_option")` 二选一：`Tag(0)` 令目标直到其下回合结束不能使用/打出除锦囊牌外的手牌（`mbcmfangzhu_ban`），`Tag(1)` 令目标直到其下回合结束所有非锁定技/非 Charlotte 技能失效（`mbcmfangzhu_baiban`） | [`activation/common.rs:526-550`](../../../../crates/engine-native-content/src/content/host/activation/common.rs#L526-L550) · [`step/skills_targets.rs:537-668`](../../../../crates/engine-native-content/src/content/host/step/skills_targets.rs#L537-L668) · [`combat_rules.rs:146-181`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs#L146-L181) · [`turn.rs:365-393`](../../../../crates/engine-native-kernel/src/kernel/events/turn.rs#L365-L393) · [`phase.rs:256-268`](../../../../crates/engine-native-kernel/src/kernel/phase.rs#L256-L268) (`HostBehavior::MbcmFangzhu`, tag 547) | [`l29_longxue_eight.rs:686-861`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L686-L861) (`seed = 96030`) | **已复核等价**（无语义差异） |
| 5 | `L29:skills:mbjuejin@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `skills` | `mbjuejin` | `archive/upstream/noname-liuli/noname/character/mobile/skill.js:14751-14847`：出牌阶段限定技（整局 1 次，`mbjuejin_used`）：依次将所有存活角色体力调整为 `1`（减少的体力 `old_hp - 1` 转为等量护甲 `hujia`），使用者额外获得 `+2` 护甲；随后将牌堆、弃牌堆及所有角色手牌/装备区/判定区中的【闪】、【桃】、【酒】全部移出游戏（进入向死存魏区 `detached_cards`），并开启全局 `mbjuejin_xiangsicunwei` 使后续进入弃牌堆的【闪】、【桃】、【酒】同样移出游戏 | [`activation/common.rs:551-573`](../../../../crates/engine-native-content/src/content/host/activation/common.rs#L551-L573) · [`step/skills_targets.rs:670-786`](../../../../crates/engine-native-content/src/content/host/step/skills_targets.rs#L670-L786) · [`machine.rs:475-492`](../../../../crates/engine-native-kernel/src/machine.rs#L475-L492) (`HostBehavior::MbJuejin`, tag 548) | [`l29_longxue_eight.rs:862-899`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L862-L899) (`seed = 96030`) | **已复核等价**（无语义差异） |
| 6 | `L29:configs:shengjian@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `configs` | `shengjian` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:350-356`：布尔型扩展配置项 `"shengjian"`（`name: "圣剑特效"`, `init: true`），声明并持久化圣剑特效开关；配合道心阈值表 `LONGXUE_THRESHOLDS`（`25/50/75/99` 四档全部 `converted = true`）提供宿主配置与解锁查询面 | [`config.json:1-9`](../../../../packages/extensions/longxue-upstream-liuli/assets/config.json#L1-L9) · [`loader/src/longxue.rs:10-101`](../../../../crates/content-native-loader/src/longxue.rs#L10-L101) (`parse_longxue_config`, `LONGXUE_THRESHOLDS`, `longxue_unlock_report`) | [`l29_longxue_eight.rs:203-232,392-412`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L203-L232) (`seed = 96001`) | **已复核等价**（无语义差异） |
| 7 | `L29:ui_hooks:_CMeffects@noname-next/docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1` | `ui_hooks` | `_CMeffects` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:304-348`：纯前端 Spine 圣剑出杀动画与击杀特效钩子（无规则状态变更） | 纯前端显示面，归属执行队列第 5 项界面阶段 | [`l29_longxue_eight.rs:408-412`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L408-L412)（验证不污染规则包与内核目录） | **简化近似**（纯前端显示面，归属执行队列第 5 项界面阶段） |
| 8 | `L29:surfaces:main1@archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.js:1-1048` | `surfaces` | `main1` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.js:1-1048`：手杀 UI 专属技能区前端 DOM 面板脚本 | 纯前端显示面，归属执行队列第 5 项界面阶段 | [`l29_longxue_eight.rs:182-201,383-391`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L182-L201) | **简化近似**（纯前端显示面，归属执行队列第 5 项界面阶段） |
| 9 | `L29:surfaces:main1.css@archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.css:1-241` | `surfaces` | `main1.css` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.css:1-241`：手杀 UI 专属技能区前端 CSS 样式表 | 纯前端显示面，归属执行队列第 5 项界面阶段 | [`l29_longxue_eight.rs:182-201,383-391`](../../../../crates/engine-native/tests/l29_longxue_eight.rs#L182-L201) | **简化近似**（纯前端显示面，归属执行队列第 5 项界面阶段） |

---

## 2. 六维度真实对局事件路径核验摘要

独立复核了 [`crates/engine-native/tests/l29_longxue_eight.rs`](../../../../crates/engine-native/tests/l29_longxue_eight.rs)（4 个集成测试）与 [`crates/engine-native/tests/d5_longxue_mbqianlong.rs`](../../../../crates/engine-native/tests/d5_longxue_mbqianlong.rs)（7 个专项测试），全程通过真实 `Engine::submit(PlayCard / ActivateSkill / NextPhase / Answer)` 推进：

1. **维度 ①（触发时机与负条件）**：
   - `mbqianlong` 在首阶段开始时按场上是否有其他存活魏势力角色持 `mbweitong` 精准区分 `+20` 与 `+60` 道心；非摸牌阶段获牌 `+5`、受伤 `+10×`、造成伤害 `+15×`，封顶 `99`；未持潜龙者道心恒为 `0`；
   - `mbcmjiushi` 正面朝上且本回合未用酒时可发动，翻至背面或已用酒时再次发动返回 `CommandOutcome::Rejected`；
   - `mbcmfangzhu` 出牌阶段限 1 次，第一段取消时回滚 `once_used` 允许同阶段重发，成功结算后同回合再发被拒，且次轮不可连续放逐上一轮目标；
   - `mbjuejin` 限定技整局仅可发动 1 次，二次发动返回 `CommandOutcome::Rejected`。
2. **维度 ②（目标/卡牌过滤与阈值边界）**：
   - `mbqianlong` 阈值 `25 / 50 / 75 / 99` 依次动态授予 `mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin`，达 `99` 时移除自身 `mbweitong`；
   - `mbcmfangzhu` `Tag(0)` 精确禁止目标使用/打出基本与装备手牌（如【杀】、【诸葛连弩】被拒），放行锦囊手牌（如【无中生有】）；`Tag(1)` 精确令目标非锁定技（如 `mbcmjiushi`）失效而保留锁定技（如 `mbqianlong`）。
3. **维度 ③（选择与行动窗口交互）**：
   - `mbcmqingzheng` 完整走通 `trigger_confirm` -> `mbcmqingzheng_suit`（不可取消，空选被拒） -> `mbcmqingzheng_target` -> `mbcmqingzheng_target_suit` 三段 `Choice` 窗口；
   - `mbcmjiushi` 完整走通主动翻面 -> `mbcmjiushi_gain_confirm` 确认获锦囊牌，以及背面受非酒诗强化伤害 -> `mbcmjiushi_turnback_confirm` 确认翻回正面 -> `mbcmjiushi_gain_confirm` 取消不获牌；
   - `mbcmfangzhu` 完整走通 `mbcmfangzhu_target` -> `mbcmfangzhu_option` (`Tag(0)` / `Tag(1)`) 两段 `Choice` 窗口。
4. **维度 ④（跨阶段多步状态联动）**：
   - 验证 `mbcmfangzhu` 禁令与非锁定技失效跨回合生效并在目标下回合结束时解除；
   - 验证 `mbjuejin` 将全场存活角色体力置 `1`、溢出体力转护甲、使用者额外 `+2` 护甲，并将全场【闪】、【桃】、【酒】移入 `detached_cards`（向死存魏区）。
5. **维度 ⑤（存档编码/解码与恢复确定性）**：
   - 在 `mbcmqingzheng` 结算后、`mbcmjiushi` 翻面获锦囊后、`mbcmfangzhu` 施加禁令后分别执行 `encode_archive(&engine.save()) -> decode_archive -> Engine::restore_with_extensions`，道心值、翻面/酒劲、技能解锁实例与禁令状态 100% 保持一致。
6. **维度 ⑥（AI `BoundedPlanner` 合法规划）**：
   - 在 `mbcmqingzheng_suit`、`mbcmjiushi_gain_confirm`、`mbcmfangzhu_option` 打开的 `Choice` 窗口中调用 `BoundedPlanner::new(16)` 的 `plan_action`，均返回合法的 `GameAction::Answer`。

---

## 3. 核验命令与实际运行结果

```bash
cargo test --offline -p engine-native --test l_rounds l29_longxue_eight
# running 4 tests
# test l29_longxue_eight::l29_mbqianlong_mbweitong_and_mbcmqingzheng_six_dimensions ... ok
# test l29_longxue_eight::l29_mbcmjiushi_six_dimensions ... ok
# test l29_longxue_eight::l29_mbcmfangzhu_and_mbjuejin_six_dimensions ... ok
# test l29_longxue_eight::l29_eight_dimension_from_fresh_install ... ok
# test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.29s

cargo test --offline -p engine-native --test misc d5_longxue_mbqianlong
# running 7 tests
# test d5_longxue_mbqianlong::longxue_package_maps_host_and_defers_external ... ok
# test d5_longxue_mbqianlong::longxue_threshold_grants_when_skill_exists ... ok
# test d5_longxue_mbqianlong::qingzheng_no_damage_when_target_discards_enough ... ok
# test d5_longxue_mbqianlong::qingzheng_discards_both_sides_and_punishes_fewer ... ok
# test d5_longxue_mbqianlong::longxue_weitong_registration_only ... ok
# test d5_longxue_mbqianlong::longxue_opening_marks_twenty_or_sixty ... ok
# test d5_longxue_mbqianlong::longxue_marks_accumulate_and_cap ... ok
# test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 421 filtered out; finished in 0.00s

python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py
# Ran 2 tests in 0.010s - OK
# Source line limit passed: 9279 files <= 2000 lines; Rust non-test files <= 1000 effective lines
```

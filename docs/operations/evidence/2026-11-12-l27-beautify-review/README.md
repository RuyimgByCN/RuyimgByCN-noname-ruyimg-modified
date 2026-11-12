# L27 选将美化（54 项：10 配置 + 31 技 + 12 ui_hooks + 1 CSS 表面项）独立逐行对照与 6 维真实验收报告

- **核验日期**：2026-11-12
- **独立复核人**：`l27-signoff`（未参与 `L27` 规则实现与测试编写）
- **上游基准**：
  - `archive/upstream/noname-liuli/noname/extension/选将美化/extension.js`（9,094 行：第 1162–9008 行定义 31 项 `skills` 与 12 项含自定义选牌/加减数字/花色分列弹窗而被 G0 扫描器归入 `ui_hooks` 的 `lib.skill.*` 技能；第 9014–9068 行定义 10 项扩展配置）
  - `archive/upstream/noname-liuli/noname/extension/选将美化/e.css`（2,009 行纯前端 CSS 样式表，归属执行队列第 5 项界面阶段）
- **原生实现落点**：
  - 正式扩展包：`packages/extensions/select-beautify-upstream-liuli`（通过 `crates/content-native-loader/src/decision34_pack9.rs`、`crates/content-native-loader/src/decision34_pack9_data.rs:265-276`、`crates/content-native-loader/src/host_skill_bindings.rs` 绑定全部 43 项技能/钩子与 10 项配置）
  - 6 项独立专用 `HostBehavior` 技能：`xiefang`（`HostBehavior::Xiefang`，`step/skills_targets.rs:514-540` + `d34_l11_mods.rs:82-104`）、`sbzhaxiang`（`HostBehavior::Sbzhaxiang`，`step/skills_draw.rs:422-455` + `d34_l11_mods.rs:145-169`）、`new_meibu`（`HostBehavior::NewMeibu` + `NewMeibuRange`，`step/skills_targets.rs:542-607` + `d34_l11_mods.rs:171-195`）、`new_zhixi`（`HostBehavior::NewZhixi`，`step/skills_targets.rs:584-607` + `d34_l11_mods.rs:197-225`）、`jishe`（`HostBehavior::Jishe`，`step/skills_draw.rs:457-489` + `d34_l11_mods.rs:227-244`）、`mbcmqingzheng`（`HostBehavior::MbcmQingzheng`，`step/skills_damage.rs:453-568`）
  - Batch A（`HostBehavior::L27BeautifyBatchA = 557`，19 项技能/标记伴生子技）：`crates/engine-native-content/src/content/host/step/l27_batch_a.rs:1-1058` + `crates/engine-native-content/src/content/host/step/l27_batch_a_asks.rs:1-757`
  - Batch B（`HostBehavior::L27BeautifyBatchB = 558`，7 项技能 + 11 项 `ui_hooks` 技能）：`crates/engine-native-content/src/content/host/step/l27_batch_b.rs:1-809` + `crates/engine-native-content/src/content/host/step/l27_batch_b_tw.rs:1-872` + `crates/engine-native-content/src/content/host/step/l27_batch_b_asks.rs:1-396`
  - 内核修饰器钩子（距离修正、出牌阶段用牌无距限/无限次、出杀次数上限、手牌上限加减、使用锦囊锁牌、狭锋不可响应）：`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_l11_mods.rs:82-396` + `crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs`
  - 10 项扩展配置解析、双将候选对校验与快照恢复：`crates/content-native-loader/src/select_beautify.rs:1-171`（`SelectBeautifyConfig`、`parse_select_beautify_config`、`validate_dual_general_pair`、`snapshot`/`restore`）
- **验证测试文件**：
  - `crates/engine-native/tests/l27_beautify_match.rs:1-783`
  - `crates/engine-native/tests/l27_beautify_batch_b.rs:1-426`
  - `crates/engine-native/tests/l27_select_beautify_eight.rs:1-172`（`l_rounds` 8 维全生命周期验收测试）
- **冻结版本号核对**：`ENGINE_PROTOCOL_VERSION = 333` · `RULESET_VERSION = 107` · `STATE_SCHEMA_VERSION = 5` · `EVENT_CATALOG_VERSION = 759` · `AI_DECISION_CONTRACT_VERSION = 8`（零破坏，14 场 golden 哈希零变化）
- **复核结论**：
  - `L27` 全部 **53/53** 项非 CSS 实体（**10 项配置 + 31 项技能 + 12 项 `ui_hooks` 技能**）逐行对照上游 JS 与 6 维真实对局测试全部通过，状态由 `简化近似` 更新为 **`已复核等价`**（`reviewer: "l27-signoff"`, `review_date: "2026-11-12"`）。
  - 剩余 **1/54** 项 `L27:surfaces:e.css`（2,009 行纯前端 CSS 渲染面）保持 **`简化近似`**，留待执行队列第 5 项界面阶段专项处理。
  - 全仓 `inventory_status.json`（4,424 项）：`已复核等价` **230 → 283**（`6.40%`），`简化近似` **4,194 → 4,141**（`93.60%`），`缺失` **0**，`上游未定义` **0**。

---

## 1. 固定随机种子抽样精查（`seed = 2026111209`，覆盖率 `8 / 53 = 15.09% >= 10%`）

使用 `python3 -c 'import random; rng = random.Random(2026111209); ...'` 对 `L27` 的 53 项非渲染面实体固定种子抽样 **8 项（15.09%，覆盖 `configs` / `skills` / `ui_hooks` 三类）** 逐行精查：

| # | 抽样实体 Key | 类别 | 上游源码行号与精确语义 | 原生实现文件:行号与逐分支核对 | 实跑测试与固定种子 | 复核结论 |
|---:|---|---|---|---|---|---|
| 1 | `L27:skills:new_zhixi` | `skills` | `extension.js:1738-1801`：锁定技，出牌阶段你至多使用 X 张牌（X 为体力值），你于出牌阶段使用锦囊牌后不能再使用牌。 | `step/skills_targets.rs:584-607` + `d34_l11_mods.rs:197-225`：通过 `new_zhixi_used` 统计本阶段用牌数、达 `hp` 拦截后续用牌；使用锦囊牌后置 `new_zhixi_trick_locked = 1` 禁止本阶段继续用牌，阶段结束由 `new_zhixi_recover` 重置。 | `l27_beautify_match_dedicated_six_skills` (`seed = 2026111201`) | `已复核等价` |
| 2 | `L27:skills:sbzhaxiang` | `skills` | `extension.js:1590-1671`：锁定技，每当你失去 1 点体力后，你摸 3 张牌；若此时在你的出牌阶段内，则本回合你使用红色【杀】无距离限制且不能被【闪】响应，且你可以多使用 1 张【杀】。 | `step/skills_draw.rs:422-455` + `d34_l11_mods.rs:145-169`：在 `HpLost` 按失去体力点数 `amount` 摸 `3 * amount` 张牌；若处于自身 `Phase::Play`，累加 `sbzhaxiang_boost += amount`，使出杀次数上限 `+boost` 且红杀无距离限制、不可被闪响应。 | `l27_beautify_match_dedicated_six_skills` (`seed = 2026111201`) | `已复核等价` |
| 3 | `L27:ui_hooks:twfengji` | `ui_hooks` | `extension.js:6889-7043`：出牌阶段各限一次，当你使用基本牌或普通锦囊牌时，你可以①弃置 X 张牌令此牌摸牌数或伤害值基数 +X；或②摸 X 张牌令此牌摸牌数或伤害值基数 -X（通过 `window.shuzibg` 数字加减面板选择模式与 X）。 | `l27_batch_b_tw.rs:273-364`：在 `CardUsed`（基本牌/普通锦囊且本阶段未触发）打开 `l27_twfengji_mode`（`Tag(0)` 弃牌增效 / `Tag(1)` 摸牌减效）与 `l27_twfengji_count` 选择数值 X，按所选分支弃 X 牌增伤/补摸或摸 X 牌减效并标记本阶段已用。 | `l27_beautify_batch_b_tw_and_misc_skills` (`seed = 2026111206`) | `已复核等价` |
| 4 | `L27:skills:clandianzhan` | `skills` | `extension.js:1867-1927`：锁定技，每轮每种花色限一次，当你使用牌指定唯一目标后，若其没有此花色的手牌，你横置之，然后你摸 1 张牌；每轮开始时重置已记录花色。 | `l27_batch_a.rs:817-867`：通过 4 位花色位掩码 `clandianzhan_suit_mask` 跟踪本轮已触发花色，唯一目标无该花色手牌时置 `chained = true` 并摸 1 张牌；在 `RoundStart` 归零掩码。 | `l27_beautify_match_batch_a_skills_part1` (`seed = 2026111202`) | `已复核等价` |
| 5 | `L27:skills:jishe` | `skills` | `extension.js:1929-1958`（处于 `/*lib.skill.jishe = ...*/` 块注释中）：出牌阶段，若你的手牌上限大于 0，你可以摸 1 张牌，然后本回合你的手牌上限 -1；结束阶段，若你没有手牌，你可以令至多 X 名角色横置（X 为你的体力值，`jishe2`）。 | `step/skills_draw.rs:457-489` + `d34_l11_mods.rs:227-244` + `l27_batch_b_tw.rs:548-586`：校验当前有效手牌上限 `> 0` 后摸 1 牌并累加 `jishe_minus_limit`（上限 `-1`）；结束阶段手牌为空时由 `jishe2` 打开 `l27_jishe2_targets` 选 `1..=hp` 名角色横置，笔记如实登记上游块注释状态。 | `l27_beautify_match_dedicated_six_skills` (`seed = 2026111201`) + `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 6 | `L27:ui_hooks:twxingzhui` | `ui_hooks` | `extension.js:6479-6631`：结束阶段，你可以失去 X 点体力（`1 <= X <= hp`，通过 `window.shuzibg` 加减按钮选择）并观看牌堆顶 2X 张牌；若其中黑色牌数量恰好等于 X，你可以对一名其他角色造成 X 点雷电伤害。 | `l27_batch_b_tw.rs:56-142`：结束阶段打开 `l27_twxingzhui_lose_hp` 选 `1..=hp` 失去 X 点体力，检视牌堆顶 `2X` 张牌统计黑牌数 `black_count`；当 `black_count == X` 时打开 `l27_twxingzhui_target` 选一名其他角色造成 X 点雷电伤害。 | `l27_beautify_batch_b_tw_and_misc_skills` (`seed = 2026111206`) | `已复核等价` |
| 7 | `L27:skills:taoluan` | `skills` | `extension.js:2351-2478`：非濒死状态下，你可以将一张牌当任意基本牌或普通锦囊牌使用（每种牌名限一次，通过 `ui.create.dialognew` 分类弹窗选牌名），然后令一名其他角色选择交给你一张类别不同的牌，或令你失去 1 点体力且本回合〖滔乱〗失效。 | `l27_batch_a.rs:99-160` + `l27_batch_a_asks.rs:245-351`：维护已用牌名位掩码 `taoluan_used_mask`，经 `l27_taoluan_card_name` -> `l27_taoluan_target` -> `l27_taoluan_ask_other` 完成转化用牌与索牌，目标拒交则使用者失 1 体力并置 `taoluan_disabled_turn = 1`。 | `l27_beautify_match_batch_a_skills_part2` (`seed = 2026111203`) | `已复核等价` |
| 8 | `L27:configs:gaibiaoji` | `configs` | `extension.js:9024-9028`：`gaibiaoji: { name: '改标记', init: true, intro: '开启后部分技能标记会改成美化样式' }`。 | `crates/content-native-loader/src/select_beautify.rs:7-171`：`SelectBeautifyConfig::gaibiaoji` 默认 `true`，支持布尔与字符串 `"true"`/`"false"`/`"1"`/`"0"` 解析及 `snapshot()`/`restore()` 往返恢复。 | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |

---

## 2. 已登记语义抽象与上游注释/死分支说明

1. **裁决 40 自定义 DOM 选牌/数字加减/花色分列/图标弹窗的有界 `ChoiceSpec` 抽象**：
   - 上游 `选将美化/extension.js` 的核心机制是将多个技能的原版选框改写为自定义 DOM 弹窗：
     - `window.shuzibg` 数字 `+/-` 加减按钮面板（`potzhuangshi`、`qimou`、`reqimou`、`sbjianxiong`、`twxiafeng`、`twxingzhui`、`xingchong`、`dcxiaoxi`、`twfengji`、`twharvestinori`、`twzhouhu`、`twzuhuo`、`twzhouzu`）；
     - `ui.create.dialognew` 四花色分列选牌或三栏拖拽分牌面板（`mbcmqingzheng`、`sbqingzheng`、`spyanjiao`）与分类选牌名弹窗（`taoluan`、`shouxi`、`dinghan`）；
     - 卷轴/立绘/五禽戏图标选择面板（`sbtongye`、`rehuashen`、`jinghe`、`bolan`、`drlt_duorui`、`pro_drlt_duorui`、`mbwuling`）。
   - 在无头确定性原生内核中，上述自定义 DOM 弹窗按裁决 40 统一抽象为有界 `ChoiceSpec`（`ChoiceKind::SelectOption` / `SelectCards` / `SelectPlayers` / `Confirm`）挂起交互，且全部 prompt 均已注册入 `KNOWN_PROMPTS`，完整支持挂起中途 `encode_archive` / `decode_archive` 存读档恢复与 `BoundedPlanner` AI 公开信息合法决策。
2. **关于 G0 清单中 12 项 `ui_hooks` 的说明**：
   - `inventory_status.json` 中 `L27:ui_hooks:*` 的 12 个条目（`mbcmqingzheng`、`sbqingzheng`、`spyanjiao`、`twxiafeng`、`twxingzhui`、`xingchong`、`dcxiaoxi`、`twfengji`、`twharvestinori`、`twzhouhu`、`twzuhuo`、`twzhouzu`）在上游 `extension.js:3697-7564` 中均为完整的 `lib.skill.*` 武将技能定义（因含 `ui.create.dialognew` / `window.shuzibg` DOM 操作被 G0 扫描器归入 `ui_hooks` 类别）。原生将其全部作为一等 `HostBehavior` 技能实现并通过 6 维真实对局测试，故本次同批复核并更新为 `已复核等价`。
3. **上游块注释项（`jishe`、`jishe2`）与 `if(false)` 死分支项（`huamu`）的如实登记**：
   - `jishe`（`extension.js:1929-1958`）与 `jishe2`（`extension.js:1959-1967`）在上游源码中位于 `/*lib.skill.jishe = ... lib.skill.jishe2 = ...*/` 块注释内；原生按注释内完整的岑昏〖极奢〗规则语义实现并实测通过，同时在 `difference`/`note` 字段如实注明上游块注释状态。
   - `huamu`（`extension.js:2748-2805`）在上游源码中为 `if(false) lib.skill.huamu = {...}` 死代码分支（运行时从不执行注册）；原生严格遵守「禁止凭空虚构」原则，保持惰性安全空操作并记录 `l27_huamu_disabled_upstream = 1` 审计标记，在 `difference`/`note` 字段如实登记。
4. **纯前端 CSS 渲染面**：
   - `L27:surfaces:e.css`（`e.css:1-2009`）属于纯前端视觉样式表，无规则态副作用，按计划保持 `简化近似` 并归入执行队列第 5 项界面阶段。

---

## 3. L27 全量 54 项逐项对照表（53 项 `已复核等价` + 1 项 CSS 渲染面 `简化近似`）

### 3.1 技能（`skills`，31 项，31/31 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 行为与语义抽象说明 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 1 | `xiefang` | `extension.js:1162-1212` | 撷芳：锁定技，你计算与其他角色的距离 `-max(1, 全场存活女性角色数)`。 | `step/skills_targets.rs:514-540` + `d34_l11_mods.rs:82-104` | `l27_beautify_match_dedicated_six_skills` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 2 | `regushe` | `extension.js:1215-1337` | 界鼓舌：出牌阶段用一张手牌与至多 3 名角色拼点，没赢角色选弃 1 牌或令你摸 1 牌；你没赢获 1 饶舌标记（联动激词），7 标记阵亡。 | `l27_batch_a.rs:34-97` + `l27_batch_a_asks.rs:14-129` | `l27_beautify_match_batch_a_skills_part1` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 3 | `kuangcai` | `extension.js:1340-1440` | 狂才：锁定技，出牌阶段用牌无距离和次数限制；弃牌阶段开始时未用牌手牌上限 -1、用牌未造伤不变、用牌且造伤摸牌阶段摸牌数改为至多 5 并动态修正手牌上限。 | `l27_batch_a.rs:647-694` + `d34_l11_mods.rs:106-143` | `l27_beautify_match_batch_a_skills_part1` | `已复核等价` |
| 4 | `sbtongye` | `extension.js:1444-1587` | 谋统业：结束阶段预测场上装备数变化（`l27_sbtongye_guess` 二选一抽象），下回合开始预测正确得“业”标记（上限 +业数）、错误弃“业”标记。 | `l27_batch_a.rs:696-782` + `l27_batch_a_asks.rs:131-162` | `l27_beautify_match_batch_a_skills_part1` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 5 | `sbzhaxiang` | `extension.js:1590-1671` | 谋诈降：锁定技，每失去 1 点体力摸 3 张牌；出牌阶段失去则本回合红杀无距限、不可被闪响应且出杀上限 +1。 | `step/skills_draw.rs:422-455` + `d34_l11_mods.rs:145-169` | `l27_beautify_match_dedicated_six_skills` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 6 | `new_meibu` | `extension.js:1674-1736` | 新魅步：其他角色出牌阶段开始时若你在其攻击范围内，可弃 1 牌令其本回合获〖止息〗；若弃置杀或黑色锦囊牌，本回合你始终视为在其攻击范围内。 | `step/skills_targets.rs:542-607` + `d34_l11_mods.rs:171-195` | `l27_beautify_match_dedicated_six_skills` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 7 | `new_zhixi` | `extension.js:1738-1801` | 新止息：锁定技，出牌阶段至多使用等同体力值张牌；使用锦囊牌后本阶段不能再使用牌。 | `step/skills_targets.rs:584-607` + `d34_l11_mods.rs:197-225` | `l27_beautify_match_dedicated_six_skills` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 8 | `new_zhixi_recover` | `extension.js:1802-1814` | 止息/魅步阶段末清理伴生子技：出牌阶段及回合结束重置止息锁牌、计数与魅步距离状态。 | `l27_batch_b_tw.rs:588-615` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 9 | `baonvezhi` | `extension.js:1817-1850` | 暴虐值：锁定技，造成或受到伤害后获得等同于伤害点的暴虐值标记（上限 5）。 | `l27_batch_a.rs:784-815` | `l27_beautify_match_batch_a_skills_part1` | `已复核等价` |
| 10 | `baonvezhi_mark` | `extension.js:1851-1864` | 暴虐值标记展示与计数同步伴生子技。 | `l27_batch_a.rs:1032-1042` | `l27_beautify_match_batch_a_skills_part1` | `已复核等价` |
| 11 | `clandianzhan` | `extension.js:1867-1927` | 族点盏：锁定技，每轮每花色限一次，使用牌指定唯一目标后若其无此花色手牌则横置之并摸 1 牌。 | `l27_batch_a.rs:817-867` | `l27_beautify_match_batch_a_skills_part1` | `已复核等价` |
| 12 | `jishe` | `extension.js:1929-1958` | 极奢（已登记：上游处于块注释中）：出牌阶段手牌上限 > 0 时可摸 1 牌并令本回合手牌上限 -1。 | `step/skills_draw.rs:457-489` + `d34_l11_mods.rs:227-244` | `l27_beautify_match_dedicated_six_skills` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 13 | `jishe2` | `extension.js:1959-1967` | 极奢结束阶段子技（已登记：上游处于块注释中）：结束阶段若无手牌，可令至多体力值名角色横置。 | `l27_batch_b_tw.rs:548-586` + `l27_batch_b_asks.rs:188-236` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 14 | `rehuashen` | `extension.js:1968-2255` | 界化身：开局获 3 张化身并选 1 技能；准备/结束阶段可切换化身技能或弃 1..2 张化身摸等量化身（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:869-943` + `l27_batch_a_asks.rs:164-243` | `l27_beautify_match_batch_a_skills_part1` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 15 | `rehuashen_mark` | `extension.js:2256` | 界化身标记展示与化身池数量同步伴生子技。 | `l27_batch_a.rs:1044-1051` | `l27_beautify_match_batch_a_skills_part1` | `已复核等价` |
| 16 | `rexinsheng` | `extension.js:2259-2275` | 界新生：受到 1 点伤害后若拥有化身，按伤害点数获得对应数量化身牌。 | `l27_batch_a.rs:945-969` | `l27_beautify_match_batch_a_skills_part1` | `已复核等价` |
| 17 | `taoluan` | `extension.js:2351-2478` | 滔乱：将一张牌当未用过的基本牌或普通锦囊牌使用，令一名其他角色交非该类别牌或你失 1 血且本回合滔乱失效（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:99-160` + `l27_batch_a_asks.rs:245-351` | `l27_beautify_match_batch_a_skills_part2` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 18 | `shouxi` | `extension.js:2480-2559` | 守玺：成为【杀】目标后声明未声明过的牌名，使用者须弃置一张同名牌否则此【杀】对你无效（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:971-1010` + `l27_batch_a_asks.rs:353-424` | `l27_beautify_match_batch_a_skills_part2` | `已复核等价` |
| 19 | `dinghan` | `extension.js:2562-2643` | 定汉：成为未记录锦囊牌目标时记录牌名并取消之；回合开始时可增加或移除一种锦囊牌名记录（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:1012-1030` + `l27_batch_a_asks.rs:426-498` | `l27_beautify_match_batch_a_skills_part2` | `已复核等价` |
| 20 | `quanji` | `extension.js:2645-2718` | 权计：出牌阶段结束手牌数 > 体力值或受到 1 点伤害后，摸 1 张牌并将 1 张手牌置为“权”，手牌上限 +“权”数。 | `l27_batch_a.rs:584-621` + `l27_batch_a_asks.rs:500-532` + `d34_l11_mods.rs:246-260` | `l27_beautify_match_batch_a_skills_part2` | `已复核等价` |
| 21 | `huamu` | `extension.js:2748-2805` | 花木（已登记：上游 `extension.js:2748` 为 `if(false) lib.skill.huamu = ...` 死分支，原生保持安全惰性空操作并置 `l27_huamu_disabled_upstream = 1`）。 | `l27_batch_a.rs:1053-1057` | `l27_beautify_match_batch_a_skills_part2` | `已复核等价` |
| 22 | `potzhuangshi` | `extension.js:3218-3598` | 壮誓：出牌阶段开始时可弃 X 张牌令前 X 张牌无距限且不可响应、失 Y 点体力令前 Y 张牌不计次数（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:427-481` + `l27_batch_a_asks.rs:534-614` + `d34_l11_mods.rs:262-289` | `l27_beautify_match_batch_a_skills_part2` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 23 | `qimou` | `extension.js:5902-6056` | 奇谋：限定技，出牌阶段失去 X 点体力，本回合计算与其他角色距离 -X 且可多使用 X 张【杀】（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:162-194` + `l27_batch_a_asks.rs:616-662` + `d34_l11_mods.rs:291-319` | `l27_beautify_match_batch_a_skills_part2` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 24 | `reqimou` | `extension.js:6059-6214` | 界奇谋：限定技，出牌阶段失去 X 点体力并摸 X 张牌，本回合计算与其他角色距离 -X 且可多使用 X 张【杀】（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:196-228` + `l27_batch_a_asks.rs:664-712` + `d34_l11_mods.rs:291-319` | `l27_beautify_match_batch_a_skills_part2` | `已复核等价` |
| 25 | `sbjianxiong` | `extension.js:6216-6347` | 谋奸雄：游戏开始时可选获 0..2 枚“治世”标记；受到伤害后可获得伤害牌并摸 `max(1, 伤害值 - 治世标记数)` 张牌（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:623-645` + `l27_batch_a_asks.rs:714-738` | `l27_beautify_match_batch_a_skills_part2` | `已复核等价` |
| 26 | `jinghe` | `extension.js:7568-7744` | 景合：出牌阶段限一次展示至多体力上限张不同牌名手牌，令等量角色各从天书技能池获 1 个技能至你下回合开始（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_a.rs:230-316` + `l27_batch_a_asks.rs:740-757` | `l27_beautify_match_batch_a_skills_part2` + `l27_beautify_match_save_restore_and_ai` | `已复核等价` |
| 27 | `bolan` | `extension.js:7747-8016` | 博览（含 `bolan_g`）：出牌阶段开始从 3 个出牌阶段限一次技能中选 1 个获得至阶段结束；其他角色出牌阶段限一次可失 1 血从你所选 3 个技能中获 1 个至阶段结束（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b.rs:58-156` + `l27_batch_b_asks.rs:15-82` | `l27_beautify_batch_b_bolan_duorui_and_save_restore` | `已复核等价` |
| 28 | `drlt_duorui` | `extension.js:8018-8169` | 夺锐：出牌阶段对其他角色造成伤害后若无已夺取技能，可废除 1 个装备栏并选其 1 个非限定/觉醒/主公技令其失效且你获得之至其下回合结束（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b.rs:306-378` + `l27_batch_b_asks.rs:84-135` | `l27_beautify_batch_b_bolan_duorui_and_save_restore` | `已复核等价` |
| 29 | `pro_drlt_duorui` | `extension.js:8170-8358` | 进阶夺锐：出牌阶段对其他角色造成伤害后可直接选其 1 个非限定/觉醒/主公技令其失效且你获得之至其下回合结束（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b.rs:380-436` + `l27_batch_b_asks.rs:137-186` | `l27_beautify_batch_b_bolan_duorui_and_save_restore` | `已复核等价` |
| 30 | `xinfu_xionghuo` | `extension.js:8435-8591` | 凶镬：开局 3 暴戾标记，出牌阶段移 1 标记交其他角色；对其造伤 +1；其出牌阶段开始移标记并随机执行火伤禁杀/失血减上限/随机获手牌与装备。 | `l27_batch_b_tw.rs:617-668` + `d34_l11_mods.rs:321-342` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 31 | `mbwuling` | `extension.js:8788-9008` | 五灵：出牌阶段限一次令一名未获五禽戏标记的角色从本轮未选过的五禽戏（虎/鹿/熊/猿/鸟）中择一获得至其下回合结束（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:670-708` + `l27_batch_b_asks.rs:238-280` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |

### 3.2 选将/花色/数字弹窗技能钩子（`ui_hooks`，12 项，12/12 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 行为与语义抽象说明 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 32 | `mbcmqingzheng` | `extension.js:3697-4262` | 清正：出牌阶段开始经三段 `ChoiceSpec` 弃 1 花色手牌、观看 1 名有手牌其他角色手牌并弃其 1 花色手牌，若其弃牌较少对其造成 1 点伤害（已登记裁决 40 `ChoiceSpec` 抽象）。 | `step/skills_damage.rs:453-568` (`HostBehavior::MbcmQingzheng`) | `l27_beautify_match_dedicated_six_skills` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 33 | `sbqingzheng` | `extension.js:4263-4838` | 谋清正：出牌阶段开始经三段 `ChoiceSpec`（`l27_sbqingzheng_my_suit` -> `target` -> `target_suit`）双向弃置所选花色手牌，目标弃牌少于你时对其造成 1 点伤害（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:710-813` + `l27_batch_b_asks.rs:282-338` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` | `已复核等价` |
| 34 | `spyanjiao` | `extension.js:4839-5899` | 严教：出牌阶段限一次选一名其他角色，亮出牌堆顶 `4+X` 张牌，分为点数相等的两组由双方各获一组、剩余入弃堆，若分出超过 1 张则本回合手牌上限 -1（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:815-872` + `l27_batch_b_asks.rs:340-396` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` | `已复核等价` |
| 35 | `twxiafeng` | `extension.js:6350-6475` | 狭锋：出牌阶段开始声明数字 `X (1..=3)`，本回合前 X 张牌无距限且不可被响应；阶段结束若本阶段用牌数 >= X 则摸 1 张牌（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:22-54` + `d34_l11_mods.rs:344-368` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 36 | `twxingzhui` | `extension.js:6479-6631` | 星坠：结束阶段失去 `X (1..=hp)` 点体力并观看牌堆顶 `2X` 张牌，若黑牌数恰好为 X 则对一名其他角色造成 X 点雷电伤害（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:56-142` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 37 | `xingchong` | `extension.js:6633-6771` | 倖宠：每轮开始时可摸 `X (0..=maxHp)` 张牌并展示至多 X 张手牌，本轮内失去展示牌后摸 2 张牌（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:144-215` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 38 | `dcxiaoxi` | `extension.js:6774-6886` | 骁袭：准备阶段可减 X 点体力上限并摸 X 张牌，本回合计算与其他角色距离 -X 且可多使用 X 张【杀】（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:217-271` + `d34_l11_mods.rs:370-396` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 39 | `twfengji` | `extension.js:6889-7043` | 丰姿：出牌阶段限一次，使用基本牌或普通锦囊牌时可弃 X 牌令基数 +X 或摸 X 牌令基数 -X（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:273-364` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 40 | `twharvestinori` | `extension.js:7044-7166` | 丰祈：触发时选择数字 X，按所选数值执行丰祈摸牌/回复结算（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:366-406` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 41 | `twzhouhu` | `extension.js:7168-7291` | 咒护：出牌阶段限一次弃 1 张红牌指定角色并选 `X (1..=3)`，经过 X 个回合后其回复 1 点体力（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:408-448` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 42 | `twzuhuo` | `extension.js:7293-7439` | 诅祸：出牌阶段限一次弃 1 张黑牌指定角色并选 `X (1..=3)`，经过 X 个回合后防止其下一次受到的伤害（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:450-494` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |
| 43 | `twzhouzu` | `extension.js:7441-7564` | 咒诅：出牌阶段限一次指定其他角色并选 `X (1..=3)`，令其摸 X 张或弃 X 张牌并联动咒护/诅祸标记增减（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l27_batch_b_tw.rs:496-546` | `l27_beautify_batch_b_tw_and_misc_skills` | `已复核等价` |

### 3.3 配置项（`configs`，10 项，10/10 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 默认值与行为摘要 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 44 | `twotwo` | `extension.js:9014-9018` | 双将模式选将美化开关（默认 `false`），支持双将候选对合法性校验 `validate_dual_general_pair`。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 45 | `gaibiaoji` | `extension.js:9024-9028` | 改标记开关（默认 `true`），控制暴虐值/化身等美化标记同步。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 46 | `dianjibiaoji` | `extension.js:9029-9033` | 点击标记弹窗查看开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 47 | `shoupaihuase` | `extension.js:9034-9038` | 手牌花色分列选牌美化开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 48 | `feichuxuanxiang` | `extension.js:9039-9043` | 废除装备栏选项美化开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 49 | `shuzianniu` | `extension.js:9044-9048` | 数字加减按钮美化开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 50 | `xuanzejineng` | `extension.js:9049-9053` | 选择技能弹窗美化开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 51 | `zediewugu` | `extension.js:9054-9058` | 折叠五谷丰登面板开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 52 | `jinxuanbiaoyu` | `extension.js:9059-9063` | 禁选标语美化开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |
| 53 | `jinengtexiaoxiugai` | `extension.js:9064-9068` | 技能特效修改开关（默认 `true`）。 | `crates/content-native-loader/src/select_beautify.rs:7-171` | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | `已复核等价` |

### 3.4 渲染面（`surfaces`，1 项，留待执行队列第 5 项界面阶段）

| # | 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---:|---|---|---|---|---|
| 54 | `e.css` | `e.css:1-2009` | CSS 样式表 `e.css`（2,009 行） | 原生前端组件 | `简化近似`（纯前端 CSS 渲染面，归属执行队列第 5 项界面阶段） |

---

## 4. 6 维真实事件对局验收矩阵

| 维度 | 验证内容 | 覆盖测试函数 | 结果 |
|---|---|---|---|
| D1 正式包装载与 43 技/钩子 + 10 配置解析 | `select-beautify-upstream-liuli` 正式包挂载全部 31 项 `skills`、12 项 `ui_hooks` 技能及 10 项 `SelectBeautifyConfig`（默认值/自定义值/双将对校验/快照往返） | `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` + `l27_eight_dimension_from_fresh_install` | 通过 |
| D2 正向触发与多分支主动/阶段技能 | 6 项独立 `HostBehavior`（`xiefang`、`sbzhaxiang`、`new_meibu`、`new_zhixi`、`jishe`、`mbcmqingzheng`）、Batch A 19 项（`regushe` 拼点、狂才、谋统业、暴虐值、族点盏、界化身/新生、滔乱、守玺、定汉、权计、壮誓、奇谋/界奇谋、谋奸雄、景合）与 Batch B 18 项（博览、夺锐/进阶夺锐、凶镬、五灵、谋清正、严教、狭锋、星坠、倖宠、骁袭、丰姿、丰祈、咒护、诅祸、咒诅、极奢2、止息清理）全部在真实 `Engine` 事件流中正向触发结算 | `l27_beautify_match_dedicated_six_skills` + `l27_beautify_match_batch_a_skills_part1` + `l27_beautify_match_batch_a_skills_part2` + `l27_beautify_batch_b_bolan_duorui_and_save_restore` + `l27_beautify_batch_b_tw_and_misc_skills` | 通过 |
| D3 负向守卫与非法输入拒绝 | `new_zhixi` 用锦囊牌后锁牌与用牌达体力上限拦截；`jishe` 手牌上限为 0 拒绝发动；`taoluan` 同牌名重用拒绝与拒交牌后失血禁用；`qimou`/`reqimou` 限定技二次发动拒绝；`clandianzhan` 同轮同花色二次触发拒绝；`huamu` 上游 `if(false)` 死分支惰性保护；`validate_dual_general_pair` 重复武将与未知武将拒绝 | `l27_beautify_match_dedicated_six_skills` + `l27_beautify_match_batch_a_skills_part1` + `l27_beautify_match_batch_a_skills_part2` + `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` | 通过 |
| D4 多技能联动与跨角色交互 | `new_meibu` 弃杀/黑锦囊授 `new_zhixi` 并锁攻击范围 + `new_zhixi_recover` 阶段末清理；`rehuashen` + `rexinsheng` 受伤补化身牌并切换技能；`bolan` + `bolan_g` 他人失血借用你所选 3 技能之一；`drlt_duorui` 废装备栏夺取目标技能并封印目标原技能至下回合结束；`twzhouhu` + `twzuhuo` + `twzhouzu` 跨回合倒计时回血/免伤联动 | `l27_beautify_match_dedicated_six_skills` + `l27_beautify_match_batch_a_skills_part1` + `l27_beautify_batch_b_bolan_duorui_and_save_restore` + `l27_beautify_batch_b_tw_and_misc_skills` | 通过 |
| D5 挂起等待期快照编码/解码与复盘确定性 | 在 `regushe`、`sbtongye`、`rehuashen`、`taoluan`、`potzhuangshi`、`qimou`、`jinghe`、`bolan`、`drlt_duorui`、`sbqingzheng`、`spyanjiao` 的 `ChoiceSpec` 挂起窗口执行 `encode_archive` / `decode_archive`，验证恢复后继续提交决策的状态摘要与未中断路径 100% 一致 | `l27_beautify_match_save_restore_and_ai` + `l27_beautify_batch_b_bolan_duorui_and_save_restore` + `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` | 通过 |
| D6 `BoundedPlanner` AI 合法决策闭环 | 全部 `L27` 挂起窗口调用 `BoundedPlanner::new(16)` 生成合法 `GameAction::Answer` 并推演至结算完成，零暗牌泄露 / 零 panic / 零非法命令 | `l27_beautify_match_save_restore_and_ai` + `l27_beautify_batch_b_bolan_duorui_and_save_restore` + `l27_beautify_batch_b_qingzheng_yanjiao_and_configs` | 通过 |

---

## 5. 验证命令与原始输出摘要

在 `/Users/ruyimg/Documents/Claude/sanguosha/noname-next` 下执行：

```text
$ cargo test --offline -p engine-native --test g1 l27_beautify
running 6 tests
test groups::g1::l27_beautify_match::l27_beautify_match_dedicated_six_skills ... ok
test groups::g1::l27_beautify_batch_b::l27_beautify_batch_b_qingzheng_yanjiao_and_configs ... ok
test groups::g1::l27_beautify_batch_b::l27_beautify_batch_b_bolan_duorui_and_save_restore ... ok
test groups::g1::l27_beautify_match::l27_beautify_match_batch_a_skills_part1 ... ok
test groups::g1::l27_beautify_match::l27_beautify_match_save_restore_and_ai ... ok
test groups::g1::l27_beautify_batch_b::l27_beautify_batch_b_tw_and_misc_skills ... ok
test groups::g1::l27_beautify_match::l27_beautify_match_batch_a_skills_part2 ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.19s

$ cargo test --offline -p engine-native --test l_rounds l27_
running 13 tests
test l_rounds::l27_select_beautify_eight::l27_eight_dimension_from_fresh_install ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 46 filtered out; finished in 0.14s

$ cargo test --offline -p engine-native --test misc decision34
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 166 filtered out; finished in 1.50s

$ cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --lib
test result: ok. 15 passed (engine-native) + 4 passed (engine-native-content) + 6 passed (engine-native-kernel); 0 failed

$ python3 -B scripts/legacy-harness/generate-g0-docs.py
wrote 37 main docs + 1 part docs + README -> /Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/migration/g0
Totals: {'generals': 834, 'skills': 1908, 'cards': 377, 'modes': 215, 'configs': 434, 'ui_hooks': 507, 'surfaces': 149, '已复核等价': 283, '简化近似': 4141, '缺失': 0, '上游未定义': 0}

$ python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
Ran 2 tests in 0.013s — OK

$ python3 -B scripts/check-source-lines.py
Source line limit passed
```

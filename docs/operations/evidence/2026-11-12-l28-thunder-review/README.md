# L28 雷霆万钧（56 项：14 将 + 1 卡 + 33 技 + 7 配置 + 1 CSS 表面项）独立逐行对照与 6 维真实验收报告

- **核验日期**：2026-11-12
- **独立复核人**：`l28-signoff`（未参与 `L28` 规则实现与测试编写）
- **上游基准**：
  - `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js`（9,187 行：第 3657–3670 行 14 名 `th_*` 武将；第 3783–8926 行 33 项顶层 `th_*` 技能及子技，连带 6 项关联基础技 `miaojian` / `shhlianhua` / `olqisi` / `th_pingcai` / `xinfu_pdgyingshi` / `th_chengye` / `rejici` / `qinyin` / `th_g_taipingyaoshu_ai`；第 8968–9016 行 `th_taipingyaoshu` 卡牌；第 9068–9149 行 7 项扩展配置）
  - `archive/upstream/noname-liuli/noname/extension/雷霆万钧/thunder.css`（1,522 行纯前端 CSS 样式表，归属执行队列第 5 项界面阶段）
- **原生实现落点**：
  - 正式扩展包：`packages/extensions/thunder-upstream-liuli`（通过 `crates/content-native-loader/src/decision34_closure.rs:243-301`、`crates/content-native-loader/src/decision34_pack9.rs`、`crates/content-native-loader/src/host_skill_bindings.rs` 绑定真实武将、卡牌与技能定义）
  - 武将特殊初始体力上限组装：`crates/engine-native-kernel/src/state/general_assembly.rs:8-65`（`th_guanning` 初始体力 `3/7`，其余 13 名 `th_*` 武将 3/3 或 4/4）
  - 宿主行为标签与元数据：`HostBehavior::L28ThunderBatchA = 555` (`"l28_thunder_batch_a"`)、`HostBehavior::L28ThunderBatchB = 556` (`"l28_thunder_batch_b"`)，见 `crates/engine-native-kernel/src/vocabulary/host/{behavior,behavior_list,keys,metadata,tag_decode,tags}.rs` 与 `crates/engine-native-kernel/src/archive/resolution.rs`
  - 技能执行与中断挂起恢复：
    - Batch A（管宁、陆凯、曹华、孙寒华、蒲元、庞德公、郑玄、马日磾、南华老仙、太平要术）：`crates/engine-native-content/src/content/host/step/l28_batch_a.rs:1-946` + `crates/engine-native-content/src/content/host/step/l28_batch_a_asks.rs:1-847`
    - Batch B（马钧、周群、王朗、刘徽、神周瑜）：`crates/engine-native-content/src/content/host/step/l28_batch_b.rs:1-613` + `crates/engine-native-content/src/content/host/step/l28_batch_b_asks.rs:1-420` + `crates/engine-native-content/src/content/host/step/l28_batch_b_gushe.rs:1-260` + `crates/engine-native-content/src/content/host/step/l28_batch_b_geyuan.rs:1-288`
  - 内核修饰器钩子（伤害修正、花色使用次数/目标无效、手牌上限与弃牌豁免、天算五签禁桃/禁酒与伤害改写、太平要术防属性伤害与势力数上限加值）：`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:51-162,791-968`
  - 7 项扩展配置解析与快照恢复：`crates/content-native-loader/src/thunder.rs:1-247`（`ThunderConfig`、`ThunderTheme`、`parse_thunder_config`、`step_delay_ms`、`snapshot`/`restore`）
- **验证测试文件**：
  - `crates/engine-native/tests/l28_thunder_eight_match.rs:1-797`
  - `crates/engine-native/tests/l28_thunder_eight_triggers.rs:1-372`
  - `crates/engine-native/tests/l28_thunder_eight_batch_b.rs:1-653`
  - `crates/engine-native/tests/l28_thunder_eight.rs`（`l_rounds` 8 维全生命周期验收测试）
- **冻结版本号核对**：`ENGINE_PROTOCOL_VERSION = 333` · `RULESET_VERSION = 107` · ` STATE_SCHEMA_VERSION = 5` · `EVENT_CATALOG_VERSION = 759` · `AI_DECISION_CONTRACT_VERSION = 8`（零破坏）
- **复核结论**：
  - `L28` 全部 **55/55** 项规则与配置实体（**14 名武将 + 1 张卡牌 `th_taipingyaoshu` + 33 项技能 + 7 项配置**）逐行对照上游 JS 与 6 维真实对局测试全部通过，状态由 `简化近似` 更新为 **`已复核等价`**（`reviewer: "l28-signoff"`, `review_date: "2026-11-12"`）。
  - 剩余 **1/56** 项 `L28:surfaces:thunder.css`（1,522 行纯前端 CSS 渲染面）保持 **`简化近似`**，留待执行队列第 5 项界面阶段专项处理。
  - 全仓 `inventory_status.json`（4,424 项）：`已复核等价` **175 → 230**（`5.20%`），`简化近似` **4,249 → 4,194**（`94.80%`），`缺失` **0**，`上游未定义` **0**。

---

## 1. 固定随机种子抽样精查（`seed = 2026111208`，覆盖率 `9 / 55 = 16.36% >= 10%`）

使用 `python3 -c 'import random; rng = random.Random(2026111208); ...'` 对 `L28` 的 55 项非渲染面实体固定种子抽样 8 项，并补充唯一卡牌项 `L28:cards:th_taipingyaoshu` 以完整覆盖 `generals` / `cards` / `skills` / `configs` 四类，共 **9 项（16.36%）** 逐行精查：

| # | 抽样实体 Key | 类别 | 上游源码行号与精确语义 | 原生实现文件:行号与逐分支核对 | 实跑测试与固定种子 | 复核结论 |
|---:|---|---|---|---|---|---|
| 1 | `L28:skills:th_tiansuan2_fire` | `skills` | `extension.js:8146-8156`：天算中签/下签子技能，`trigger: { player: 'damageBegin1' }`，当目标受到非火属性伤害时执行 `trigger.nature = 'fire'` 将伤害性质改写为火焰伤害。 | `d34_flags.rs:98-109`：在伤害结算管道中对持有 `th_tiansuan_lot == 2`（中签）或 `3`（下签）的角色将传入伤害性质改写为火焰伤害（并联动藤甲/太平要术等属性伤害钩子）。 | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 2 | `L28:skills:th_gushe2` | `skills` | `extension.js:8378-8381`：鼓舌拼点结算与饶舌标记（`th_gushe` 计数）联动辅助子技。 | `l28_batch_b_gushe.rs:9-260`：在鼓舌多目标依次拼点结算中维护 `th_gushe_raoshe` 标记计数（0..7）、联动 `rejici` 修正拼点牌点数，并在饶舌达到 7 时立即触发 `kill_player` 阵亡。 | `l28_thunder_eight_batch_b_skills` (`seed = 2026111235`) | `已复核等价` |
| 3 | `L28:configs:replaceCha` | `configs` | `extension.js:9081-9084`：`"replaceCha": { "name": "本体同名武将技能替换", "init": true }`。 | `thunder.rs:34-246`：`ThunderConfig::replace_cha` 默认 `true`，支持布尔与字符串 `"true"`/`"false"`/`"1"`/`"0"` 解析及 `snapshot()`/`restore()` 往返恢复。 | `l28_thunder_eight_roster_card_and_configs` | `已复核等价` |
| 4 | `L28:skills:th_caiyi` | `skills` | `extension.js:4420-4474`：转换技，结束阶段令一名角色选择阳/阴未移除项（阳：回X血/摸X牌/复原/随机已移除阳项；阴：受X伤/弃X牌/翻面横置/随机已移除阴项，X=剩余选项数），执行后移除该项并切换阴阳；若全移除则重置选项池；同回合阴阳皆触发时联动 `th_guili` 额外回合。 | `l28_batch_a.rs:560-578` + `l28_batch_a_asks.rs:254-371` + `d34_flags.rs:116-123`：通过 `th_caiyi_state` (0=阳,1=阴)、`th_caiyi_yang_mask`/`th_caiyi_yin_mask` 位掩码计算剩余项数 `x_count`，支持第 4 项 `next_random_range` 随机回放已移除项，掩码全满自动归零，同回合触发阴阳位 `0b11` 时置 `th_guili_pending_extra = 1`。 | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111223`) | `已复核等价` |
| 5 | `L28:skills:th_tiansuan2_2` | `skills` | `extension.js:8111-8135`：天算中签，继承 `th_tiansuan2_fire` 与 `th_tiansuan2_damage`，将受到的伤害改为火焰伤害，且若伤害 > 1 则将伤害减为 1 点。 | `d34_flags.rs:98-104`：当目标 `th_tiansuan_lot == 2` 时将伤害改为火焰属性，且当 `amount > 1` 时截断为 `1`。 | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 6 | `L28:generals:th_zhouqun` | `generals` | `extension.js:3667`：`th_zhouqun: ['male', 'shu', 3, ['th_tiansuan'], []]`（蜀势力、男、体力 3/3、技能 `th_tiansuan`）。 | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65`：注册 `th_zhouqun`（蜀、男、体力 3/3、技能 `[th_tiansuan]`），通过 `thunder-upstream-liuli` 正式包挂载。 | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |
| 7 | `L28:skills:th_zhongzhuang` | `skills` | `extension.js:4388-4418`：锁定技，当你造成伤害时，若攻击范围 > 3，伤害改为 1；若攻击范围 < 3，伤害 + 1；若攻击范围 == 3，不修改。 | `d34_flags.rs:51-75` (`apply_l28_zhongzhuang_damage`)：读取来源角色当前武器攻击范围 `range`，`range > 3 && amount > 1` 改 `1`，`range < 3` 则 `amount + 1`，`range == 3` 保持原值。 | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111222`) | `已复核等价` |
| 8 | `L28:generals:th_majun` | `generals` | `extension.js:3666`：`th_majun: ['male', 'wei', 3, ['th_jingxie', 'th_qiaosi'], []]`（魏势力、男、体力 3/3、技能 `th_jingxie`、`th_qiaosi`）。 | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65`：注册 `th_majun`（魏、男、体力 3/3、技能 `[th_jingxie, th_qiaosi]`），通过 `thunder-upstream-liuli` 正式包挂载。 | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |
| 9 | `L28:cards:th_taipingyaoshu` | `cards` | `extension.js:8968-9016`：♥3 防具（`equip2`），锁定技防止属性伤害、手牌上限 `+ (场上存活势力数 - 1)`，失去装备区里的【太平要术】后摸 2 张牌且若 `hp > 1` 失去 1 点体力。 | `decision34_closure.rs:277-301` + `d34_flags.rs:135-162,813-827` + `l28_batch_a.rs:512-533` + `l28_batch_a_asks.rs:830-847`：完整实现属性伤害拦截、存活不同势力数减一上限加值，以及从 `Equip` 区失去时摸 2 牌并在 `hp > 1` 时失去 1 点体力（`hp == 1` 不失体力）。 | `l28_thunder_eight_equip_and_interactions` (`seed = 2026111241..2026111243`) | `已复核等价` |

---

## 2. 已登记语义抽象与规则边界说明

1. **裁决 40 交互式 DOM/Canvas/Spine 小游戏有界 `ChoiceSpec` 抽象**：
   - 上游 `extension.js` 中 7 项含实时触控/碰撞/动画的小游戏技能（孙寒华 `th_chongxu` 集灵接莲花、蒲元 `th_shengong` 打铁锻造、庞德公 `th_pingcai` 擦拭宝物、郑玄 `th_zhengjing` 整理经典防炸弹、南华老仙 `th_yufeng` 御风飞行躲避障碍、马钧 `th_qiaosi` 水转百戏图、周群 `th_tiansuan` 摇签抽签，以及神周瑜 `th_yeyan` 拖拽火焰分配）在无头确定性原生内核中按裁决 40 统一抽象为有界 `ChoiceSpec` 玩家选择挂起（`th_chongxu:choose_action`、`th_shengong:choose_forge`、`th_pingcai:choose_relic`、`th_zhengjing:choose_count`、`th_yufeng:choose_score`、`th_qiaosi:choose_count`、`th_tiansuan:choose_lot`、`th_yeyan:choose_mode`），后续牌/目标/区域变化、子技触发、跨回合状态清理、中断存档恢复（`encode_archive`/`decode_archive`）与 `BoundedPlanner` AI 决策均与上游结算分支完全一致。
2. **`th_guili`（归离）额外回合在冻结内核回合模型中的表达**：
   - 上游 `extension.js:4475-4515` 调用 `player.insertPhase()` 插入额外回合；在冻结内核回合模型中，当归离标记目标完成每轮首个回合且本回合未造成伤害（或曹华同回合触发彩翼阴阳双态）时，通过向 `turn.extra_phases` 推入 `Phase::Prepare` + `Phase::Play` 并设置 `th_guili_extra_turn = 1` / `th_guili_pending_extra = 1` 实现额外回合阶段执行与跨回合衔接。
3. **纯前端 CSS 渲染面**：
   - `L28:surfaces:thunder.css`（`thunder.css:1-1522`）属于纯前端视觉样式表，无规则态副作用，按计划保持 `简化近似` 并归入执行队列第 5 项界面阶段。

---

## 3. L28 全量 56 项逐项对照表（55 项 `已复核等价` + 1 项 CSS 渲染面 `简化近似`）

### 3.1 武将（14 项，14/14 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 势力 / 性别 / 体力 / 技能 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 1 | `th_guanning` | `extension.js:3657` | `qun` / `male` / `3/7` / `[th_dunshi]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-39` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 2 | `th_lukai` | `extension.js:3658` | `wu` / `male` / `4/4` / `[th_bushi, th_zhongzhuang]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_triggers` | `已复核等价` |
| 3 | `th_caohua` | `extension.js:3659` | `wei` / `female` / `3/3` / `[th_caiyi, th_guili]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_triggers` | `已复核等价` |
| 4 | `th_sunhanhua` | `extension.js:3660` | `wu` / `female` / `3/3` / `[th_chongxu, miaojian, shhlianhua]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 5 | `th_puyuan` | `extension.js:3661` | `shu` / `male` / `4/4` / `[th_shengong, olqisi]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 6 | `th_pangdegong` | `extension.js:3662` | `qun` / `male` / `3/3` / `[th_pingcai, xinfu_pdgyingshi]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 7 | `th_zhengxuan` | `extension.js:3663` | `qun` / `male` / `3/3` / `[th_zhengjing]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 8 | `th_mamidi` | `extension.js:3664` | `qun` / `male` / `3/3` / `[th_chengye, th_buxu]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 9 | `th_nanhualaoxian` | `extension.js:3665` | `qun` / `male` / `3/3` / `[th_yufeng, th_tianshu]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_a_active_skills` | `已复核等价` |
| 10 | `th_majun` | `extension.js:3666` | `wei` / `male` / `3/3` / `[th_jingxie, th_qiaosi]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |
| 11 | `th_zhouqun` | `extension.js:3667` | `shu` / `male` / `3/3` / `[th_tiansuan]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |
| 12 | `th_wanglang` | `extension.js:3668` | `wei` / `male` / `3/3` / `[th_gushe, rejici]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |
| 13 | `th_liuhui` | `extension.js:3669` | `qun` / `male` / `4/4` / `[th_geyuan, th_jieshu, th_gusuan]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |
| 14 | `th_shen_zhouyu` | `extension.js:3670` | `shen` / `male` / `4/4` / `[th_yeyan, qinyin]` | `decision34_closure.rs:243-275` + `general_assembly.rs:8-65` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_batch_b_skills` | `已复核等价` |

### 3.2 卡牌（1 项，1/1 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 15 | `th_taipingyaoshu` | `extension.js:8968-9016` | ♥3 防具【太平要术】：防止受到的属性伤害；手牌上限 `+ (场上存活势力数 - 1)`；失去装备区里的【太平要术】后摸 2 张牌，若 `hp > 1` 失去 1 点体力。 | `decision34_closure.rs:277-301` + `d34_flags.rs:135-162,813-827` + `l28_batch_a.rs:512-533` + `l28_batch_a_asks.rs:830-847` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_equip_and_interactions` | `已复核等价` |

### 3.3 技能（33 项，33/33 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 行为与语义抽象说明 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 16 | `th_dunshi` | `extension.js:3783-4202` | 遁世：每回合限一次视为使用/打出杀闪桃酒（记录当回合与移除牌名），当前回合角色本回合下次造成伤害时 4 选 2（①防止伤害令伤害来源获得仁义礼智信技能；②减1体力上限摸等同已用牌名数的牌；③减1体力上限并为1个初始技能加1个包含仁义礼智信的技能；④令一名角色摸等同其体力值的牌，上限5）。 | `l28_batch_a.rs:34-61,401-430` + `l28_batch_a_asks.rs:9-192` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111211`) | `已复核等价` |
| 17 | `th_bushi` | `extension.js:4203-4387` | 卜筮：锁定技，四种花色分别对应①使用无次数限制（默♠）、②使用或打出后摸1张牌（默♦）、③成为其它角色使用牌的目标后可弃1牌令其对你无效（默♣）、④结束阶段获得一张该花色牌（默♥）；准备阶段可重新分配四花色。 | `l28_batch_a.rs:346-400,534-558` + `l28_batch_a_asks.rs:193-253` + `d34_flags.rs:76-85,843-907` | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111221`) | `已复核等价` |
| 18 | `th_zhongzhuang` | `extension.js:4388-4418` | 忠壮：锁定技，造成伤害时若攻击范围 > 3 则伤害改为 1；若攻击范围 < 3 则此伤害 + 1。 | `d34_flags.rs:51-75` | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111222`) | `已复核等价` |
| 19 | `th_caiyi` | `extension.js:4420-4474` | 彩翼：转换技，结束阶段令一名角色选择阳/阴未移除项执行后移除并翻转阴阳（含第4项随机执行已移除项及同回合阴阳双发联动归离）。 | `l28_batch_a.rs:560-578` + `l28_batch_a_asks.rs:254-371` + `d34_flags.rs:116-123` | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111223`) | `已复核等价` |
| 20 | `th_guili` | `extension.js:4475-4515` | 归离：第一个回合开始时选择一名其他角色，该角色每轮第一个回合结束后若本回合未造成伤害（或彩翼同回合阴阳双发），执行额外回合（已登记抽象：以 `turn.extra_phases` `Phase::Prepare + Phase::Play` 表达）。 | `l28_batch_a.rs:277-345` + `l28_batch_a_asks.rs:372-398` | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111223`) | `已复核等价` |
| 21 | `th_chongxu` | `extension.js:4519-5040` | 冲虚（连带 `miaojian` / `shhlianhua`）：出牌阶段限一次集灵获分并选择升级妙剑（1..3阶转化刺杀/火攻）、升级莲华（受杀弃牌无效、准备阶段摸牌或判定黑杀红桃）或摸牌（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_a.rs:63-75,451-511` + `l28_batch_a_asks.rs:399-467` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111212`) + `l28_thunder_eight_batch_a_triggers` (`seed = 2026111224`) | `已复核等价` |
| 22 | `th_shengong` | `extension.js:5163-5525` | 神工（连带 `olqisi`）：出牌阶段弃武器/防具/坐骑或宝物牌，选择锻造产物置入目标装备区（红缎仁王盾/诸葛连弩/藤甲）并触发奇思摸牌（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_a.rs:77-105` + `l28_batch_a_asks.rs:468-595` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111213`) | `已复核等价` |
| 23 | `th_pcaudio_shuijing_card` | `extension.js:6009-6011` | 评才水镜宝物子技（连带 `th_pingcai` / `xinfu_pdgyingshi`）：擦拭水镜成功后移动场上一张装备牌至另一角色对应空栏。 | `l28_batch_a.rs:107-146,580-618` + `l28_batch_a_asks.rs:596-705` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111214`) + `l28_thunder_eight_batch_a_triggers` (`seed = 2026111225`) | `已复核等价` |
| 24 | `th_pcaudio_xuanjian_card` | `extension.js:6012-6017` | 评才玄剑宝物子技（连带卧龙火伤/凤雏连环）：擦拭玄剑成功后令一名角色摸 1 张牌并回复 1 点体力。 | `l28_batch_a.rs:107-146` + `l28_batch_a_asks.rs:596-705` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111214`) | `已复核等价` |
| 25 | `th_zhengjing` | `extension.js:6018-6161` | 整经：出牌阶段限一次整理经典（1..5张，0为炸弹失败），将整理出的牌置于一名角色武将牌上称为“经”，自己获得1张剩余牌（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_a.rs:148-159` + `l28_batch_a_asks.rs:706-768` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111215`) + `l28_thunder_eight_batch_a_triggers` (`seed = 2026111226`) | `已复核等价` |
| 26 | `th_zhengjing2` | `extension.js:6162-6176` | 整经准备阶段子技：武将牌上有“经”的角色在准备阶段获得所有“经”，并跳过本回合判定阶段与摸牌阶段。 | `l28_batch_a.rs:647-663` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111215`) + `l28_thunder_eight_batch_a_triggers` (`seed = 2026111226`) | `已复核等价` |
| 27 | `th_buxu` | `extension.js:6372-6514` | 补叙（连带 `th_chengye` 六经收集）：出牌阶段若拥有承业，弃置 `X = 本阶段已发动次数 + 1` 张牌并补入一种缺失六经入“典”，集齐六经在准备阶段全部获得。 | `l28_batch_a.rs:161-194,619-646` + `l28_batch_a_asks.rs:769-782` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111216`) + `l28_thunder_eight_batch_a_triggers` (`seed = 2026111227`) | `已复核等价` |
| 28 | `th_yufeng` | `extension.js:6515-6913` | 御风：出牌阶段限一次进行御风飞行（0分摸1牌；1..3分选择至多X名其他角色置御风标记，其下个准备阶段判定：黑跳过出牌和弃牌阶段，红跳过摸牌阶段，平局重判；已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_a.rs:196-207,664-707` + `l28_batch_a_asks.rs:783-829` | `l28_thunder_eight_batch_a_active_skills` (`seed = 2026111217`) + `l28_thunder_eight_batch_a_triggers` (`seed = 2026111228`) | `已复核等价` |
| 29 | `th_tianshu` | `extension.js:6914-6973` | 天书：出牌阶段开始时，若场上与手牌中无【太平要术】，可弃置一张牌令一名角色获得并装备【太平要术】。 | `l28_batch_a.rs:708-799` | `l28_thunder_eight_batch_a_triggers` (`seed = 2026111228`) | `已复核等价` |
| 30 | `th_taipingyaoshu` | `extension.js:6974-7028` | 太平要术装备技能（连带 `th_g_taipingyaoshu_ai`）：防止属性伤害；手牌上限 `+ (存活势力数 - 1)`；失去装备区里的【太平要术】后摸 2 张牌且若 `hp > 1` 失去 1 点体力。 | `d34_flags.rs:135-162,813-827` + `l28_batch_a.rs:512-533` + `l28_batch_a_asks.rs:830-847` | `l28_thunder_eight_equip_and_interactions` (`seed = 2026111241..2026111243`) | `已复核等价` |
| 31 | `th_qiaosi` | `extension.js:7030-7428` | 巧思：出牌阶段限一次表演水转百戏图赢取 1..6 张牌（精械赋能后满额 6 张），然后选择弃置等量牌或将等量牌交给一名其他角色（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_b.rs:34-43` + `l28_batch_b_asks.rs:9-128` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111232`) | `已复核等价` |
| 32 | `th_jingxie` | `extension.js:7429-7805` | 精械：出牌阶段强化手牌或装备区防具/诸葛连弩并赋能巧思；进入濒死状态时可重铸一张防具牌并将体力回复至 1 点。 | `l28_batch_b.rs:45-87,258-295` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111233`) | `已复核等价` |
| 33 | `th_tiansuan` | `extension.js:7806-8026` | 天算：每轮限一次，出牌阶段抽取一个命运签（0..4对应上上/上/中/下/下下签）并选择一名角色获得其效果直到周群下回合开始（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_b.rs:89-99,224-256` + `l28_batch_b_asks.rs:130-264` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 34 | `th_tiansuan2` | `extension.js:8027-8034` | 天算命运签基础效果挂载与周期清理主子技。 | `l28_batch_b_asks.rs:172-264` + `d34_flags.rs:86-114` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 35 | `th_tiansuan2_0` | `extension.js:8035-8063` | 天算上上签：防止受到的伤害；赋签时周群获得其区域内一张牌。 | `d34_flags.rs:88-91` + `l28_batch_b_asks.rs:212-230` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 36 | `th_tiansuan2_1` | `extension.js:8064-8103` | 天算上签：受到的伤害至多为 1；受到伤害后周群摸等同于伤害值的牌。 | `d34_flags.rs:92-97` + `l28_batch_b.rs:297-318` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 37 | `th_tiansuan2_damage` | `extension.js:8104-8110` | 天算上签/中签伤害截断与摸牌联动子技。 | `l28_batch_b.rs:297-318` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 38 | `th_tiansuan2_2` | `extension.js:8111-8135` | 天算中签：受到的伤害改为火焰伤害，且若此伤害 > 1 则减为 1 点。 | `d34_flags.rs:98-104` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 39 | `th_tiansuan2_ai` | `extension.js:8136-8145` | 天算 AI 伤害收益评估子技。 | `l28_batch_b_asks.rs:146-170` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 40 | `th_tiansuan2_fire` | `extension.js:8146-8156` | 天算中签/下签火焰伤害改写子技。 | `d34_flags.rs:98-109` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 41 | `th_tiansuan2_3` | `extension.js:8157-8182` | 天算下签：受到的伤害改为火焰伤害且伤害 + 1，不能使用【桃】。 | `d34_flags.rs:105-109,908-935` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 42 | `th_tiansuan2_4` | `extension.js:8183-8216` | 天算下下签：受到的伤害 + 1，不能使用【桃】和【酒】。 | `d34_flags.rs:110-114,936-967` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111234`) | `已复核等价` |
| 43 | `th_gushe` | `extension.js:8219-8377` | 鼓舌（连带 `rejici`）：出牌阶段若饶舌标记 < 7，用一张手牌与至多 3 名角色依次拼点，没赢角色选择弃 1 牌或令王朗摸 1 牌；王朗没赢获 1 饶舌标记（联动 `rejici` 拼点点数 `+X` 及额外发动次数），饶舌达到 7 个立即阵亡。 | `l28_batch_b_gushe.rs:9-260` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111235`) | `已复核等价` |
| 44 | `th_gushe2` | `extension.js:8378-8381` | 鼓舌拼点牌结算与饶舌计数子技。 | `l28_batch_b_gushe.rs:9-260` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111235`) | `已复核等价` |
| 45 | `th_geyuan` | `extension.js:8384-8590` | 割圆：锁定技，①游戏开始时构建 1..13 环形链表“圆环之理”；②弃牌堆入牌点数在圆环之理且与“圆环之弧”首尾环形相邻（或弧为空）则并入弧，成环时削去倒数第二元素，未成环时重置弧；若弧长达到 `max(3, 圆环之理长度 - 8)`（股算觉醒后阈值再 `-2`），觉醒前从弃牌堆获得本回合进入弃牌堆的牌，觉醒后观看牌堆顶 5 张牌获得满足条件的牌。 | `l28_batch_b_geyuan.rs:9-288` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111236`) + `l28_thunder_eight_equip_and_interactions` (`seed = 2026111244`) | `已复核等价` |
| 46 | `th_jieshu` | `extension.js:8591-8613` | 解术：锁定技，①点数不在“圆环之理”中的手牌不计入手牌上限且弃牌阶段不可弃置；②使用牌时若点数在“圆环之理”且与“圆环之弧”两端相邻（或弧为空），摸 1 张牌。 | `l28_batch_b_geyuan.rs:145-169` + `d34_flags.rs:791-812` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111236`) + `l28_thunder_eight_equip_and_interactions` (`seed = 2026111244`) | `已复核等价` |
| 47 | `th_gusuan` | `extension.js:8614-8633` | 股算：觉醒技，任意角色回合结束时，若“圆环之理”长度为 3，减 1 点体力上限并修改〖割圆〗。 | `l28_batch_b_geyuan.rs:256-288` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111236`) + `l28_thunder_eight_equip_and_interactions` (`seed = 2026111244`) | `已复核等价` |
| 48 | `th_yeyan` | `extension.js:8636-8926` | 业炎（连带 `qinyin`）：限定技，出牌阶段可对一名角色造成 1 点火焰伤害，或弃置 4 张不同花色手牌并失去 3 点体力对目标造成 2/3 点火焰伤害（已登记裁决 40 `ChoiceSpec` 抽象）。 | `l28_batch_b.rs:116-143` + `l28_batch_b_asks.rs:266-420` | `l28_thunder_eight_batch_b_skills` (`seed = 2026111237`) | `已复核等价` |

### 3.4 配置项（7 项，7/7 `已复核等价`）

| # | 实体 ID | 上游 `file:line` | 默认值与行为摘要 | 原生落点 | 6 维测试入口 | 状态 |
|---:|---|---|---|---|---|---|
| 49 | `gameSpeed` | `extension.js:9068-9080` | 小游戏速度（默认 `"4"` 常速，`0..=8` 档映射步进延迟毫秒，越界回退 `4`） | `crates/content-native-loader/src/thunder.rs:8-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |
| 50 | `replaceCha` | `extension.js:9081-9084` | 本体同名武将技能替换（默认 `true`） | `crates/content-native-loader/src/thunder.rs:34-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |
| 51 | `hidePack` | `extension.js:9085-9088` | 隐藏小游戏武将专属包（默认 `true`） | `crates/content-native-loader/src/thunder.rs:35-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |
| 52 | `UIItroduc` | `extension.js:9091-9108` | UI补丁使用指南说明项（默认 `"intro"`） | `crates/content-native-loader/src/thunder.rs:42-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |
| 53 | `yuqiTheme` | `extension.js:9109-9119` | UI主题（默认 `"follow"`，支持 `default`/`decade`/`shousha`/`follow`） | `crates/content-native-loader/src/thunder.rs:20-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |
| 54 | `timeTheme` | `extension.js:9120-9129` | 进度条风格（默认 `"follow"`，支持 `decade`/`shousha`/`follow`，`default` 归一为 `follow`） | `crates/content-native-loader/src/thunder.rs:20-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |
| 55 | `standPath` | `extension.js:9134-9149` | 立绘存放目录（默认 `"extension/雷霆万钧/image/stand/"`，支持千幻聆音立绘目录与空串回退） | `crates/content-native-loader/src/thunder.rs:41-246` | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | `已复核等价` |

### 3.5 渲染面（1 项，留待执行队列第 5 项界面阶段）

| # | 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---:|---|---|---|---|---|
| 56 | `thunder.css` | `thunder.css:1-1522` | CSS 样式表 `thunder.css`（1,522 行） | 原生前端组件 | `简化近似`（纯前端 CSS 渲染面，归属执行队列第 5 项界面阶段） |

---

## 4. 6 维真实事件对局验收矩阵

| 维度 | 验证内容 | 覆盖测试函数 | 结果 |
|---|---|---|---|
| D1 正式包装载与名册/卡牌/配置解析 | `thunder-upstream-liuli` 正式包挂载 14 名 `th_*` 武将（含 `th_guanning` `3/7` 初始体力）、`th_taipingyaoshu` ♥3 防具、33 项 `th_*` 技能与 6 项关联基础技、7 项 `ThunderConfig` 默认值/自定义值/非法回退与快照往返 | `l28_thunder_eight_roster_card_and_configs` + `l28_thunder_eight_dimension_acceptance` | 通过 |
| D2 正向触发与多分支主动技能 | 管宁 `th_dunshi` 视为用杀 + 伤害预防 4 选 2；孙寒华 `th_chongxu` 集灵升级 + `miaojian` 1..3 阶刺杀/火攻；蒲元 `th_shengong` 锻造置装 + `olqisi` 摸牌；庞德公 `th_pingcai` 卧龙/凤雏/水镜/玄剑四宝物；郑玄 `th_zhengjing` 置“经”；马日磾 `th_buxu` 弃牌补六经；南华老仙 `th_yufeng` 0 分摸牌与高分标记；马钧 `th_qiaosi` 赢牌弃置/交人 + `th_jingxie` 强化与濒死重铸回血；周群 `th_tiansuan` 五签完整效果；王朗 `th_gushe` + `rejici` 拼点与 7 饶舌阵亡；刘徽 `th_geyuan` + `th_jieshu` + `th_gusuan` 圆环削减与觉醒；神周瑜 `th_yeyan` 1 点/3 点火伤 + `qinyin` 弃牌全场回血/失血 | `l28_thunder_eight_batch_a_active_skills` + `l28_thunder_eight_batch_a_triggers` + `l28_thunder_eight_batch_b_skills` | 通过 |
| D3 负向守卫与非法输入拒绝 | `th_dunshi` 同回合二次发动拒绝与已移除牌名不可再选；`th_zhongzhuang` 攻击范围 `== 3` 不改伤害；`th_taipingyaoshu` 在 `hp == 1` 时失去装备只摸 2 牌不失体力；`th_tiansuan2_3` 下签禁桃、`th_tiansuan2_4` 下下签禁桃且禁酒；`th_jieshu` 非圆环点数手牌在弃牌阶段不可弃置且不计入上限 | `l28_thunder_eight_batch_a_triggers` + `l28_thunder_eight_batch_b_skills` + `l28_thunder_eight_equip_and_interactions` | 通过 |
| D4 多技能联动与跨角色交互 | `th_caiyi` 同回合阴阳双发联动 `th_guili` 额外回合；`th_jingxie` 强化防具赋能 `th_qiaosi` 满额 6 张牌；`th_gushe` 没赢获饶舌联动 `rejici` 拼点点数 `+X`；`th_geyuan` 削减圆环至 3 联动 `th_gusuan` 回合结束觉醒并修改 `th_geyuan` 阈值与牌堆顶选牌 | `l28_thunder_eight_batch_a_triggers` + `l28_thunder_eight_batch_b_skills` + `l28_thunder_eight_equip_and_interactions` | 通过 |
| D5 挂起等待期快照编码/解码与复盘确定性 | 在 `th_dunshi`、`th_chongxu`、`th_shengong`、`th_pingcai`、`th_zhengjing`、`th_yufeng`、`th_qiaosi`、`th_tiansuan`、`th_yeyan` 的 `ChoiceSpec` 挂起窗口执行 `encode_archive` / `decode_archive`，验证恢复后继续提交决策的状态摘要与未中断路径 100% 一致 | `l28_thunder_eight_batch_a_active_skills` + `l28_thunder_eight_batch_b_skills` + `l28_thunder_eight_equip_and_interactions` | 通过 |
| D6 `BoundedPlanner` AI 合法决策闭环 | 全部 `L28` 挂起窗口调用 `BoundedPlanner::new(AiLevel::Expert)` 生成合法 `EngineCommand::SubmitChoice` 并推演至结算完成，零 panic / 零非法命令 | `l28_thunder_eight_batch_a_active_skills` + `l28_thunder_eight_batch_b_skills` + `l28_thunder_eight_equip_and_interactions` | 通过 |

---

## 5. 验证命令与原始输出摘要

在 `/Users/ruyimg/Documents/Claude/sanguosha/noname-next` 下执行：

```text
$ cargo test --offline -p engine-native --test g1 l28_thunder_eight
running 5 tests
test groups::g1::l28_thunder_eight_match::l28_thunder_eight_roster_card_and_configs ... ok
test groups::g1::l28_thunder_eight_triggers::l28_thunder_eight_batch_a_triggers ... ok
test groups::g1::l28_thunder_eight_batch_b::l28_thunder_eight_equip_and_interactions ... ok
test groups::g1::l28_thunder_eight_match::l28_thunder_eight_batch_a_active_skills ... ok
test groups::g1::l28_thunder_eight_batch_b::l28_thunder_eight_batch_b_skills ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.03s

$ cargo test --offline -p engine-native --test l_rounds l28_thunder_eight
running 1 test
test l_rounds::l28_thunder_eight::l28_thunder_eight_dimension_acceptance ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 0.18s

$ cargo test --offline -p engine-native --test misc decision34
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 166 filtered out; finished in 1.55s

$ cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --lib
test result: ok. 15 passed (engine-native) + 4 passed (engine-native-content) + 6 passed (engine-native-kernel); 0 failed

$ python3 -B scripts/legacy-harness/generate-g0-docs.py
wrote 37 main docs + 1 part docs + README -> /Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/migration/g0
Totals: {'generals': 834, 'skills': 1908, 'cards': 377, 'modes': 215, 'configs': 434, 'ui_hooks': 507, 'surfaces': 149, '已复核等价': 230, '简化近似': 4194, '缺失': 0, '上游未定义': 0}

$ python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
Ran 2 tests in 0.013s — OK

$ python3 -B scripts/check-source-lines.py
Source line limit passed: 9297 files <= 2000 lines; Rust non-test files <= 1000 effective lines
```

# 2026-11-12 · Queue Item 2 Source 10：`L19 末日乱斗` 69/69 全量独立复核与 6 维真实对局验收证据

- **核验日期**：2026-11-12
- **独立复核 Agent**：`l19-signoff`（未参与 `L19` 规则实现与测试编写）
- **上游来源**：`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js`（3,411 行，共 **69 项** G0 登记实体：**2 武将 + 1 配置项 + 66 技能**，无独立 CSS/渲染面文件）
- **正式扩展包**：[package.npack.json](../../../../packages/extensions/doomsday-brawl-upstream-liuli/package.npack.json)（`doomsday-brawl-upstream-liuli`）
- **原生落点**：
  - 宿主行为与裁决 40 交互问答：[l19_batch_a.rs](../../../../crates/engine-native-content/src/content/host/step/l19_batch_a.rs)（`HostBehavior::L19DoomsdayBatchA = 559`）、[l19_batch_a_asks.rs](../../../../crates/engine-native-content/src/content/host/step/l19_batch_a_asks.rs)、[l19_batch_b.rs](../../../../crates/engine-native-content/src/content/host/step/l19_batch_b.rs)（`HostBehavior::L19DoomsdayBatchB = 560`）、[l19_batch_b_asks.rs](../../../../crates/engine-native-content/src/content/host/step/l19_batch_b_asks.rs)
  - 独立宿主行为 3 技：[skills_status.rs](../../../../crates/engine-native-content/src/content/host/step/skills_status.rs#L73-L118)（`HostBehavior::Zaibian`、`HostBehavior::Xunmeng`）、[skills_recovery.rs](../../../../crates/engine-native-content/src/content/host/step/skills_recovery.rs#L58-L65)（`HostBehavior::Shishi`）
  - 建局修饰与视为技桥接：[d34_l11_mods.rs](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_l11_mods.rs#L112-L662)（`baozou`、`_wujinmoshiMaxHandCard`/`E`、`_wujinmoshiDamage`/`E`、`_tamodeskill`、`_guanmodeskill`）、[d34_view_as.rs](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_view_as.rs#L510-L523)（`ganran` 手牌装备牌视为【铁索连环】）
  - 模式配置、僵尸目录与身份轮转链：[doomsday.rs](../../../../crates/content-native-loader/src/doomsday.rs#L1-L234)、[decision34_pack9.rs](../../../../crates/content-native-loader/src/decision34_pack9.rs#L514-L563)、[host_skill_bindings.rs](../../../../crates/content-native-loader/src/host_skill_bindings.rs#L221-L321)
- **验收测试**：[l19_doomsday_match.rs](../../../../crates/engine-native/tests/l19_doomsday_match.rs)（6 项真实对局测试）、[l19_doomsday_brawl_eight.rs](../../../../crates/engine-native/tests/l19_doomsday_brawl_eight.rs)（4 项 8 维与模式契约测试）
- **版本门禁**：`ENGINE_PROTOCOL_VERSION = 333`、`RULESET_VERSION = 107`、`STATE_SCHEMA_VERSION = 5`、`EVENT_CATALOG_VERSION = 759`、`AI_DECISION_CONTRACT_VERSION = 8`（零版本膨胀，14 个 golden 哈希零漂移）
- **复核结论**：`L19 末日乱斗` 全部 **69 / 69 项**（`2 将 + 1 配置 + 66 技 = 100.00%`）均已通过 `l19-signoff` 逐行对照上游 JS 与 6 维真实事件对局核验，状态更新为 `已复核等价`；`L19` 成为第 6 个整源 100% `已复核等价` 来源（继 `U01` 95/95、`U06` 11/11、`U07` 36/36、`L06` 3/3、`L22` 8/8 之后）。全仓 37 来源 `已复核等价` 由 `283` 升至 **`352 / 4,424`（`7.96%`）**，`简化近似` 降至 `4,072`。

---

## 1. 固定随机种子抽检（`seed = 2026111210`，抽样 12 / 69 项 = 17.39% ≥ 10%）

使用 `random.Random(2026111210).sample(l19_keys, 10)` 从 `L19` 的 69 个 G0 实体键中固定种子无放回抽取 10 项（覆盖配置项与技能），并合并 `generals` 类别的 2 名僵尸武将（`jiangshifz`、`jiangshinj`），共 **12 项（17.39%）** 进行逐行深审与测试断言核对：

| # | 类别 | 实体 ID | 上游 `file:line` | 上游 JS 语义摘要 | 原生实现 `file:line` | 对应真实测试函数 | 独立复核结论 |
|---:|---|---|---|---|---|---|---|
| 1 | `generals` | `jiangshifz` | `extension.js:331,3383` | 5 血男将（`['male','ye',5,['xunmeng','zaibian','ganran','wansha','paoxiao']]`，331 行注释、3383 行正式定义） | `decision34_pack9.rs:518-534` + `doomsday.rs:28-35` | `l19_doomsday_match_dedicated_three_skills` + `l19_eight_dimension_from_fresh_install` | `已复核等价` |
| 2 | `generals` | `jiangshinj` | `extension.js:332,3384` | 3 血女将（332 行注释作 `male`，3384 行正式定义为 `['female','ye',3,['baozou','wansha','xueji','shishi','ganran']]`） | `decision34_pack9.rs:535-550` + `doomsday.rs:36-43` | `l19_doomsday_match_dedicated_three_skills` + `l19_eight_dimension_from_fresh_install` | `已复核等价` |
| 3 | `configs` | `zuan_brawl` | `extension.js:3363-3381` | 乱斗模式选择配置（默认 `'0'` 关闭，含 `'0'..'10'` 共 11 档模式映射） | `doomsday.rs:46-91` + `assets/config.json:1-16` | `l19_doomsday_brawl_modes_and_zombie_catalog` + `l19_eight_dimension_from_fresh_install` | `已复核等价` |
| 4 | `skills` | `_jiangshiTx` | `extension.js:637-654` | 僵尸化纯台词/音效/滤镜特效钩子（`game.log` / `playAudio` / `style.filter`，零规则态修改） | `l19_batch_a.rs:94-99`（惰性零副作用注册） | `l19_doomsday_match_batch_a_core_skills` + `l19_eight_dimension_from_fresh_install` | `已复核等价`（登记纯表现层惰性说明） |
| 5 | `skills` | `_useskill` | `extension.js:2296-2311` | 乱斗使用卡牌时主视角增加 2 点经验与 1 枚金币 | `l19_batch_b.rs:58,727-751` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 6 | `skills` | `_zhudieskill` | `extension.js:2630-2748` | 守卫剑阁主公或内奸塔阵亡时结算剑阁胜负并奖励获胜方 88 经验与 88 金币 | `l19_batch_b.rs:65,448-485,823-832` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 7 | `skills` | `chanceIdentity7` | `extension.js:915-926` | 兼并模式块注释内第 7 段身份轮转（农民 `nongmin` → 先锋 `xianfeng`） | `l19_batch_a.rs:76-93` + `doomsday.rs:146-205` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` + `l19_doomsday_brawl_modes_and_zombie_catalog` | `已复核等价`（登记块注释惰性说明） |
| 8 | `skills` | `_anshaskill` | `extension.js:3250-3349` | 暗杀模式开局标记刺杀目标与刺客并摸 1 牌，目标阵亡置 `ansha_victory=1` 并奖 50 经验 50 金币 | `l19_batch_b.rs:73,704-725,860-870` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 9 | `skills` | `chanceIdentity2` | `extension.js:855-866` | 兼并模式块注释内第 2 段身份轮转（主公 `zhu` → 忠臣 `zhong`） | `l19_batch_a.rs:76-93` + `doomsday.rs:146-205` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` + `l19_doomsday_brawl_modes_and_zombie_catalog` | `已复核等价`（登记块注释惰性说明） |
| 10 | `skills` | `chanceIdentity12` | `extension.js:975-986` | 兼并模式块注释内第 12 段身份轮转（中坚 `zhongjian` → 主帅 `zhushuai`） | `l19_batch_a.rs:76-93` + `doomsday.rs:146-205` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` + `l19_doomsday_brawl_modes_and_zombie_catalog` | `已复核等价`（登记块注释惰性说明） |
| 11 | `skills` | `wujinmoshiRE2` | `extension.js:1879-1925` | 无尽模式准备阶段消耗 100 金币强化体力上限/手牌上限/摸牌数/回血/伤害 | `l19_batch_a.rs:51,522-558` + `doomsday.rs:94-137` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 12 | `skills` | `_expskill` | `extension.js:2253-2279` | 回合开始/结束检查 `exp >= level * 20`（`<100` 级）循环扣经验升级并奖金币与额外摸牌 | `l19_batch_b.rs:56,229-269,753-776` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |

---

## 2. 已登记语义抽象与上游源码特殊项说明

1. **2 名僵尸武将（`jiangshifz`、`jiangshinj`）的注释行与正式定义行对照**：
   - 上游 `extension.js:331-332` 为单行注释 `//lib.character.jiangshifz=['male','ye',5,...]` 与 `//lib.character.jiangshinj=['male','ye',3,...]`；真正生效的定义位于 `extension.js:3383-3384` `package.character.character`，其中 `jiangshinj` 性别修正为 `female`。原生实现严格按生效定义注册 `jiangshifz`（5 血男将）与 `jiangshinj`（3 血女将）。
2. **16 项块注释 `chanceIdentity*` 技能（`extension.js:843-1032`）**：
   - 上游 `jianbingmoshi.init` 第 `841–1032` 行被 `/* ... */` 块注释包裹（上游改用 `828–834` 行 `gameStart` 统一置 `player.group = 'ye'`），运行时不会挂载或触发。原生在 `HostBehavior::L19DoomsdayBatchA` 中对 `chanceIdentity`、`chanceIdentity0`、`chanceIdentity2`..`chanceIdentity15` 保持惰性不可发动（零战斗副作用），同时在 `doomsday::identity_chain(seats)` 中完整保留 16 步身份轮转纯函数与席位门限表。
3. **4 项纯表现层 / DOM 角标技能（`_jiangshiTx`、`_jiangshiTx2`、`_jiangshiTx3`、`_keepideskill`）**：
   - `_jiangshiTx*`（`637-697` 行）仅输出台词日志、播放音效及设置头像滤镜；`_keepideskill`（`2044-2067` 行）仅刷新 DOM 身份角标文字与颜色（`我方` / `敌军`），均无任何规则态改动。原生将其注册为惰性无副作用技能，已在 `inventory_status.json` 的 `note` 与 `difference` 字段逐项如实注明。
4. **裁决 40 `ChoiceSpec` 与无头模式状态旗标抽象**：
   - `ganran2`（`ganran2:choose_equip`）、`_huzhu`（`_huzhu:discard_tao` + `_huzhu:choose_target`）、`_yuanqiskill`（`_yuanqiskill:choose_mode` + `_yuanqiskill:choose_skill` / `_yuanqiskill:remove_skill`）均通过 `ChoiceSpec`（`SelectCards` / `SelectPlayers` / `PickTag`）暂停等待玩家或 AI 决策；无尽波次、祖安经验/等级/转生与各乱斗模式胜负转移通过座位确定性 `vm_aux.flags` 建模，支持 `dump_snapshot` / `from_snapshot` 完整恢复。

---

## 3. L19 末日乱斗 69 项全量逐项核验表

| # | 类别 | 实体 ID | 上游 `file:line` | 原生落点 | 验收测试 | 状态 |
|---:|---|---|---|---|---|---|
| 1 | `generals` | `jiangshifz` | `extension.js:331,3383` | `decision34_pack9.rs:518-534`, `doomsday.rs:28-35` | `l19_doomsday_match_dedicated_three_skills` | `已复核等价` |
| 2 | `generals` | `jiangshinj` | `extension.js:332,3384` | `decision34_pack9.rs:535-550`, `doomsday.rs:36-43` | `l19_doomsday_match_dedicated_three_skills` | `已复核等价` |
| 3 | `skills` | `baozou` | `extension.js:333-347` | `d34_l11_mods.rs:283-290`, `l19_batch_a.rs:33` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 4 | `skills` | `shishi` | `extension.js:348-355` | `skills_recovery.rs:58-65` (`HostBehavior::Shishi`) | `l19_doomsday_match_dedicated_three_skills` | `已复核等价` |
| 5 | `skills` | `xunmeng` | `extension.js:356-366` | `skills_status.rs:102-118` (`HostBehavior::Xunmeng`) | `l19_doomsday_match_dedicated_three_skills` | `已复核等价` |
| 6 | `skills` | `zaibian` | `extension.js:367-377` | `skills_status.rs:73-100`, `doomsday.rs:208-219` (`HostBehavior::Zaibian`) | `l19_doomsday_match_dedicated_three_skills` | `已复核等价` |
| 7 | `skills` | `ganran` | `extension.js:378-434` | `d34_view_as.rs:510-523`, `l19_batch_a.rs:34,119-128` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 8 | `skills` | `ganran2` | `extension.js:435-481` | `l19_batch_a.rs:34,130-141`, `l19_batch_a_asks.rs:84-215` | `l19_doomsday_match_batch_a_core_skills`, `l19_doomsday_match_save_restore_and_ai` | `已复核等价` |
| 9 | `skills` | `_jisuangailv` | `extension.js:483-496` | `l19_batch_a.rs:41,148-159,494-496` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 10 | `skills` | `_tuizhi` | `extension.js:497-513` | `l19_batch_a.rs:42,160-175,496-498` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 11 | `skills` | `_tuizhi2` | `extension.js:514-531` | `l19_batch_a.rs:43,176-184,499-500` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 12 | `skills` | `_jiangshi` | `extension.js:532-573` | `l19_batch_a.rs:44,185-210,501-502` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 13 | `skills` | `_jiangshi2` | `extension.js:574-591` | `l19_batch_a.rs:45,211-249,409-424` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 14 | `skills` | `_jiangshi3` | `extension.js:592-605` | `l19_batch_a.rs:46,211-249,409-424` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 15 | `skills` | `_jiangshi4` | `extension.js:606-636` | `l19_batch_a.rs:47,250-303,425-445` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 16 | `skills` | `_jiangshiTx` | `extension.js:637-654` | `l19_batch_a.rs:94-99`（纯表现层惰性） | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 17 | `skills` | `_jiangshiTx2` | `extension.js:655-668` | `l19_batch_a.rs:94-99`（纯表现层惰性） | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 18 | `skills` | `_jiangshiTx3` | `extension.js:669-697` | `l19_batch_a.rs:94-99`（纯表现层惰性） | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 19 | `skills` | `_huzhu` | `extension.js:698-738` | `l19_batch_a.rs:35,142-146`, `l19_batch_a_asks.rs:217-363` | `l19_doomsday_match_batch_a_core_skills`, `l19_doomsday_match_save_restore_and_ai` | `已复核等价` |
| 20–35 | `skills` | `chanceIdentity`, `chanceIdentity2`..`15`, `chanceIdentity0`（16 项） | `extension.js:843-1032` | `l19_batch_a.rs:76-93`（块注释惰性） + `doomsday.rs:146-205` | `l19_doomsday_match_batch_a_wujin_and_chance_identity`, `l19_doomsday_brawl_modes_and_zombie_catalog` | `已复核等价` |
| 36 | `skills` | `_gainSkill` | `extension.js:1033-1064` | `l19_batch_a.rs:48,304-354,446-455` | `l19_doomsday_match_batch_a_core_skills` | `已复核等价` |
| 37 | `skills` | `_wujinmoshiRE` | `extension.js:1833-1868` | `l19_batch_a.rs:49,355-391,456-481`, `doomsday.rs:116-143` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 38 | `skills` | `wujinmoshiRE1` | `extension.js:1869-1878` | `l19_batch_a.rs:50,502-521` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 39 | `skills` | `wujinmoshiRE2` | `extension.js:1879-1925` | `l19_batch_a.rs:51,522-558`, `doomsday.rs:94-137` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 40 | `skills` | `_wujinmoshiDamage` | `extension.js:1926-1937` | `d34_l11_mods.rs:649-656`, `l19_batch_a.rs:52,392-399` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 41 | `skills` | `_wujinmoshiDamageE` | `extension.js:1938-1949` | `d34_l11_mods.rs:657-662`, `l19_batch_a.rs:53,400-405` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 42 | `skills` | `_wujinmoshiMaxHandCard` | `extension.js:1950-1957` | `d34_l11_mods.rs:112-119`, `l19_batch_a.rs:33` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 43 | `skills` | `_wujinmoshiMaxHandCardE` | `extension.js:1958-1965` | `d34_l11_mods.rs:120-125`, `l19_batch_a.rs:33` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 44 | `skills` | `_wujinmoshiDraw` | `extension.js:1966-1977` | `l19_batch_a.rs:54,559-568` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 45 | `skills` | `_wujinmoshiDrawE` | `extension.js:1978-1993` | `l19_batch_a.rs:55,569-578` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 46 | `skills` | `_wujinmoshiRecover` | `extension.js:1994-2005` | `l19_batch_a.rs:56,579-586` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 47 | `skills` | `_wujinmoshiRecoverE` | `extension.js:2006-2021` | `l19_batch_a.rs:57,587-595` | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 48 | `skills` | `_keepideskill` | `extension.js:2044-2067` | `l19_batch_a.rs:94-99`（纯 DOM 角标惰性） | `l19_doomsday_match_batch_a_wujin_and_chance_identity` | `已复核等价` |
| 49 | `skills` | `_levelskill` | `extension.js:2068-2252` | `l19_batch_b.rs:55,152-227` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 50 | `skills` | `_expskill` | `extension.js:2253-2279` | `l19_batch_b.rs:56,229-269,753-776` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 51 | `skills` | `_exp2skill` | `extension.js:2280-2295` | `l19_batch_b.rs:57,270-296,872-897` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 52 | `skills` | `_useskill` | `extension.js:2296-2311` | `l19_batch_b.rs:58,727-751` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 53 | `skills` | `_killskill` | `extension.js:2312-2328` | `l19_batch_b.rs:59,298-311,778-791` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 54 | `skills` | `_relifeskill` | `extension.js:2329-2421` | `l19_batch_b.rs:60,247-269` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 55 | `skills` | `_yuanqiskill` | `extension.js:2422-2494` | `l19_batch_b.rs:34,93-150`, `l19_batch_b_asks.rs:76-371` | `l19_doomsday_match_batch_b_progression_and_brawl_modes`, `l19_doomsday_match_save_restore_and_ai` | `已复核等价` |
| 56 | `skills` | `_zhongzhubuff` | `extension.js:2495-2514` | `l19_batch_b.rs:61,313-336` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 57 | `skills` | `_jingchengadddie` | `extension.js:2515-2535` | `l19_batch_b.rs:62,338-360,793-805` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 58 | `skills` | `_zhongfandieskill` | `extension.js:2536-2574` | `l19_batch_b.rs:63,362-415,807-821` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 59 | `skills` | `_tamodeskill` | `extension.js:2575-2629` | `d34_l11_mods.rs:466-473`, `l19_batch_b.rs:64,417-446` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 60 | `skills` | `_zhudieskill` | `extension.js:2630-2748` | `l19_batch_b.rs:65,448-485,823-832` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 61 | `skills` | `_qunxiongzhengbaskill` | `extension.js:2749-2799` | `l19_batch_b.rs:66,487-513` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 62 | `skills` | `_guanmodeskill` | `extension.js:2800-2846` | `d34_l11_mods.rs:474-482`, `l19_batch_b.rs:67,515-531` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 63 | `skills` | `_guandieskill` | `extension.js:2847-2987` | `l19_batch_b.rs:68,533-582,834-846` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 64 | `skills` | `_guixinmodeskill` | `extension.js:2988-3053` | `l19_batch_b.rs:69,584-627,848-858` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 65 | `skills` | `_huanhaiskill` | `extension.js:3054-3121` | `l19_batch_b.rs:70,629-650` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 66 | `skills` | `_seadreammodeskill` | `extension.js:3122-3206` | `l19_batch_b.rs:71,652-682` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 67 | `skills` | `_dietime` | `extension.js:3207-3249` | `l19_batch_b.rs:72,684-702` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 68 | `skills` | `_anshaskill` | `extension.js:3250-3349` | `l19_batch_b.rs:73,704-725,860-870` | `l19_doomsday_match_batch_b_progression_and_brawl_modes` | `已复核等价` |
| 69 | `configs` | `zuan_brawl` | `extension.js:3363-3381` | `doomsday.rs:46-91`, `assets/config.json:1-16` | `l19_doomsday_brawl_modes_and_zombie_catalog`, `l19_eight_dimension_from_fresh_install` | `已复核等价` |

---

## 4. 六维真实对局测试矩阵（`l19_doomsday_match.rs` + `l19_doomsday_brawl_eight.rs`）

| 维度 | 验收要求 | 覆盖测试函数与核心断言 |
|---|---|---|
| 1. 建局组装与包生命周期 | 从干净目录安装 `doomsday-brawl-upstream-liuli`，启停、升级、卸载与锁内容绑定 | `l19_eight_dimension_from_fresh_install`、`l19_doomsday_brawl_modes_and_zombie_catalog`（验证 `jiangshifz` 5 血男将、`jiangshinj` 3 血女将、`zuan_brawl` 11 档模式及 66 技绑定） |
| 2. 正向触发与真实事件链 | 经 `PlayCard` / `ActivateSkill` / `AdvancePhase` / 伤害与阵亡钩子推进，禁直接篡改状态伪造断言 | `l19_doomsday_match_dedicated_three_skills`（`zaibian`/`shishi`/`xunmeng`）、`l19_doomsday_match_batch_a_core_skills`（`baozou`/`ganran`/`ganran2`/`_jisuangailv`/`_tuizhi`/`_jiangshi`..`4`/`_huzhu`/`_gainSkill`）、`l19_doomsday_match_batch_a_wujin_and_chance_identity`、`l19_doomsday_match_batch_b_progression_and_brawl_modes` |
| 3. 边界、拦截与负向路径 | 手牌非唯一最多不可无限出杀、免疫延时锦囊、块注释 `chanceIdentity*` 与纯表现层 `_jiangshiTx*`/`_keepideskill` 零副作用 | `l19_doomsday_match_batch_a_core_skills`（`baozou` 平手不加成）、`l19_doomsday_match_batch_a_wujin_and_chance_identity`（16 项 `chanceIdentity*` 与 `_keepideskill` 惰性断言）、`l19_doomsday_match_batch_b_progression_and_brawl_modes`（`_tamodeskill`/`_guanmodeskill` 延时锦囊拦截） |
| 4. 多组件联动与阶段转移 | 关羽三阶段变身、剑阁复活与塔阵亡、归心同化、幻海迷城关卡推进、生死时速倒计时、暗杀刺杀 | `l19_doomsday_match_batch_b_progression_and_brawl_modes`（覆盖模式 4–10 全部状态机与经验/金币/转生联动） |
| 5. 存档恢复（Snapshot Round-Trip） | 在 `ChoiceSpec` 交互暂停点（`ganran2:choose_equip`、`_huzhu:discard_tao`、`_yuanqiskill:choose_mode`）执行 `dump_snapshot` / `from_snapshot` | `l19_doomsday_match_save_restore_and_ai`（验证快照恢复后 `state_hash` 完全一致，继续提交 `Reply` 后结算结果与不中断路径一致） |
| 6. AI 预算与合法决策 | AI 在 `ganran2`、`_huzhu`、`_yuanqiskill` 等自定义 `ChoiceSpec` 暂停点在预算内产出合法 `Reply` | `l19_doomsday_match_save_restore_and_ai`（`decide_with_ai` 在预算内返回合法选牌/选人/选标签动作并推进对局） |

---

## 5. 验证命令与原始通过结果

```text
$ cargo test --offline -p engine-native --test g1 l19_doomsday
running 6 tests
test l19_doomsday_match::l19_doomsday_match_batch_a_core_skills ... ok
test l19_doomsday_match::l19_doomsday_match_batch_a_wujin_and_chance_identity ... ok
test l19_doomsday_match::l19_doomsday_match_batch_b_progression_and_brawl_modes ... ok
test l19_doomsday_match::l19_doomsday_match_dedicated_three_skills ... ok
test l19_doomsday_match::l19_doomsday_match_save_restore_and_ai ... ok
test l19_doomsday_match::l19_doomsday_match_up37_bridge_delegates_all_66_skills ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 259 filtered out; finished in 0.07s

$ cargo test --offline -p engine-native --test l_rounds l19_
running 9 tests
test l19_doomsday_brawl_eight::l19_doomsday_brawl_modes_and_zombie_catalog ... ok
test l19_doomsday_brawl_eight::l19_doomsday_wujin_and_identity_chain_contracts ... ok
test l19_doomsday_brawl_eight::l19_doomsday_zombie_skills_and_coexistence ... ok
test l19_doomsday_brawl_eight::l19_eight_dimension_from_fresh_install ... ok
test l19_l23_brawl_mark_martial_elo_desktop::l19_doomsday_brawl_zombie_catalog_and_modes ... ok
test l19_l23_brawl_mark_martial_elo_desktop::l19_l23_coexistence_and_disable_isolation ... ok
test l19_l23_brawl_mark_martial_elo_desktop::l19_l23_determinism_and_snapshot_round_trip ... ok
test l19_l23_brawl_mark_martial_elo_desktop::l19_l23_fifteen_dimension_acceptance_matrix ... ok
test l19_l23_brawl_mark_martial_elo_desktop::l19_l23_install_enable_disable_upgrade_uninstall_lifecycle ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.14s

$ cargo test --offline -p engine-native --test misc decision34
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 210 filtered out; finished in 1.54s

$ cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --lib
test result: ok. 15 passed (engine-native) + 4 passed (engine-native-content) + 6 passed (engine-native-kernel); 0 failed
```

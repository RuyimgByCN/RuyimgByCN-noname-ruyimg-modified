# 2026-11-12 · Queue Item 2 Source 11：`L20 标记补充` 92/92 非 CSS 项独立复核与 6 维真实对局验收证据

- **核验日期**：2026-11-12
- **独立复核 Agent**：`l20-signoff / l20-verifier`（未参与 `L20` 规则实现与测试编写；经首轮审查 `#136/#137`、二轮复审 `#150/#151/#152` 及 `#154` 补丁修复后终审通过）
- **上游来源**：`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`（6,519 行，共 **95 项** G0 登记实体：**3 张卡牌 + 15 项配置 + 28 项技能 + 46 项 `ui_hooks` 技能/钩子 + 3 项纯 CSS 表面文件**）
- **正式扩展包**：[package.npack.json](../../../../packages/extensions/mark-supplement-upstream-liuli/package.npack.json)（`mark-supplement-upstream-liuli`）
- **原生落点**：
  - 宿主行为与裁决 40 交互问答：[l20_batch_a.rs](../../../../crates/engine-native-content/src/content/host/step/l20_batch_a.rs)（`HostBehavior::L20MarkBatchA = 561`）、[l20_batch_a_asks.rs](../../../../crates/engine-native-content/src/content/host/step/l20_batch_a_asks.rs)、[l20_batch_b.rs](../../../../crates/engine-native-content/src/content/host/step/l20_batch_b.rs)（`HostBehavior::L20MarkBatchB = 562`）、[l20_batch_b_ext.rs](../../../../crates/engine-native-content/src/content/host/step/l20_batch_b_ext.rs)、[l20_batch_b_mouyi.rs](../../../../crates/engine-native-content/src/content/host/step/l20_batch_b_mouyi.rs)、[l20_batch_b_asks.rs](../../../../crates/engine-native-content/src/content/host/step/l20_batch_b_asks.rs)
  - 共享及独立宿主行为：[l27_batch_a.rs](../../../../crates/engine-native-content/src/content/host/step/l27_batch_a.rs) + [l27_batch_a_asks.rs](../../../../crates/engine-native-content/src/content/host/step/l27_batch_a_asks.rs)（`shouxi`）、[skills_recovery.rs](../../../../crates/engine-native-content/src/content/host/step/skills_recovery.rs)（`kurou`、`_xigua_xueliangfanbei_`、`_xigua_mopaifanbei_`、`_xigua_zhuwanzhongji_`）
  - 建局修饰、花色锁定与视为技桥接：[d34_l11_mods.rs](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_l11_mods.rs)（`_yinyang_mark_add`、`diaohulishan`、`aozhan`、`tspowei`、`fz_liegong`、`fz_gzpaoxiao`、`xinfu_lingren`）、[d34_view_as.rs](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_view_as.rs)（`_zhulianbihe_mark_tao`、`aozhan`、`fz_new_rewusheng`、`fz_new_longdan`）、[catalog_queries.rs](../../../../crates/engine-native-kernel/src/catalog_queries.rs)（`xinhongyan` 黑桃视为红桃 + 铁骑封印非锁定技动态判定）
  - 整肃展示卡与 15 项扩展配置解析：[mark_supplement.rs](../../../../crates/content-native-loader/src/mark_supplement.rs)、[host_skill_bindings.rs](../../../../crates/content-native-loader/src/host_skill_bindings.rs)
- **验收测试**：[l20_mark_supplement_match.rs](../../../../crates/engine-native/tests/l20_mark_supplement_match.rs)（3 项大规模 6 维真实对局测试）、[l20_mark_supplement_eight.rs](../../../../crates/engine-native/tests/l20_mark_supplement_eight.rs)（8 维与生命周期验收）、[l20_liegong_batch3.rs](../../../../crates/engine-native/tests/l20_liegong_batch3.rs)
- **版本门禁**：`ENGINE_PROTOCOL_VERSION = 333`、`RULESET_VERSION = 107`、`STATE_SCHEMA_VERSION = 5`、`EVENT_CATALOG_VERSION = 759`、`AI_DECISION_CONTRACT_VERSION = 8`（零版本膨胀，14 个 golden 哈希零漂移）
- **复核结论**：`L20 标记补充` 全部 **92 / 92 项非 CSS 实体**（`3 卡 + 15 配置 + 28 技 + 46 ui_hooks = 100.00%` 非 CSS 覆盖）均已通过 `l20-signoff / l20-verifier` 逐行对照上游 JS 与 6 维真实事件对局核验，状态更新为 `已复核等价`；其余 **3 项纯 CSS 表面文件**（`extension.css`、`group.css`、`mark.css`）保持 `简化近似`（归属执行队列第 5 项界面阶段）。全仓 37 来源 `已复核等价` 由 `352` 升至 **`444 / 4,424`（`10.04%`）**，`简化近似` 降至 `3,980`。

---

## 1. 固定随机种子抽检（`seed = 2026111211`，抽样 10 / 92 项非 CSS 实体 = 10.87% ≥ 10%）

使用 `random.Random(2026111211).sample(l20_non_css_keys, 10)` 从 `L20` 的 92 个非 CSS G0 实体键中固定种子无放回抽取 **10 项（10.87%）** 进行逐行深审与测试断言核对：

| # | 类别 | 实体 ID | 上游 `file:line` | 上游 JS 语义摘要 | 原生实现 `file:line` | 对应真实测试函数 | 独立复核结论 |
|---:|---|---|---|---|---|---|---|
| 1 | `ui_hooks` | `_player_game_guagua` | `extension.js:2802-2920` | 准备阶段纯结算胜负特效/音效钩子（零规则态修改） | `l20_batch_a.rs:334-345`（被动呈现钩子，无主动出牌入口） | `l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions` | `已复核等价`（登记纯表现层钩子说明） |
| 2 | `configs` | `guozhanmeihua` | `extension.js:6423-6427` | 布尔配置「国战美化」（默认 `true`，控制国战先驱/珠联璧合/阴阳鱼/野心家标记美化） | `mark_supplement.rs:185,205,290` + `assets/config.json` | `l20_configs_15_and_zhengsu_cards_3_and_dedicated_9_skills_six_dimensions` | `已复核等价` |
| 3 | `ui_hooks` | `_o_o_recovertrigger` | `extension.js:2512-2555` | `recoverEnd` 统计本回合回复量并在达阈值时播放妙手回春/医术高超特效（零规则态改动） | `l20_batch_a.rs:219-231,563-595` | `l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions` | `已复核等价` |
| 4 | `skills` | `wengua5` | `extension.js:6145-6154` | 全局 `phaseZhunbeiBegin` 锁定触发，当场上有存活 `wengua` 持有者时向当前回合角色授予当回合临时技 `wengua2`（可交 1 牌置顶或置底并各摸 1 牌） | `l20_batch_a.rs:167-218,521-561` + `l20_batch_a_asks.rs:167-230` | `l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions` | `已复核等价` |
| 5 | `configs` | `daojianfu` | `extension.js:6449-6458` | 选项配置「暴击音效/动画触发阈值」（默认 `"4.5"`） | `mark_supplement.rs:146-163,189,209` | `l20_configs_15_and_zhengsu_cards_3_and_dedicated_9_skills_six_dimensions` | `已复核等价` |
| 6 | `configs` | `jinengxiugai` | `extension.js:6398-6402` | 布尔配置「技能修改和技能描述修改」（默认 `true`） | `mark_supplement.rs:180,200,285` | `l20_configs_15_and_zhengsu_cards_3_and_dedicated_9_skills_six_dimensions` | `已复核等价` |
| 7 | `ui_hooks` | `_wanjunqushou_gua` | `extension.js:2494-2510` | 万军取首击杀特效纯视觉钩子（零规则态改动） | `l20_batch_a.rs:334-345`（被动呈现钩子） | `l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions` | `已复核等价` |
| 8 | `ui_hooks` | `_diankuangtulu_gua` | `extension.js:2475-2491` | 癫狂屠戮单回合累计造成 ≥3 点伤害时触发特效钩子（零规则态改动） | `l20_batch_a.rs:219-231,563-595` | `l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions` | `已复核等价` |
| 9 | `ui_hooks` | `_player_baojiB` | `extension.js:2636-2651` | `damageBegin4` 伤害 = 2 时暴击数字特效钩子 | `l20_batch_a.rs:233-252,597-618` | `l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions` | `已复核等价` |
| 10 | `skills` | `zhengsu` | `extension.js:730-1039` | 整肃核心结算：出牌阶段监听用牌/置装累计擂进（≥3 张点数严格递增）或变阵（≥2 张同花色），弃牌阶段监听弃牌累计鸣止（≥2 张不同花色），弃牌阶段结束达标则由达标方打开 `zhengsu:choose_reward` 二选一摸 2 牌或回 1 血 | `l20_batch_b.rs:309,356-395,507-561,769-793` | `l20_batch_b_20_canonical_skills_six_dimensions` | `已复核等价` |

---

## 2. 首轮/二轮复审发现问题与 `#154` 闭环修复核对记录

1. **Batch A（国战标记、调虎离山、鏖战、问卦与 37 项特效钩子）**：
   - 37 项纯表现层特效钩子及 `zhulianbihe_mark`/`yinyang_mark`/`yexinjia_mark` 移除虚构的出牌阶段主动发动入口，按上游触发条件被动记录表现标记且零规则态污染（注：上游 `_player_baojiF` `extension.js:2707` 写为 `event > 6` 对象比较恒假，原生在 `damage == 6` 记录表现标记且无规则副作用，已登记说明）。
   - `xianqu_mark`（`extension.js:1695-1738`）：移除虚构的「观看手牌」选项，仅允许指定有暗置武将牌的其他角色并选择观看主将或副将（单暗将自动观看）。
   - `diaohulishan`（`extension.js:1432-1488`）：改为纯粹状态技（防止伤害/失去体力/回复体力，不可使用牌、不可被救、不可成为牌目标，移出座次距离计算 `undist`，且移除虚构的弃牌拦截）。
   - `_aozhan_judge` 与 `_zhulianbihe_mark_tao`（`extension.js:1747-1800, 2152-2218`）：修正实体【桃】在鏖战下不可对任何角色（含濒死角色）使用且不可救人，而珠联璧合虚拟【桃】在鏖战下仍可合法使用。
   - `_xigua_xueliangfanbei_`（`extension.js:2261-2272`）：改为全局 `GameStart` 触发，消除首回合前受伤导致翻倍基数偏差。
2. **Batch B（20 项主技能 + 9 项专技/共享技）**：
   - `zuoxing`（`extension.js:6155-6204`）：上游整段处于 `/* ... */` 块注释内，原生已解除 `HostBehavior::L20MarkBatchB` 绑定并保持惰性。
   - `fz_liegong` 与 `fz_xinkuanggu`：`HostBehavior::FzLiegong` 与 `HostBehavior::FzXinKuanggu` 直接桥接至 `l20_batch_b` 完整实现（含 `shouyue` 在场时烈弓攻击范围 +1、狂骨按伤害点数循环二选一回血或摸牌）。
   - `fz_new_rewusheng`、`fz_new_longdan`、`fz_new_tieji`、`sbtieji`：补齐 `shouyue` 在场时武圣任意颜色转【杀】、龙胆仅在经龙胆转化的【杀】/【闪】上触发派生伤害/回血及 `fz_new_longdan_draw` 摸牌、铁骑双将无 `shouyue` 时选封主/副将及动态判定非锁定技失效（含封锁转化技与主动技）、`sbtieji` 遍历全部【杀】目标。
   - `zhenyi_spade` 与 `xinhongyan`：移除非上游 `peony_mark`，对齐花色标签顺序，仅在判定牌亮出后（`judge_pre_finalized`）触发且 `zhenyi_spade` 优先于 `xinhongyan` 入窗，并在 `Catalog::effective_card_suit` 接通 `xinhongyan` 手牌/装备区黑桃视为红桃。
   - `tspowei`、`mibei`、`rechuhai`、`xinfu_lingren`、`rekuanshi`/`kuanshi`/`kuanshi2`、`shouxi`、`sbduanliang`、`shouye`、`mbmowang`：全部按 `#151/#152` 复审意见逐条对齐触发时机、目标合法性、装备置入事件、可取消询问及 `mbdanggu` 前置条件。

---

## 3. L20 标记补充 95 项全量分类汇总表

| 类别 | 数量 | 实体范围 | 状态分布 |
|---|---:|---|---|
| `cards` | 3 | `zhengsu_leijin`、`zhengsu_mingzhi`、`zhengsu_bianzhen` | **3 / 3 `已复核等价`** |
| `configs` | 15 | `jinengxiugai`、`biaojixiugai`、`zhijiechifa`、`doudizhu_guanqiuxiaoguo`、`huajiatexiao`、`guozhanmeihua`、`shoushajishatexiao`、`shiyongkapaitixiao`、`youxishengfu`、`daojianfu`、`daojianfu_type`、`xinshadoudizhu`、`xlfbmoshi`、`mpfbmoshi`、`zhujizhongwang` | **15 / 15 `已复核等价`** |
| `skills` | 28 | `zhengsu`、`zhenyi_spade`、`xinhongyan`、`xianqu_mark`、`zhulianbihe_mark`、`yinyang_mark`、`yexinjia_mark`、`xinfu_langxi`、`tspowei`、`qingyu`、`mibei`、`fz_new_rewusheng`、`fz_gzpaoxiao`、`fz_new_longdan`、`fz_new_tieji`、`fz_liegong`、`fz_xinkuanggu`、`rekuanshi`、`kuanshi`、`shouxi`、`rechuhai`、`xinfu_lingren`、`sbtieji`、`sbduanliang`、`shouye`、`wengua`、`wengua5`、`kurou` | **28 / 28 `已复核等价`** |
| `ui_hooks` | 46 | `_xigua_feiyanbahu_`、`_shiqihujia_`、`_guozhanChooseControlCHG`、`diaohulishan`、`_guozhan_marks`、`_zhulianbihe_mark_tao`、`_yinyang_mark_add`、`_guozhan_diebiaoji_`、`_guozhan_duiweibiaoji_gua_`、`_guozhan_duiweibiaoji_gua_gua_`、`_aozhan_judge`、`_guozhanhalfchghp`、`_xigua_xueliangfanbei_`、`_xigua_mopaifanbei_`、`_xigua_zhuwanzhongji_`、`_shoushajisha_`、`_shoushajisha_delete_`、`_diankuangtulu_gua`、`_wanjunqushou_gua`、`_o_o_recovertrigger`、`_o_o_miaoshouhuichun`、`_o_o_yishugaochao`、`_losehp_`、`_mianshang_`、`_player_baojiA`..`H`（8 项）、`_jiubuff_`、`_jiubuff_xiaoshi_`、`_player_game_guagua`、`_guozhan_quanjubiaoji_guagua`、`_shiyongkapaitexiao_`、`_shiyongkapaitexiao_xi_gua_`、`_shanghaipaishiyongtexiao_`、`_player_daojianfu_xxgg_fuzi_`、`_binglintu`、`_dysp`、`kuanshi2`、`mbmowang`、`zuoxing`、`_chongzhu` | **46 / 46 `已复核等价`** |
| `surfaces` | 3 | `extension.css`（881 行）、`group.css`（17 行）、`mark.css`（61 行） | **3 / 3 `简化近似`**（纯 CSS 视觉表面，归属队列第 5 项界面阶段） |
| **合计** | **95** | [G0 · L20 标记补充](../../../migration/g0/L20-标记补充.md) | **`已复核等价 92` / `简化近似 3` / `缺失 0`** |

---

## 4. 验证命令与原始通过结果

```text
$ cargo test --offline -p engine-native --test g1 l20_mark
running 3 tests
test l20_mark_supplement_match::l20_configs_15_and_zhengsu_cards_3_and_dedicated_9_skills_six_dimensions ... ok
test l20_mark_supplement_match::l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions ... ok
test l20_mark_supplement_match::l20_batch_b_20_canonical_skills_six_dimensions ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 201 filtered out; finished in 0.37s

$ cargo test --offline -p engine-native --test l_rounds l20_
running 11 tests
test l20_config_mods_batch1::absent_no_mods ... ok
test l20_config_mods_batch1::mopai_doubles_draw ... ok
test l20_config_mods_batch1::xueliang_doubles_hp ... ok
test l20_kurou_batch2::kurou_absent_rejected ... ok
test l20_kurou_batch2::kurou_loses_hp_and_draws ... ok
test l20_kurou_batch2::zhuwanzhongji_succession ... ok
test l20_liegong_batch3::liegong_direct_hit_when_hand_ge_hp ... ok
test l20_liegong_batch3::xinkuanggu_draws_or_recovers ... ok
test l20_mark_supplement_eight::l20_mark_supplement_batch_a_aliases_configs_cards_guozhan_and_ui_hooks_save_restore ... ok
test l20_mark_supplement_eight::l20_mark_supplement_batch_b_twenty_canonical_skills_and_save_restore ... ok
test l20_mark_supplement_eight::l20_mark_supplement_eight_dimension_acceptance ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 70 filtered out; finished in 0.16s

$ cargo test --offline -p engine-native --test misc decision34
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 415 filtered out; finished in 1.56s

$ python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py
Ran 2 tests in 0.010s — OK
Source line limit passed: 9319 files <= 2000 lines; Rust non-test files <= 1000 effective lines
```

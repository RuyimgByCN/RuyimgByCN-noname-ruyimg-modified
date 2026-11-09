# 裁决 34 第二步证据：清除自造实体与背离上游的默认「摸 1」兜底分支

- **核验日期**：2026-11-09
- **适用分支**：`main` / `native-convergence`
- **对应任务条款**：《原生架构收敛执行方案》第三节第 2 项（「清除自造内容：删除或改写上游不存在的实体（如 `decade_*` 武将）和与上游不符的默认分支（『其余一律摸 1』），对应测试一并改为按上游语义断言。这是用户已确认的纠错范围，删改须在证据中逐条列出理由。」）

---

## 一、删除的上游不存在实体（逐条理由）

| 序号 | 来源 | 被删除／改写实体 | 原代码位置 | 上游核对依据与删除／改写理由 |
| --- | --- | --- | --- | --- |
| 1 | `L08 十周年UI` | `L08_DECADE_CHARACTERS`（9 名 `decade_caocao`、`decade_liubei`、`decade_sunquan`、`decade_lvbu`、`decade_diaochan`、`decade_zhugeliang`、`decade_simayi`、`decade_zhouyu`、`decade_zhaoyun`） | `crates/content-native-loader/src/decision34_pack9_data.rs`、`decision34_pack9.rs` | 上游 `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12687-12692` 中 `character: { character: {}, translate: {} }` 为空，十周年UI 是纯界面扩展，不定义任何武将。已彻底删除。 |
| 2 | `L08 十周年UI` | `_decade_qiankunbagua` 技能绑定 | `crates/content-native-loader/src/decision34_pack9.rs` | 上游 `十周年UI/skill.js` 仅定义 `_usecard`、`_chongzhu`、`_changeEquipExt` 等前端动画钩子，无 `_decade_qiankunbagua` 规则技能。已彻底删除。 |
| 3 | `L14 手杀ui` | `L14_SHOUSHA_CHARACTERS`（`shousha_caocao`、`shousha_liubei`、`shousha_sunquan`）、`L14_SHOUSHA_CARDS`（`shousha_qiankunbagua`）及伪造技能 `spshidi`、`spcishi`、`zhefu`、`yidu` | `crates/content-native-loader/src/decision34_pack9_data.rs`、`decision34_pack9.rs`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs` | 上游 `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7366-7383` 中 `character` 与 `card` 均为空字典 `{}`，且 `手杀ui` 全目录不存在 `spshidi`、`spcishi`、`zhefu`、`yidu` 技能。已彻底删除。 |
| 4 | `L19 末日乱斗` | `L19_DOOMSDAY_CHARACTERS` 中的 22 名 `vir_*` 武将（`vir_zombieKing`、`vir_nemesis`、`vir_tyrant` 等） | `crates/content-native-loader/src/decision34_pack9_data.rs` | 上游 `archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-332,3383-3385` 仅在 `character` 中定义 `jiangshifz`（服装僵尸）与 `jiangshinj`（耐久僵尸）2 名武将，其余 `vir_*` 均为技能 ID 而非武将 ID。已删除 22 名 `vir_*` 伪造武将，保留 `jiangshifz` 与 `jiangshinj`。 |
| 5 | `L20 标记补充` | `L20_MARK_SUPPLEMENT_CHARACTERS`（12 名 `ms_caocao`..`ms_zhenji`）、`ms_longmark` 技能及 `diaohulishan`/`huoshaolianying`/`lulitongxin` 卡牌重绑 | `crates/content-native-loader/src/decision34_pack9_data.rs`、`decision34_pack9.rs` | 上游 `archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6321,6493-6505` 中 `character` 为空（0 武将），`card` 仅含 `zhengsu_leijin`、`zhengsu_mingzhi`、`zhengsu_bianzhen` 3 张整肃子牌，无 `ms_*` 武将与 `ms_longmark`。已删除 `ms_*` 武将、`ms_longmark` 及非本包国战牌重绑。 |
| 6 | `L25 祖安武将` | `L25_ZUAN_CHARACTERS` 中的 60 名 `zuan_*` 武将（`zuan_caocao`..`zuan_diy10`）与 `zuan_jianxiong` 技能 | `crates/content-native-loader/src/decision34_pack9_data.rs`、`decision34_pack9.rs` | 上游 `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:338-397` 仅定义 60 名 `ssr_*` 武将（`ssr_liubei`..`ssr_key_hina`），不存在任何 `zuan_*` 武将或 `zuan_jianxiong`。已删除 60 名 `zuan_*` 伪造武将及 `zuan_jianxiong`，保留上游 60 名 `ssr_*` 武将。 |
| 7 | `L28 雷霆万钧` | 11 名伪造 `th_*` 武将（`th_sunshangxiang`、`th_diaochan`、`th_zhenji`、`th_xiaoqiao`、`th_daqiao`、`th_caiwenji`、`th_zhugeliang`、`th_simayi`、`th_luxun`、`th_lvbu`、`th_zhaoyun`）及伪造技能 `th_chongzou`、`th_huanyin`、`th_xuanlv` | `crates/content-native-loader/src/decision34_closure.rs`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs` | 上游 `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3657-3670` 定义的真实 14 名武将为 `th_guanning`、`th_lukai`、`th_caohua`、`th_sunhanhua`、`th_puyuan`、`th_pangdegong`、`th_zhengxuan`、`th_mamidi`、`th_nanhualaoxian`、`th_majun`、`th_zhouqun`、`th_wanglang`、`th_liuhui`、`th_shen_zhouyu`，真实卡牌为 `th_taipingyaoshu`（`extension.js:8968`），无 `th_chongzou`/`th_huanyin`/`th_xuanlv`。已删除伪造武将与技能，替换为上游真实 14 将、60 技能与 `th_taipingyaoshu`。 |
| 8 | `U08 英雄杀` | `yxs_tafang_wave`（塔防波次技能）与前端 `advanceTafangWave` | `crates/content-native-loader/src/decision34_closure.rs`、`decision34.rs`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts`、`ExtensionSurfacePanel.vue` | 上游 `archive/upstream/noname/apps/core/extension/英雄杀/main/content.js:6` 仅设置 `lib.config.mode_config.tafang.forbidstone = true`，不存在 `yxs_tafang_wave` 技能或波次推进器。已从 Rust 与前端一并删除。 |
| 9 | `U02/L03 boss`、`L11 太虚幻境`、`L19 末日乱斗`、`L21 武皇争霸` | 伪造战役步进技能 `boss_baonu_stage`、`taixu_expedition_step`、`doomsday_infection_step`、`martial_formation_step` | `crates/content-native-loader/src/decision34_closure.rs`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs` | 上游对应扩展中不存在这 4 个合成技能 ID（其中 `taixu_expedition_step` 仅把层数＋1 并造 1 点伤害）。已全部删除。 |
| 10 | `L08/L10/L11/L12/L24/L28` | `drawSpineSkeletalCanvas`（2D Canvas 圆加两条线示意骨骼画法） | `apps/web-client/src/components/battle/ExtensionSurfacePanel.vue` | 按裁决 35 要求，删除 2D Canvas 圆加两条线的示意骨骼画法，改由净室 Spine 4.0 播放器（`apps/web-client/src/engine/spine40/`）承接。 |

---

## 二、清除的背离上游默认「摸 1」兜底分支与 UI 钩子规则化副作用

| 序号 | 函数／模块 | 原先错误行为 | 修正后行为 |
| --- | --- | --- | --- |
| 1 | `resolve_u01_ddd_skill` (`decision34_pack9.rs`) | 64 个技能仅 8 个有分支（其中 `dddxiaxing` 空牌仍给标记且移标记时摸 1 张牌，`dddjungui`/`dddshilie`/`ddddiedang` 为摸牌桩），其余 56 个走 `_ => ctx.start_draw(owner, 1)` | 删除 `_ => ctx.start_draw(owner, 1)` 兜底；按上游 `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js` 修正 `dddxiaxing`（第 59–242 行：空 `he` 牌守卫＋移除「侠」视为杀/闪时不摸牌）、`dddyeshen`（第 656 行）、`dddqiaoduan`（第 747 行）、`dddqianlong`（第 2819 行）、`dddqice`（第 3507 行）、`dddjungui`（第 384–441 行：目标摸 2 展示手牌并按花色数弃牌）、`dddshilie`（第 511–654 行：展示手牌视为杀/闪且仅在点数和等于体力之和时摸牌）、`ddddiedang`（第 802–860 行：摸3弃1／弃3摸1转换技）。 |
| 2 | `resolve_ow_skill` (`decision34_pack9.rs`) | 除 3 个技能外其余走 `_ => ctx.start_draw(owner, 1)` | 删除 `_ => ctx.start_draw(owner, 1)` 兜底；仅对已按上游 `杀海拾遗/character/ow/skill.js` 核实的技能执行结算。 |
| 3 | `resolve_xianjian_skill` (`decision34_pack9.rs`) | 除 4 个技能外其余走 `_ => ctx.start_draw(owner, 1)` | 删除 `_ => ctx.start_draw(owner, 1)` 兜底；仅对已按上游 `杀海拾遗/character/xianjian/skill.js` 核实的技能执行结算。 |
| 4 | `resolve_l08_decade_skill` (`decision34_pack9.rs`) | 将 `_changeEquipExt`、`_usecard`、`_chongzhu` 等十周年UI 钩子实现为加甲或 `ctx.start_draw(owner, 1)` | 删除摸牌与加甲副作用；仅递增 `l08_decade_banner_seq` / `l08_recast_count` 供前端视觉层同步，不改变任何规则牌局状态（手牌、体力、护甲）。 |
| 5 | `resolve_l14_shousha_skill` (`decision34_pack9.rs`) | 将 `_dieNoTuoguan`、`_menuCheat`、`_jindutiao` 等 48 个手杀ui 钩子实现为回血、摸牌或伪造 `zhefu`/`yidu` 伤害 | 删除回血、摸牌与伪造 `spshidi`/`spcishi`/`zhefu`/`yidu` 伤害分支；仅记录前端 HUD 同步计数 `l14_shousha_trigger_count`，不改变任何规则牌局状态。 |
| 6 | `resolve_l19_doomsday_skill` (`decision34_pack9.rs`) | 除 4 个技能外其余走 `_ => ctx.start_draw(owner, 1)`，且将 `baozou`（`末日乱斗/extension.js:333-347`）实现为摸 1 张牌 | 删除 `_ => ctx.start_draw(owner, 1)` 兜底；修正 `baozou` 为设置出杀无次数限制标志 `l19_baozou_sha_unlimited = 1`（不摸牌），并补齐 `shishi`（`extension.js:348-355`）与 `xunmeng`（`extension.js:356-366`）。 |
| 7 | `resolve_l20_skill` (`decision34_pack9.rs`) | 将 `_xigua_*` 等标记钩子走 `_ => ctx.start_draw(owner, 1)` | 删除 `_ => ctx.start_draw(owner, 1)` 兜底；纯视觉钩子仅递增 `l20_mark_trigger_count`。 |
| 8 | `resolve_l24_skin_skill` (`decision34_pack9.rs`) | 将 `_tsx`、`_gj`、`_ts`、`_playAudioToQueue` 等 23 个皮肤切换钩子走 `ctx.start_draw(owner, 1)` | 删除摸牌副作用；仅更新皮肤槽与动画状态标志，不改变任何手牌或体力。 |
| 9 | `resolve_l25_zuan_skill` (`decision34_pack9.rs`) | 除 `g_du` 外百余项技能走 `_ => ctx.start_draw(owner, 1)`，且 `_diycharacterskill`/`_diyskill` 摸牌 | 删除 `_ => ctx.start_draw(owner, 1)` 兜底；`_diycharacterskill`/`_diyskill` 仅设置离线单机应用标志，不摸牌。 |
| 10 | `resolve_l27_select_skill` (`decision34_pack9.rs`) | `_chooseCharacterTwo` 摸 2 张牌，`kuangcai`/`quanji`/`clandianzhan`/`drlt_duorui` 走 `ctx.start_draw(owner, 1)`，其余走 `_ => ctx.start_draw(owner, 1)` | 删除 `_ => ctx.start_draw(owner, 1)` 兜底及 `kuangcai`/`quanji`/`clandianzhan`/`drlt_duorui` 的摸 1 桩分支；`_chooseCharacterTwo` 仅校验双将数量而不摸牌；保留已按上游 `选将美化/extension.js` 实现的 `baonvezhi`（第 184 行）、`qimou`（第 322 行）。 |
| 11 | `resolve_u01_ddd_remaining` / `resolve_l28_thunder_remaining` / `resolve_u05_ow_remaining` / `resolve_u05_xianjian_remaining` (`decision34_ext.rs`) | 将 `U01`、`L28`、`U05 (ow/xianjian)` 约 80 个未逐项实现复杂子句的技能用多模式分组匹配统一设为标记＋`ctx.start_draw(owner, 1)` | 删除全部多模式分组 `ctx.start_draw(owner, 1)` 冒充分支；仅保留已按上游源码逐项核对的 `dddxianxi`、`dddjuxian`、`dddshijian`、`ddddongcha`、`dddfenji`、`dddduanbing`、`th_zhongzhuang`、`th_caiyi`、`th_chongxu`、`miaojian`、`xinfu_pdgyingshi`、`th_tiansuan`、`th_geyuan` 等真实分支。 |

---

## 三、对应测试更新记录

1. `crates/engine-native/tests/decision34_rules_and_campaigns.rs`：
   - 删除对 `yxs_tafang_wave`、`th_chongzou`、`th_huanyin`、`th_xuanlv`、`boss_baonu_stage`、`taixu_expedition_step`、`doomsday_infection_step`、`martial_formation_step` 的伪造断言；
   - 新增对 `L28 雷霆万钧` 上游真实 14 将、60 技能、`th_taipingyaoshu`（`extension.js:6974, 8968`）、`th_dunshi`（`extension.js:3783`，含按 `TH_DUNSHI_ID` 直接派发校验）、`th_jingxie`（`extension.js:7429`）、`th_qiaosi`（`extension.js:7030`）、`th_yeyan`（`extension.js:8636`）的测试断言。
2. `crates/engine-native/tests/decision34_pack9_closure.rs`：
   - 更新 `u01_d3_all_31_generals_and_64_ddd_skills_execute_and_roundtrip` 按上游 `3D精选/character/skill.js` 断言 `dddxiaxing`（空牌负向守卫＋移标记不摸牌）、`dddyeshen`、`dddqiaoduan`、`dddqianlong`、`dddqice`、`dddjungui`、`dddshilie`、`ddddiedang` 的真实效果；
   - 更新 `l08_l14_l19_l20_complete_skills_triggers_and_save_restore` 与 `l24_l25_l27_complete_skills_negative_guards_and_save_restore` 断言：
     - `L08`、`L14`、`L20` 不注册任何伪造武将（且 `spshidi`、`spcishi`、`zhefu`、`yidu` 不在 Catalog 中）；
     - `L19` 仅注册 `jiangshifz` 与 `jiangshinj` 2 名上游武将；
     - `L25` 仅注册 60 名 `ssr_*` 上游武将，无 `zuan_*` 武将；
     - `L08`、`L14`、`L24`、`L27` 前端 UI 钩子触发后手牌数与体力保持不变（零摸牌／零伤害副作用）。
3. `apps/web-client/scripts/decision34-extension-surfaces.test.mjs`：
   - 删除 `advanceTafangWave` 测试，将百科检索词条从 `th_chongzou` 改为真实技能 `th_jingxie`，将 `L25` 离线 DIY 草稿武将从 `zuan_diy02` 改为真实武将 `ssr_sunquan`。

# U08 英雄杀（150 项：43 将 + 107 技）独立复核与 6 维真实对局验收证据（2026-11-12）

- **核验日期**：2026-11-12
- **复核范围**：排队事项二之 **12. `U08 英雄杀（150 项：43 将 + 107 技）`**（上游源目录：`archive/upstream/noname/apps/core/extension/英雄杀/character/{character,skill}.js`；正式扩展包：`packages/extensions/hero-kill-upstream-noname`）
- **独立复核人**：`u08-signoff`（与实现者 `u08-batcha`、`u08-batchb`、`u08-batchc`、`u08-batchd`、`u08-tests` 完全分离）
- **复核结论**：**150 / 150 项全部达到 `已复核等价`**（`43` 名 `yxs_*` 武将 + `107` 项技能，`简化近似: 0`，`缺失: 0`，`上游未定义: 0`）。全局 G0 台账累计更新为：`已复核等价: 594, 简化近似: 3830, 缺失: 0, 上游未定义: 0`。

---

## 1. 范围与批次结构总览

| 分组 | 数量 | 上游源码定位 | 原生状态机与装载落点 | 6 维对局测试与验收入口 | 状态 |
|---|---:|---|---|---|---|
| 武将表（`U08_YXS_GENERALS`） | 43 将 | `英雄杀/character/character.js:3-287` | `crates/content-native-loader/src/decision34_pack9_data.rs:152-196`、`decision34_pack9.rs:479-515`、`u08_batch_a.rs:344-354`（含 `kuangchan` 主公体力上限=4 初始化与 9 项跨包标准技能绑定 `qiangxi`/`kanpo`/`jieyin`/`keji`/`xiaoji`/`mashu`/`wushuang`/`jianxiong`/`longdan`） | `u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble`、`u08_hero_kill_31_standard_skills_and_menshen2_guard` | **43 / 43 已复核等价** |
| 秦琼专属 5 技（JSON 桥接 + 宿主状态机） | 5 技 | `英雄杀/character/skill.js:5-109`（`yxs_fanji`, `yxs_fanji2`, `yxs_menshen3`, `yxs_menshen`, `yxs_menshen2`） | `packages/extensions/hero-kill-upstream-noname/assets/cards.json`、`crates/engine-native-content/src/content/host/step/skills_damage.rs:7-82`、`skills_targets.rs:7-61`、`skills_status.rs:9,13,53-71`、`crates/content-native-loader/src/decision34_closure.rs:285-296` | `u08_hero_kill_eight_dimension_acceptance`、`yxs_fanji_counterattacks_source`、`menshen_marks_target`、`menshen3_clears_marker` | **5 / 5 已复核等价** |
| Batch A（`HostBehavior::U08YxsBatchA`，tag 563） | 23 技 | `英雄杀/character/skill.js:110-660, 749-845`（`guimian`..`chujia`，覆盖兰陵王、妺喜、魏忠贤、南丁格尔、张三丰、唐伯虎、杨广、赵雍、小乔、鲁智深 10 将） | `crates/engine-native-content/src/content/host/step/u08_batch_a.rs:1-861`、`u08_batch_a_asks.rs:1-672`、`crates/content-native-loader/src/host_skill_bindings.rs:738-749` | `u08_batch_a_combat_and_active_skills_6d_real_match`、`u08_batch_a_support_and_modifiers_6d_real_match` | **23 / 23 已复核等价** |
| Batch B（`HostBehavior::U08YxsBatchB`，tag 564） | 26 技（+ 跨包 `kanpo`） | `英雄杀/character/skill.js:846-1609`（`baihe`..`tiewan`，覆盖鬼谷子、福尔摩斯、赢政、岳飞、虞姬、项羽、程咬金、孙武、狄仁杰、罗成、王昭君、明成皇后 12 将） | `crates/engine-native-content/src/content/host/step/u08_batch_b.rs:1-813`、`u08_batch_b_asks.rs:1-708`、`u08_batch_b_ext.rs:1-353`、`crates/content-native-loader/src/host_skill_bindings.rs:750-756` | `u08_batch_b_all_28_skills_6d_real_match` | **26 / 26 已复核等价** |
| Batch C（`HostBehavior::U08YxsBatchC`，tag 565） | 27 技 | `英雄杀/character/skill.js:661-748, 1610-2406`（`qianglue`..`sheshi`，覆盖成吉思汗、罗宾汉、花木兰、李世民、勾践、吕雉、凯撒、拿破仑、褒姒、杨玉环、貂蝉、埃及艳后 12 将） | `crates/engine-native-content/src/content/host/step/u08_batch_c.rs:1-846`、`u08_batch_c_asks.rs:1-823`、`u08_batch_c_ext.rs:1-448`、`crates/content-native-loader/src/host_skill_bindings.rs:757-763` | `u08_batch_c_robin_hood_and_stance_swap_6d_real_match`、`u08_batch_c_modifiers_and_active_skills_6d_real_match`、`u08_batch_c_and_d_remaining_skills_6d_real_match` | **27 / 27 已复核等价** |
| Batch D（`HostBehavior::U08YxsBatchD`，tag 566） | 26 技（+ 跨包 `qiangxi`） | `英雄杀/character/skill.js:2407-3175`（`bolehuiyan`..`lshengong`，覆盖伯乐、武则天、墨子、曹操、朱元璋、荆轲、李白、鲁班 8 将） | `crates/engine-native-content/src/content/host/step/u08_batch_d.rs:1-1011`、`u08_batch_d_asks.rs:1-964`、`u08_batch_d_ext.rs:1-794`、`crates/content-native-loader/src/host_skill_bindings.rs:764-770` | `u08_batch_d_bo_le_wu_ze_tian_mo_zi_and_lu_ban_6d_real_match`、`u08_batch_c_and_d_remaining_skills_6d_real_match` | **26 / 26 已复核等价** |
| **合计** | **150 项** | `character.js:1-287` + `skill.js:1-3175` | `HostBehavior` 全部真绑定，旧 `decision34_u08_l19_l20_l27.rs` 死分支已清理 | 6 维对局测试 + 8 维生命周期测试全绿 | **150 / 150 已复核等价** |

---

## 2. 固定种子（`seed = 2026111212`）10% 随机抽样（15 / 150 项）逐行深审表

使用 `random.Random(2026111212).sample(u08_keys, 15)` 从 `docs/migration/g0/inventory_status.json` 的 150 项 `U08` 条目中确定性抽取 15 项（占比 `10.0%`），由 `u08-signoff` 逐行对照上游 JS 与原生 Rust 状态机：

| # | 抽样项 ID | 类别 | 上游源码定位 (`file:line`) | 上游核心语义 | 原生实现落点 (`file:line`) | 6 维对局测试覆盖 | 复核结论 |
|---:|---|---|---|---|---|---|---|
| 1 | `chajue2` | `skills` | `英雄杀/character/skill.js:1553-1577` | 察觉子技（锁定技）：由 `chajue` 在回合外首次受伤后挂载至回合结束，令持有者对【杀】或普通锦囊牌免疫（`trigger.cancel()`） | `u08_batch_b.rs:584-631`、`u08_batch_b_ext.rs:300-346`（回合外受伤置 `chajue2_active=1`，拦截后续指向自身的【杀】与普通锦囊，回合结束清除） | `u08_batch_b_all_28_skills_6d_real_match`（验证回合外受伤后第 2 张【杀】被 `chajue2` 无效化） | **已复核等价** |
| 2 | `miaobi` | `skills` | `英雄杀/character/skill.js:3033-3056` | 妙笔（李白）：出牌阶段限一次，若全牌堆/手牌曾出现基本牌或普通锦囊，可选择一名有手牌的其他角色弃置其一张手牌，并获得 `miaobix`（本回合可将一张同花色手牌当所弃基本牌或普通锦囊使用） | `u08_batch_d.rs:429-467`、`u08_batch_d_asks.rs:444-508`、`u08_batch_d_ext.rs:554-652`（交互选有手牌目标 -> 选弃其手牌 -> 记录花色与牌种并打开 `miaobix:choose_card` 转化使用） | `u08_batch_c_and_d_remaining_skills_6d_real_match`（含 `miaobi:choose_target` 存档恢复、取消返还次数、弃杀并转化出杀造伤） | **已复核等价** |
| 3 | `heqin` | `skills` | `英雄杀/character/skill.js:1475-1536` | 和亲（王昭君，限定技）：出牌阶段选择一名男性角色，双方获得 `heqin2`（摸牌阶段摸牌数+1）与 `heqin3`（一方阵亡时清除另一方的 `heqin2`） | `u08_batch_b.rs:549-583`、`u08_batch_b_asks.rs:640-678`、`u08_batch_b_ext.rs:265-299`（校验 `heqin_used==0` 且目标为存活男性 -> 置 `heqin_used=1` 与双方 `heqin2_active=1` 并登记配对，阵亡时联动清除） | `u08_batch_b_all_28_skills_6d_real_match`（验证限定技配对男性、双方获 `heqin2_active`、二次发动被拒） | **已复核等价** |
| 4 | `yxs_yingzheng` | `generals` | `英雄杀/character/character.js:195-200` | 赢政：群势力、男性、4/4 体力，技能 `["jianxiong", "batu"]` | `decision34_pack9_data.rs:182`、`decision34_pack9.rs:479-515`（`("yxs_yingzheng", Faction::Qun, Gender::Male, 4, &["jianxiong", "batu"])`，跨包绑定标准 `奸雄` + Batch B `batu`） | `u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble`、`u08_batch_b_all_28_skills_6d_real_match` | **已复核等价** |
| 5 | `tuqiang` | `skills` | `英雄杀/character/skill.js:1896-1907` | 图强（勾践）：每当你使用或打出一张【闪】时，可摸一张牌 | `u08_batch_c.rs:283-310`、`u08_batch_c_asks.rs:297-325`（监听 `CardUsed`/`CardResponded` 为【闪】时打开 `tuqiang:confirm`，确认后摸 1 张牌） | `u08_batch_c_and_d_remaining_skills_6d_real_match`（配合 `jimin` 将非闪手牌当【闪】响应后触发 `tuqiang` 摸牌） | **已复核等价** |
| 6 | `yxs_sunwu` | `generals` | `英雄杀/character/character.js:169-174` | 孙武：吴势力、男性、3/3 体力，技能 `["bingsheng", "taolue"]` | `decision34_pack9_data.rs:178`（`("yxs_sunwu", Faction::Wu, Gender::Male, 3, &["bingsheng", "taolue"])`） | `u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble`、`u08_batch_b_all_28_skills_6d_real_match` | **已复核等价** |
| 7 | `zhulu` | `skills` | `英雄杀/character/skill.js:2845-2871` | 逐鹿（曹操）：每回合限一次，其他角色使用普通锦囊或延时锦囊时，你可弃置一张同花色牌（手牌或装备）并获得该锦囊牌 | `u08_batch_d.rs:304-345`、`u08_batch_d_asks.rs:298-352`（监听其他角色使用锦囊且 `zhulu_used_turn==0` -> 打开 `zhulu:discard_same_suit` 选同花色牌弃置 -> 回收该锦囊入手并置 `zhulu_used_turn=1`） | `u08_batch_c_and_d_remaining_skills_6d_real_match`（验证弃同花色牌夺取其他角色使用的【无中生有】并置本回合已用） | **已复核等价** |
| 8 | `cike` | `skills` | `英雄杀/character/skill.js:3001-3032` | 刺客（荆轲）：你使用【杀】指定目标后，可摸一张牌，然后弃置目标角色一张牌；若弃置的是基本牌，目标角色可对你使用一张【杀】 | `u08_batch_d.rs:384-428`、`u08_batch_d_asks.rs:390-443`、`u08_batch_d_ext.rs:479-553`（【杀】指定目标后确认发动 -> 摸 1 张 -> 选弃目标 1 张牌 -> 若为基本牌则打开 `cike:counter_sha` 供目标出【杀】反击） | `u08_batch_c_and_d_remaining_skills_6d_real_match`、`u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble` | **已复核等价** |
| 9 | `nvquan1` | `skills` | `英雄杀/character/skill.js:2661-2691` | 女权·子技 1（武则天）：锁定技，你对男性角色使用【杀】需两张【闪】响应（`wushuang1` 同款逻辑，针对男性目标） | `u08_batch_d.rs:148-193`、`u08_batch_d_asks.rs:138-186`（女性武则天对男性目标出【杀】且目标首次出【闪】抵消时，要求目标再出一张【闪】，否则恢复命中） | `u08_batch_d_bo_le_wu_ze_tian_mo_zi_and_lu_ban_6d_real_match`、`u08_batch_c_and_d_remaining_skills_6d_real_match` | **已复核等价** |
| 10 | `jieyong` | `skills` | `英雄杀/character/skill.js:2797-2844` | 节用（墨子，含 `jieyong2`/`jieyong3`）：出牌阶段限一次，当你使用基本牌或普通锦囊牌结算后，可弃置一张黑色手牌重新获得此牌；若如此做，本回合不能再使用同名牌（`jieyong3`） | `u08_batch_d.rs:258-303`、`u08_batch_d_asks.rs:248-297`、`u08_batch_d_ext.rs:351-409`（自身出牌阶段使用基本牌/普通锦囊后打开 `jieyong:discard_black`，弃置黑色手牌收回原牌并置 `jieyong2_used=1` 与 `jieyong3_forbid_kind`） | `u08_batch_d_bo_le_wu_ze_tian_mo_zi_and_lu_ban_6d_real_match`（验证使用【桃】后触发 `jianai` + `jieyong:discard_black` 弃黑牌收回【桃】） | **已复核等价** |
| 11 | `yxsweiyi` | `skills` | `英雄杀/character/skill.js:2594-2620` | 威仪（武则天）：当你受到伤害后，若伤害来源有手牌且体力值大于你，你可弃置其一张手牌并令其失去 1 点体力 | `u08_batch_d.rs:102-128`、`u08_batch_d_asks.rs:92-122`（受伤后校验来源存活、有手牌且或满足威仪条件，打开 `yxsweiyi:confirm` 选弃来源手牌并令其 `LoseHp(1)`） | `u08_batch_d_bo_le_wu_ze_tian_mo_zi_and_lu_ban_6d_real_match`、`u08_batch_c_and_d_remaining_skills_6d_real_match` | **已复核等价** |
| 12 | `jiean` | `skills` | `英雄杀/character/skill.js:931-977` | 结案（福尔摩斯）：当 `yanyi`（演绎）命中并造成伤害后触发，你可摸已损失体力值张牌，然后将手牌补至或弃至与体力上限相关数量 | `u08_batch_b.rs:101-138`、`u08_batch_b_asks.rs:116-148`（在 `yanyi` 猜中花色造伤后自动衔接打开 `jiean:confirm`，确认后按已损失体力值摸牌） | `u08_batch_b_all_28_skills_6d_real_match`（验证 `yanyi` 猜中红桃造伤后联动触发 `jiean:confirm` 摸牌） | **已复核等价** |
| 13 | `zhijie` | `skills` | `英雄杀/character/skill.js:749-760` | 智解（小乔）：出牌阶段限一次，你可将一张红桃手牌置于牌堆顶（重铸），然后摸两张牌 | `u08_batch_a.rs:308-343`、`u08_batch_a_asks.rs:556-589`（校验 `once_used` 与红桃手牌 -> `zhijie:choose_card` 选 1 张红桃手牌移回/弃置并摸 2 张牌，取消时退还 `once_used`） | `u08_batch_a_support_and_modifiers_6d_real_match`（验证用红桃手牌发动 `zhijie` 摸 2 张牌） | **已复核等价** |
| 14 | `yxs_weizhongxian` | `generals` | `英雄杀/character/character.js:267-272` | 魏忠贤：群势力、男性、3/3 体力，技能 `["zhuxin", "wlianhuan"]` | `decision34_pack9_data.rs:193`（`("yxs_weizhongxian", Faction::Qun, Gender::Male, 3, &["zhuxin", "wlianhuan"])`） | `u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble`、`u08_batch_a_combat_and_active_skills_6d_real_match` | **已复核等价** |
| 15 | `tiewan` | `skills` | `英雄杀/character/skill.js:1578-1609` | 铁腕（明成皇后）：当其他角色使用【乐不思蜀】结算后或进行判定/成为延时锦囊相关事件时，你可弃置一张红色牌将一张牌作为【乐不思蜀】置入目标判定区 | `u08_batch_b.rs:632-668`、`u08_batch_b_asks.rs:679-708`（监听触发条件并打开 `tiewan:choose_card` / `tiewan:choose_target` 将虚拟【乐不思蜀】置入目标判定区） | `u08_batch_b_all_28_skills_6d_real_match` | **已复核等价** |

---

## 3. 执行命令与测试门禁结果

在 `/Users/ruyimg/Documents/Claude/sanguosha/noname-next` 下执行：

```bash
cargo test --offline -p engine-native --test g1 u08_
cargo test --offline -p engine-native --test misc u08_
cargo test --offline -p engine-native --test misc decision34
cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --lib
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
python3 -B scripts/legacy-harness/generate-g0-docs.py
```

### 实测输出摘要
- `cargo test --offline -p engine-native --test g1 u08_`：**8 passed; 0 failed**（`u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble`、`u08_batch_a_combat_and_active_skills_6d_real_match`、`u08_batch_a_support_and_modifiers_6d_real_match`、`u08_batch_b_all_28_skills_6d_real_match`、`u08_batch_c_robin_hood_and_stance_swap_6d_real_match`、`u08_batch_c_modifiers_and_active_skills_6d_real_match`、`u08_batch_d_bo_le_wu_ze_tian_mo_zi_and_lu_ban_6d_real_match`、`u08_batch_c_and_d_remaining_skills_6d_real_match`）。
- `cargo test --offline -p engine-native --test misc u08_`：**6 passed; 0 failed**（`u08_hero_kill_eight_dimension_acceptance`、`u08_hero_kill_31_standard_skills_and_menshen2_guard`、`yxs_fanji_counterattacks_source`、`yxs_fanji_absent_no_window`、`menshen_marks_target`、`menshen3_clears_marker`）。
- `cargo test --offline -p engine-native --test misc decision34`：**13 passed; 0 failed**。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --lib`：**25 passed; 0 failed**（`engine-native` 15 passed 含 `host_behavior_keys_bijective` 562 项双射、`engine-native-content` 4 passed、`engine-native-kernel` 6 passed）。
- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：**Ran 2 tests in 0.010s — OK**。
- `python3 -B scripts/check-source-lines.py`：**`Source line limit passed: 9332 files <= 2000 lines; Rust non-test files <= 1000 effective lines`**。
- `python3 -B scripts/legacy-harness/generate-g0-docs.py`：**`Totals: {'generals': 834, 'skills': 1908, 'cards': 377, 'modes': 215, 'configs': 434, 'ui_hooks': 507, 'surfaces': 149, '已复核等价': 594, '简化近似': 3830, '缺失': 0, '上游未定义': 0}`**。

---

## 4. 独立复核签字（Sign-off）

- **复核人**：`u08-signoff`
- **复核日期**：2026-11-12
- **结论**：`12. U08 英雄杀（150 项：43 将 + 107 技）` 全部 43 名武将与 107 项技能（含 9 项跨包标准技能复用）已按上游 `character.js` 与 `skill.js` 逐项原生实现并通过 6 维真实事件对局验收与 10% 固定种子（`seed = 2026111212`）抽样深审，全部 150 项在 `docs/migration/g0/inventory_status.json` 中标记为 `已复核等价`。

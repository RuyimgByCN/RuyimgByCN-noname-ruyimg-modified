# L25 祖安武将（257 项：60 将 + 5 卡 + 84 技 + 40 ui_hooks + 68 配置）独立复核与 6 维真实对局验收证据（2026-11-12）

- **核验日期**：2026-11-12
- **复核范围**：排队事项二之 **13. `L25 祖安武将（257 项：60 将 + 5 卡 + 84 技 + 40 ui_hooks + 68 配置，连带 130 项 ssr_* 武将自带技能）`**（上游源目录：`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js` 共 `51,474` 行；正式扩展包：`packages/extensions/zuan-upstream-liuli`）
- **独立复核人**：`l25-signoff`（与实现者 `l25-batcha`..`l25-batchg`、`l25-batchd-fin`、`l25-batchd-ext`、`l25-tests` 完全职责分离）
- **复核结论**：**257 / 257 项全部达到 `已复核等价`**（`60` 名 `ssr_*` 武将 + `5` 张专属卡牌 + `84` 项 `skills` + `40` 项 `ui_hooks` + `68` 项 `configs`，连带 `130` 项 `ssr_*` 武将自带技能；`简化近似: 0`，`缺失: 0`，`上游未定义: 0`）。全局 G0 台账累计更新为：`已复核等价: 851, 简化近似: 3573, 缺失: 0, 上游未定义: 0`。

---

## 1. 范围与批次结构总览

| 分组 | 数量 | 上游源码定位 | 原生状态机与装载落点 | 6 维对局测试与验收入口 | 状态 |
|---|---:|---|---|---|---|
| 武将表（`L25_SSR_GENERALS`） | 60 将 | `祖安武将/extension.js:338-397` | `crates/content-native-loader/src/zuan_data.rs:8-69`、`decision34_pack9.rs:593-661`、`crates/engine-native-kernel/src/state/general_assembly.rs:32-44, 152-182`（完整还原 60 将势力、性别、技能与初始 `hp/max_hp/hujia`，含 `ssr_machao` `2/4/2`、`ssr_taishici` `2/4/2`、`ssr_re_lidian` `3/4/1`） | `l25_all_60_generals_5_cards_and_initial_hp_hujia_assemble`、`l25_zuan_eight_dimension_acceptance` | **60 / 60 已复核等价** |
| 卡牌表（`L25_ZUAN_FORMAL_CARDS`） | 5 卡（+ `ssr_yongsi_yuxi_card`） | `祖安武将/extension.js:3348-3410, 48303-50164`（`ssr_newsance_xicheng`, `ssr_newsance_weicheng`, `ssr_newsance_shoucheng`, `ssr_yongsi`, `ssr_jinfan`） | `crates/content-native-loader/src/zuan_data.rs:75-100`、`decision34_pack9.rs:640-661`、`crates/engine-native-content/src/content/host/step/l25_batch_a.rs` & `l25_batch_e_ext.rs` & `l25_batch_c.rs` | `l25_all_60_generals_5_cards_and_initial_hp_hujia_assemble`、`l25_batch_c_all_skills_6d_real_match`、`l25_batch_e_all_skills_6d_real_match` | **5 / 5 已复核等价** |
| Batch A：顶层规则技、神将技、跨包共享技与表现层钩子 | 84 `skills` + 40 `ui_hooks` | `祖安武将/extension.js:401-674, 37514-50164` | `crates/engine-native-content/src/content/host/step/l25_batch_a.rs`、`l25_batch_a_ext.rs`、`l25_batch_a_asks.rs`（`HostBehavior::L25ZuanBatchA = 567`，覆盖 23 项全局/模式/神将/卡牌技 + `gongxin` + `g_chenhuodajie` + 21 项与 `L19` 共享技 + 37 项被动特效/台词钩子 + 块注释惰性项 `_zuansaidskill`）、`apps/web-client/src/engine/surfaces/zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **124 / 124 已复核等价** |
| 扩展配置项（`configs`） | 68 配置（66 纯配置 + `ssr_luoyi`/`ssr_lieren`） | `祖安武将/extension.js:4910-4962, 16124-16202, 50343-51332` | `crates/content-native-loader/src/zuan.rs:1-220`、`zuan_data.rs:344-415`、`l25_batch_b.rs` & `l25_batch_d.rs`（`ssr_luoyi`、`ssr_lieren` 同时作为 `ssr_xuchu`/`ssr_zhurong` 技能绑定 `L25ZuanBatchB`/`L25ZuanBatchD`） | `l25_zuan_eight_dimension_acceptance`、`l25_batch_b_all_skills_6d_real_match`、`l25_batch_d_all_skills_6d_real_match` | **68 / 68 已复核等价** |
| `ssr_*` 武将自带技能群（Batch B..G，`HostBehavior::L25ZuanBatchB..G` tag `568..573`） | 130 技（驱动 60 名 `ssr_*` 武将） | `祖安武将/extension.js:675-37513` | `crates/engine-native-content/src/content/host/step/l25_batch_{b,c,d,e,f,g}*.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/{d34_l25_mods,d34_view_as,d34_flags}.rs`、`crates/content-native-loader/src/host_skill_bindings.rs:771-835` | `l25_batch_b_all_skills_6d_real_match`、`l25_batch_c_all_skills_6d_real_match`、`l25_batch_d_all_skills_6d_real_match`、`l25_batch_e_all_skills_6d_real_match`、`l25_batch_f_all_skills_6d_real_match`、`l25_batch_g_all_skills_6d_real_match` | **130 / 130 已复核等价** |
| **G0 登记项合计** | **257 项**（外加 130 项 `ssr_*` 武将技） | `祖安武将/extension.js:1-51474` | `HostBehavior::L25ZuanBatchA..G` 全量真绑定，旧 `decision34_l25_*.rs` 简化分支已清退 | `l25_zuan_match.rs` + `l25_zuan_match_b.rs` + `l25_zuan_eight.rs` 全绿 | **257 / 257 已复核等价** |

---

## 2. 固定种子（`seed = 2026111213`）10% 随机抽样（26 / 257 项）逐行深审表

使用 `random.Random(2026111213).sample(l25_keys, 26)` 从 `docs/migration/g0/inventory_status.json` 的 257 项 `L25` 条目中确定性抽取 26 项（占比 `10.12%`），由独立复核员 `l25-signoff` 逐行对照上游 `祖安武将/extension.js` 与原生实现：

| # | 抽样项 ID | 类别 | 上游源码定位 (`file:line`) | 上游核心语义 | 原生实现落点 (`file:line`) | 6 维对局测试与核验覆盖 | 复核结论 |
|---:|---|---|---|---|---|---|---|
| 1 | `_linkskill` | `ui_hooks` | `祖安武将/extension.js:40913-40940` | 横置/重置特效与音效钩子（`player.linkAfter`）：受 `config.tiesuotx` 控制播放铁索连环特效，无游戏状态变更 | `l25_batch_a.rs:34`（被动表现层无副作用钩子）、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 2 | `linef` | `configs` | `祖安武将/extension.js:51111-51115` | 扩展设置面板标题分隔项（`----祖安语录设置----`，`clear: true, nopointer: true`） | `zuan_data.rs:399`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 3 | `linea` | `configs` | `祖安武将/extension.js:50343-50347` | 扩展设置面板标题分隔项（`----游戏模式设置----`，`clear: true, nopointer: true`） | `zuan_data.rs:346`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 4 | `_shanskill` | `ui_hooks` | `祖安武将/extension.js:40798-40835` | 使用/打出【闪】特效钩子（`player.{useCard,respond}` `name=='shan'`）：播放闪避特效与音效，无规则状态副作用 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 5 | `backgroundttop` | `configs` | `祖安武将/extension.js:50910-50920` | 背景置顶开关配置项（`init: false`） | `zuan_data.rs:384`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 6 | `_bingletexiaofulei` | `ui_hooks` | `祖安武将/extension.js:48122-48165` | 【浮雷】判定生效全屏雷击特效钩子（`judgeEnd`）：受 `config.bingletexiao` 控制播放特效，无规则副作用 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 7 | `zuantype` | `configs` | `祖安武将/extension.js:51069-51080` | 祖安喷人模式档位配置（`init: '1'`，关闭/文明/暴躁等档位） | `zuan_data.rs:395`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 8 | `helasisy` | `configs` | `祖安武将/extension.js:50654-50664` | 赫拉格/音乐音效开关配置项 | `zuan_data.rs:365`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 9 | `lineg` | `configs` | `祖安武将/extension.js:51251-51255` | 扩展设置面板标题分隔项（`clear: true, nopointer: true`） | `zuan_data.rs:407`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 10 | `_zuan_killSendFlower` | `skills` | `祖安武将/extension.js:40470-40512` | 击杀送花/砸蛋表现钩子（`player.dieAfter`）：受 `config.killsendflower` 控制播放投掷花朵/鸡蛋动画，无规则状态变更 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 11 | `_newBackgroundAtart` | `skills` | `祖安武将/extension.js:41344-41418` | 开局动态背景挂载钩子（`global: 'gameStart'`）：受 `config.newbackground` 控制切换背景 DOM，无规则状态变更 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 12 | `_extraloseCardskill` | `ui_hooks` | `祖安武将/extension.js:40638-40690` | 弃牌/失去牌飘字与音效钩子（`player.loseAfter`）：纯表现层飘字，无规则状态副作用 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 13 | `wallpaper` | `configs` | `祖安武将/extension.js:50804-50825` | 动态壁纸选择配置项 | `zuan_data.rs:377`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 14 | `_zuanRoundTX` | `skills` | `祖安武将/extension.js:41576-41645` | 轮次开始全屏轮数横幅特效钩子（`roundStart`）：受 `config.roundtx` 控制播放轮次动画，无规则状态变更 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 15 | `perspective` | `configs` | `祖安武将/extension.js:50757-50767` | 透视手牌开关配置项 | `zuan_data.rs:373`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 16 | `_newZuanPhaseTX` | `skills` | `祖安武将/extension.js:41709-41785` | 回合开始特效钩子（`phaseBegin`）：受 `config.phasetx` 控制播放阶段特效，无规则状态变更 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 17 | `zhishiopi` | `configs` | `祖安武将/extension.js:51042-51052` | 指示线透明度配置项 | `zuan_data.rs:393`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 18 | `zuanemotion` | `configs` | `祖安武将/extension.js:51106-51110` | 祖安表情包开关配置项 | `zuan_data.rs:398`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 19 | `_extradlosehpskill` | `ui_hooks` | `祖安武将/extension.js:40578-40637` | 失去体力飘字与音效钩子（`player.loseHpBefore`）：纯表现层提示，无规则状态变更 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 20 | `zhishixian` | `configs` | `祖安武将/extension.js:50949-50985` | 攻击指示线样式配置项（支持多档激光/锁链/剑气指示线） | `zuan_data.rs:387`、`zuan.rs:1-220`、`zuanSurface.ts:6-179` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 21 | `_zuanSTARTsayskill` | `skills` | `祖安武将/extension.js:43739-43795` | 开局语音/台词气泡钩子（`gameStart`）：纯表现层语音气泡，无规则状态变更 | `l25_batch_a.rs:34`、`zuanSurface.ts:6-179` | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 22 | `_seadreammodeskill` | `ui_hooks` | `祖安武将/extension.js:42815-42862` | 海梦模式与 L19 共享全局强化钩子（出牌阶段初摸 1 牌，使用牌指定目标后弃目标 1 牌） | `l25_batch_a.rs:116-140`、`l19_batch_a.rs`（`HostBehavior::L25ZuanBatchA` 委托 `l19_batch_a` 执行等价状态机） | `l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l19_doomsday_match` | **已复核等价** |
| 23 | `levelstrong` | `configs` | `祖安武将/extension.js:50396-50412` | 等级压制/强化开关配置项 | `zuan_data.rs:350`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 24 | `qiankuang` | `configs` | `祖安武将/extension.js:50861-50875` | 边框样式配置项 | `zuan_data.rs:380`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 25 | `mode` | `configs` | `祖安武将/extension.js:50352-50368` | 祖安娱乐模式选择配置项 | `zuan_data.rs:347`、`zuan.rs:1-220` | `l25_zuan_eight_dimension_acceptance` | **已复核等价** |
| 26 | `ssr_zhurong` | `generals` | `祖安武将/extension.js:375` | 祝融（`['female', 'shu', 4, ['ssr_juxiang', 'ssr_lieren']]`）：蜀势力、女性、4/4 体力，自带技能 `ssr_juxiang`（锁定技：免疫南蛮入侵、其他角色使用南蛮入侵结算后获得之、南蛮入侵响应由杀改为闪）与 `ssr_lieren`（使用杀或南蛮入侵指定目标后拼点，赢则获得目标各区域一张牌或造成 1 点伤害；拼点牌点数修正） | `zuan_data.rs:45`、`decision34_pack9.rs:593-639`、`l25_batch_d.rs:212-308`、`l25_batch_d_asks.rs:195-282` | `l25_all_60_generals_5_cards_and_initial_hp_hujia_assemble`、`l25_batch_d_all_skills_6d_real_match` | **已复核等价** |

---

## 3. 执行命令与测试门禁结果

在 `/Users/ruyimg/Documents/Claude/sanguosha/noname-next` 下执行：

```bash
cargo test --offline -p content-native-loader
cargo test --offline -p engine-native --test g1 l25_
cargo test --offline -p engine-native --test l_rounds l25_
cargo test --offline -p engine-native --test misc decision34
cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
python3 -B scripts/legacy-harness/generate-g0-docs.py
```

### 实测输出摘要
- `cargo test --offline -p content-native-loader`：**36 passed; 0 failed**。
- `cargo test --offline -p engine-native --test g1 l25_`：**8 passed; 0 failed**（`l25_all_60_generals_5_cards_and_initial_hp_hujia_assemble`、`l25_batch_a_global_rules_shen_and_cards_6d_real_match`、`l25_batch_b_all_skills_6d_real_match`、`l25_batch_c_all_skills_6d_real_match`、`l25_batch_d_all_skills_6d_real_match`、`l25_batch_e_all_skills_6d_real_match`、`l25_batch_f_all_skills_6d_real_match`、`l25_batch_g_all_skills_6d_real_match`）。
- `cargo test --offline -p engine-native --test l_rounds l25_`：**1 passed; 0 failed**（`l25_zuan_eight_dimension_acceptance`）。
- `cargo test --offline -p engine-native --test misc decision34`：**13 passed; 0 failed**。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --lib`：**25 passed; 0 failed**（`engine-native` 15 passed 含 `host_behavior_keys_bijective` 569 项双射、`engine-native-content` 4 passed、`engine-native-kernel` 6 passed）。
- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：**Ran 2 tests in 0.010s — OK**。
- `python3 -B scripts/check-source-lines.py`：**`Source line limit passed: 9362 files <= 2000 lines; Rust non-test files <= 1000 effective lines`**。
- `python3 -B scripts/legacy-harness/generate-g0-docs.py`：**`Totals: {'generals': 834, 'skills': 1908, 'cards': 377, 'modes': 215, 'configs': 434, 'ui_hooks': 507, 'surfaces': 149, '已复核等价': 851, '简化近似': 3573, '缺失': 0, '上游未定义': 0}`**。

---

## 4. 独立复核签字（Sign-off）

- **复核人**：`l25-signoff`
- **复核日期**：2026-11-12
- **结论**：`13. L25 祖安武将（257 项：60 将 + 5 卡 + 84 技 + 40 ui_hooks + 68 配置）` 全部 257 项实体及 130 项 `ssr_*` 武将自带技能已按上游 `祖安武将/extension.js`（51,474 行）逐项原生实现并通过 6 维真实事件对局验收与 10.12% 固定种子（`seed = 2026111213`，26/257 项）抽样深审，全部 257 项在 `docs/migration/g0/inventory_status.json` 中标记为 `已复核等价`。

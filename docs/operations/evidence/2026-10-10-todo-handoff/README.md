# 2026-10-10 待办整理：源码核对摘要

核验位置：`noname-next` git `729f0d9a`。它相对基线 `72583b2b` 只改了文档，产品源码相同；规则 v192、NGSA 存档格式 v41。本轮读了源码、上游归档和既有证据，把[进度主稿](../../../architecture/native-progress.md#后续待做按阻塞关系)的待做表改成可直接领取的任务。**产品源码和包数据都没有改，也没有重跑全量测试**；只因改了覆盖清单，重跑了清单联动守护（见 §6）。全量结果仍以[2026-10-10 进度核对](../2026-10-10-progress-check/README.md)为准。下文行号均对应 `729f0d9a`。

## 1. host.rs 的 14 个 `TODO(事故恢复)` 空分支（主稿 T1）

查找方法：
- 用 `rg -n "TODO\(事故恢复\)" crates/engine-native/src/content/host.rs` 定位空分支。
- 用各行为的技能键常量（`WUDIAN_*_KEY`、`XJ_TANNANG_KEY`、`GJ_FOOD_YUANBAOROU_KEY`）执行 `rg -nw`，找到效果实现的位置。
- 上游定义在 `archive/upstream/noname/apps/core/extension/` 下按技能名定位，看其顶层键。

14 个分支都在宿主机器的 `step` 里，内容只有 `let _ = ctx; Flow::Done`。规则基准把它们对应的批次写成目录标记技、内联钩或被动无机器：效果由内核在固定位置按技能键实现，宿主机器本身没有分段。

旁证是工作区外的 `~/host-recovery-20261010/replayed_pre_incident_16128.rs`。它是较早批次的会话回放版本，不在版本控制中。它的 `step`（约 1257–1267 行）把 WudianCiqiu、GujianFoodYuanbaorou 等“被动状态行为：无机器步骤”并为同一个 `=> Flow::Done` 分支，MoshouImmunity 也是 `=> Flow::Done`。其余 11 项在该版本之后才加入，回放文件里没有。该目录的 43 个文件（约 30 MB）都是重建中间产物，没有完整的事故前原文。

| 行为（tag） | 空分支 | 包 / 技能 | 上游定义（相对 extension/） | 原生效果位置 | 规则条目 | 回归测试 |
|---|---|---|---|---|---|---|
| MoshouImmunity（9） | host.rs | 玩点论杀 / moshou | 玩点论杀/character/skill.js:43，只有 mod | host.rs `blocks_delayed_trick`，乐/兵在命令边界过滤 | v36 | g1_wudian `moshou_blocks_delayed_trick_target`、`feishi_moshou_declaration_wired_to_host_behavior` |
| GujianFoodYuanbaorou（28） | host.rs | 古剑 / 元宝肉伴随技 | 杀海拾遗/card/gujian/skill.js:343，trigger+mod | engine.rs、2501（杀次数校验）、3329 | v48 | g1_shahai_gujian `food_effects_batch_two` 只断言“额外一张杀放行”，没有每回合一次上限的负例；其中 `def_ybr` 取出后未用 |
| WudianCiqiu（97） | host.rs | 玩点论杀 / ciqiu | 玩点论杀/character/skill.js:75，trigger forced | kernel.rs（commit_damage） | v111 | g1_wudian `hanlong_ciqiu_sha_healthy_bonus_and_dying_mark` |
| WudianChezhen（102） | host.rs | 玩点论杀 / chezhen | 玩点论杀/character/skill.js:408，只有 mod | state.rs:700（距离） | v115 | g1_wudian `tianyu_chezhen_distance_mod` |
| WudianTuji（105） | host.rs | ow / tuji | 杀海拾遗/character/ow/skill.js:2163，只有 mod | state.rs:663（距离） | v118 | g1_wudian `ow_shibing_tuji_and_mujing` |
| WudianGuangshuSpade（111） | host.rs | ow / guangshu_spade | ow/skill.js:3110，trigger forced | kernel.rs（阶段结束直接造成雷伤）；另有 `TriggerMeta`（host.rs，PhaseEnd）与过滤（host.rs） | v119 | g1_wudian `zhixuzhiguang_guangshu_suit_marks` |
| WudianDuwen（114） | host.rs | ow / duwen | ow/skill.js:1477，trigger forced | kernel.rs（commit_damage） | v122 | g1_wudian `heibaihe_duwen_equal_hand_damage_bonus` |
| WudianFeiren（126） | host.rs | ow / feiren | ow/skill.js:2331，trigger+mod | kernel.rs；engine.rs、2447 | v129 | g1_wudian `yuanshi_feiren_bonus_range_and_club_extra_target`、`yuanshi_feiren_chain_damage_not_bonused` |
| WudianXiandan（139） | host.rs | ow / xiandan | ow/skill.js:2105，trigger direct | kernel.rs（ShaResolve 内联询问） | v140 | g1_wudian `sishen_xiandan_red_direct_black_bonus` |
| WudianJijia（140） | host.rs | ow / jijia | ow/skill.js:1975，trigger+mod+init | kernel.rs（伤害抵扣）、2006（手牌上限） | v141 | g1_wudian `dva_jijia_absorbs_damage_and_hand_limit` |
| WudianLongxiang（147） | host.rs | xianjian / longxiang | 杀海拾遗/character/xianjian/skill.js:2061，trigger，非 forced（有 check） | kernel.rs（ShaResolve 内联，不询问；v146 已登记此口径） | v146 | g1_wudian `wenhui_longxiang_random_discard_diff` |
| XianjianTannang（149） | host.rs | xianjian / tannang | xianjian/skill.js:3948，viewAs+mod | engine.rs（梅花当顺手牵羊）、2623（距离豁免） | v148 | g1_wudian `hanlingsha_tannang_snatch_unlimited_range` |
| XianjianXuanyan（158） | host.rs | xianjian / xuanyan | xianjian/skill.js:2740，只有 `group: ["xuanyan2", "xuanyan3"]` | 子技 XianjianXuanyan2/3：过滤在 host.rs/3698，执行在 host.rs/10939 | v152 | g1_wudian `xuanxiao_xuanyan_ningbin_fire_thunder` |
| WudianLueying（161） | host.rs | xianjian / lueying | xianjian/skill.js:1910，trigger，非 forced，usable 1 | kernel.rs（ShaResolve 内联） | v154 | g1_wudian `muchanglan_lueying_gain_and_victim_discard` |

结论：上游只有 mod/group 的 4 项（墨守、车阵、突击、玄炎）在宿主机器里没有事可做。其余 10 项上游有 trigger/content，原生改为内核内联实现，空分支与规则基准记的设计一致。逐项确认后可删掉 TODO，并写明效果所在位置。

**附带发现（不属于空分支问题，归主稿 T11）**：掠影的 v154 条目写“登记差异：无”，但与上游不一致。上游 filter 要求目标有牌**且**另有一名其他有牌角色才能发动，而且是非强制发动。原生 kernel.rs 只要目标有牌就先获得一张，再按条件询问选人，也没有发动确认。

## 2. CI Gate 1b 的 12 条 lib 警告（主稿 T3）

来源：[gate1b-check.log](../2026-10-10-progress-check/gate1b-check.log)。逐条读了源码，都是冗余绑定、多余 `mut` 或重建残留语句，没有发现对应的逻辑缺失。

| 位置 | 警告 | 阅读结论 | 处理建议 |
|---|---|---|---|
| host.rs | 未用变量 `oc` | 偷梁换柱结算：`if let Some(oc) = own_card` 只用来判断有没有自选牌，同色比较用的是 `own_color` | 改为 `if own_card.is_some()` |
| host.rs | 未用变量 `eff` | 乐动 Act 段算了没用；TargetChoice 段（9801 行）重新读回合状态后才使用 | 删除 Act 段这一行 |
| host.rs | 未用变量 `picked` | 羊皮卷发现：外层 `if let Some(picked)` 被内层用 RNG 选出的同名变量遮蔽 | 改为 `if !cands.is_empty()`，RNG 调用次数不变 |
| host.rs | 未用变量 `d` | 空的 `if let` 块，旁注写明 target 未用，属重建残留 | 删除整块 |
| host.rs | 字段赋值给自己 | `self.card = self.card; // 保持本体` | 删除 |
| host.rs | 未用变量 `def` | `sync_land_skill` 首行查依夏兰重铸技，后文按映射逐个查，这一行没用上 | 删除 |
| host.rs | 未用导入 `HostBehavior as B` | `card_kind_host_skills` 内的别名没用到 | 删除 |
| state/general_assembly.rs:17 | 未用变量 `chongzheng` | `debug_add_fellow` 只用 match 判断是否 Boss 模式 | 改用 `matches!` 判断 |
| kernel.rs | 未用变量 `boss` | Boss 死亡座位的重整 tick 只用阈值 | 该绑定改为 `_` |
| kernel.rs | 多余的 `mut` | KOF 终局分支里 `mut games` 未被修改 | 去掉 `mut` |
| presentation.rs:185 | 未用变量 `count` | 重整 tick 映射为 `Recovered { amount: 0 }`，计数没用上 | 改为 `count: _` |
| snap.rs | 未用变量 `liuli_card` | 流离选择派生只用 `liuli_stage`，模式里已有 `..` | 删除该字段绑定 |

## 3. 环境变量调试输出（主稿 T2）

`rg -n 'env::var' crates/engine-native/src` 共 4 处命中：

| 位置 | 开关 | 所在函数 | 读取频率 | 关联 |
|---|---|---|---|---|
| resolution.rs:100 | `PT_DBG2` | `push_frame` | 每次压帧读一次，只在 Death 帧打印 | — |
| resolution.rs:287 | `PT_DBG2` | `plan_triggers` | 每次规划触发读一次 | — |
| resolution.rs:324 | `PT_DBG` | `plan_triggers` | 每次规划触发读一次 | ignore 测试 `g1_huanle.rs xly_state_growth_probe` 的说明写着“PT_DBG=1 观察规模曲线”；该测试自己用 eprintln 输出 PROBE 行，不依赖此处 |
| machine.rs:518 | `XLY_DBG` | `expire_temporary_skills` | 每次处理临时技能到期 | 写轮眼临时技能诊断 |

`xly_state_growth_probe` 的 ignore 说明已出现在 10-07 [帧生命周期证据](../2026-10-07-e03-frame-lifecycle/test-logs.txt)的测试日志中。可见 PT_DBG 不是 10-10 恢复期才加的。PT_DBG2 与 XLY_DBG 何时引入，git 基线里查不出来。主稿原写“恢复期调试探针”，本次改为按来源描述。

## 4. 其他核对

- **cardpile 装载**：`g1_cardpile.rs` 读取已安装包的 `assets/pile.json`，再用测试内的 `parse_spec`（140 行）转成 `PileSpec`，最后调 `engine_native::content::pile::extension_pack_from_pile`。`content-native-loader` 只有 `bridge_pack`/`bridge_generals`（lib.rs:286/389），没有 pile 入口。10-07 迁移的“六桥”实际是 huanle/hearth/gwent/yunchou/d3/wuxing，cardpile 不在其中。
- **转换包测试**：15 个包里，除 g1_cardpile 外，其余 g1 测试和 `e02_c3_lifecycle` 都经 `content_native_loader::bridge_pack/bridge_generals` 装载。ow、xianjian 两个武将包的技能回归写在 g1_wudian，但没有真实宿主全生命周期测试。
- **调试测试**：`tests/dbg4.rs` 的 `dbg_nullify` 只打印、无断言，文件时间 9-16，不是本次事故的产物。
- **恢复期备份**：`src/content/host.rs.pre_assembly`（520,051 字节）和 `host.rs.recovered-v192`（559,445 字节）都在 git 基线里，不参与编译。
- **上游变化面**：来自 [inventory-check.txt](../2026-10-10-progress-check/inventory-check.txt)。noname `5c02a745..df2682e6` 的改动集中在三处：
  - `mode/guozhan`，33 个文件；
  - `card/guozhan.js`、`card/huodong.js`、`card/jiubian.js`；
  - `library/element/content.ts`、`player.js`、`character.js` 和 `library/skill.js`；另有 `mode/{chess,single,tafang}.js`。

  扩展目录与 `card/standard.js` 没有改动。Liuli 有 10 个来源实例的字节变化，已转换的 wuxing、cardpile 不在其中。
- **Liuli 本地修改**：9-23 与 10-06 都按工作区字节核验，当时 50 个文件有本地修改。10-08 执行 `git reset`（reflog 记为 `moving to HEAD`）后只剩 7 个，stash 为空。被丢弃的 43 个文件改动在 git 里没有记录，以哪种口径为准需要用户决定（主稿 D1）。**同日更正**：复算表明其中 40 个只是换行（CRLF→LF）差异，内容与 9-23 一致，只有史诗卡牌 3 个文件有 148 字节无法由换行解释，见[文档清理核对](../2026-10-10-doc-cleanup/README.md)；主稿 D1 已据此改写。
- **wasm 与前端工具**：本机已装 `wasm32-unknown-unknown` 目标，以及 `wasm-pack`、`pnpm`、`node`。磁盘余量 215 GiB。

## 5. 文档改动

- [进度主稿](../../../architecture/native-progress.md#后续待做按阻塞关系)：“后续待做”改为 T1–T14 任务表，加了“接手方式”和“需用户决定”。整理前的原表原样移入[批次流水存档](../../../architecture/native-progress-history.md#2026-10-10-整理前的待做表原文)。
- [覆盖清单](../../../native-coverage-manifest.md)范围 C 第一条，原写“存档未知牌种编号与载荷读尽仍未修”。实际 10-07 已加固，回归 `archive_hardening_rejects_unknown_kind_and_trailing_bytes` 在 10-10 全量中通过（[日志](../2026-10-10-progress-check/engine-native-tests.log) 6922 行），已改正。
- [两仓台账](../../../migration/upstream-extension-reconstruction.md) E02 行的“下一步”仍写“迁移其余六条桥接路径”和炉石键规范化。这两项 10-07 已完成，已改为当前剩余项。
- 发现但未改的不一致：覆盖清单 `kof_session` 行仍是“近似”，并写着“会话矛盾仍接受…恢复续跑待修”，与同页模式行和规则基准（`kof_session_consistency_and_midrestore` 已固化）不一致，需对照测试实际覆盖再改（主稿 T14）。掠影的差异见 §1。

## 6. 验证

见同目录 [checks.txt](checks.txt)：本地链接与锚点检查，以及覆盖清单联动守护 `cargo test --offline -p engine-native --test replay_scenarios`。

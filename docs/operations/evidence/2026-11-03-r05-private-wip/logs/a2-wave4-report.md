# R05-A2 wave4 迁移报告：TURN_BEGIN_WINDOW(21) + PREPARE_BEGIN(10)

- 工作区：`/private/tmp/noname-r05/worktree`（分支 r05-private）
- 基线 HEAD 起点：4c5ac53a（期间协调者并入 wave4a/4b/4c＝b1024db8，最终树含两方改动）
- 依据：`docs/operations/evidence/2026-11-03-r05-private-wip/audit.json`
- 核验日期：2026-11-03；命令：`CARGO_TARGET_DIR=/private/tmp/noname-r05/target CARGO_INCREMENTAL=0 timeout 1500 cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast`（日志：`/private/tmp/noname-r05/evidence/a2-wave4.log`）
- 结果：**1916 通过 / 1 失败 / 75 忽略**——唯一失败仍为基线已登记的 `replay_scenarios::replay_scenarios_match_golden`（golden 失配清单与 wave3-verify 日志逐条一致，本轮未新增漂移；重录归协调者 J 步）
- 成功 31 条 / 跳过 0 条（K23/K46/K47/K48 及 K02/K04/K09 为内核项归协调者，未计入本波）

## TURN_BEGIN_WINDOW（21 条：H06,H12,H17,H19,H22,H24,H25,H26,H28,H30,H32,H33,H35,H39,H40,H41,H43,H44,H45,H49,S14）

三处同构迁移：metadata `event: EventKind::PhaseBegin → EventKind::TurnBegin`（BeforeTriggers 段派发）；filtering 匹配改 `EventData::TurnBegin{player}`（保留 json 注明的本人/global 门）；factory/step 的 Phase 帧提取臂同步。

| id | 技能 | metadata（file:line） | filtering（file:line） | factory/step | 测试影响 |
|---|---|---|---|---|---|
| H06 | YxsMenshen3 门神·清除 | metadata/common.rs:193 | filtering/common.rs:135（TurnBegin player==owner；**Death 面补本人门 player==owner**＝上游 dieBegin） | — | u08_menshen_batch2.rs：清除点从「任意阶段开始」推进到持有者下一回合开始（48 步内早停），A2 H06 注释 |
| H12 | Mailun 脉轮 | metadata/common.rs:416 | filtering/common.rs:379 | — | swd_mailun_modes.rs 4 用例：窗口改在 0 号下一回合开始（新增 advance_to_turn_begin 助手）；mode3 改同种子对照局差值断言（第二轮起随机将修正叠加，绝对值断言不再成立） |
| H17 | YunchouDiyguhuo 蛊惑·变体 | metadata/common.rs:655 | filtering/common.rs:488 | factory.rs:226（Phase→TurnBegin 臂） | 无（nc08_upstream_yunchou_guhuo 未锁具体相位，全绿） |
| H19 | GujianFoodYougeng 酉羹 | metadata/common.rs:757 | filtering/common.rs:645 | — | 无（nc08_upstream_gujian_yougeng 全绿） |
| H22 | GujianFoodZiyangdan 紫阳丹 | metadata/common.rs:819 | filtering/common.rs:650 | step/gujian.rs:425（注释同步） | g1_shahai_gujian/remaining.rs：End 帧不衰减（保持 3），advance_to_play 途经下一回合开始衰减到 2，A2 H22 注释 |
| H24 | HuanleXuelunyangCard 雪轮羊 | metadata/common.rs:912 | filtering/common.rs:681 | — | 无（g1_huanle/xly 全绿） |
| H25 | SwdHyunshen2 魂身·收束 | metadata/swd.rs:18 | filtering/swd.rs:29 | — | 无（g1_shahai_swd/hyunshen 全绿） |
| H26 | SwdDaixing2 待行·清甲 | metadata/swd.rs:37 | filtering/swd.rs:40 | — | 无（g1_shahai_swd/daixing 全绿） |
| H28 | SwdJikong 集控 | metadata/swd.rs:127（主事件 TurnBegin；extra CardsLost 保持） | filtering/swd.rs:105（TurnBegin 臂＋CardsLost 臂保持） | — | swd_jikong_thunder_sha.rs 2 用例重写：确认窗在下一回合开始；cancel_refunds 改 once_used 状态断言（usable:1 消费点在选目标步），A2 H28 注释 |
| H30 | SwdJinlin2 锦鳞·退 | metadata/swd.rs:185 | filtering/swd.rs:77 | — | g1_shahai_swd/jinlin.rs：阶段 tick 改回合开始 tick（debug_set_turn 不发 TurnBegin，改自然导航 advance_to_next_turn_of），A2 H30 注释 |
| H32 | SwdQinglongzhiguiHeld 青龙之圭 | metadata/swd.rs:382 | filtering/swd.rs:387 | — | 无（nc08_upstream_swd_qinglong 全绿） |
| H33 | SwdHuanglinzhicongHeld 黄麟之琮 | metadata/swd.rs:390 | filtering/swd.rs:458 | — | nc08_upstream_swd_huanglin.rs：candidates 探针 PhaseBegin→TurnBegin，A2 H33 注释 |
| H35 | SwdHujing 壶境 | metadata/swd.rs:497 | filtering/swd.rs:992 | — | 无（lianyaohu_hujing 全绿） |
| H39 | SwdZhenjiu2 枕流触发面 | metadata/swd.rs:848 | filtering/swd.rs:668 | — | 无 |
| H40 | SwdHuanxing 幻形 | metadata/swd.rs:895 | filtering/swd.rs:695 | — | 无（g1_shahai_swd/huanxing 全绿） |
| H41 | SwdHuajin2 化金触发面 | metadata/swd.rs:1001（extra_events PhaseBegin→TurnBegin；主事件 DamageBefore 保持） | filtering/swd.rs:814（自清臂→TurnBegin） | step/swd_damage_reactions.rs:187（Phase{Judge,begin} 臂→TurnBegin） | 无（g1_shahai_swd/huajin 全绿） |
| H43 | SwdFengyinState 机关凤封印 | metadata/swd.rs:1117 | filtering/swd.rs:1062 | — | 无（g1_shahai_gwent 全绿） |
| H44 | SwdGouhunluoSub 勾魂锣·倒计时 | metadata/swd.rs:1173 | filtering/swd.rs:1100（仅来源 player 的回合） | — | g1_wudian/swd.rs：倒计时导航跨约三轮，drain 改 robust 应答（强制 discard_to_limit 的 Pass 不被接受曾致挂死），A2 H44 注释 |
| H45 | WudianGeju 割据 | metadata/wudian.rs:60 | filtering/common.rs:716 | — | 无（g1_wudian/murongziying 全绿） |
| H49 | XianjianQianfang 千方 | metadata/xianjian.rs:415 | filtering/xianjian.rs:241 | — | 无（t9_d5_surface 全绿） |
| S14 | BOSS_SHEDU 蛇毒 | standard/boss_skills.rs:191（PhaseBegin→TurnBegin） | standard/skills/shed_u.rs:10（TurnBegin{player}+shedu>0） | — | 无（boss_xiangliu 全绿） |

## PREPARE_BEGIN（10 条：H02,H09,H14,H15,S03,S05,S09,S11,S12,S13）

metadata event 保持 `EventKind::PhaseBegin`（注释登记），过滤收紧 `EventData::Phase{player, phase: Phase::Prepare, begin: true}`。

| id | 技能 | metadata（file:line，仅注释） | filtering（file:line） | 测试影响 |
|---|---|---|---|---|
| H02 | HlssFengqi 起风 | metadata/common.rs:107 | filtering/common.rs:45（Judge→Prepare） | 无（t11_boss_zhugeliang_hlss 全绿） |
| H09 | Xiuluo 修罗 | metadata/common.rs:299 | filtering/common.rs:230（Judge→Prepare，判定区/手牌门保持） | nc08_upstream_xiuluo.rs：源码锁面改 Phase::Prepare（差异消除），A2 H09 注释；swd_xiuluo_judge_clear 全绿 |
| H14 | BossQingyi 青翼 | metadata/common.rs:591（与 H15 共臂） | content/host/boss.rs:25 wing_round_gate（Judge→Prepare）※ | 无（boss_qiongqi 全绿） |
| H15 | BossChiyi 赤翼 | 同上 | 同上 | 同上 |
| S03 | GUAN_XING 观星 | standard/standard_skills.rs:158（注释） | standard/skills/guan_xing.rs:11（Prepare） | nc08_upstream_guanxing.rs：差异①锁面从「判定阶段占位」改为「A2 S03 对齐」，A2 注释 |
| S05 | LUO_SHEN 洛神 | standard/standard_skills.rs:389（注释） | standard/skills/luo_shen.rs:10（Prepare） | nc08_upstream_luoshen.rs：差异①锁面同样更新，A2 S05 注释 |
| S09 | SHEN_ZHI 神智 | standard/standard_skills.rs:548（注释） | standard/skills/shen_zhi.rs:11（Prepare） | native_kernel/ganfuren.rs：确认门挂起相位 Judge→Prepare（两处导航条件），A2 S09 注释 |
| S11 | WANG_ZUN 妄尊 | standard/standard_skills.rs:618（注释） | standard/skills/wang_zun.rs:12（Prepare＋player≠owner global 门保持） | 无（native_kernel/yuanshu 全绿） |
| S12 | BOSS_FUTAI 覆泰 | standard/boss_skills.rs:57（注释） | standard/skills/futai.rs:9（Prepare） | 无（boss_futai2 全绿） |
| S13 | BOSS_LUANCHANG 乱常（南蛮面） | standard/boss_skills.rs:114（注释） | standard/skills/luan_chang.rs:11（Prepare；万箭面 End 保持 kernel 臂） | 无（boss_taowu 全绿） |

※ H14/H15 说明：审计 native_filter_loc 明示 `common.rs:853+boss.rs:23-52`——wing_round_gate 是过滤实现的一部分且仅此两调用方；列表路径未列 host/boss.rs，但 CI `-D warnings` 下改为内联复制会留下死代码同样过不了门，故在原函数收紧相位（1 行语义改动＋注释），特此登记。

## 其他登记

- **adr0786_j2_package_versions.rs（shiji 锁）**：失败源于协调者同会话提交的 D02（`phase_prepare` 解锁，b1024db8）——完整包拒装锚从 `phase_prepare` 变为首个仍未映射的 `game_started`；按新行为更新断言并注明 A2 D02，测试意图（完整包仍拒装）不变。
- **金哈希**：`replay_scenarios_match_golden` 失配清单（13+ 条 actual/expected）与 wave3-verify 日志逐字节一致——本轮与协调者 wave4a-c 均未进一步漂移，重录归协调者（J 步三步探针）。
- **行数门禁**：`check-source-lines.py` 报 `kernel/events.rs`（1017 有效行）、`archive/machine_encode.rs`（1007 有效行）超 1000 有效行——`git stash` 验证为 HEAD（协调者 wave4a-c）既有，非本轮引入；本轮 31 个文件无门禁违例。
- **cargo check**：三 crate 0 error；`--all-targets` 警告仅协调者 kernel 三处（events.rs:717、phase/judge.rs:3 两处，HEAD 既有），本轮文件零警告。

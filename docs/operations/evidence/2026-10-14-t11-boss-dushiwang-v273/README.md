# T11 都市王片（规则 v273，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（平等王批 cfc8596d 后续片）。上游依据：`boss_dushiwang`（都市王·黄中庸/male/qun/**hp4**/skills `[boss_remen, boss_zhifen, boss_huoxing]`，apps/core/mode/boss.js:1366）；`boss_remen`@3814 / `boss_zhifen`@3856 / `boss_huoxing`@3879；显示名 boss.js:11290–11294——**显示名勘误两处**：boss_zhifen=炙焚（勘验草稿误记「炽焚」）、boss_huoxing=火刑（误记「祸刑」）。

## 三技语义与原生落法

- **热闷**（查询型单面，双消费点）：防具区没有牌时——①plain【杀】（无属性，`kind == Sha`，火/雷杀不覆盖）目标不可指定（PlayCard 拒绝位 `IllegalTargets`，造异 targetEnabled 同族；上游 useCardToBefore cancel→目标不可指定近似登记）；②【南蛮入侵】/【万箭齐发】目标队列窗口前跳过（`AoeResolveMachine` 死亡跳过环扩展 `dushiwang_remen_blocks`——无懈窗与响应段一并跳过）。防具槽口径=`equip_slot()==Armor` 全族（铁面同款）。
- **炙焚**（查询型单面，Prepare-begin 臂，**持有者域**——上游 trigger player phaseZhunbeiBegin）：持有者准备阶段——座位序首个其他（上游 randomGet→确定性近似登记）的首张手牌（上游 randomGet→首张近似）获得（`CardsGiven` 日志）并受 1 点火焰伤害（无来源同上游）。
- **火刑**（Death 触发遗言机，`MachineSnapshot::BossHuoxing` 新变体 tag79/机哈希 79/a0 变体 77）：你死亡时，全部其他存活角色按座位序各受 1 点火焰伤害（上游 sort seat 逐发同口径；emitted=下一待扫描座位索引，中途死亡由存活过滤跳过）。

条目 `("都市王", Qun, &[热闷, 炙焚, 火刑], 4, Male)`（BOSS_GENERALS[51]）；`STANDARD_RULESET_VERSION` 272→**273**（存档不变 v63）。FROZEN 尾追加「热闷」「炙焚」「火刑」（169–171 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_dushiwang_remen_sha_reject_then_armor`（槽空 plain 杀拒+火杀可指定+装八卦后可指定）、`boss_dushiwang_remen_aoe_skip`（南蛮队列 1 号零询问零伤害+2/3 Pass 各-1）、`boss_dushiwang_zhifen_prepare_steal_and_fire`（0 号首张手牌归 1 号+0 号-1 火伤+日志+**他人准备阶段不触发回归**）、`boss_dushiwang_huoxing_death_fire_all`（真死→0/2/3 各-1+往返）——4/4 绿（[日志](dushiwang-targeted.log)） |
| 版本探针 | 273→272 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v273-actuals.txt)），重录依据注释链追加 v273 条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **643/0/0**（639→643，新增 4 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**——[日志](gate1b-all-targets.log)（首跑抓出测试面 `drop(引用)` 1 条——T21 同族 lint，已修后复跑） |

## 已知边界与登记

- 热闷目标不可指定近似（上游为使用后逐目标取消——仅剩其一时上游整卡取消、牌的去向差异登记）；炙焚座位序首个+首张近似；火刑中途死亡跳过（上游预计算列表发完=对已死者不再伤，语义等价域）。
- **坑位**：①炙焚臂漏 `holder == player` 门（上游 player 域触发）——准备阶段技从他回合也会触发，定向回归（他人准备阶段不再 -1）抓出后修；②0 号同回合第二杀撞 ShaLimit——第二杀换座位回合；③测试面 `drop(引用)` 触发 Gate 拒绝（T21 同族）。
- **十殿阎罗 11/13**：楚江王/五官王/宋帝王/卞城王/泰山王/转轮王/孟婆/阎罗王/平等王/都市王✓（+秦广王待查）。余：地藏王（不覆/无量/大愿/谛听，含判定改写机）+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。

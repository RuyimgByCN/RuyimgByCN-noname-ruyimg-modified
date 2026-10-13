# T11 地藏王片（规则 v274，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（都市王批 7300a459 后续片）。上游依据：`boss_dizangwang`（地藏王·金乔觉/male/qun/**hp8**/skills `[boss_bufo, boss_wuliang, boss_dayuan, boss_diting]`，apps/core/mode/boss.js:1406）；`boss_bufo`@4199 / `boss_wuliang`@4226 / `boss_diting`@4333；显示名 boss.js:11331–11337——**显示名勘误**：boss_bufo=不佛（勘验草稿误记「不覆」）。

## 三技语义与原生落法

- **不佛**（查询型双面）：①伤害入口——受到大于 1 点的伤害时改为其中 1 点（上游 damageBegin4 num>1→num-1 同口径；灵渠归零检查**之前**，灵渠读调整后终值；两技分属不同武将不共存，序未登记）；②准备阶段——距离 1 以内的其他存活各受 1 点火焰伤害（**有来源**同上游 `target.damage(1, player, "fire")`；炼狱同款臂内循环 + `computed_distance` 查询）。
- **无量**（查询型双面）：①结束阶段摸 2（End-begin 臂 `draw_cards`）；②准备阶段体力<3 时回复至 3（`ctx.recover`——v234 坑位；双封顶 min(3-hp, maxHp-hp)）。**建局摸 3 面（gameDrawAfter/enterGame）登记未落**（建局面，随装配批）。
- **谛听**（查询型单面）：`computed_distance` 双向链——持有者算到他人 -1、他人算到持有者 +1（义从同位；上游 mod globalFrom/-1+globalTo/+1 同口径）。
- **大愿：登记不落**——global judge 判定改写机（两段 chooseControl 定点数+花色，AI 态度选面），属判定改写扩展面族（天道/鬼道同族），引擎无态度承载；**条目只列已转换技**（宋帝王先例），随判定改写批另立。

条目 `("地藏王", Qun, &[不佛, 无量, 谛听], 8, Male)`（BOSS_GENERALS[52]）；`STANDARD_RULESET_VERSION` 273→**274**（存档不变 v63）。FROZEN 尾追加「不佛」「无量」「谛听」（172–174 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_dizangwang_bufo_damage_cap`（3 点伤→2/1 点伤→1）、`boss_dizangwang_bufo_prepare_range1_fire`（谛听停用隔离——相邻 0/2 号各-1、距离 2 的 3 号不受）、`boss_dizangwang_wuliang_prepare_recover_end_draw`（hp2→准备回复至 3+结束摸 2）、`boss_dizangwang_diting_distance_both_ways`（1→3 可杀[2→1]+3→1 不可杀[2→3]）——4/4 绿（[日志](dizangwang-targeted.log)） |
| 版本探针 | 274→273 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v274-actuals.txt)），重录依据注释链追加 v274 条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **647/0/0**（643→647，新增 4 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 已知边界与登记

- 大愿判定改写机未落（条目不列）；无量建局摸 3 面未落；不佛①与灵渠归零共存序未登记（不同武将不共存）。
- 谛听距离链为查询型只读（义从同位，不含马匹修饰顺序问题——与义从同栈叠加）。
- **十殿阎罗 12/13**：楚江王/五官王/宋帝王/卞城王/泰山王/转轮王/孟婆/阎罗王/平等王/都市王/地藏王✓（+秦广王待查；大愿登记不落）。余：庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。

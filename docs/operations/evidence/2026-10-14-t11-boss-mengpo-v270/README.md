# T11 孟婆片（规则 v270，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 7dd6348d 延续，本日收尾）。上游依据：`boss_mengpo`（孟婆·孟姜女/female/qun/**hp3**/skills `[boss_shiyou, boss_wanghun, boss_wangshi]`，apps/core/mode/boss.js:1396）；`boss_shiyou`@4026 / `boss_wanghun`@4128 / `boss_wangshi`@4056 / `boss_wangshi2`@4073；显示名 boss.js:11320–11324。**显示名勘误两处**：勘验存量误记「拾又」→拾忧、「忘世」→往事（同泰山王批勘误先例）。

## 三技语义与原生落法

- **拾忧**（查询型单面，`DiscardToLimitMachine` 弃牌 commit 前臂）：其他角色弃牌阶段弃置牌后，持有者获得其中**花色互异**的牌——上游 `chooseCardButton` 可选（花色互异过滤）→**锁定化恒发动+按弃牌序贪心取首个未见花色**确定性近似登记；牌自弃牌堆拨入持有者手牌（`CardsGiven` 日志）。
- **往事**（查询型单面）：Prepare-begin 臂（苦酒 jiu_active 同位）——其他角色回合开始置 `PlayerState.wangshi_ban`；PlayCard 拒绝位（`CardLimit`）+ 回合结束全清（反噬 terra 全清同位）。上游 randomGet basic/trick/equip→**恒定基本牌**近似登记（`CardKind::is_basic_kind`=杀族/闪/桃/酒族/手里剑）；上游敌方面（getEnemies）→**全部其他**近似登记。
- **忘魂**（Death 触发遗言机，`MachineSnapshot::BossWanghun` 新变体 tag76/机哈希 76/a0 变体 74，黑绳 Death 前置窗口同位）：你死亡时——①清全场往事封禁（上游敌方 removeSkill boss_wangshi2 同口径）；②座位序前两个其他存活各禁用首个启用技（`SkillInstance.enabled` 直写+`SkillToggled`，隐贼同款；上游 randomGet 两敌将×randomGet 可失技→**确定性近似**登记；skills 为 BTreeMap 按 (座位,def 序) 确定性）。
- **登记未落（随 boss 模式扩展批）**：`_status.shidianyanluo_mengpodie` 模式旗；回魂牌堆注入与回收面（`boss_mengpohuihun(1)`，mode_boss 全局技）。

条目 `("孟婆", Qun, &[拾忧, 忘魂, 往事], 3, Female)`（BOSS_GENERALS[48]）；`STANDARD_RULESET_VERSION` 269→**270**；`ARCHIVE_FORMAT_VERSION` 62→**63**（wangshi_ban 入玩家段 enc/dec + canonical hash）。FROZEN 尾追加「拾忧」「忘魂」「往事」（160–162 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_mengpo_wangshi_basic_ban_turn_clear`（2 号封禁→杀拒/无中可用→回合末全清→封禁态往返）、`boss_mengpo_shiyou_distinct_suit_gain`（hp 压 1 弃 2 张→花色互异子集三面对账[手牌/弃牌堆/日志]）、`boss_mengpo_wanghun_death_clear_and_disable`（真死→清旗+0/2 号碎盾禁用+3 号不受影响+禁用态往返）——3/3 绿（[日志](mengpo-targeted.log)） |
| 字段批探针 | **三值互异**：269 提交时（无字段）golden 2/2 全绿（stash 实测）；269+字段与 270+字段两态 14 场景全差且 actual 互异（[269+字段](probe-b1-v269-plus-field.txt) / [270+字段](probe-b2-v270-plus-field.txt)），版本位移可见、无行为面扰动；按字段批全量重录 |
| 14 golden | 程序化行级重录 14/14，重录依据注释链追加 v270 字段批条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **633/0/0**（630→633，新增 3 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0** |

## 已知边界与登记

- 拾忧锁定化恒发动+贪心花色互异、往事恒定基本牌+全其他（含打出面响应窗未落——仅使用位 PlayCard 校验，封冻同口径族）、忘魂随机敌将/随机技→座位序前两个其他存活各首个启用技（已登记差异族）。
- 忘魂禁用计数=实际禁用满 2 才止（上游随机 pick 可能落空消耗；同属随机→确定性近似域）。
- shidianyanluo_mengpodie 模式旗、回魂牌堆注入/回收（boss_mengpohuihun）未落，随 boss 模式扩展批。
- **十殿阎罗 8/13**：楚江王/五官王/宋帝王/卞城王/泰山王/转轮王/孟婆✓（+秦广王待查）。余：阎罗王/都市王/平等王/地藏王+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。

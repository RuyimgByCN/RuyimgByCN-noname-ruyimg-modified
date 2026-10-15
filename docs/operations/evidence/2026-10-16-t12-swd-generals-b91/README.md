# T12 swd 武将面批 91：姬良/少微/麦星（三将补技）

- 日期：2026-10-16
- 范围：三 partial 补技——**79/79 名不变**（整将 30→32）；新增 HostBehavior tag 389–391
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（jqimou/jianji/toudan）

## 三技语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd姬良 | 极谋 jqimou | SwdJqimou(tag 389)=Damage frequent——回合外受伤后摸 1（上游 damageEnd+currentPhase!=player 门；使用牌续段=主动选择面登记差异不收）——**姬良整将** |
| swd少微 | 剑击 jianji | SwdJianji(tag 390)=ActivateSkill usable:1——SelectCard 弃装备牌（he 双区）→ TargetChoice 射程内 → 虚拟杀+摸 1（上游 useCard sha + draw 同口径；unequip 附加技面登记不收）——**少微整将** |
| swd麦星 | 头弹 toudan | SwdToudan(tag 391)=ActivateSkill usable:1——SelectCard 黑桃 he → TargetChoice 其他存活 → 目标 1 火伤+自受害者起（含自己）座位序各弃一张 he（确定性首张登记差异）——**麦星整将** |

## 验证

- **定向回归** `swd_generals_b91_jqimou_jianji_toudan`：
  - 剑击正例：弃装备→虚拟杀命中 p1（8→7）。
  - 头弹正例：黑桃→p2 受 1 火伤（8→7）+链式弃置全链走完。
  - 极谋正例：回合外受伤（推进至 p2 杀 p1）→摸牌增量断言。
- **全量**：engine-native **803 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **79/79 名不变；整将 30→32**（姬良/少微/麦星整将）。
- 四项用户决策维持待用户裁决清单登记。

# T12 swd 卡牌面补切批⑨：四非珍面——移乱（2026-10-15）

## 承载

**四非珍面 sifeizhenmian**（新 kind tag 155，equip5 宝物栏，spade4 入包）
+ **移乱 yiluan**（tag 303，出牌限一次装备技）：TargetChoice 选其他有手
牌角色 → Act 判定（draw_judge_card）——非红桃则目标**用首张手牌**（真
实牌 CardUsed use_as=kind 指向座位序首个其他存活者）。

登记差异：上游随机牌/随机目标+hasUseTarget 可用性过滤 → 原生首张牌+座
位序首个其他存活者确定性。注册差异：usable:1 走 limits。

## 接线要点（新原语口径）

真实牌自动使用（v177 同口径）：机器侧**不预取**手牌，卡留目标手牌由
CardUsed 提交段自取（take+emit=双取 panic 实证）；子帧结算期间机器再步
进需幂等守卫（cards 非空即完成）。

golden 不变、无版本 bump（行为门控）。

## 测试（swd_cards_equip5_b39_sifeizhenmian_yiluan）

| 面 | 断言 |
|---|---|
| 正例 | 判定非红桃（预控黑顶）→ 目标首张黑杀被用指向座位序首 p0（4→3）、目标 2→1 |
| 反例 | 无其他有手牌角色 → activate 拒 |

测试侧勘误：take+emit 双取 panic（v177 口径）、目标期望修正（座位序首
=p0 而非选人时的 p2）、冻结键数 301（.count 实测驱动）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **48/48** |
| engine-native 全量 | **767/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 67/92 行；equip5 余 4 kind（书锤/散灯金环/昊天塔/玉如意——书锤多
  杀循环/玉如意摸牌改写/散灯金环+昊天塔判定改写为中复杂）+器 13+神草结
  3+神秘果 2+多余 13 处置。
- 批⑩ 候选：器族 jiqi（先勘察通用规则承载）或 equip5 中复杂组。

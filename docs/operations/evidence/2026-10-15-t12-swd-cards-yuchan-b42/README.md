# T12 swd 卡牌面补切批⑫：玉蝉八卦 autoViewAs 面（2026-10-15）

## 承载

8 个新 yuchan kind（tag 165-172，器族手持；包 76→**84 张**）+ autoViewAs
转化面：手持时视作对应基本牌使用（PlayCard viewAs 链玉蝉臂，use_as 自动
声明）——乾=杀/坤=草药/震=酒/巽=桃/坎=神秘果/离=天仙酒/艮=封印之丹/
兑=雪霁冰雹。你的行动时门由 PlayCard 自身回合限制承载。

登记差异（挂批⑬）：换形态随机 mutation（g_yuchan_swap 上游 useCardAfter
随机切换形态——native 实体不可变需实体置换机）+煅造装备面（per-yuchan
equip skills yuchanX_equip1-4）。

## 测试（swd_cards_trick_b42_yuchan_autoviewas）

| 面 | 断言 |
|---|---|
| 玉蝉乾=杀 | 打 p1 4→3 |
| 玉蝉巽=桃 | p0 受伤 2→3 |
| 玉蝉坎=神秘果 | 无直用面同拒 |
| 玉蝉乾反例 | 杀无目标拒 |

发牌敏感回归：灵剑测试首武器实体改全域摘除（第四变体同修）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **50/50** |
| engine-native 全量 | **769/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 84/92 行；余 8 行 = 神草结 3+神秘果 2+器族玉蝉 swap/equip 面+多余
  13 处置（对账）+窗口机批 5 件。
- 批⑬ 候选：神草结 3 或窗口机批。

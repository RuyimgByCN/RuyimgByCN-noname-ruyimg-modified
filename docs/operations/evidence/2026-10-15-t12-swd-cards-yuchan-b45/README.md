# T12 swd 卡牌面补切批⑮：玉蝉换形——swap 面收口（2026-10-15）

## 承载

**玉蝉换形 yuchan_swap**（tag 311，器族手持触发技，InitHook::SwdJiqiHeld
授予列表增项）：自己用牌结算后（CardUsed Last、silent forced），手牌中每
张玉蝉实体轮换为八卦序下一 kind 的实体（乾→坤→震→巽→坎→离→艮→兑→
乾），替换实体经牌堆/弃堆确定性发现首张。

登记差异：上游 randomGet 随机换形+init 原地变形（实体 re-init）→ 原生确
定性下一 kind+实体置换（旧入弃堆守恒、新从发现区移除）。

**守恒缺口修复**：首版置换 hand[i] 后旧实体未入弃堆即弃堆移除 newc——
实体消失由 save/restore 守卫抓漏（xixue 老测试跨批暴露：手持玉蝉者用牌
即触发换形）。修法：to_discard(oldc) 先行。

煅造装备面（g_yuchan_equip+per-yuchan equip skills）：依赖煅造机制，登记
结构性暂缓。

## 测试（swd_cards_yuchan_b45_swap_rotation）

| 面 | 断言 |
|---|---|
| 换形① | 用桃后手持乾轮换为坤（坤实体入手、乾让位） |
| 换形② | 用坤（as 草药）后剩余甘轮换为兑 |
| InitHook | 换形技开局已授予全员 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **53/53** |
| engine-native 全量 | **772/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 92 行数据面全量入包（含 4+2 机批暂缓与多余 13 处置待决）。
- 批⑯ 候选：窗口机批（MachineSnapshot 变体）或转回武将面剩余 58 名。

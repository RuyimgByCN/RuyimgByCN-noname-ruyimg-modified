# T12 swd 卡牌面补切批⑭：玉如意——摸牌窗口内建钩（2026-10-15）

## 承载

**玉如意 yuruyi**（kind tag 164 既有，heart9 已入包）：装备者自己摸牌阶
段从牌堆末 20 张窗口（pop 摸牌位）中取**最大点数**牌移至摸牌位——内建
kernel 钩（commit_phase Draw begin 固定摸牌前直改牌堆序），无新
MachineSnapshot 变体。

登记差异：上游按牌价值加权**随机**取+任意 drawBegin → 原生最大点数确定
性+收敛为阶段摸牌钩。初版走 HostBehavior 触发路线（tag 310）因触发机排
队时序晚于固定摸牌而废弃，改内建钩（shenen 同位）后行为注册整体回撤
（tags_unique/键数 309→307 回降）。

## 测试（swd_cards_equip5_b44_yuruyi_draw_window）

| 面 | 断言 |
|---|---|
| 正例 | 布阵 rank2 压摸牌位+rank13 在窗口内 → p0 下一摸牌阶段 rank13 入手 |
| 驱动 | 须真正离开 p0 回合再回（left_p0 旗，否则循环首查即 break） |

发牌敏感同族坑第五变体：灵剑首武器实体已在发牌手牌——全域摘除（批⑫同
修）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **52/52** |
| engine-native 全量 | **771/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 84/92 行；equip5 家族行为面全部收口（书锤/散灯金环/昊天塔 3 件机批
  暂缓登记）。余 16 行 = 神草结已完成✓、神秘果 2（窗口机批）+器族玉蝉
  swap/装备面+多余 13 处置（对账）。
- 批⑮ 候选：窗口机批（MachineSnapshot 变体）或玉蝉 swap 面。

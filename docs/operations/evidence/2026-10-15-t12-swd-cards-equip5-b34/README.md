# T12 swd 卡牌面补切批④：equip5 三宝物——乾坤袋/神农鼎/女娲石（2026-10-15）

## 承载

3 个新 CardKind（tag 144/145/146，equip5 宝物栏；包 54→**57 张**）：

- **乾坤袋 qiankundai**（club1）：手牌上限 +1 纯 mod（kernel 手牌上限钩
  壶境同位，零机器）。
- **神农鼎 shennongding**（tag 292 club13）：装备技出牌限一次——受伤且
  手牌 ≥2 时弃 2 张手牌回复 1（SelectCard min2+recover；once_per_turn
  走 limits 臂）。女娲石/神农鼎/乾坤袋效果经**装备技实例通道**
  （card_kind_host_skills 注册 defs+card equip_skills 数组桥接授予）。
- **女娲石 nvwashi**（tag 293 heart13）：global Dying 触发确认门——濒死
  者 hp≤0 且自己 hp>1 时目标回复 1、自己失去 1（LoseHp emit）。提取臂
  首版漏挂（第五犯）由回归抓漏。
- **批③缺口修复**：炼妖壶卡条目补 `equip_skills` 数组（原依赖测试手工
  插实例，实战装备不授技——grant_equip_skills 读卡 equip_skills）。
- 无字段/无版本 bump：golden 不变（钩子全持技门控，标准场景不可达）。

## 测试（swd_cards_equip5_b34_qiankundai_shennongding_nvwashi）

| 面 | 断言 |
|---|---|
| 乾坤袋 | 装备后弃牌窗 min = 手-(hp+1)（6 手 hp4 → min1） |
| 神农鼎正例 | 受伤弃 2 回 1（3→4，手牌清 2） |
| 神农鼎反例 | 满血 activate 拒 |
| 女娲石正例 | p2 濒死确认 → p2 0→1 脱离濒死、p0 4→3 |
| 女娲石反例 | 自己 hp=1 不触发 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **43/43** |
| engine-native 全量 | **762/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 57/92 行；equip5 余 9 kind（书锤/散灯金环/东皇钟/昊天塔/玉如意/空
  洞印/昆仑镜/龟灵之桃/四非珍面——书锤多杀循环/玉如意摸牌改写/昊天塔判
  定改写为中复杂）。
- 批⑤ 候选：锦囊族 8 张或 equip5 中复杂三件。

# ADR 0723：N06 距离/传导参照差分与奖惩先后时序断言

日期：2026-09-16
状态：已实施（engine-native）
关联：[0713](0713-native-judge-grouping-version4.md)（判定排序）、[0720](0720-native-judgement-order-equip-qualification-v6.md)、[0722](0722-native-n05-active-judgement-matrix-reshuffle.md)、进度文档 N06

## 背景与决定

进度文档 N06 登记：距离/传导/奖惩先后缺少独立验收——既有 `equip_distance_matrix`
golden 只锁定当前行为，没有独立参照模型；奖惩（杀反贼摸三/主公杀忠臣弃全）与终局判定
的先后只有分立用例，没有「同命令段内先奖惩后终局」的时序断言。

### 1. 距离/传导参照实现差分（`distance_range_reference_differential`）

测试内实现独立参照模型，与引擎全对全比对：

- **座位距离**：存活座位环上顺/逆时针取小（死亡座位不计数）；
- **距离修正**：守方 +1 驪计数、攻方 −1 馬计数，饱和减、下限 1；
- **攻击范围**：装备武器射程最大值，无武器为 1；
- **传导**：`next_alive` 顺时针跳过死座；`seat_order_from` 为存活环自起点旋转。

覆盖矩阵：2 个种子 × 4–8 人身份局 × 四阶段（无装备 → 前 4 人真实 PlayCard 装 +1 馬 →
再装 −1 馬 → 忠臣除内奸制造死座后），每阶段全对全比对。混合装备分布（8 人局仅 4 人有马，
目录每名马仅 4 张）亦被参照覆盖。任何一项不符即失败，不再依赖 golden 自重放。

### 2. 奖惩先后时序断言（`last_rebel_reward_precedes_victory`）

死亡机器的契约：**奖励结算 → 胜负判定**在同一命令段内。回归锁定：

1. 4 人身份局，忠臣先除内奸（无奖惩、不终局——反贼仍存活）；
2. 主公杀最后反贼：`杀反贼摸三` 的三张牌必须与 `LordSide` 终局在同一命令的最终状态入账
   （终局不短路奖励）；手牌数 delta 断言 + 事件顺序断言
   （`PlayerDied` 先于 `CardsDrawn(3)`）。

## 影响

- 测试 134 项（native_kernel 76、boundary_snapshot 53、场景 2、源码 2、调试 1），净增 2。
- 规则版本不变（6）；golden 不变。`equip_distance_matrix` golden 与差分模型并立：
  前者锁行为，后者锁语义契约。
- N06 剩余：同脚本重放/恢复事件对照（逐命令）、多牌稳定顺序扩展、
  旧线等价负载对照（N12）。

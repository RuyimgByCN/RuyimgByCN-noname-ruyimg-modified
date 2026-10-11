# ADR 0781：可变座位环原语（swapSeat）

日期：2026-10-11　|　状态：已采纳
关联：[T19b](../architecture/native-progress.md)、`crates/engine-native/src/state.rs`、[ADR 0780](0780-card-instance-mod-primitive.md)

## 背景

上游国战阵法（zhenfa）依赖可变座位关系：`game.swapSeat(a, b)` 重排座位
环（长蛇阵换座成队列、破阵诀随机重排）。原生 PlayerId=座位索引，座位环
恒等（`next = (p+1) % n`），不可重排——zhenfa 长蛇阵/破阵诀因此 deferred。

## 决策

**GameStateCore 新增显式座位环 `seat_ring: Vec<PlayerId>`**：

1. 建局恒等初始化（0..n）；`swap_seat(a, b)` 交换环位置（PlayerId 身份
   不变——手牌/装备/区域全部随身份走，仅座位顺序变化）。
2. 座位语义全部经由 `ring_pos`/`ring_next`/`ring_prev` 访问器；环长度与
   玩家数不符时退回恒等位（防御）。
3. 下游适配：`next_alive`/`seat_order_from`/`alive_neighbors`/`inline`
   /`inline_with`/`computed_distance`(seat_distance)/kernel Boss 重整巡
   席——全部改走环访问器；`+1 % n` 直算禁止。
4. 存档：`GameStateCore` 尾部追加 `seat_ring`（len+pid 序列，恒等排列亦
   显式编码），存档格式 45→**46**（v45 字节被版本门禁拒绝，无生产迁移数
   据）。

### 备选方案（弃）

1. **交换 PlayerState**（改身份）：手牌/装配/标记全随索引，交换破坏全部
   引用，不可行。
2. **座位映射覆盖表**（仅登记换座差异）：查询面（next/prev/distance）
   处处需要二级查表，侵入面与全环方案相同而无不变量收益。

### 登记差异

- 上游 swapSeat 第四参（是否保留相对方位等 UI 语义）不建模。
- 死亡座位在环中保留（阵法判定经由存活过滤）——与上游 getNext 跳过死亡
  的差异在 T19a 已登记，维持。

## 验收

`g1_shahai_zhenfa.rs::zf_changshe_pozhen_swap_seat`：未入队列施放长蛇阵
→ 与最近友方换座（环变化断言）；换座后队列判定成立；再次施放全队列各摸
1（施放者净 0）；环不变量（重排后仍为恒等排列的谓词：每 id 恰一次）。
存档往返含 seat_ring（v46 全量编码）。

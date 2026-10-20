# T12 运筹武将面批 9：max_hp 减面——蝉缘·变体（于吉整将转正，2026-10-14）

## 任务来源

[T12 运筹武将面](../../../migration/upstream-extension-reconstruction.md)批 8 续批：
三类可立项缺口中体量最小的「max_hp 减面」立项落地。

## 承载

- **内核 max_hp 减面**（machine.rs）：`Ctx::lose_max_hp(player, n)`——上限
  -n（截断）+ 体力钳上限 + `LogEvent::MaxHpLost` 日志。**不加 MaxHpChanged
  事件种**（新事件种九处接线成本，当前无消费者——登记边界，后续消费者出现
  再立项）。max_hp 字段已有（值变化自然入哈希）——无新字段/无存档 bump/
  golden 不变。
- **presentation**：`PresentationEvent::MaxHpLost` + 文本投影（"体力上限 -n"）。
- **蝉缘·变体 YunchouDiychanyuan**（tag 254）：Death 触发（强制）+ 有来源
  filter + 来源 lose_max_hp(1)（上游 `trigger.source.loseMaxHp(true)` 同口径
  ——loseMaxHp 的 hp 钳语义按保守实现：hp>max 收齐）。

## 测试设计要点

死亡通道布阵三坑（回归迭代）：死者手牌桃自救 → 全场清手；濒死窗在无桃时
不开直接死亡 ✓。负例面：无涉者上限不变。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | Diychanyuan filter 桩化 → 批 9 测试 **FAILED**（0/1） |
| 定向 | g1_shahai_yunchou **23/23** |
| engine-native 全量 | **737/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面：**10/10 名、16/20 条目转正、4 暂缓**（六整将：+于吉；曹操
  2/3、于吉 2/2 整将）。
- 剩余暂缓（4）：奸雄·变体（身份改写面）、悲歌（本体实现）、归汉（濒死
  主动技面）、巧思（骰子+多步主动技）。
- max_hp 减面泛化：后续武将/卡牌包的 loseMaxHp 类效果直接承载。

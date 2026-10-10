# ADR 0716：判定/获得事件三件套与 N05/N08 回归补全

- 状态：已接受（2026-09-18）。
- 关联：`crates/engine-native/`（`src/kernel.rs`、`src/machine.rs`）；
  前序 [ADR 0712](0712-native-first-official-skills.md)、
  [ADR 0713](0713-native-judge-grouping-version4.md)、
  [ADR 0714](0714-native-guidao-tiandu-semantics-n04d.md)。

## 独立复核与覆盖更正（2026-09-20）

三个事件类型和直接跳阶段、双持有者都改判、他人天妒负向、成功救活机制测试均已落地。
但仅搜索事件存在未验证次数/全序：seed 233 无改判的一次判定产生两次 Finalized；
seed 207 主动改判加天妒 Confirm 得到 Judged→Rewritten→Gained→Finalized，违背下述目标顺序。
须统一最终判定日志的唯一发出点，并以完整序列断言冻结与规则一致的时点。

成功救活用例是 3 血承受 3 伤害到 0，手持 3 桃只消费 1 桃，剩 2 桃；不是连续三桃救援，且未保存恢复。
原 seed 209 的单桃恢复矩阵仍死亡。日志增量一致不等于日志语义正确，主动/成功救活恢复仍待补。

## 背景

复核指出：鬼道/天妒切片缺少**最终判定与获得事件**——kernel 只在初翻牌时
记录 `Judged`，改判与天妒直接修改手牌/弃牌堆，宿主与行为测试无法断言
「最终判定牌是什么」「何时被获得」。同时 N05 剩余分支（足量桃救活）、
N08 负向对照（他人天妒持有者不越权、双持有者都改判、skip 直接断言）需要
工程固化。

## 决策

### 1. 判定事件三件套（宿主日志事件，不入状态/哈希）

- `JudgementRewritten { player, card, replaced }`：每次鬼道改判——
  `player` 打出 `card` 代替判定牌，被替换的 `replaced` 归改判者所有。
- `JudgementFinalized { player, card }`：改判询问全部结束后、效果生效前，
  `card` 即最终判定牌（无改判者时为初翻牌）。
- `JudgementGained { player, card }`：天妒持有者确认获得，`card` 进入手牌。
- 目标事件顺序约定（当前实现不完全符合，见复核）：`Judged`（初翻）→ `JudgementRewritten`×N →
  `JudgementFinalized` → 效果 → `JudgementGained`（若有）。

### 2. 行为断言补全

- 鬼道改判测试新增事件断言（Rewritten 归属/Finalized 牌面）与
  `skip_play_phase` **直接断言**（Judge 结束 → Draw 后 phase 直达 Discard）。
- 双持有者都改判用例：后改者替换先改者的牌并获得之（`replaced == black2`），
  最终以最后改判牌生效。
- N05 足量桃救活：1 号 3 血 + 乐/闪电，手持三桃、用一桃自救（0→1 即脱离濒死），
  存活、判定段完成、守恒；N02 死亡清队列在 aligned 排序下为防御路径。
- N08 他人天妒负向：天妒在 0 号、判定属 1 号——不出现 `tian_du_gain`
  询问、无 `JudgementGained` 事件（owner_is_target 的反向锁定）。

## 验证（2026-09-18，`cargo test --offline -p engine-native`，共 114 passed / 0 failed）

- 机制测试 64→66：新增足量桃救活、他人天妒负向两项；鬼道/双改判/天妒
  测试补事件与直接效果断言。
- 14 个 golden 不变（新事件仅入宿主日志，不入状态/哈希）；无新增编译警告。

## 当批限制与后续（最新优先级见当前进度）

- `JudgementRewritten` 未含「获得旧牌」独立事件（旧牌归属以手牌状态与
  `replaced` 载荷断言）；如需表现层单独事件再增。
- N09（新线性能基线）为下一个主要批次。

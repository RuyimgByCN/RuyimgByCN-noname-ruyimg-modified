# ADR 0728：N05/N02/N09 小批闭环——矩阵扩面、死亡清理终态与峰值内存

日期：2026-09-16
状态：已实施（engine-native）
关联：[0722](0722-native-n05-active-judgement-matrix-reshuffle.md)、[0726](0726-native-n05-negative-hp-rescue-rounds.md)、[0721](0721-native-runner-judgement-load-exit0.md)、进度文档 N05/N02/N09

## 背景与决定

按进度文档优先级收尾三组小项：无懈×洗牌并入通用矩阵与 Decline 分支矩阵用例（N05）、
死亡清理专用回归（N02）、峰值/存活内存（N09）。

### 1. N05：矩阵扩面（净增 3）

- `pause_roundtrip_nullify_reshuffle_chain`：无懈×洗牌布阵（南蛮多目标 + 持无懈者）
  并入 `assert_pause_roundtrip` 通用矩阵，无懈链逐点往返；真实 DeckReshuffled×恢复
  组合仍由 `savage_multi_nullify_judge_reshuffle_restore` 承担。
- `pause_roundtrip_guidao_pass`：改判询问放弃（Pass）分支逐点往返。
- `pause_roundtrip_tiandu_decline`：天妒询问放弃分支——Confirm 类选择的放弃 =
  **空应答**（`allow_cancel`），矩阵 harness 相应放宽空应答合法性（仅限可放弃选择）；
  覆盖 judge_held→弃置的机器持牌转移路径。

### 2. N02：死亡清理终态回归（`judge_owner_death_settles_zones_and_conservation`）

双闪电判定致死（第一道命中致 -2 濒死、全场无桃死亡；第二道仍在队列）。
关键架构事实：**伤害延迟结算**——判定队列在死亡结算前处理完毕（未命中者按规则
转移下家判定区），循环起点的「死亡清队列」分支在当前架构下不可达（防御性保留）。
回归断言完全结算后的终态：死亡者判定区清空、无悬挂选择、两张闪电各恰存活于
一个位置（弃置或转移）、全量多重集守恒。

### 3. N09：峰值/存活内存

`CountingAlloc` 增加 `LIVE_BYTES`/`PEAK_LIVE_BYTES`（alloc/dealloc 实时净额与峰值），
runner 末尾输出 `peak_live_bytes`/`live_at_exit_bytes`。版本 7 全量复跑
（20 局 seed 9000）退出 0：peak_live=2,620,698、live_at_exit=720,214 字节
（含 runner 自身样本缓冲）。关闭计数器对照仍待做。

## 影响

- 测试 147 项（native_kernel 83、boundary_snapshot 59、场景 2、源码 2、调试 1），净增 4。
- 规则版本不变（7）；golden 不变；runner 聚合与版本 7 前一致。

# 产品级**长程稳定性**切片（game-server soak，2026-10-19）

- 工作包：**NC08 / 产品级长程**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`apps/game-server/tests/long_run_soak.rs`（**已跟踪**）
- 设备/工具链：Apple M3 Max / rustc 1.95.0 / debug profile

## 1 设计与实测

**8 个房间 × 25 轮 × 4 条命令 = 800 条 `EndPhase`**，**每轮每房间**做一次「快照」探活（共 **200 次**）。

```
RESULT soak rooms=8 rounds=25 per_round=4 submitted=800 accepted=27 snapshots=200 max_revision=5
       elapsed_ms=2626 throughput_cmds_per_s=305
```

| 判据 | 结果 |
|---|---|
| 800 条命令全程无 panic / 无死锁 | ✅ |
| **200 次快照全部响应**（每轮每房间）| ✅ |
| 8 个房间**互不干扰**（各自独立修订号）| ✅ |
| 权威修订号推进（`max_revision > 0`）| ✅（=5）|
| 收尾后 8 个房间仍可响应 | ✅ |

## 2 **限制（必须读）**

本用例**只发 `EndPhase`**：对局推进到「**等待选择**」后该命令不再合法 ⇒
**修订号停在小值（≈5）**、`accepted=27/800`。

⇒ 因此本用例证明的是「**长时间多房间运行下服务端不崩、不死锁、房间始终可响应**」，
**不证明**「完整对局可在 server 侧长时间连续推进」。

**已登记待办**：按 `CommandOutcome.view` 的 `legal_actions` **构造 `Answer`/`PlayCard`/`ActivateSkill`**
命令，形成**真正的对局机器人**，才能做「整局长程」压测。

## 3 与八项的关系

| 判据 | 状态 |
|---|---|
| **产品路径（server）长时稳定性** | ✅ 本文件 |
| 真实网络 / 多机 / 浏览器 | ❌ 未做 |
| 完整对局连续推进 | ❌ 未做（需 answer 机器人）|
| 内存/句柄泄漏观测 | ❌ 未做（本轮只测响应性与耗时）|

## 4 复现

```bash
cd noname-next
cargo test --offline -p game-server --test long_run_soak -- --nocapture
```

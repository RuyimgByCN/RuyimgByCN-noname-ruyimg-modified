# NC12 持续负载：短程通过 + **长程崩溃（真实缺陷）**（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）「≥30 分钟持续负载」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**短程 20 万命令通过**；**长程（2000 万命令目标）在 AI 规划模拟中崩溃** → **真实缺陷，已登记**

## 1 短程（20 万命令）通过

`./target/release/examples/nc12_bench long tier=swd seats=4 budget=8 min_cmds=200000 max_games=100000 sample_every=500`
（[long-load-short-200k.log](long-load-short-200k.log)，约 20 秒）：

| 指标 | 实测 |
|---|---|
| `total_cmds` | **200,378** |
| `result` | **OK**（`min_cmds_reached=true`） |
| `plan_action_ns` p50 / p95 | 31,083 / 93,000 |
| `save_cycle_total_ns` | 476,625 |

## 2 长程（目标 ≥30 分钟）**崩溃**

同参数但 `min_cmds=20000000`（[long-load-crash.log](long-load-crash.log)）：

```
thread 'main' panicked at ...: card 93 not in hand of 0
```

- 崩溃发生在 **AI 规划模拟**路径（`plan_action` 的候选探测/模拟 ✓），与本仓库此前修复过的
  「`redact_opponent_hands` 重分配实体牌 id 致挂起机器引用失效」**同类但不同点**（该修复已使
  遮蔽仅在静止态做身份遮蔽 + 非静止态仅打乱牌库顺序）→ **说明仍有未覆盖路径**；
- **未取得 ≥30 分钟连续负载数据**：长程目标**未验收**；
- 崩溃前已完成的 `RESULT` 行数：见日志（用于界定崩溃发生的时间点）。

## 3 待办（NC12 长程与稳定性）

- [ ] **定位并修复 `card 93 not in hand of 0`**：抓取崩溃时的完整回溯（`RUST_BACKTRACE=1`）
      与触发种子/命令序号，判定是「规划模拟的隐藏信息遮蔽」还是「AI 候选探测越界」，
      修后重跑长程
- [ ] 修复后重跑 **≥30 分钟**连续负载（内存/句柄/时延随时间的漂移）
- [ ] 温控与降频影响（需外部测量工具）
- [ ] 其余平台档位、真实网络 RTT、旧线对照

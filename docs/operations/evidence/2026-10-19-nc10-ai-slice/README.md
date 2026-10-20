# NC10 验收切片：**AI 信息边界** + **预算/取消**（2026-10-19）

- 工作包：**NC10（AI 信息边界与有界排程）**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc10_ai_boundary_and_cancel.rs`
- 原始输出：[run.txt](run.txt)

## 对应方案验收（NC10 第 3 段）

| 方案要求 | 本切片 |
|---|---|
| 「同信息集替换暗牌不改变不应受影响的输入/行为」 | ✅ **测输入**：替换**他人暗牌**后，座位 0 的策略输入**逐字节相同** |
| 「取消与过期」 | ✅ **测取消**：预置取消标志 ⇒ `Plan.cancelled=true`、`nodes_used=0`；**过期**（陈旧计划提交被拒）由 `Plan::apply` 的 `Stale` 路径覆盖，见 `nc12_bench` 与 RTT 证据 |
| 「并发压力」 | 部分：`PlanPool` 既有并行路径 + 本会话 190 对/22 包跑批压测；**未做**专项并发尾延迟 |
| 「不同模式/座位/种子/预算的质量及尾延迟」 | ❌ **未做基线**（无质量判据，方案禁止用少数种子胜率代替）|

## 结果

```
DBG NC10 取消：cancelled=true nodes_used=0 budget_exhausted=false
DBG NC10 预算：nodes_used=1 budget_exhausted=false timed_out=false cancelled=false
DBG NC10 信息边界：座位0输入长度 before=1730 after=1730（应相同）
test result: ok. 3 passed; 0 failed; 0 ignored
```

| 判据 | 结果 |
|---|---|
| 替换**他人暗牌**后座位 0 输入不变 | ✅ **before == after**（1730 字节，逐字节相同）|
| 取消标志生效 | ✅ `cancelled=true`、`nodes_used=0` |
| 极小预算 | ✅ `nodes_used=1`（≤ 预算）|

## 说明（**如实**）

1. **信息边界的测法**：比较 `engine.ai_session(seat).input()` 的 `Debug` 形式。该输入是**只读投影**，
   若将来投影里混入他人暗牌身份，**长度或内容会变** ⇒ 本断言即失效（**回归防线**）；
2. **未覆盖**：`SeatPolicy` 的**行为**一致性（本切片只保证**输入**相同；行为一致性要求同一输入 ⇒ 同一输出，
   由策略的**纯函数性**保证，未单独断言）；
3. **取消的语义**是「节点边界检查」⇒ 取消后 `nodes_used=0`（在建第一个节点前就返回）。

---

## 4 ✅ 第二轮：**并发压力与尾延迟**（方案验收第 3 条）

用例 `pool_parallel_matches_sequential_and_records_tail`：同一引擎快照，`PlanPool` 以 **1 / 4 worker**
各跑 **30 次四座规划**，并断言**结果逐一相同**（模块文档承诺「并行 == 顺序」）。

```
DBG NC10 并发：workers=1 n=30 p50=526µs p95=600µs p99=977µs
DBG NC10 并发：workers=4 n=30 p50=315µs p95=369µs p99=389µs
test result: ok. 4 passed; 0 failed; 0 ignored
```

| 判据 | 结果 |
|---|---|
| **并行结果 == 顺序结果**（逐 Plan 比较 seat/action/nodes_used/budget_exhausted） | ✅ 30×2 次全部相同 |
| 四座规划返回 4 个 Plan | ✅ |
| **尾延迟基线**（同机同负载） | `workers=1`: p50 **526 µs** / p95 **600 µs** / p99 **977 µs**；`workers=4`: p50 **315 µs** / p95 **369 µs** / p99 **389 µs** |
| 观察（**不是**优化声明） | 本机本负载下 4 worker 的 p50 约 **1.67×** 于单 worker，**p99 从 977 µs 降到 389 µs**（尾延迟更稳）|

### 4.1 口径（**必须读**）

1. 这是**并发配置对比**（1 vs 4 worker），**不是**新旧实现对比，**不能**当作「优化收益」；
2. 样本 **n=30**、单机、`--release`；**p99 只代表本采样**，样本量不足以支撑严格分位数结论；
3. **未覆盖**：不同**模式/座位/种子/预算**的**质量**基线（方案要求「先建立基线再决定更强搜索」，
   并明确**禁止**用少数种子胜率或旧 `engine-ai` 成绩声称全面达标）。

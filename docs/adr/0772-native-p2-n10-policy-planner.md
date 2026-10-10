# ADR 0772：P2/N10 策略类型隔离、全程计时与有界任务调度

> **2026-09-22 第 13 轮独立复核：本批局部交付保留，全部关闭结论不成立。**
> 当前状态以 [进度主稿](../architecture/native-progress.md) 与 [第 15 轮复现](../operations/evidence/2026-09-23-native-review-15/README.md) 为准；第 13 轮证据为历史。

PolicyInput 类型、同步一层预算搜索、预置取消/时限/陈旧修订回归已落地，尚无工作线程池。
第 13 轮曾复现同可见状态交换对手暗闪导致计划换目标、run_policy/apply_plan 高水位 DuplicateCommand、
respond_shan max=1 漏合法闪。第 15 轮实测真实提交的高水位旧反例已修；隐牌/漏闪仍未修，规划克隆内固定序号在高水位 2,000,000 时把合法候选退化为 NextPhase。apply_plan 的 Ok 可包着 Rejected，验收仍须断言内层结果。
n10_table 没有帧提交；本轮 14207 样本中 12628 次 ShaLimit，计时只是 CPU 策略提交调用（不含 fallback）。N10 仍开放。

## 原批次记录（完成声明按上方复核更正）

日期：2026-09-22（第七十三批）
状态：已完成（P2/N10 关闭）
上游：[0741](0741-native-n10-native-table-latency.md) / [0771](0771-native-p2-n05-n06-composite-acceptance.md)

## 决策

1. **策略类型隔离**：`view::PolicyInput`（视图 + 询问投影 + 修订号 +
   配额余量 + 本座位手牌类别）与 `view::SeatPolicy` trait——策略实现
   类型上只能看到只读投影，`&Engine` 不可达；`Engine::run_policy`
   负责输入组装与主通道提交（修订号取自输入组装时刻）。
2. **计时口径更正**：n10_table 的延迟样本改为「输入组装 → 命令提交
   完成」的全程耗时（5 局 14,207 样本 P50≈5.4µs / P95≈27.8µs），
   取代旧口径「仅 submit 调用耗时」；报告字段注明
   `metric=input-to-frame-submission`。
3. **有界任务调度**（`ai` 模块）：`BoundedPlanner`（节点预算/时限/
   取消标志）+ `Engine::plan_action`——在**克隆引擎**上模拟候选动作
   （计算与规则推进隔离），启发式评分（敌方体力下降 − 己方体力下降）
   选优；`Plan` 携带提交时修订号，`Engine::apply_plan` 对修订号陈旧
   的计划返回 `Stale`，预置取消标志立即返回 `Cancelled`。
4. **候选生成**：多选询问（min≥2）生成单一合法集合（按序取 min），
   单选询问逐选项生成；非法候选（ShaLimit 等）不计分。

## 验收

- `n10_policy_input_isolation`：输入组装不改权威状态；策略驱动推进。
- `n10_planner_isolation_and_budget`：规划零改动权威状态；节点预算
  生效（nodes ≤ budget）。
- `n10_planner_cancel_and_deadline`：预置取消立即生效；0ms 时限立即
  超时。
- `n10_planner_stale_rejection`：规划后修订号前进 → `Stale` 拒绝。
- `n10_planner_baseline_completes_games`：规划器托管 1v1 三局全部
  真实完局。
- 全量 **231 passed / 0 failed**。

## 影响

P2/N10 行关闭。剩余：P2/N09、N12 同负载性能与跨端证据（固定模式/
牌序/动作的真实接受/拒绝计数对照，属独立系统性批次）。

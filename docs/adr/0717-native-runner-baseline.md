# ADR 0717：N09 首批——新线无界面 runner 与部分基准记录（phase19）

- 状态：已接受（2026-09-19）。
- 关联：`crates/engine-native/examples/n09_runner.rs`；
  基线数据与范围声明：[phase19](../operations/phase19-native-baseline.md)。

## 当前运行更正（2026-09-20）

本文保留首批工具意图与当时记录。当前 runner 已追加分配计数和判定负载，但
`cargo run --offline --release -p engine-native --example n09_runner -- 20 9000` 实跑退出 101：
FFA/身份输出后，判定负载首次正常 Pending(gui_dao_rewrite) 被当成异常。

后续还需修正微秒截断后伪装 ns、wall_us 计时范围、aggregate 拒绝原因遗漏、
判定 events 未赋值、分配统计未涵盖判定负载等口径。判定策略首项是 Pass，修掉 panic 后也不等于主动改判负载。
详见 [phase19 的原始记录与验收条件](../operations/phase19-native-baseline.md)。下文旧数据不是当前完整复验结论。

## 决策

1. 新线 runner 以 example 形式落位（`n09_runner.rs`，纯 std 依赖）：
   FFA 4 人攻杀脚本与 5 人身份局纯命令脚本两类负载；每条命令计时，
   输出接受/拒绝（按原因）、事件数、终局、P50/P95/P99 与终态摘要（尚无跨目标对照）
   （每局终态 `state_hash`）。
2. 初始 save 克隆样本测量（并非完整每命令检查点成本）：`Engine::save`（StateSnapshot 全量克隆）
   平均 137–143 ns（release，2000 samples，中局 4 人状态）。
3. 身份局负载设每局 6000 次外层迭代上限（内含多次提交/应答）：被动脚本下的内奸 1v1 合法长局
   不收敛时以 `converged=false` 记录并跳过（非缺陷）。
4. 基线数据与范围声明落位 operations/phase19（版本 5）；不与旧线
   phase17 数字对比（负载不等价）。

## 当批验证记录（2026-09-19，非本轮完整复验）

`cargo run --release -p engine-native --example n09_runner -- 20 9000`：
FFA 20 局全部终局；身份局 18/20 收敛、2 局长局记录；112 项测试全绿。

## 限制与后续

后续已有判定负载和累计分配计数，但负载失败、统计范围与精度仍待修复；原始输出本轮已保留。
完整性能基线、峰值内存/设备信息、跨目标对照与宿主接入归 N09 剩余/N11。

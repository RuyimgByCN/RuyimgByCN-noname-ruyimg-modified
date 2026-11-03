# perf-ubuntu start-hash 失败的决定性 A/B：Linux amd64 容器（2026-11-03）

条件与 CI 完全对齐：`docker run --platform linux/amd64 rust:1-slim`（glibc、**rustc 1.99.1（58cf8f9c3 2026-10-13）＝ CI 两次失败 run 的同一编译器**）；源码只读挂载、宿主 registry 只读缓存、私有 target、`--offline --locked --release`、LTO。

| 树 | benchmark-runner start-hash p50（3 次） |
| --- | --- |
| HEAD `a33e96d8`（engine-core 与已发布 `96236c96` 相同） | 66.5／66.1／66.3 µs |
| baseline `9f4b7423`（CI 绿 run 的提交） | 65.1／65.0／64.8 µs |

**差异 +2%（+1.3µs），持平。** 按预登记判据（同环境 ≥30% 劣化才判代码回归）：CI perf-ubuntu 的 111／142µs（历史同 job 48–78µs）判定为**宿主噪声／runner 池漂移，非代码回归**。

## 四环境无回归矩阵（同一代码对比 9f4→96236c96）

| 环境 | baseline | 新代码 | 结论 |
| --- | --- | --- | --- |
| 本机 arm64 release | ≈50.2µs | ≈49.3µs | 持平 |
| 本机 x86_64（Rosetta） | ≈86.4µs | ≈85.5µs | 持平 |
| **Linux amd64 容器（glibc＋rustc 1.99.1＝CI）** | ≈65.0µs | ≈66.3µs | +2%，持平 |
| CI macOS | 70µs | 74µs | +6%，预算内 |
| CI Ubuntu（两次 attempt） | 48µs（历史 48–78） | 111／142µs | **唯一离群** |

原始输出：[container.log](container.log)、[run.sh](run.sh)。容器内绝对值（≈65µs）介于 CI ubuntu（48–78）与本机 Rosetta（≈85）之间，量级合理。

## 收口

失败按原样登记（不删 attempt1/2）；据此把 CI 预算步改为**最多 3 次尝试、全部日志保留在步骤输出**的测量稳健性方案（延迟下界型指标取「任一次通过」等价于 min-of-N 中位数；真实回归仍会三次全红），见同批 ci.yml 提交与进度主稿条目。

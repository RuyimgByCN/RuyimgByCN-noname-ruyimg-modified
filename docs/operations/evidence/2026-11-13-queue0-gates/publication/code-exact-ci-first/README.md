# 第0项代码精确SHA的CI核验

记录：2026-11-13T23:03:20+08:00；代码 `1d59f42b8b493e1fbac7a0e8b8c781ccef1ea61c`；[run 43442212867](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43442212867)／attempt 1／success。全部10作业成功；不是历史99d的CI。完整[结果](result.json)、[jobs](jobs.json)、[原始日志包](complete-job-logs.zip)和[逐作业计数](job-log-summary.json)。

| 日志聚合文件 | rustc实报版本 | Rust通过／失败／忽略 | Node通过／失败／跳过 |
| --- | --- | --- | --- |
| 0_gates-ubuntu.txt | 1.100.0 | 7838／0／8 | 17／0／0 |
| 1_build-ubuntu-latest.txt | 1.100.0 | 872／0／0 | 无该类测试输出 |
| 2_build-windows-latest.txt | 1.100.0 | 872／0／0 | 无该类测试输出 |
| 3_desktop-windows-latest.txt | 1.100.0 | 无该类测试输出 | 无该类测试输出 |
| 4_perf-macos-latest.txt | 1.100.0 | 无该类测试输出 | 无该类测试输出 |
| 5_desktop-macos-latest.txt | 1.100.0 | 无该类测试输出 | 无该类测试输出 |
| 6_build-macos-latest.txt | 1.100.0 | 872／0／0 | 无该类测试输出 |
| 7_native-web.txt | 1.100.0 | 2579／0／6 | 489／0／0 |
| 8_desktop-ubuntu-latest.txt | 1.100.0 | 无该类测试输出 | 无该类测试输出 |
| 9_perf-ubuntu-latest.txt | 1.100.0 | 无该类测试输出 | 无该类测试输出 |

只从zip根目录的每作业完整聚合日志取计数，不重复叠加目录内分步日志；没有Rust／Node输出的安装器和性能作业不虚报测试数。平台作业成功及安装器artifact不能代替移动真机、签名／公证、Spine用户资源与37来源整包八项签署。

4_perf-macos-latest.txt 性能预算逐次退出码：attempt 1 exit 1, attempt 2 exit 0。原始首失败与后续内部重试均保留在日志包。
9_perf-ubuntu-latest.txt 性能预算逐次退出码：attempt 1 exit 0。原始首失败与后续内部重试均保留在日志包。
CI三平台build作业执行workspace构建与库测试；完整workspace和native-web在Linux执行。本机macOS的无过滤完整命令见本地门禁表；没有据此声称Windows完整集成测试或移动真机通过。
工具链分别记录：此次CI stable实报1.100.0；更新日志中的from 1.99.1是原预装版本。本机新增测试及严格all-targets使用1.99.1，WASM本地补跑用已装目标的1.95.0；没有修改持久工具链或CI配置。

## 2026-11-13T23:04:40+08:00 性能首试与内置补跑的精确记录

macOS的 `start-hash-p50` 首试164µs／预算100µs，exit1；第二次69µs／100µs，exit0。其他三项预算均通过。Ubuntu只有attempt1通过，没有失败或重试。两次macOS运行属于同一代码及同一job的既有重试机制，不能据此推断性能改进；原始输出在zip的 `4_perf-macos-latest.txt`，结构化[首次失败记录](performance-first-failure.json)单列。没有修改预算阈值、CI策略或代码。

# Main alias9 最终四门禁与窄身份审计

核验日期：2026-10-30。HEAD `da7e4c019461a6c467a9dd2de849f3b1a7494b86`。四项门禁均 exit 0；完整 preflight 21/21，脚本单测 192/192、跳过 0。仅本证据目录新增文件，父独立 review / commit 阶段仍待执行，不表示整 I02、UP37、37 来源或 Goal 完成。

| 维护入口 | 结果 | wall 秒 | locked 秒 | queue + launch overhead 秒 |
|---|---|---:|---:|---:|
| `python3 -B -m unittest discover -s scripts -p test_check_source_lines.py` | exit0；2 tests | 0.092640 | 0.066634 | 0.026006 |
| `python3 -B scripts/check-source-lines.py` | exit0；8992 files | 8.017223 | 7.989056 | 0.028167 |
| `python3 -B scripts/check_doc_line_refs.py --strict` | exit0；1065 refs；越界0 | 8.208791 | 8.178140 | 0.030651 |
| `python3 -B scripts/preflight.py` | exit0；21/21 | 327.421834 | 327.392302 | 0.029532 |

纯 queue 未独立测量，JSON 为 null；表中差值含启动开销，不能解释为精确排队时间。四次命令均通过 `/tmp/noname-run-serialized.py` 和全局 `/tmp/noname-phase4-heavy-command.lock`。没有 selected-gate 补跑或失败重试。[命令索引](command-index.json)与[主摘要](summary.json)保留精确 argv、时间、退出码和磁盘记录。

[入口](narrow-entry.json)、[出口](narrow-exit.json)、[比较](narrow-comparison.json)确认 source9、CI4、按 admission 及已批准 CI 三项覆盖后的 A53、18 个 WASM 模块/公共镜像文件、37 个 tracked dist 文件、guard 与父四份文档逐项不变。A53 的原 admission 中三项已经由 CI13 正式覆盖，未把旧三项哈希误充为现状。source9 聚合 SHA 为 `82fca1f0e76e48df0f557f095bacb1d8c6ce5280eb7beef12431418fd32de978`，与父功能交接相同。HEAD、空 index 及 index 字节均不变；[其余工作区状态](unrelated-status-comparison.json)287 条逐项相同。[版本声明](version-pins.json)实读为 330 / 105 / ABI758 / WS6 / asset schema2。

[preflight 原始入口输出](04-preflight.log)保留所有 21 项状态。入口原实现只打印子项摘要，本轮通过局部 `PYTHONPATH` 的 `sitecustomize.py` 记录其原始 subprocess 结果，不改 argv、stdout/stderr、return code 或源码。21 份 stdout 与 stderr 由 [子项索引](preflight-children.jsonl)定位。两档本地链接仅在 preflight 内执行：7 / 1910 文件，坏链0；未重复运行链接扫描。

[警告、失败与忽略清单](all-warnings-failures-ignored.json)保留脚本单测 `report_phase_arms.py` 第117行的98次 unclosed-file ResourceWarning、1条卡面覆盖提示、docrefs 六类缺失路径提示；越界0、失败0、跳过0。未过滤警告或宣称 warning 全清。[原 cleanup exit1](../main-acceptance/initial-cleanup-red.json)及其原日志继续原样保留，没有重跑功能验收来覆盖历史失败。

[stage3 独占使用记录](exclusive-stage3-lease.json)记录了先前租约已释放、lsof 无占用后才使用指定缓存。CARGO_INCREMENTAL=0 / jobs2；[唯一 Cargo 调用](cargo-invocations.jsonl)为 offline/no-deps metadata，不含编译。调用前空闲101347598336字节；[incremental 入口](incremental-before.json)和[出口](incremental-after.json)完全相同。本轮没有自有临时构建产品需要删除，缓存及 dependencies 保留给父下一次明确授权的验收。

没有重跑 main11Node / 167frontend / typecheck / sync / verify / 8824 payload / Vue121decode14play / 214GET9errors，也没有运行 Workspace、native3、conformance、WASM构建、codec、性能、17195/461/12359媒体hash或 private331/compat14/NC10 工作。没有源码、docs 正文、index、stage、commit、push、remote、settings、defaultdist 或 shareddeps 修改。最终边界仍需父独立审阅；已知原五份 Chrome 音频失败及其他来源未完范围不因本轮门禁全绿而关闭。

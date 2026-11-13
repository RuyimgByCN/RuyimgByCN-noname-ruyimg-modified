# 2026-11-14T02:46:31.509449+08:00 酒诗恢复修复精确代码SHA CI

代码 `5c73d1d9158145e33f98f2018013914667d13964` 已按裁决25正常发布到native-convergence；记录时源码tracked clean，版本333／NGSA107／schema5／ABI759／WS8。[提交](../code-commit-first.json)、[原始推送与10项守卫自检](../code-batch-first.json)。

[同一run43485811587](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43485811587)，push事件、attempt1，首次观测状态为queued／None，初始作业数10；[查询命令](initial-status.json)、[原始运行](run-initial.json)、[原始作业](jobs-initial.json)。未重启/补跑，不预称成功；终态另增记录。

本地完整三crate2447／0／6、workspace7843／0／8及其余门禁/最终preflight21项通过，[本地实际命令](../../restore-fix/final-local-gates.md)。清单628／3796／0、本批撤回6旧等价、新签0、八项0/37，队列1仍部分，不开始队列2。界面/Spine/设备签名/第六阶段仍待。

## 2026-11-14T03:15:03.245179+08:00 首次CI终态通过

记录时HEAD `5c73d1d9158145e33f98f2018013914667d13964` + 文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。同一run43485811587的push事件attempt1为10成功／0失败／0取消，agent补跑0。两平台性能脚本的内部第一次也均exit0。

| 作业 | Rust通过/失败/忽略/过滤 | 实际rustc | 原有性能内部尝试 |
| --- | --- | --- | --- |
| build-macos-latest | 872/0/0/0 | 1.100.0 | 不适用 |
| native-web | 2584/0/6/0 | 1.100.0 | 不适用 |
| gates-ubuntu | 7843/0/8/0 | 1.100.0 | 不适用 |
| desktop-ubuntu-latest | 无Rust test汇总 | 1.100.0 | 不适用 |
| desktop-windows-latest | 无Rust test汇总 | 1.100.0 | 不适用 |
| perf-macos-latest | 无Rust test汇总 | 1.100.0 | 第1次exit0 |
| build-windows-latest | 872/0/0/0 | 1.100.0 | 不适用 |
| desktop-macos-latest | 无Rust test汇总 | 1.100.0 | 不适用 |
| build-ubuntu-latest | 872/0/0/0 | 1.100.0 | 不适用 |
| perf-ubuntu-latest | 无Rust test汇总 | 1.100.0 | 第1次exit0 |

作业间测试重复，不能累加为独立产品覆盖；build矩阵的--lib不替代gates-ubuntu的完整workspace7843／0／8／0。新增酒诗回归在native-web与gates-ubuntu日志均实际通过。CI stable为1.100.0；本机1.99.1严格all-targets及1.95.0 WASM分开记录。忽略项与安装包上传不代表真机、签名、公证、安装或整包规则验收。

[完整run](run-final.json)、[逐job元数据](jobs-final.json)、[实际API命令与逐job汇总](terminal-summary.json)、[watch命令/退出码](watch-first.json)和[原始watch日志](watch-first.log)保留。原始[日志ZIP](logs-first.zip)为1865393B／SHA256 `29739c480bd44168b4acf029b4d405d2d1661c4415112f8701dcda2b6fb10b5d`，148份解包文本/10131765B与ZIP逐字节核符且CRC通过，见[字节校验](raw-log-integrity.json)。Git index的现有CRLF归一另记，不重写工作区或ZIP。[追加记录前的格式守卫失败](terminal-format-first-failure.json)单列，不冒充CI或产品失败。

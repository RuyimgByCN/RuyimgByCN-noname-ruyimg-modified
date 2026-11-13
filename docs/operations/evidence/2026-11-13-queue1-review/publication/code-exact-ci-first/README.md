# 队列1部分增量精确代码CI（2026-11-14）

## 2026-11-14T01:02:23+08:00 发布和首次观察

- 代码HEAD `1fa8ac6cc3a70446436d97ff49aa19ea4626e22d`；本地代码提交后tracked clean，记录后只有文档/证据dirty。规则333／NGSA107／schema5／ABI759／WS8，清单634／3790／0，新增等价签署0，八项0/37。
- 按裁决25只对明确URL/ref正常push；[精确守卫及10项自检/实际结果](../code-batch-first.json)证明远端由bd58ef06d前进到本SHA，原pre-push和Git持久配置不变、临时守卫已删除；[代码提交树](../code-commit.json)与本地验证树一致。
- [CI run43468568765](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43468568765)，push事件、branch native-convergence、head_sha精确匹配；attempt1，初始10作业进行中，见[发现原始响应](discovery-first.json)、[运行](run-initial.json)、[作业](jobs-initial.json)。
- 只用实际 `gh run watch 43468568765 --repo RuyimgByCN/RuyimgByCN-noname-ruyimg-modified --exit-status --interval 30` 观察同一run；不派发、不重启、不补跑。终态、完整日志及工具链将另增记录。

[本地全部必跑门禁与边界](../../final-local-gates.md)、[指定上游投影](../../runtime-comparison.md)、[37来源634／3790／0](../../source-counts-reviewed.md)；队列1仍部分，不将CI当作规则等价或产品/平台验收。


## 2026-11-14T01:33:42+08:00 同一精确代码CI终态

代码 `1fa8ac6cc3a70446436d97ff49aa19ea4626e22d` 已发布；[run43468568765](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43468568765) attempt1全部10作业success／0 failure／0 cancelled。没有由agent重启或补跑。记录时HEAD为该代码加文档/证据dirty，版本333／107／5／759／WS8；清单634／3790／0、新增等价0、八项0/37。

| 作业 | 结果 | Rust通过/失败/忽略/过滤 | 实际日志中的工具链 |
| --- | --- | --- | --- |
| perf-ubuntu-latest | success | 无Rust测试计数（构建/预算/打包范围） | 1.100.0 |
| perf-macos-latest | success | 无Rust测试计数（构建/预算/打包范围） | 1.100.0 |
| desktop-ubuntu-latest | success | 无Rust测试计数（构建/预算/打包范围） | 1.100.0 |
| desktop-macos-latest | success | 无Rust测试计数（构建/预算/打包范围） | 1.100.0 |
| build-windows-latest | success | 872/0/0/0 | 1.100.0 |
| native-web | success | 2583/0/6/0 | 1.100.0 |
| build-ubuntu-latest | success | 872/0/0/0 | 1.100.0 |
| build-macos-latest | success | 872/0/0/0 | 1.100.0 |
| gates-ubuntu | success | 7842/0/8/0 | 1.100.0 |
| desktop-windows-latest | success | 无Rust测试计数（构建/预算/打包范围） | 1.100.0 |

各作业计数有重叠，不能相加为唯一测试数。build三平台矩阵为workspace `--lib`，完整集成门禁在Linux；native-web中四项新增聚贤回归通过。CI实际stable为1.100.0，本地严格检查1.99.1、WASM1.95.0另列[工具链](../../toolchain-observation.json)，不称同一工具链。

性能步骤原有最多3次内部尝试，按原工作流执行；所有实际尝试保留：`[["perf-ubuntu-latest", [{"attempt": 1, "exit": 0}]], ["perf-macos-latest", [{"attempt": 1, "exit": 0}]]]`。这与workflow run_attempt或agent补跑分开记录，不把内部首失败隐藏。

[终态原始run](run-final.json)、[全部作业/步骤](jobs-final.json)、[安装器artifact元数据](artifacts-final.json)、[全部日志ZIP](logs-first.zip)、[逐作业解析/命令/指纹](terminal-summary.json)。三个安装器上传只证明构建和上传，不代表本机安装、签名/公证、真机或UI验收。

CLI首次读取单作业日志因控制转义输出保护而exit1，保留[job-log-download-first-failure.json](job-log-download-first-failure.json)；加`--allow-escape-sequences`后仅将原始字节写文件，不向终端执行。这不是CI作业失败或重新运行。

队列1仍部分，实际5技能13分支24次仅为指定投影；剩余10来源30候选、L22自动规则及其他队列不据绿色CI关闭。代码CI结果与发布证据将形成纯文档后续提交；当前记录时该文档提交尚未发生。


## 2026-11-14T01:38:57+08:00 暂存日志字节口径更正

Git依现有配置将26份Windows日志文本视图的CRLF归一化为LF；此前暂存审计中 `raw_ci_logs_preserved_byte_for_byte=true` 不适用于全部Git文本blob，撤回该解释。原始ZIP的SHA256 `83295d5edb78f2e7c3c1b4040a4fbf59af6d48b50ad5d1815b206a55efe8ef48` 与暂存blob一致，148个ZIP条目与本地下载原始文件逐字节一致；文本视图只有换行归一化，无其他字节变化。见[逐文件指纹与更正](../raw-log-normalization-correction.json)。未修改持久Git配置或产品输入；此前记录保留。

# NC03／NC09：公开事件粗粒度文案

- 实际执行：2026-10-28 10:17:51 +0800 → 2026-10-28 10:23:47 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`d769f216e0b7c4919fbe5d917a8f35c6c619152a`](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 不变。

## 反例与修复

原生公开事件码是粗粒度信息：[投影](../../../../crates/engine-native/src/view.rs)把伤害造成和伤害防止均映射为 4，把进入濒死和救回映射为 6，把选择打开和回答映射为 10；用牌／响应、阶段变化／回合结束也共用各自的码。先前本地 Worker 与联机适配都把这些码分别显示为「造成伤害」「进入濒死」「等待选择」等，会误报实际事件。

`git archive HEAD` 干净导出只叠加两条前端反例并借用已安装的依赖及生成 WASM：定向 **0／2／0、exit 1**（[日志](red-clean.log)）；再叠加修复后 **2／0／0、exit 0**（[日志](targeted-clean.log)）。两条路径现共用一份标签表，对合并含义使用「伤害结算」「濒死状态变化」「选择状态变化」等中性文案；其他已有码及未知码回退维持覆盖。事件码、Worker／WS 消息结构、权威状态和规则没有变化。

真实 headless Chrome 使用本批最终前端构建和重新构建的 web WASM，启动真实 Worker 并点击自动一步，实际码 8 在战斗日志显示「阶段或回合推进」，浏览器异常 **0**（[驱动](browser-events.mjs)、[日志](browser-final.log)）。事件码 4／6／10 的客户端断言使用适配层桩，联机覆盖使用模拟 socket；本批没有构造真实联机防伤／救回／选择完成的浏览器场景，也不代表完整产品验收。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 最终前端 **91／0／0**，typecheck 通过，均 exit 0（[测试](frontend-test-final.log)、[类型](typecheck-final.log)）。 |
| `CARGO_NET_OFFLINE=true pnpm --dir apps/web-client build:native`；`pnpm --dir apps/web-client exec vite build --outDir <临时目录>` | 当前内核源码重新构建 web WASM 与最终前端产物，均 exit 0（[WASM 构建](build-native.log)、[产物 SHA-256](wasm-sha256.txt)、[前端构建](frontend-build-final.log)、[临时目录](dist-directory-final.txt)）；wasm-pack 预构建下载不可用后使用本地工具链完成。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/noname-nc10-trap-red-target.rbskPH cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1610／0／73**、WASM **63／0／0**、加载器 **32／0／0**；合计 **1705／0／73**、357 个测试目标，exit 0（[日志](rust-full.log)、[统计](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。编译有既存未使用警告。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 全部通过，结果见[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)及[暂存](staged-check.log)。 |

NC03 和 NC09 仍为部分：事件细节、真实联机对应场景、UI 全流程与平台验收未完成。用户原有八份文档改动保留，仅暂存本批追加行。

首次暂存检查发现失败日志中四行空白含尾随空格；仅规范化这些空白后复查，测试结论不变。

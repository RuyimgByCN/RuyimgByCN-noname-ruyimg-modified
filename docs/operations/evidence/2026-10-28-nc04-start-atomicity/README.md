# NC04：失败建局保留当前对局

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)；源码基准 [`f7d9fa9f468eba3f485fb8ed536f5f1370866cd0`](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 不变。

## 反例与修复

Worker 原来先把新引擎赋给当前句柄，再生成首个视图：若视图生成失败，回包虽为 `ok:false`，旧局却已丢失。例如四人局的观察座位为 4 时，`view_json` 拒绝越界座位，但随后保存得到的是另一个 seed 的新局。标准局、身份局和有包／无包分支共用此路径。

以 `git archive HEAD apps/web-client` 导出干净源码，只叠加[回归测试](../../../../apps/web-client/scripts/nc04-start-atomicity.test.mjs)，借用本机依赖和由该 HEAD 原生源码构建的 web WASM；Node 模拟 Worker 入口使用真实 WASM，**0／2／0，exit 1**（[日志](red-clean.log)）。上一批对应同一 Worker 源码构建的生产前端在真实 Chrome 中亦复现旧存档被替换（[日志](browser-red.log)）。

现先准备候选引擎并生成完整信封，成功后才替换旧句柄；信封失败时立即释放候选引擎并保留原异常。定向测试 **2／0／0**（[日志](green-targeted.log)），包括四类越界观察者建局、注入权威动作投影异常、候选资源释放、失败后旧局可观察／保存及合法新局替换。真实 Chrome 使用本批生产构建：上述四类失败建局的旧存档逐字节不变、旧视图可读，合法新局替换，浏览器／Worker 异常 0，**4 项通过**（[驱动](browser-worker.mjs)、[日志](browser-green.log)）。消息结构及合法规则行为不变。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端 **95／0／0**，类型检查通过，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)）。 |
| `pnpm --dir apps/web-client exec vite build --outDir /tmp/nc04-start-dist.GKhsPv` | 生产前端构建通过（[日志](frontend-build.log)）；复用上一批重建且本批原生源码未改的 web WASM（[本批摘要](wasm-sha256.txt)、[上一批构建](../2026-10-28-nc03-wasm-actor-bounds/build-native.log)）。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。复用上一批独立 target 及其可执行文件映射；本批没有修改 Rust 源码。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 均通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。日志行尾空白已规范化，不改变测试结论。 |

NC04 仍为部分：其他故障矩阵、恢复路径和 B7 挂起恢复缺陷尚未闭合。本批只处理本地 Worker 建局，未覆盖联机建局。原有八份文档修改保留，主稿仅暂存本批追加行。

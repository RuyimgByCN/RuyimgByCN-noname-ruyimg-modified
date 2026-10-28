# NC04：本地 Worker 故障与关闭时结束等待

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)；源码基准 [7a91ad5ce2eff60a2e917b1bfffc0de382c2a335](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 和消息结构不变。

## 反例与修复

本地 Worker 报错时原实现只记录控制台，关闭时直接清空等待表：正在执行的请求没有成功也没有失败，界面保持忙碌。初始化被关闭后立即重连时，旧初始化的清理还可能终止新 Worker。

[回归测试](../../../../apps/web-client/scripts/nc04-worker-failure.test.mjs)使用生产 EngineClient 和可控 Worker。干净 HEAD 导出前端并叠加测试后 **1／5／0**，5 项超时、1 项通过（[日志](red-clean.log)，exit 1）。修复后连同此前结构化克隆测试 **10／0／0**（[日志](green-targeted.log)，exit 0）。覆盖错误和不可解码回包时拒绝所有等待请求、保留句柄、迟到回包、关闭后重连、初始化重连竞争与空闲实例不干扰共享等待。失败请求的性能采样被删除；对用户提示结果未知，不自动重放。故障时保留 Worker，让仍可用的实例能核对和处理后续请求；显式关闭仍会终止。

[Chrome 实验](browser-worker-fault.mjs)在生产页面中拦截并丢弃一次尚未发送给真实原生 Worker 的请求，再注入错误／解码错误事件。前一批构建作基准 **0／4**（[日志](browser-red.log)），本批构建 **4／0**、意外异常 0（[日志](browser-green.log)）：两类事件都显示未知结果、解除忙碌；由于实验明确丢弃了原请求，人工重试可安全观察到真实 Worker 修订号 0→1。此实验未模拟已经提交后回包丢失或 Worker 进程真正崩溃，不能推断这些情况下重试安全。

首次反例脚本没有超时，执行约 19 秒后被中止（[日志](red-harness-unbounded.log)）；加 500 ms 限时后获得上述干净基准。第一次浏览器试用 `onmessageerror` 时 **2／2**（[日志](browser-harness-initial.log)），诊断显示浏览器注入事件被普通监听器观察到但属性处理器未触发（[日志](browser-debug.log)）；改用 `addEventListener` 后重测通过。这里的事件是注入的，未自然触发真实反序列化错误。

暂存差异首检发现两份测试日志末尾有空行（[日志](staged-check-initial.log)），去除末尾空行后复检通过，不改变测试结果。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `node --import ./scripts/ts-register.mjs --test scripts/nc04-worker-failure.test.mjs scripts/nc03-worker-clone.test.mjs`（在前端目录） | **10／0／0**，exit 0（[日志](green-targeted.log)）；单独干净基准 **1／5／0**，exit 1。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck`；`pnpm --dir apps/web-client smoke:native` | 前端 **114／0／0**、类型检查通过、真实 WASM Worker 冒烟 **62／0**，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)、[冒烟](native-smoke.log)）。 |
| `pnpm --dir apps/web-client exec vite build --outDir <独立目录>`；`node browser-worker-fault.mjs <dist> <Chrome>` | 前端构建通过（[日志](frontend-build.log)、[目录](dist-dir.txt)）；浏览器注入实验基准 **0／4**、本批 **4／0**，详见上文。Rust 未改，复用 web WASM（[摘要](wasm-sha256.txt)、[构建](../2026-10-28-nc03-wasm-actor-bounds/build-native.log)）。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。编译含既有未使用代码警告。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 均通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。日志行尾空白规范化，不改变结果。 |

NC04 保持部分：界面目前没有自动核对权威局面或请求超时，静默失效仍可能等待；真实崩溃／提交后回包丢失、联机和 B7 挂起存档恢复仍待验证或修复。发生结果未知时不能假定命令未执行。原有八份文档修改保留，主稿仅暂存本批追加行。

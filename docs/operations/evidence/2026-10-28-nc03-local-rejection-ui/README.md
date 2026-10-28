# NC03／NC04：本地原生命令拒绝的界面反馈

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)；源码基准 [f03d011594fb695d2b9090cd91e480f25a8ba2dd](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 与 Worker 消息结构不变。

## 反例和修复

原生窄壳以成功传输的信封携带 `result.outcome = rejected` 及原因；战斗页此前只显示修订号，出牌／技能仍写成功日志。自动步进把同一结果嵌在 JSON 字符串内；计划失效、取消或拒绝时，连续自动也未停止。

[浏览器驱动](browser-rejections.mjs)使用生产前端、真实 Chrome 和原生 WASM Worker：[基准](browser-red-expanded.log) **0／3**（exit 1），[修复后](browser-green-expanded.log) **3／0**（exit 0），控制台异常 0。结束阶段用另一有效座位发命令，由真实引擎返回 `NotYourTurn`，断言状态摘要和修订号未改变。出牌测试为绕过尚缺的手牌投影，向测试视图注入一张虚构牌与动作，并在夹具中将 Vue 响应式目标数组转换为普通数组；真实引擎返回 `WrongPhase`，修复前错误写入“出牌”成功日志、修复后显示拒绝而无该日志。连续自动的嵌套 `InvalidChoice` 回包是**故障注入**，修复前又执行 10 步，修复后没有继续执行；不把注入当成引擎真实拒绝。

界面先应用信封中的权威视图，再统一识别本地结束阶段、出牌、技能、选择和自动步进的结果：拒绝显示原因并跳过成功日志；自动错误停止连续运行；挂起及畸形结果不再冒充成功，挂起暂无法从本地界面续行。常见原因使用中文，未知原因保留原文。联机客户端原有拒绝处理未改。干净 HEAD 的前端源码叠加新测试 **3／2／0**（[日志](unit-red-clean.log)，缺少新处理函数）；本批定向 **5／0／0**（[日志](unit-green.log)）。

首次出牌浏览器验证未到引擎：Vue 响应式目标数组令 `postMessage` 报 `DataCloneError`，导致测试等待回包超时（[初次](browser-play-initial-green.log)、[诊断](browser-play-debug.log)）；这是真实独立缺陷，不视为本批已修。基准前端同样超时（[日志](browser-play-initial-red.log)），本批完成后另作修复。未验证联机错误提示的异步覆盖、真实挂起续行或完整产品场景。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck`；`pnpm --dir apps/web-client smoke:native` | 前端 **101／0／0**、类型检查通过、真实 WASM Worker 冒烟 **62／0**，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)、[冒烟](native-smoke.log)）。 |
| `pnpm --dir apps/web-client exec vite build --outDir /tmp/nc03-rejection-dist.l2SDMF` | 生产前端构建通过（[日志](frontend-build.log)）；Rust 源码未改，复用前批生成的 web WASM（[摘要](wasm-sha256.txt)、[构建](../2026-10-28-nc03-wasm-actor-bounds/build-native.log)）。 |
| `node browser-rejections.mjs <dist> <Chrome>` | 基准 0／3、修复后 3／0，详见上文。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。复用前批独立 target 与映射。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 均通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。日志行尾空白规范化，不改变结果。 |

NC03／NC04 均仍为部分；局面显示、联机链路、暂停续行和 B7 等验收尚未闭合。原有八份文档修改保留，主稿仅暂存本批追加行。

首次本地链接检查时暂存检查日志尚未创建，报本证据目录内 1 个坏链（[日志](local-links-initial.log)）；创建该日志后重跑通过。首次暂存检查指出两份测试日志文件尾有空行（[日志](staged-check-initial.log)）；清理末尾空行后重跑通过。

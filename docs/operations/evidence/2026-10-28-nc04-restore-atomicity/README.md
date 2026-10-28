# NC04：读档投影失败保留原局

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)；源码基准 [`0218eb6c5f6bc9e3fa50ce473af13ea26682b60c`](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 不变。

## 反例与修复

`restore_hex` 自身会先解码并校验，再替换内部引擎；但适配层随后才生成 UI 信封，若投影失败，Worker 返回 `ok:false` 时原局已被新存档覆盖。例如当前四座观察者为座位 3，恢复两座存档后视图座位越界：返回失败，保存内容却已改变，旧视图无法读取。

`git archive HEAD apps/web-client` 导出干净源码并只叠加[回归测试](../../../../apps/web-client/scripts/nc04-restore-atomicity.test.mjs)，借用本机依赖及该 HEAD 原生源码生成的 web WASM：**1／2／0，exit 1**（[日志](red-clean.log)）。上一批生产前端在真实 Chrome Worker 也复现了旧档被替换（[日志](browser-red.log)）。第一次测试曾错误要求读档后存档字节完全等于输入，导致第三个测试也失败（[初次日志](red-clean-initial.log)）：已有 `Engine::restore` 按 N11-a 将 `auth_generation` 加一，使恢复前后字节不同；[实测](archive-roundtrip.log)状态摘要相同。修正为与独立恢复后的存档比较，再取得前述 **1／2／0**；未重录任何 golden。一次重跑准备时 `cp` 使用了错误工作目录，测试仍使用旧文件，修正路径后得到有效日志。

现先以临时引擎恢复并生成信封，成功后才在原引擎执行恢复；临时引擎无论成功或失败都释放。原引擎对象与规划参数保持原样；失败时旧局、事件游标与动作缓存不变，成功时按原约定清空游标并使缓存失效。定向 **11／0／0**（[日志](green-targeted.log)）：覆盖有包／无包的观察者越界、注入投影故障、正式恢复故障、资源释放及成功恢复。真实 Chrome 使用生产 Worker 验证失败后旧存档逐字节不变、旧视图可读，以及合法恢复的状态摘要一致；浏览器／Worker 异常 0，**4 项通过**（[驱动](browser-worker.mjs)、[日志](browser-green.log)）。消息结构、规则与恢复时的令牌代次行为不变。

为保留现有引擎引用和规划设置，本实现成功恢复时解码两次。Apple M3 Max、Node v26.12.1、同一 web WASM，20 次预热后各 200 次的恢复＋信封 p50/p95：四座标准 **0.061/0.109 → 0.097/0.123 ms**，八座标准 **0.082/0.108 → 0.118/0.133 ms**，四座 huanle 包 **0.085/0.118 → 0.147/0.196 ms**（[方法](restore-benchmark.mjs)、[环境](environment.txt)、[前](benchmark-before.log)、[后](benchmark-after.log)）。这是本机 Node 微基准，并非大包、浏览器或移动端性能验收；未来具备原子交换状态的窄壳接口时可消除重复解码。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck`；`pnpm --dir apps/web-client smoke:native` | 前端 **99／0／0**、类型检查通过、真实 WASM Worker 冒烟 **62／0**，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)、[冒烟](native-smoke.log)）。 |
| `pnpm --dir apps/web-client exec vite build --outDir /tmp/nc04-restore-dist.05PU0J` | 生产前端构建通过（[日志](frontend-build.log)）；本批 Rust 源码未改，复用此前重新生成的 web WASM（[摘要](wasm-sha256.txt)、[构建](../2026-10-28-nc03-wasm-actor-bounds/build-native.log)）。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）；复用前批独立 target 与映射。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 均通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。日志行尾空白已规范化，不改变测试结论。 |

NC04 仍为部分：其他故障矩阵、联机恢复及 B7 挂起恢复缺陷尚未闭合。原有八份文档修改保留，主稿仅暂存本批追加行。

# NC03／NC04：WASM 座位与整数边界

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)，源码基准 [`12fac3d1866f1f554f583106841474e9f8d86ad8`](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 不变。

## 反例与修复

原生窄壳把 `u32` 命令座位和目标直接截成 `u8`，因此 256 可变成座位 0；选择应答还忽略传入座位，以选择者身份代答。JavaScript 到 WASM 的绑定和 `Uint32Array` 也会截断数值，故仅修 Rust 不够。干净 HEAD 经 `git archive` 导出并只叠加 Rust 反例，独立 target 定向运行 **0／3／0，exit 101**（[日志](red-clean.log)）；前端干净 HEAD 加边界测试、借用本机安装依赖和生成包运行 **0／2／0，exit 1**（[日志](red-frontend-clean.log)），暴露入参未先校验。前端红测准备时第一次 `cp` 路径误写导致找不到测试，修正路径后取得上述有效日志。

窄壳拒绝不可表示的 actor 与目标，并在读取选择候选前核对实际作答座位；合法座位仍交内核判断。Worker 适配层在建局、选将、命令和选择入口校验 JS 数值，防止人数／座位／牌／技能编号经绑定截断。错误沿用既有异常或命令拒绝结果结构；合法命令、权威状态摘要与规则均未修改。定向 Rust **3／0／0**（[日志](green-rust.log)），前端边界及既有适配、选将测试 **22／0／0**（[日志](green-frontend-targeted.log)，其中真实 WASM 选将部分使用当时已生成的旧产物；最终全套使用本批重建产物）。

真实 Chrome 使用本批构建的生产前端 Worker 与 WASM：7 条大整数建局、选将和命令请求均被拒且存档不变；其他座位代答返回 `WrongPlayer`、存档不变，随后正常选择可继续；Worker／浏览器异常 0（[驱动](browser-worker.mjs)、[日志](browser-worker.log)）。此验证只证明本地 Worker 的数值与座位边界，不是联机用户身份认证。直接调用生成的 wasm-bindgen JavaScript API 时，超过 `u32` 的 JS 数值可能在进入 Rust 以前截断；本批产品 Worker 已在此之前校验，其他直接调用方仍需自行校验。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端 **93／0／0**，类型检查通过，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)）。 |
| `CARGO_NET_OFFLINE=true pnpm --dir apps/web-client build:native`；`pnpm --dir apps/web-client exec vite build --outDir /tmp/nc03-actor-dist.vnCev3` | 当前源码重建 web WASM 和生产前端，均 exit 0（[WASM](build-native.log)、[SHA-256](wasm-sha256.txt)、[前端](frontend-build.log)）。wasm-pack 预构建工具下载不可用后使用本地工具链完成。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。有既存未使用警告。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 全部通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。 |

NC03／NC04 仍为部分；其他产品场景、联机身份校验及 B7 挂起恢复缺陷仍待处理。原有八份文档修改保留，主稿仅暂存本批追加行。首次暂存检查发现三份日志末尾多余空行（[记录](staged-check-initial.log)）；规范化日志行尾空白后复查通过，测试结论不变。

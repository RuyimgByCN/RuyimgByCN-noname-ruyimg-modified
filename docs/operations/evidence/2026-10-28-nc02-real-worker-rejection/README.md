# NC02／NC04：真实浏览器 Worker 异常包与恢复验证

- 实际执行：2026-10-28 09:02:51 +0800 → 2026-10-28 09:08:15 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`de4baa26f9b5610e47dfafff9cc63a2fd6b06fe4`](source-head.txt)。规则 v323、存档格式 v105、全局原生 ABI 753 不变。本批仅新增验证脚本及证据，不改变产品协议和规则。

## 范围与结果

从该 HEAD 的 `git archive` 独立导出，离线构建 Web 原生 WASM 和 Vite 生产产物；用隔离配置的 headless Chrome 在本地临时页直接创建构建产物中的原生 Worker。脚本核对生产主包引用了该 Worker chunk，并通过浏览器级 CDP 确认其 `worker` 目标属于测试页；脚本、Chrome 及产物摘要见[环境](environment.txt)。这是真实独立 Worker 与消息入口，但临时页不经过产品 UI，不能代替完整 UI、跨浏览器或目标平台验收。Vite 构建还复制了现有的旧线 WASM 以满足构建依赖，测试只驱动本次独立构建的原生 Worker。

真实 VM 夹具及两份修复前生成的异常档经现有消息协议验证（[脚本](../../../../apps/web-client/scripts/nc02-real-worker-rejection.mjs)、[日志](browser-attempt-04.log)）：

- 包触发索引被篡改、模块 `local_count` 被篡改、跳转落入立即数、入口落入立即数，共四类异常；每类均在普通局启动、身份局启动、恢复三个入口拒绝，共 12 次拒绝，且每次拒绝后当前对局的存档逐字节不变。后两类恢复使用真实旧异常档；前两类使用合法档与异常包组合。
- 合法包选将后仍拥有 VM 技能；阶段结束效果正确摸一张牌，保存、恢复并重放同一命令的状态哈希一致。页面异常、页面控制台错误及 Worker error 事件均为 0。
- **8 项检查通过、0 失败**。NC02 与 NC04 均仍为部分；用户安装 VM 包、签名及资源绑定、跨包组合、续行来源认证与其余故障／恢复矩阵未由本批覆盖。

## 失败记录与定位

首次复用先前红测用过的 `/tmp/noname-nc04-boundary-target.Qd5mZZ`，WASM 全量 **60 通过／1 失败／0 忽略**，控制流测试表现为旧校验器仍接受异常包（[原日志](engine-native-wasm.log)）。同一 HEAD 以独立 `git archive` 和全新 target 重编，定向测试 **1/0/0**（[复现](clean-head-reproduction.log)），全量 **61/0/0**（[日志](engine-native-wasm-clean.log)）。在旧临时 target 只清除 `rule-vm` 构建产物并重编其调用方后同一定向测试转为 **1/0/0**（[诊断](cache-diagnosis.log)）；可定位为复用该红测目录产生的旧 rule-vm 构建缓存，后续全量采用独立 target。

浏览器探针前两次因 Chrome 报告的 Worker 目标 URL 为空而超时（[一次](browser-attempt-01.log)、[二次](browser-attempt-02.log)）；改用 CDP 目标的页面父子关系识别。第三次在全部 12 次异常拒绝通过后，错误地要求成功恢复前后存档逐字节相同（[日志](browser-attempt-03.log)）；实际恢复有意递增存档中的认证代次以轮换旧令牌，日志显示载荷仅一字节和校验和变化。最终检查改为验证恢复前后状态哈希及恢复后的重放，失败操作仍逐字节比较原局存档。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=<独立 target> pnpm --dir <HEAD 导出>/apps/web-client build:native`；`vite build --outDir <独立导出>/dist` | 两项 exit 0；[WASM 构建](wasm-build.log)、[Vite 构建](vite-build.log)。导出路径见[记录](build-path.txt)。 |
| `node apps/web-client/scripts/nc02-real-worker-rejection.mjs <dist> <repo> <Chrome>` | 真实 Chrome Worker **8/0**，exit 0（[日志](browser-attempt-04.log)）。 |
| `cargo test --offline -p engine-native-wasm --no-fail-fast`；`cargo test --offline -p content-native-loader --no-fail-fast` | 干净 HEAD、独立 target：**61/0/0** 和 **32/0/0**，均 exit 0（[WASM](engine-native-wasm-clean.log)、[loader](content-native-loader-clean.log)；[导出与 target](clean-repro-path.txt)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck`；`node --check apps/web-client/scripts/nc02-real-worker-rejection.mjs` | 前端 **87/0/0**，类型检查与脚本语法检查均 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)、[语法](node-check.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 自检 **1/0**，**8636** 文件合格，**1843** 文件链接 0 坏链，ADR 检查通过；均 exit 0（[自检](source-unit.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)）。 |

本批前的八份用户文档改动保留；进度与台账仅暂存本批追加行。

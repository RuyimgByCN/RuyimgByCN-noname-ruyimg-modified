# NC02：已有 VM 包副本的开局与恢复

- 实际执行：2026-10-28 09:46:11 +0800 → 2026-10-28 09:54:35 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；开工源码 [`829f1ad7bb0046776435b3617daf2b874c235316`](source-head.txt)。规则 v323、存档格式 v105、全局原生 ABI 753 不变。

## 反例、修复与范围

以 `git archive HEAD` 导出干净源码，叠加[反例测试](../../../../apps/web-client/scripts/native-vm-snapshot.test.mjs)，并指定独立 `CARGO_TARGET_DIR` 运行 `node --import ./scripts/ts-register.mjs --test scripts/native-vm-snapshot.test.mjs`：真实编译器夹具及空模块 npack 均被前端报“缺少可装载的卡牌、武将或牌堆数据”，**0 通过／2 失败，exit 1**（[日志](red-clean.log)、[临时目录](red-clean-directory.txt)，执行后已清理）。同一包类型此前已由原生 Worker 接受。

前端共享校验器现在允许 `modules` 为数组的完整包；`modules` 存在但不是数组时，即使附有其他资产也明确拒绝，现有资产类型校验保持不变。开局内容锁与存档副本按原包传递；真正的 npack、模块摘要与 ABI 校验仍在原生 Worker 加载时进行，没有变更 Worker 消息结构。测试还覆盖内容锁指纹不符、撤销、空模块与坏资产。

浏览器在独立临时 profile 中**预置**原包副本和启用锁，经过真实 UI、原生 Worker 建局、存档、推进、恢复，状态哈希恢复到原值；篡改模块后的读档被拒绝，拒绝前后存档字节相等（[脚本](browser-vm-only.mjs)、[输出](browser.log)、[截图](browser.png)）。独立 Worker 冒烟额外覆盖身份局和坏模块的开局／恢复拒绝。该夹具的清单仍声明武将资产，但本次没有附加资产；这只证明 VM 载荷的传递和恢复，**不证明资源完整性、VM 技能可由武将装配，或已通过扩展管理页安装**。

首次 Worker 冒烟中新增的两项断言失败（[工作树](native-worker-smoke.log)、[干净 HEAD 叠加本批测试复现](smoke-clean-repro.log)）；同一导出恢复 HEAD 原脚本后 **60／0**（[日志](smoke-clean-original.log)）。原因是测试错误要求恢复前后归档逐字节相同：内核恢复时递增并保存会话令牌代次以使旧令牌失效。改以状态哈希核验恢复一致，并以**恢复后的**存档作为拒绝前后字节比较基准；最终 62／0。没有把首次失败记成产品故障。

首次 `pnpm --dir apps/web-client build` 覆盖了开工时干净的受跟踪 `dist` 文件；本批恢复了这五个被本次构建覆盖的文件，之后将 Vite 构建到独立临时目录并重跑浏览器（[首次构建](frontend-build.log)、[首次浏览器](browser-first.log)、[临时目录](web-dist.txt)）。本批没有暂存构建产物。

## 验证

| 命令 | 完整结果 |
|---|---|
| `node --import ./scripts/ts-register.mjs --test scripts/native-vm-snapshot.test.mjs`（`apps/web-client`） | **2／0／0**，exit 0（[日志](focused.log)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 测试 **89／0／0**；类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `CARGO_NET_OFFLINE=true pnpm --dir apps/web-client build:native`；`pnpm --dir apps/web-client smoke:native` | 新构建实际 web WASM；Worker 冒烟 **62／0**，均 exit 0（[构建](build-native.log)、[冒烟](native-worker-smoke-final.log)）。 |
| `pnpm --dir apps/web-client exec vite build --outDir <临时目录>`；`node browser-vm-only.mjs <临时目录> <仓库> <Chrome> <截图>` | 构建 exit 0；headless Chrome **5／0**、异常 0、exit 0（[构建](frontend-build-external.log)、[浏览器](browser.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | engine-native **1605／0／73**、engine-native-wasm **62／0／0**、content-native-loader **32／0／0**、game-server **360／0／0**；合计 **2059／0／73**、381 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。 |
| 干净验证提交 `cargo check --offline --workspace --all-targets` | 验证提交 [`488d520595c60c4b243490de45bc4421c2fef325`](validation-commit.txt)，检查前后工作树均干净，exit 0（[日志](workspace-check-clean.log)、[前](validation-status-before.log)、[后](validation-status-after.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1／0**；**8641** 个受维护文件不超 2000 行，均 exit 0（[自检](source-line-tests.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | **1847** 个文件、0 坏链；ADR 索引通过；工作树无空白错误，均 exit 0（[链接](local-links.log)、[ADR](adr-index.log)、[工作树](diff-check.log)）。 |

NC02 保持部分：独立 VM 包安装与版本生命周期、真实资源绑定、跨包规则引用、更多包类型及目标平台验收仍待完成。本批保留开工时的八份用户文档改动，进度和台账仅提交本批追加行。

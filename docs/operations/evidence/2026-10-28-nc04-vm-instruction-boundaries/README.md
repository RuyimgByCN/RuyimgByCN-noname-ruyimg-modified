# NC04：VM 指令边界与续行落点

**口径更正（2026-10-28）**：此处的冒烟在 Node 中调用实际 Web WASM 和 `nativeBackend.ts`，没有创建浏览器 Worker 线程；原文“真实 Worker”及相应计数不能证明线程级消息链。

- 实际执行：2026-10-28 00:39:56 +0800 → 2026-10-28 01:26:03 +0800（[结束](end-time.txt)）；源码基准 `5fc86d1739391e98c1b87bbca08d324cb76d57f4`（[开始](start-time.txt)、[源码 HEAD](source-head.txt)）。规则 v323、原生存档 v105、旧线 snapshot schema 2 和原生 ABI 753 不变；合法字节码的执行和序列化不变，无 golden 重录。原本可被接受的无效控制流包及其旧档现会明确拒绝。

## 反例与修复

基于 `git archive HEAD` 的干净导出，叠加本批测试并使用[独立 target](cargo-target-path.txt)：原校验器仅线性解码，入口及有条件／无条件跳转可落入立即数字节或代码末尾。实测将 772 嵌在 `CONST_I32` 的立即数内，ABI 标签扫描报告未使用 772，两套解释器却从跳入的位置发出了 `ChooseOtherTargetsBySkippedPhaseCountDamageDraw`；伪造续行 PC 也可完成同样操作。修复前 **1 passed / 3 failed**（[红测](red-rule-vm-corrected.log)）。第一次红测因测试字段名写错而编译失败，已修正并保留[原日志](red-rule-vm.log)。

真实、指纹自洽的 ABI 753 包在原版本中被 schema、桥、普通局、身份局与旧档恢复接收；同类 ABI 758 包还可通过 schema（[包红测](red-native-abi758.log)）。旧线直接加载并恢复异常冻结快照（[旧线红测](red-core.log)）；签名正确的异常升级包此前能安装并激活（[安装红测](red-runtime.log)）。这只证明上述包接收及解释器发出请求，并未证明原生游戏接受此效果并落账。

共享校验器现在先记录所有指令边界，要求模块入口和每个跳转目标落在实际指令上；效果 ABI 扫描对所有 ABI 都执行结构校验。装载期 `VerifiedModule` 额外记录 `HOST_CHOICE` 和 `HOST_CHOICE_RANGE` 的续行点，两套解释器续行只允许这些点，避免运行期重扫完整模块。原模块序列化字节保持不变。包安装、原生桥、Worker、旧线加载及序列化恢复沿现有校验路径继承拒绝；正常选择、范围选择、前跳／倒跳、合法档恢复与 VM 序列化有回归。

修复前真实生成的两份异常原生档保存在[Worker 夹具](../../../../apps/web-client/scripts/fixtures/vm-control-flow/old-native-saves.json)，异常旧线快照见[旧档](old-legacy-snapshot.json)。修复后两份原生档、旧线档、异常的新包及带合法签名的升级包都拒绝；正常对局及内容锁不受失败操作影响。Web WASM 后端适配层的普通局、身份局和旧档均拒绝，失败后已有对局存档字节不变（[后端冒烟日志](worker-smoke.log)）。

## 仍未完成

续行点检查不认证续行来源：同一模块中如有另一个真实选择点，伪造 PC 仍可能跳到那里。独立[探针](uncovered-resume-authentication.rs)实测真实 PC 8 产生 0 个效果、伪造为另一个合法选择点 PC 17 后产生摸 7 请求（[日志](resume-authentication-probe.log)）；这是直接 VM API 反例，未声称真实 Worker 存档已被攻破。旧线快照、资源字节及来源认证、`max_stack` 绑定，以及 NC04 故障／回滚／恢复全矩阵仍须继续。NC02 与 NC04 都保持部分。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p rule-vm --test control_flow_boundaries -- --nocapture` | **4/0/0**；含两种选择、两个解释器、越界／正反跳及序列化（[日志](target-rule-vm-final.log)）。 |
| `NC04_BOUNDARY_SAVES_INPUT=<Worker 夹具> cargo test --offline -p engine-native-wasm --test nc04_vm_control_flow -- --nocapture`；`NC04_BOUNDARY_LEGACY_INPUT=<旧档> cargo test --offline -p engine-core --test nc04_vm_control_flow -- --nocapture` | 各 **1/0/0**，覆盖真实旧档和合法档（[原生](target-native-final.log)、[旧线](target-core.log)）。 |
| `cargo test --offline -p content-runtime --test nc04_vm_control_flow -- --nocapture` | **1/0/0**，两种异常的签名升级均被拒绝、现用锁不变（[日志](target-runtime.log)）；在导出上覆盖本批源码，[逐文件一致](runtime-final-source-match.txt)。 |
| 仓库追踪的全部 npack 模块控制流盘点 | **144 份 npack、13,771 个模块、0 个无效目标**（含历史与重复副本；[文件清单](package-files.txt)、[探针](package-inventory.rs)、[日志](package-inventory.log)）。此项不代表这些包全部通过 ABI、资源或功能验收。 |
| `cargo test --offline -p <crate> --no-fail-fast` | 共 **6514/0/73**（通过/失败/忽略）：[rule-vm 21/0/0](rule-vm-full.log)、[engine-native 1588/0/72](engine-native-full.log)、[wasm 61/0/0](engine-native-wasm-full.log)、[loader 32/0/0](content-native-loader-full.log)、[content-runtime 36/0/0](content-runtime-full-final.log)、[engine-core 3846/0/1](engine-core-full.log)、[game-server 360/0/0](game-server-full.log)、[content-compiler 541/0/0](content-compiler-full.log)、[content-schema 0/0/0](content-schema-full.log)、[engine-ai 29/0/0](engine-ai-full.log)。全部 exit 0（[状态](full-status.txt)、[运行时最终状态](content-runtime-full-final-status.txt)、[计数](full-counts.txt)）；运行时重跑使用与当前源码[一致](runtime-final-source-match.txt)的独立导出。 |
| `cargo check --offline --workspace --all-targets` | exit 0（[日志](check-final-clean-head.log)）：以暂存源码树创建临时提交 `c2dc79185811420ba9d4b05afaf7fe9210ed66fc`，独立工作树和独立 target 执行；该树与当前本批文件[逐一一致](check-final-source-match.txt)，工作树执行[前](check-final-status-before.txt)、[后](check-final-status-after.txt)均干净，已[移除](check-final-worktree-remove.log)。临时提交只用于验证。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native` | 构建 exit 0、Web WASM 与后端适配层冒烟 **60/0**（[构建](wasm-build.log)、[冒烟](worker-smoke.log)）；导出叠加本批源码与夹具，代码[逐文件一致](wasm-source-match.txt)，node_modules 与独立构建 target 使用链接。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端 **87/0/0**、类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；新增测试 `rustfmt --check` | 均 exit 0；行数自检 **1/0**、**8634** 文件合格、**1841** 文件链接 0 坏链、ADR 无未登记冲突（[自检](source-unit-final.log)、[行数](source-lines-final.log)、[链接](local-links-final.log)、[ADR](adr-index-final.log)、[格式](rustfmt-final.log)）。 |

开工前已有八份文档改动；台帐与进度主稿仅暂存本批追加行，其余改动保留。

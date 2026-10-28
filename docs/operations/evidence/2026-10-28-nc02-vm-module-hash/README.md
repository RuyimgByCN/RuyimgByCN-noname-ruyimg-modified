# NC02：校验现有 VM 模块指纹

**口径更正（2026-10-28）**：此处的冒烟在 Node 中调用实际 Web WASM 和 `nativeBackend.ts`，没有创建浏览器 Worker 线程；原文“真实 Worker”及相应计数不能证明线程级消息链。

- 实际执行：2026-10-28 00:24:11 +0800 → 2026-10-28 00:35:26 +0800（[开始](start-time.txt)、[结束](end-time.txt)；均实跑 `date`）；开工源码基准 `41fca6161aa43a4468bee868eebfb3c1ae7c484b`（[HEAD](source-head.txt)）。
- 保持原生规则 v323、原生存档格式 v105、旧线快照 schema 2、宿主 ABI 753 和已有模块指纹算法。合法包和合法旧档语义不变；之前被接收的模块指纹与内容不一致的包及其旧档，现会在装载或恢复时明确拒绝。

## 反例及改动

以 `git archive HEAD` 导出干净源码，叠加三份新回归测试、使用独立 target：入口、local 数、能力位、代码或声明指纹五种改动均可通过完整包校验、原生桥、普通／身份局开局及其旧档恢复；修改包摘要的字段后反例会重算外层摘要，测试只留下模块内部指纹不一致（[原生红测](red-native.log)、[5 份实际旧档](old-native-saves.json)，0/1、exit 101）。旧线直接装包、已安装包重复装载及仅改入口的真实旧快照均被接受（[红测](red-core.log)、[旧快照](old-legacy-snapshot.json)，0/1、exit 101）。签名正确但 local 数与指纹不一致的包此前可安装并激活（[红测](red-runtime-final.log)，0/1、exit 101）；首次运行未准备夹具资源而先报错（[日志](red-runtime.log)），去掉夹具资产并重算资源摘要后重跑，补充实际返回值才确认该反例（[中间日志](red-runtime-corrected.log)）。导出路径曾在红测后叠加本批源码用于下方补充探针。

复用 `BytecodeModule::compute_logic_hash` 的既有算法验证模块名称、代码、local 数、能力位和入口；`ContentPackage::validate`、原生桥、旧线直接装包（在重复包快速返回之前）及旧线快照恢复都调用该校验。内容运行时和服务端冻结包经完整包校验继承此检查。两个手写运行时测试夹具此前用代码的裸 blake3 而非模块指纹，现改用既有算法；不修改任何正式包。此前清点的 144 个仓库追踪 npack JSON 共 13,771 个模块均匹配（[清点](../2026-10-28-nc02-vm-package-hash/module-inventory.log)，含重复及历史夹具，不代表第三方包）。

修复后五种输入在普通／身份开局、合法档换入坏包、五份实际异常旧档恢复中均明确报模块 `logic_hash mismatch`；合法包的开局和恢复仍通过。旧线相同包重复装载和空注册表装载都拒绝异常入口，实际旧快照恢复报 `ContentCompatibility`；合法快照和英逝／平寇 ABI 兼容回归通过。签名正确的异常升级包拒绝且现用内容锁不变。Web WASM 后端适配层的普通局、身份局和恢复均拒绝异常模块，失败后当前局存档字节不变（[后端冒烟日志](worker-smoke.log)）。

## 剩余边界

模块自校验不是来源认证：如同时修改内容和重算指纹，单凭包内声明无法证明来源。旧线快照的状态摘要未绑定冻结模块本体；[独立探针](uncovered-snapshot-authentication.rs)实际修改入口并重算模块指纹，保持原状态摘要，修复后的快照仍被接受（[日志](snapshot-diagnostic.log)，1/0、exit 0）。现有模块指纹也不包含 `max_stack`；内嵌资源字节绑定、精简资产载荷、正式 VM 安装入口及签名认证仍待处理。旧档一致性检查不能视为完整的防篡改或上游等价证明。

## 验证

| 命令 | 完整结果 |
|---|---|
| `NC02_MODULE_OLD_SAVES_INPUT=<证据目录>/old-native-saves.json cargo test --offline -p engine-native-wasm --test vm_module_hash --test vm_package_integrity --test vm_package_worker -- --nocapture` | **6 passed / 0 failed / 0 ignored**（[日志](target-native.log)）。 |
| `NC02_MODULE_LEGACY_SNAPSHOT_INPUT=<证据目录>/old-legacy-snapshot.json cargo test --offline -p engine-core --test nc02_vm_module_hash --test pa12_tag656_compat -- --nocapture` | **4/0/0**（[日志](target-core.log)）。 |
| `cargo test --offline -p content-runtime --test nc02_vm_module_hash --test atomic_install -- --nocapture` | **8/0/0**（[日志](target-runtime.log)）。 |
| `cargo test --offline -p <crate> --no-fail-fast`，对以下九包逐一执行 | 全部 exit 0。engine-native **1588/0/72**、engine-native-wasm **60/0/0**、content-native-loader **32/0/0**、content-runtime **35/0/0**、engine-core **3845/0/1**、game-server **360/0/0**、content-compiler **541/0/0**、content-schema **0/0/0**、engine-ai **29/0/0**（通过/失败/忽略；[状态](rust-full-status.txt)、[计数](rust-full-counts.txt)、[内核](engine-native-full.log)、[WASM](engine-native-wasm-full.log)、[加载器](content-native-loader-full.log)、[运行时](content-runtime-full.log)、[旧线内核](engine-core-full.log)、[服务器](game-server-full.log)、[编译器](content-compiler-full.log)、[模式](content-schema-full.log)、[AI](engine-ai-full.log)）。使用[独立 target](cargo-target-path.txt)，关闭增量及 test/dev 调试信息。 |
| `cargo check --offline --workspace --all-targets` | exit 0（[日志](check-clean-head.log)）。因新增公共 schema 校验函数，先以暂存源码创建临时提交 `a696fc8eff1cc9a89a784891eb5568d31de80519`，在其独立工作树、[独立 target](check-target-path.txt)执行；测量时 [HEAD](check-head.txt) 即此提交，工作树[执行前](check-worktree-status.txt)和[执行后](check-worktree-status-after.txt)均无改动，源码与当时暂存树[逐文件无差异](check-staged-difference.txt)。临时提交只用于验证，不作为最终交付提交；干净工作树已[移除](check-worktree-cleanup.txt)。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native` | WASM 实际构建 exit 0，Web WASM 与后端适配层冒烟 **58/0**（[构建](wasm-build.log)、[冒烟](worker-smoke.log)）。导出基于开工 HEAD 并覆盖本批源码，逐文件[对照一致](build-source-match.txt)；前端依赖通过指向工作区 node_modules 的链接读取。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端 **87/0/0**、typecheck exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `rustfmt --check --edition 2021 <三份新增测试与探针>` | exit 0（[日志](rustfmt.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | 行数检查单测 **1/0**，8630 个受维护文件均 ≤2000 行（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 本地链接和 ADR 检查均 exit 0（[链接](local-links.log)、[ADR](adr-index.log)）。 |

额外运行 `git diff --cached --check` 时，原样保留的测试日志末尾空行使全范围检查 exit 2；限定源码及主稿后 exit 0（[全范围输出](diff-check-all.log)、[限定输出](diff-check-source.log)、[状态](diff-check-status.txt)）。

开工前已有八份文档改动；本批对混合的进度和台帐只暂存追加行，其余原样保留。

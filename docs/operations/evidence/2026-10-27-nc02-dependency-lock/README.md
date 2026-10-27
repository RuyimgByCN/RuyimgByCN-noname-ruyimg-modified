# NC02：浏览器与 WASM 的跨包依赖锁

- 实际执行：2026-10-27 23:18:14 +0800 → 2026-10-27 23:27:00 +0800（开工与结束各运行 `date '+%F %T %z'`）；源码基准 `79f0e91f02f6ad8cad774cdcba1d611bdb276b85`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。原有八处文档改动保留。

## 反例、修复与兼容边界

服务端内容运行时核对依赖槽位、精确版本和可选逻辑指纹；浏览器开局／冻结存档恢复和 WASM 包入口此前未核对。对 `git archive HEAD` 的[独立导出](red-path.txt)仅叠加测试，Rust 使用独立 `CARGO_TARGET_DIR`：浏览器定向 **15 passed / 1 failed**（[日志](red-frontend.log)），九类缺失、错误或无效依赖在开局和恢复均被接受；WASM 定向 **0/1、exit 101**（[日志](red-wasm.log)），普通局与身份局均接受并能保存恢复。补强的反例在同一干净源码导出上生成了[九份旧版存档](old-saves.json)（[日志](red-wasm-old-saves.log)）。测试以正式包的卡牌／武将数据为基础，人工附加依赖声明；它不是签名包或真实性验证。

浏览器包载荷校验依赖字段，组装开局或冻结副本时核对整个锁；WASM 共享包装载入口在普通局、身份局与恢复前作同样检查。缺包、命名空间／版本／非零指纹不符，或依赖声明无效、重复、自引用时拒绝；省略指纹及全零指纹保留服务端既有通配语义，合法循环依赖允许。没有依赖的旧包行为不变。以前被错误接受、但依赖锁无效的旧存档现在明确拒绝；上述九份旧存档逐个验证为拒绝（[日志](old-saves-restore.log)）。本批不改规则、存档格式或 ABI 版本。

真实 Worker 冒烟首轮 **54/1**（[日志](worker-smoke.log)）：新增测试错误地用成功恢复前的存档字节作为后续拒绝操作的基线。另用 `git archive HEAD` 的[干净导出](restore-diag-path.txt)、独立 `CARGO_TARGET_DIR` 和[诊断测试](restore-diagnostic-red.rs)复现 **0/1**（[日志](restore-clean-head-red.log)）；恢复前后状态摘要相等，但授权代次 `0→1`。恢复测试把代次归一后存档字节完全相等，**1/0**（[测试](restore-diagnostic-green.rs)、[日志](restore-clean-head-green.log)）；源码的 `Engine::restore_with_catalog` 明确轮换授权代次以使旧令牌失效。于是改用成功恢复后、拒绝操作之前的字节作基线；最终九次拒绝均保持字节不变（[日志](worker-smoke-after-restore.log)）。

## 验证

| 命令 | 完整结果 |
|---|---|
| `NC02_DEPENDENCY_OLD_SAVES_INPUT=<证据目录>/old-saves.json cargo test --offline -p engine-native-wasm --test installed_package_payload installed_dependency_lock_is_checked_on_start_and_restore -- --nocapture` | **1 passed / 0 failed / 0 ignored**；含九份实际旧档拒绝、有效锁、通配及循环依赖（[日志](old-saves-restore.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p content-runtime -p engine-ai --no-fail-fast` | exit 0；依次 **1584/0/71、57/0/0、32/0/0、34/0/0、29/0/0**（通过/失败/忽略；[日志](rust-full.log)、[计数](rust-counts.txt)）。 |
| `cargo check --offline --workspace --all-targets` | 隔离索引生成的[验证提交](validation-commit.txt)以源码基准为父提交，仅含六个本批源码和测试文件；在[独立干净检出](validation-path.txt)、检出外 `CARGO_TARGET_DIR` 运行，exit 0（[日志](workspace-check.log)，[检查前状态](validation-before.log)、[检查后状态](validation-after.log)均为空）。上游 `archive` 链接放在检出外。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native` | 构建实际 WASM exit 0；Worker onmessage 冒烟最终 **55/0**（[构建](wasm-build.log)、[冒烟](worker-smoke-after-restore.log)）。依赖来源为真实包数据加测试声明；没有覆盖工作区生成产物。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **87/0/0**；类型检查 exit 0（[测试](frontend-verified.log)、[类型](typecheck-verified.log)）。 |
| `rustfmt --check --edition 2021 crates/engine-native-wasm/src/installed_packs.rs crates/engine-native-wasm/tests/installed_package_payload.rs` | exit 0（[日志](rustfmt.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；8622 个文件通过（[单测](source-unit.log)、[行数](source-lines-final.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 链接及 ADR 检查通过（[链接](local-links.log)、[ADR](adr.log)）。 |

NC02 仍为部分完成：本批只验证依赖声明与锁的一致性，不重新计算或认证完整包指纹；未实现 VM 本地安装、依赖启停事务或跨包真实规则引用，也未验收全部混合包类型、真实浏览器 UI 全流程或目标平台。

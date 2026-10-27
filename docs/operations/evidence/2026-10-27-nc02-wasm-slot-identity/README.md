# NC02：WASM 安装包槽位重复拒绝

- 实际执行：2026-10-27 22:53:25 +0800 → 2026-10-27 23:00:07 +0800（开始和结束各运行 `date '+%F %T %z'`）；源码基准 `47796ad33cc0f5996640916fe57555527ecbd23e`（[时间](start-time.txt)、[源码](source-head.txt)）。原有 8 个在途文档改动保留，本批只暂存自身代码、测试及回填。

## 反例、根因与修复

`git archive HEAD` 导出到[独立目录](clean-path.txt)，只叠加新增测试，使用该目录内独立 `CARGO_TARGET_DIR`。同一 `(namespace, package_id)` 的两个安装副本中，第二个更换版本并清空武将资产，原普通建局仍接受，测试 **0/1、exit 101**（[反例](red-clean-head-04.log)）。此前去重使用包含版本与资产摘要的最终名称，两个不同版本或内容因名称不同而不会命中同槽重复；另一些输入恰好因技能重名在更下游被拒绝，不能作为槽位校验。

共用 `packs_from_json` 先以 `(namespace, package_id)` 预检安装槽位，重复即报错，然后才桥接任何包；普通开局、身份局和恢复共用该入口。同 ID 的不同命名空间仍是不同槽位；裸 legacy 载荷仍沿用原有检查。原来已经建出的非法重复槽位存档如今明确拒绝恢复；合法单槽版本及其状态、规则与存档编码未变，没有重录 golden 或升版本。服务端冻结房间和浏览器内容锁已有同槽去重，本次修复 WASM 入口，不声称跨包依赖或全部包验证已经完成。

设计反例过程中，首次使用完全相同包遇到旧的“重复包”错误，测试要求新错误文案而失败（[记录](red-clean-head.log)）；其次更换版本但保留同名技能，被更下游的技能冲突挡住（[记录](red-clean-head-02.log)）。将第二副本的武将数组清空后，独立建局确实接受（[记录](red-clean-head-03.log)、[明确入口](red-clean-head-04.log)）。这些失败均在上述干净导出及独立 target 上复现，不归因于工作区改动。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native-wasm --test installed_package_payload` | **5/0/0**：同槽相同副本、不同版本、同版本不同内容均在普通/身份开局和恢复被拒；不同命名空间的同 ID 可装载（[日志](wasm-targeted.log)） |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | exit 0；engine-native **1584/0/71**、wasm **54/0/0**、loader **32/0/0**、game-server **360/0/0**（通过/失败/忽略；[日志](rust-full.log)、[计数](rust-full-counts.txt)） |
| `cargo test --offline -p engine-native-wasm` | 格式化后的最终文件全量 **54/0/0**、exit 0（[日志](wasm-full-final.log)） |
| `CARGO_NET_OFFLINE=true pnpm --dir <导出>/apps/web-client build:native`；`pnpm --dir <导出>/apps/web-client smoke:native` | 最终源码重建的实际 WASM + Worker onmessage 在 Node 冒烟 **52/0**；普通局、身份局、恢复均报重复槽位，拒绝后原局存档字节不变（[构建](native-build-final.log)、[冒烟](native-smoke-final.log)）。导出只叠加本批源文件/测试，依赖目录链接到现有 node_modules，target 留在导出内；未覆盖用户安装的前端产物。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **80/0/0**；类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)） |
| `rustfmt --check --edition 2021` 两个改动的 Rust 文件 | 最终 exit 0（[输出](rustfmt-final.log)）。首次检查失败既包含本批排版，也包含原文件旧排版；原 `HEAD` 文件单独检查同样失败（[首次](rustfmt-check.log)、[原文件](rustfmt-head.log)）。随后仅格式化这两个文件并重跑 wasm 全量和 WASM 构建/冒烟；其余 Rust 套件在格式化前已通过。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；**8621** 个文件通过（[单测](source-line-tests.log)、[行数](source-lines.log)） |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | **1831** 文件、0 坏链（[日志](local-links.log)）；ADR exit 0（[日志](adr-index.log)） |

本次不是浏览器 UI 全流程或第三方安装验收。NC02 仍需本地 VM 安装入口、跨包依赖及多类型场景；对非法旧存档明确拒绝的影响如上。

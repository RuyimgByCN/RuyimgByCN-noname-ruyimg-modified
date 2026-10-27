# NC08：四座与八座原生／WASM 挂起恢复轨迹

- 实际执行：2026-10-27 23:12:58 +0800 → 2026-10-27 23:17:20 +0800（开工与结束各运行 `date '+%F %T %z'`）；源码基准 `2db94ba86b278c69224769bd09bf74f6b96ad3bd`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。原有八处文档改动保留。

## 范围与结果

固定种子 `20261027` 下，以正式 huanle 卡牌包和真实 VM 测试包组成同一输入，四座与八座各给 0 号位指定 VM 武将，再各执行 80 条命令。遇到选择时由所属座位选择最少数量的前几个选项；否则由当前座位推进阶段。验证 VM 结束阶段触发实际摸牌，且其他座位的私有选项对 0 号位隐藏、对所属座位可见。

两组均在第 9 条命令后、其他座位私有选择尚未回答时保存。原生和 WASM 分别恢复，并与各自直跑结果逐步相等；两目标的直跑及恢复前后逐步对照也相等。比较字段包括选择 ID、所属座位、最少和实际选项数、隐私标志、行动者、回合／阶段、0 号位手牌数、修订号、状态摘要和新增公开事件**码**；不比较公开事件完整载荷。四座有 9 次、八座有 11 次其他座位的私有选择。结束摘要分别为：

- 四座：`d95ecae5d6bd15fd440d22981789c0d3bdd647159a7642cc2505d5ed9f8c5ca9`
- 八座：`66e8d496226a8e770cb38f0c6269b19f6e2508ffd2184a57069bb64802f995fa`

输入及逐步结果见[原生轨迹](native.json)、[WASM 轨迹](wasm.json)、[对照结果](wasm-compare.log)和[对照脚本](wasm-multiseat.mjs)。`git archive HEAD` 导出到[独立目录](clean-path.txt)，仅叠加本批 Rust 测试，使用独立 `CARGO_TARGET_DIR` 运行原生轨迹；同一干净导出用独立构建目录生成实际 WASM，Node 加载生成模块并逐步比较。其 `node_modules` 指向工作区已有依赖；未覆盖工作区生成产物。

## 验证

| 命令 | 结果 |
|---|---|
| `NC08_MULTISEAT_OUTPUT=<导出>/native.json CARGO_TARGET_DIR=<导出>/target-independent cargo test --offline --manifest-path <导出>/Cargo.toml -p engine-native-wasm --test cross_target_multiseat -- --nocapture` | 独立导出加本批测试 **1 passed / 0 failed / 0 ignored**（[日志](native-test.log)）。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <导出>/apps/web-client build:native`；`node wasm-multiseat.mjs <导出>/apps/web-client/src/worker/native-pkg <导出>/native.json <导出>/wasm.json` | 实际 WASM 构建 exit 0；四座和八座各 80 步、保存恢复及原生／WASM 对照通过（[构建](wasm-build.log)、[对照](wasm-compare.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | exit 0；engine-native **1584/0/71**、engine-native-wasm **56/0/0**、content-native-loader **32/0/0**（通过/失败/忽略；[日志](rust-full.log)、[计数](rust-counts.txt)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端 **86/0/0**，类型检查 exit 0（[测试](frontend.log)、[类型](typecheck.log)）。 |
| `rustfmt --check --edition 2021 crates/engine-native-wasm/tests/cross_target_multiseat.rs` | exit 0（[日志](rustfmt.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；8622 个文件通过（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 链接检查和 ADR 检查通过（[链接](local-links.log)、[ADR](adr.log)）。 |

NC08 仍为部分完成：本批是固定种子的普通局样本，初始随机发牌和摸牌纳入状态摘要，未覆盖所有随机技能、多种模式、真实浏览器／服务端对局、实体设备、37 来源或旧线／上游规则差分，不代表上游语义等价或全平台验收。

# NC08：身份局四／八座原生与 WASM 轨迹

- 实际执行：2026-10-28 09:55:45 +0800 → 2026-10-28 10:02:09 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`2e49ff99587b940d1d01c1f6f2ee953ab7e0f201`](source-head.txt)。本批只增加测试和证据，规则 v323、存档格式 v105、全局原生 ABI 753 不变。

## 对照范围

固定种子 `20261028`，使用真实 VM 夹具和欢乐卡牌包，四／八座身份局各给 0 号位选 VM 武将，再各执行 80 次推进或选择应答。原生直跑与执行 9 条命令后、他人私有选择待答时保存恢复的轨迹一致；实际 WASM 的直跑及恢复轨迹与原生逐步一致，VM 武将在结束阶段实际摸牌。四座／八座分别遇到 **9／11** 次其他座位私有选择。

逐步比较行动者、阶段、选择 ID／归属／数量与隐私、0 号手牌数、修订号、状态摘要及新增公开事件码；还在开局、挂起、恢复及最终状态比较所有座位的 WASM 已导出视图，包括本人具体手牌、对手手牌遮蔽、体力、装备、判定区和技能。原生侧另检查每个座位的身份投影：本人及主公可见、其他存活者隐藏，观战者只见主公；各检查点均无人死亡，**不证明死亡后身份公开**。

WASM `view_json` 当前没有导出身份字段，本地 Worker 把已有 `SeatView.identity` 固定为 `null`。因此身份可见性这里只证明原生侧，**没有将它列为跨目标通过**；是否补充该接口已向用户请示，尚未实施。本批没有完成整局、真实浏览器／WS、其他身份座位数、多种子、上游差分或目标平台验收，NC08 仍为部分。

四座最终摘要 `4a31d9d212947ef42f30067783a850bbcf32883f08e4ceaa91dc60f9f6c017a2`；八座 `c4641a1ba8aba8d905e486017de604f5c3020d0da4a96d4c0b8976cdaf6c96db`。输入与结果见[原生](native.json)、[WASM](wasm.json)、[对照日志](wasm-compare.log)及[脚本](wasm-identity.mjs)。

## 构建与失败记录

以 `git archive HEAD` 导出干净源码，仅叠加本批测试并使用[独立 Cargo 目录](clean-target.txt)。首次测试模块路径配置错误，工作树与干净导出均出现 `E0583`（[工作树](native-targeted.log)、[干净复现](native-clean-repro.log)）；修正测试文件的 `#[path]` 后，干净导出定向 **1／0／0**（[日志](native-clean-final.log)）。临时导出和本会话独立 Cargo 缓存已清理；这不是产品故障。

WASM 使用[上一切片](../2026-10-28-nc02-vm-snapshot/README.md)从 `829f1ad7bb0046776435b3617daf2b874c235316` 构建的 web 目标产物；该基准到本批开工 HEAD 的 `crates`、`content`、`packages`、`extensions` 及 Cargo 配置差异为零（[比较](wasm-source-diff.log)、[WASM 摘要](wasm-sha256.txt)）。本批不修改产品源码，没有将旧构建当作新构建。

## 验证

| 命令 | 完整结果 |
|---|---|
| `NC08_IDENTITY_OUTPUT=<输出> CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<独立目录> cargo test --offline --manifest-path <干净导出>/Cargo.toml -p engine-native-wasm --test cross_target_multiseat identity::identity_multiseat_vm_and_package_match_restore_trajectories -- --exact --nocapture` | **1／0／0**，另 1 测试被过滤，exit 0（[日志](native-clean-final.log)）。 |
| `node wasm-identity.mjs <实际 WASM 包> <原生轨迹> <WASM 轨迹>` | 四／八座各 **80 步**、挂起恢复及所列可见视图一致，exit 0（[日志](wasm-compare.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | engine-native **1605／0／73**、engine-native-wasm **63／0／0**、content-native-loader **32／0／0**；合计 **1700／0／73**、356 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[映射](test-artifacts.jsonl)、[脚本](count-tests.py)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 测试 **89／0／0**、类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `rustfmt --edition 2021 --check crates/engine-native-wasm/tests/cross_target_multiseat/identity.rs`；`python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | 格式通过；自检 **1／0**；**8642** 个受维护文件合格，均 exit 0（[格式](rustfmt.log)、[自检](source-line-tests.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | **1848** 个文件、0 坏链；ADR 索引通过；工作树无空白错误，均 exit 0（[链接](local-links.log)、[ADR](adr-index.log)、[工作树](diff-check.log)）。 |

本批保留开工时的八份用户文档改动，进度和台账只提交本批追加行。

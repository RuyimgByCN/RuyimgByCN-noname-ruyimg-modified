# UP37-R05 涕泣／宝梳／避祸 完整闭环与全量验收证据（2026-11-05）

- **核验日期**：2026-11-05
- **授权与合同**：[裁决 28 / 29 / 31](../2026-10-28-user-decisions/README.md)、[R05 运行时合同 v3+v4+v5](../2026-10-31-r05-runtime-contract/README.md)
- **版本轴**：`STANDARD_RULESET_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`RULE_VM_ABI_VERSION = 759`、`PROTOCOL_VERSION = 8`（WS8）
- **前序阶段证据**：A1–F/G 阶段性日志与 168 条阶段/回合触发审计详见 [`../2026-11-03-r05-private-wip/`](../2026-11-03-r05-private-wip/README.md)

---

## 1. 分批实现与红→绿闭环摘要

| 批次 | 范围 | 核心落点 | 红→绿与全量证据 |
|---|---|---|---|
| **R05-A1 & A2** | `Phase::Prepare` + `EventKind::TurnBegin`/`TurnEndWindow` + 168 条阶段/回合触发审计与 106 条迁移 + K21/K25/K10/K22a/K18 | `crates/engine-native-kernel/src/kernel/phase.rs`、`crates/engine-native-kernel/src/kernel/events/turn.rs`、`filtering/*.rs`、`metadata/*.rs` | [`../2026-11-03-r05-private-wip/REPORT.md`](../2026-11-03-r05-private-wip/REPORT.md)、[`../2026-11-03-r05-private-wip/README.md`](../2026-11-03-r05-private-wip/README.md) |
| **R05-B & C & C2** | `D/R/G` 摸牌事件族（`DrawPhase`/`Draw`/`DrawGain`）、`detached_cards` 游离区、191 处同步 `draw_cards` 全部迁移至异步 `Ctx::start_draw` + `HostStage::DrawSettle` 续行位（删除 `Ctx::draw_cards`） | `crates/engine-native-kernel/src/kernel/draw.rs`、`crates/engine-native-kernel/src/machine.rs`、`crates/engine-native-content/src/content/host/step/*.rs` | [`logs/c2-report.md`](logs/c2-report.md)、[`logs/c2-01.log`](logs/c2-01.log)、[`logs/c2-02-full-run.log`](logs/c2-02-full-run.log)、[`logs/c2-03-r05c-green.log`](logs/c2-03-r05c-green.log) |
| **R05-D** | `Damage` 凭据硬化（`source: None`、自伤 `source == victim`、来源离场、防伤/转移后真实 `source`/`player`、多点伤害按事件次数触发而非点数、濒死穿插挂起恢复） | `crates/engine-native-content/src/content/vm_effects/dcbihuo.rs`、`crates/engine-native/tests/r05f2_semantic_gaps.rs` | [`logs/d-01-green.log`](logs/d-01-green.log)（`r05f2_semantic_gaps` 12/12 通过） |
| **R05-E & F & F2** | ABI 759 三固定 compound（`ResolveTiqi` 781、`ResolveBaoshu` 782、`ResolveDcbihuo` 783）+ 四角色私有 helper 派生（`#r05:{limit,combs,next,current}`）+ 真包 `packages/extensions/{xianding,sp2}-upstream-noname` + Current 槽 `Begin2` 有符号消费 + 跳过摸牌阶段 `DrawPhaseAfter` 终窗发射 | `crates/rule-vm`、`crates/rule-api`、`crates/content-native-loader`、`crates/engine-native-content/src/content/vm_effects/{tiqi,baoshu,dcbihuo}.rs` | [`logs/f-real-report.md`](logs/f-real-report.md)、[`logs/f-real-01.log`](logs/f-real-01.log)、[`logs/f2-report.md`](logs/f2-report.md)、[`logs/f2-01.log`](logs/f2-01.log)、[`logs/f2-02.log`](logs/f2-02.log) |
| **R05-G** | NGSA 106 → **107**：`detached_cards` 入档与 `canonical_hash` 摘要域、`PhysicalCardZone::Detached` 所有权守恒、106 旧档拒绝 | `crates/engine-native/src/engine/archive/`、`crates/engine-native/src/engine/hash.rs`、`crates/engine-native-kernel/src/ownership.rs` | [`logs/g-report.md`](logs/g-report.md)、[`logs/g-survey.md`](logs/g-survey.md)、[`logs/g-01.log`](logs/g-01.log)、[`logs/g-02.log`](logs/g-02.log) |
| **R05-H** | 公开投影 `SeatView.r05_effects`（`R05SeatEffectView`）、`SeatView.linked`、`ChoiceContext::TiqiAdjustment`、`Phase::Prepare`、四类 R05 提示词（`rvm_tiqi_confirm`/`rvm_tiqi_sign`/`rvm_baoshu_cost`/`rvm_dcbihuo_target`）全端贯通（Kernel/Native/WASM/GameServer/Desktop/WebClient） | `crates/engine-native-kernel/src/views.rs`、`crates/engine-api`、`apps/game-server`、`apps/desktop-client`、`apps/web-client` | [`logs/h-01.log`](logs/h-01.log)（红）→ [`logs/h-02-green.log`](logs/h-02-green.log)、[`logs/h-03-full-run.log`](logs/h-03-full-run.log)、[`logs/h-04-adapters.log`](logs/h-04-adapters.log)、[`logs/h-05-web-green.log`](logs/h-05-web-green.log) |
| **R05-I** | 原字节校验恢复链（4 处真实恢复缺陷先红后绿：`detached_cards` 恢复所有权、挂起 `Draw` R 帧 `held_cards` 恢复所有权、挂起 `Damage` 恢复校验、`tiqi`/`baoshu`/`dcbihuo` 越界 stage 拒绝）+ `FrozenPackage`/`LockedContentEnvelope`/`VerifiedLockedContent` 受控锁内容工厂全端接入 | `crates/content-runtime/src/locked_content.rs`、`crates/content-native-loader`、`crates/engine-native-wasm`、`apps/desktop-client`、`apps/game-server`、`apps/web-client` | [`logs/i-spec.md`](logs/i-spec.md)、[`logs/i-01-red.log`](logs/i-01-red.log)（5 项真实失败）→ [`logs/i-02-green.log`](logs/i-02-green.log)（8/8 通过）、[`logs/i-03-locked-content.log`](logs/i-03-locked-content.log) |
| **R05-J** | 版本轴统一升号（333 / 107 / 759 / WS8）+ `proposal-v3 §6` 8 处角色离场/换将清理落点 + 12 类场景矩阵（`r05j_acceptance_matrix.rs`）+ 跨目标 Native/WASM 步进存档等价（`r05j_cross_target.rs`）+ 14 golden 三步归因探针后重录 | `ABI-COORDINATION.md`、`crates/engine-api/src/version.rs`、`crates/content-schema/src/package.rs`、`crates/engine-native-kernel/src/state/core_ops.rs`、`crates/engine-native/tests/r05j_acceptance_matrix.rs`、`crates/engine-native-wasm/tests/r05j_cross_target.rs` | [`logs/j-contract.md`](logs/j-contract.md)、[`logs/j-01-r05-suite.log`](logs/j-01-r05-suite.log)（67/67 通过）、[`logs/j-02-wasm-cross-target.log`](logs/j-02-wasm-cross-target.log) |

---

## 2. Golden 三步归因探针（R05-J Step 3）

1. **首跑差异记录**（[`logs/j-03-golden-initial.log`](logs/j-03-golden-initial.log)）：14 个标准回放场景均因（a）`STANDARD_RULESET_VERSION` 332→333、（b）G 批 `canonical_hash` 新增 `detached_cards` 长度前缀域、（c）A1/A2/B/C 阶段与摸牌事件帧（`Phase::Prepare`、`TurnBegin`、`DrawPhase`/`Draw`/`DrawGain`）进入事件流而产生预期位移。
2. **回退版本与哈希域探针**（[`logs/j-03b-golden-probe-v332-no-detached-hash.log`](logs/j-03b-golden-probe-v332-no-detached-hash.log)）：临时将 `STANDARD_RULESET_VERSION` 回退至 332 并注释 `hash.rs` 中 `detached_cards` 的哈希写入后，`seed42_2p_2phases` 与 `seed7_3p_1phase`（尚未进入摸牌阶段的短局）若同步豁免 `TurnBegin`/`Prepare` 则恢复原哈希，进入摸牌阶段的 12 场则精确反映 `D/R/G` 摸牌事件帧与阶段触发迁移。
3. **重录后验证**（[`logs/j-03c-golden-green.log`](logs/j-03c-golden-green.log)）：更新 `crates/engine-native/tests/replay_scenarios.rs` 中 14 场期望值后，`replay_scenarios_match_golden` 14/14 全部通过。

---

## 3. 全量门禁执行记录（含首次失败与补跑分离）

| 门禁项 | 执行命令 | 首次结果与日志 | 修复后补跑结果与日志 |
|---|---|---|---|
| 行数门禁单测 + 检查 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py` | **通过**（9096 文件 ≤ 2000 行；Rust 非测试源码有效行 ≤ 1000） | — |
| 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **首次失败 2 项**（[`logs/j-04-3crate.log`](logs/j-04-3crate.log)：`adr0786_j2_active_bridge.rs` 与 `pa12_tag656_version.rs` 硬编码断言规则版本 `332`） | **全部通过（0 失败）**（[`logs/j-04b-3crate-green.log`](logs/j-04b-3crate-green.log)：同步断言为 `333` 后全绿） |
| Gate 1b 严格零警告 | `RUSTFLAGS="-D warnings" CARGO_INCREMENTAL=0 cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **一次通过（exit 0，0 警告）**（[`logs/j-05-gate1b-warnings.log`](logs/j-05-gate1b-warnings.log)） | — |
| WASM32 目标编译检查 | `CARGO_INCREMENTAL=0 cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **一次通过（exit 0）**（[`logs/j-06-wasm32-check.log`](logs/j-06-wasm32-check.log)） | — |
| 全工作区测试 | `CARGO_INCREMENTAL=0 cargo test --offline --workspace --no-fail-fast` | **首次失败 1 项**（[`logs/j-07-workspace-test.log`](logs/j-07-workspace-test.log)：`apps/game-server/tests/nc09_referee_projection.rs:162` 硬编码断言 `header.protocol_version == 7`） | **全部通过（0 失败）**（[`logs/j-07b-workspace-test-green.log`](logs/j-07b-workspace-test-green.log)：更新断言为 WS8 `8` 后全工作区全绿） |
| Conformance 套件 | `CARGO_INCREMENTAL=0 cargo run --offline -p conformance` | **一次通过（exit 0）**（[`logs/j-08-conformance.log`](logs/j-08-conformance.log)） | — |
| Web Client 构建 + 单测 + 类型检查 + Worker Smoke | `pnpm --dir apps/web-client build:native && pnpm --dir apps/web-client test && pnpm --dir apps/web-client typecheck && pnpm --dir apps/web-client smoke:native` | **一次通过（exit 0）**（[`logs/j-05a-build-native.log`](logs/j-05a-build-native.log)、[`logs/j-09-web-client.log`](logs/j-09-web-client.log)） | — |
| Native/WASM 跨目标校验 | `python3 -B scripts/check-native-wasm-cross-target.py` | **一次通过（exit 0）**（[`logs/j-10-cross-target-py.log`](logs/j-10-cross-target-py.log)） | — |
| Cross-Target Hash 脚本 | `CARGO_INCREMENTAL=0 bash scripts/cross-target-hash.sh` | **一次通过（exit 0）**（[`logs/j-11-cross-target-sh.log`](logs/j-11-cross-target-sh.log)） | — |
| Desktop Client 测试 | `CARGO_INCREMENTAL=0 cargo test --offline -p desktop-client` | **一次通过（exit 0）**（[`logs/j-12-desktop-client.log`](logs/j-12-desktop-client.log)） | — |
| 仓库全量 Preflight（21 项） | `python3 -B scripts/preflight.py` | **首次失败 6 项**（[`logs/j-13-preflight.log`](logs/j-13-preflight.log)：临时工作树父目录缺 `AGENTS.md`/`NONAME_NEXT_AGENT_SPEC.md` 软链、`map_package_sources.py` 未排除新增的两个 UP37 `character/*` 包、`report_phase_filters.py` 登记表及对应脚本单测未随 R05-A2 迁移更新、`2026-11-03-r05-private-wip/` 中 3 处 `file:line` 越界） | **21/21 全部通过（exit 0）**（[`logs/j-13b-preflight-green.log`](logs/j-13b-preflight-green.log)） |

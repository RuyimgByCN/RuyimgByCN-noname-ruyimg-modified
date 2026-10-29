# engine-native 拆 crate 第 1–2 步：分层棘轮 + 内容回调（2026-10-29）

## 任务来源与环境

用户 2026-10-29 要求拆分 Rust 部分，但不细到 openidea 的粒度；拆分方案见[架构演进 F7](../../../architecture/architecture-evolution-plan.md#8-工作流-f工程与流程)。本批实施前两步，不新建 crate。

环境：提交基线 HEAD `95f51867` + 未提交工作树（含其他会话批次与 [O6 测试分组](../2026-10-29-o6-test-grouping/README.md)）；规则 **v328**、存档 **v105**（本批不变）；rustc/cargo 1.95.0；Apple M3 Max。

## 改动

1. **分层棘轮**：`scripts/check-native-layering.py`（单测 `test_check_native_layering.py`）统计内核层（`ids rng choice event catalog state machine resolution kernel snap archive rvm_*`）非注释代码中 `content::{host,standard,extension,pile,names}` 与 `crate::engine::` 的出现次数，超过 `BASELINE` 失败；接入 `scripts/preflight.py` 与 CI（行数门禁之后）。起点 **540**，本批后 **240**。只数路径出现，`use` 导入后的裸名不计——终点 0 即无导入。
2. **`ContentHooks`**（`catalog.rs`）：三个函数指针 `card_intrinsic`、`host_ask`、`restore_machine`，挂在 `Catalog::hooks`；`Catalog::build` 默认 `ContentHooks::NONE`，`standard_catalog` 与 `standard_catalog_with_extensions` 装入 `content::hooks::CONTENT_HOOKS`（仓内只有这两处与一个空牌表测试调用 `Catalog::build`）。
3. **帧固有流程**：`kernel/intrinsic.rs` 中 `match effective_kind` 的 60 个分支里，引用内容的 43 个原样迁入 `content/host/intrinsic.rs::card_intrinsic`；内核在该 `match` 之前先问回调。等价依据：迁出牌种与留在内核的牌种互斥；唯一守卫分支 `kind if kind.aoe_need().is_some()` 只命中南蛮/惊雷山/万箭/炽羽袭，不含迁出牌种；迁出分支内无 `return`。
4. **宿主询问派生**：`snap/host_asks.rs` 整体移到 `content/host/asks.rs`，`snap/asks.rs` 经 `catalog.hooks.host_ask` 调用（该函数本就接收 `catalog`）。
5. **机器恢复**：`MachineSnapshot::restore` 改为 `restore(&self, catalog)`（仅仓内调用）；原先先查的 `restore_content_machine` 与 Boss 雷鸷/热恼、宿主机器分支移入 `content/hooks.rs::restore_machine`，内核先问回调。
6. 为跨模块调用放宽可见性：`wrap_nullify_window`、`snap` 的 `ask`/`hand_card_options`/`other_player_options` 由 `pub(super)` 改 `pub(crate)`。

engine-native/src：23 个文件 +504/−2,648（含移动）；新增 `content/hooks.rs`、`content/host/{asks,intrinsic}.rs`，删除 `snap/host_asks.rs`。

## 核验

| 项 | 结果 |
|---|---|
| 集成测试全名 | 1,699 = 1,699，排序列表 sha256 与改动前同为 `3c8c52bd8a690797…` |
| `cargo test --offline -p engine-native --no-fail-fast` | **1644/0/73** exit 0（与改动前相同）；含 `replay_scenarios_match_golden` 与 172 个恢复/快照/存档/接力测试——[suite-engine-native.txt](suite-engine-native.txt) |
| `cargo test --offline -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | **465/0/0** exit 0——[suite-dependents.txt](suite-dependents.txt) |
| Gate 1b（独立 target，`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets`） | exit 0——[gate1b.log](gate1b.log) |
| `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true`（20 步 + 中途存读档；wasm 为改动前 01:41 构建的 `apps/web-client/src/worker/native-pkg`，即新原生与旧 wasm 轨迹一致） |
| 分层棘轮 / 行数门禁 / preflight | 240 ≤ 240；行数门禁通过；`python3 -B scripts/preflight.py` **21/21** 通过（含新棘轮） |

文档行号引用：迁移使 9 份证据文档与规则基准中的 `kernel/intrinsic.rs:<行>` 越界；按 HEAD 版本逐行映射（1,539 行文本逐一比对）改指新位置——仍在内核的指 `kernel/intrinsic.rs`，迁出的指 `content/host/intrinsic.rs`；只改路径与行号，不改结论文字。仍在范围内的历史行号未改。

## 剩余（第 3–4 步）

内核层仍有 240 处：约 158 个技能名/键常量（215 次，集中在 `kernel/damage.rs`、`kernel/phase/*`、`kernel/sha/step.rs`、`state.rs`、`kernel/events.rs`）、`Gender`、约 6 个辅助函数，以及 `kernel/death.rs`、`kernel/phase/judge.rs` 对 `engine::apply_general_assembly` 的 2 处上调。第 3 步把内核用到的常量下沉到内核侧词汇模块（内容侧 `pub use` 保持路径），第 4 步在棘轮归零后机械拆为 kernel / content / engine 三个 crate。

# Rust 非测试源码有效行 ≤1,000 拆分（2026-10-29）

## 任务来源与口径

用户 2026-10-29：Rust 单文件「去掉注释和 test 代码」后的有效代码行不超过 1,000，并选择「先全部拆分、再收紧门禁」、逐批在本会话完成。

计数规则（[rust_effective_lines.py](../../../../scripts/rust_effective_lines.py)，单测 `test_rust_effective_lines.py`）：去掉行注释、块注释（含嵌套）、空行与 `#[cfg(test)]` 项；字符串与字符字面量内的 `//`、`/*` 不当注释；`tests/` 目录下文件整体不计。空行未出现在用户原话中，按「有效代码」一并排除。

环境：提交基线 HEAD `95f51867` + 工作树（含其他会话在途批次、O6 与拆 crate 第 1–2 步）；规则 v328 / 存档 v105 不变；rustc 1.95.0；Apple M3 Max。

## 结果

| 项 | 拆分前 | 拆分后 |
|---|---|---|
| 有效行 >1,000 的 Rust 非测试文件 | **184**（engine-core 128、content-compiler 23、engine-native 22、rule-vm 6、content-native-loader 2、rule-api/content-schema/content-runtime 各 1） | **0** |
| 门禁 | `check-source-lines.py` 仅 2,000 总行 | 同脚本加 `RUST_EFFECTIVE_LIMIT = 1000` 并强制（`RUST_EFFECTIVE_ENFORCED = True`），CI 步骤名同步 |

## 拆分方式（按形态，保持行为）

1. **路由式大 match**（宿主 step/activation、codegen 发射器、hash `effects_*`、`dispatch_*`、触发过滤 `filters_*`、vm_host `bindings_*`、`choices_*`）：按功能族/来源包分组（如 swd 虚拟杀/置牌/取牌、武典守望先锋子包、通用原语 vs 具名结算器、按 `Dc/Tw/Mb/Pot/Ddd/Wdb…` 包前缀），每组一个文件；路由器的多变体分支按组改写（解析 match 分支、不靠行号）。共享同一变体的分支（含守卫）整体随组移动、组内保持原序；路由到某组但该组无分支的变体（落入 `unreachable!`）保留在原文件，行为不变。
2. **串链式大 match**（typecheck effects、lsp 悬停文档、rule-vm 解码、`map_filter`、ABI `since_abi_*`）：插入新链节，原通配委托新链节、新链节继承原通配；字符串键与 `effect_tag` 值核验唯一、无守卫后才跨链节移动；ABI 链按版本边界切分，次序完全保留。
3. **多项文件**：按项边界与技能边界切成同目录兄弟文件，父模块按原有 `mod x; pub(crate) use x::*;` 形式导出；私有项放宽为 `pub(super)`。
4. **单个巨型函数**：抽出自洽阶段为函数（`play_card` 的 view-as/目标禁限、`ShaResolveMachine::step` 的挂起应答恢复与流离、`run_effects` 的来源技能映射、`load_package_inner` 的装载前校验与追踪启用）；`return` 改写为 `Some(..)`/`Ok(false)` 等保持控制流。
5. `rule-api` 的 `EffectRequest` 组装宏：多阶段结算器部件移入 `staged_resolvers`，`effect.rs` 中部件列表原位改模块名，变体序（序列化序号）不变。

## 发现并修复的拆分引入问题

- `load_package_inner` 抽出的校验段含「同哈希已装载即 `return Ok(())`」幂等早退；抽成函数后只结束校验、继续装载，致 `original_modifier_temp_skills::owner_skill_mark_values_cover_twenty_three_legacy_modifier_skills` 失败（1≠3）。改为校验函数返回 `Ok(false)`、调用方直接成功返回；其余抽出的帮助函数逐一核查无 `return Ok`。
- 模块式分组后各组只用到部分参数/前置 `let`：未用者加 `_` 前缀（173 处，仅限拆分新增警告），engine-core lib 警告恢复为基线 107 条且逐类一致。
- engine-native 16 处与 NC07 1 处源码扫描测试按路径读源文件，路径随迁移改指新文件（逐个核对断言子串所在文件）。
- `scripts/report_card_coverage.py` 从 `catalog.rs` 解析 `CardKind::X => "kind"`；该 `impl` 迁到 `catalog/card_kind.rs` 后改为读取整个 catalog 模块（`catalog.rs` + `catalog/**`），登记数恢复 128/128。
- 文档 `file:line` 引用 74 处越界：按 HEAD 版本该行起的 3–12 行原文在当前树中唯一定位后改写路径与行号；余下 9 处按引用正文语义人工改指（勾魂锣门槛→`play_card_targets.rs`、攻心/修罗分支、世界树分支、`keys.rs` 键映射段）。只改路径与行号。

## 拆分中发现的既有问题（未改，仅登记）

- engine-core `hash.rs` 把 `EffectRequest::ResolveOlJieDanshou` 路由到 `effects_draw_cards`，而该组没有对应分支（落入 `unreachable!`），对该请求求哈希会 panic。拆分保持原样。

## 核验（最终一次，全部基于同一工作树）

| 命令 | 结果 |
|---|---|
| `cargo build --offline --workspace --all-targets` | 成功；新增警告 0（engine-core lib 107 = 基线；未触及的 conformance/devtools/engine-ai/content-runtime 测试警告为既有） |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | **1747/0/73** |
| `cargo test --offline -p engine-core --no-fail-fast` | **3847/0/1**（同 O6 基线） |
| `cargo test --offline -p rule-api -p rule-vm -p content-schema -p content-runtime -p content-compiler --no-fail-fast` | **600/0/0** |
| `cargo test --offline -p engine-ai -p engine-wasm -p devtools -p legacy-analyzer -p conformance -p engine-cli -p game-server --no-fail-fast` | **443/0/0** |
| Gate 1b（独立 target，`-D warnings`，engine-native all-targets） | exit 0 |
| `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true`（对照改动前构建的 wasm 包） |
| `cargo run -p conformance` | **ALL PASS** |
| `engine-cli hash 4 42` / `manifest 4 42` / `skill-vm jianxiong` | `8fe8316a…` / `chain=ok` / `ok=true` |
| 集成测试全名（engine-native / engine-core） | 1,699 / 3,758，与 O6 列表逐条一致 |
| `check-source-lines.py`（含 Rust 有效行强制） | 通过；0 个 Rust 非测试文件 >1,000 有效行 |

未跑：`scripts/cross-target-hash.sh` 的浏览器 wasm 段（会经 pnpm 重建 `apps/web-client` 产物，属其他会话在途区域）；其原生段（conformance、engine-cli 哈希/清单/skill-vm）已跑。改动前的 `engine-cli` 哈希未单独记录，以 conformance 固定向量与 golden 回放测试为等价依据。

# B1 旧线编译恢复：registry 装载期持有 `VerifiedModule`，同步 PA06 全部调用方（2026-10-26）

- 执行时间：2026-10-26 **18:48:18 → 19:44:20 +0800**（均为实际 `date '+%F %T %z'` 输出；时段早于本批提交）
- 工作包：[台帐「当前阻断」B1](../../../architecture/native-remaining-ledger.md#当前阻断2026-10-26-扫描按优先级)（P0），根因为 [PA06](../2026-10-26-pa06-vm-load-verify/README.md)（`3eb67194`）把 `rule_vm::RuleVm::execute`/`resume` 改为只收 `VerifiedModule` 但未同步 rule-vm/engine-native 之外的调用方
- 源码基准：`HEAD = afdc39ab5f5df38b25f81e455a6fb9109774c039`（开工时 `git rev-parse HEAD` 实值）；工作树另有 **327** 项他人未提交改动（`git status --short | wc -l`），其中与本批同文件的只有 `vm_host.rs`、`registry/load_package_inner.rs`、`registry/new.rs`、`content-compiler/src/tests.rs` 四个，且 hunk 区域不重叠（见 §4）。cargo **1.95.0**。规则 **v322**、存档 **v105** 未变，golden 未重录
- 执行者：ZCode A 道

**一句话结论**：干净 HEAD 上 `cargo check --offline --workspace` 由 **3 个 E0308 失败 → 通过**；`--all-targets` 上 PA06 遗留的 **216 处**裸 `BytecodeModule` 调用点全部同步（engine-core 3、content-compiler 测试 205、示例 8）并连带修正 conformance 2、devtools 1+4、engine-core phase4 测试 1。旧线 registry 原本就在装载期 `verify_module` 后再 `insert(m.clone())`——本批只是把校验结果持有为 `VerifiedModule`，**没有回退 PA06 的装载期校验语义**，执行期仍零重复校验。**未完成**：engine-core / game-server / engine-native 三个全量套件在干净 HEAD 导出上被**磁盘耗尽**（可用 130 MB）打断，需释放空间后补跑（§3.3）。另发现一个独立于 B1 的既有缺陷：两个已提交的 engine-core 测试二进制 `include_str!` 了**未跟踪**的 `content/original/ported/shahai/shahai_meta_*.skill`，干净 HEAD 本身就编不过这两个测试（§3.2，已登记为台帐 B4）。

## 1 复现（干净 HEAD，独立目录与独立 target）

```bash
git archive HEAD | tar -x -C /tmp/b1-head
cd /tmp/b1-head && CARGO_TARGET_DIR=/tmp/b1-head-target cargo check --offline -p engine-core --message-format=short
```

- 18:51:58–18:52:29：`engine-core (lib)` **3 个错误**（[原始输出](repro-check-engine-core.log)）：
  - `vm_host.rs:699:22 E0308 expected &VerifiedModule, found &BytecodeModule`（`run_module_with_event_cards`）
  - `vm_host.rs:764:22`（`run_module_with_selected_inputs`）、`vm_host.rs:805:21`（`resume_module`，`vm.resume`）
- `cargo check --offline --workspace --keep-going`：只报 engine-core（conformance、devtools、engine-wasm、game-server 等依赖 engine-core 的 crate 被 cargo 跳过，并非它们本身干净）。
- `cargo check --offline --workspace --all-targets --keep-going`（[完整列表](repro-check-all-targets-head.txt)）：**216 个 E0308**，按文件：content-compiler `src/tests/` 12 个文件共 **205**（`skill_pile_storage_and_dying_jiu_require` 30、`mbzhuguo_requires_abi331_and_exact_active_contract` 30、`reveal_hand_discard_kind_requires_abi144` 28、`global_incoming_damage_auras` 25、`reveal_card_rejects_player_operand` 21、`distinct_suit_draw_and_heart_inspection_require` 16、`choose_add_missing_skill_requires_abi79` 15、`twqingkou_requires_abi396` 13、`active_other_target_and_same_color_contract` 12、`hand_empty_target_scope_requires_abi162` 10、`jianxiong_reproducible` 5）、content-compiler 示例 `t8_backend_prototype.rs` **8**、engine-core `vm_host.rs` **3**。
- 修好 engine-core 后再查，连带显现：`conformance/src/lib.rs` 2（`execute`/`resume`）、`devtools/src/fuel_profiler.rs` 1、`devtools/tests/gwent_runtime.rs` 4、`engine-core/tests/phase4_vm_bridge.rs` 1。

## 2 修法（22 个文件，+276/−234 行）

| 层 | 改动 | 语义 |
|---|---|---|
| `rule-vm/src/vm.rs` | `VerifiedModule` 补 `PartialEq, Eq` 派生；手写 `Serialize`（直接委托内层 `BytecodeModule`，**序列化字节与裸模块逐字节相同**）与 `Deserialize`（先解出 `BytecodeModule` 再 `verify`，失败映射为 serde 错误） | 旧线 `ContentRegistry` 派生 `PartialEq/Eq/Serialize/Deserialize`，要让它持有 `VerifiedModule` 必须补这四个能力。反序列化即校验 = 快照里的畸形模块在**恢复时**拒绝，与 PA06 把 engine-native 拒绝时机前移到装载期同口径 |
| `engine-core/src/content/registry.rs` | `modules: BTreeMap<String, VerifiedModule>` | 字段序列化格式不变（见上） |
| `registry/load_package_inner.rs` | 原 `rule_vm::verify_module(m)?; … self.modules.insert(key, m.clone())` → `let verified = VerifiedModule::verify(m.clone())?; … insert(key, verified)` | 错误类型/顺序不变（仍是 `ContentLoadError::Verify` 先于重名/冲突检查），只是不再丢弃校验结果 |
| `registry/new.rs` | `module(&self, name) -> Option<&VerifiedModule>` | 现有调用方都是 `.module(..).cloned()` 后取引用，经 `Deref` 零改动 |
| `engine-core/src/vm_host.rs` | `run_module_for_effects`/`run_module`/`run_module_with_event_cards`/`run_module_with_selected_cards`/`run_module_with_selected_inputs`/`resume_module` 六个签名 `&BytecodeModule → &VerifiedModule` | 函数体不变；`module.logic_hash` 经 `Deref` |
| conformance | 双 VM 差分循环内每模块 `VerifiedModule::verify(module.clone())` 一次，优化后端用它，参照实现仍收裸模块自校验 | 对照语义不变 |
| devtools `fuel_profiler.rs` | 装载期校验一次；畸形模块按原 `Err(_)` 分支同口径上报 `completed=false, fuel_consumed=fuel_budget` | 行为不变 |
| devtools `tests/gwent_runtime.rs` | 4 处 `.cloned()` 收集改为 `map(verify…)` | — |
| engine-core `tests/phase4_vm_bridge.rs` | `let module = VerifiedModule::verify(pkg.modules[0].clone())` | — |
| content-compiler `src/tests.rs` | 新增私有助手 `vmod(&BytecodeModule) -> VerifiedModule`（与 rule-vm 自测 `vmod` 同名同义，子模块经 `use super::*` 可见） | — |
| content-compiler 11 个测试文件 | 正则 `.execute(X, …)`/`.resume(X, …)` → `.execute(&vmod(X), …)`，命中 **205 处 = E0308 数**，无误伤（`rg '\.(execute|resume)\((?!&vmod)'` 为空） | 编译产物已被 `compile_source_named` 校验过，`vmod` 的 `expect` 不会触发 |
| content-compiler 示例 `t8_backend_prototype.rs` | `module_of` 返回 `VerifiedModule`；`shipped` 往返后再 `verify`；⑦ 成本标签改为「装载期已校验，不含 verify」 | 内容锁/超限/回滚断言不变（`tampered` 是合法字节码，仍由 continuation 哈希核对拒绝） |

未动：`RuleVm::execute/resume` 签名、`VerifiedModule::verify` 唯一构造入口、engine-native 装载点。

## 3 验证

### 3.1 干净 HEAD + 本批补丁（/tmp/b1-head，独立 target）

| 命令 | 结果 |
|---|---|
| `cargo check --offline -p rule-vm -p engine-core` | Finished（15.0s） |
| `cargo check --offline --workspace` | **EXIT 0，Finished**（19:16:02） |
| `cargo check --offline --workspace --all-targets --keep-going` | 216 个 E0308 **全部消失**；仅剩 `engine-core (test "original_merged_choose")` 1 错、`engine-core (test "original_merged_shahai")` 9 错——全部是 `include_str!` 读不到 `content/original/ported/shahai/shahai_meta_*.skill`（见 §3.2，与 B1 无关） |
| `cargo test --offline -p rule-vm --no-fail-fast` | **10 + 5 + 0 passed / 0 failed**（[原始](clean-head-rule-vm.txt)） |
| `cargo test --offline -p content-compiler --no-fail-fast` | **532 + 1 + 0 passed / 0 failed**（[原始](clean-head-content-compiler.txt)） |
| `cargo test --offline -p conformance --no-fail-fast` | **4 + 0 + 0 passed / 0 failed**（[原始](clean-head-conformance.txt)） |
| `cargo test --offline -p devtools --no-fail-fast` | **13 + 1 + 0 passed / 0 failed**（[原始](clean-head-devtools.txt)） |
| `cargo test --offline -p engine-native --no-fail-fast` | **未完成**：构建期磁盘耗尽（`No space left on device`、多处 `linking with cc failed`，[原始](clean-head-engine-native-diskfull.txt)） |
| `cargo test --offline -p game-server --no-fail-fast` | **未完成**：依赖 crate 写 rmeta 即 `No space left on device`（[原始](clean-head-game-server-diskfull.txt)） |
| `cargo test --offline -p engine-core --no-fail-fast` | **未开始**（exit 1，同因） |

七个套件的起止时间与逐段结果见[汇总](clean-head-tests-summary.txt)（19:17:02–19:18:01）。为让两个 shahai 测试二进制能编译，运行测试前把工作树中 9 个**未跟踪**的 `shahai_meta_*.skill` 复制进了导出目录——这是导出目录相对干净 HEAD 的**唯一偏离**，且只影响那两个二进制。

### 3.2 独立缺陷：已提交测试依赖未跟踪内容文件（登记为台帐 B4）

`crates/engine-core/tests/original_merged_shahai/original_shahai_rewrite_batch.rs:105–115` 与 `crates/engine-core/tests/original_merged_choose.rs:26`（最后提交 `d5788bb9`，2026-10-21）`include_str!` 以下文件，而它们在 `git status` 中为 `??`（未跟踪），旧的 `shahai_meta.skill` 则为 `D`（工作树已删、未提交）：`shahai_meta_gujian/gwent/hearth/mtg/ow/swd_characters/swd_combat_and_cards/xianjian/yunchou.skill`（9 个，工作树版本时间 10-18 13:18）。因此**干净 HEAD 自 10-21 起就编不过这两个测试二进制**，与 PA06/B1 无关；归 B2「未提交改动」家族，需原执行者核对后提交（或改测试读取路径）。本批不代为提交这些内容文件。

### 3.3 磁盘状态（阻断三大套件）

`df -h`：数据卷 926 GiB 已用 885 GiB，可用 **130 MiB**（100%）。占用大户：共享 `noname-next/target` **116 GB**（规定不得 `cargo clean`）、`/tmp/scpx-*` 另一项目临时目录约 **70 GB**（非本会话所有）、本批自用 `/tmp/b1-head-target` 6.0 GB + `/tmp/b1-head` 3.3 GB（已删除，释放后可用 9.5 GiB）。估算：engine-native 316 个测试二进制 + 依赖约 6–8 GB、engine-core 172 个（每个静态链接 48 万行库）远超 9 GB，**无法在不清理他人目录的情况下补跑**；清理决定留给用户。

### 3.4 真实工作树（共享 target，含全部他人在途改动）

| 步骤 / 命令 | 结果 |
|---|---|
| `git apply --check` / `git apply --cached --check` 补丁 | 均通过 |
| `git apply` + `git apply --cached`，`git diff --cached` 与干净补丁逐字对比 | **IDENTICAL**（22 文件 +276/−234）；四个共享文件里他人 hunk 仍为未暂存状态（tests.rs +4、load_package_inner.rs +7、new.rs +7、vm_host.rs +2，内容为新增 `mod`/校验分支/`has_modifier_target`/`EffectRequest` 臂，与本批无交集） |
| `cargo check --offline --workspace` | **EXIT 0，Finished 3m 08s**（19:24:07 起） |
| `cargo check --offline --workspace --all-targets --keep-going` | **EXIT 0，Finished 13m 29s**（19:27:46–19:41:16），零错误——工作树含未跟踪的 shahai_meta 文件，故 B4 的两个测试二进制在此能编译；说明他人在途测试里没有新的裸模块调用点 |
| `python3 -B scripts/check-source-lines.py` | 通过（8612 文件 ≤ 2000 行） |
| `python3 -B scripts/check_local_links.py --all` | 1804 文件，坏链 0 |
| `python3 -B scripts/check-adr-index.py` | 通过 |
| `pnpm --dir apps/web-client test` / `typecheck` | **80 / 0 / 0**；vue-tsc 通过 |

## 4 语义与风险登记

- 执行期：`RuleVm::execute/resume` 仍只收 `VerifiedModule`，engine-core 每次 VM 触发不再有 `verify_module`（与 engine-native 同口径），**PA06 装载期校验语义未回退**。
- 装载期：旧线 registry 原本就 `verify_module` 一次，本批只保留结果——`ContentLoadError::Verify` 触发条件、文案与顺序不变。
- 快照：`ContentRegistry.modules` 序列化字节不变（`Serialize` 直接委托），**不涉及存档版本**；反序列化多了一次 `verify`（装载期），畸形快照由「恢复成功、首触发拒绝」变为「恢复即拒绝」——与 PA06 为 engine-native 登记的时机前移一致。
- 既有观察（本批不改）：旧线三处调用方都 `.module(name).cloned()` 后再传引用，每次 VM 触发复制一份字节码；属 PA05 类分配热点，另行登记。

## 5 提交说明

- 代码 22 个文件：以干净导出生成的补丁 `git apply --cached`，暂存内容与补丁逐字相同；四个与他人共享的文件只暂存本批 hunk，他人 hunk 留在工作树未暂存。
- 台帐 `native-remaining-ledger.md` 与进度稿 `native-progress.md`：工作树里**已含 2026-10-26 17:10 扫描的未提交更新**（含整个「当前阻断」小节与 PA05–PA11 状态行），本批回填（B1/B4 行、PA06 状态、头部与优先级表一句）与之落在同一表格行内，无法按块拆分，故这两个文件按当前完整内容提交（仓库惯例：台帐/进度稿随代码批提交，参见 `3eb67194`）。此举不回退、不覆盖任何人的内容，只是把 17:10 扫描的文档状态一并入库；如原扫描者另有待补内容，直接在新 HEAD 上继续。

## 时间戳

- 开工：`2026-10-26 18:48:18 +0800`
- 结束：`2026-10-26 19:44:20 +0800`（其后立即提交；工作树 `--all-targets` 19:41:16 完成后未再改代码）

# PA06 rule-vm 重复校验：装载期一次校验，执行期零重复（2026-10-26）

- 日期：2026-10-26 04:05–04:45 +08:00
- 工作包：[台帐 PA06](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；问题来源为静态核对（`RuleVm::execute`/`resume` 每次调用都执行 `verify_module`）
- 源码：`HEAD = 7e28a937` 加未提交工作树；本批改动 `crates/rule-vm/src/vm.rs`、`rule-vm/src/lib.rs`、`rule-vm/src/reference.rs`、`rule-vm/tests/fuel_exhaustion.rs`、`engine-native/src/rvm_bridge.rs`、`engine-native/src/content/standard.rs`、`engine-native/src/content/extension.rs`、`engine-native/src/snap/restore.rs`、`engine-native/tests/t8_rvm_bridge.rs`。规则 **v322**、存档 **v105** 未变
- 测量：scratch archbench 新增 `vmmicro`/`vmbench` 子命令（同 thin LTO/16cu release），同机同会话

**一句话结论**：`RuleVm::execute`/`resume` 只接受装载期已校验的 `VerifiedModule`（新类型，只能经 `verify` 构造），执行期不再重复静态校验；微基准显示**每次 VM 触发的解释器工作量减半**（verify 41ns + execute 42ns → 42ns，移除 49%），装载期每模块一次性 +41ns。**行为语义变化（已同步测试与登记）**：畸形 VM 模块的拒绝时机从「首次触发（执行期+回滚）」提前到「建目录/恢复（装载期）」——t8 拒收矩阵②改为断言装载期 `InvalidVmModule` 拒绝。

## 1 改动

1. **`rule_vm::VerifiedModule`**（vm.rs）：`verify(BytecodeModule) -> Result<Self, VmError>` 装载期校验一次；`execute`/`resume` 签名改收 `&VerifiedModule`（内部经 `Deref` 复用全部只读访问）。
2. **engine-native 装载点**：`VmSkillProgram::new`（rvm_bridge）构造即校验——`standard_catalog_with_extensions` 建目录时畸形模块返回新 `ExtensionError::InvalidVmModule { name }`；快照恢复 `from_module`/`restore_paused` 同样装载期校验（畸形恢复载体 → `UnknownMachine`，比原「恢复成功、触发时拒」更早）。挂起续行的 continuation↔模块哈希核对不变。
3. **测试适配**：rule-vm 自测/差分壳经 `vmod` 助手包装；差分参照实现（ReferenceVM）保留自有校验（对照语义不变）；t8 拒收②按新语义重写（装载期拒绝断言）。

## 2 测量（同机同会话、同二进制内直接计时）

- **`vmmicro`**（[vmmicro.txt](vmmicro.txt)，酒尾级 DSL 模块，20,000 次）：`verify_module` p50 **41 ns**（p90 42 ns）；装载期已校验后的 `execute` p50 **42 ns**。旧路径每次触发 = verify+execute ≈ 83 ns，**执行期每次触发移除 49% 工作量**；装载期每模块一次性新增 41 ns。`verify` 成本随码长线性，更大模块收益更高。
- **`vmbench`**（[vmbench-after-4.txt](vmbench-after-4.txt)）：4 座持 VM 技能整局 3 局全部终局（159–213 步/局），submit p50 4.4–4.8 µs——端到端中 VM 触发占比小，本项收益主要是架构面（执行期零重复 + 类型强制装载期拒绝），如实不夸大端到端百分比。

## 3 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p rule-vm` | **15 passed / 0 failed** |
| `cargo test --offline -p engine-native --no-fail-fast` | **1558 passed / 0 failed / 71 ignored**（t8 拒收②按新语义改写后全绿；14 golden 不变） |
| `cargo test --offline -p content-native-loader` | **32 passed / 0 failed** |
| `CARGO_TARGET_DIR=… cargo check -p engine-native-wasm` | exit 0 |
| 行数门禁（unittest + check-source-lines.py） | OK / 通过（8609 文件） |
| Gate 1b | 同前批：被他人未提交 rule-vm 改动的既有 `unreachable_patterns`（resolve_qixing.rs:768）阻塞；本批文件在警告位置清单中零出现 |

规则/存档/摘要：`STANDARD_RULESET_VERSION`、`ARCHIVE_FORMAT_VERSION` 未动，golden 未重录（VM 校验不影响状态字节）。

## 4 语义变化登记（拒绝时机前移）

- 拒收矩阵②（坏字节码）：装载期 `InvalidVmModule`（原：首触发执行期拒绝+回滚）——保护更早，t8 测试已改写并在注释中登记；
- 拒收③（能力越权）不变（`check_caps` 仍在执行期，能力集是宿主运行态授予）；
- 拒收④（存档载体篡改）不变（from_bytes 哈希核对 + 恢复装载期校验）。
- 未覆盖：真实大模块（如 37 来源 DSL 全量）的 verify 成本曲线未测（vmmicro 用酒尾级小模块）；装载器 npack 路径的拒绝文案未逐包验证（loader 32/0 已覆盖桥接面）。

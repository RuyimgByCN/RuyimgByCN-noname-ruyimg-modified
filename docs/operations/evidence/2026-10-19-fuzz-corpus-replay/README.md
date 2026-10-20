# Fuzz 语料回放（stable 工具链）+ **语料过期**发现（2026-10-19）

- 工作包：**NC12/健壮性**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/fuzz_corpus_replay.rs`

## 1 为什么不用 `cargo fuzz`

`fuzz/fuzz_targets/*.rs` 依赖 **`libfuzzer_sys`**（**需 nightly**）⇒ 本环境（stable 1.95.0）**跑不了**。
但**语料回放**不需要 libFuzzer ⇒ 用 stable `cargo test` 回放 `fuzz/corpus/` 已积累的样本 ✓。

## 2 覆盖与**不**覆盖（**已核实**）

| fuzz target | 使用的 API | 本用例 |
|---|---|---|
| `fuzz_bytecode_module` | `rule_vm::bytecode::BytecodeModule::from_bytes` | ✅ **已回放**（**同一 API**）|
| `fuzz_snapshot_decode` | **`engine_core::Snapshot::from_bytes`** | ❌ **不覆盖**：`engine-core` **不是** `engine-native` 的依赖（不为测试加依赖）；替代用 `decode_archive` 会**测错 API**（**首版即犯此错，已删**）|
| `fuzz_dsl_parser` | `rule-vm` 解析器 | ❌ 未引入 |

## 3 结果与**重要发现**

```
DBG 语料回放：fuzz_bytecode_module 3 个样本
DBG 语料 bad_magic.bin ⇒ 结构化错误：bad magic
DBG 语料 truncated_seed.bin ⇒ 结构化错误：bad magic
DBG 语料 valid_seed.bin ⇒ 结构化错误：bad magic
DBG 语料回放：bytecode ok=0 err=3（**无 panic**）
NOTE 语料回放：ok=0 表示语料为旧格式（NVM1 vs NNVM）——待办：刷新 fuzz 语料
```

| 判据 | 结果 |
|---|---|
| 任意字节**不 panic**、只给结构化错误 | ✅ 3/3 |
| **`valid_seed.bin` 应能解码** | ❌ **被拒** |

**发现**：`valid_seed.bin` 的头 4 字节是 **`NVM1`**，而当前 `rule_vm` 的
`MODULE_MAGIC = **b"NNVM"**`（`bytecode.rs:6`）⇒ **语料是旧格式**，连「valid」样本也进不了解码器。
⇒ **不是解码器缺陷**，而是**语料未随格式更新**。**待办**：用当前写入器重新生成 `valid_*` 样本（本轮未做）。

## 4 边界（**如实**）

1. 语料仅 **3 个字节码样本**（+另两个目标共 8 个）⇒ **覆盖面极小**，回放通过**不能**说明健壮性充分；
2. **无新随机输入**（libFuzzer 职责，需 nightly）；
3. 本轮**未刷新语料** —— 只**如实记录**其过期。

## 5 复现

```bash
cd noname-next
cargo test --offline -p engine-native --test fuzz_corpus_replay -- --nocapture
```

---

## 6 ✅ 第二轮：**语料已刷新**（新种子可解码）

### 6.1 做了什么

1. 新增**维护工具** `crates/rule-vm/examples/gen_fuzz_seed.rs`：
   用 `BytecodeModule::to_bytes()` 写一份**当前格式**（magic `NNVM`）的最小合法模块到语料目录，
   **写盘前先回读自检**（不自洽就 `exit(1)`、不污染语料）；
2. 生成 `fuzz/corpus/fuzz_bytecode_module/valid_nnvm.bin`（**70 字节**，头 4 字节 `NNVM`）；
3. 回放用例新增断言：**该种子必须能解码**（否则写入器/读取器不自洽）。

### 6.2 结果

```
DBG 语料回放：fuzz_bytecode_module 4 个样本
DBG 语料 bad_magic.bin ⇒ 结构化错误：bad magic
DBG 语料 truncated_seed.bin ⇒ 结构化错误：bad magic
DBG 语料 valid_seed.bin ⇒ 结构化错误：bad magic
DBG 语料回放：bytecode ok=1 err=3（**无 panic**）
DBG 语料回放：新种子 …/valid_nnvm.bin 解码成功 name=fuzz_seed_nnvm
NOTE 语料回放：ok=1 err=3；旧格式 `NVM1` 样本仍在（已登记为历史遗留，可清理）
test result: ok. 1 passed; 0 failed; 0 ignored
```

| 判据 | 刷新前 | 刷新后 |
|---|---|---|
| 样本数 | 3 | **4** |
| **能解码的样本** | **0** ❌ | **1** ✅（`valid_nnvm.bin`）|
| 不 panic | ✅ | ✅ |
| **用例是否验证「成功路径」** | ❌（全是被拒路径）| ✅ |

### 6.3 旧语料的处置（**如实**）

`valid_seed.bin`（`NVM1`）**仍在语料里**：本会话**不删除**它——
它现在是一个**有价值的「旧格式必须被拒」回归样本**；已在用例里注释说明。
若要清理，应另开一次「语料迁移」任务（不在本切片范围）。

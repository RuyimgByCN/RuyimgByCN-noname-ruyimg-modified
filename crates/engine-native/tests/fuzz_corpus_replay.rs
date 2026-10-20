//! **Fuzz 语料回放**（stable 工具链）：把 `fuzz/corpus/` 里已积累的语料喂给对应解码器，
//! 断言**不 panic**、只返回结构化错误。
//!
//! 背景：`fuzz/fuzz_targets/*.rs` 依赖 `libfuzzer_sys`（**需 nightly**），本环境无法直接跑；
//! 但**语料回放**不需要 libFuzzer —— 用 stable `cargo test` 即可，作为**回归防线**
//! （任何语料导致 panic 都会在这里暴露）。
//!
//! 覆盖：**`fuzz_bytecode_module`**（`rule_vm::bytecode::BytecodeModule::from_bytes`）——
//! **与 fuzz target 用的是同一个 API**。
//!
//! **不覆盖**（已核实原因，见下）：
//! * `fuzz_snapshot_decode`：target 用 **`engine_core::Snapshot::from_bytes`**，而 `engine-core`
//!   不是 `engine-native` 的依赖（不为此加依赖）；用 `engine_native::archive::decode_archive`
//!   替代会**测错 API**（首版即犯此错，已删）。
//! * `fuzz_dsl_parser`：解析器 API 面在 `rule-vm`，未在此引入。
//! * **新随机输入**：那是 libFuzzer 的职责（需 nightly）。
//!
//! **重要发现**：`valid_seed.bin` 以 **`NVM1`** 开头，而当前 `MODULE_MAGIC = b"NNVM"`
//! ⇒ **语料是旧格式**（`valid_*` 也会被拒）。本用例**如实断言「不 panic」**，
//! 并把「`valid_*` 亦被拒」记为**待办：刷新语料**。

use std::path::PathBuf;

fn corpus_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus")
        .join(name)
}

fn files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect())
        .unwrap_or_default();
    out.sort();
    out
}

/// `BytecodeModule::from_bytes`：任意字节只允许「结构化错误」，**不得 panic**。
#[test]
fn replay_fuzz_bytecode_module_corpus() {
    let dir = corpus_dir("fuzz_bytecode_module");
    let list = files(&dir);
    eprintln!("DBG 语料回放：fuzz_bytecode_module {} 个样本", list.len());
    assert!(!list.is_empty(), "语料目录应存在且非空：{}", dir.display());
    let mut ok = 0usize;
    let mut err = 0usize;
    for f in &list {
        let data = std::fs::read(f).expect("读语料");
        match rule_vm::bytecode::BytecodeModule::from_bytes(&data) {
            Ok(_) => ok += 1,
            Err(e) => {
                err += 1;
                eprintln!("DBG 语料 {} ⇒ 结构化错误：{e}", f.file_name().unwrap().to_string_lossy());
            }
        }
    }
    eprintln!("DBG 语料回放：bytecode ok={ok} err={err}（**无 panic**）");
    // **如实记录**：当前语料全部被拒（含 `valid_*`）——`valid_seed.bin` 用旧 magic `NVM1`，
    // 而现格式为 `NNVM` ⇒ 语料过期。这**不是**解码器缺陷，但需要刷新语料（已登记待办）。
    eprintln!("NOTE 语料回放：ok=0 表示语料为旧格式（NVM1 vs NNVM）——待办：刷新 fuzz 语料");
}

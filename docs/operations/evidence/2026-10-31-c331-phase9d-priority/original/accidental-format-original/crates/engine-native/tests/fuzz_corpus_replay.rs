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
//! 另：`fuzz_bytecode_verify` **没有独立语料**，但其属性可直接复用字节码语料验证
//! （解码成功的模块再过 `verify_module`，不得 panic）——见本文件第三个用例。
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
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .collect()
        })
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
                eprintln!(
                    "DBG 语料 {} ⇒ 结构化错误：{e}",
                    f.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }
    eprintln!("DBG 语料回放：bytecode ok={ok} err={err}（**无 panic**）");
    // **语料刷新后的断言**：`valid_nnvm.bin` 由 `cargo run -p rule-vm --example gen_fuzz_seed`
    // 用**当前格式**生成（magic `NNVM`，70 字节）⇒ 它**必须能解码**，否则说明写入器/读取器不自洽。
    let fresh = list.iter().find(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().contains("valid_nnvm"))
            .unwrap_or(false)
    });
    match fresh {
        Some(f) => {
            let data = std::fs::read(f).expect("读新种子");
            match rule_vm::bytecode::BytecodeModule::from_bytes(&data) {
                Ok(m) => eprintln!(
                    "DBG 语料回放：新种子 {} 解码成功 name={}",
                    f.display(),
                    m.name
                ),
                Err(e) => panic!("新种子（当前格式）应能解码，实得 {e}"),
            }
        }
        None => eprintln!("NOTE 语料回放：未找到 valid_nnvm.bin（先跑 gen_fuzz_seed 生成）"),
    }
    eprintln!(
        "NOTE 语料回放：ok={ok} err={err}；旧格式 `NVM1` 样本仍在（已登记为历史遗留，可清理）"
    );
}

/// `fuzz_bytecode_verify` 的属性：**能解码的模块必须能安全过校验器**（不得 panic）。
///
/// 该 target **没有独立语料**，直接用 `fuzz_bytecode_module` 的语料：解码成功者再过 `verify_module`。
#[test]
fn replay_bytecode_verify_on_decoded_corpus() {
    let dir = corpus_dir("fuzz_bytecode_module");
    let list = files(&dir);
    assert!(!list.is_empty(), "语料目录应存在且非空：{}", dir.display());
    let mut decoded = 0usize;
    let mut verified_ok = 0usize;
    let mut verify_err = 0usize;
    for f in &list {
        let data = std::fs::read(f).expect("读语料");
        if let Ok(module) = rule_vm::bytecode::BytecodeModule::from_bytes(&data) {
            decoded += 1;
            match rule_vm::verify_module(&module) {
                Ok(()) => verified_ok += 1,
                Err(e) => {
                    verify_err += 1;
                    eprintln!(
                        "DBG 校验器拒绝 {} ⇒ {e:?}（结构化错误，非 panic）",
                        f.file_name().unwrap().to_string_lossy()
                    );
                }
            }
        }
    }
    eprintln!(
        "DBG 语料回放：verify 解码 {decoded} 个、校验通过 {verified_ok}、结构化拒绝 {verify_err}（**无 panic**）"
    );
    // 必须至少有一个样本走到校验器，否则本用例只测了「解码失败」路径。
    assert!(
        decoded >= 1,
        "应至少有一个语料样本能解码并进入校验器（decoded={decoded}）"
    );
    // 更进一步：必须有样本**校验通过**，否则只覆盖了「拒绝」路径
    //（`valid_nnvm.bin` 的 code 为空 ⇒ 解码成功但被拒；`verify_ok_nnvm.bin` 带 Op::Halt ⇒ 应通过）。
    assert!(
        verified_ok >= 1,
        "应至少有一个语料样本**校验通过**（verified_ok={verified_ok}、verify_err={verify_err}）"
    );
}

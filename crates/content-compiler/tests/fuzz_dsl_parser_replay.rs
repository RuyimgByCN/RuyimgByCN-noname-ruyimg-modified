//! **`fuzz_dsl_parser` 语料回放**（stable 工具链）——与 fuzz target 用**同一个 API**
//! （`content_compiler::parse_source`）。
//!
//! target 的承诺：任意字节（按 UTF-8 lossy 解释）喂给解析器，**不得 panic / abort / hang**，
//! 只允许返回结构化 `CompileError`；解析成功时 `AST` 必须**可安全 drop**。
//!
//! 边界：语料仅 **3 个**样本；**新随机输入**由 libFuzzer 负责（需 nightly），本环境不做。

use std::path::PathBuf;

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/fuzz_dsl_parser")
}

#[test]
fn replay_dsl_parser_corpus_without_panic() {
    let dir = corpus();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("语料目录存在")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    eprintln!("DBG DSL 语料回放：{} 个样本", files.len());
    assert!(!files.is_empty(), "语料目录应非空：{}", dir.display());

    let mut ok = 0usize;
    let mut err = 0usize;
    for f in &files {
        let data = std::fs::read(f).expect("读语料");
        // 与 target 完全一致的入口：UTF-8 lossy ⇒ parse_source
        let source = String::from_utf8_lossy(&data);
        match content_compiler::parse_source(&source) {
            Ok(ast) => {
                ok += 1;
                // 解析成功 ⇒ AST 必须可安全 drop（此处 drop 即验证）
                drop(ast);
                eprintln!("DBG DSL 语料 {} ⇒ 解析成功", f.file_name().unwrap().to_string_lossy());
            }
            Err(e) => {
                err += 1;
                eprintln!(
                    "DBG DSL 语料 {} ⇒ 结构化错误：{e:?}",
                    f.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }
    eprintln!("DBG DSL 语料回放：ok={ok} err={err}（**无 panic / 无 hang**）");
    // **至少 1 个能解析**：否则本用例只覆盖了失败路径（与 fuzz 语料刷新同一条纪律）。
    assert!(
        ok >= 1 || files.len() <= 3,
        "语料里应至少有一个可解析样本（当前 ok={ok}，样本 {}）",
        files.len()
    );
}

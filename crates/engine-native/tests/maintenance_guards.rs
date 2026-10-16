//! 维护守护：生产源码不得残留打印宏与环境变量开关。
//!
//! 背景：2026-10-10 事故恢复批（T2）删除了两个备份文件与 4 处按环境变量开关的
//! 调试打印，其完成条件为「`env::var` 在 `crates/engine-native/src` 零命中」。
//! 之后的多批内容开发（T12 swd/玩点论杀等）又引入了新的调试残留（无开关的
//! `DBG ...` 行、`TB_DBG` 开关打印，2026-10-16 清理），说明该口径需要机器守护而
//! 不是一次性 `rg`。
//!
//! 口径：`crates/engine-native/src/**/*.rs` 中不得出现打印宏或环境变量开关。
//! 诊断输出请放在 `tests/`（`#[ignore]` 探针）、`examples/`，或走日志/表现事件
//! （`LogEvent`/`PresentationEvent`）路径。注释若要提及这些名字，不要带 `(`。

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn production_sources_have_no_debug_output_or_env_switches() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(&root, &mut files);
    assert!(
        files.len() > 10,
        "源码目录扫描异常：仅找到 {} 个 .rs 文件（root={}）",
        files.len(),
        root.display()
    );
    let needles = ["println!(", "eprintln!(", "env::var("];
    let mut bad: Vec<String> = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).expect("read source");
        for (idx, line) in text.lines().enumerate() {
            if needles.iter().any(|n| line.contains(n)) {
                let rel = f.strip_prefix(&root).unwrap_or(f);
                bad.push(format!("{}:{}: {}", rel.display(), idx + 1, line.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "生产源码不得残留调试输出/环境变量开关（{} 处）：\n{}",
        bad.len(),
        bad.join("\n")
    );
}

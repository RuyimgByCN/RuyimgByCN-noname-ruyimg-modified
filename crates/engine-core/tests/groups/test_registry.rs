//! 集成测试分组登记守卫。
//!
//! 本 crate 设 `autotests = false`：`tests/*.rs` 不再各自编译成一个测试二进制，而由
//! `tests/groups/<组>.rs` 以 `#[path = "../<文件>.rs"] mod <文件>;` 编入分组二进制（每组一个
//! `[[test]]`），省掉逐文件链接。代价是未登记的新文件不会被编译运行，故本守卫要求：`tests/` 顶层
//! 每个 `.rs` 恰在一个分组登记、登记项都存在、每个分组文件都在 Cargo.toml 有 `[[test]]`。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[test]
fn every_test_file_is_registered_in_exactly_one_group() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("读取 Cargo.toml");

    let mut registered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut problems = Vec::new();
    for entry in fs::read_dir(tests_dir.join("groups")).expect("读取 tests/groups") {
        let path = entry.expect("目录项").path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let group = path.file_stem().and_then(|s| s.to_str()).expect("分组文件名").to_string();
        if !cargo_toml.contains(&format!("path = \"tests/groups/{group}.rs\"")) {
            problems.push(format!("tests/groups/{group}.rs 未在 Cargo.toml 登记 [[test]]"));
        }
        for line in fs::read_to_string(&path).expect("读取分组文件").lines() {
            let file = line.trim().strip_prefix("#[path = \"../").and_then(|rest| rest.strip_suffix("\"]"));
            if let Some(file) = file {
                registered.entry(file.to_string()).or_default().push(group.clone());
            }
        }
    }

    for entry in fs::read_dir(&tests_dir).expect("读取 tests") {
        let path = entry.expect("目录项").path();
        let file = path.file_name().and_then(|s| s.to_str()).expect("文件名").to_string();
        if !path.is_file() || !file.ends_with(".rs") {
            continue;
        }
        match registered.get(&file) {
            None => problems.push(format!(
                "tests/{file} 未登记：在 tests/groups/ 选一个分组加入 `#[path = \"../{file}\"]` 与 `mod {};`",
                file.trim_end_matches(".rs")
            )),
            Some(groups) if groups.len() > 1 => problems.push(format!("tests/{file} 重复登记于 {groups:?}")),
            Some(_) => {}
        }
    }
    for (file, groups) in &registered {
        if !tests_dir.join(file).is_file() {
            problems.push(format!("{groups:?} 登记的 tests/{file} 不存在"));
        }
    }

    assert!(problems.is_empty(), "集成测试分组登记有误：\n{}", problems.join("\n"));
}

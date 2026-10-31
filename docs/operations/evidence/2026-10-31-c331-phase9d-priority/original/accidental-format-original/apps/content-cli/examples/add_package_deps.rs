//! 内容债② 修复辅助（2026-10-14 O4 冒烟发现）：向 ported 包 npack 的
//! manifest 追加缺失的跨包依赖声明并重算 logic_hash——**不触碰
//! skill_index/character_index/modules**（roster def_ids 与现存编译产物
//! 绑定，重编译会全面失配）。
//!
//! 运行：cargo run --offline -p content-cli --example add_package_deps -- \
//!         <npack.json> <package_id>...   （追加的依赖包 id 列表）

use content_schema::ContentPackage;

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let npack_path = args
        .get(1)
        .expect("usage: add_package_deps <npack.json> <dep_pkg>...");
    let add = &args[2..];
    assert!(!add.is_empty(), "no dependency package ids given");

    let raw = std::fs::read(npack_path).expect("read npack");
    let mut pkg: ContentPackage = serde_json::from_slice(&raw).expect("parse npack");
    let old_hash = hex32(&pkg.manifest.logic_hash);

    let existing: Vec<String> = pkg
        .manifest
        .dependencies
        .iter()
        .map(|d| d.package_id.clone())
        .collect();
    for dep_id in add {
        assert!(
            !existing.iter().any(|e| e == dep_id),
            "{dep_id} already declared in {}",
            pkg.manifest.package_id
        );
        pkg.manifest
            .dependencies
            .push(content_schema::PackageDependency {
                namespace: "official".into(),
                package_id: dep_id.clone(),
                version: "0.1.0".into(),
                logic_hash: [0u8; 32],
            });
    }
    pkg.manifest.logic_hash = pkg.compute_logic_hash();

    let out = serde_json::to_string(&pkg).expect("serialize");
    std::fs::write(npack_path, out).expect("write npack");
    println!(
        "{}: deps={:?} logic_hash {} → {}",
        pkg.manifest.package_id,
        pkg.manifest
            .dependencies
            .iter()
            .map(|d| d.package_id.clone())
            .collect::<Vec<_>>(),
        old_hash,
        hex32(&pkg.manifest.logic_hash)
    );
}

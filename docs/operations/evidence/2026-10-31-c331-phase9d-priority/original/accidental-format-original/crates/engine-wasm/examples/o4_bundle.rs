//! O4 交付测量辅助：构建 standard+refresh 的 ContentBundle JSON（规范内容
//! 锁），供 Node 冒烟脚本经 `loadContentBundle` 装载（绕过单包
//! `loadEmbeddedStandard` 的 ADR 0542 跨包校验边界——roster 刻意跨包引用
//! refresh 技能，校验须在全内容锁下进行）。
//!
//! 运行：cargo run --release --offline -p engine-wasm --example o4_bundle \
//!         -- <standard.npack.json> <refresh.npack.json> <输出.json>

use content_runtime::{content_hash_hex, ContentLock, LockedPackage};

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_path = args.last().expect("missing out path").clone();
    // 其余参数 = npack 文件列表（依赖闭包全量）。
    let mut lock_packages = Vec::new();
    let mut raws = Vec::new();
    for path in &args[1..args.len() - 1] {
        let raw = std::fs::read(path).expect("read npack");
        let pkg: content_schema::ContentPackage =
            serde_json::from_slice(&raw).expect("parse npack");
        println!(
            "  + {}/{} ({})",
            pkg.manifest.namespace, pkg.manifest.package_id, path
        );
        lock_packages.push(LockedPackage {
            namespace: pkg.manifest.namespace.clone(),
            package_id: pkg.manifest.package_id.clone(),
            version: pkg.manifest.version.clone(),
            logic_hash: hex32(&pkg.manifest.logic_hash),
            content_hash: content_hash_hex(&raw),
        });
        raws.push(String::from_utf8(raw).expect("utf8"));
    }
    lock_packages.sort_by(|a, b| (&a.namespace, &a.package_id).cmp(&(&b.namespace, &b.package_id)));

    let lock = ContentLock::from_packages(lock_packages);
    let bundle = serde_json::json!({
        "content_lock": lock,
        "packages": raws,
    });
    std::fs::write(
        out_path.clone(),
        serde_json::to_string(&bundle).expect("serialize bundle"),
    )
    .expect("write bundle");
    println!("bundle written: {out_path}");
}

//! 手工维护包的哈希重算工具（2026-10-17）。
//!
//! 读取包目录下的 `package.npack.json`，按 `asset_paths` 重算每个资产条目的
//! `source_hash`（= blake3(资产字节)），再重算 `logic_hash`/`asset_hash` 并写回。
//! 供手工维护的转换包（如 swd）在改动 `assets/*.json` 后同步校验哈希。
//!
//! 运行：`cargo run --offline -p content-runtime --example rehash_npack -- <package_dir>`

use blake3::hash;
use content_schema::ContentPackage;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: rehash_npack <package_dir>"),
    );
    let npack_path = dir.join("package.npack.json");
    let bytes = std::fs::read(&npack_path).expect("读 npack");
    let mut pkg = ContentPackage::from_json_bytes(&bytes).expect("解析 npack");
    if let Some(manifest) = pkg.asset_manifest.as_mut() {
        for entry in manifest.entries.iter_mut() {
            if let Some(rel) = pkg.asset_paths.get(&entry.asset_id) {
                let data =
                    std::fs::read(dir.join(rel)).unwrap_or_else(|e| panic!("读资产 {rel}: {e}"));
                entry.source_hash = *hash(&data).as_bytes();
            }
        }
    }
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    std::fs::write(&npack_path, pkg.to_json_bytes().unwrap()).expect("写 npack");
    println!("rehashed {}", npack_path.display());
}

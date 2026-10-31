//! One-shot: recompute Data asset entry source_hash (blake3) from file on disk.
use std::env;
use std::fs;
use std::path::Path;

fn hash_file(path: &Path) -> [u8; 32] {
    *blake3::Hasher::new()
        .update(&fs::read(path).expect("read"))
        .finalize()
        .as_bytes()
}

fn main() {
    let npack = env::args()
        .nth(1)
        .expect("usage: rehash_cards_asset <npack.json> <asset_rel>");
    let rel = env::args()
        .nth(2)
        .expect("usage: rehash_cards_asset <npack.json> <asset_rel>");
    let root = Path::new(&npack).parent().expect("parent").to_path_buf();
    let bytes = fs::read(&npack).expect("read");
    let mut pkg = content_schema::ContentPackage::from_json_bytes(&bytes).expect("parse");
    let mut updated = 0usize;
    if let Some(manifest) = pkg.asset_manifest.as_mut() {
        for e in manifest.entries.iter_mut() {
            if pkg
                .asset_paths
                .get(&e.asset_id)
                .map(|p| *p == rel)
                .unwrap_or(false)
            {
                e.source_hash = hash_file(&root.join(&rel));
                updated += 1;
            }
        }
    }
    println!("updated {updated} entries");
    let out = pkg.to_json_bytes().expect("serialize");
    fs::write(&npack, out).expect("write");
}

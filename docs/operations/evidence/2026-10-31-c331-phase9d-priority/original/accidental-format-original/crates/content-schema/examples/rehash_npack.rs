//! One-shot: recompute logic/asset hashes after surgical character edits.
use std::env;
use std::fs;

fn main() {
    let path = env::args()
        .nth(1)
        .expect("usage: rehash_npack <npack.json>");
    let bytes = fs::read(&path).expect("read");
    let mut pkg = content_schema::ContentPackage::from_json_bytes(&bytes).expect("parse");
    pkg.manifest.content_schema_version = pkg.required_content_schema();
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    pkg.validate().expect("validate");
    let out = pkg.to_json_bytes().expect("serialize");
    fs::write(&path, out).expect("write");
    println!("rehashed {path}");
}

//! 开发夹具：生成一个含 VM 技能与武将的完整原生包，供真实服务端 E2E 使用。
//!
//! 用法：cargo run -p game-server --example emit_vm_fixture -- /tmp/vm-fixture

fn main() {
    let root = std::env::args().nth(1).expect("output dir");
    let source = r#"
skill vm_fixture_phase_end(990031) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;
    let mut package = content_compiler::compile_source_named("vm-fixture", source).unwrap();
    package.manifest.version = "0.1.0".into();
    package.manifest.logic_hash = package.compute_logic_hash();
    let characters = serde_json::json!({
        "source": "test",
        "upstream_path": "vm-fixture",
        "upstream_commit": "0000000000000000000000000000000000000000",
        "pack": { "name": "vm-fixture", "version": 1, "description": "NC02 E2E" },
        "generals": [{
            "id": "vm_fixture",
            "name": "vm_fixture",
            "faction": "qun",
            "hp": 4,
            "gender": "male",
            "skills": [{ "name": package.trigger_index[0].module, "title": "夹具" }]
        }]
    });
    let character_bytes = serde_json::to_vec(&characters).unwrap();
    let mut npack = serde_json::to_value(&package).unwrap();
    npack["asset_manifest"] = serde_json::json!({
        "schema_version": 1,
        "package_id": "vm-fixture",
        "total_compressed_size": 0,
        "total_decoded_size": 0,
        "entries": []
    });
    let entries = npack["asset_manifest"]["entries"].as_array_mut().unwrap();
    entries.push(serde_json::json!({
        "asset_id": "vm-fixture:characters",
        "source_hash": blake3::hash(&character_bytes).as_bytes(),
        "kind": "Data", "image_variants": [], "audio_variants": [],
        "priority": "Lazy", "depends_on": [], "platforms": []
    }));
    npack["asset_paths"]["vm-fixture:characters"] = serde_json::json!("assets/character.json");
    let mut package: content_schema::ContentPackage = serde_json::from_value(npack).unwrap();
    package.manifest.asset_hash = package.compute_asset_hash();

    let dir = std::path::Path::new(&root);
    std::fs::create_dir_all(dir.join("vm-fixture/assets")).unwrap();
    std::fs::write(
        dir.join("vm-fixture/package.npack.json"),
        package.to_json_bytes().unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("vm-fixture/assets/character.json"),
        character_bytes,
    )
    .unwrap();
    eprintln!("wrote {}", dir.display());
}

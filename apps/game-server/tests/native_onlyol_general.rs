//! N01：onlyOL 现有 npack + ol_zhuhuan 角色资产，经正式 NativeContent 安装/装载/选将。

#[path = "../src/native.rs"]
mod native;

use native::NativeContent;

const ONLYOL: &[u8] =
    include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

#[test]
fn onlyol_requires_abi758_and_native_rejects_until_supported() {
    let characters = serde_json::json!({
        "source": "test",
        "upstream_path": "onlyOL",
        "upstream_commit": "0000000000000000000000000000000000000000",
        "pack": { "name": "onlyOL", "version": 1, "description": "N01" },
        "generals": [{
            "id": "ol_zhuhuan",
            "name": "ol_zhuhuan",
            "faction": "wu",
            "hp": 4,
            "gender": "male",
            "skills": [
                { "name": "ol_fenli_phase_draw", "title": "奋励" },
                { "name": "ol_fenli_phase_play", "title": "奋励" },
                { "name": "ol_fenli_phase_discard", "title": "奋励" },
                { "name": "ol_pingkou_phase_end", "title": "平寇" }
            ]
        }]
    });
    let character_bytes = serde_json::to_vec(&characters).unwrap();
    let mut npack: serde_json::Value = serde_json::from_slice(ONLYOL).unwrap();
    let manifest = npack["asset_manifest"].as_object_mut().unwrap();
    let entries = manifest["entries"].as_array_mut().unwrap();
    entries.push(serde_json::json!({
        "asset_id": "onlyOL:characters",
        "source_hash": blake3::hash(&character_bytes).as_bytes(),
        "kind": "Data", "image_variants": [], "audio_variants": [],
        "priority": "Lazy", "depends_on": [], "platforms": []
    }));
    npack["asset_paths"]["onlyOL:characters"] = serde_json::json!("assets/character.json");
    let mut package: content_schema::ContentPackage = serde_json::from_value(npack).unwrap();
    package.manifest.asset_hash = package.compute_asset_hash();
    let npack = package.to_json_bytes().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("onlyOL");
    std::fs::create_dir_all(dropin.join("assets")).unwrap();
    std::fs::write(dropin.join("package.npack.json"), &npack).unwrap();
    std::fs::write(dropin.join("assets/character.json"), &character_bytes).unwrap();

    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    let error = content.import().expect_err("native host 753 must reject ABI 758 package");
    assert!(
        error.contains("rule_vm_abi host=753 package=758"),
        "unexpected import error: {error}"
    );
}

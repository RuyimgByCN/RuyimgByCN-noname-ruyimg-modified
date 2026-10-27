//! Worker 必须核验完整 npack 的声明摘要，而不是信任可变载荷。
use engine_native_wasm::test_support::{
    engine_from_archive, engine_from_packs, engine_identity_from_packs, hex_encode,
};
use serde_json::{json, Value};

const DSL: &str = r#"
skill integrity_probe(991002) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;

#[test]
fn full_vm_package_hash_is_checked_before_start_and_restore() {
    let package = content_compiler::compile_source_named("integrity", DSL).unwrap();
    let alternative = content_compiler::compile_source_named(
        "integrity",
        &DSL.replace("draw_cards(self, 1)", "draw_cards(self, 2)"),
    )
    .unwrap();
    assert_ne!(package.modules[0].code, alternative.modules[0].code);
    let good = serde_json::to_value(&package).unwrap();
    let good_packs = json!([good.clone()]).to_string();
    let engine = engine_from_packs(4, 42, &good_packs).expect("valid package");
    let good_save = hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    assert_eq!(
        engine_from_archive(&good_save, &good_packs)
            .unwrap()
            .state_hash(),
        engine.state_hash()
    );
    assert!(engine_identity_from_packs(4, 42, &good_packs).is_ok());

    let mut changed_code = good.clone();
    changed_code["modules"][0]["code"] = json!(alternative.modules[0].code);
    let mut changed_trigger = good.clone();
    changed_trigger["trigger_index"][0]["event"] = json!("phase_draw");
    let mut changed_logic_hash = good.clone();
    changed_logic_hash["manifest"]["logic_hash"][0] = json!(package.manifest.logic_hash[0] ^ 1);
    let mut changed_asset_hash = good.clone();
    changed_asset_hash["manifest"]["asset_hash"][0] = json!(1);

    let mut accepted = Vec::new();
    let mut old_saves = Vec::new();
    for (label, expected, item) in [
        ("code", "logic_hash mismatch", changed_code),
        ("trigger", "logic_hash mismatch", changed_trigger),
        ("logic", "logic_hash mismatch", changed_logic_hash),
        ("asset", "asset_hash mismatch", changed_asset_hash),
    ] {
        let packs = json!([item]).to_string();
        for (entry, result) in [
            ("normal", engine_from_packs(4, 42, &packs)),
            ("identity", engine_identity_from_packs(4, 42, &packs)),
        ] {
            match result {
                Ok(engine) => {
                    accepted.push(format!("{label}/{entry}"));
                    if entry == "normal" {
                        let save =
                            hex_encode(&engine_native::archive::encode_archive(&engine.save()));
                        if engine_from_archive(&save, &packs).is_ok() {
                            accepted.push(format!("{label}/old-save-restore"));
                        }
                        old_saves.push(json!({"case": label, "expected": expected, "packs": packs, "hex": save}));
                    }
                }
                Err(error) => assert!(error.contains(expected), "{label}/{entry}: {error}"),
            }
        }
        let error = engine_from_archive(&good_save, &packs).err().unwrap();
        if !error.contains(expected) {
            accepted.push(format!("{label}/restore-not-validated: {error}"));
        }
    }
    if let Some(path) = std::env::var_os("NC02_VM_HASH_OLD_SAVES_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&old_saves).unwrap()).unwrap();
    }
    if let Some(path) = std::env::var_os("NC02_VM_HASH_OLD_SAVES_INPUT") {
        let saves: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(saves.len(), 4);
        for save in saves {
            let error = engine_from_archive(
                save["hex"].as_str().unwrap(),
                save["packs"].as_str().unwrap(),
            )
            .err()
            .unwrap();
            assert!(
                error.contains(save["expected"].as_str().unwrap()),
                "{}: {error}",
                save["case"]
            );
        }
    }
    assert!(
        accepted.is_empty(),
        "invalid VM package hashes accepted: {accepted:?}"
    );
}

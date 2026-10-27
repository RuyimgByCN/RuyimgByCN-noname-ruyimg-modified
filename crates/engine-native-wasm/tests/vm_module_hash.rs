//! Existing module fingerprints must be checked before bridge, start, and restore.
use engine_native_wasm::test_support::{
    engine_from_archive, engine_from_packs, engine_identity_from_packs, hex_encode,
};
use serde_json::{json, Value};

const DSL: &str = r#"
skill module_hash_probe(991004) {
  trigger phase_end owner_is_target {
    effect recover_hp(self, 1);
  }
}
"#;

#[test]
fn changed_module_fields_cannot_bypass_package_and_worker_validation() {
    let original = content_compiler::compile_source_named("module_hash", DSL).unwrap();
    let alternative = content_compiler::compile_source_named(
        "module_hash",
        &DSL.replace("recover_hp(self, 1)", "recover_hp(self, 2)"),
    )
    .unwrap();
    assert_ne!(original.modules[0].code, alternative.modules[0].code);
    let good = json!([original.clone()]).to_string();
    original.validate().unwrap();
    content_native_loader::bridge_vm_skill_decls(&original).unwrap();
    let engine = engine_from_packs(4, 42, &good).unwrap();
    let good_save = hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    assert_eq!(
        engine_from_archive(&good_save, &good).unwrap().state_hash(),
        engine.state_hash()
    );
    engine_identity_from_packs(4, 42, &good).unwrap();

    let mut accepted = Vec::new();
    let mut old_saves = Vec::new();
    for field in ["entry", "locals", "caps", "code", "fingerprint"] {
        let mut changed = original.clone();
        let module = &mut changed.modules[0];
        match field {
            "entry" => module.entry_pc = (module.code.len() - 1) as u32,
            "locals" => module.local_count += 1,
            "caps" => module.caps_mask ^= 1,
            "code" => module.code = alternative.modules[0].code.clone(),
            "fingerprint" => module.logic_hash[0] ^= 1,
            _ => unreachable!(),
        }
        // An attacker can recompute the outer hash, but the module's existing
        // fingerprint must still agree with its actual semantic fields.
        changed.manifest.logic_hash = changed.compute_logic_hash();
        let packs = json!([changed.clone()]).to_string();
        if changed.validate().is_ok() {
            accepted.push(format!("{field}/schema"));
        }
        if content_native_loader::bridge_vm_skill_decls(&changed).is_ok() {
            accepted.push(format!("{field}/bridge"));
        }
        for (entry, result) in [
            ("normal", engine_from_packs(4, 42, &packs)),
            ("identity", engine_identity_from_packs(4, 42, &packs)),
        ] {
            match result {
                Ok(engine) => {
                    accepted.push(format!("{field}/{entry}"));
                    if entry == "normal" {
                        let save =
                            hex_encode(&engine_native::archive::encode_archive(&engine.save()));
                        if engine_from_archive(&save, &packs).is_ok() {
                            accepted.push(format!("{field}/old-save"));
                        }
                        old_saves.push(json!({"case":field,"packs":packs,"hex":save}));
                    }
                }
                Err(error) => assert!(
                    error.contains("module") && error.contains("logic_hash mismatch"),
                    "{field}/{entry}: {error}"
                ),
            }
        }
        match engine_from_archive(&good_save, &packs) {
            Ok(_) => accepted.push(format!("{field}/good-save")),
            Err(error) if !error.contains("logic_hash mismatch") => {
                accepted.push(format!("{field}/good-save: {error}"));
            }
            Err(_) => {}
        }
    }
    if let Some(file) = std::env::var_os("NC02_MODULE_OLD_SAVES_OUTPUT") {
        std::fs::write(file, serde_json::to_vec_pretty(&old_saves).unwrap()).unwrap();
    }
    if let Some(file) = std::env::var_os("NC02_MODULE_OLD_SAVES_INPUT") {
        let saves: Vec<Value> = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
        assert_eq!(saves.len(), 5);
        for save in saves {
            let error = engine_from_archive(
                save["hex"].as_str().unwrap(),
                save["packs"].as_str().unwrap(),
            )
            .err()
            .unwrap();
            assert!(
                error.contains("logic_hash mismatch"),
                "{}: {error}",
                save["case"]
            );
        }
    }
    assert!(
        accepted.is_empty(),
        "invalid module fingerprints accepted: {accepted:?}"
    );
}

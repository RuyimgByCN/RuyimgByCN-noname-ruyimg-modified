//! Invalid VM entry/jump targets cannot cross the package or Worker boundary.
use engine_native_wasm::test_support::{
    engine_from_archive, engine_from_packs, engine_identity_from_packs, hex_encode,
};
use rule_vm::{bytecode::effect_tag, op, BytecodeModule};
use serde_json::{json, Value};

const DSL: &str = r#"
skill nc04_control_probe(991005) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;

fn malformed(case: &str) -> content_schema::ContentPackage {
    let mut package = content_compiler::compile_source_named("nc04_control", DSL).unwrap();
    package.manifest.rule_vm_abi_version = 753;
    let module = &mut package.modules[0];
    let [lo, hi] =
        effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW.to_le_bytes();
    match case {
        "jump" => {
            module.code = vec![
                op::HOST_SELF,
                op::HOST_SELF,
                op::JUMP,
                1,
                0,
                op::CONST_I32,
                op::HOST_EFFECT_EXT,
                lo,
                hi,
                op::HALT,
                op::HALT,
            ];
            module.entry_pc = 0;
        }
        "entry" => {
            module.code = vec![op::CONST_I32, op::HALT, 0, 0, 0, op::HALT];
            module.entry_pc = 1;
        }
        _ => unreachable!(),
    }
    module.logic_hash = BytecodeModule::compute_logic_hash(
        &module.name,
        &module.code,
        module.local_count,
        module.caps_mask,
        module.entry_pc,
    );
    package.manifest.logic_hash = package.compute_logic_hash();
    package
}

#[test]
fn module_targets_are_checked_before_normal_identity_and_restore() {
    let good = content_compiler::compile_source_named("nc04_control", DSL).unwrap();
    let good_packs = json!([good.clone()]).to_string();
    let engine = engine_from_packs(4, 42, &good_packs).unwrap();
    let good_save = hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    assert_eq!(
        engine_from_archive(&good_save, &good_packs)
            .unwrap()
            .state_hash(),
        engine.state_hash()
    );
    engine_identity_from_packs(4, 42, &good_packs).unwrap();

    let mut accepted = Vec::new();
    let mut old_saves = Vec::new();
    for case in ["jump", "entry"] {
        let package = malformed(case);
        let packs = json!([package.clone()]).to_string();
        if package.validate().is_ok() {
            accepted.push(format!("{case}/schema"));
        }
        if content_native_loader::bridge_vm_skill_decls(&package).is_ok() {
            accepted.push(format!("{case}/bridge"));
        }
        let mut new_abi = package.clone();
        new_abi.manifest.rule_vm_abi_version = 758;
        new_abi.manifest.logic_hash = new_abi.compute_logic_hash();
        if new_abi.validate().is_ok() {
            accepted.push(format!("{case}/schema-abi758"));
        }
        if content_native_loader::bridge_vm_skill_decls(&new_abi).is_ok() {
            accepted.push(format!("{case}/bridge-abi758"));
        }
        for (name, result) in [
            ("normal", engine_from_packs(4, 42, &packs)),
            ("identity", engine_identity_from_packs(4, 42, &packs)),
        ] {
            match result {
                Ok(engine) => {
                    accepted.push(format!("{case}/{name}"));
                    if name == "normal" {
                        let save =
                            hex_encode(&engine_native::archive::encode_archive(&engine.save()));
                        if engine_from_archive(&save, &packs).is_ok() {
                            accepted.push(format!("{case}/old-save"));
                        }
                        old_saves.push(json!({"case": case, "packs": packs, "hex": save}));
                    }
                }
                Err(error) => assert!(
                    error.contains("instruction boundary") || error.contains("out of range"),
                    "{case}/{name}: {error}"
                ),
            }
        }
        if engine_from_archive(&good_save, &packs).is_ok() {
            accepted.push(format!("{case}/good-save"));
        }
    }
    if let Some(file) = std::env::var_os("NC04_BOUNDARY_SAVES_OUTPUT") {
        std::fs::write(file, serde_json::to_vec_pretty(&old_saves).unwrap()).unwrap();
    }
    if let Some(file) = std::env::var_os("NC04_BOUNDARY_SAVES_INPUT") {
        let saves: Vec<Value> = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
        assert_eq!(saves.len(), 2);
        for save in saves {
            let error = engine_from_archive(
                save["hex"].as_str().unwrap(),
                save["packs"].as_str().unwrap(),
            )
            .err()
            .unwrap();
            assert!(
                error.contains("instruction boundary") || error.contains("out of range"),
                "{}: {error}",
                save["case"]
            );
        }
    }
    assert!(
        accepted.is_empty(),
        "malformed VM control flow accepted: {accepted:?}"
    );
}

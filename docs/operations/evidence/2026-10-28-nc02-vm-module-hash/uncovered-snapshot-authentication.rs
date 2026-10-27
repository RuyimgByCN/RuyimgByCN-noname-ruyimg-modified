use engine_core::{snapshot::Snapshot, Engine};
use serde_json::json;

#[test]
fn self_consistent_rewritten_frozen_module_is_not_bound_to_state_hash() {
    let package = content_compiler::compile_source_named(
        "forged_snapshot",
        "skill probe(991005) { trigger phase_end owner_is_target { effect recover_hp(self, 1); } }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let snapshot = engine.snapshot();
    let mut module = package.modules[0].clone();
    module.entry_pc = (module.code.len() - 1) as u32;
    module.logic_hash = rule_vm::BytecodeModule::compute_logic_hash(
        &module.name,
        &module.code,
        module.local_count,
        module.caps_mask,
        module.entry_pc,
    );
    content_schema::validate_vm_module_hash(&module).unwrap();
    let mut value = serde_json::to_value(&snapshot).unwrap();
    let modules = value["state"]["content"]["modules"]
        .as_object_mut()
        .unwrap();
    let frozen = modules
        .values_mut()
        .find(|m| m["name"] == json!(module.name))
        .unwrap();
    frozen["entry_pc"] = json!(module.entry_pc);
    frozen["logic_hash"] = json!(module.logic_hash);
    let rewritten = Snapshot::from_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(snapshot.state_hash, rewritten.state_hash);
    rewritten.restore().unwrap();
    eprintln!("snapshot with changed entry and recomputed module hash restores under unchanged state_hash");
}

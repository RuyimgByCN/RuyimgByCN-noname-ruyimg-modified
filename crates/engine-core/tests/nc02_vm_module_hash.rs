//! Direct legacy loading and snapshot restoration must verify module fingerprints.
use engine_core::{
    snapshot::{Snapshot, SnapshotError},
    Engine,
};
use serde_json::json;

const DSL: &str = r#"
skill module_hash_probe(991004) {
  trigger phase_end owner_is_target {
    effect recover_hp(self, 1);
  }
}
"#;

#[test]
fn direct_load_and_old_frozen_snapshot_reject_changed_entry() {
    let package = content_compiler::compile_source_named("module_hash", DSL).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let snapshot = engine.snapshot();
    snapshot.restore().unwrap();
    let mut changed = package.clone();
    let module = &mut changed.modules[0];
    module.entry_pc = (module.code.len() - 1) as u32;
    assert_eq!(changed.compute_logic_hash(), package.manifest.logic_hash);

    let mut value = serde_json::to_value(&snapshot).unwrap();
    let modules = value["state"]["content"]["modules"]
        .as_object_mut()
        .unwrap();
    let frozen = modules
        .values_mut()
        .find(|m| m["name"] == json!(package.modules[0].name))
        .unwrap();
    frozen["entry_pc"] = json!(changed.modules[0].entry_pc);
    let bytes = serde_json::to_vec(&value).unwrap();
    let corrupted = Snapshot::from_bytes(&bytes).unwrap();
    assert_eq!(snapshot.state_hash, corrupted.state_hash);
    let mut accepted = Vec::new();
    if engine.load_package(&changed).is_ok() {
        accepted.push("same package already loaded");
    }
    if Engine::new().load_package(&changed).is_ok() {
        accepted.push("new package load");
    }
    if corrupted.restore().is_ok() {
        accepted.push("frozen snapshot");
    }
    if let Some(file) = std::env::var_os("NC02_MODULE_LEGACY_SNAPSHOT_OUTPUT") {
        std::fs::write(file, &bytes).unwrap();
    }
    if let Some(file) = std::env::var_os("NC02_MODULE_LEGACY_SNAPSHOT_INPUT") {
        let bytes = std::fs::read(file).unwrap();
        let old = Snapshot::from_bytes(&bytes).unwrap();
        assert!(
            matches!(old.restore(), Err(SnapshotError::ContentCompatibility(ref error)) if error.contains("logic_hash mismatch"))
        );
    }
    assert!(
        accepted.is_empty(),
        "invalid module fingerprint accepted: {accepted:?}"
    );
}

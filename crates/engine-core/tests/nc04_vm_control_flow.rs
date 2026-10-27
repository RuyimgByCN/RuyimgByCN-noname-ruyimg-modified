//! The legacy package loader and frozen snapshots reject invalid VM targets.
use engine_core::{snapshot::Snapshot, Engine};
use rule_vm::{bytecode::effect_tag, op, BytecodeModule};

const DSL: &str = r#"
skill nc04_control_probe(991005) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;

fn malformed() -> content_schema::ContentPackage {
    let mut package = content_compiler::compile_source_named("nc04_control", DSL).unwrap();
    package.manifest.rule_vm_abi_version = 753;
    let module = &mut package.modules[0];
    let [lo, hi] =
        effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW.to_le_bytes();
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
fn direct_load_and_frozen_snapshot_reject_interior_jump() {
    let good = content_compiler::compile_source_named("nc04_control", DSL).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&good).unwrap();
    engine.snapshot().restore().unwrap();
    let package = malformed();
    let mut accepted = Vec::new();
    if engine.load_package(&package).is_ok() {
        accepted.push("already-loaded slot");
    }
    let mut old = Engine::new();
    if old.load_package(&package).is_ok() {
        accepted.push("direct load");
        let bytes = old.snapshot().to_bytes().unwrap();
        if Snapshot::from_bytes(&bytes)
            .and_then(|s| s.restore())
            .is_ok()
        {
            accepted.push("old frozen snapshot");
        }
        if let Some(file) = std::env::var_os("NC04_BOUNDARY_LEGACY_OUTPUT") {
            std::fs::write(file, &bytes).unwrap();
        }
    }
    if let Some(file) = std::env::var_os("NC04_BOUNDARY_LEGACY_INPUT") {
        let bytes = std::fs::read(file).unwrap();
        let error = Snapshot::from_bytes(&bytes)
            .and_then(|s| s.restore())
            .err()
            .expect("malformed old snapshot must be rejected")
            .to_string();
        assert!(
            error.contains("instruction boundary") || error.contains("out of range"),
            "{error}"
        );
    }
    assert!(accepted.is_empty(), "malformed VM accepted: {accepted:?}");
}

//! Phase 4: DSL compile → Rule VM → engine effect bridge.

use content_compiler::compile_source;
use engine_api::PlayerId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::vm_host::run_module_for_effects;
use rule_api::EffectRequest;

const DAMAGE_DRAW: &str = r#"
skill damage_draw(1001) {
  trigger damage owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;

#[test]
fn dsl_to_vm_to_engine_draw() {
    let pkg = compile_source(DAMAGE_DRAW).unwrap();
    let module = rule_vm::VerifiedModule::verify(pkg.modules[0].clone()).unwrap();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
    let p2 = PlayerId::new(2);
    let hand_before = tx.state().hand_count(p2);

    let effects =
        run_module_for_effects(tx.state(), &module, 0, p2.raw(), p2.raw(), 1, 0, 0, 10_000)
            .unwrap();
    assert_eq!(effects.len(), 1);
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(p2), hand_before + 1);
}

#[test]
fn package_logic_hash_stable() {
    let a = compile_source(DAMAGE_DRAW).unwrap();
    let b = compile_source(DAMAGE_DRAW).unwrap();
    assert_eq!(a.manifest.logic_hash, b.manifest.logic_hash);
    let json = a.to_json_bytes().unwrap();
    let c = content_schema::ContentPackage::from_json_bytes(&json).unwrap();
    assert_eq!(c.manifest.logic_hash, a.manifest.logic_hash);
    let _ = EffectRequest::EmitRuleEvent {
        code: 0,
        payload: vec![],
    };
}

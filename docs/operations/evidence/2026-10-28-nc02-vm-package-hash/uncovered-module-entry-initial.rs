use engine_native_wasm::test_support::{assign_general_for, engine_from_packs};
use serde_json::json;

const DSL: &str = r#"
skill entry_probe(991003) {
  trigger phase_end owner_is_target {
    effect recover_hp(self, 1);
  }
}
"#;

fn phase_end_hp(package: &content_schema::ContentPackage) -> i32 {
    let mut payload = serde_json::to_value(package).unwrap();
    payload["characters"] = json!({
        "source": "test", "upstream_path": "entry", "upstream_commit": "0000000000000000000000000000000000000000",
        "pack": { "name": "entry", "version": 1, "description": "test" },
        "generals": [{ "id": "entry_probe", "name": "entry_probe", "faction": "qun", "hp": 4,
            "gender": "male", "skills": [{ "name": package.trigger_index[0].module }] }]
    });
    let mut engine = engine_from_packs(4, 42, &json!([payload]).to_string()).unwrap();
    assign_general_for(&mut engine, 0, "entry_probe").unwrap();
    engine.debug_set_hp(0, 3);
    for step in 0..80u64 {
        if engine.turn().current == 0 && engine.turn().phase == engine_native::Phase::End {
            return engine.view_for(0).players[0].hp;
        }
        let (actor, action) = if let Some(choice) = engine.pending_choice() {
            (
                choice.actor,
                engine_native::GameAction::Choose {
                    choice_id: choice.choice_id,
                    index: 0,
                },
            )
        } else {
            (engine.turn().current, engine_native::GameAction::NextPhase)
        };
        let result = engine.submit(engine_native::GameCommand {
            actor,
            expected_revision: engine.revision(),
            dedup_id: 20000 + step,
            action,
        });
        assert!(
            !matches!(result, engine_native::CommandOutcome::Rejected(_)),
            "{result:?}"
        );
    }
    panic!("phase End not reached")
}

#[test]
fn module_entry_pc_not_covered_by_current_package_hash() {
    let original = content_compiler::compile_source_named("entry", DSL).unwrap();
    let mut changed = original.clone();
    let module = &mut changed.modules[0];
    assert_eq!(module.code.last(), Some(&1));
    module.entry_pc = (module.code.len() - 1) as u32;
    assert_eq!(original.manifest.logic_hash, changed.compute_logic_hash());
    original.validate().unwrap();
    changed.validate().unwrap();
    let before = phase_end_hp(&original);
    let after = phase_end_hp(&changed);
    eprintln!("same package hash, original HP={before}, changed entry HP={after}");
    assert_eq!((before, after), (4, 3));
}

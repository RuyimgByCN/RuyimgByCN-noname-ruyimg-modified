//! ADR0786 J2: author DSL must expose active handlers and share their trigger identity.
mod nc08_common;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};

const DSL: &str = r#"
skill j2_identity(9910786) {
  trigger phase_play owner_is_target { effect storage_set(self_skill, 1, 7); }
  active play_phase targets 0 0 { effect storage_set(self_skill, 2, 9); effect draw_cards(self, 1); }
}
"#;

fn package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("j2-identity", DSL).unwrap()
}

fn pack() -> ExtensionPack {
    let pkg = package();
    let vm = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
    let general = serde_json::json!({
        "pack": {"name":"j2-identity","version":1},
        "generals":[{"name":"j2-author","faction":"qun","gender":"male","hp":4,
        "skills":[{"name":"j2_identity"}]}]
    });
    let known = vm.decls.iter().map(|d| d.name.clone()).collect();
    let generals =
        content_native_loader::bridge_generals_with_known(&general.to_string(), &known).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "j2-identity",
            version: 1,
            description: "J2 author",
        },
        generals: generals.generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: vm.decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

#[test]
fn author_active_index_is_available_after_formal_bridge() {
    let pkg = package();
    assert_eq!(pkg.active_index.len(), 1);
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
    assert!(
        out.decls.iter().any(|d| d.name == "j2_identity"),
        "active logical skill is missing: {:?}",
        out.decls
    );
}

#[test]
fn author_active_and_trigger_share_skill_identity_and_storage() {
    let pack = pack();
    let mut engine = Engine::new_with_extensions(9910786, 4, &[&pack]).unwrap();
    engine.assign_general_named(0, "j2-author").unwrap();
    nc08_common::advance_to_play(&mut engine, 0);
    let skill = engine
        .catalog()
        .skill_by_name("j2_identity")
        .expect("logical skill");
    assert!(engine.state_core().skills.contains_key(&(0, skill)));
    assert_eq!(
        engine.state_core().vm_aux.storage.get("j2_identity#1"),
        Some(&7)
    );
    let before = engine.state_core().player(0).hand.len();
    let out = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: nc08_common::next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(engine.state_core().player(0).hand.len(), before + 1);
    assert_eq!(
        engine.state_core().vm_aux.storage.get("j2_identity#2"),
        Some(&9)
    );
    assert!(!engine
        .state_core()
        .vm_aux
        .storage
        .keys()
        .any(|k| k.contains("active_play_phase") || k.contains("phase_play#")));
}

#[test]
fn unsupported_active_contracts_and_entry_pc_reject_explicitly() {
    for field in 0..5 {
        let mut pkg = package();
        match field {
            0 => pkg.active_index[0].entry_pc = 1,
            1 => pkg.active_index[0].uses_per_turn = Some(2),
            2 => pkg.active_index[0].uses_per_turn_source = Some("owner_hp".into()),
            3 => pkg.active_index[0].owner_missing_skill = Some(9910786),
            _ => pkg.active_index[0].targets.as_mut().unwrap().hand_nonempty = true,
        }
        pkg.manifest.logic_hash = pkg.compute_logic_hash();
        let error = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap_err();
        assert!(
            matches!(
                error,
                content_native_loader::BridgeError::InvalidField { .. }
            ),
            "{error:?}"
        );
    }
}

#[test]
fn active_skill_retains_every_trigger_handler() {
    let source = DSL.replace("  active play_phase", "  trigger phase_play owner_is_target { effect storage_set(self_skill, 3, 11); }\n  active play_phase");
    let pkg = content_compiler::compile_source_named("j2-multiple", &source).unwrap();
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
    assert_eq!(out.decls.len(), 1);
    let engine_native::rvm_bridge::VmFilterPolicy::Handlers(set) = &out.decls[0].filter else {
        panic!("logical handlers");
    };
    assert_eq!(set.triggers.len(), 2);
    let mut pack = pack();
    pack.vm_skills = out.decls;
    let mut e = Engine::new_with_extensions(9910786, 4, &[&pack]).unwrap();
    e.assign_general_named(0, "j2-author").unwrap();
    nc08_common::advance_to_play(&mut e, 0);
    assert_eq!(e.state_core().vm_aux.storage.get("j2_identity#1"), Some(&7));
    assert_eq!(
        e.state_core().vm_aux.storage.get("j2_identity#3"),
        Some(&11)
    );
}

#[test]
fn current_gate_rejects_native_rules328_archive_without_migration() {
    let e = Engine::new(9910786, 4);
    let mut snapshot = e.save();
    snapshot.ruleset_version = 328;
    snapshot.core.ruleset_version = 328;
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snapshot))
            .unwrap();
    assert!(matches!(
        Engine::restore(&decoded),
        Err(engine_native::snap::SnapshotError::RuleVersionMismatch {
            snapshot: 328,
            catalog: 333
        })
    ));
}

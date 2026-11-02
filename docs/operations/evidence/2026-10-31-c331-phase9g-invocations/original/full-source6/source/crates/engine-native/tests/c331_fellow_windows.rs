//! Existing Boss addFellow extends inventory, never creates a Normal arrangeTrigger map.
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Timing;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    let o = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    });
    assert!(!matches!(o, CommandOutcome::Rejected(_)), "{o:?}");
}
fn pack(stage: Timing, new_timing: Timing) -> ExtensionPack {
    let source="skill c331_fellow_grant(420003000) { trigger phase_play_end always optional { effect storage_set(self_skill,1,1); } } skill c331_fellow_peer(420003001) locked { trigger phase_play_end always { effect storage_set(self_skill,1,1); } } skill c331_fellow_new(420003002) locked { trigger phase_play_end always { effect storage_set(self_skill,1,1); } }";
    let pkg = content_compiler::compile_source(source).unwrap();
    let mut p = ExtensionPack {
        def: ExtensionDef {
            name: "c331-fellows",
            version: 1,
            description: "real addFellow",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&pkg)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let probe = Engine::new_with_extensions(1, 4, &[&p]).unwrap();
    let new_def = probe.catalog().skill_by_name("c331_fellow_new").unwrap();
    let grant=content_compiler::compile_source(&format!("skill c331_fellow_grant(420003000) {{ trigger phase_play_end always optional {{ effect add_skill(5,{new_def}); }} }}")).unwrap();
    p.vm_skills[0] = content_native_loader::bridge_vm_skill_decls(&grant)
        .unwrap()
        .decls
        .remove(0);
    p.vm_skills[0].trigger.timing = stage;
    p.vm_skills[2].trigger.timing = new_timing;
    p
}
fn opened(p: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(331505, 4, &[p]).unwrap();
    e.state_core_mut().mode = engine_native::GameMode::Boss {
        boss: 0,
        chongzheng: 0,
        fellows: vec![],
        stage: engine_native::state::BossStageState::default_none(),
    };
    e.debug_set_turn(1, Phase::Play, 1);
    e.debug_grant(1, "c331_fellow_grant");
    e.debug_grant(1, "c331_fellow_peer");
    let o = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: GameAction::NextPhase,
    });
    assert!(!matches!(o, CommandOutcome::Rejected(_)), "{o:?}");
    e
}
fn finish(e: &mut Engine) {
    if e.open_choice()
        .is_some_and(|c| c.spec.prompt == "trigger_order")
    {
        let id = e.catalog().skill_by_name("c331_fellow_grant").unwrap();
        answer(e, vec![ChoiceOption::Skill(id)]);
    }
    answer(e, vec![ChoiceOption::Confirm]);
}
fn same(a: &Engine, b: &Engine) {
    let expected = a.save();
    let mut actual = b.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
}
#[test]
fn actual_fellow_inventory_and_grants_use_only_existing_normal_and_global_maps() {
    for stage in [Timing::First, Timing::Normal, Timing::Last] {
        for timing in [Timing::First, Timing::Normal, Timing::Last] {
            let p = pack(stage, timing);
            let mut e = opened(&p);
            assert_eq!(e.add_boss_fellow(4), Some(4));
            let snap = engine_native::archive::decode_archive(
                &engine_native::archive::encode_archive(&e.save()),
            )
            .unwrap();
            for park in &snap.resolution.parks {
                if let engine_native::MachineSnapshot::TriggerOrder {
                    owners,
                    normal_owners,
                    ..
                } = &park.machine
                {
                    assert_eq!(owners.len(), 5);
                    assert_eq!(*normal_owners, 4);
                }
            }
            let mut r = Engine::restore_with_extensions(&snap, &[&p])
                .expect("fellow without a skill still restores");
            let cursor = e.log().len();
            finish(&mut e);
            finish(&mut r);
            same(&e, &r);
            assert_eq!(&e.log()[cursor..], r.log());
            let def = e.catalog().skill_by_name("c331_fellow_new").unwrap();
            assert!(
                e.state_core().skills.contains_key(&(4, def)),
                "actual VM grants the actual new fellow"
            );
            let count = e
                .log()
                .iter()
                .filter(|l| {
                    matches!(
                        l,
                        engine_native::LogEvent::SkillTriggered {
                            skill: "c331_fellow_new",
                            owner: 4
                        }
                    )
                })
                .count();
            let expected = usize::from(
                timing == Timing::Last || (timing == Timing::First && stage == Timing::First),
            );
            assert_eq!(count, expected, "{stage:?}/{timing:?}");
            let snap = engine_native::archive::decode_archive(
                &engine_native::archive::encode_archive(&e.save()),
            )
            .unwrap();
            let restored = Engine::restore_with_extensions(&snap, &[&p]).unwrap();
            same(&e, &restored);
        }
    }
}

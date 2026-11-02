//! C331: handler identity is bound to the actual frame window through execution and restore.
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
fn pack() -> ExtensionPack {
    let source="skill c331_damage_windows(4200000200) locked { trigger damage_before owner_is_target { effect storage_set(self_skill,1,1); effect discard_hand_equip_count(self,1); } trigger damage_before owner_is_target { effect storage_set(self_skill,2,1); } trigger damage owner_is_target { effect draw_cards(self,1); effect discard_hand_equip_count(self,1); } }";
    let p = content_compiler::compile_source(source).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c331_damage_windows",
            version: 1,
            description: "actual damage windows",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .expect("trigger-only before/after handlers can bind the existing frame window")
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn answer(e: &mut Engine) {
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.prompt, "rvm_discard_pick");
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![c.spec.options[0]],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
fn wrong_window_rejects(e: &Engine, p: &ExtensionPack, kind: engine_native::event::EventKind) {
    let engine_native::rvm_bridge::VmFilterPolicy::Handlers(set) = &p.vm_skills[0].filter else {
        panic!("handlers");
    };
    let bytes = set
        .triggers
        .iter()
        .find(|t| t.meta.event == kind)
        .unwrap()
        .program
        .module
        .to_bytes();
    let mut snap = e.save();
    let m = snap
        .resolution
        .parks
        .iter_mut()
        .find_map(|p| match &mut p.machine {
            engine_native::MachineSnapshot::Rvm { module, .. } => Some(module),
            _ => None,
        })
        .unwrap();
    *m = bytes;
    assert!(
        Engine::restore_with_extensions(
            &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
                .unwrap(),
            &[p]
        )
        .is_err(),
        "a valid sibling module must not switch the owning frame window"
    );
}
fn same(a: &Engine, b: &Engine) {
    let expected = a.save();
    let mut actual = b.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
}
#[test]
fn before_storage_and_after_draw_once_survive_both_commit_pauses() {
    let p = pack();
    let mut e = Engine::new_with_extensions(331400, 4, &[&p]).unwrap();
    e.debug_grant(1, "c331_damage_windows");
    let hand = e.state_core().player(1).hand.len();
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert_eq!(
        e.state_core().vm_aux.storage.get("c331_damage_windows#1"),
        Some(&1)
    );
    assert_eq!(e.state_core().player(1).hp, 4);
    assert_eq!(e.state_core().player(1).hand.len(), hand);
    wrong_window_rejects(&e, &p, engine_native::event::EventKind::Damage);
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snap = engine_native::archive::decode_archive(&bytes).unwrap();
    let mut r = Engine::restore_with_extensions(&snap, &[&p]).unwrap();
    let cursor = e.log().len();
    answer(&mut e);
    answer(&mut r);
    same(&e, &r);
    assert_eq!(&e.log()[cursor..], r.log());
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        hand,
        "after handler draws exactly one after the before discard"
    );
    assert_eq!(
        e.state_core().vm_aux.storage.get("c331_damage_windows#2"),
        Some(&1),
        "second before handler runs in the same before window"
    );
    wrong_window_rejects(&e, &p, engine_native::event::EventKind::DamageBefore);
    let bytes = engine_native::archive::encode_archive(&e.save());
    let mut r2 = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).unwrap(),
        &[&p],
    )
    .unwrap();
    answer(&mut e);
    answer(&mut r);
    answer(&mut r2);
    same(&e, &r);
    same(&e, &r2);
    assert_eq!(e.state_core().player(1).hand.len(), hand - 1);
    assert!(e.open_choice().is_none());
    assert_eq!(
        e.log()
            .iter()
            .filter(|l| matches!(
                l,
                engine_native::LogEvent::SkillTriggered {
                    skill: "c331_damage_windows",
                    ..
                }
            ))
            .count(),
        2
    );
}

#[test]
fn gujian_food_ordering_matches_the_actual_liuli_add_list_formula() {
    use engine_native::content::host::HostBehavior as H;
    for h in [
        H::GujianFoodGudonggeng,
        H::GujianFoodLiyutang,
        H::GujianFoodMapodoufu,
        H::GujianFoodQingtuan,
        H::GujianFoodTanhuadong,
        H::GujianFoodXiajiao,
        H::GujianFoodYougeng,
        H::GujianFoodZiyangdan,
    ] {
        let meta = h.trigger_meta().unwrap();
        let silent = h == H::GujianFoodZiyangdan;
        assert_eq!(meta.priority, i32::from(silent));
        assert_eq!(meta.order_selectable, !silent);
        assert_eq!(meta.timing, engine_native::event::Timing::Normal);
    }
}

fn trigger_only_cannot_disguise_slot(intrinsic: bool) {
    use engine_native::resolution::{Slot, NO_FRAME};
    let p = pack();
    let mut e = Engine::new_with_extensions(331400, 4, &[&p]).unwrap();
    e.debug_grant(1, "c331_damage_windows");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let engine_native::rvm_bridge::VmFilterPolicy::Handlers(set) = &p.vm_skills[0].filter else {
        panic!("handlers");
    };
    let after = set
        .triggers
        .iter()
        .find(|t| t.meta.event == engine_native::event::EventKind::Damage)
        .unwrap()
        .program
        .module
        .to_bytes();
    let mut snap = e.save();
    let park = snap
        .resolution
        .parks
        .iter_mut()
        .find(|p| matches!(p.machine, engine_native::MachineSnapshot::Rvm { .. }))
        .unwrap();
    let Slot::Trigger { frame } = park.slot else {
        panic!("trigger frame");
    };
    let engine_native::MachineSnapshot::Rvm { module, .. } = &mut park.machine else {
        unreachable!();
    };
    *module = after;
    park.slot = if intrinsic {
        Slot::Intrinsic { frame }
    } else {
        Slot::Trigger { frame: NO_FRAME }
    };
    assert!(
        Engine::restore_with_extensions(
            &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
                .unwrap(),
            &[&p]
        )
        .is_err(),
        "module identity must require its actual trigger frame before slot dispatch"
    );
}
#[test]
fn trigger_only_no_frame_sentinel_cannot_hide_wrong_window() {
    trigger_only_cannot_disguise_slot(false);
}
#[test]
fn trigger_only_intrinsic_slot_cannot_hide_wrong_window() {
    trigger_only_cannot_disguise_slot(true);
}

#[test]
fn actual_active_catalog_module_keeps_its_intrinsic_resume_path() {
    use engine_native::resolution::{Slot, NO_FRAME};
    let source="skill c331_active_window(4200000201) { active play_phase targets 0 0 { effect storage_set(self_skill,1,1); effect discard_hand_equip_count(self,1); } trigger damage owner_is_target { effect draw_cards(self,1); } }";
    let pkg = content_compiler::compile_source(source).unwrap();
    let mut p = pack();
    p.vm_skills = content_native_loader::bridge_vm_skill_decls(&pkg)
        .unwrap()
        .decls;
    let mut e = Engine::new_with_extensions(331401, 4, &[&p]).unwrap();
    e.debug_set_turn(1, engine_native::Phase::Play, 1);
    e.debug_grant(1, "c331_active_window");
    let def = e.catalog().skill_by_name("c331_active_window").unwrap();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let snap = e.save();
    assert!(snap.resolution.parks.iter().any(|p| matches!(
        p.machine,
        engine_native::MachineSnapshot::Rvm { .. }
    ) && p.slot
        == Slot::Intrinsic { frame: NO_FRAME }));
    let mut restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
            .unwrap(),
        &[&p],
    )
    .unwrap();
    let cursor = e.log().len();
    answer(&mut e);
    answer(&mut restored);
    same(&e, &restored);
    assert_eq!(&e.log()[cursor..], restored.log());
}

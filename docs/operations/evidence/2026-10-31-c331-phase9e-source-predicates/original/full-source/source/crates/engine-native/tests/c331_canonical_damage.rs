use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::DamageNature;
use engine_native::{Engine, Phase};
fn pack(body: &str) -> ExtensionPack {
    let source =
        format!("skill c9_target(99109001) {{ trigger damage owner_is_target {{ {body} }} }}");
    let p = content_compiler::compile_source_named("c9-damage", &source).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c9-damage",
            version: 1,
            description: "real VM damage window",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn engine(p: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(11901, 3, &[p]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(1, "c9_target");
    e
}
#[test]
fn c9_cancelled_root_does_not_dispatch_target_damage() {
    let p = pack("effect draw_cards(self,1);");
    let mut e = engine(&p);
    e.state_core_mut().player_mut(1).jidong_active = true;
    let hand = e.state_core().player(1).hand.len();
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        hand,
        "cancelled damage has no target damage window"
    );
}
#[test]
fn c9_target_damage_runs_before_dying_subtree() {
    let p = pack("effect recover_hp(self,1);");
    let mut e = engine(&p);
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(
        e.debug_hp(1),
        1,
        "target damage skill recovers before dying collection"
    );
    assert!(e.open_choice().is_none(), "no premature rescue choice");
}
#[test]
fn c9_chain_child_does_not_overtake_parent_dying() {
    let mut e = Engine::new(11902, 3);
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_set_hp(1, 1);
    e.debug_move_named_card_to_hand(0, "tao");
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(e.open_choice().is_some(), "parent rescue is pending");
    assert_eq!(
        e.debug_hp(2),
        4,
        "chain child waits for complete parent dying subtree"
    );
}
fn optional_pack() -> ExtensionPack {
    let p=content_compiler::compile_source_named("c9-optional","skill c9_pause(99109002) { trigger damage owner_is_target optional { effect draw_cards(self,1); } }").unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c9-optional",
            version: 1,
            description: "real target optional pause",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
#[test]
fn c9_full_armor_retains_source_quantity_positive_receipt_and_zero_hp_loss() {
    let p = optional_pack();
    let mut e = Engine::new_with_extensions(11903, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9_pause");
    e.debug_set_hujia(1, 3);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert!(e.open_choice().is_some());
    let s = e.save();
    let d = s
        .resolution
        .stack
        .iter()
        .find_map(|f| {
            if let engine_native::event::EventData::Damage(d) = &f.data {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(d.amount, 2);
    assert_eq!(d.settlement.committed_damage, Some(2));
    assert_eq!(d.settlement.hp_loss, 0);
    assert_eq!(e.debug_hp(1), 4);
    assert_eq!(e.state_core().player(1).hujia, 1);
    let bytes = engine_native::archive::encode_archive(&s);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(decoded, s);
    let restored = Engine::restore_with_extensions(&decoded, &[&p]).unwrap();
    let mut after = restored.save();
    after.auth_generation = s.auth_generation;
    assert_eq!(after, s);
    assert_eq!(e.state_hash(), restored.state_hash());
}
#[test]
fn c9_changed_root_amount_matches_damage_log_and_later_vm_context() {
    let source="skill c9_modify(99109003) { trigger damage_before owner_is_target { effect modify_current_damage(self,1); } } skill c9_pause(99109002) { trigger damage owner_is_target optional { effect draw_cards(self,1); } }";
    let p = content_compiler::compile_source_named("c9-modify", source).unwrap();
    let p = ExtensionPack {
        def: ExtensionDef {
            name: "c9-modify",
            version: 1,
            description: "actual VM before and target windows",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(11904, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9_modify");
    e.debug_grant(1, "c9_pause");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 2);
    let s = e.save();
    let d = s
        .resolution
        .stack
        .iter()
        .find_map(|f| match &f.data {
            engine_native::event::EventData::Damage(d) => Some(d),
            _ => None,
        })
        .unwrap();
    assert_eq!(d.amount, 2);
    assert_eq!(d.settlement.committed_damage, Some(2));
    assert!(e.log().iter().any(|x| matches!(
        x,
        engine_native::machine::LogEvent::DamageDealt { amount: 2, .. }
    )));
    let mut forged = s.clone();
    let f = forged
        .resolution
        .stack
        .iter_mut()
        .find(|f| matches!(f.data, engine_native::event::EventData::Damage(_)))
        .unwrap();
    f.stage = engine_native::event::Stage::Commit;
    assert!(Engine::restore_with_extensions(&forged, &[&p]).is_err());
}
#[test]
fn c9_real_vm_source_window_uses_qualified_root_and_byte_restore() {
    let raw=content_compiler::compile_source_named("c9-source","skill c9_source(99109005) { trigger damage owner_is_source optional { effect draw_cards(self,event_amount); } }").unwrap();
    let mut decls = content_native_loader::bridge_vm_skill_decls(&raw)
        .unwrap()
        .decls;
    decls[0].trigger.event = engine_native::event::EventKind::DamageSource;
    let p = ExtensionPack {
        def: ExtensionDef {
            name: "c9-source",
            version: 1,
            description: "real declared VM source binding",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(11905, 3, &[&p]).unwrap();
    e.debug_grant(0, "c9_source");
    let hand = e.state_core().player(0).hand.len();
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let s = e.save();
    let window = s
        .resolution
        .stack
        .iter()
        .find(|f| matches!(f.data, engine_native::event::EventData::DamageWindow { .. }))
        .unwrap();
    assert_eq!(window.kind, engine_native::event::EventKind::DamageSource);
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&s))
            .unwrap();
    let mut r = Engine::restore_with_extensions(&decoded, &[&p]).unwrap();
    let mut normalized = r.save();
    normalized.auth_generation = s.auth_generation;
    assert_eq!(normalized, s);
    let mut forged = s.clone();
    let frame = forged
        .resolution
        .stack
        .iter_mut()
        .find(|f| matches!(f.data, engine_native::event::EventData::DamageWindow { .. }))
        .unwrap();
    if let engine_native::event::EventData::DamageWindow { root, .. } = &mut frame.data {
        *root = frame.id;
    }
    assert!(Engine::restore_with_extensions(&forged, &[&p]).is_err());
    for game in [&mut e, &mut r] {
        let choice = game.open_choice().unwrap().clone();
        let out = game.submit(engine_native::GameCommand {
            actor: choice.observer,
            expected_revision: game.revision(),
            dedup_id: game.revision() + 1,
            action: engine_native::GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Confirm],
            },
        });
        assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    }
    assert_eq!(e.state_core().player(0).hand.len(), hand + 2);
    assert_eq!(e.state_hash(), r.state_hash());
    let mut a = e.save();
    let mut b = r.save();
    a.auth_generation = 0;
    b.auth_generation = 0;
    assert_eq!(a, b);
}
#[test]
fn c9_zero_window_runs_once_and_signed_overflow_is_atomic() {
    let raw = content_compiler::compile_source_named(
        "c9-zero",
        "skill c9_zero(99109006) { trigger damage owner_is_target { effect draw_cards(self,1); } }",
    )
    .unwrap();
    let mut decls = content_native_loader::bridge_vm_skill_decls(&raw)
        .unwrap()
        .decls;
    decls[0].trigger.event = engine_native::event::EventKind::DamageZero;
    let p = ExtensionPack {
        def: ExtensionDef {
            name: "c9-zero",
            version: 1,
            description: "genuine zero window handler",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(11906, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9_zero");
    let hand = e.state_core().player(1).hand.len();
    e.debug_deal_damage(Some(0), 1, 0, DamageNature::Normal);
    assert_eq!(e.state_core().player(1).hand.len(), hand + 1);
    assert_eq!(e.debug_hp(1), 4);
    let p=content_compiler::compile_source_named("c9-overflow","skill c9_overflow(99109007) { trigger damage_before owner_is_target { effect modify_current_damage(self,1); } }").unwrap();
    let p = ExtensionPack {
        def: ExtensionDef {
            name: "c9-overflow",
            version: 1,
            description: "checked signed boundary",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(11907, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9_overflow");
    let before = e.save();
    assert!(matches!(
        e.debug_deal_damage(Some(0), 1, i32::MAX as u32, DamageNature::Normal),
        engine_native::CommandOutcome::Rejected(_)
    ));
    assert_eq!(e.save(), before);
}
#[test]
fn c9_infinite_hp_keeps_positive_source_receipt() {
    let p = optional_pack();
    let mut e = Engine::new_with_extensions(11908, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9_pause");
    e.debug_set_max_hp(1, i32::MAX as u32);
    e.debug_set_hp(1, i32::MAX);
    e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    assert!(e.open_choice().is_some());
    let s = e.save();
    let d = s
        .resolution
        .stack
        .iter()
        .find_map(|f| match &f.data {
            engine_native::event::EventData::Damage(d) => Some(d),
            _ => None,
        })
        .unwrap();
    assert_eq!(d.amount, 3);
    assert_eq!(d.settlement.committed_damage, Some(3));
    assert_eq!(d.settlement.hp_loss, 0);
    assert_eq!(e.debug_hp(1), i32::MAX);
}
#[test]
fn c9_lose_hp_replacement_has_no_damage_receipt_or_target_trigger() {
    let p = pack("effect draw_cards(self,1);");
    let mut e = engine(&p);
    e.debug_grant(0, "绝情");
    e.debug_set_hp(1, 1);
    e.debug_move_named_card_to_hand(0, "tao");
    let hand = e.state_core().player(1).hand.len();
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let s = e.save();
    let d = s
        .resolution
        .stack
        .iter()
        .find_map(|f| match &f.data {
            engine_native::event::EventData::Damage(d) => Some(d),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        d.settlement.outcome,
        engine_native::event::DamageOutcome::ReplacedByLoseHp
    );
    assert_eq!(d.settlement.committed_damage, None);
    assert_eq!(e.state_core().player(1).hand.len(), hand);
    assert!(!e
        .log()
        .iter()
        .any(|x| matches!(x, engine_native::machine::LogEvent::DamageDealt { .. })));
}
#[test]
fn c9b_before_is_an_actual_child_with_root_continuation_before_hp() {
    let raw=content_compiler::compile_source_named("c9b-before","skill c9b_before(99109101) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } }").unwrap();
    let p = ExtensionPack {
        def: ExtensionDef {
            name: "c9b-before",
            version: 1,
            description: "preHP root binding",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&raw)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(12001, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9b_before");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let s = e.save();
    assert_eq!(e.debug_hp(1), 4);
    assert!(
        s.resolution.stack.iter().any(|f| matches!(
            f.data,
            engine_native::event::EventData::DamageWindow { .. }
        ) && f.kind
            == engine_native::event::EventKind::DamageBefore),
        "Before must be a real ordinary child window"
    );
    assert!(
        s.resolution.parks.iter().any(|p| matches!(
            p.machine,
            engine_native::snap::MachineSnapshot::DamageFlow { .. }
        )),
        "root Flow already owns preHP progression"
    );
}

//! Approved27 owner-only source metadata; source does not certify a root.
use engine_native::{CommandOutcome, Engine, Phase};

#[test]
fn native_optional_confirm_publishes_own_source_only() {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    assert!(matches!(
        engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal),
        CommandOutcome::Pending { .. }
    ));
    let before = engine_native::archive::encode_archive(&engine.save());
    if let Ok(directory) = std::env::var("SOURCE27_EVIDENCE") {
        let path = std::path::Path::new(&directory);
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(path.join("actual-native-ask.ngsa"), &before).unwrap();
        std::fs::write(
            path.join("actual-native-ask.hex"),
            before
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        )
        .unwrap();
    }
    let own = format!("{:?}", engine.choice_view_for(Some(1)).unwrap());
    eprintln!("SOURCE27_OWN {own}");
    assert!(
        own.contains("source_skill: Some(\"遗计\")"),
        "owner must see the actual native ask source: {own}"
    );
    assert_eq!(format!("{:?}", engine.choice_view_referee().unwrap()), own);
    assert_eq!(format!("{:?}", engine.ai_session(1).choice().unwrap()), own);
    let other = format!("{:?}", engine.choice_view_for(Some(0)).unwrap());
    assert!(!other.contains("source_skill"));
    assert!(other.contains("trigger_confirm"));
    assert_eq!(
        engine_native::archive::encode_archive(&engine.save()),
        before
    );
}

#[test]
fn native_known_optional_families_publish_their_exact_names() {
    for name in ["遗计", "刚烈", "奸雄"] {
        let mut engine = Engine::new(2306, 3);
        engine.debug_set_turn(0, Phase::Play, 1);
        for seat in 0..3 {
            engine.debug_clear_hand(seat);
        }
        engine.debug_grant(1, name);
        let sha = engine.debug_place(0, "sha");
        let result = engine.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action: engine_native::GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        });
        assert!(
            matches!(result, CommandOutcome::Pending { .. }),
            "{name}: {result:?}"
        );
        let Some(engine_native::ChoiceView::Own { source_skill, .. }) =
            engine.choice_view_for(Some(1))
        else {
            panic!("{name}: own ask");
        };
        assert_eq!(source_skill, Some(name));
    }
}

#[test]
fn native_source_is_stable_across_private_deck_rng_and_counters() {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let mut saved = engine.save();
    saved.core.deck.reverse();
    saved.core.rng = engine_native::rng::RuleRng::from_seed(99);
    saved.core.vm_aux.drawn.insert(1, u32::MAX);
    let left = Engine::restore(&engine.save()).unwrap();
    let right = Engine::restore(&saved).unwrap();
    assert_eq!(
        format!("{:?}", left.ai_session(1).input()),
        format!("{:?}", right.ai_session(1).input())
    );
    assert_eq!(
        format!("{:?}", left.event_view_for(1)),
        format!("{:?}", right.event_view_for(1))
    );
}
#[test]
fn actual_vm_ask_and_restored_vm_gate_sources_stay_unknown() {
    use engine_native::content::extension::{ExtensionDef, ExtensionPack};
    let package=content_compiler::compile_source_named("source-vm","skill source_vm(99107927) { active play_phase targets 0 0 { effect resolve_ol_jie_danshou(self); } }").unwrap();
    let decls = content_native_loader::bridge_vm_skill_decls(&package)
        .unwrap()
        .decls;
    let name = decls[0].name.clone();
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "source-vm",
            version: 1,
            description: "source rejection",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    };
    let mut engine = Engine::new_with_extensions(2306, 3, &[&pack]).unwrap();
    engine.debug_set_turn(0, Phase::Play, 1);
    engine.debug_grant(0, &name);
    let skill = engine.catalog().skill_by_name(&name).unwrap();
    let result = engine.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: engine.revision() + 1,
        action: engine_native::GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(result, CommandOutcome::Pending { .. }),
        "{result:?}"
    );
    let Some(engine_native::ChoiceView::Own { source_skill, .. }) = engine.choice_view_for(Some(0))
    else {
        panic!("VM ask must exist")
    };
    assert_eq!(source_skill, None);
    assert_eq!(
        engine.open_choice().unwrap().spec.prompt,
        "rvm_danshou_option"
    );
    let mut native = Engine::new_with_extensions(2306, 3, &[&pack]).unwrap();
    native.debug_set_turn(0, Phase::Play, 1);
    native.debug_grant(1, "遗计");
    native.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    native.debug_grant(1, &name);
    let mut saved = native.save();
    let engine_native::MachineSnapshot::OptionalGate { def, .. } =
        &mut saved.resolution.parks.last_mut().unwrap().machine
    else {
        panic!("native gate")
    };
    *def = native.catalog().skill_by_name(&name).unwrap();
    // C rejects rebinding a native parked trigger to an unrelated VM-active definition.
    assert!(Engine::restore_with_extensions(&saved, &[&pack]).is_err());
    let optional = content_compiler::compile_source_named("source-vm-optional", "skill source_vm_optional(99107928) { trigger damage owner_is_target optional { effect draw_cards(self,1); } }").unwrap();
    let optional_pack = engine_native::content::extension::ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "source-vm-optional",
            version: 1,
            description: "actual optional VM provenance",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&optional)
            .unwrap()
            .decls,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut actual = Engine::new_with_extensions(2306, 3, &[&optional_pack]).unwrap();
    actual.debug_set_turn(0, Phase::Play, 1);
    actual.debug_grant(1, "source_vm_optional");
    actual.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(actual.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let snapshot = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
        &actual.save(),
    ))
    .unwrap();
    let restored = Engine::restore_with_extensions(&snapshot, &[&optional_pack]).unwrap();
    for engine in [&actual, &restored] {
        assert!(matches!(
            engine.choice_view_for(Some(1)),
            Some(engine_native::ChoiceView::Own {
                source_skill: None,
                ..
            })
        ));
    }
}

#[test]
fn source_initial_binding_does_not_certify_the_next_pending_prompt() {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let original = engine.save();
    let mut with_later_park = original.clone();
    with_later_park
        .resolution
        .parks
        .push(engine_native::ParkSnapshot {
            resume_depth: original.resolution.stack.len(),
            slot: engine_native::resolution::Slot::Trigger {
                frame: engine_native::resolution::NO_FRAME,
            },
            machine: engine_native::MachineSnapshot::JiJie {
                owner: 0,
                stage: engine_native::snap::JiJieStageSnap::Ask,
                card: None,
            },
        });
    let restore = |snapshot: &engine_native::StateSnapshot| {
        Engine::restore(
            &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
                snapshot,
            ))
            .unwrap(),
        )
        .unwrap()
    };
    let mut left = restore(&original);
    let mut right = restore(&with_later_park);
    let input = format!("{:?}", left.ai_session(1).input());
    assert_eq!(input, format!("{:?}", right.ai_session(1).input()));
    assert_eq!(left.event_view_for(1), right.event_view_for(1));
    let pre_events = format!("{:?}", left.event_view_for(1));
    assert!(matches!(
        left.choice_view_for(Some(1)),
        Some(engine_native::ChoiceView::Own {
            source_skill: Some("遗计"),
            ..
        })
    ));
    let mut pending = Vec::new();
    for (label, candidate) in [("p-only", &mut left), ("p-then-q", &mut right)] {
        let before = engine_native::archive::encode_archive(&candidate.save());
        let choice = candidate.open_choice().unwrap().clone();
        let result = candidate.submit(engine_native::GameCommand {
            actor: choice.observer,
            expected_revision: candidate.revision(),
            dedup_id: candidate.revision() + 1,
            action: engine_native::GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Confirm],
            },
        });
        assert!(
            matches!(result, CommandOutcome::Pending { .. }),
            "{label}: {result:?}"
        );
        let next = candidate.open_choice().unwrap();
        pending.push((next.spec.prompt, next.observer));
        eprintln!(
            "SOURCE27_PREFIX {label}: {result:?}; first Pending={:?}",
            pending.last().unwrap()
        );
        if let Ok(directory) = std::env::var("SOURCE27_EVIDENCE") {
            let path = std::path::Path::new(&directory);
            std::fs::create_dir_all(path).unwrap();
            std::fs::write(
                path.join(format!("source-prefix-{label}-before.ngsa")),
                before,
            )
            .unwrap();
            std::fs::write(
                path.join(format!("source-prefix-{label}-input.txt")),
                &input,
            )
            .unwrap();
            std::fs::write(
                path.join(format!("source-prefix-{label}-events.txt")),
                &pre_events,
            )
            .unwrap();
            std::fs::write(
                path.join(format!("source-prefix-{label}-after.ngsa")),
                engine_native::archive::encode_archive(&candidate.save()),
            )
            .unwrap();
        }
    }
    assert_eq!(pending[0], ("jiji_target", 1));
    assert_eq!(pending[1], ("jijie_give_target", 0));
}

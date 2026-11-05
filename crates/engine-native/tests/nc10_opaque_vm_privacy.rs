//! A reachable VM local caches an opponent's private hand color before HOST_CHOICE.
use engine_native::ai::BoundedPlanner;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook, VmSkillDecl};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use rule_vm::bytecode::{caps, effect_tag, op, query_tag};

fn constant(code: &mut Vec<u8>, value: u32) {
    code.push(op::CONST_U32);
    code.extend(value.to_le_bytes());
}

fn pack() -> ExtensionPack {
    let mut code = vec![];
    constant(&mut code, 3); // rule seat 3 = native opponent 2
    constant(&mut code, 1); // red
    code.extend([op::HOST_QUERY, query_tag::HAND_COLOR_COUNT, op::STORE, 0]);
    constant(&mut code, 0);
    constant(&mut code, 1);
    code.extend([
        op::HOST_CHOICE,
        2,
        op::STORE,
        1,
        op::LOAD,
        1,
        op::JUMP_IF_FALSE,
    ]);
    let jump = code.len();
    code.extend([0, 0]);
    constant(&mut code, 1);
    constant(&mut code, 3);
    code.extend([
        op::LOAD,
        0,
        op::HOST_EFFECT,
        effect_tag::DEAL_DAMAGE as u8,
        op::HALT,
    ]);
    let destination = code.len();
    code[jump..jump + 2].copy_from_slice(&((destination - jump - 2) as i16).to_le_bytes());
    constant(&mut code, 1);
    constant(&mut code, 4);
    constant(&mut code, 1);
    code.extend([op::HOST_EFFECT, effect_tag::DEAL_DAMAGE as u8, op::HALT]);
    ExtensionPack {
        def: ExtensionDef {
            name: "nc10-opaque",
            version: 1,
            description: "cached hidden color",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        init_hook: InitHook::None,
        enabled: true,
        vm_skills: vec![VmSkillDecl {
            r05_identity: None,
            declaration_hash: None,
            presentation_binding: None,
            name: "nc10_opaque".into(),
            module: rule_vm::BytecodeModule {
                name: "nc10-opaque".into(),
                code,
                local_count: 2,
                max_stack: 8,
                entry_pc: 0,
                logic_hash: [41; 32],
                caps_mask: caps::RUN_QUERIES | caps::REQUEST_CHOICE | caps::EMIT_EFFECTS,
            },
            trigger: engine_native::catalog::TriggerMeta {
                event: engine_native::EventKind::PhaseEnd,
                timing: engine_native::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![],
            },
            filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
        }],
    }
}

fn fixture(pack: &ExtensionPack, red: bool) -> Engine {
    let mut e = Engine::new_with_extensions(991_100, 4, &[pack]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, "nc10_opaque");
    for index in 0..4 {
        let deck_index = e
            .state_core()
            .deck
            .iter()
            .position(|id| {
                let suit = e.catalog().cards[*id as usize].suit;
                matches!(
                    suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                ) == red
            })
            .unwrap();
        let core = e.state_core_mut();
        std::mem::swap(&mut core.players[2].hand[index], &mut core.deck[deck_index]);
    }
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: GameAction::NextPhase,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_choice");
    Engine::restore_with_extensions(&e.save(), &[pack]).unwrap();
    e
}

#[test]
fn opaque_vm_private_local_does_not_change_choice() {
    let pack = pack();
    let a = fixture(&pack, false);
    let mut b = fixture(&pack, true);
    b.state_core_mut()
        .vm_aux
        .storage
        .insert("opaque#private".into(), 42);
    assert_eq!(
        format!("{:?}", a.ai_session(0).input()),
        format!("{:?}", b.ai_session(0).input())
    );
    assert_eq!(a.ai_session(0).choice(), b.ai_session(0).choice());
    assert_eq!(a.ai_session(0).events(), b.ai_session(0).events());
    let original = [a.save(), b.save()];
    for (engine, expected) in [(&a, 0i32), (&b, 4i32)] {
        let snapshot = engine.save();
        let engine_native::MachineSnapshot::Rvm {
            paused: Some(paused),
            owner,
            ..
        } = &snapshot.resolution.parks.iter().find(|p| matches!(p.machine, engine_native::MachineSnapshot::Rvm { .. })).expect("VM owner machine").machine
        else {
            panic!("VM paused")
        };
        assert_eq!(*owner, 0);
        assert_eq!(
            i32::from_le_bytes(paused.frames[2..6].try_into().unwrap()),
            expected
        );
        eprintln!(
            "NC10_OPAQUE pc={} local0={} frames={:?}",
            paused.pc, expected, paused.frames
        );
    }
    let planner = BoundedPlanner::new(48)
        .with_policy_variant(1)
        .with_lookahead(2);
    let p = a.plan_action(0, &planner);
    let q = b.plan_action(0, &planner);
    eprintln!("NC10_OPAQUE left={p:?} right={q:?}");
    assert_eq!(
        format!("{p:?}"),
        format!("{q:?}"),
        "cached unknown hand color leaked through VM local 0"
    );
    assert_eq!(p.nodes_used, 0);
    assert_eq!([a.save(), b.save()], original);
    let damage_after = |action| {
        let mut trial = Engine::restore_with_extensions(&b.save(), &[&pack]).unwrap();
        let before: i32 = trial
            .state_core()
            .players
            .iter()
            .skip(1)
            .map(|p| p.hp)
            .sum();
        let outcome = trial.submit(GameCommand {
            actor: 0,
            expected_revision: trial.revision(),
            dedup_id: trial.save().last_dedup + 1,
            action,
        });
        assert!(!matches!(outcome, CommandOutcome::Rejected(_)));
        before
            - trial
                .state_core()
                .players
                .iter()
                .skip(1)
                .map(|p| p.hp)
                .sum::<i32>()
    };
    let conservative_damage = damage_after(p.action.clone());
    let private_damage = damage_after(GameAction::Answer {
        choice_id: b.open_choice().unwrap().id,
        selection: vec![ChoiceOption::Tag(1)],
    });
    eprintln!("NC10_LEAF_COST private_red_count=4 leaf_damage={conservative_damage} private_continuation_damage={private_damage}");
    assert_eq!((conservative_damage, private_damage), (1, 4));
    for source in [&a, &b] {
        let mut restored = Engine::restore_with_extensions(&source.save(), &[&pack]).unwrap();
        assert!(
            matches!(&p.action, GameAction::Answer { selection, .. } if selection.len() == 1 && matches!(selection[0], ChoiceOption::Tag(_)))
        );
        assert!(!matches!(
            restored.apply_plan(&p).unwrap(),
            CommandOutcome::Rejected(_)
        ));
    }
}

#[test]
fn author_dsl_private_query_in_pending_effects_is_a_public_search_leaf() {
    let source = "skill nc10_author(991042) { trigger card_targeted always { let reds = hand_color_count(self, red); effect discard_hand_equip_count(event_player, 1); effect deal_damage(self, event_player, reds + 1); } }";
    let pkg = content_compiler::compile_source_named("nc10-author", source).unwrap();
    let mut pack = pack();
    pack.def.name = "nc10-author";
    pack.vm_skills = content_native_loader::bridge_vm_skill_decls(&pkg)
        .unwrap()
        .decls;
    pack.vm_skills.truncate(1);
    let mut pair = Vec::new();
    for red in [false, true] {
        let mut e = Engine::new_with_extensions(991_102, 4, &[&pack]).unwrap();
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_grant(1, &pack.vm_skills[0].name);
        let sha = e.debug_move_named_card_to_hand(0, "sha");
        for index in 0..e.state_core().players[1].hand.len() {
            let deck_index = e
                .state_core()
                .deck
                .iter()
                .position(|id| {
                    let suit = e.catalog().cards[*id as usize].suit;
                    matches!(
                        suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    ) == red
                })
                .unwrap();
            let core = e.state_core_mut();
            std::mem::swap(&mut core.players[1].hand[index], &mut core.deck[deck_index]);
        }
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: 0,
            dedup_id: 1,
            action: GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        assert_eq!(
            (
                e.open_choice().unwrap().observer,
                e.open_choice().unwrap().spec.prompt
            ),
            (0, "rvm_discard_pick")
        );
        Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        pair.push(e);
    }
    assert_eq!(
        format!("{:?}", pair[0].ai_session(0).input()),
        format!("{:?}", pair[1].ai_session(0).input())
    );
    assert_eq!(
        pair[0].ai_session(0).choice(),
        pair[1].ai_session(0).choice()
    );
    assert_eq!(
        pair[0].ai_session(0).events(),
        pair[1].ai_session(0).events()
    );
    for budget in [0, 48] {
        let planner = BoundedPlanner::new(budget)
            .with_policy_variant(1)
            .with_lookahead(2);
        let p = pair[0].plan_action(0, &planner);
        let q = pair[1].plan_action(0, &planner);
        assert_eq!(format!("{p:?}"), format!("{q:?}"));
        assert_eq!(p.nodes_used, 0);
        for e in &pair {
            let mut restored = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
            assert!(!matches!(
                restored.apply_plan(&p).unwrap(),
                CommandOutcome::Rejected(_)
            ));
        }
        planner.cancel();
        assert!(pair[0].plan_action(0, &planner).cancelled);
    }
}

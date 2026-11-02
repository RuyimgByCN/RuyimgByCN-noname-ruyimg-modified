//! Other-choice roots are public idle advice; explicit parallel reveal remains legal.
use engine_native::ai::{ApplyPlanError, BoundedPlanner, Plan};
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{
    ChoiceOption, ChoiceView, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
    StateSnapshot,
};

fn restored(snapshot: &StateSnapshot, packs: &[&ExtensionPack]) -> Engine {
    let bytes = engine_native::archive::encode_archive(snapshot);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(decoded, *snapshot);
    let typed = Engine::restore_with_extensions(snapshot, packs).unwrap();
    let archived = Engine::restore_with_extensions(&decoded, packs).unwrap();
    let mut expected = snapshot.clone();
    expected.auth_generation += 1; // Restore intentionally invalidates old authority tokens.
    assert_eq!(typed.save(), expected);
    assert_eq!(archived.save(), expected);
    archived
}

fn submit(engine: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    engine.submit(GameCommand {
        actor,
        expected_revision: engine.revision(),
        dedup_id: engine.save().last_dedup + 1,
        action,
    })
}

fn yiji() -> Engine {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_place(0, "sha");
    engine.debug_place(1, "tao");
    engine.debug_place(2, "shan");
    engine.debug_grant(0, "仁德");
    engine.debug_grant(1, "遗计");
    assert!(matches!(
        engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal),
        CommandOutcome::Pending { .. }
    ));
    assert!(matches!(
        engine.ai_session(1).choice(),
        Some(ChoiceView::Own {
            prompt: "trigger_confirm",
            source_skill: Some("遗计"),
            ..
        })
    ));
    engine
}

fn ordinary_pair(dead: bool) -> [Engine; 2] {
    let mut left = yiji().save();
    if dead {
        left.core.players[0].alive = false;
        left.core.players[0].hp = 0;
    }
    let mut right = left.clone();
    std::mem::swap(&mut right.core.players[1].hand[0], &mut right.core.deck[0]);
    std::mem::swap(&mut right.core.players[2].hand[0], &mut right.core.deck[1]);
    assert_ne!(left.core.players[1].hand, right.core.players[1].hand);
    [restored(&left, &[]), restored(&right, &[])]
}

fn record(name: &str, text: &str, snapshots: &[StateSnapshot; 2]) {
    if let Ok(directory) = std::env::var("NC10_OTHER_EVIDENCE") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join(format!("{name}.txt")), text).unwrap();
        for (side, snapshot) in snapshots.iter().enumerate() {
            std::fs::write(
                directory.join(format!("{name}-{side}.ngsa")),
                engine_native::archive::encode_archive(snapshot),
            )
            .unwrap();
        }
    }
}

fn assert_public_idle(pair: [Engine; 2], packs: &[&ExtensionPack], name: &str) {
    let inputs = pair
        .each_ref()
        .map(|e| format!("{:?}", e.ai_session(0).input()));
    let events = pair.each_ref().map(|e| e.ai_session(0).events());
    assert_eq!(inputs[0], inputs[1]);
    assert_eq!(events[0], events[1]);
    assert!(matches!(
        pair[0].ai_session(0).choice(),
        Some(ChoiceView::Other { observer: 1, .. })
    ));
    let originals = pair.each_ref().map(|e| e.save());
    let bytes = originals
        .each_ref()
        .map(engine_native::archive::encode_archive);
    let logs = pair.each_ref().map(|e| e.log().to_vec());
    let mut samples = Vec::new();
    let mut evidence = format!("inputs={inputs:#?}\nevents={events:#?}\n");
    for budget in [48, 64, 0, 1] {
        for depth in [1, 2] {
            for variant in [0, 1, 2] {
                for mode in ["normal", "cancelled", "deadline0"] {
                    let mut planner = BoundedPlanner::new(budget)
                        .with_lookahead(depth)
                        .with_policy_variant(variant);
                    if mode == "cancelled" {
                        planner.cancel();
                    }
                    if mode == "deadline0" {
                        planner = planner.with_deadline_ms(0);
                    }
                    let plans = pair.each_ref().map(|e| e.plan_action(0, &planner));
                    evidence.push_str(&format!("budget={budget} depth={depth} variant={variant} mode={mode} plans={plans:?}\n"));
                    samples.push((plans, mode));
                }
            }
        }
    }
    record(name, &evidence, &originals);
    for (plans, mode) in samples {
        assert_eq!(
            format!("{:?}", plans[0]),
            format!("{:?}", plans[1]),
            "complete Plan must use public inputs only"
        );
        for (engine, plan) in pair.iter().zip(&plans) {
            let expected = Plan {
                seat: 0,
                revision: engine.revision(),
                action: GameAction::NextPhase,
                nodes_used: 0,
                budget_exhausted: false,
                timed_out: false,
                cancelled: mode == "cancelled",
                sim_faults: 0,
            };
            assert_eq!(
                format!("{plan:?}"),
                format!("{expected:?}"),
                "Other root must return before simulation"
            );
            let mut trial = restored(&engine.save(), packs);
            let before = trial.save();
            let before_bytes = engine_native::archive::encode_archive(&before);
            let before_log = trial.log().to_vec();
            if plan.cancelled {
                assert!(matches!(
                    trial.apply_plan(plan),
                    Err(ApplyPlanError::Cancelled)
                ));
            } else {
                assert!(matches!(
                    trial.apply_plan(plan).unwrap(),
                    CommandOutcome::Rejected(RejectReason::ChoicePending)
                ));
            }
            assert_eq!(trial.save(), before);
            assert_eq!(
                engine_native::archive::encode_archive(&trial.save()),
                before_bytes
            );
            assert_eq!(trial.log(), before_log);
        }
    }
    assert_eq!(pair.each_ref().map(|e| e.save()), originals);
    assert_eq!(
        pair.each_ref()
            .map(|e| engine_native::archive::encode_archive(&e.save())),
        bytes
    );
    assert_eq!(pair.each_ref().map(|e| e.log().to_vec()), logs);
}

#[test]
fn ordinary_other_choice_returns_before_simulation() {
    let live = yiji();
    let live_before = live.save();
    let live_log = live.log().to_vec();
    assert!(!live_log.is_empty());
    let live_plan = live.plan_action(0, &BoundedPlanner::new(48));
    assert_eq!((live_plan.nodes_used, live_plan.sim_faults), (0, 0));
    assert_eq!(live.save(), live_before);
    assert_eq!(live.log(), live_log);
    let pair = ordinary_pair(false);
    for engine in &pair {
        assert!(engine.legal_actions(0).is_empty());
        let choice = engine.open_choice().unwrap();
        for action in [
            GameAction::NextPhase,
            GameAction::PlayCard {
                card: engine.debug_hand(0)[0],
                targets: vec![2],
            },
            GameAction::ActivateSkill {
                skill: engine.catalog().skill_by_name("仁德").unwrap(),
            },
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Confirm],
            },
            GameAction::TimeoutAction {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Confirm],
            },
            GameAction::RevealGeneral { slot: 0 },
        ] {
            assert!(
                engine.probe_action(0, &action).is_err(),
                "prototype non-owner unexpectedly authorized {action:?}"
            );
        }
    }
    assert_public_idle(pair, &[], "ordinary");
}

#[test]
fn dead_other_actor_returns_before_simulation() {
    assert_public_idle(ordinary_pair(true), &[], "dead");
}

#[test]
fn other_idle_keeps_explicit_parallel_reveals_and_owner_answers() {
    for compensatory in [false, true] {
        let (engine, actor) = if !compensatory {
            (yiji(), 0)
        } else {
            let mut engine = Engine::new(240, 3);
            engine.debug_set_turn(0, Phase::Play, 1);
            for seat in 0..3 {
                engine.debug_clear_hand(seat);
            }
            let weapon = engine.debug_place(0, "fangtian");
            assert!(matches!(
                submit(
                    &mut engine,
                    0,
                    GameAction::PlayCard {
                        card: weapon,
                        targets: vec![]
                    }
                ),
                CommandOutcome::Accepted { .. }
            ));
            engine.debug_place(1, "shan");
            let sha = engine.debug_place(0, "sha");
            assert!(matches!(
                submit(
                    &mut engine,
                    0,
                    GameAction::PlayCard {
                        card: sha,
                        targets: vec![1, 2]
                    }
                ),
                CommandOutcome::Pending { .. }
            ));
            (engine, 2)
        };
        let mut snapshot = engine.save();
        snapshot.core.mode = engine_native::state::GameMode::Kingdoms;
        snapshot.core.players[actor as usize].general_slots[0] = Some(2);
        let mut engine = restored(&snapshot, &[]);
        let choice = engine.open_choice().unwrap().clone();
        assert_eq!(choice.observer, 1);
        assert!(matches!(
            engine.ai_session(actor).choice(),
            Some(ChoiceView::Other { .. })
        ));
        let before = engine.save();
        for variant in [0, 1, 2] {
            let plan = engine.plan_action(
                actor,
                &BoundedPlanner::new(48)
                    .with_policy_variant(variant)
                    .with_lookahead(2),
            );
            eprintln!("REVEAL_CONTROL compensatory={compensatory} variant={variant} plan={plan:?}");
            assert_eq!(
                plan.action,
                GameAction::NextPhase,
                "existing planner must never have selected RevealGeneral"
            );
        }
        assert_eq!(engine.save(), before);
        // legal_actions omits parallel reveals; formal submission remains authoritative.
        assert!(engine.legal_actions(actor).is_empty());
        assert!(engine.is_legal_action(actor, &GameAction::RevealGeneral { slot: 0 }));
        let out = submit(&mut engine, actor, GameAction::RevealGeneral { slot: 0 });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
        assert_eq!(engine.revision(), before.core.revision + 1);
        assert_eq!(engine.open_choice(), Some(&choice));
        assert!(engine.save().core.players[actor as usize].revealed_generals[0]);
        record(
            if compensatory {
                "compensatory-reveal"
            } else {
                "current-reveal"
            },
            &format!("actor={actor} outcome={out:?} unchanged_choice={choice:?}"),
            &[before, engine.save()],
        );
        let mut continued = restored(&engine.save(), &[]);
        let plan = continued.plan_action(1, &BoundedPlanner::new(48));
        assert!(matches!(plan.action, GameAction::Answer { .. }));
        assert!(!matches!(
            continued.apply_plan(&plan).unwrap(),
            CommandOutcome::Rejected(_)
        ));
        restored(&continued.save(), &[]);
    }
}

#[test]
fn own_answer_and_no_choice_search_still_advance_authority() {
    let mut owner = restored(&yiji().save(), &[]);
    let plan = owner.plan_action(1, &BoundedPlanner::new(48));
    assert_eq!(plan.nodes_used, 0);
    assert!(
        matches!(&plan.action, GameAction::Answer { selection, .. } if selection == &[ChoiceOption::Confirm])
    );
    let before = owner.revision();
    assert!(matches!(
        owner.apply_plan(&plan).unwrap(),
        CommandOutcome::Pending { .. }
    ));
    assert_eq!(owner.revision(), before + 1);
    restored(&owner.save(), &[]);
    let mut active = Engine::new(2306, 3);
    active.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        active.debug_clear_hand(seat);
    }
    let sha = active.debug_place(0, "sha");
    let attack = GameAction::PlayCard {
        card: sha,
        targets: vec![1],
    };
    assert!(active.open_choice().is_none());
    assert!(active.legal_actions(0).contains(&attack));
    for variant in [0, 1, 2] {
        let plan = active.plan_action(
            0,
            &BoundedPlanner::new(48)
                .with_policy_variant(variant)
                .with_lookahead(2),
        );
        assert!(plan.nodes_used > 0);
        assert!(matches!(plan.action, GameAction::PlayCard { .. }));
        let mut trial = restored(&active.save(), &[]);
        assert!(!matches!(
            trial.apply_plan(&plan).unwrap(),
            CommandOutcome::Rejected(_)
        ));
    }
}

fn cached_pair() -> (ExtensionPack, [Engine; 2]) {
    let dsl = "skill nc10_other_cached(99107928) { active play_phase targets 0 0 { let reds = hand_color_count(3, red); effect deal_damage(self,2,1); effect deal_damage(self,self,reds+1); } }";
    let package = content_compiler::compile_source_named("nc10-other-cached", dsl).unwrap();
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "nc10-other-cached",
            version: 1,
            description: "nested native ask over private VM effects",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&package)
            .unwrap()
            .decls,
        init_hook: Default::default(),
        enabled: true,
    };
    let pair = [false, true].map(|red| {
        let mut engine = Engine::new_with_extensions(99107928, 3, &[&pack]).unwrap();
        engine.debug_set_turn(0, Phase::Play, 1);
        engine.debug_grant(0, "nc10_other_cached");
        engine.debug_grant(1, "遗计");
        engine.debug_place(0, "sha");
        for index in 0..engine.state_core().players[2].hand.len() {
            let deck_index = engine
                .state_core()
                .deck
                .iter()
                .position(|id| {
                    matches!(
                        engine.catalog().cards[*id as usize].suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    ) == red
                })
                .unwrap();
            let core = engine.state_core_mut();
            std::mem::swap(&mut core.players[2].hand[index], &mut core.deck[deck_index]);
        }
        let skill = engine.catalog().skill_by_name("nc10_other_cached").unwrap();
        assert!(matches!(
            submit(&mut engine, 0, GameAction::ActivateSkill { skill }),
            CommandOutcome::Pending { .. }
        ));
        assert_eq!(engine.open_choice().unwrap().spec.prompt, "trigger_confirm");
        engine
    });
    assert!(!pair[0].log().is_empty());
    assert_eq!(pair[0].event_view_for(0), pair[1].event_view_for(0));
    assert_eq!(
        format!("{:?}", pair[0].ai_session(0).input()),
        format!("{:?}", pair[1].ai_session(0).input())
    );
    let pair = pair
        .each_ref()
        .map(|engine| restored(&engine.save(), &[&pack]));
    (pack, pair)
}

#[test]
fn ordinary_other_choice_over_real_cached_vm_effects_is_public_idle() {
    let (pack, pair) = cached_pair();
    let amounts = pair.each_ref().map(|engine| {
        engine
            .save()
            .resolution
            .parks
            .iter()
            .flat_map(|park| {
                let engine_native::MachineSnapshot::Rvm { commit_pending, .. } = &park.machine
                else {
                    return Vec::new();
                };
                engine_native::rvm_ops::dec_effects(commit_pending)
                    .unwrap()
                    .into_iter()
                    .filter_map(|effect| {
                        if let rule_api::EffectRequest::DealDamage { amount, .. } = effect {
                            Some(amount)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });
    assert_ne!(
        amounts[0], amounts[1],
        "reachable private pending effects must differ"
    );
    eprintln!("ORDINARY_OTHER_VM private pending damage={amounts:?}");
    assert_public_idle(pair, &[&pack], "ordinary-cached-vm");
}

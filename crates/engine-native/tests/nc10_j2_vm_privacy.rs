//! NC10/J2: an accepted effect continuation must not expose cached private queries to AI.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{
    ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot, Phase,
};

const DSL: &str = "skill nc10_j2_cached(99107899) { active play_phase targets 0 0 { let reds = hand_color_count(2, red); effect resolve_ol_jie_danshou(self); effect deal_damage(self,self,reds+1); } }";

fn pack() -> ExtensionPack {
    let package = content_compiler::compile_source_named("nc10-j2-cached", DSL).unwrap();
    let decls = content_native_loader::bridge_vm_skill_decls(&package)
        .unwrap()
        .decls;
    let known = decls.iter().map(|decl| decl.name.clone()).collect();
    let generals = content_native_loader::bridge_generals_with_known(
        &serde_json::json!({
            "pack": {"name": "nc10-j2-cached", "version": 1},
            "generals": [{"name": "author", "hp": 4, "faction": "qun", "gender": "male",
                "skills": decls.iter().map(|decl| serde_json::json!({"name": decl.name})).collect::<Vec<_>>() }]
        }).to_string(),
        &known,
    ).unwrap().generals;
    ExtensionPack {
        def: ExtensionDef {
            name: "nc10-j2-cached",
            version: 1,
            description: "cached private query",
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

fn submit(engine: &mut Engine, action: GameAction) {
    let outcome = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: engine.save().last_dedup + 1,
        action,
    });
    assert!(
        !matches!(outcome, CommandOutcome::Rejected(_)),
        "{outcome:?}"
    );
}

fn answer(engine: &mut Engine, selection: Vec<O>) {
    let choice_id = engine.open_choice().unwrap().id;
    submit(
        engine,
        GameAction::Answer {
            choice_id,
            selection,
        },
    );
}

fn fixture(pack: &ExtensionPack, red: bool) -> Engine {
    let mut engine = Engine::new_with_extensions(99107868, 4, &[pack]).unwrap();
    engine.assign_general_named(0, "author").unwrap();
    engine.debug_set_turn(0, Phase::Play, 1);
    engine.debug_set_hp(0, 4);
    engine.debug_clear_hand(0);
    let spare = engine.debug_place(0, "sha");
    engine.debug_place(0, "tao");
    engine.debug_place(0, "shan");
    assert_eq!(engine.state_core().players[1].hand.len(), 4);
    for index in 0..4 {
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
        std::mem::swap(&mut core.players[1].hand[index], &mut core.deck[deck_index]);
    }
    let skill = engine.catalog().skill_by_name("nc10_j2_cached").unwrap();
    submit(&mut engine, GameAction::ActivateSkill { skill });
    for (prompt, selection) in [
        ("rvm_danshou_option", vec![O::Tag(1)]),
        ("rvm_danshou_target", vec![O::Player(0)]),
        ("rvm_danshou_card", vec![O::Card(spare)]),
    ] {
        assert_eq!(engine.open_choice().unwrap().spec.prompt, prompt);
        answer(&mut engine, selection);
    }
    let choice = engine.open_choice().unwrap();
    assert_eq!(choice.spec.prompt, "rvm_danshou_discard");
    assert_eq!((choice.spec.min, choice.spec.options.len()), (1, 2));
    Engine::restore_with_extensions(&engine.save(), &[pack]).unwrap();
    engine
}

fn cached_damage(engine: &Engine) -> Vec<u32> {
    engine
        .save()
        .resolution
        .parks
        .iter()
        .flat_map(|park| {
            let MachineSnapshot::Rvm {
                paused,
                commit_pick,
                committing,
                commit_state,
                commit_pending,
                ..
            } = &park.machine
            else {
                return Vec::new();
            };
            assert!(
                paused.is_none()
                    && commit_pick.is_none()
                    && *committing
                    && !commit_state.is_empty()
            );
            engine_native::rvm_ops::dec_effects(commit_pending)
                .unwrap()
                .iter()
                .filter_map(|effect| {
                    if let rule_api::EffectRequest::DealDamage { amount, .. } = effect {
                        Some(*amount)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

#[test]
fn cached_private_damage_cannot_change_j2_public_choice_plan() {
    let pack = pack();
    let pair = [fixture(&pack, false), fixture(&pack, true)];
    assert_eq!(cached_damage(&pair[0]), vec![1]);
    assert_eq!(cached_damage(&pair[1]), vec![5]);
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
    let original = [pair[0].save(), pair[1].save()];
    for budget in [0, 48] {
        let planner = BoundedPlanner::new(budget)
            .with_policy_variant(1)
            .with_lookahead(2);
        let plans = [
            pair[0].plan_action(0, &planner),
            pair[1].plan_action(0, &planner),
        ];
        eprintln!(
            "NC10_J2_PRIVATE budget={budget} black={:?} red={:?}",
            plans[0], plans[1]
        );
        assert_eq!(
            format!("{:?}", plans[0]),
            format!("{:?}", plans[1]),
            "cached private damage must not select the discard"
        );
        for (engine, plan) in pair.iter().zip(&plans) {
            assert_eq!((plan.nodes_used, plan.sim_faults), (0, 0));
            let snapshot = engine.save();
            let bytes = engine_native::archive::encode_archive(&snapshot);
            let mut restored = Engine::restore_with_extensions(
                &engine_native::archive::decode_archive(&bytes).unwrap(),
                &[&pack],
            )
            .unwrap();
            assert!(!matches!(
                restored.apply_plan(plan).unwrap(),
                CommandOutcome::Rejected(_)
            ));
            Engine::restore_with_extensions(&restored.save(), &[&pack]).unwrap();
        }
    }
    for engine in &pair {
        let mut sampled = engine.clone();
        redact_opponent_hands(&mut sampled, 0);
        assert_eq!(sampled.save().resolution, engine.save().resolution);
        assert_eq!(sampled.state_core().players, engine.state_core().players);
        assert_eq!(
            sampled.state_core().vm_aux.piles,
            engine.state_core().vm_aux.piles
        );
        Engine::restore_with_extensions(&sampled.save(), &[&pack]).unwrap();
    }
    assert_eq!(
        [pair[0].save(), pair[1].save()],
        original,
        "planning changed authority"
    );
}

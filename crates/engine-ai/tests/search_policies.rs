//! Phase 8 acceptance: isolation, reproducibility, stale discard.

use engine_ai::{
    apply_if_fresh, build_information_set, legal_actions, AiRuntime, BeamSearch, HeuristicSearch,
    MctsSearch, SearchBudget, StateFork,
};
use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;

fn start(seed: u64) -> Engine {
    let mut eng = Engine::new();
    let r = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(r, CommandResult::Accepted { .. }));
    eng
}

#[test]
fn fork_does_not_touch_authority_hash() {
    let eng = start(5);
    let h = eng.state_hash();
    let mut fork = StateFork::from_authority(&eng, 99);
    let actor = eng.state().turn.current;
    let acts = legal_actions(fork.engine(), actor);
    if let Some(a) = acts.first() {
        let cmd = a.to_command(actor, 0);
        let _ = fork.apply(cmd);
    }
    assert_eq!(eng.state_hash(), h);
}

#[test]
fn heuristic_reproducible_under_fixed_budget() {
    let eng = start(8);
    let actor = eng.state().turn.current;
    let budget = SearchBudget {
        max_nodes: 40,
        max_depth: 4,
    };
    let rt = AiRuntime::new(123);
    let mut p1 = HeuristicSearch;
    let mut p2 = HeuristicSearch;
    let d1 = rt.decide(&eng, actor, &mut p1, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut p2, budget).unwrap();
    assert_eq!(d1.command.kind, d2.command.kind);
    assert_eq!(d1.base_revision, d2.base_revision);
}

#[test]
fn mcts_reproducible_under_fixed_nodes() {
    let eng = start(9);
    // advance to play for more actions
    let mut eng = eng;
    let actor = eng.state().turn.current;
    eng.submit(Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    let actor = eng.state().turn.current;
    let budget = SearchBudget {
        max_nodes: 48,
        max_depth: 3,
    };
    let rt = AiRuntime::new(7);
    let mut a = MctsSearch::default();
    let mut b = MctsSearch::default();
    let d1 = rt.decide(&eng, actor, &mut a, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut b, budget).unwrap();
    assert_eq!(d1.command, d2.command);
}

#[test]
fn beam_reproducible() {
    let eng = start(10);
    let actor = eng.state().turn.current;
    let budget = SearchBudget {
        max_nodes: 30,
        max_depth: 3,
    };
    let rt = AiRuntime::new(3);
    let mut a = BeamSearch { width: 3 };
    let mut b = BeamSearch { width: 3 };
    let d1 = rt.decide(&eng, actor, &mut a, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut b, budget).unwrap();
    assert_eq!(d1.command, d2.command);
}

#[test]
fn info_set_no_foreign_hand() {
    let eng = start(12);
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let view = build_information_set(&eng, p1);
    for cid in eng.state().hand_cards(p2) {
        assert!(!view
            .observer
            .known_cards
            .iter()
            .any(|c| c.id == cid && c.def.is_some()));
    }
}

#[test]
fn apply_fresh_works_then_stale_fails() {
    let mut eng = start(15);
    let actor = eng.state().turn.current;
    let rt = AiRuntime::new(1);
    let mut policy = HeuristicSearch;
    let decision = rt
        .decide(&eng, actor, &mut policy, SearchBudget::default())
        .unwrap();
    // First apply succeeds
    apply_if_fresh(&mut eng, &decision).unwrap();
    // Same decision now stale
    let err = apply_if_fresh(&mut eng, &decision).unwrap_err();
    assert!(matches!(err, engine_ai::AiError::Stale { .. }));
}

#[test]
fn up37_n01_danshou_fenli_pingkou_ai_search_rng_isolation_and_reproducibility() {
    use engine_api::{LegalAction, SkillDefId};
    use engine_core::{effect::run_effects, transaction::Transaction, Phase};
    use rule_api::EffectRequest;

    const DANSHOU_SRC: &str =
        include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill");
    const FENLI_SRC: &str =
        include_str!("../../../content/original/ported/onlyOL/ol_fenli.skill");
    const PINGKOU_SRC: &str =
        include_str!("../../../content/original/ported/onlyOL/ol_pingkou.skill");

    let source = format!("{DANSHOU_SRC}\n{FENLI_SRC}\n{PINGKOU_SRC}");
    let pkg = content_compiler::compile_source_named("up37-n01-ai", &source).unwrap();

    let mut eng = start(20261024);
    eng.load_package(&pkg).unwrap();
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(3_750_129_445),
            },
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(2_744_558_996),
            },
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(2_376_678_765),
            },
            EffectRequest::DrawCards {
                player: actor,
                count: 4,
            },
        ],
    )
    .unwrap();
    let snap = engine_core::snapshot::Snapshot::capture(&tx.commit().state);
    eng.restore_snapshot(&snap).unwrap();

    let actions = legal_actions(&eng, actor);
    let danshou_action = actions
        .iter()
        .find(|a| matches!(a, LegalAction::ActivateSkill { targets, card_targets, .. } if targets.is_empty() && card_targets.is_empty()))
        .cloned()
        .expect("legal_actions must include ol_jie_danshou ActivateSkill");

    let authority_hash = eng.state_hash();
    let authority_rng = eng.state().rng;
    let budget = SearchBudget {
        max_nodes: 32,
        max_depth: 3,
    };

    // HeuristicSearch, BeamSearch, and MctsSearch must keep authority RuleRng and state_hash untouched,
    // stay within bounded node budgets, and be reproducible per seed.
    let rt = AiRuntime::new(20261105);
    let d_heur_1 = rt.decide(&eng, actor, &mut HeuristicSearch, budget).unwrap();
    let d_heur_2 = rt.decide(&eng, actor, &mut HeuristicSearch, budget).unwrap();
    assert_eq!(d_heur_1.command, d_heur_2.command);
    assert!(d_heur_1.nodes_used >= 1 && d_heur_1.nodes_used <= budget.max_nodes);

    let d_beam_1 = rt
        .decide(&eng, actor, &mut BeamSearch { width: 3 }, budget)
        .unwrap();
    let d_beam_2 = rt
        .decide(&eng, actor, &mut BeamSearch { width: 3 }, budget)
        .unwrap();
    assert_eq!(d_beam_1.command, d_beam_2.command);
    assert!(d_beam_1.nodes_used >= 1 && d_beam_1.nodes_used <= budget.max_nodes);

    let d_mcts_1 = rt
        .decide(&eng, actor, &mut MctsSearch::default(), budget)
        .unwrap();
    let d_mcts_2 = rt
        .decide(&eng, actor, &mut MctsSearch::default(), budget)
        .unwrap();
    assert_eq!(d_mcts_1.command, d_mcts_2.command);
    assert!(d_mcts_1.nodes_used >= 1 && d_mcts_1.nodes_used <= budget.max_nodes);

    assert_eq!(eng.state_hash(), authority_hash);
    assert_eq!(eng.state().rng, authority_rng);

    // Activate ol_jie_danshou and verify AI search resolves the 4-option choice barrier cleanly.
    let rev = eng.state().revision;
    assert!(matches!(
        eng.submit(danshou_action.to_command(actor, rev)),
        CommandResult::Accepted { .. }
    ));
    assert!(eng.state().has_pending_choice());
    let option_choices = legal_actions(&eng, actor);
    assert_eq!(
        option_choices.len(),
        4,
        "4 danshou options in legal_actions"
    );

    let choice_hash = eng.state_hash();
    let choice_rng = eng.state().rng;
    let choice_decision = rt
        .decide(&eng, actor, &mut MctsSearch::default(), budget)
        .unwrap();
    assert!(choice_decision.nodes_used >= 1 && choice_decision.nodes_used <= budget.max_nodes);
    assert_eq!(eng.state_hash(), choice_hash);
    assert_eq!(eng.state().rng, choice_rng);
    apply_if_fresh(&mut eng, &choice_decision).unwrap();
}

#[test]
fn up37_r10_potshanxi_twgonghuan_olhuiyun_sxrmzhengsi_ai_finite_valuation_and_rng_isolation() {
    use engine_api::{LegalAction, SkillDefId};
    use engine_core::{effect::run_effects, transaction::Transaction, Phase};
    use rule_api::EffectRequest;

    const POTSHANXI_SRC: &str =
        include_str!("../../../content/original/ported/bingshi/potshanxi.skill");
    const TWLUANCHOU_SRC: &str =
        include_str!("../../../content/original/ported/tw/twluanchou.skill");
    const OLHUIYUN_SRC: &str =
        include_str!("../../../content/original/ported/sp/olhuiyun.skill");
    const SXRMZHENGSI_SRC: &str =
        "skill sxrmzhengsi(3539390052) { active play_phase uses_per_turn 1 { effect draw_cards(self, 1); } }";

    let source = format!("{POTSHANXI_SRC}\n{TWLUANCHOU_SRC}\n{OLHUIYUN_SRC}\n{SXRMZHENGSI_SRC}");
    let pkg = content_compiler::compile_source_named("up37-r10-ai", &source).unwrap();

    let mut eng = start(20261025);
    eng.load_package(&pkg).unwrap();
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(2_372_417_115), // potshanxi
            },
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(2_306_392_424), // twluanchou
            },
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(3_503_235_693), // twgonghuan
            },
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(4_044_368_124), // olhuiyun
            },
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(3_539_390_052), // sxrmzhengsi
            },
        ],
    )
    .unwrap();
    let snap = engine_core::snapshot::Snapshot::capture(&tx.commit().state);
    eng.restore_snapshot(&snap).unwrap();

    let actions = legal_actions(&eng, actor);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, LegalAction::ActivateSkill { .. })),
        "R10 active/view-as skills must be enumerated in legal_actions"
    );

    let authority_hash = eng.state_hash();
    let authority_rng = eng.state().rng;
    let budget = SearchBudget {
        max_nodes: 32,
        max_depth: 3,
    };
    let rt = AiRuntime::new(9001);

    let d1 = rt.decide(&eng, actor, &mut HeuristicSearch, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut HeuristicSearch, budget).unwrap();
    assert_eq!(d1.command, d2.command);
    assert!(d1.nodes_used >= 1 && d1.nodes_used <= budget.max_nodes);

    let m1 = rt
        .decide(&eng, actor, &mut MctsSearch::default(), budget)
        .unwrap();
    let m2 = rt
        .decide(&eng, actor, &mut MctsSearch::default(), budget)
        .unwrap();
    assert_eq!(m1.command, m2.command);
    assert!(m1.nodes_used >= 1 && m1.nodes_used <= budget.max_nodes);

    assert_eq!(eng.state_hash(), authority_hash);
    assert_eq!(eng.state().rng, authority_rng);
    apply_if_fresh(&mut eng, &d1).unwrap();
}

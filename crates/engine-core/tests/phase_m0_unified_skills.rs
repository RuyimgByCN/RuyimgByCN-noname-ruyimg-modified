//! M0 unified skill surface: DSL effects + phase_end + ActivateSkill.

use content_compiler::compile_source;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{Engine, BOOTSTRAP_JIANXIONG};
use rule_api::EffectRequest;

const PACK: &str = include_str!("../../../content/standard/standard_m0.skill");

fn start_cmd(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

#[test]
fn package_compiles_triggers_and_actives() {
    let pkg = compile_source(PACK).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| t.event == "damage"));
    assert!(pkg.trigger_index.iter().any(|t| t.event == "phase_end"));
    assert_eq!(pkg.active_index.len(), 1);
    assert_eq!(pkg.modifier_index.len(), 2);
}

#[test]
fn jianxiong_ignores_cardless_damage() {
    let pkg = compile_source(PACK).unwrap();
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(start_cmd(7, 2));
    let mut tx = Transaction::begin(eng.state().clone());
    let owner = PlayerId::new(1);
    for player in [owner, PlayerId::new(2)] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        }],
    )
    .unwrap();
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::new(2),
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn fankui_opens_optional_card_take() {
    let pkg = compile_source(PACK).unwrap();
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(start_cmd(11, 2));

    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(5),
        }],
    )
    .unwrap();
    if tx.state().hand_count(p2) == 0 {
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: p2,
                count: 2,
            }],
        )
        .unwrap();
    }
    let p1_before = tx.state().hand_count(p1);
    let p2_before = tx.state().hand_count(p2);
    assert!(p2_before >= 1);

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: p2,
            target: p1,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert!(tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(p1), p1_before);
    assert_eq!(tx.state().hand_count(p2), p2_before);
}

#[test]
fn biyue_draws_on_phase_end_event() {
    let pkg = compile_source(PACK).unwrap();
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(start_cmd(33, 2));
    let owner = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(4),
        }],
    )
    .unwrap();
    let before = tx.state().hand_count(owner);
    let mut fires = TriggerFireRecord::new();
    let result = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(34),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        before + 1,
        "accepted biyue phase_end draws 1"
    );
}

#[test]
fn zhiheng_activate_skill_in_play_phase() {
    let pkg = compile_source(PACK).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&pkg).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 55, "identity-standard").unwrap();
    // bootstrap alone does not commit a revision; mark match as started.
    tx.state_mut().revision = 1;
    let actor = tx.state().turn.current;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: SkillDefId::new(10),
        }],
    )
    .unwrap();
    if tx.state().hand_count(actor) == 0 {
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: actor,
                count: 2,
            }],
        )
        .unwrap();
    }
    let skill_id = tx
        .state()
        .players
        .get(actor)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|sid| {
            tx.state()
                .skills
                .get(*sid)
                .map(|s| s.def.raw() == 10)
                .unwrap_or(false)
        })
        .expect("zhiheng instance");

    let before = tx.state().hand_count(actor);
    let card = tx.state().hand_cards(actor)[0];
    let state = tx.state().clone();
    let (new_state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        pipe.result
    );
    assert_eq!(
        new_state.hand_count(actor),
        before,
        "zhiheng discard1+draw1 keeps hand size"
    );
}

#[test]
fn deal_damage_effect_compiles_and_runs() {
    let src = r#"
skill retaliate(20) {
  trigger damage owner_is_target {
    effect deal_damage(self, event_source, 1);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(start_cmd(9, 2));
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(20),
        }],
    )
    .unwrap();
    let hp2_before = tx.state().players.get(p2).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: p2,
            target: p1,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    // p1 takes damage, retaliates 1 to p2
    let hp2_after = tx.state().players.get(p2).unwrap().hp;
    assert_eq!(hp2_after, hp2_before - 1);
}

//! Golden coverage for 标孟达「赚」.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZHUAN: u32 = 4_034_502_794;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sixiang/stdzhuan.skill"),
    r#"
skill draw_followup(999) locked {
  trigger cards_drawn owner_gains_multiple_outside_own_draw {
    effect draw_cards(self, 1);
  }
}
"#,
);

fn setup(extra_draw_trigger: bool) -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("stdzhuan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_715, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = old_skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZHUAN),
    });
    if extra_draw_trigger {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(999),
        });
        effects.push(EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Play.to_u8(),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn first_damage_draws_three_then_source_takes_one_and_second_damage_does_not_fire() {
    let (state, owner, source) = setup(false);
    let old_card = state.hand_cards(owner)[0];
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let (state, first) = damage(state, source, owner);
    assert_eq!(first, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    assert_eq!(state.resolution.pending_observer, source.raw());
    assert_eq!(state.hand_count(owner), owner_hand + 3);

    let (state, resolved) = resolve(state, source, old_card.raw(), 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(old_card).unwrap().owner, source);
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand + 1);

    let hand_after_first = state.hand_count(owner);
    let (state, second) = damage(state, source, owner);
    assert!(second.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_after_first);
}

#[test]
fn unsourced_damage_only_draws_three() {
    let (state, owner, _) = setup(false);
    let hand = state.hand_count(owner);
    let (state, result) = damage(state, PlayerId::NONE, owner);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand + 3);
}

#[test]
fn draw_triggers_finish_before_source_chooses_a_card() {
    let (state, owner, source) = setup(true);
    let hand = state.hand_count(owner);
    let (state, result) = damage(state, source, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(state.hand_count(owner), hand + 4);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    assert_eq!(state.resolution.continuation_payload[0], owner.raw());
    assert_eq!(state.resolution.continuation_payload[1], source.raw());
}

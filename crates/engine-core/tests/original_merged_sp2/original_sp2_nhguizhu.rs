//! Golden for SP2「鬼助」的全局濒死与每回合限一次语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NHGUIZHU: u32 = 2_454_936_215;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NHGUIZHU && !skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NHGUIZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn offer(tx: &mut Transaction, dying: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn accept(tx: &Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn first_dying_event_is_optional_and_second_in_the_same_turn_is_suppressed() {
    let (mut tx, owner, other) = isolated_state();
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(&tx, owner, 1);
    assert_eq!(state.hand_count(owner), hand + 2);

    let mut same_turn = Transaction::begin(state);
    let effects = offer(&mut same_turn, owner).unwrap();
    assert!(effects.is_empty());
    assert!(!same_turn.state().has_pending_choice());
}

#[test]
fn another_players_new_turn_resets_the_limit_and_dead_owner_cannot_trigger() {
    let (mut tx, owner, other) = isolated_state();
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let mut state = accept(&tx, owner, 2);

    state.turn.current = owner;
    state.turn.phase = Phase::End;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other);

    let mut next_turn = Transaction::begin(state);
    assert_eq!(
        offer(&mut next_turn, other).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let (mut dead_owner, owner, other) = isolated_state();
    dead_owner.state_mut().players.get_mut(owner).unwrap().alive = false;
    let effects = offer(&mut dead_owner, other).unwrap();
    assert!(effects.is_empty());
    assert!(!dead_owner.state().has_pending_choice());
}

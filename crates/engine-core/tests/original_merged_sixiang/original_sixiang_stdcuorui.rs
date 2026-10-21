//! Golden for 四象标牛金「挫锐」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const STDCUORUI: u32 = 2_683_321_015;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated(player_count: u8, seed: u64) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDCUORUI && !skill.locked));

    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(STDCUORUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, players)
}

fn set_hand(tx: &mut Transaction, player: PlayerId, count: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }],
    )
    .unwrap();
    if count > 0 {
        run_effects(tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    }
    assert_eq!(tx.state().hand_count(player), count);
}

fn accept(state: &GameState, owner: PlayerId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn game_start_refills_to_seven_and_does_not_offer_a_noop_choice() {
    let (mut tx, owner, _) = isolated(8, 20_260_716_01);
    set_hand(&mut tx, owner, 2);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::GameStarted { player_count: 8 }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx.state(), owner, 1);
    assert_eq!(state.hand_count(owner), 7);

    let mut filled = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut filled,
        &[DomainEvent::GameStarted { player_count: 8 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!filled.state().has_pending_choice());
}

#[test]
fn only_the_killer_refills_to_the_post_death_alive_count() {
    let (mut tx, owner, players) = isolated(4, 20_260_716_02);
    set_hand(&mut tx, owner, 1);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: players[1],
                killer: owner,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx.state(), owner, 2);
    assert_eq!(state.hand_count(owner), 3);

    let mut not_killer = Transaction::begin(state);
    set_hand(&mut not_killer, owner, 1);
    run_effects(
        &mut not_killer,
        vec![EffectRequest::KillPlayer {
            player: players[2],
            killer: players[3],
        }],
    )
    .unwrap();
    assert_eq!(not_killer.state().hand_count(owner), 1);
    assert!(!not_killer.state().has_pending_choice());
}

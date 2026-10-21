//! Golden for 始计「悬存」：其他角色结束阶段按手牌与体力差摸牌，至多两张。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XUANCUN: u32 = 2_417_220_435;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn setup(owner_hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == XUANCUN && skill.name == "xuancun" && !skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XUANCUN
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_not_target_hand_below_hp"
            && trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2_417_220_435),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 2_417_220_435,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let owner = seats[0];
    let ending_player = seats[1];
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
        skill_def: SkillDefId::new(XUANCUN),
    });
    let current_hand = tx.state().hand_count(owner);
    assert!(owner_hand <= current_hand);
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: current_hand - owner_hand,
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 4;
    owner_state.max_hp = 4;
    (tx, owner, ending_player)
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let state = tx.state();
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
fn hand_deficit_is_capped_at_two_cards_for_the_ending_player() {
    let (mut tx, owner, ending_player) = setup(0);
    let before = tx.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert_eq!(state.hand_count(ending_player), before + 2);
}

#[test]
fn one_card_deficit_draws_exactly_one_card() {
    let (mut tx, owner, ending_player) = setup(3);
    let before = tx.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);
    assert_eq!(state.hand_count(ending_player), before + 1);
}

#[test]
fn full_hand_and_owners_own_end_phase_do_not_offer_xuancun() {
    let (mut full, _, ending_player) = setup(4);
    assert!(dispatch_for_events(
        &mut full,
        &[phase_end(ending_player)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!full.state().has_pending_choice());

    let (mut own_turn, owner, _) = setup(0);
    assert!(dispatch_for_events(
        &mut own_turn,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!own_turn.state().has_pending_choice());
}

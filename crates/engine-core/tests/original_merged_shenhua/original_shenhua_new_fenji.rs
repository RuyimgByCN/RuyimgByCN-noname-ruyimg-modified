//! Golden for 神话再临「奋激」：空手牌角色回合结束时摸二，发动者失去一点体力。

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

const NEW_FENJI: u32 = 3_465_257_033;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == NEW_FENJI && skill.name == "new_fenji" && !skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NEW_FENJI
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_alive_event_player_hand_empty"
            && trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3_465_257_033),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 3_465_257_033,
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_FENJI),
        },
        EffectRequest::DiscardFromHand {
            player: ending_player,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, ending_player)
}

fn end_turn_event(player: PlayerId) -> DomainEvent {
    DomainEvent::PhaseEnded {
        current: player,
        phase: TurnPhaseTag::End,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn empty_handed_turn_player_draws_two_before_owner_loses_hp() {
    let (mut tx, owner, ending_player) = setup();
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(tx.state().hand_count(ending_player), 0);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[end_turn_event(ending_player)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(ending_player), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}

#[test]
fn nonempty_turn_player_does_not_offer_fenji() {
    let (mut tx, owner, ending_player) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: ending_player,
            count: 1,
        }],
    )
    .unwrap();
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let target_hand = tx.state().hand_count(ending_player);

    assert!(dispatch_for_events(
        &mut tx,
        &[end_turn_event(ending_player)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(ending_player), target_hand);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn owner_can_trigger_fenji_at_the_end_of_their_own_turn() {
    let (mut tx, owner, _) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[end_turn_event(owner)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}

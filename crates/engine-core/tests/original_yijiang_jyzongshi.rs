//! Golden coverage for 简雍「纵适」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JYZONGSHI: u32 = 2_485_328_360;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn inject_discard(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner,
        face_up: true,
    });
    state.zones.get_mut(discard).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_is_initiator: bool) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (initiator, target) = (seats[0], seats[1]);
    let owner = if owner_is_initiator {
        initiator
    } else {
        target
    };
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JYZONGSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let initiator_card = inject_discard(tx.state_mut(), initiator, 99_001);
    let target_card = inject_discard(tx.state_mut(), target, 99_002);
    (
        tx.commit().state,
        initiator,
        target,
        initiator_card,
        target_card,
    )
}

fn open_finished(
    state: GameState,
    initiator: PlayerId,
    target: PlayerId,
    initiator_card: CardId,
    target_card: CardId,
    winner: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let event = DomainEvent::HandCompareFinished {
        player: initiator,
        target,
        player_card: initiator_card,
        target_card,
        winner,
    };
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    committed.state
}

fn accept(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_jyzongshi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == JYZONGSHI && skill.name == "jyzongshi" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JYZONGSHI
            && trigger.event == "hand_compare_finished"
            && trigger.filter == "owner_hand_compare_participant_result_card_in_discard"
            && trigger.optional
    }));
}

#[test]
fn result_card_matches_initiator_target_win_loss_and_tie_semantics() {
    for (index, owner_is_initiator, winner_kind) in [
        (0_u64, true, 0_u8),
        (1, true, 1),
        (2, true, 2),
        (3, false, 0),
        (4, false, 1),
        (5, false, 2),
    ] {
        let (state, initiator, target, initiator_card, target_card) =
            setup(2_026_072_300 + index, owner_is_initiator);
        let owner = if owner_is_initiator {
            initiator
        } else {
            target
        };
        let opponent = if owner_is_initiator {
            target
        } else {
            initiator
        };
        let winner = match winner_kind {
            0 => owner,
            1 => opponent,
            _ => PlayerId::NONE,
        };
        let expected = if owner_is_initiator {
            if winner == owner {
                target_card
            } else {
                initiator_card
            }
        } else if winner == owner {
            initiator_card
        } else {
            target_card
        };
        let other = if expected == initiator_card {
            target_card
        } else {
            initiator_card
        };
        let state = open_finished(
            state,
            initiator,
            target,
            initiator_card,
            target_card,
            winner,
        );
        let state = accept(state, owner, 9_000 + index);
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(expected).unwrap().zone, hand);
        assert_eq!(state.cards.get(other).unwrap().zone, discard);
    }
}

#[test]
fn unavailable_designated_card_does_not_prompt() {
    let (mut state, initiator, target, initiator_card, target_card) = setup(2_026_072_306, true);
    let hand = state.zones.find(ZoneKind::Hand, initiator).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    state
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .retain(|card| *card != initiator_card);
    state
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(initiator_card);
    state.cards.get_mut(initiator_card).unwrap().zone = hand;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HandCompareFinished {
            player: initiator,
            target,
            player_card: initiator_card,
            target_card,
            winner: target,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}

//! Golden coverage for 从俭（shenhua 张绣，ABI 524）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DRLT_CONGJIAN: u32 = 3_110_735_242;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/drlt_congjian.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    source: PlayerId,
    other: PlayerId,
    hand_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("congjian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let other = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DRLT_CONGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_130),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        source,
        other,
        hand_card: card,
    }
}

fn fire_used(tx: Transaction, user: PlayerId, card: CardId, targets: Vec<PlayerId>) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn alloc_trick_card(tx: &mut Transaction, zone: engine_api::ZoneId) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_131),
        kind: CardKind::TrickNanman,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_multi_target_trick_contract() {
    let compiled = content_compiler::compile_source_named("congjian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 524);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DRLT_CONGJIAN)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_is_target_multi_target_trick_has_hand_equip"
    );
    assert!(trigger.optional);
}

#[test]
fn gives_card_to_other_target_and_draws_one() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // 南蛮入侵 targets the owner and another player.
    let source_hand = tx.state().zones.find(ZoneKind::Hand, setup.source).unwrap();
    let trick = alloc_trick_card(&mut tx, source_hand);
    let tx = fire_used(tx, setup.source, trick, vec![setup.owner, setup.other]);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Recipient choice: only the other target is offered.
    assert!(state.has_pending_choice(), "recipient choice pending");
    state = resolve_choice(state, setup.owner, vec![setup.other.raw()], 2);
    // Card choice: the single hand card.
    assert!(state.has_pending_choice(), "card choice pending");
    let hand_before = state.hand_count(setup.owner);
    state = resolve_choice(state, setup.owner, vec![setup.hand_card.raw()], 3);

    assert_eq!(
        state.hand_count(setup.owner),
        hand_before,
        "gave one, drew one"
    );
    assert!(
        state.hand_cards(setup.other).contains(&setup.hand_card),
        "the other target received the given card"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_leaves_everything_in_place() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let source_hand = tx.state().zones.find(ZoneKind::Hand, setup.source).unwrap();
    let trick = alloc_trick_card(&mut tx, source_hand);
    let tx = fire_used(tx, setup.source, trick, vec![setup.owner, setup.other]);
    let state = tx.commit().state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = next;
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_eq!(state.hand_count(setup.other), 0);
    assert!(!state.has_pending_choice());
}

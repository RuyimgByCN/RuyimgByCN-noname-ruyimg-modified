//! Golden coverage for 固政（shenhua 张昭张纮，ABI 513）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const GUZHENG: u32 = 4_159_731_006;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/guzheng.skill");

fn fire_discard_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Discard,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
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

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

/// Owner holds 固政; `discarder` discards two cards into the discard pile during their own
/// Discard phase; the discard-phase slice base is armed like the engine's phase transition.
fn setup() -> (GameState, PlayerId, PlayerId, Vec<CardId>) {
    let package = content_compiler::compile_source_named("guzheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, discarder) = (seats[0], seats[1]);
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
        skill_def: SkillDefId::new(GUZHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = discarder;
    // The discard phase begins: arm the one-based lost-card slice base.
    let lost_len = tx.state().turn.lost_cards.len() as i32;
    tx.state_mut().turn.set_flag(
        engine_core::state::turn_flag::DISCARD_PHASE_LOST_BASE,
        lost_len.saturating_add(1),
    );
    let c1 = alloc_card(&mut tx, discarder, CardKind::Sha, 96_120, Suit::Spade);
    let c2 = alloc_card(&mut tx, discarder, CardKind::Shan, 96_121, Suit::Heart);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, discarder).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![c1, c2],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    (tx.commit().state, owner, discarder, vec![c1, c2])
}

#[test]
fn package_exposes_discard_phase_return_contract() {
    let compiled = content_compiler::compile_source_named("guzheng", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 513);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == GUZHENG)
        .unwrap();
    assert_eq!(trigger.event, "phase_discard_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_discarded_in_discard_phase"
    );
    assert!(trigger.optional);
}

#[test]
fn returns_one_discard_and_owner_takes_the_rest() {
    let (state, owner, discarder, discarded) = setup();
    let tx = fire_discard_end(Transaction::begin(state), discarder);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, owner, vec![1], 1);
    assert!(state.has_pending_choice(), "return-card choice pending");
    state = resolve_choice(state, owner, vec![discarded[0].raw()], 2);
    assert!(state.has_pending_choice(), "take-rest choice pending");
    state = resolve_choice(state, owner, vec![1], 3);

    assert!(
        state.hand_cards(discarder).contains(&discarded[0]),
        "the discarder regained their returned card"
    );
    assert!(
        state.hand_cards(owner).contains(&discarded[1]),
        "the owner took the remaining discard"
    );
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(discarder), 1);
}

#[test]
fn returns_one_discard_and_gives_up_the_rest() {
    let (state, owner, discarder, discarded) = setup();
    let tx = fire_discard_end(Transaction::begin(state), discarder);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![1], 1);
    state = resolve_choice(state, owner, vec![discarded[1].raw()], 2);
    state = resolve_choice(state, owner, vec![0], 3);

    assert!(
        state.hand_cards(discarder).contains(&discarded[1]),
        "the discarder regained their returned card"
    );
    assert_eq!(
        state.hand_count(owner),
        0,
        "the owner declined the remaining discards"
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state.cards.get(discarded[0]).unwrap().zone,
        discard,
        "the unreturned discard stays in the discard pile"
    );
}

#[test]
fn skipping_the_choice_keeps_everything_in_place() {
    let (state, owner, discarder, discarded) = setup();
    let tx = fire_discard_end(Transaction::begin(state), discarder);
    let state = tx.commit().state;
    // Decline the optional confirm.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
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
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(discarder), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in discarded {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
}

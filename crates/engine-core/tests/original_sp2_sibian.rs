//! Golden coverage for 思变（sp2 张温，ABI 470）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SIBIAN: u32 = 3_021_138_179;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/sibian.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
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

fn fire_draw(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn empty_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand),
            })
            .collect(),
    )
    .unwrap();
}

/// Seed `specs` (rank, def) onto the top of the draw pile; returns their ids.
fn seed_pile(tx: &mut Transaction, specs: &[(u8, u32)]) -> Vec<CardId> {
    let pile = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let mut ids = Vec::new();
    for (rank, def) in specs {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(*def),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: *rank,
            weapon_range: 0,
            zone: pile,
            owner: PlayerId::NONE,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(pile).unwrap().cards.push(card);
        ids.push(card);
    }
    ids
}

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SIBIAN),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn gains_extremes_gives_middle_to_min_hand_player() {
    let package = content_compiler::compile_source_named("sibian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 470);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, other);
    // Top of the pile: ranks 3, 5, 5, 9 (9 = max, 3 = min -> gained; 5,5 -> middle).
    seed_pile(
        &mut tx,
        &[(3, 96_000), (5, 96_001), (5, 96_002), (9, 96_003)],
    );
    let pile_len_before = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .and_then(|id| tx.state().zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let tx = fire_draw(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "min-hand target choice pending");
    assert_eq!(state.hand_count(owner), 2, "gained the max and min cards");
    assert_eq!(
        state.turn.draw_phase_fixed_count(),
        Some(0),
        "ordinary draw skipped"
    );
    state = drive_choice(state, owner, vec![other.raw()], false);
    assert_eq!(state.hand_count(other), 2, "received the middle cards");
    let pile_len = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .and_then(|id| state.zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();
    assert_eq!(pile_len, pile_len_before - 4, "four cards left the pile");
}

#[test]
fn cancel_sends_middle_to_discard() {
    let package = content_compiler::compile_source_named("sibian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, other);
    seed_pile(
        &mut tx,
        &[(3, 96_000), (5, 96_001), (5, 96_002), (9, 96_003)],
    );

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let tx = fire_draw(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![], true);
    assert_eq!(state.hand_count(owner), 2, "extremes still gained");
    assert_eq!(state.hand_count(other), 0, "middle not given");
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let discard_len = state.zones.get(discard).map(|z| z.cards.len()).unwrap();
    assert!(discard_len >= 2, "middle cards went to the discard pile");
}

//! Golden coverage for 制衡（collab 经典孙权，ABI 467）.

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

const DCZHIHENG: u32 = 2_730_395_750;
const SOURCE: &str = include_str!("../../../../content/original/ported/collab/dczhiheng.skill");

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

fn fire_play(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Play,
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

/// Deterministic hands: empty both, seed owner with the given cards. Returns their ids.
fn seed_owner_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    target: PlayerId,
    specs: &[(u32, CardKind, Suit, u8)],
) -> Vec<CardId> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for (zone_kind, player) in [(ZoneKind::Hand, owner), (ZoneKind::Hand, target)] {
        let zone_id = tx.state().zones.find(zone_kind, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(zone_id).unwrap().cards.clone();
        let moves = cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(zone_id),
            })
            .collect();
        run_effects(tx, moves).unwrap();
    }
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut ids = Vec::new();
    for (def, kind, suit, rank) in specs {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(*def),
            kind: *kind,
            suit: *suit,
            rank: *rank,
            weapon_range: 0,
            zone: owner_hand,
            owner,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(owner_hand)
            .unwrap()
            .cards
            .push(card);
        ids.push(card);
    }
    ids
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCZHIHENG),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
}

#[test]
fn discards_and_draws_equal_with_bonus_when_all_hand_discarded() {
    let package = content_compiler::compile_source_named("dczhiheng-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 467);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let ids = seed_owner_hand(
        &mut tx,
        owner,
        target,
        &[
            (96_000, CardKind::Sha, Suit::Spade, 5),
            (96_001, CardKind::Sha, Suit::Heart, 6),
            (96_002, CardKind::Tao, Suit::Club, 7),
            (96_003, CardKind::Tao, Suit::Diamond, 8),
        ],
    );
    assert_eq!(tx.state().hand_count(owner), 4);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "discard choice pending");
    // Discard all four hand cards -> draw 4 + 1 bonus.
    let all = ids.iter().map(|id| id.raw()).collect();
    state = drive_choice(state, owner, all, false);
    assert_eq!(state.hand_count(owner), 5, "drew 4 + 1 bonus");
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        state.players.get(owner).unwrap().max_hp as i32,
        "no hp change"
    );
}

#[test]
fn no_bonus_when_hand_kept() {
    let package = content_compiler::compile_source_named("dczhiheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let ids = seed_owner_hand(
        &mut tx,
        owner,
        target,
        &[
            (96_000, CardKind::Sha, Suit::Spade, 5),
            (96_001, CardKind::Sha, Suit::Heart, 6),
        ],
    );
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    // Discard only one of the two hand cards -> draw exactly 1.
    state = drive_choice(state, owner, vec![ids[0].raw()], false);
    assert_eq!(state.hand_count(owner), 2, "1 kept + 1 drawn");
}

#[test]
fn equip_cards_are_discardable() {
    let package = content_compiler::compile_source_named("dczhiheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let ids = seed_owner_hand(
        &mut tx,
        owner,
        target,
        &[
            (96_000, CardKind::Sha, Suit::Spade, 5),
            (96_001, CardKind::Weapon, Suit::Club, 7),
        ],
    );
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    // A weapon card in the hand zone is a discardable hand-equip card: discarding it
    // (while keeping one hand card) draws exactly one.
    state = drive_choice(state, owner, vec![ids[1].raw()], false);
    assert_eq!(state.hand_count(owner), 2, "1 kept + 1 drawn");
}

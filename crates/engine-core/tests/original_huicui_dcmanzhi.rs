//! Golden coverage for 蛮智（huicui 孟优，ABI 475）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCMANZHI: u32 = 4_194_306_664;
const GIVE_MARK: u32 = 5_0000_0001;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcmanzhi.skill");

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
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

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
        skill_def: SkillDefId::new(DCMANZHI),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
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

fn seed_hand(tx: &mut Transaction, player: PlayerId, specs: &[(CardKind, u32)]) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    specs
        .iter()
        .map(|(kind, def)| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: *kind,
                suit: Suit::Spade,
                rank: 5,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

#[test]
fn give_branch_target_hands_cards_and_uses_sha() {
    let package = content_compiler::compile_source_named("dcmanzhi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 475);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    // Empty both hands so the virtual Sha cannot be dodged and card counts are exact.
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    // Target holds a Sha + a Tao to hand over.
    seed_hand(
        &mut tx,
        target,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "branch choice pending");
    state = resolve_choice(state, owner, vec![1], false, 1); // give branch
    assert!(state.has_pending_choice(), "give target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 2);
    assert!(
        state.has_pending_choice(),
        "target's give-card choice pending"
    );
    let target_hand: Vec<u32> = state
        .hand_cards(target)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(target_hand.len(), 2);
    state = resolve_choice(state, target, target_hand, false, 3);

    assert_eq!(state.hand_count(owner), 2, "owner gained the two cards");
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp_before - 1,
        "target's virtual Sha hit the owner"
    );
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn gain_branch_gains_gives_back_and_draws() {
    let package = content_compiler::compile_source_named("dcmanzhi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    // Deterministic hands: empty both, then seed exactly.
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    let owner_seeded = seed_hand(
        &mut tx,
        owner,
        &[
            (CardKind::Sha, 96_000),
            (CardKind::Sha, 96_001),
            (CardKind::Tao, 96_002),
        ],
    );
    // Target holds two cards to be taken.
    seed_hand(
        &mut tx,
        target,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![2], false, 1); // gain branch
    assert!(state.has_pending_choice(), "gain target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 2);
    assert!(
        state.has_pending_choice(),
        "owner's take-card choice pending"
    );
    let target_hand: Vec<u32> = state
        .hand_cards(target)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(target_hand.len(), 2);
    state = resolve_choice(state, owner, target_hand, false, 3);
    assert!(
        state.has_pending_choice(),
        "owner's give-back choice pending"
    );
    // Give back exactly two of the owner's own cards (same count as taken).
    let give_back: Vec<u32> = owner_seeded[0..2].iter().map(|c| c.raw()).collect();
    state = resolve_choice(state, owner, give_back, false, 4);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained 2, gave 2, drew 1"
    );
    assert_eq!(state.hand_count(target), 2, "target got its cards back");
}

#[test]
fn branches_are_once_per_turn() {
    let package = content_compiler::compile_source_named("dcmanzhi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    // Four cards: the give branch takes two, the gain branch still has two left.
    seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100),
            (CardKind::Tao, 96_101),
            (CardKind::Sha, 96_102),
            (CardKind::Tao, 96_103),
        ],
    );

    // Use the give branch at Prepare.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![1], false, 1);
    state = resolve_choice(state, owner, vec![target.raw()], false, 2);
    let target_hand: Vec<u32> = state
        .hand_cards(target)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(target_hand.len(), 4);
    state = resolve_choice(state, target, target_hand[0..2].to_vec(), false, 3);
    // Give mark set; the owner was hit by the Sha.
    let owner_state = state.players.get(owner).unwrap();
    assert!(
        owner_state
            .marks
            .iter()
            .any(|(mark, count)| *mark == GIVE_MARK && *count > 0),
        "give branch marked as used"
    );

    // At the same turn's End, only the gain branch is offered: choose 2 -> gain target choice.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let tx = fire_phase(tx, owner, TurnPhaseTag::End);
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "end-phase branch choice pending"
    );
    state = resolve_choice(state, owner, vec![2], false, 4);
    assert!(state.has_pending_choice(), "gain target choice still opens");
}

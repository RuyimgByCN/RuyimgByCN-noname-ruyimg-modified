//! Golden coverage for 连篇（sp2 苏飞，ABI 472）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_LIANPIAN: u32 = 3_795_462_236;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp2/xinfu_lianpian.skill");

fn alloc_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "play must be accepted: {:?}",
        result.result
    );
    next
}

fn drive_choice(
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

fn setup(tx: &mut Transaction, owner: PlayerId, others: &[PlayerId]) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_LIANPIAN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the other players' hands so every card resolves without responses.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &other in others {
        let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
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
}

#[test]
fn shared_target_draws_and_gives() {
    let package = content_compiler::compile_source_named("lianpian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 472);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let nanman = alloc_card(&mut tx, owner, CardKind::TrickNanman, 96_001);
    let hand_before = tx.state().hand_count(owner);
    let a_hand_before = tx.state().hand_count(a);

    let mut tx = Transaction::begin(tx.commit().state);
    // First card: Sha at A -> LAST_CARD_TARGETS = {A}.
    let state = play(tx.commit().state, owner, sha, vec![a], 1);
    // Second card: Nanman (targets A + B) shares target A with the previous Sha.
    let state = play(state, owner, nanman, vec![], 2);
    assert!(
        state.resolution.pending_choice_id != 0,
        "give choice pending"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before - 1,
        "two cards used, one drawn: net -1 before the give"
    );
    let state = drive_choice(state, owner, vec![a.raw()], false, 3);
    assert_eq!(
        state.hand_count(a),
        a_hand_before + 1,
        "A received the card"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before - 2,
        "two cards used, drawn card given away"
    );
}

#[test]
fn no_choice_without_shared_target() {
    let package = content_compiler::compile_source_named("lianpian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let juedou = alloc_card(&mut tx, owner, CardKind::TrickJuedou, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    // Sha at A -> LAST_CARD_TARGETS = {A}.
    let state = play(tx.commit().state, owner, sha, vec![a], 1);
    // Juedou at B shares no target with the previous Sha.
    let state = play(state, owner, juedou, vec![b], 2);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no lianpian choice without a shared target"
    );
}

#[test]
fn cancel_keeps_drawn_card() {
    let package = content_compiler::compile_source_named("lianpian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let nanman = alloc_card(&mut tx, owner, CardKind::TrickNanman, 96_001);
    let hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![a], 1);
    let state = play(state, owner, nanman, vec![], 2);
    assert!(state.resolution.pending_choice_id != 0);
    let state = drive_choice(state, owner, vec![], true, 3);
    assert_eq!(
        state.hand_count(owner),
        hand_before - 1,
        "cancelled give keeps the drawn card (2 used + 1 drawn)"
    );
    assert_eq!(state.hand_count(a), 0, "A received nothing");
}

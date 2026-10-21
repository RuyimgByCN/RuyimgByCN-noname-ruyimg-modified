//! Golden coverage for 拱坚（sp2 朱儁，ABI 473）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GONGJIAN: u32 = 3_309_202_552;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp2/gongjian.skill");

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

fn drive_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
        skill_def: SkillDefId::new(GONGJIAN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the others' hands so cards resolve without responses.
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

/// Seed cards into a player's hand; returns their ids.
fn seed_hand(tx: &mut Transaction, player: PlayerId, specs: &[(CardKind, u32)]) -> Vec<CardId> {
    specs
        .iter()
        .map(|(kind, def)| alloc_card(tx, player, *kind, *def))
        .collect()
}

#[test]
fn shared_target_discards_and_gains_sha() {
    let package = content_compiler::compile_source_named("gongjian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 473);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    // A holds a Sha + a Tao; B holds a Sha.
    let a_cards = seed_hand(
        &mut tx,
        a,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    seed_hand(&mut tx, b, &[(CardKind::Sha, 96_102)]);
    let owner_hand_before = tx.state().hand_count(owner);
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    // First Sha at A -> LAST_SHA_TARGETS = {A}.
    let mut state = play(tx.commit().state, owner, sha1, vec![a], 1);
    // The native per-turn Sha limit is one; clear the flag so the test can use two
    // (identity-standard in a real game grants "杀 unlimited" to everyone).
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    // Second Sha at A shares the target -> gongjian fires.
    let state = play(tx.commit().state, owner, sha2, vec![a], 2);
    assert!(
        state.resolution.pending_choice_id != 0,
        "discard choice pending"
    );
    // Discard only A's Sha.
    let state = drive_choice(state, owner, vec![a_cards[0].raw()], 3);
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained A's Sha (two own Shas used, one gained)"
    );
    assert_eq!(
        state.hand_count(a),
        1,
        "A kept only the Tao (Sha discarded and gained)"
    );
}

#[test]
fn no_choice_without_shared_target() {
    let package = content_compiler::compile_source_named("gongjian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    seed_hand(&mut tx, a, &[(CardKind::Sha, 96_100)]);
    seed_hand(&mut tx, b, &[(CardKind::Sha, 96_101)]);
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    let mut state = play(tx.commit().state, owner, sha1, vec![a], 1);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    // Second Sha at B shares no target with the first.
    let state = play(tx.commit().state, owner, sha2, vec![b], 2);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no gongjian choice without a shared target"
    );
}

#[test]
fn only_sha_cards_are_gained() {
    let package = content_compiler::compile_source_named("gongjian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, a, b) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner, &[a, b]);
    // A holds a Sha + a Tao; B holds a Sha (never targeted, no overlap with A).
    let a_cards = seed_hand(
        &mut tx,
        a,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    seed_hand(&mut tx, b, &[(CardKind::Sha, 96_102)]);
    let owner_hand_before = tx.state().hand_count(owner);
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001);

    let mut tx = Transaction::begin(tx.commit().state);
    let mut state = play(tx.commit().state, owner, sha1, vec![a], 1);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    let state = play(tx.commit().state, owner, sha2, vec![a], 2);
    assert!(state.resolution.pending_choice_id != 0);
    // Discard both of A's cards: only the Sha comes back to the owner.
    let state = drive_choice(state, owner, a_cards.iter().map(|id| id.raw()).collect(), 3);
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained the Sha, the Tao stays in the discard pile"
    );
    assert_eq!(state.hand_count(a), 0, "A lost both cards");
}

//! Golden coverage for 揽娇（sxrm 嗔曹操，ABI 479）.

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

const SXRMLANJIAO: u32 = 3_315_968_200;
const SOURCE: &str = include_str!("../../../content/original/ported/sxrm/sxrmlanjiao.skill");

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
        skill_def: SkillDefId::new(SXRMLANJIAO),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
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

fn seed_hand(
    tx: &mut Transaction,
    player: PlayerId,
    specs: &[(CardKind, u32, Suit)],
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    specs
        .iter()
        .map(|(kind, def, suit)| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: *kind,
                suit: *suit,
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
fn stop_gains_the_viewed_cards() {
    let package = content_compiler::compile_source_named("sxrmlanjiao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 479);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // The target holds exactly two cards (the first two will be viewed).
    let target_cards = seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Heart),
        ],
    );
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice(), "rechoose choice pending");
    // The target stops (0): the owner gains the two viewed cards.
    state = resolve_choice(state, target, vec![0], false, 2);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 2,
        "gained the two viewed cards"
    );
    assert!(
        state.hand_cards(owner).contains(&target_cards[0])
            && state.hand_cards(owner).contains(&target_cards[1]),
        "both viewed cards moved to the owner"
    );
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn rechoose_loops_then_stop() {
    let package = content_compiler::compile_source_named("sxrmlanjiao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // Four cards: two viewed, rechoose (lose 1 + draw 1), two more viewed, then stop.
    let _target_cards = seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Heart),
            (CardKind::Sha, 96_102, Suit::Club),
            (CardKind::Tao, 96_103, Suit::Diamond),
        ],
    );
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice());
    // Rechoose: target loses 1 HP, draws 1, and the loop re-opens.
    state = resolve_choice(state, target, vec![1], false, 2);
    assert!(state.has_pending_choice(), "loop re-opened");
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 1,
        "rechoose cost 1 HP"
    );
    // Stop on the second iteration: the owner gains the new two viewed cards.
    state = resolve_choice(state, target, vec![0], false, 3);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 2,
        "gained the second view's two cards"
    );
    assert_eq!(
        state.hand_count(target),
        3,
        "target kept 4 + 1 drawn - 2 gained"
    );
}

#[test]
fn per_target_once_per_turn() {
    let package = content_compiler::compile_source_named("sxrmlanjiao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Heart),
            (CardKind::Sha, 96_102, Suit::Club),
            (CardKind::Tao, 96_103, Suit::Diamond),
        ],
    );

    let mut tx = Transaction::begin(tx.commit().state);
    // First use: stop, mark the target.
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    state = resolve_choice(state, target, vec![0], false, 2);
    assert_eq!(state.hand_count(owner), 2, "first use gained two cards");

    // A second play-phase firing offers no target (the mark is set).
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_play(tx, owner);
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no target offered after the per-target use"
    );
}

//! Delayed judge tricks: 乐 / 兵粮 / 闪电.

use engine_api::CardDefId;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

/// Strip a player's skills so delayed-trick mechanics tests are independent of
/// which character was randomly dealt (e.g. 陆逊 谦逊 immunity to 乐).
fn strip_skills(tx: &mut Transaction, p: PlayerId) {
    let skills: Vec<_> = tx.state().players.get(p).unwrap().skills.clone();
    for s in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill: s }]).unwrap();
    }
}

fn pass_wuxie(
    mut st: engine_core::state::GameState,
    mut pipe: engine_core::command::PipelineResult,
    mut key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    while st.has_pending_choice()
        && st.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.clone().expect("wuxie");
        key += 1;
        let (s2, p2) = apply_command(
            st,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(p2.result, CommandResult::Accepted { .. }),
            "{:?}",
            p2.result
        );
        st = s2;
        pipe = p2;
    }
    (st, pipe)
}

fn start(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

#[test]
fn deck_has_delayed_tricks() {
    let mut eng = Engine::new();
    eng.submit(start(1, 4));
    let kinds: Vec<_> = eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert!(kinds.iter().any(|k| *k == CardKind::TrickLe));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickBingliang));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickShandian));
}

#[test]
fn play_le_to_judge_zone() {
    let mut eng = Engine::new();
    eng.submit(start(21, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    strip_skills(&mut tx, other);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject_card_to_hand(&mut tx, actor, hand, CardKind::TrickLe, Suit::Club, 6);
    let le = *tx.state().hand_cards(actor).last().unwrap();

    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: le,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let (state, _pipe) = pass_wuxie(state, pipe, 1);
    let jz = state.zones.find(ZoneKind::Judge, other).unwrap();
    assert!(
        state.zones.get(jz).unwrap().cards.iter().any(|c| {
            state
                .cards
                .get(*c)
                .map(|x| x.kind == CardKind::TrickLe)
                .unwrap_or(false)
        }),
        "乐 should sit in target judge zone"
    );
}

#[test]
fn bingliang_skip_draw_goes_to_play() {
    let mut eng = Engine::new();
    eng.submit(start(33, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject_card_to_hand(
        &mut tx,
        actor,
        hand,
        CardKind::TrickBingliang,
        Suit::Spade,
        10,
    );
    let card = *tx.state().hand_cards(actor).last().unwrap();
    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let (state, _pipe) = pass_wuxie(state, pipe, 1);

    // Victim's judge phase with forced non-club judgment
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Judge;
    force_draw_top_suit(&mut tx, Suit::Heart, 2);
    let hand_before = tx.state().hand_count(other);
    let (st2, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(st2.turn.phase, Phase::Play, "phase={:?}", st2.turn.phase);
    assert_eq!(
        st2.hand_count(other),
        hand_before,
        "兵粮 should skip drawing 2"
    );
}

#[test]
fn le_skip_play_after_draw() {
    let mut eng = Engine::new();
    eng.submit(start(44, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    strip_skills(&mut tx, other);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject_card_to_hand(&mut tx, actor, hand, CardKind::TrickLe, Suit::Club, 6);
    let le = *tx.state().hand_cards(actor).last().unwrap();
    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: le,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let (state, _pipe) = pass_wuxie(state, pipe, 1);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Judge;
    force_draw_top_suit(&mut tx, Suit::Spade, 5); // not heart → skip play
    let (st2, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // Should have drawn and be in Draw phase with skip_play pending
    assert_eq!(st2.turn.phase, Phase::Draw, "phase={:?}", st2.turn.phase);
    let (st3, pipe) = apply_command(
        st2,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(
        st3.turn.phase,
        Phase::Discard,
        "乐 should skip play → discard, got {:?}",
        st3.turn.phase
    );
}

#[test]
fn lightning_pass_or_hit() {
    let mut eng = Engine::new();
    eng.submit(start(55, 3));
    let actor = eng.state().turn.current;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject_card_to_hand(
        &mut tx,
        actor,
        hand,
        CardKind::TrickShandian,
        Suit::Spade,
        1,
    );
    let card = *tx.state().hand_cards(actor).last().unwrap();
    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let (state, _pipe) = pass_wuxie(state, pipe, 1);
    let jz = state.zones.find(ZoneKind::Judge, actor).unwrap();
    assert!(
        !state.zones.get(jz).unwrap().cards.is_empty(),
        "lightning on self judge"
    );

    // Resolve with non-hit judgment (Heart) → pass to next
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Judge;
    force_draw_top_suit(&mut tx, Suit::Heart, 5);
    let (st2, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let self_j = st2.zones.find(ZoneKind::Judge, actor).unwrap();
    assert!(
        st2.zones.get(self_j).unwrap().cards.is_empty(),
        "lightning should leave owner after pass"
    );
    // Some other player has it
    let any = st2.players.iter_live().any(|p| {
        st2.zones
            .find(ZoneKind::Judge, p.id)
            .and_then(|z| st2.zones.get(z))
            .map(|z| {
                z.cards.iter().any(|c| {
                    st2.cards
                        .get(*c)
                        .map(|x| x.kind == CardKind::TrickShandian)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    assert!(any, "lightning should be on some judge zone");
}

fn inject_card_to_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    hand: ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(8000 + kind.tag() as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let cid = tx.state_mut().cards.alloc(card);
    if let Some(z) = tx.state_mut().zones.get_mut(hand) {
        z.cards.push(cid);
    }
    if let Some(c) = tx.state_mut().cards.get_mut(cid) {
        c.zone = hand;
        c.owner = owner;
    }
}

fn force_draw_top_suit(tx: &mut Transaction, suit: Suit, rank: u8) {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(7000 + rank as u32),
        kind: CardKind::Sha,
        suit,
        rank,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    };
    let cid = tx.state_mut().cards.alloc(card);
    if let Some(z) = tx.state_mut().zones.get_mut(draw) {
        z.cards.push(cid);
    }
    if let Some(c) = tx.state_mut().cards.get_mut(cid) {
        c.zone = draw;
    }
}

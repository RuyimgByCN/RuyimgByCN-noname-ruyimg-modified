//! 无懈可击：锦囊生效前反制链（奇偶抵消）.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9200 + kind.tag() as u32 * 50 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let cid = tx.state_mut().cards.alloc(card);
    if let Some(z) = tx.state_mut().zones.get_mut(zone) {
        z.cards.push(cid);
    }
    if let Some(c) = tx.state_mut().cards.get_mut(cid) {
        c.zone = zone;
        c.owner = owner;
    }
    cid
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<_> = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn other_of(eng: &Engine, actor: PlayerId) -> PlayerId {
    eng.state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id
}

/// Drain all 无懈 windows by always passing.
fn pass_all_wuxie(
    mut st: engine_core::state::GameState,
    mut pipe: engine_core::command::PipelineResult,
    mut key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
    u64,
) {
    while st.has_pending_choice()
        && st.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.clone().expect("wuxie choice");
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
    (st, pipe, key)
}

#[test]
fn deck_includes_wuxie() {
    let mut eng = Engine::new();
    eng.submit(start(50, 4));
    assert!(eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .any(|c| c.kind == CardKind::TrickWuxie));
}

#[test]
fn wuxie_not_playable_in_play_phase() {
    let mut eng = Engine::new();
    eng.submit(start(51, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let wx = inject(&mut tx, actor, hand, CardKind::TrickWuxie, Suit::Spade, 11);

    let (_st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: wx,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Rejected { .. }),
        "{:?}",
        pipe.result
    );
}

#[test]
fn wuxie_nullifies_wuzhong() {
    let mut eng = Engine::new();
    eng.submit(start(52, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let wuzhong = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    let wx = inject(&mut tx, other, hand_b, CardKind::TrickWuxie, Suit::Club, 12);
    let hand_before = tx.state().hand_count(actor);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: wuzhong,
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
    assert!(st.has_pending_choice());
    assert_eq!(
        st.resolution.continuation_kind,
        ContinuationKind::WuxieWindow
    );
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.observer, other);
    assert!(choice.options.iter().any(|o| o.token == wx.raw()));

    // other plays 无懈
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![wx.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    // Counter-window may open for actor; pass all remaining
    let (st3, _pipe3, _) = pass_all_wuxie(st2, pipe2, 2);
    // 无中 was nullified: actor only lost the used card (hand empty after clear+inject used)
    // after clear, injected wuzhong used, no draw → hand 0
    assert_eq!(st3.hand_count(actor), hand_before.saturating_sub(1));
    // 无懈 discarded
    assert!(!st3.hand_cards(other).contains(&wx));
}

#[test]
fn double_wuxie_restores_wuzhong() {
    let mut eng = Engine::new();
    eng.submit(start(53, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let wuzhong = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickWuzhong,
        Suit::Heart,
        8,
    );
    let wx1 = inject(
        &mut tx,
        other,
        hand_b,
        CardKind::TrickWuxie,
        Suit::Spade,
        11,
    );
    let wx2 = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickWuxie,
        Suit::Diamond,
        12,
    );

    let (mut st, mut pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: wuzhong,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    // Seat order starts at actor (has wx2) → pass; then other plays wx1; then actor counters with wx2.
    let mut key = 1u64;
    let mut played_wx1 = false;
    let mut played_wx2 = false;
    for _ in 0..8 {
        if !st.has_pending_choice()
            || st.resolution.continuation_kind != ContinuationKind::WuxieWindow
        {
            break;
        }
        let choice = pipe.choice.clone().expect("wuxie choice in pipe");
        key += 1;
        let (sel, cancelled, who) = if choice.observer == other
            && !played_wx1
            && choice.options.iter().any(|o| o.token == wx1.raw())
        {
            played_wx1 = true;
            (vec![wx1.raw()], false, other)
        } else if choice.observer == actor
            && played_wx1
            && !played_wx2
            && choice.options.iter().any(|o| o.token == wx2.raw())
        {
            played_wx2 = true;
            (vec![wx2.raw()], false, actor)
        } else {
            (vec![], true, choice.observer)
        };
        let (s2, p2) = apply_command(
            st,
            Command {
                actor: who,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: sel,
                    cancelled,
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
    let (st_final, _, _) = pass_all_wuxie(st, pipe, key);
    assert!(played_wx1 && played_wx2, "both 无懈 should be played");
    // used wuzhong + wx2, draw 2 → hand 2
    assert_eq!(
        st_final.hand_count(actor),
        2,
        "double wuxie should let 无中 resolve (draw 2)"
    );
}

#[test]
fn wuxie_nullifies_guohe_no_discard() {
    let mut eng = Engine::new();
    eng.submit(start(54, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let guohe = inject(&mut tx, actor, hand_a, CardKind::TrickGuohe, Suit::Spade, 3);
    let keep = inject(&mut tx, other, hand_b, CardKind::Sha, Suit::Heart, 1);
    let wx = inject(&mut tx, other, hand_b, CardKind::TrickWuxie, Suit::Club, 13);
    let o_n = tx.state().hand_count(other);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: guohe,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let c1 = pipe.choice.unwrap();
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: c1.choice_id,
                selections: vec![wx.raw()],
                cancelled: false,
            },
        },
    );
    let (st3, pipe3, _) = pass_all_wuxie(st2, pipe2, 2);
    // No PickTargetCard — 过河 nullified
    assert!(
        !st3.has_pending_choice()
            || st3.resolution.continuation_kind != ContinuationKind::PickTargetCard,
        "guohe should not open pick after nullify: cont={:?}",
        st3.resolution.continuation_kind
    );
    assert!(st3.hand_cards(other).contains(&keep));
    // other: had keep+wx, spent wx → 1 card
    assert_eq!(st3.hand_count(other), o_n - 1);
    let _ = pipe3;
}

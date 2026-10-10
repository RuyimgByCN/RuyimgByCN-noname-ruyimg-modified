//! 过河全区域点选 + 火攻亮牌比花色.

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
        def: CardDefId::new(9100 + kind.tag() as u32 * 40 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: kind.is_equip() || kind.is_delayed_trick(),
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

fn other_of(eng: &Engine, actor: PlayerId) -> PlayerId {
    eng.state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id
}

fn clear_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

#[test]
fn deck_includes_huogong() {
    let mut eng = Engine::new();
    eng.submit(start(40, 4));
    let kinds: Vec<_> = eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert!(kinds.iter().any(|k| *k == CardKind::TrickHuogong));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickGuohe));
}

#[test]
fn guohe_discards_equip_card() {
    let mut eng = Engine::new();
    eng.submit(start(41, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;

    // Isolate the card effect from random opening hands (especially 无懈).
    for player in [actor, other] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }

    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let weapon = inject(&mut tx, other, equip, CardKind::Weapon, Suit::Spade, 6);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let guohe = inject(&mut tx, actor, hand, CardKind::TrickGuohe, Suit::Club, 3);

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
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(st.has_pending_choice());
    let choice = pipe.choice.unwrap();
    assert!(
        choice.options.iter().any(|o| o.token == weapon.raw()),
        "equip card {} should be pickable (protected={}): {:?}",
        weapon.raw(),
        engine_core::query::card_discard_protected_from(&st, actor, other, weapon),
        choice.options,
    );

    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![weapon.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    let eq = st2.zones.find(ZoneKind::Equip, other).unwrap();
    let eq_cards = st2
        .zones
        .get(eq)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    assert!(
        !eq_cards.contains(&weapon),
        "weapon should be discarded from equip"
    );
    let discard = st2
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let d_cards = st2
        .zones
        .get(discard)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    assert!(d_cards.contains(&weapon));
}

#[test]
fn guohe_discards_judge_card() {
    let mut eng = Engine::new();
    eng.submit(start(42, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;

    // Clear hand + equip so only judge remains
    let ohand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let ohand_cards: Vec<_> = tx.state().hand_cards(other);
    if !ohand_cards.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: ohand_cards,
                to: discard,
                from: Some(ohand),
            }],
        )
        .unwrap();
    }

    let judge = tx.state().zones.find(ZoneKind::Judge, other).unwrap();
    let le = inject(&mut tx, other, judge, CardKind::TrickLe, Suit::Heart, 6);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let guohe = inject(&mut tx, actor, hand, CardKind::TrickGuohe, Suit::Spade, 4);

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
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // Seed 42 deals the actor a 无懈可击: the engine opens the trick wuxie
    // window first. Pass it (cancel), then the discard choice appears.
    let (st, pipe) = if st.resolution.continuation_kind == ContinuationKind::WuxieWindow {
        let wuxie_choice = pipe.choice.expect("wuxie window");
        let (s2, p2) = apply_command(
            st,
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(9),
                kind: CommandKind::ResolveChoice {
                    choice_id: wuxie_choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(p2.result, CommandResult::Accepted { .. }));
        (s2, p2)
    } else {
        (st, pipe)
    };
    let choice = pipe.choice.unwrap();
    assert!(choice.options.iter().any(|o| o.token == le.raw()));

    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![le.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    let jz = st2.zones.find(ZoneKind::Judge, other).unwrap();
    let j_cards = st2
        .zones
        .get(jz)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    assert!(!j_cards.contains(&le));
}

#[test]
fn shunshou_cannot_take_judge() {
    let mut eng = Engine::new();
    eng.submit(start(43, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;

    // Only judge card on target
    let ohand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let ohand_cards: Vec<_> = tx.state().hand_cards(other);
    if !ohand_cards.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: ohand_cards,
                to: discard,
                from: Some(ohand),
            }],
        )
        .unwrap();
    }
    let judge = tx.state().zones.find(ZoneKind::Judge, other).unwrap();
    inject(&mut tx, other, judge, CardKind::TrickLe, Suit::Club, 6);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let shun = inject(
        &mut tx,
        actor,
        hand,
        CardKind::TrickShunshou,
        Suit::Diamond,
        3,
    );

    let (_st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: shun,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    // No hand/equip → shunshou illegal
    assert!(
        matches!(pipe.result, CommandResult::Rejected { .. }),
        "expected reject, got {:?}",
        pipe.result
    );
}

#[test]
fn huogong_same_suit_deals_damage() {
    let mut eng = Engine::new();
    eng.submit(start(44, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;

    // Clear hands and inject controlled cards
    for p in [actor, other] {
        let hand = tx.state().zones.find(ZoneKind::Hand, p).unwrap();
        let cards: Vec<_> = tx.state().hand_cards(p);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }

    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let huogong = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickHuogong,
        Suit::Heart,
        2,
    );
    let match_card = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Spade, 7);
    let shown = inject(&mut tx, other, hand_b, CardKind::Shan, Suit::Spade, 2);
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: huogong,
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
    assert!(st.has_pending_choice());
    let c1 = pipe.choice.unwrap();
    assert_eq!(c1.observer, other);

    // Target shows Spade 闪
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: c1.choice_id,
                selections: vec![shown.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert!(st2.has_pending_choice(), "actor should discard same suit");
    let c2 = pipe2.choice.unwrap();
    assert_eq!(c2.observer, actor);
    assert!(c2.options.iter().any(|o| o.token == match_card.raw()));

    let (st3, pipe3) = apply_command(
        st2,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: c2.choice_id,
                selections: vec![match_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe3.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe3.result
    );
    assert_eq!(st3.players.get(other).unwrap().hp, hp - 1);
    // match card discarded
    assert!(!st3.hand_cards(actor).contains(&match_card));
}

#[test]
fn huogong_cancel_no_damage() {
    let mut eng = Engine::new();
    eng.submit(start(45, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;

    for p in [actor, other] {
        let hand = tx.state().zones.find(ZoneKind::Hand, p).unwrap();
        let cards: Vec<_> = tx.state().hand_cards(p);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }

    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let huogong = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickHuogong,
        Suit::Diamond,
        12,
    );
    inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Heart, 1); // wrong suit
    let shown = inject(&mut tx, other, hand_b, CardKind::Tao, Suit::Club, 3);
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: huogong,
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
                selections: vec![shown.raw()],
                cancelled: false,
            },
        },
    );
    // Actor has no Club → fire fails without second choice
    if st2.has_pending_choice() {
        let c2 = pipe2.choice.unwrap();
        let (st3, pipe3) = apply_command(
            st2,
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(3),
                kind: CommandKind::ResolveChoice {
                    choice_id: c2.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe3.result, CommandResult::Accepted { .. }));
        assert_eq!(st3.players.get(other).unwrap().hp, hp);
    } else {
        assert_eq!(st2.players.get(other).unwrap().hp, hp);
    }
}

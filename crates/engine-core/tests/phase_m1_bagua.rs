//! 八卦阵：需闪时判定红心当闪.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content, Engine};
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
    def: u32,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
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

#[test]
fn bagua_token_offered_when_equipped() {
    let mut eng = Engine::new();
    eng.submit(start(70, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Spade,
        2,
        content::DEF_BAGUA,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1, 10);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
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
    assert_eq!(
        st.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.observer, other);
    assert!(
        choice
            .options
            .iter()
            .any(|o| o.token == content::BAGUA_RESPONSE_TOKEN),
        "bagua token should be offered: {:?}",
        choice.options
    );
}

#[test]
fn bagua_heart_negates_sha() {
    let mut eng = Engine::new();
    eng.submit(start(71, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Spade,
        2,
        content::DEF_BAGUA,
    );
    // Force Heart on top of draw (last = top)
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        Suit::Heart,
        5,
        9991,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Club, 1, 10);
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let choice = pipe.choice.unwrap();
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![content::BAGUA_RESPONSE_TOKEN],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert!(
        !st2.has_pending_choice()
            || st2.resolution.continuation_kind != ContinuationKind::RespondToSha
    );
    assert_eq!(
        st2.players.get(other).unwrap().hp,
        hp,
        "heart bagua blocks 杀"
    );
}

#[test]
fn bagua_non_heart_can_still_play_shan() {
    let mut eng = Engine::new();
    eng.submit(start(72, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Spade,
        2,
        content::DEF_BAGUA,
    );
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let shan = inject(&mut tx, other, hand_b, CardKind::Shan, Suit::Club, 2, 50);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        Suit::Spade,
        3,
        9992,
    );
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Diamond, 1, 10);
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let c1 = pipe.choice.unwrap();
    // bagua fail (spade)
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: c1.choice_id,
                selections: vec![content::BAGUA_RESPONSE_TOKEN],
                cancelled: false,
            },
        },
    );
    assert!(st2.has_pending_choice(), "should re-ask after bagua fail");
    let c2 = pipe2.choice.unwrap();
    assert!(c2.options.iter().any(|o| o.token == shan.raw()));
    let (st3, pipe3) = apply_command(
        st2,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: c2.choice_id,
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe3.result, CommandResult::Accepted { .. }));
    assert_eq!(st3.players.get(other).unwrap().hp, hp);
}

#[test]
fn bagua_fail_no_shan_takes_damage() {
    let mut eng = Engine::new();
    eng.submit(start(73, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Spade,
        2,
        content::DEF_BAGUA,
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    // two non-hearts: first bagua fail, second bagua fail then damage on cancel
    inject(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        Suit::Club,
        4,
        9993,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1, 10);
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
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
                selections: vec![content::BAGUA_RESPONSE_TOKEN],
                cancelled: false,
            },
        },
    );
    // fail → re-ask; cancel to take damage
    assert!(st2.has_pending_choice());
    let c2 = pipe2.choice.unwrap();
    let (st3, pipe3) = apply_command(
        st2,
        Command {
            actor: other,
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
    assert_eq!(st3.players.get(other).unwrap().hp, hp - 1);
}

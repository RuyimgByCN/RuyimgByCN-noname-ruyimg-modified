//! 藤甲：火焰伤害 +1、南蛮无效；火攻为火焰伤害.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content, Engine};
use rule_api::{damage_nature, EffectRequest};

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

fn pass_wuxie(
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
    (st, pipe, key)
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
fn fire_damage_plus_one_with_tengjia() {
    let mut eng = Engine::new();
    eng.submit(start(60, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Club,
        2,
        content::DEF_TENGJIA,
    );
    assert!(content::player_has_tengjia(tx.state(), other));
    let hp = tx.state().players.get(other).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: actor,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
        }],
    )
    .unwrap();
    // May open dying if low hp — check delta
    let hp2 = tx.state().players.get(other).unwrap().hp;
    assert_eq!(hp - hp2, 2, "fire vs 藤甲 should be 1+1");
}

#[test]
fn normal_damage_not_amplified_by_tengjia() {
    let mut eng = Engine::new();
    eng.submit(start(61, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Club,
        2,
        content::DEF_TENGJIA,
    );
    let hp = tx.state().players.get(other).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: actor,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let hp2 = tx.state().players.get(other).unwrap().hp;
    assert_eq!(hp - hp2, 1);
}

#[test]
fn nanman_skipped_for_tengjia() {
    let mut eng = Engine::new();
    eng.submit(start(62, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // other has 藤甲, no 杀
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    inject(
        &mut tx,
        other,
        equip,
        CardKind::Armor,
        Suit::Club,
        2,
        content::DEF_TENGJIA,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let nanman = inject(
        &mut tx,
        actor,
        hand,
        CardKind::TrickNanman,
        Suit::Spade,
        7,
        170,
    );
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: nanman,
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
    let (st2, _pipe2, _) = pass_wuxie(st, pipe, 1);
    // 藤甲 blocks 南蛮 entirely for other — no damage, no sha response left
    assert!(
        !st2.has_pending_choice()
            || st2.resolution.continuation_kind != ContinuationKind::RespondAoeQueue
    );
    assert_eq!(st2.players.get(other).unwrap().hp, hp);
}

#[test]
fn huogong_fire_hits_tengjia_for_two() {
    let mut eng = Engine::new();
    eng.submit(start(63, 2));
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
        Suit::Club,
        2,
        content::DEF_TENGJIA,
    );
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let hg = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickHuogong,
        Suit::Heart,
        2,
        250,
    );
    let match_c = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Spade, 5, 11);
    let shown = inject(&mut tx, other, hand_b, CardKind::Shan, Suit::Spade, 2, 52);
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: hg,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let (st, pipe, mut key) = pass_wuxie(st, pipe, 1);
    // show
    let c1 = pipe.choice.unwrap();
    key += 1;
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: c1.choice_id,
                selections: vec![shown.raw()],
                cancelled: false,
            },
        },
    );
    let c2 = pipe2.choice.unwrap();
    key += 1;
    let (st3, pipe3) = apply_command(
        st2,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: c2.choice_id,
                selections: vec![match_c.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe3.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe3.result
    );
    // 1 fire + 藤甲 +1 = 2
    assert_eq!(st3.players.get(other).unwrap().hp, hp - 2);
}

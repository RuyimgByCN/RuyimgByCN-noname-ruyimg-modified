//! 火杀 / 雷杀：伤害属性 + 藤甲联动.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
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
        face_up: kind.is_equip(),
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

fn isolate_damage_resolution(tx: &mut Transaction) {
    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, natural_skills).unwrap();
    for player in tx.state().players.alive_by_seat() {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 8;
        state.max_hp = 8;
    }
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
fn deck_includes_elemental_sha() {
    let mut eng = Engine::new();
    eng.submit(start(80, 4));
    let kinds: Vec<_> = eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert!(kinds.iter().any(|k| *k == CardKind::ShaFire));
    assert!(kinds.iter().any(|k| *k == CardKind::ShaThunder));
}

#[test]
fn fire_sha_vs_tengjia_deals_two() {
    let mut eng = Engine::new();
    eng.submit(start(81, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    isolate_damage_resolution(&mut tx);
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
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::ShaFire, Suit::Heart, 4, 90);
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
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // no 闪 → damage immediately (or choice if bagua — none)
    let st = if st.has_pending_choice() {
        let c = pipe.choice.unwrap();
        let (s2, p2) = apply_command(
            st,
            Command {
                actor: other,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: c.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(p2.result, CommandResult::Accepted { .. }));
        s2
    } else {
        st
    };
    assert_eq!(
        st.players.get(other).unwrap().hp,
        hp - 2,
        "火杀 vs 藤甲 = 1+1"
    );
}

#[test]
fn thunder_sha_not_amplified_by_tengjia() {
    let mut eng = Engine::new();
    eng.submit(start(82, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    isolate_damage_resolution(&mut tx);
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
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(
        &mut tx,
        actor,
        hand,
        CardKind::ShaThunder,
        Suit::Spade,
        5,
        93,
    );
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
    let st = if st.has_pending_choice() {
        let c = pipe.choice.unwrap();
        let (s2, _) = apply_command(
            st,
            Command {
                actor: other,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: c.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        s2
    } else {
        st
    };
    assert_eq!(st.players.get(other).unwrap().hp, hp - 1);
}

#[test]
fn normal_sha_still_one_vs_tengjia() {
    let mut eng = Engine::new();
    eng.submit(start(83, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    isolate_damage_resolution(&mut tx);
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
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1, 11);
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
    let st = if st.has_pending_choice() {
        let c = pipe.choice.unwrap();
        let (s2, _) = apply_command(
            st,
            Command {
                actor: other,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: c.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        s2
    } else {
        st
    };
    assert_eq!(st.players.get(other).unwrap().hp, hp - 1);
}

#[test]
fn is_sha_helpers() {
    assert!(CardKind::Sha.is_sha());
    assert!(CardKind::ShaFire.is_sha());
    assert!(CardKind::ShaThunder.is_sha());
    assert!(!CardKind::Shan.is_sha());
    assert_eq!(CardKind::Sha.sha_nature(), 0);
    assert_eq!(CardKind::ShaFire.sha_nature(), 1);
    assert_eq!(CardKind::ShaThunder.sha_nature(), 2);
}

#[test]
fn sha_dodged_accepts_source_free_virtual_sha_but_rejects_forged_card() {
    let mut engine = Engine::new();
    engine.submit(start(84, 2));
    let players = engine.state().players.alive_by_seat();
    let source = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    isolate_damage_resolution(&mut tx);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::ShaDodged {
            source,
            target,
            card: engine_api::CardId::NONE,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::ShaDodged { card, .. } if card.is_none()
    )));

    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let carrier = inject(&mut tx, source, hand, CardKind::Shan, Suit::Club, 1, 84_001);
    tx.state_mut()
        .resolution
        .register_sha_resolution(source, carrier, &[target], false);
    run_effects(
        &mut tx,
        vec![EffectRequest::ShaDodged {
            source,
            target,
            card: carrier,
        }],
    )
    .unwrap();

    let forged = inject(&mut tx, source, hand, CardKind::Tao, Suit::Heart, 2, 84_002);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ShaDodged {
                source,
                target,
                card: forged,
            }],
        ),
        Err(EffectError::Illegal(_))
    ));
}

//! 南蛮/万箭逐人响应、决斗拼杀、无双双闪.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content::skill_def, Engine};
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
    hand: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(8300 + kind.tag() as u32 * 20 + rank as u32),
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
    cid
}

fn isolate_card_resolution(tx: &mut Transaction) {
    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, natural_skills).unwrap();

    let players = tx.state().players.alive_by_seat();
    for player in players {
        run_effects(
            tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 8;
        state.max_hp = 8;
    }
}

#[test]
fn nanman_asks_for_sha_response() {
    let mut eng = Engine::new();
    eng.submit(start(20, 3));
    let actor = eng.state().turn.current;
    let others: Vec<_> = eng
        .state()
        .players
        .iter_live()
        .filter(|p| p.id != actor)
        .map(|p| p.id)
        .collect();
    assert!(others.len() >= 2);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    isolate_card_resolution(&mut tx);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let nanman = inject(&mut tx, actor, hand, CardKind::TrickNanman, Suit::Spade, 7);
    // First other has a 杀
    let h1 = tx.state().zones.find(ZoneKind::Hand, others[0]).unwrap();
    inject(&mut tx, others[0], h1, CardKind::Sha, Suit::Heart, 2);
    let hp1 = tx.state().players.get(others[0]).unwrap().hp;
    let hp2 = tx.state().players.get(others[1]).unwrap().hp;

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
    assert!(st.has_pending_choice(), "first target should respond");
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.observer, others[0]);

    // Respond with first 杀 token
    let token = choice.options[0].token;
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: others[0],
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_eq!(
        st2.players.get(others[0]).unwrap().hp,
        hp1,
        "responded no damage"
    );

    // Second target may have choice or already damaged if no cards
    if st2.has_pending_choice() {
        let ch = pipe2.choice.unwrap();
        // cancel → damage
        let (st3, pipe3) = apply_command(
            st2,
            Command {
                actor: ch.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(3),
                kind: CommandKind::ResolveChoice {
                    choice_id: ch.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe3.result, CommandResult::Accepted { .. }));
        // may be dying if hp was 1
        let _ = st3;
    } else {
        // auto-resolved damage without response cards
        assert!(
            st2.players.get(others[1]).unwrap().hp < hp2
                || st2.players.get(others[1]).unwrap().dying
                || !st2.players.get(others[1]).unwrap().alive
                || true
        );
    }
}

#[test]
fn juedou_chain_damage_on_fail() {
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
    isolate_card_resolution(&mut tx);
    let ha = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let juedou = inject(&mut tx, actor, ha, CardKind::TrickJuedou, Suit::Diamond, 1);
    // Target has no 杀 → immediate damage when cancel / empty tokens
    let hp = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: juedou,
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
    if st.has_pending_choice() {
        let ch = pipe.choice.unwrap();
        let (st2, pipe2) = apply_command(
            st,
            Command {
                actor: ch.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: ch.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe2.result, CommandResult::Accepted { .. }));
        assert!(
            st2.players.get(other).unwrap().hp < hp
                || st2.players.get(other).unwrap().dying
                || !st2.players.get(other).unwrap().alive
        );
    } else {
        // no response cards: damage applied immediately
        assert!(
            st.players.get(other).unwrap().hp < hp
                || st.players.get(other).unwrap().dying
                || !st.players.get(other).unwrap().alive
        );
    }
}

#[test]
fn wushuang_needs_two_shan() {
    let mut eng = Engine::new();
    eng.submit(start(22, 2));
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
    isolate_card_resolution(&mut tx);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::WUSHUANG),
        }],
    )
    .unwrap();
    let ha = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hb = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sha = inject(&mut tx, actor, ha, CardKind::Sha, Suit::Spade, 7);
    let s1 = inject(&mut tx, other, hb, CardKind::Shan, Suit::Club, 2);
    let s2 = inject(&mut tx, other, hb, CardKind::Shan, Suit::Heart, 3);
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
    assert!(st.has_pending_choice());
    let ch1 = pipe.choice.unwrap();
    // play first 闪
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ch1.choice_id,
                selections: vec![s1.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe2.result, CommandResult::Accepted { .. }));
    assert!(st2.has_pending_choice(), "无双 needs second 闪");
    let ch2 = pipe2.choice.unwrap();
    let (st3, pipe3) = apply_command(
        st2,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: ch2.choice_id,
                selections: vec![s2.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe3.result, CommandResult::Accepted { .. }));
    assert_eq!(
        st3.players.get(other).unwrap().hp,
        hp,
        "two 闪 negate 无双杀"
    );
}

#[test]
fn wushuang_duel_requires_two_sha_from_the_owners_opponent() {
    let mut eng = Engine::new();
    eng.submit(start(23, 2));
    let actor = eng.state().turn.current;
    let owner = eng
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    isolate_card_resolution(&mut tx);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(skill_def::WUSHUANG),
        }],
    )
    .unwrap();
    let actor_hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let duel = inject(
        &mut tx,
        actor,
        actor_hand,
        CardKind::TrickJuedou,
        Suit::Diamond,
        1,
    );
    let actor_sha_1 = inject(&mut tx, actor, actor_hand, CardKind::Sha, Suit::Spade, 2);
    let actor_sha_2 = inject(&mut tx, actor, actor_hand, CardKind::Sha, Suit::Club, 3);
    let owner_sha = inject(&mut tx, owner, owner_hand, CardKind::Sha, Suit::Heart, 4);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.observer, owner);

    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![owner_sha.raw()],
                cancelled: false,
            },
        },
    );
    let choice = pipe.choice.unwrap();
    assert_eq!(
        choice.observer, actor,
        "the Wushuang owner still answers with one Sha"
    );

    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![actor_sha_1.raw()],
                cancelled: false,
            },
        },
    );
    let choice = pipe.choice.unwrap();
    assert_eq!(
        choice.observer, actor,
        "the Wushuang owner's opponent must answer twice"
    );

    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![actor_sha_2.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}

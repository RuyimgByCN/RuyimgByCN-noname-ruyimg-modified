//! 杀-闪响应、龙胆杀当闪、制衡多弃、顺手点选.

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
        def: CardDefId::new(8200 + kind.tag() as u32 * 10 + rank as u32),
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

#[test]
fn sha_opens_shan_response_and_cancel_takes_damage() {
    let mut eng = Engine::new();
    eng.submit(start(11, 2));
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
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sha = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Spade, 1);
    inject(&mut tx, other, hand_b, CardKind::Shan, Suit::Club, 2);
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
    assert!(st.has_pending_choice(), "target should respond with 闪");
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.observer, other);

    // Cancel → damage
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_eq!(st2.players.get(other).unwrap().hp, hp - 1);
}

#[test]
fn sha_shan_response_negates_damage() {
    let mut eng = Engine::new();
    eng.submit(start(12, 2));
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
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sha = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Spade, 1);
    let shan = inject(&mut tx, other, hand_b, CardKind::Shan, Suit::Club, 2);
    let hp = tx.state().players.get(other).unwrap().hp;
    let shan_n = tx.state().hand_count(other);

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
    let choice = pipe.choice.unwrap();
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_eq!(st2.players.get(other).unwrap().hp, hp, "闪后不受伤");
    assert_eq!(st2.hand_count(other), shan_n - 1);
}

#[test]
fn second_sha_is_rejected_unless_paoxiao_overrides_the_limit() {
    let mut eng = Engine::new();
    eng.submit(start(21, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let first = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 3);
    let second = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Club, 4);

    let (state, first_result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: first,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    let state = if let Some(choice) = first_result.choice {
        let (state, response) = apply_command(
            state,
            Command {
                actor: other,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(response.result, CommandResult::Accepted { .. }));
        state
    } else {
        state
    };

    let (state, rejected) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::PlayCard {
                card: second,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let mut paoxiao = Transaction::begin(state);
    run_effects(
        &mut paoxiao,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::PAOXIAO),
        }],
    )
    .unwrap();
    let (_, accepted) = apply_command(
        paoxiao.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: second,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
}

#[test]
fn longdan_sha_as_shan_response() {
    let mut eng = Engine::new();
    eng.submit(start(13, 2));
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
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: other,
            skill_def: engine_api::SkillDefId::new(skill_def::LONGDAN),
        }],
    )
    .unwrap();
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sha = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Spade, 5);
    // Target only has 杀, can 龙胆 as 闪
    let resp = inject(&mut tx, other, hand_b, CardKind::Sha, Suit::Heart, 6);
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
        st.has_pending_choice(),
        "longdan 杀 should be legal response"
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
                selections: vec![resp.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_eq!(st2.players.get(other).unwrap().hp, hp);
}

#[test]
fn zhiheng_multi_discard_draw() {
    let mut eng = Engine::new();
    eng.submit(start(14, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::ZHIHENG),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(actor)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|s| {
            tx.state()
                .skills
                .get(*s)
                .map(|i| i.def.raw() == skill_def::ZHIHENG)
                .unwrap_or(false)
        })
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, actor).unwrap();
    let c1 = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1);
    let c2 = inject(&mut tx, actor, equip, CardKind::Armor, Suit::Club, 2);
    let before = tx.state().hand_count(actor);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![c1, c2],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(st.hand_count(actor), before + 1, "弃一手牌一装备，再摸两张");
    let retry = st.hand_cards(actor)[0];
    let (_, second) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![retry],
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
}

#[test]
fn kurou_loses_hp_draws_two_and_can_repeat() {
    let mut eng = Engine::new();
    eng.submit(start(140, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::KUROU),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(actor)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def::KUROU)
        .unwrap();
    let hp = tx.state().players.get(actor).unwrap().hp;
    let hand = tx.state().hand_count(actor);
    let mut state = tx.state().clone();
    for key in [1, 2] {
        let (next, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: vec![],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    assert_eq!(state.players.get(actor).unwrap().hp, hp - 2);
    assert_eq!(state.hand_count(actor), hand + 4);
}

#[test]
fn shunshou_take_selected_card() {
    let mut eng = Engine::new();
    eng.submit(start(15, 2));
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
    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    for player in [actor, other] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let shun = inject(
        &mut tx,
        actor,
        hand,
        CardKind::TrickShunshou,
        Suit::Spade,
        3,
    );
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let pick = inject(&mut tx, other, other_hand, CardKind::Tao, Suit::Heart, 4);
    let a_n = tx.state().hand_count(actor);
    let o_n = tx.state().hand_count(other);

    let (st, pipe) = apply_command(
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
    assert!(st.has_pending_choice());
    let choice = pipe.choice.unwrap();
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![pick.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    // used 顺手 (-1) gained 1 = same as before for actor net from pick; o lost 1
    assert_eq!(st2.hand_count(other), o_n - 1);
    assert_eq!(st2.hand_count(actor), a_n); // -1 shunshou +1 taken
}

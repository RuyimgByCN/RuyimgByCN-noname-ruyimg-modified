//! View-as skills (武圣 / 龙胆) and 过河点选.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content::skill_def, Engine};

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

fn clear_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| rule_api::EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    engine_core::effect::run_effects(tx, effects).unwrap();
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    engine_core::effect::run_effects(
        tx,
        vec![rule_api::EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn package_has_view_as() {
    let mut eng = Engine::new();
    eng.submit(start(1, 4));
    let vs = eng
        .content()
        .view_as_defs_for_skill(engine_api::SkillDefId::new(skill_def::WUSHENG));
    assert!(!vs.is_empty());
    assert_eq!(vs[0].as_kind, "sha");
    assert_eq!(vs[0].filter, "red");
}

#[test]
fn wusheng_red_card_as_sha() {
    let mut eng = Engine::new();
    eng.submit(start(7, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    // Grant 武圣
    engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::WUSHENG),
        }],
    )
    .unwrap();
    let skill_id = tx
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
                .map(|i| i.def.raw() == skill_def::WUSHENG)
                .unwrap_or(false)
        })
        .unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    // Red non-sha card (Heart Tao)
    inject(&mut tx, actor, hand, CardKind::Tao, Suit::Heart, 3);
    let card = *tx.state().hand_cards(actor).last().unwrap();
    let hp_before = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(st.players.get(other).unwrap().hp, hp_before - 1);
    // Card consumed
    assert!(st
        .cards
        .get(card)
        .map(|c| {
            st.zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .map(|d| c.zone == d)
                .unwrap_or(false)
        })
        .unwrap_or(false));
}

#[test]
fn longdan_shan_as_sha() {
    let mut eng = Engine::new();
    eng.submit(start(8, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::LONGDAN),
        }],
    )
    .unwrap();
    let skill_id = tx
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
                .map(|i| i.def.raw() == skill_def::LONGDAN)
                .unwrap_or(false)
        })
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject(&mut tx, actor, hand, CardKind::Shan, Suit::Club, 2);
    let card = *tx.state().hand_cards(actor).last().unwrap();
    let hp_before = tx.state().players.get(other).unwrap().hp;
    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(st.players.get(other).unwrap().hp, hp_before - 1);
}

#[test]
fn guohe_opens_choice_then_discards_selected() {
    let mut eng = Engine::new();
    eng.submit(start(9, 2));
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
    // Ensure other has cards
    if tx.state().hand_count(other) == 0 {
        engine_core::effect::run_effects(
            &mut tx,
            vec![rule_api::EffectRequest::DrawCards {
                player: other,
                count: 2,
            }],
        )
        .unwrap();
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject(&mut tx, actor, hand, CardKind::TrickGuohe, Suit::Spade, 3);
    let guohe = *tx.state().hand_cards(actor).last().unwrap();
    let other_hand = tx.state().hand_cards(other);
    let pick = other_hand[0];
    let other_n = other_hand.len() as u32;

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
    let (st, pipe, key) = pass_wuxie(st, pipe, 1);
    assert!(st.has_pending_choice(), "should open pick-card choice");
    let choice = pipe.choice.expect("choice");
    assert_eq!(choice.min_count, 1);

    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key + 1),
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
    assert_eq!(st2.hand_count(other), other_n - 1);
}

#[test]
fn qixi_black_card_as_guohe() {
    // 甘宁 奇袭: a black card viewed as 过河拆桥 discards a target's card.
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
    engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::QIXI),
        }],
    )
    .unwrap();
    let skill_id = *tx
        .state()
        .players
        .get(actor)
        .unwrap()
        .skills
        .iter()
        .find(|s| {
            tx.state()
                .skills
                .get(**s)
                .map(|i| i.def.raw() == skill_def::QIXI)
                .unwrap_or(false)
        })
        .unwrap();

    // Actor holds a black card (Spade); target holds a card to be discarded.
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 5);
    let card = *tx.state().hand_cards(actor).last().unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    if tx.state().hand_count(other) == 0 {
        inject(&mut tx, other, other_hand, CardKind::Shan, Suit::Club, 2);
    }
    let other_n = tx.state().hand_count(other);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // Guohe opens a wuxie window; decline it, then the target's card is discarded.
    let (st, pipe, mut key) = pass_wuxie(st, pipe, 100);
    let _ = &pipe;
    // If a discard-target pick choice is pending, resolve it by taking the first option.
    let st = if st.has_pending_choice() {
        let choice = pipe.choice.clone().expect("pick choice");
        key += 1;
        let sel = choice.options.first().map(|o| o.token).unwrap_or(0);
        let (s2, p2) = apply_command(
            st,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![sel],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(p2.result, CommandResult::Accepted { .. }),
            "{:?}",
            p2.result
        );
        s2
    } else {
        st
    };
    // The target lost a card to 过河拆桥 (guohe discards one of the target's cards).
    assert!(
        st.hand_count(other) < other_n,
        "target should have lost a hand card to 过河拆桥 (before={other_n}, after={})",
        st.hand_count(other)
    );
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    hand: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(8100 + kind.tag() as u32),
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

//! 鬼才改判、铁索连环、借刀.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content::skill_def, Engine};
use rule_api::EffectRequest;

/// Pass every open 无懈 window (headless-style).
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
        def: CardDefId::new(9000 + kind.tag() as u32 * 30 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_delayed_trick(),
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

#[test]
fn tiesuo_toggle_and_chain_damage() {
    let mut eng = Engine::new();
    eng.submit(start(30, 3));
    let actor = eng.state().turn.current;
    let others: Vec<_> = eng
        .state()
        .players
        .iter_live()
        .filter(|p| p.id != actor)
        .map(|p| p.id)
        .collect();
    let a = others[0];
    let b = others[1];

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let tiesuo = inject(&mut tx, actor, hand, CardKind::TrickTiesuo, Suit::Club, 10);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: tiesuo,
                targets: vec![a, b],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let (st, _pipe) = pass_wuxie(st, pipe, 1);
    assert!(st
        .players
        .get(a)
        .unwrap()
        .marks
        .iter()
        .any(|(m, c)| *m == engine_core::content::MARK_CHAIN && *c > 0));
    assert!(st
        .players
        .get(b)
        .unwrap()
        .marks
        .iter()
        .any(|(m, c)| *m == engine_core::content::MARK_CHAIN && *c > 0));

    let hp_b = st.players.get(b).unwrap().hp;
    let mut tx = Transaction::begin(st);
    // Elemental damage to A should spread to B and clear chains.
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: actor,
            target: a,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(b).unwrap().hp, hp_b - 1);
    assert!(!tx
        .state()
        .players
        .get(a)
        .unwrap()
        .marks
        .iter()
        .any(|(m, c)| *m == engine_core::content::MARK_CHAIN && *c > 0));
}

#[test]
fn guicai_can_rewrite_le_judgment() {
    let mut eng = Engine::new();
    eng.submit(start(31, 2));
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
    // Put 乐 on other, grant 鬼才 to actor with a Heart hand card to rewrite to 无效
    let judge = tx.state().zones.find(ZoneKind::Judge, other).unwrap();
    inject(&mut tx, other, judge, CardKind::TrickLe, Suit::Spade, 6);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::GUICAI),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let heart = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Heart, 5);
    // Force judgment flip to Spade (乐 would hit) then guicai rewrites to Heart
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(&mut tx, PlayerId::NONE, draw, CardKind::Sha, Suit::Spade, 3);

    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Judge;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(st.has_pending_choice(), "鬼才 should open rewrite choice");
    let ch = pipe.choice.unwrap();
    assert_eq!(ch.observer, actor);

    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ch.choice_id,
                selections: vec![heart.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    // Heart rewrite → 乐无效 → no SKIP_PLAY
    assert_eq!(st2.turn.flag(turn_flag::SKIP_PLAY), 0);
}

#[test]
fn tiesuo_recast_draws() {
    let mut eng = Engine::new();
    eng.submit(start(32, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let tiesuo = inject(&mut tx, actor, hand, CardKind::TrickTiesuo, Suit::Spade, 11);
    let before = tx.state().hand_count(actor);
    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: tiesuo,
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
    // -1 tiesuo +1 draw
    assert_eq!(st.hand_count(actor), before);
}

#[test]
fn deck_has_jiedao_tiesuo() {
    let mut eng = Engine::new();
    eng.submit(start(1, 4));
    let kinds: Vec<_> = eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert!(kinds.iter().any(|k| *k == CardKind::TrickJiedao));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickTiesuo));
}

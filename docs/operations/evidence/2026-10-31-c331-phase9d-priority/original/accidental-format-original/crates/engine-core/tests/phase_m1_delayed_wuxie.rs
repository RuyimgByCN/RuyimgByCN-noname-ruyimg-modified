//! 延时锦囊置入前无懈：被无懈则不进判定区.

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
        def: CardDefId::new(9300 + kind.tag() as u32 * 20 + rank as u32),
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

fn strip_booted_skills(tx: &mut Transaction) {
    // Roster closure: booted standard characters may carry skills that
    // interfere with card-flow tests (e.g. 旧界李典 询询/忘隙). These tests
    // only exercise delayed-trick card flow, so drop every booted skill.
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, skills).unwrap();
}

#[test]
fn le_nullified_stays_out_of_judge() {
    let mut eng = Engine::new();
    eng.submit(start(90, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    for player in [actor, other] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let le = inject(&mut tx, actor, hand_a, CardKind::TrickLe, Suit::Spade, 6);
    let wx = inject(&mut tx, other, hand_b, CardKind::TrickWuxie, Suit::Club, 12);

    let (st, pipe) = apply_command(
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
    assert_eq!(
        st.resolution.continuation_kind,
        ContinuationKind::WuxieWindow
    );
    let c1 = pipe.choice.unwrap();
    assert_eq!(c1.observer, other);

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
    let (st3, _, _) = pass_wuxie(st2, pipe2, 2);

    let jz = st3.zones.find(ZoneKind::Judge, other).unwrap();
    let jcards = st3
        .zones
        .get(jz)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    assert!(
        !jcards.contains(&le),
        "nullified 乐 must not enter judge zone"
    );
    let discard = st3
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let dcards = st3
        .zones
        .get(discard)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    assert!(dcards.contains(&le), "乐 stays in discard when nullified");
    assert!(dcards.contains(&wx), "无懈 discarded");
}

#[test]
fn le_resolves_into_judge_when_no_wuxie() {
    let mut eng = Engine::new();
    eng.submit(start(91, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    strip_booted_skills(&mut tx);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let le = inject(&mut tx, actor, hand, CardKind::TrickLe, Suit::Heart, 6);

    let (st, pipe) = apply_command(
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
    let (st2, _, _) = pass_wuxie(st, pipe, 1);
    let jz = st2.zones.find(ZoneKind::Judge, other).unwrap();
    assert!(st2
        .zones
        .get(jz)
        .map(|z| z.cards.contains(&le))
        .unwrap_or(false));
    assert!(st2.cards.get(le).map(|c| c.face_up).unwrap_or(false));
}

#[test]
fn shandian_nullified_not_on_self() {
    let mut eng = Engine::new();
    eng.submit(start(92, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    strip_booted_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sd = inject(
        &mut tx,
        actor,
        hand_a,
        CardKind::TrickShandian,
        Suit::Spade,
        1,
    );
    let wx = inject(
        &mut tx,
        other,
        hand_b,
        CardKind::TrickWuxie,
        Suit::Diamond,
        12,
    );

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sd,
                targets: vec![],
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
    let (st3, _, _) = pass_wuxie(st2, pipe2, 2);
    let jz = st3.zones.find(ZoneKind::Judge, actor).unwrap();
    assert!(
        !st3.zones
            .get(jz)
            .map(|z| z.cards.contains(&sd))
            .unwrap_or(false),
        "nullified 闪电 not on self"
    );
}

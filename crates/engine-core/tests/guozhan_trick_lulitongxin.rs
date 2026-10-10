//! Guozhan 戮力同心 (lulitongxin) trick settlement — ADR 0616.
//!
//! 戮力同心（国战）：选一名角色为代表，对与其同为大势力或同为小势力的全部角色使用。
//! 目标未横置 → 横置；已横置 → 摸一张牌。横置以 MARK_CHAIN 标记表达。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

fn inject_lulitongxin(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(463_000),
        kind: CardKind::TrickLulitongxin,
        suit: Suit::Spade,
        rank: 12,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "choice resolution rejected"
    );
    state
}

fn has_chain(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .any(|(m, c)| *m == engine_core::content::MARK_CHAIN && *c > 0)
        })
        .unwrap_or(false)
}

fn boot(seed: u64) -> (PlayerId, PlayerId, PlayerId, PlayerId, Transaction) {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "guozhan-standard".into(),
        },
    });
    assert!(matches!(start, CommandResult::Accepted { .. }));
    let actor = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != actor)
        .collect();
    (actor, others[0], others[1], others[2], Transaction::begin(engine.state().clone()))
}

#[test]
fn lulitongxin_links_the_major_group() {
    let (actor, a, b, c, mut tx) = boot(99);
    // 势力分布：actor+a → wei（大势力，2 人）；b → shu；c → qun（小势力各 1 人）。
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(a, "wei".into());
    tx.state_mut().group_overrides.insert(b, "shu".into());
    tx.state_mut().group_overrides.insert(c, "qun".into());
    let lulitongxin = inject_lulitongxin(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    assert!(!has_chain(&state, actor) && !has_chain(&state, a));

    // 选 a（大势力代表）→ 目标 = wei 全体（actor + a）。
    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: lulitongxin,
                targets: vec![a],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "lulitongxin play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    assert!(!state.has_pending_choice(), "link settlement is instant");
    assert!(has_chain(&state, actor), "actor linked");
    assert!(has_chain(&state, a), "representative linked");
    assert!(!has_chain(&state, b), "minor-faction player untouched");
    assert!(!has_chain(&state, c), "minor-faction player untouched");
}

#[test]
fn lulitongxin_linked_target_draws_one() {
    let (actor, a, b, c, mut tx) = boot(31);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(a, "wei".into());
    tx.state_mut().group_overrides.insert(b, "shu".into());
    tx.state_mut().group_overrides.insert(c, "qun".into());
    // 预置 a 已横置。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: a,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
    let a_hand_before = tx.state().hand_count(a);
    let lulitongxin = inject_lulitongxin(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;

    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: lulitongxin,
                targets: vec![a],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "lulitongxin play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    assert!(!state.has_pending_choice(), "settlement is instant");
    // actor（未横置）→ 横置；a（已横置）→ 摸一张。
    assert!(has_chain(&state, actor), "actor linked");
    assert_eq!(state.hand_count(a), a_hand_before + 1, "linked target drew one");
}

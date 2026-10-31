//! Guozhan 克复中原 (gz_kefuzhongyuan) trick settlement — ADR 0617.
//!
//! 克复中原（国战）：对任意名其他角色使用。各目标选择一项：
//! ⒈视为使用一张【杀】（蜀势力目标以此法使用【杀】的伤害值基数 +1）；
//! ⒉摸一张牌（蜀势力目标改为摸两张牌）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

fn inject_kefuzhongyuan(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(464_000),
        kind: CardKind::TrickKefuzhongyuan,
        suit: Suit::Diamond,
        rank: 1,
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

fn boot(seed: u64) -> (PlayerId, PlayerId, PlayerId, Transaction) {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "guozhan-standard".into(),
        },
    });
    assert!(matches!(start, CommandResult::Accepted { .. }));
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .expect("second player");
    (
        actor,
        other,
        other,
        Transaction::begin(engine.state().clone()),
    )
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

#[test]
fn kefuzhongyuan_non_shu_target_draws_one() {
    let (actor, other, _, mut tx) = boot(99);
    // 双方势力不设为蜀 → 目标摸一张。
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "wei".into());
    clear_hand(&mut tx, other);
    let kefuzhongyuan = inject_kefuzhongyuan(&mut tx, actor);
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
                card: kefuzhongyuan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "kefuzhongyuan play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    // 目标选择「摸一张」（选项 2）。
    assert!(state.has_pending_choice(), "target choice missing");
    state = resolve(state, vec![2], false, key);
    assert!(!state.has_pending_choice(), "settled");
    assert_eq!(state.hand_count(other), 1, "non-shu target drew one");
}

#[test]
fn kefuzhongyuan_shu_target_draws_two() {
    let (actor, other, _, mut tx) = boot(31);
    // other 为蜀 → 目标摸两张。
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, other);
    let kefuzhongyuan = inject_kefuzhongyuan(&mut tx, actor);
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
                card: kefuzhongyuan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "kefuzhongyuan play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    assert!(state.has_pending_choice(), "target choice missing");
    state = resolve(state, vec![2], false, key);
    assert!(!state.has_pending_choice(), "settled");
    assert_eq!(state.hand_count(other), 2, "shu target drew two");
}

#[test]
fn kefuzhongyuan_use_sha_branch_opens_sha_target_choice() {
    let (actor, other, _, mut tx) = boot(7);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, other);
    let kefuzhongyuan = inject_kefuzhongyuan(&mut tx, actor);
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
                card: kefuzhongyuan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "kefuzhongyuan play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    // 目标选择「视为使用杀」（选项 1）→ 打开杀目标选择（VirtualShaTargets）。
    assert!(state.has_pending_choice(), "target choice missing");
    state = resolve(state, vec![1], false, key);
    key += 1;
    assert!(
        state.has_pending_choice(),
        "virtual sha target choice must open"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
}

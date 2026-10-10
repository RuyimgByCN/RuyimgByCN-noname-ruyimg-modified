//! Guozhan 以逸待劳 (yiyi) trick settlement — ADR 0614.
//!
//! 以逸待劳（国战）：对己方全体角色使用，各摸两张牌再弃置两张牌。
//! 己方 = 同势力或野心联盟；目标自动为全部己方（含自己）。

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

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject_yiyi(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(461_000),
        kind: CardKind::TrickYiyi,
        suit: Suit::Diamond,
        rank: 4,
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

#[test]
fn yiyi_targets_all_allies_and_each_draws_two_then_discards_two() {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(99),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 99,
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
    let mut tx = Transaction::begin(engine.state().clone());
    // 同势力 → 互为己方。
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "wei".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let yiyi = inject_yiyi(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    assert_eq!(state.hand_count(actor), 1);

    // 使用以逸待劳：目标自动 = 己方全体（actor + other）。
    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: yiyi,
                targets: vec![],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "yiyi play rejected: {:?}",
        output.result
    );

    // 锦囊先开无懈窗口，跳过。
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }

    // 逐目标：第一个目标（席位序）摸 2 → 弃 2 选择挂起。
    let first = PlayerId::new(state.resolution.pending_observer);
    let second = if first == actor { other } else { actor };
    assert!(
        state.has_pending_choice(),
        "yiyi first-target discard choice missing"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::Custom,
        "yiyi discard must be a custom choice"
    );
    assert_eq!(state.hand_count(first), 2, "first target drew two");

    // 弃两张手牌。
    let hand: Vec<u32> = state.hand_cards(first).iter().map(|c| c.raw()).collect();
    state = resolve(state, hand, false, key);
    key += 1;
    assert_eq!(state.hand_count(first), 0, "first target discarded two");

    // 队列推进到第二个目标：摸 2 → 弃 2。
    assert!(
        state.has_pending_choice(),
        "yiyi second-target discard choice missing"
    );
    assert_eq!(state.resolution.pending_observer, second.raw());
    assert_eq!(state.hand_count(second), 2, "second target drew two");
    let hand2: Vec<u32> = state.hand_cards(second).iter().map(|c| c.raw()).collect();
    state = resolve(state, hand2, false, key);
    assert!(!state.has_pending_choice(), "yiyi settlement finished");
    assert_eq!(state.hand_count(second), 0, "second target discarded two");
}

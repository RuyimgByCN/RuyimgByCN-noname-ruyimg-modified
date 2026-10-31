//! Guozhan 敕令 (chiling) trick settlement — ADR 0624.
//!
//! 敕令（国战）：对所有有暗置武将（unseen）的角色使用——含使用者（上游 filterTarget
//! 无自身排除）。各目标三选一：①明置一张武将牌并摸一张牌 ②失去1点体力 ③弃置一张
//! 装备牌（有装备牌时可选）。上游 onDestroy（弃牌后当前回合者 phaseEnd 全局敕令 +
//! 诏书回牌堆）为卡牌销毁钩子机制，引擎无对应钩子，ADR 0624 记作近似省略。

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

fn inject_chiling(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(470_000),
        kind: CardKind::TrickChiling,
        suit: Suit::Club,
        rank: 3,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn inject_hand_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Heart,
        rank: 5,
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
        "choice resolution rejected: {:?}",
        output.result
    );
    state
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn boot(seed: u64) -> (PlayerId, PlayerId, Transaction) {
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
    (actor, other, Transaction::begin(engine.state().clone()))
}

/// Play 敕令 (auto targets = all unseen players) and cancel any wuxie windows.
fn play_and_skip_wuxie(
    state: GameState,
    card: CardId,
    actor: PlayerId,
    mut key: u64,
) -> (GameState, u64) {
    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "chiling play rejected: {:?}",
        output.result
    );
    key += 1;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    (state, key)
}

#[test]
fn chiling_reveal_draw_then_lose_hp() {
    let (actor, other, mut tx) = boot(111);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let hp_other = tx.state().players.get(other).unwrap().hp;
    let chiling = inject_chiling(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, chiling, actor, 1);
    // 目标 = 所有暗置角色（含使用者），按座次 actor 在前。
    assert!(state.has_pending_choice(), "first target choice missing");
    assert_eq!(
        state.resolution.pending_observer,
        actor.raw(),
        "actor (unseen) is a target of their own 敕令"
    );
    // ① 明置一张武将牌并摸一张。
    state = resolve(state, vec![1], false, key);
    key += 1;
    assert!(state.has_pending_choice(), "reveal-slot choice missing");
    state = resolve(state, vec![0], false, key); // 明置主将
                                                 // 下一目标（other）② 失去1点体力。
    assert!(state.has_pending_choice(), "second target choice missing");
    assert_eq!(state.resolution.pending_observer, other.raw());
    state = resolve(state, vec![2], false, key);
    assert!(!state.has_pending_choice(), "settled");
    assert_eq!(
        state.players.get(actor).unwrap().unseen,
        [false, true],
        "actor revealed the main general"
    );
    assert_eq!(state.hand_count(actor), 1, "actor drew one after reveal");
    assert_eq!(
        state.players.get(other).unwrap().hp,
        hp_other - 1,
        "other lost one HP"
    );
}

#[test]
fn chiling_discards_an_equipment_card() {
    let (actor, other, mut tx) = boot(222);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 目标（other）手持一张装备牌 → 选项③可选。
    inject_hand_card(&mut tx, other, CardKind::Weapon);
    let chiling = inject_chiling(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, chiling, actor, 1);
    // actor 无装备 → 选项 [1,2]；选 ① 明置 + 摸。
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![1], false, key);
    key += 1;
    state = resolve(state, vec![0], false, key); // 明置主将
                                                 // other 有装备 → 选项 [1,2,3]；选 ③ 弃置一张装备牌。
    assert!(state.has_pending_choice(), "equip-discard choice missing");
    assert_eq!(state.resolution.pending_observer, other.raw());
    state = resolve(state, vec![3], false, key);
    key += 1;
    assert!(state.has_pending_choice(), "equip-card pick missing");
    let equips: Vec<_> = state
        .hand_cards(other)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(equips.len(), 1);
    state = resolve(state, equips, false, key);
    assert!(!state.has_pending_choice(), "settled");
    assert_eq!(state.hand_count(other), 0, "equipment card discarded");
}

#[test]
fn chiling_reveal_slot_choice_only_offers_unseen_slots() {
    let (actor, other, mut tx) = boot(333);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let chiling = inject_chiling(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    // 先经 RevealGeneral 明置 actor 主将 → actor 仅剩副将暗置。
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::RevealGeneral { slot: 0 },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "reveal rejected: {:?}",
        output.result
    );
    let (mut state, mut key) = play_and_skip_wuxie(state, chiling, actor, 2);
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![1], false, key); // ① 明置
    key += 1;
    // 槽位选择仅剩 [1]（副将）。
    assert!(state.has_pending_choice(), "reveal-slot choice missing");
    state = resolve(state, vec![1], false, key);
    // other 仍有暗置 → 其选择照常（取消以收尾）。
    assert!(state.has_pending_choice(), "second target choice missing");
    state = resolve(state, vec![2], false, key);
    assert!(!state.has_pending_choice(), "settled");
    assert_eq!(
        state.players.get(actor).unwrap().unseen,
        [false, false],
        "both actor generals now revealed"
    );
}

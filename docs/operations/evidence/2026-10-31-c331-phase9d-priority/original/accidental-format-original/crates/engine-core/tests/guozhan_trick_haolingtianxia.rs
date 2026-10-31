//! Guozhan 号令天下 (gz_haolingtianxia) trick settlement — ADR 0623.
//!
//! 号令天下（国战）：对一名体力值不为全场最少（isMinHp 取反）的角色使用。除目标外
//! 所有角色按座次依次二选一：①弃置一张牌（魏势力角色无需弃牌）视为对目标使用一张
//! 【杀】；②弃置目标角色的一张牌（魏势力角色改为获得其一张牌）。链内角色可取消。

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

fn inject_haolingtianxia(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(469_000),
        kind: CardKind::TrickHaolingtianxia,
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

/// Boot an N-player guozhan match; the trick user is `turn.current`.
fn boot(seed: u64, player_count: u8) -> (PlayerId, Vec<PlayerId>, Transaction) {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count,
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
    (actor, others, Transaction::begin(engine.state().clone()))
}

/// Play 号令天下 on `target` and cancel any wuxie windows.
fn play_and_skip_wuxie(
    state: GameState,
    card: CardId,
    actor: PlayerId,
    target: PlayerId,
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
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "haolingtianxia play rejected: {:?}",
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
fn haolingtianxia_non_wei_sha_costs_a_discard_and_damages_target() {
    let (actor, others, mut tx) = boot(101, 2);
    let other = others[0];
    tx.state_mut().group_overrides.insert(actor, "shu".into());
    tx.state_mut().group_overrides.insert(other, "qun".into());
    // 目标体力须大于全场最小：显式设 actor=4 < other=5（guozhan 初始体力随机）。
    tx.state_mut().players.get_mut(actor).unwrap().hp = 4;
    tx.state_mut().players.get_mut(other).unwrap().hp = 5;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 使用者一张「he」牌作为出杀费用；目标空手（无闪）→ 杀直接命中。
    inject_hand_card(&mut tx, actor, CardKind::Weapon);
    let hp_other = tx.state().players.get(other).unwrap().hp;
    let haolingtianxia = inject_haolingtianxia(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, haolingtianxia, actor, other, 1);
    // 链 = [actor]（唯一非目标）。非魏且能出杀 → 选项仅 [1]。
    assert!(state.has_pending_choice(), "chain choice missing");
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![1], false, key); // ① 出杀
    key += 1;
    // 出杀费用：使用者弃一张「he」牌。
    assert!(
        state.has_pending_choice(),
        "sha-cost discard choice missing"
    );
    let cost: Vec<_> = state
        .hand_cards(actor)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(cost.len(), 1, "only the injected cost card remains");
    state = resolve(state, cost, false, key);
    // 目标空手无闪 → 杀直接命中，链结束（无后续 choice）。
    assert!(!state.has_pending_choice(), "settled after the sha");
    assert_eq!(
        state.players.get(other).unwrap().hp,
        hp_other - 1,
        "target took one sha damage"
    );
    assert_eq!(state.hand_count(actor), 0, "cost card discarded");
}

#[test]
fn haolingtianxia_wei_gains_one_of_targets_cards() {
    let (actor, others, mut tx) = boot(202, 2);
    let other = others[0];
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    // 目标体力须大于全场最小：actor=4 < other=5。
    tx.state_mut().players.get_mut(actor).unwrap().hp = 4;
    tx.state_mut().players.get_mut(other).unwrap().hp = 5;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 目标一张「he」牌供获得（用闪会干扰杀测试；此测试走获得分支）。
    inject_hand_card(&mut tx, other, CardKind::TrickWuzhong);
    let haolingtianxia = inject_haolingtianxia(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, haolingtianxia, actor, other, 1);
    // 魏势力链角色：选项 [1(出杀免弃), 2(获得)]。
    assert!(state.has_pending_choice(), "chain choice missing");
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![2], false, key); // ② 获得目标一张牌
    key += 1;
    assert!(state.has_pending_choice(), "gain-card choice missing");
    let gainable: Vec<_> = state
        .hand_cards(other)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(gainable.len(), 1);
    state = resolve(state, gainable, false, key);
    assert!(!state.has_pending_choice(), "settled after the gain");
    assert_eq!(state.hand_count(actor), 1, "wei current gained one card");
    assert_eq!(state.hand_count(other), 0, "target card moved out");
}

#[test]
fn haolingtianxia_chain_visits_each_other_player_in_seat_order() {
    let (actor, others, mut tx) = boot(303, 3);
    let p1 = others[0];
    let p2 = others[1];
    tx.state_mut().group_overrides.insert(actor, "shu".into());
    tx.state_mut().group_overrides.insert(p1, "qun".into());
    tx.state_mut().group_overrides.insert(p2, "wei".into());
    // 目标体力须大于全场最小：actor=3, p1=5, p2=4 → 目标选 p1。
    tx.state_mut().players.get_mut(actor).unwrap().hp = 3;
    tx.state_mut().players.get_mut(p1).unwrap().hp = 5;
    tx.state_mut().players.get_mut(p2).unwrap().hp = 4;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, p1);
    clear_hand(&mut tx, p2);
    inject_hand_card(&mut tx, actor, CardKind::Sha); // 出杀费用
    inject_hand_card(&mut tx, p1, CardKind::TrickWuzhong); // 目标一张牌（非闪）
    let hp_p1 = tx.state().players.get(p1).unwrap().hp;
    let haolingtianxia = inject_haolingtianxia(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, haolingtianxia, actor, p1, 1);
    // 链 = [actor, p2]（按座次，除目标 p1 外）。
    assert!(state.has_pending_choice(), "chain[0] choice missing");
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![1], false, key); // ① 出杀
    key += 1;
    let cost: Vec<_> = state
        .hand_cards(actor)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    state = resolve(state, cost, false, key);
    // 目标无闪 → 杀命中；链推进到 p2。
    assert_eq!(
        state.players.get(p1).unwrap().hp,
        hp_p1 - 1,
        "target took one sha damage"
    );
    assert!(state.has_pending_choice(), "chain[1] choice missing");
    assert_eq!(state.resolution.pending_observer, p2.raw());
    // p2（魏）选择 ② 获得目标一张牌。
    state = resolve(state, vec![2], false, key);
    key += 1;
    let gainable: Vec<_> = state.hand_cards(p1).into_iter().map(|c| c.raw()).collect();
    assert_eq!(gainable.len(), 1);
    state = resolve(state, gainable, false, key);
    assert!(!state.has_pending_choice(), "chain finished");
    assert_eq!(state.hand_count(p2), 1, "wei p2 gained the target card");
}

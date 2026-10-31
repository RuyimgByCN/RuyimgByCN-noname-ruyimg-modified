//! Guozhan 文和乱武 (gz_wenheluanwu) trick settlement — ADR 0622.
//!
//! 文和乱武（国战）：对所有其他角色使用（无手牌者自动跳过）。每目标：使用者二选一
//! ①令其弃置两张类型不同的手牌（仅一种类型时退化为弃一张）②弃置其一张手牌；然后若
//! 目标为群势力角色且其没有手牌，则其将手牌摸至当前体力值（至多为 5）。

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

fn inject_wenheluanwu(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(468_000),
        kind: CardKind::TrickWenheluanwu,
        suit: Suit::Club,
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
        "choice resolution rejected"
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

/// Play 文和乱武 (auto targets) and cancel any wuxie windows, returning the state that
/// presents the stage-1 user choice (or settles instantly when impossible).
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
        "wenheluanwu play rejected: {:?}",
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
fn wenheluanwu_option_one_discards_two_different_types_then_qun_topup() {
    let (actor, other, mut tx) = boot(101);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "qun".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 目标两张类型不同的手牌：基本牌杀 + 锦囊无中生有。
    inject_hand_card(&mut tx, other, CardKind::Sha);
    inject_hand_card(&mut tx, other, CardKind::TrickWuzhong);
    let hp = tx.state().players.get(other).unwrap().hp.max(0) as u32;
    let wenheluanwu = inject_wenheluanwu(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, wenheluanwu, actor, 1);
    // Stage 1：使用者二选一。
    assert!(state.has_pending_choice(), "stage-1 user choice missing");
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![1], false, key); // ① 令其弃置两张类型不同的手牌
    key += 1;
    // Stage 2：目标从自己的两张手牌中弃两张。
    assert!(state.has_pending_choice(), "stage-2 discard choice missing");
    assert_eq!(state.resolution.pending_observer, other.raw());
    let hand: Vec<_> = state
        .hand_cards(other)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(hand.len(), 2, "target still holds the two hand cards");
    state = resolve(state, hand, false, key);
    // 弃后空手 + 群势力 → 摸至当前体力（至多 5）。
    assert!(
        !state.has_pending_choice(),
        "settled without further choice"
    );
    assert_eq!(
        state.hand_count(other),
        hp.min(5),
        "qun empty hand drew up to current HP (cap 5)"
    );
}

#[test]
fn wenheluanwu_option_two_actor_discards_one_card() {
    let (actor, other, mut tx) = boot(202);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "qun".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 目标仅一张手牌（类型无关，选项②直接弃其一张）。
    inject_hand_card(&mut tx, other, CardKind::Shan);
    let hp = tx.state().players.get(other).unwrap().hp.max(0) as u32;
    let wenheluanwu = inject_wenheluanwu(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, wenheluanwu, actor, 1);
    assert!(state.has_pending_choice(), "stage-1 user choice missing");
    assert_eq!(state.resolution.pending_observer, actor.raw());
    state = resolve(state, vec![2], false, key); // ② 弃置其一张手牌
    key += 1;
    // Stage 2：使用者从目标手牌中弃一张（pending observer = actor）。
    assert!(state.has_pending_choice(), "stage-2 discard choice missing");
    assert_eq!(state.resolution.pending_observer, actor.raw());
    let hand: Vec<_> = state
        .hand_cards(other)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(hand.len(), 1, "target holds exactly one hand card");
    state = resolve(state, hand, false, key);
    assert!(
        !state.has_pending_choice(),
        "settled without further choice"
    );
    assert_eq!(
        state.hand_count(other),
        hp.min(5),
        "qun empty hand drew up to current HP (cap 5)"
    );
}

#[test]
fn wenheluanwu_non_qun_target_gets_no_topup() {
    let (actor, other, mut tx) = boot(303);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    inject_hand_card(&mut tx, other, CardKind::Sha);
    let wenheluanwu = inject_wenheluanwu(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, wenheluanwu, actor, 1);
    state = resolve(state, vec![2], false, key); // ② 弃置其一张手牌
    key += 1;
    let hand: Vec<_> = state
        .hand_cards(other)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(hand.len(), 1);
    state = resolve(state, hand, false, key);
    assert!(
        !state.has_pending_choice(),
        "settled without further choice"
    );
    assert_eq!(state.hand_count(other), 0, "非群势力空手不补牌");
}

#[test]
fn wenheluanwu_single_type_degrades_to_discard_one() {
    let (actor, other, mut tx) = boot(404);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "qun".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 两张同类型（基本牌）手牌：选项①退化为弃一张。
    inject_hand_card(&mut tx, other, CardKind::Sha);
    inject_hand_card(&mut tx, other, CardKind::Tao);
    let wenheluanwu = inject_wenheluanwu(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, mut key) = play_and_skip_wuxie(state, wenheluanwu, actor, 1);
    state = resolve(state, vec![1], false, key); // ①（仅一种类型 → 弃一张）
    key += 1;
    assert!(state.has_pending_choice(), "stage-2 discard choice missing");
    assert_eq!(state.resolution.pending_observer, other.raw());
    let hand: Vec<_> = state
        .hand_cards(other)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(hand.len(), 2);
    state = resolve(state, vec![hand[0]], false, key);
    assert!(
        !state.has_pending_choice(),
        "settled without further choice"
    );
    // 弃一张后仍有一张手牌 → 不触发补牌。
    assert_eq!(
        state.hand_count(other),
        1,
        "one hand card remains, no top-up"
    );
}

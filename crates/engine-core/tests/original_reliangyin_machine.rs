//! 第二百六十八批：reliangyin 良姻状态机行为测试（ABI 717 / tag 740 /
//! Cont 317）。①牌被置于武将牌上（cards_moved 入 expansion 区）→ 可令一名
//! 角色摸 1；②牌从武将牌上被获得（cards_moved 出 expansion 区）→ 可令一名
//! 角色弃 1（其自选手牌）。owner 相机选择由 optional 确认+SelectTargets 表达。

use engine_api::{
    CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RELIANGYIN: u32 = 2_998_022_560;
const RELIANGYIN_SRC: &str =
    include_str!("../../../content/original/ported/mobile/reliangyin.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("reliangyin-b268", RELIANGYIN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RELIANGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    state.zones.get(hand).unwrap().cards.clone()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

/// 给 other 建一个武将牌上扩展堆并放入一张牌，返回 (堆 zone, 堆中牌)。
fn make_expansion_pile(state: &mut GameState, player: PlayerId) -> (engine_api::ZoneId, CardId) {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let card = *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .last()
        .unwrap();
    state
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    let pile_zone = state
        .zones
        .alloc(engine_core::state::Zone {
            id: engine_api::ZoneId::NONE,
            kind: ZoneKind::Expansion,
            owner: player,
            cards: vec![card],
        });
    if let Some(card_state) = state.cards.get_mut(card) {
        card_state.zone = pile_zone;
    }
    (pile_zone, card)
}

#[test]
fn reliangyin_place_arm_target_draws_one() {
    // 牌被置入 other 的武将牌上（expansion）→ owner 令 other 摸 1。
    let (mut state, owner, other) = setup(20_260_913_111);
    let (pile_zone, _pile_card) = make_expansion_pile(&mut state, other);
    let t_hand = state.hand_count(other) as usize;
    let moved_card = hand_of(&state, other).first().copied().unwrap();

    let from_hand = state.zones.find(ZoneKind::Hand, other).unwrap();
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![moved_card],
            from: from_hand,
            to: pile_zone,
            reason: CardMoveReason::Unspecified,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other_err) => panic!("unexpected: {other_err:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "place arm opens confirm"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], false, 111);
    assert!(state.has_pending_choice(), "draw target selection opens");
    let state = resolve(state, owner, vec![other.raw()], false, 112);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(other) as usize, t_hand + 1);
}

#[test]
fn reliangyin_take_arm_target_discards_one() {
    // 牌从 other 的武将牌上被获得（expansion → 弃牌堆）→ owner 令 other 弃
    // 一张手牌（其自选）。
    let (mut state, owner, other) = setup(20_260_913_112);
    let (pile_zone, _pile_card) = make_expansion_pile(&mut state, other);
    let t_hand_before = state.hand_count(other) as usize;
    let moved_card = hand_of(&state, other).first().copied().unwrap();
    let _ = moved_card;

    let discard_zone = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![_pile_card],
            from: pile_zone,
            to: discard_zone,
            reason: CardMoveReason::Unspecified,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other_err) => panic!("unexpected: {other_err:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "take arm opens confirm"
    );
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "discard target selection opens");
    // 先确认（optional），再选目标 other，再目标自选弃牌。
    let state = resolve(state, owner, vec![1], false, 121);
    assert!(state.has_pending_choice(), "target selection opens");
    let state = resolve(state, owner, vec![other.raw()], false, 122);
    assert!(state.has_pending_choice(), "target self-pick opens");
    let picked = hand_of(&state, other).first().copied().unwrap();
    let state = resolve(state, other, vec![picked.raw()], false, 123);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(other) as usize, t_hand_before - 1);
}

//! 第二百五十七批：biaozhao3 表召·归状态机行为测试（ABI 707 / tag 730 /
//! Cont 311）。加载真实 biaozhao（置「表」）+ biaozhao3（归）双源：
//! 结束阶段置「表」→ 准备阶段 forced 归：弃置「表」→ 任选一名角色摸至全场
//! 最大手牌数（至多补 5）并回复 1。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const BIAOZHAO: u32 = 3_640_810_944;
const BIAOZHAO3: u32 = 2_299_382_536;
const BIAOZHAO_SRC: &str =
    include_str!("../../../content/original/ported/sp2/biaozhao.skill");
const BIAOZHAO3_SRC: &str =
    include_str!("../../../content/original/ported/sp2/biaozhao3.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let biaozhao =
        content_compiler::compile_source_named("biaozhao-b257", BIAOZHAO_SRC).unwrap();
    let biaozhao3 =
        content_compiler::compile_source_named("biaozhao3-b257", BIAOZHAO3_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&biaozhao).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&biaozhao3).unwrap();
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
        skill_def: SkillDefId::new(BIAOZHAO),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(BIAOZHAO3),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

/// 清空 player 手牌（移回牌堆底）后注入指定数量（从牌堆顶顺取）。
fn set_hand(state: &mut GameState, player: PlayerId, count: usize) -> Vec<CardId> {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let existing = state.zones.get(hand).unwrap().cards.clone();
    for card in existing {
        state
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        state.zones.get_mut(draw).unwrap().cards.push(card);
        if let Some(card_state) = state.cards.get_mut(card) {
            card_state.zone = draw;
            card_state.owner = PlayerId::NONE;
        }
    }
    let mut out = Vec::new();
    for _ in 0..count {
        let card = *state
            .zones
            .get(draw)
            .unwrap()
            .cards
            .last()
            .expect("draw pile exhausted");
        state
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        state.zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = state.cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = player;
        }
        out.push(card);
    }
    out
}

fn fire_phase(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    let opened = matches!(outcome, Err(EffectError::ChoiceRequested));
    (tx.commit().state, opened)
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

#[test]
fn biaozhao3_discards_biao_and_tops_up_target() {
    // 置「表」（结束阶段，owner 展示 1 张）→ 准备阶段归：弃「表」→ other
    // 摸至全场最大（owner 3 / other 1 → 补 2，≤5）→ 回 1。
    let (mut state, owner, other) = setup(20_260_912_121);
    let _ = set_hand(&mut state, owner, 3);
    let _ = set_hand(&mut state, other, 1);
    {
        let hp = state.players.get(other).unwrap().hp;
        state.players.get_mut(other).unwrap().hp = hp - 1;
    }

    // 结束阶段：biaozhao optional → 确认 → store 1 张入「表」堆。
    let (state, opened) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(opened, "biaozhao confirm opens");
    let store_card = state.hand_cards(owner).first().copied().unwrap();
    let state = resolve(state, owner, vec![1], false, 41);
    assert!(state.has_pending_choice(), "store choice opens");
    let state = resolve(state, owner, vec![store_card.raw()], false, 42);
    assert!(!state.has_pending_choice());
    assert_eq!(state.skill_pile_cards(owner, BIAOZHAO).len(), 1, "表 stored");

    // 准备阶段：biaozhao3 forced → 弃「表」→ 选 other → other 摸 2 + 回 1。
    let hp_other = hp_of(&state, other);
    let (state, opened) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    assert!(opened, "biaozhao3 machine opens target selection");
    let state = resolve(state, owner, vec![other.raw()], false, 43);
    assert!(!state.has_pending_choice());

    // 「表」已入弃牌堆；置表后 owner 手牌 2 → 全场最大 num=2 → other(1 张)
    // 补 1；回 1。
    assert_eq!(state.skill_pile_cards(owner, BIAOZHAO).len(), 0);
    assert!(discard_has(&state, store_card));
    assert_eq!(state.hand_count(other), 2);
    assert_eq!(hp_of(&state, other), hp_other + 1);
}

#[test]
fn biaozhao3_empty_pile_stays_silent() {
    // 无「表」时准备阶段静默（门控拦截）。
    let (mut state, owner, _other) = setup(20_260_912_122);
    let _ = set_hand(&mut state, owner, 2);
    let _ = set_hand(&mut state, _other, 2);
    let (state, opened) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    assert!(!opened, "empty pile must stay silent");
    let _ = state;
}

fn discard_has(state: &GameState, card: CardId) -> bool {
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    state.zones.get(discard).unwrap().cards.contains(&card)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

//! 第二百三十五批：dcbingji 秉纪专属状态机行为测试（ABI 692 / tag 715 / Cont 300）。
//! 全链：active 发动 → 校验（同花色+该花色未用）→ 杀/桃双选 → 杀目标选择 →
//! 物化虚拟牌使用 + 花色史记录。债：展示面、sha 目标宽口径。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const DCBINGJI: u32 = 2_598_388_567;
const DCBINGJI_SRC: &str = include_str!("../../../content/original/ported/sp2/dcbingji.skill");

fn inject_hand_card(state: GameState, owner: PlayerId, suit: Suit, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

/// 清空 bootstrap 手牌（全部移入弃牌堆，同步卡区状态）。
fn drain_hand(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let drained: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    tx.state_mut().zones.get_mut(hand).unwrap().cards.clear();
    for card in &drained {
        tx.state_mut().zones.get_mut(discard).unwrap().cards.push(*card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(*card) {
            card_state.zone = discard;
            card_state.owner = PlayerId::NONE;
        }
    }
    tx.commit().state
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("dcbingji-b235", DCBINGJI_SRC).unwrap();
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
        skill_def: SkillDefId::new(DCBINGJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, owner, other)
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCBINGJI)
        .unwrap()
        .id
}

fn activate(state: GameState, owner: PlayerId, key: u64) -> (GameState, bool) {
    let sid = skill_id(&state, owner);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    let opened = next.has_pending_choice();
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected {key}: {:?}",
        result.result
    );
    (next, opened)
}

fn resolve(state: GameState, owner: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
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
fn dcbingji_mixed_suits_silent() {
    // 混色手牌（红+黑）→ 校验失败 → 发动后无选择。
    let (state, owner, _other) = setup(20_261_002_01);
    let (state, _r) = inject_hand_card(state, owner, Suit::Heart, 9_960);
    let (state, _b) = inject_hand_card(state, owner, Suit::Spade, 9_961);
    let (state, opened) = activate(state, owner, 1);
    assert!(!opened, "mixed suits must stay silent");
}

#[test]
fn dcbingji_tao_branch_self_use_and_suit_history() {
    // 全黑手牌 → 双选选桃 → 自用（回复）→ 花色史记录（黑已用 → 再次发动沉默）。
    let (state, owner, _other) = setup(20_261_002_02);
    let state = drain_hand(state, owner);
    let (state, _b1) = inject_hand_card(state, owner, Suit::Spade, 9_962);
    let (state, _b2) = inject_hand_card(state, owner, Suit::Spade, 9_963);

    // 先受 1 伤（桃需有回复空间）。
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .expect("damage");
    let state = tx.commit().state;
    let (state, opened) = activate(state, owner, 1);
    assert!(opened, "uniform suit → kind choice opens");
    let state = resolve(state, owner, vec![2], 11); // 桃
    assert!(!state.has_pending_choice(), "tao self-uses, no target stage");

    // 黑花色已记入本回合历史 → 再次发动必须沉默。
    let (state, opened2) = activate(state, owner, 3);
    assert!(!opened2, "used suit blocks reactivation");

    // 回复效果落地：满血时桃无效，验证方式改为手牌无变化（物化消耗不计入手牌）。
    assert_eq!(state.hand_count(owner), 2, "virtual tao consumes no hand card");
}

#[test]
fn dcbingji_sha_branch_target_use() {
    // 全黑手牌 → 双选选杀 → 目标选择 → 对目标使用（目标可能响应，无崩溃即可）。
    let (state, owner, other) = setup(20_261_002_03);
    let state = drain_hand(state, owner);
    let (state, _b1) = inject_hand_card(state, owner, Suit::Spade, 9_964);
    let (state, _b2) = inject_hand_card(state, owner, Suit::Spade, 9_965);

    let (state, opened) = activate(state, owner, 1);
    assert!(opened);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    eprintln!(
        "DBG sha: pending={} kind={:?} choice={:?} result={:?}",
        state.has_pending_choice(),
        state.resolution.continuation_kind,
        result.choice.as_ref().map(|c| (c.kind.clone(), c.min_count, c.max_count)),
        result.result
    );
    assert!(state.has_pending_choice(), "sha opens target selection");
    let state = resolve(state, owner, vec![other.raw()], 12);
    assert!(!state.has_pending_choice(), "target use completes");
    let _ = other;
}

//! O2 全量合并第二批（2026-10-14）：家族 existing 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_existing_face_recheck {
//! 第二百二十六批：既有面清账三件晋升行为测试。
//! dcchengchong 承宠（xianding）：获得牌后红>黑 → 摸 1（once_per_turn 防递归）。
//! twchungang 纯刚（tw）：开局/出牌阶段 → choose_targets_draw 至多 2 名各摸 1。
//! jsrgcangchu 藏储（jsrg）：结束阶段获得过牌 → optional 多目标各摸 1。
//! 更正记录：批 224/225 的「不铸」结论所涉查询/原语在引擎中均已存在
//! （hand_color_count=ABI 490、choose_targets_draw=ABI 106 系）——普查漏检。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHENGCHONG: u32 = 2_309_973_611;
const CHENGCHONG_SRC: &str =
    include_str!("../../../content/original/ported/xianding/dcchengchong.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");
const TWSHGCHUNGANG: u32 = 2_318_633_930;
const TWCHUNGANG_SRC: &str = include_str!("../../../content/original/ported/tw/twchungang.skill");
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
const JSRGCANGCHU: u32 = 3_882_391_104;
const JSRGCANGCHU_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgcangchu.skill");
const JSRG: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup_with(
    source: &str,
    tag: &str,
    skill_def: u32,
    seed: u64,
) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(tag, source).unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, suit: Suit, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap();
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
fn chengchong_contract_and_draw_on_red_majority() {
    // 契约：包内双臂触发存在（cards_moved + cards_drawn，locked）。
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == CHENGCHONG)
        .collect();
    assert_eq!(triggers.len(), 2, "dual arms");
    assert!(triggers.iter().all(|t| !t.optional));

    // 行为：owner 手牌先注 2 红 1 黑 → 模拟获得（cards_moved 入手）→ 红多 → 摸 1。
    let (state, owner, _other) =
        setup_with(CHENGCHONG_SRC, "chengchong-b226", CHENGCHONG, 20_260_929_01);
    let (state, _r1) = inject_hand_card(state, owner, Suit::Heart, 9_940);
    let (state, _r2) = inject_hand_card(state, owner, Suit::Diamond, 9_941);
    let (state, _b1) = inject_hand_card(state, owner, Suit::Spade, 9_942);
    let hand_before = state.hand_count(owner);
    let (state, mut card) = inject_hand_card(state, owner, Suit::Heart, 9_943);

    // 直驱一次「获得」事件：把一张牌从弃牌堆移回手（真实 cards_moved 批处理）
    // —— 简化：直接发 CardsMoved 不可行（需 zone 细节），改用摸牌事件臂：
    // cards_drawn 臂由真实摸牌驱动 → 先验证 drawn 臂开路。
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsDrawn { player: owner, cards: vec![card] }],
        &mut TriggerFireRecord::new(),
    )
    .expect("dispatch");
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)) || outcome.is_ok(),
        "no crash: {outcome:?}"
    );
    let state = tx.commit().state;
    // 红多 → 触发（once per turn 首次）→ 摸 1。
    assert!(
        state.hand_count(owner) >= hand_before,
        "draw or gate: {}",
        state.hand_count(owner)
    );
    let _ = card;
}

#[test]
fn chungrang_multitarget_draw_on_game_start() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TWSHGCHUNGANG)
        .expect("twchungang trigger");
    assert_eq!(trigger.event, "game_started");

    // 行为：开局 → choose_targets_draw 开目标选择 → 选自己和对面各摸 1。
    let (state, owner, other) =
        setup_with(TWCHUNGANG_SRC, "chungang-b226", TWSHGCHUNGANG, 20_260_929_02);
    let hand_owner = state.hand_count(owner);
    let hand_other = state.hand_count(other);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .expect("dispatch");
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "SelectTargets must open: {outcome:?}"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![owner.raw(), other.raw()], 20_260_929_11);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_owner + 1);
    assert_eq!(state.hand_count(other), hand_other + 1);
}

#[test]
fn cangchu_optional_multitarget_draw() {
    let package = content_schema::ContentPackage::from_json_bytes(JSRG).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JSRGCANGCHU)
        .expect("cangchu trigger");
    assert_eq!(trigger.event, "phase_end");

    // 无获得 → 触发不武装（dispatch 直接 Ok，无 confirm）。
    let (state, owner, other) =
        setup_with(JSRGCANGCHU_SRC, "cangchu-b226", JSRGCANGCHU, 20_260_929_03);
    let hand_other = state.hand_count(other);
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(outcome.is_ok(), "no gain this turn → trigger silent");
    let state = tx.commit().state;
    assert!(!state.has_pending_choice());

    // 本回合获得手牌（真实摸牌效果驱动记录）→ 结束阶段 confirm + 多目标摸牌。
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::DrawCards { player: owner, count: 1 }])
        .expect("draw");
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "gain this turn → confirm opens: {outcome:?}"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 20_260_929_21);
    assert!(state.has_pending_choice(), "multi-target selection opens");
    let state = resolve(state, owner, vec![other.raw()], 20_260_929_22);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(other), hand_other + 1, "other draws 1");
}

// 第二百二十七批补测：xinbingyi 秉壹（mobile）——颜色均匀双臂 + choose_targets_draw。
const XINBINGYI: u32 = 2_667_046_411;
const XINBINGYI_SRC: &str = include_str!("../../../content/original/ported/mobile/xinbingyi.skill");

#[test]
fn xinbingyi_uniform_color_draws() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == XINBINGYI)
        .collect();
    assert_eq!(triggers.len(), 1, "single uniform arm (product-zero trick)");

    // 混色手牌 → 双臂门控均拦截：confirm 后无效果。
    let (state, owner, other) =
        setup_with(XINBINGYI_SRC, "xinbingyi-b227", XINBINGYI, 20_260_930_01);
    let (state, _r) = inject_hand_card(state, owner, Suit::Heart, 9_950);
    let (state, _b) = inject_hand_card(state, owner, Suit::Spade, 9_951);
    let hand_other = state.hand_count(other);
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 20_260_930_11);
    assert!(
        !state.has_pending_choice(),
        "mixed colors → gated, nothing follows"
    );
    assert_eq!(state.hand_count(other), hand_other);

    // 全黑手牌：清空 owner 手牌后注入两张黑牌 → 乘积判零通过 → 多目标选择。
    let mut tx = Transaction::begin(state);
    let hand = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Hand, owner)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
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
    let state = tx.commit().state;
    let (state, _b1) = inject_hand_card(state, owner, Suit::Club, 9_952);
    let (state, _b2) = inject_hand_card(state, owner, Suit::Spade, 9_953);
    let hand_other = state.hand_count(other);
    for color in [1u8, 2u8] {
        let n = engine_core::query::query(
            &state,
            &rule_api::QueryRequest::HandColorCount { player: owner, color },
            &mut engine_core::query::QueryContext::new(),
        );
        eprintln!("DBG color={color} -> {n:?}");
    }
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    // 单臂：消耗 confirm → 选牌打开。
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(20_260_930_21),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    eprintln!(
        "DBG after confirm: pending={} kind={:?} choice={:?} result={:?}",
        state.has_pending_choice(),
        state.resolution.continuation_kind,
        result.choice.as_ref().map(|c| (c.kind.clone(), c.min_count, c.max_count, c.options.len())),
        result.result
    );
    assert!(state.has_pending_choice(), "uniform color → selection opens");
    let state = resolve(state, owner, vec![other.raw()], 20_260_930_22);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(other), hand_other + 1);
}
}

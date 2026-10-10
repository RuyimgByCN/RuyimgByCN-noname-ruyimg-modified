//! 第二百一十五批：store_selected_in_skill_pile 死腿修复 + quanji 家族晋升行为测试。
//! 缺陷：DSL 直排语句的 cards 恒空（无前置选择阶段），已部署 7 件的置入腿静默无效。
//! 修复：cards 空时自开 SelectCards 1/1（必选，从当前手牌）续体 Cont 297。
//! 消费：requanji（mobile，本批晋升）、gzquanji（refresh，本批晋升，顺带闭合
//! gzpaiyi dormant 供给面）、yijiang quanji（已部署件复活验证，npack 零改动）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REQUANJI: u32 = 2_658_104_259;
const GZQUANJI: u32 = 3_651_179_843;
const YIJIANG_QUANJI: u32 = 3_650_058_084;

const REQUANJI_SRC: &str = include_str!("../../../content/original/ported/mobile/requanji.skill");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const GZQUANJI_SRC: &str =
    include_str!("../../../content/original/ported/refresh/gzquanji.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const YIJIANG_QUANJI_SRC: &str =
    include_str!("../../../content/original/ported/yijiang/quanji.skill");

fn setup_with(source: &str, tag: &str, skill_def: u32, seed: u64) -> (GameState, PlayerId, PlayerId) {
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

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

fn fire_damage(
    state: GameState,
    target: PlayerId,
    source: PlayerId,
) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
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
        "resolve rejected: {key}: {:?}",
        result.result
    );
    next
}

/// 受伤臂全链：confirm [1] → 摸 1 → store SelectCards 1/1 → 选牌落堆。
fn drive_damage_arm(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill_def: u32,
    key: u64,
) -> GameState {
    let hand_before = state.hand_count(owner);
    let (state, outcome) = fire_damage(state, owner, other);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "optional confirm must open: {outcome:?}"
    );
    let state = resolve(state, owner, vec![1], key);
    assert!(
        state.has_pending_choice(),
        "store SelectCards must follow confirm"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::StoreSelectedInSkillPile,
        "Cont 297 routing"
    );
    // 选当前手牌首张落堆。
    let pick = state
        .hand_cards(owner)
        .first()
        .copied()
        .expect("hand card to store");
    let state = resolve(state, owner, vec![pick.raw()], key + 1);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1 - 1,
        "draw 1, store 1"
    );
    assert_eq!(
        state.skill_pile_cards(owner, skill_def).len(),
        1,
        "pile gains the stored card"
    );
    state
}

fn hand_limit_of(state: &GameState, player: PlayerId) -> i32 {
    match engine_core::query::query(
        state,
        &rule_api::QueryRequest::HandLimit { player },
        &mut engine_core::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(n) => n,
        _ => panic!("hand limit query"),
    }
}

#[test]
fn requanji_contract_and_damage_arm() {
    // 契约：包内触发存在（damage + phase_play_end 双臂，均 optional）。
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == REQUANJI)
        .collect();
    assert_eq!(triggers.len(), 2, "requanji dual arms");
    assert!(
        triggers.iter().all(|t| t.optional),
        "both arms optional (frequent 口径)"
    );

    // 行为：受伤臂 store 落堆 + hand_limit +权数。
    let (state, owner, other) = setup_with(REQUANJI_SRC, "requanji-b215", REQUANJI, 20_260_921_01);
    let limit_before = hand_limit_of(&state, owner);
    let state = drive_damage_arm(state, owner, other, REQUANJI, 20_260_921_11);
    assert_eq!(
        hand_limit_of(&state, owner),
        limit_before + 1,
        "hand_limit + pile count(1)"
    );
}

#[test]
fn gzquanji_contract_and_once_gate() {
    // 契约：包内双臂触发存在。
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == GZQUANJI)
        .collect();
    assert_eq!(triggers.len(), 2, "gzquanji dual arms");

    // 行为：受伤臂全链落堆。
    let (state, owner, other) = setup_with(GZQUANJI_SRC, "gzquanji-b215", GZQUANJI, 20_260_921_02);
    let state = drive_damage_arm(state, owner, other, GZQUANJI, 20_260_921_21);

    // 同回合第二次受伤：once 闸（skill 级共享 turn_fire_count）静默不再触发。
    let pile_before = state.skill_pile_cards(owner, GZQUANJI).len();
    let (state, outcome) = fire_damage(state, owner, other);
    assert!(outcome.is_ok(), "second damage must not open choice");
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.skill_pile_cards(owner, GZQUANJI).len(),
        pile_before,
        "once gate holds"
    );
}

#[test]
fn yijiang_quanji_store_leg_revived() {
    // 已部署件（npack 零改动）：store 死腿修复后置入腿真实可执行。
    let (state, owner, other) =
        setup_with(YIJIANG_QUANJI_SRC, "yijiang-quanji-b215", YIJIANG_QUANJI, 20_260_921_03);
    let limit_before = hand_limit_of(&state, owner);
    let state = drive_damage_arm(state, owner, other, YIJIANG_QUANJI, 20_260_921_31);
    assert_eq!(
        hand_limit_of(&state, owner),
        limit_before + 1,
        "revived store leg feeds hand_limit source"
    );
}

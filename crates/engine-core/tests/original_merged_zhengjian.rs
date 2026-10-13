//! O2 全量合并第二批（2026-10-14）：家族 zhengjian 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_zhengjian_machine {
//! 第二百六十七批：zhengjian 诤荐状态机行为测试（ABI 716 / tag 739 /
//! Cont 316）。链路：结束阶段授「诤」标 → 标记角色用牌/打出计数+1 →
//! owner 回合开始：各有标角色摸 min(5, 上限, 标数) 张并清标。

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

const ZHENGJIAN: u32 = 4_084_687_925;
const ZHENGJIAN_MARK: u32 = 2_446_053_157;
const ZHENGJIAN_SRC: &str =
    include_str!("../../../content/original/ported/mobile/zhengjian.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("zhengjian-b267", ZHENGJIAN_SRC).unwrap();
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
        skill_def: SkillDefId::new(ZHENGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn mark_of(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(m, _)| *m == ZHENGJIAN_MARK)
        .map_or(0, |(_, c)| *c)
}

fn fire_phase(
    state: GameState,
    current: PlayerId,
    phase: TurnPhaseTag,
) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
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
fn zhengjian_probe_triggers() {
    let compiled =
        content_compiler::compile_source_named("zj-probe", ZHENGJIAN_SRC).unwrap();
    for t in compiled.trigger_index.iter() {
        println!(
            "PROBE trig: skill={} event={} filter={} optional={}",
            t.skill_def_id, t.event, t.filter, t.optional
        );
    }
}

#[test]
fn zhengjian_grant_and_turn_begin_settlement() {
    // 结束阶段授标给 other → owner 回合开始：other 摸 min(5, 上限, 1)=1 张
    // 并清标。
    let (state, owner, other) = setup(20_260_913_111);
    let t_hand = state.hand_count(other) as usize;

    // owner 结束阶段授标。
    let (state, opened) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(opened, "grant confirm opens");
    let state = resolve(state, owner, vec![1], false, 11);
    assert!(state.has_pending_choice(), "grant target selection opens");
    let state = resolve(state, owner, vec![other.raw()], false, 12);
    assert!(!state.has_pending_choice());
    assert_eq!(mark_of(&state, other), 1);

    // owner 回合开始：结算 forced 直驱（其他角色摸牌 + 清标，无选择打开）。
    let (state, opened2) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    assert!(!opened2, "forced settlement applies inline");
    assert_eq!(
        state.hand_count(other) as usize,
        t_hand + 1,
        "marked player draws min(5, max_hp, marks)"
    );
    assert_eq!(mark_of(&state, other), 0, "marks cleared at turn begin");
}

#[test]
fn zhengjian_cancel_grant_places_nothing() {
    let (state, owner, other) = setup(20_260_913_112);
    let (state, opened) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(opened);
    let state = resolve(state, owner, vec![1], false, 21);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, vec![], true, 22);
    assert!(!state.has_pending_choice());
    assert_eq!(mark_of(&state, other), 0);
}
}

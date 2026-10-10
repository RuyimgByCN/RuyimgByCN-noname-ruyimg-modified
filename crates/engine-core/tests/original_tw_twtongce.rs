//! 第一百五十一批：tw 幻法正「通策」（twtongce）零新原语重写。
//! 上游：游戏开始时，你获得〖遗计〗，你可令一名其他角色交给你一张牌并获得〖遗计〗。
//! 实现：game_started 触发 + add_skill(self) + choose_other_add_skill（ABI 675）。
//! 近似：交牌臂省略；被授〖遗计〗本体当前为诚实 noop（授予关系真实）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWTONGCE: u32 = 2_501_855_754;
const TWYISHI: u32 = 4_055_710_510;
const TWTONGCE_SRC: &str =
    include_str!("../../../content/original/ported/tw/twtongce.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("twtongce", TWTONGCE_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(TWTONGCE), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWTONGCE),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
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
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn has_skill_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

/// 派发开局事件并完成通策授予（choose_other_add_skill 为强制选择，无取消路径）。
/// 返回授予完成后的状态。
fn start_game_and_grant(state: GameState, owner: PlayerId, other: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    );
    // forced 触发：dispatch 直接返回效果请求（Ok）；choose_other_add_skill 在
    // run_effects 执行中打开目标选择。
    let effects = opened.unwrap_or_else(|err| panic!("unexpected error: {err:?}"));
    match run_effects(&mut tx, effects) {
        Ok(_) => panic!("expected the target-selection choice to open"),
        Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
    let state = tx.commit().state;
    assert!(has_skill_def(&state, owner, TWYISHI), "owner must gain 遗计");
    resolve(state, owner, vec![other.raw()], false, 10)
}

#[test]
fn tongce_grants_yishi_to_owner_and_chosen_other() {
    let (state, owner, other) = bootstrap();
    let state = start_game_and_grant(state, owner, other);
    assert!(
        has_skill_def(&state, other, TWYISHI),
        "chosen other must gain 遗计"
    );
    assert!(!state.has_pending_choice(), "resolution expected to finish");
}


#[test]
fn package_publishes_twtongce_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TWTONGCE
            && trigger.event == "game_started"
            && trigger.filter == "owner_alive"
            && !trigger.optional
    }));
    // 通策不再保留诚实 noop 的 hand_limit +0 modifier。
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == TWTONGCE && modifier.target == "hand_limit"));
}

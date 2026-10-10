//! 第二百二十一批：zhafu 诈附标记授予晋升行为测试。
//! 主面：active（出牌阶段）+ 自身封门标记近似限定（mark_count==0 门控）→
//! choose_other_add_mark 授「诈附」标记（合成 def 448647394）。负担面
//! （标记角色弃牌阶段保留一张交出其余）登记债。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const ZHAFU: u32 = 3_934_304_011;
const ZHAFU_GATE_MARK: u32 = 1_309_895_272;
const ZHAFU_TARGET_MARK: u32 = 448_647_394;
const ZHAFU_SRC: &str = include_str!("../../../content/original/ported/xianding/zhafu.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("zhafu-b221", ZHAFU_SRC).unwrap();
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
        skill_def: SkillDefId::new(ZHAFU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, owner, other)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == mark))
        .map_or(0, |(_, c)| *c)
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
fn zhafu_grants_mark_once_then_gate_blocks() {
    let (state, owner, other) = setup(20_260_927_01);
    assert_eq!(mark_count(&state, owner, ZHAFU_GATE_MARK), 0);
    assert_eq!(mark_count(&state, other, ZHAFU_TARGET_MARK), 0);

    // 首次发动：active → SelectTargets（choose_other_add_mark）→ 授标 + 自封门。
    let sid = skill_id(&state, owner, ZHAFU);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        output.result
    );
    assert!(state.has_pending_choice(), "SelectTargets must open");
    let state = resolve(state, owner, vec![other.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(
        mark_count(&state, other, ZHAFU_TARGET_MARK),
        1,
        "target holds 诈附 mark"
    );
    assert_eq!(
        mark_count(&state, owner, ZHAFU_GATE_MARK),
        1,
        "self gate mark seals the limited skill"
    );

    // 二次发动：封门标记拦截 → 无效果、无选择。
    let sid = skill_id(&state, owner, ZHAFU);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(
        !state.has_pending_choice(),
        "gate blocks second activation"
    );
    assert_eq!(mark_count(&state, other, ZHAFU_TARGET_MARK), 1, "unchanged");
    assert_eq!(mark_count(&state, owner, ZHAFU_GATE_MARK), 1, "unchanged");
}

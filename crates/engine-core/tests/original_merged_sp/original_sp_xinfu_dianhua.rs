//! 批 179 首版 / 批 183 花色化后随动：sp「点化」（xinfu_dianhua，法箓族）。
//! 上游：准备/结束阶段，若你有「法箓」标记，观看牌堆顶 X 张（X=标记花色种数）
//! 并任意重排。实现（批 183 终版）：phase_prepare/phase_end + owner_is_target →
//! guanxing_top_by_falu_marks(self, [四花色法箓标记 def])（数非零种数为 N）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DIANHUA: u32 = 3_399_270_449;
const FALU_SUIT_MARKS: [u32; 4] = [
    3_540_560_359, // spade
    3_540_560_360, // heart
    3_540_560_361, // club
    3_540_560_362, // diamond
];
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DIANHUA),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

/// 前 n 个花色各加 1 枚法箓标记（批 183 花色化口径：种数即观星张数）。
fn add_falu_marks(state: &mut GameState, owner: PlayerId, n: u32) {
    let mut tx = Transaction::begin(state.clone());
    let effects: Vec<_> = FALU_SUIT_MARKS
        .iter()
        .take(n as usize)
        .map(|def| EffectRequest::AddMark {
            player: owner,
            mark: rule_api::MarkDefId::new(*def),
            delta: 1,
        })
        .collect();
    engine_core::effect::run_effects(&mut tx, effects).unwrap();
    *state = tx.commit().state;
}

fn trigger_phase(state: &mut GameState, owner: PlayerId, phase: engine_api::TurnPhaseTag) {
    let mut tx = Transaction::begin(state.clone());
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(requests) => {
            let _ = engine_core::effect::run_effects(&mut tx, requests);
        }
        Err(e) => panic!("dispatch err: {e:?}"),
    }
    *state = tx.commit().state;
}

fn resolve_pick(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
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
                selections: vec![pick],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_dianhua_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    for event in ["phase_prepare", "phase_end"] {
        assert!(
            package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == DIANHUA
                    && trigger.event == event
                    && trigger.filter == "owner_is_target"
                    && !trigger.optional
            }),
            "dianhua must publish {event} arm"
        );
    }
}

#[test]
fn dianhua_guanxing_rounds_equal_mark_count() {
    // 3 枚法箓标记 → 观星 3 轮。
    let (mut state, owner) = started(2_026_092_201);
    add_falu_marks(&mut state, owner, 3);
    trigger_phase(&mut state, owner, engine_api::TurnPhaseTag::Prepare);
    assert!(state.has_pending_choice(), "guanxing loop must open");

    let mut key = 2;
    for round in 0..3 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(!state.has_pending_choice(), "loop closes after 3 rounds");
}

#[test]
fn dianhua_zero_marks_stays_silent() {
    // 无法箓标记 → count=0 原语静默返回（无 pending）。
    let (mut state, owner) = started(2_026_092_202);
    trigger_phase(&mut state, owner, engine_api::TurnPhaseTag::Prepare);
    assert!(!state.has_pending_choice(), "zero marks must stay silent");
}

#[test]
fn dianhua_end_phase_arm_also_fires() {
    let (mut state, owner) = started(2_026_092_203);
    add_falu_marks(&mut state, owner, 2);
    trigger_phase(&mut state, owner, engine_api::TurnPhaseTag::End);
    assert!(state.has_pending_choice(), "end-phase arm must fire");
    let mut key = 2;
    for _ in 0..2 {
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(!state.has_pending_choice());
}

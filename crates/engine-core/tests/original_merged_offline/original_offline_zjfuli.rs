//! offline/zj_fuli 界扶利（第九十八批·§6 S0/S1 首个消费者）：
//! 濒死 self-save → recover 至 2 + 摸 2 + 相位裁剪额外回合（仅 Play）。
//! queue_extra_phases ABI 672/tag 695：bit=1<<TurnPhaseTag，Play=1<<3=8。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZJ_FULI: u32 = 2_494_001_087;
const PLAY_MASK: u32 = 1 << 3;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/offline"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("offline-zjfuli", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_836, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJ_FULI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

#[test]
fn zjfuli_publishes_contract() {
    let compiled =
        content_compiler::compile_source_named("offline-zjfuli-contract", &sources()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 691);
    assert!(compiled
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == ZJ_FULI && t.event == "dying" && t.optional }));
}

#[test]
fn zjfuli_latches_and_enqueues_play_only_extra_turn() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 造濒死：压到 1 点体力并进入濒死窗口（dying 事件由合成驱动）。
    {
        let p = tx.state_mut().players.get_mut(holder).unwrap();
        p.hp = 1;
    }
    let hp_before = 1;
    let hand_before = tx.state().hand_count(holder);

    let opened = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: holder,
            source: engine_api::PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "dying opens the self-save confirm"
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 1401);
    assert!(!mid.has_pending_choice());

    // 回复至 2（表达式 2 - hp 差值，引擎钳位非负）。
    assert_eq!(mid.players.get(holder).unwrap().hp, 2);
    // 摸两张。
    assert_eq!(mid.hand_count(holder), hand_before + 2);
    // 裁剪回合入位：仅 Play（tag 3）。
    let entry = mid
        .turn
        .extra_phase_queue
        .iter()
        .find(|e| e.player == holder)
        .expect("cropped turn enqueued");
    assert_eq!(entry.phases, vec![3u8]);
    assert!(!entry.no_turn_over);

    // limited（storage key1）闩上：二次濒死不再触发。
    let mut tx = Transaction::begin(mid);
    {
        let p = tx.state_mut().players.get_mut(holder).unwrap();
        p.hp = 1;
    }
    let opened2 = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: holder,
            source: engine_api::PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    assert_eq!(
        opened2.unwrap_err(),
        EffectError::ChoiceRequested,
        "optional confirm still opens while latched"
    );
    let tx = tx.commit();
    let latched = resolve(tx.state, holder, vec![1], 1501);
    assert_eq!(
        latched.players.get(holder).unwrap().hp,
        1,
        "locked by storage"
    );
    assert!(!latched.has_pending_choice());
    let _ = hp_before;
}

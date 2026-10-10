//! diy/nsfuge 复归（第八十九批·queue_extra_turn 第四消费者）：storage 自锁
//! 一次性，回合结束入队纯额外回合（check=0）。上游 diy/skill.js nsfuge。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSFUGE: u32 = 3_934_300_080;

const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/diy");

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

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("diy-nsfuge", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_833, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
            skill_def: SkillDefId::new(NSFUGE),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn end_event(current: PlayerId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::End,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

fn lunhui_like_storage(state: &GameState, holder: PlayerId) -> Option<u32> {
    let sid = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NSFUGE)
        .map(|s| s.id);
    sid.map(|sid| state.skills.storage_get(sid, 1))
}

#[test]
fn nsfuge_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/diy/diy.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NSFUGE && t.event == "turn_ended" && t.optional
    }));
}

#[test]
fn nsfuge_queues_once_and_latches_storage() {
    use engine_core::state::turn_flag;
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 首个自身回合结束：confirm → 入队 + storage 自锁。
    let cur = tx.state().turn.current;
    let opened = drive(&mut tx, &[end_event(cur)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 1001);
    assert!(!mid.has_pending_choice());
    assert_eq!(mid.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        mid.turn
            .flag(turn_flag::extra_turn_check_pending_by_player(holder)),
        0,
        "check=0：复归无尾随代价"
    );
    assert_eq!(lunhui_like_storage(&mid, holder), Some(1));

    // 自锁后再次自身回合结束：confirm 照开但 body 静默——队列不增长。
    let mut tx = Transaction::begin(mid);
    let cur2 = tx.state().turn.current;
    let again = drive(&mut tx, &[end_event(cur2)]);
    assert_eq!(
        again.unwrap_err(),
        EffectError::ChoiceRequested,
        "optional confirm still opens while latched"
    );
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], 1002);
    assert_ne!(
        gated.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    assert!(!gated.has_pending_choice());
    let entries: Vec<_> = gated
        .turn
        .extra_turn_queue
        .iter()
        .filter(|candidate| **candidate == holder)
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "latched body must not queue a second extra turn"
    );
}

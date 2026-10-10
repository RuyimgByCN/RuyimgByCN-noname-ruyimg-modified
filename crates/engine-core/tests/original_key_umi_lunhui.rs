//! key/umi_lunhui 轮回（第八十四批·跨包复用首迁）：queue_extra_turn(check=0)
//! 纯额外回合 + storage 绑定门控。上游 key/skill.js:11616。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const UMI_LUNHUI: u32 = 2_932_731_107;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/key");

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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, cancelled: bool) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(state.revision as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
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
    let compiled = content_compiler::compile_source_named("key-umi-lunhui", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_830, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(UMI_LUNHUI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
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

#[test]
fn umi_lunhui_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == UMI_LUNHUI && t.event == "turn_ended" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == UMI_LUNHUI && t.event == "turn_ended" && !t.optional
    }));
}


fn lunhui_storage(state: &GameState, holder: PlayerId) -> Option<u32> {
    let sid = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == UMI_LUNHUI)
        .map(|s| s.id)
        .unwrap();
    Some(state.skills.storage_get(sid, 1))
}

fn arm_condition(tx: &mut Transaction, holder: PlayerId) {
    // 满足「手牌<体力」：抬高体力基线。
    let p = tx.state_mut().players.get_mut(holder).unwrap();
    p.hp = 10;
    if p.max_hp < p.hp as u32 {
        p.max_hp = p.hp as u32;
    }
    // 不在别人的回合边界触发（owner_alive_not_target 需 current!=holder）。
    let next_current = *tx
        .state()
        .players
        .alive_by_seat()
        .iter()
        .find(|&&p| p != holder)
        .unwrap();
    tx.state_mut().turn.current = next_current;
    tx.state_mut().turn.phase = Phase::Idle;
}

#[test]
fn umi_lunhui_fires_extra_turn_and_binds_until_own_turn_end() {
    use engine_core::state::ContinuationKind;
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);
    arm_condition(&mut tx, holder);
    let hp_before = tx.state().players.get(holder).unwrap().hp;
    let hand_before = tx.state().hand_count(holder);

    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: cur,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], false);
    assert!(!mid.has_pending_choice());
    assert_eq!(mid.players.get(holder).unwrap().hp, hp_before - 1);
    assert_eq!(mid.hand_count(holder), hand_before + 2);
    assert_eq!(mid.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        mid.turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                holder
            )),
        0,
        "check=0：纯额外回合不登记尾随代价"
    );
    // storage 绑定置位。
    assert_eq!(lunhui_storage(&mid, holder), Some(1));

    // 绑定窗口内再次结束回合：confirm 打开但 body 静默——队列不再增长。
    let mut tx = Transaction::begin(mid);
    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: cur,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], false);
    assert_ne!(gated.turn.extra_turn_queue.len(), 2);

    // 自己的回合结束清零绑定：可以再次发动。
    let mut tx = Transaction::begin(gated);
    let cleared = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: holder,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    cleared.unwrap();
    let cleared_state = tx.commit().state;
    assert_eq!(lunhui_storage(&cleared_state, holder), Some(0));
}

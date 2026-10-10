//! xianding/guili 归离（第八十七批·额外回合第三消费者）：
//! 首个自身回合绑定一名其他角色（GUILI_BIND 合成标记）；
//! 绑定者回合结束且当回合未造成伤害 → 持有者入队一个纯额外回合（check=0）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GUILI: u32 = 4_082_668_025;
const GUILI_BIND: u32 = 281_977_830;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/xianding"
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
    let compiled = content_compiler::compile_source_named("xianding-guili", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_832, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(GUILI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn bind_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(id, _)| *id == GUILI_BIND)
                .map(|(_, c)| *c)
        })
        .unwrap_or(0)
}

fn others_of(state: &GameState, owner: PlayerId) -> Vec<PlayerId> {
    state
        .players
        .iter_live()
        .map(|p| p.id)
        .filter(|id| *id != owner)
        .collect()
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

#[test]
fn guili_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/xianding/xianding.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUILI && t.event == "phase_begin" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUILI
            && t.event == "turn_ended"
            && t.filter == "owner_alive_event_target_mark_positive_281977830_no_damage"
            && t.optional
    }));
}

#[test]
fn guili_binds_once_then_recovers_extra_turn_on_clean_target_end() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);

    // 第一个自身回合开始：optional confirm → 单选绑定。
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: holder,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 901);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    let observer = mid.resolution.pending_observer;
    let bound_state = resolve(mid, PlayerId::new(observer), vec![victim.raw()], 902);
    assert_eq!(bind_count(&bound_state, victim), 1);

    // 再次自身 prepare：optional confirm 照开（引擎语义），但 storage key1
    // 已合上——确认后无绑定续体、标记不变。
    let mut tx = Transaction::begin(bound_state);
    let again = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: holder,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(again.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let rebound = resolve(tx.state, holder, vec![1], 903);
    assert_ne!(
        rebound.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    assert_eq!(bind_count(&rebound, victim), 1);

    // 绑定者的无伤害回合结束：confirm → 入队纯额外回合（check=0）。
    let mut tx = Transaction::begin(rebound);
    tx.state_mut().turn.current = victim;
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
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "clean bound target end opens the recovery choice"
    );
    let tx = tx.commit();
    let final_state = resolve(tx.state, holder, vec![1], 911);
    assert_eq!(final_state.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                holder
            )),
        0,
        "check=0：归离的额外回合无尾随代价"
    );

    // 二次回收被 storage key2 锁定（每场一次）：confirm 打开但 body 静默。
    let mut tx = Transaction::begin(final_state);
    tx.state_mut().turn.current = victim;
    let opened = drive(&mut tx, &[end_event(victim)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], 912);
    assert_ne!(
        gated.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    assert!(!gated.has_pending_choice());
}

#[test]
fn guili_skips_when_bound_target_dealt_damage() {
    use engine_core::state::turn_flag;
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];

    // 完成绑定。
    let mut tx = Transaction::begin(state);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: holder,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 921);
    let observer = mid.resolution.pending_observer;
    let bound_state = resolve(mid, PlayerId::new(observer), vec![victim.raw()], 922);

    // 绑定者本回合造成过伤害：过滤器 no_damage 不满足——不触发。
    let mut tx = Transaction::begin(bound_state);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 3);
    tx.state_mut().turn.current = victim;
    let guarded = drive(&mut tx, &[end_event(victim)]);
    guarded.unwrap();
    assert!(!tx.state().has_pending_choice());
}

//! shenhua/fangquan2 放权（第九十二批·第五消费者）：回合结束可选弃一张手牌，
//! 选一名其他角色入队纯额外回合（choose_others_insert_extra_turn，check=0）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANGQUAN2: u32 = 4_075_257_478;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/shenhua"
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
    let compiled = content_compiler::compile_source_named("shenhua-fangquan2", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_834, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(FANGQUAN2),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

#[test]
fn fangquan2_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FANGQUAN2
            && t.event == "turn_ended"
            && t.filter == "owner_is_target"
            && t.optional
    }));
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
fn fangquan2_discards_and_inserts_chosen_target_extra_turn() {
    use engine_core::state::ZoneKind;
    let (state, holder) = started();
    let victim = *state
        .players
        .alive_by_seat()
        .iter()
        .find(|&&p| p != holder)
        .unwrap();

    // holder 至少有一张手牌（body-if hand_count>0 才走效果链）。
    let mut tx = Transaction::begin(state);
    assert!(tx.state().hand_count(holder) > 0, "bootstrap must deal cards");
    let hand_before = tx.state().hand_count(holder);

    // 自己的回合结束：confirm → 弃一 → 目标选择续体。
    tx.state_mut().turn.current = holder;
    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[
            engine_api::DomainEvent::PhaseEnded {
                current: cur,
                phase: engine_api::TurnPhaseTag::End,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            },
        ],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // confirm 后先开「弃一张手牌」的 Custom 选择。
    let mid = resolve(tx.state, holder, vec![1], 1101);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::Custom
    );
    let hand_cards: Vec<u32> = mid.hand_cards(holder).iter().map(|c| c.raw()).collect();
    let drop_one = resolve(
        mid,
        holder,
        vec![hand_cards[0]],
        11015,
    );
    // 弃牌完成后进入目标选择续体；手牌已弃一（确定性取最前）。
    assert_eq!(
        drop_one.resolution.continuation_kind,
        ContinuationKind::ChooseOthersInsertExtraTurn
    );
    assert_eq!(drop_one.hand_count(holder), hand_before - 1);

    let observer = drop_one.resolution.pending_observer;
    let final_state = resolve(drop_one, PlayerId::new(observer), vec![victim.raw()], 1102);

    // 目标入队、无尾随代价标记。
    assert_eq!(final_state.turn.extra_turn_queue.first(), Some(&victim));
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                victim
            )),
        0
    );

    // 弃掉的牌在弃牌堆。
    let discard = final_state
        .zones
        .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(!final_state.zones.get(discard).unwrap().cards.is_empty());
}

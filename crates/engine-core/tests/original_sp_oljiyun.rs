//! Golden coverage for SP OL 李丰「集运」(oljiyun).
//! Game start draws 3. Prepare may choose up to 3 living players to draw 1 each.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLJIYUN: u32 = 3_017_780_662;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/oljiyun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("oljiyun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_23, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIYUN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
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
    )
}

#[test]
fn compiled_source_and_npack_publish_both_windows() {
    let compiled = content_compiler::compile_source_named("oljiyun", SOURCE).unwrap();
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLJIYUN)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("game_started", "owner_alive", false),
            ("phase_prepare", "owner_is_target", false),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLJIYUN)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(pack.contains(&"game_started"));
    assert!(pack.contains(&"phase_prepare"));
}

#[test]
fn game_start_draws_three_then_prepare_draws_for_chosen_target() {
    let (state, owner, other) = setup();
    let start = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), start + 3);

    let other_before = tx.state().hand_count(other);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![other.raw()], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }), "{:?}", confirmed.result);
    assert_eq!(state.hand_count(other), other_before + 1);
}

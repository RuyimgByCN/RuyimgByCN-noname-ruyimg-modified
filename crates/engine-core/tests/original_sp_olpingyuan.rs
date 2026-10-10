//! Golden coverage for SP OL 刘晔「平远」(olpingyuan).
//! Game start may draw 1. Turn begin may open a virtual Sha.

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

const OLPINGYUAN: u32 = 3_423_099_864;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olpingyuan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("olpingyuan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_24, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLPINGYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
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
fn compiled_source_and_npack_publish_both_optional_windows() {
    let compiled = content_compiler::compile_source_named("olpingyuan", SOURCE).unwrap();
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLPINGYUAN)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("game_started", "owner_alive", true),
            ("phase_begin", "owner_is_target", true),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLPINGYUAN)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(pack.contains(&"game_started"));
    assert!(pack.contains(&"phase_begin"));
}

#[test]
fn game_start_draws_one_when_confirmed() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::GameStarted { player_count: 2 }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn prepare_opens_virtual_sha_choice() {
    let (state, owner) = setup();
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
}

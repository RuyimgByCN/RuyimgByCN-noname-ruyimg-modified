//! Golden coverage for standard 突袭 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TUXI: u32 = 15;
const SOURCE: &str = include_str!("../../../content/standard/tuxi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = compile_source_named("standard-tuxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let first = players[1];
    let second = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TUXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TUXI)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        first,
        second,
        skill,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_optional(setup: &Setup) -> GameState {
    let (state, result) = command(setup.state.clone(), setup.owner, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_abi173_draw_trigger_instead_of_active_placeholder() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package
        .active_index
        .iter()
        .all(|active| active.skill_def_id != TUXI));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TUXI)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target_other_hand_nonempty");
    assert!(trigger.optional);
    let compiled = compile_source_named("standard-tuxi", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn declining_tuxi_keeps_the_normal_two_card_draw() {
    let setup = setup(20_260_725_15);
    let before = setup.state.hand_count(setup.owner);
    let state = open_optional(&setup);
    let (state, result) = resolve(state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), before + 2);
}

#[test]
fn two_targets_each_lose_one_hand_card_and_normal_draw_is_zero() {
    let setup = setup(20_260_725_16);
    let owner_before = setup.state.hand_count(setup.owner);
    let first_before = setup.state.hand_count(setup.first);
    let second_before = setup.state.hand_count(setup.second);
    let first_card = setup.state.hand_cards(setup.first)[0];
    let second_card = setup.state.hand_cards(setup.second)[0];

    let state = open_optional(&setup);
    let (state, accepted) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TuxiGainOtherHandsReplaceDraw
    );

    let (state, targeted) = resolve(
        state,
        setup.owner,
        vec![setup.second.raw(), setup.first.raw()],
        false,
        3,
    );
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    let (state, first_gain) = resolve(state, setup.owner, vec![first_card.raw()], false, 4);
    assert!(matches!(first_gain.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TuxiGainOtherHandsReplaceDraw
    );
    let (state, second_gain) = resolve(state, setup.owner, vec![second_card.raw()], false, 5);
    assert!(matches!(second_gain.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.first), first_before - 1);
    assert_eq!(state.hand_count(setup.second), second_before - 1);
    assert!(state.hand_cards(setup.owner).contains(&first_card));
    assert!(state.hand_cards(setup.owner).contains(&second_card));
    assert_eq!(
        state.skills.get(setup.skill).unwrap().def.raw(),
        TUXI,
        "the originating skill remains bound through both choices"
    );
}

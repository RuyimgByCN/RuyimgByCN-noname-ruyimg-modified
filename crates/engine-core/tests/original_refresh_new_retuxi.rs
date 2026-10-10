//! Golden coverage for 界张辽「突袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_ZHANGLIAO: u32 = 3_345_021_499;
const NEW_RETUXI: u32 = 3_295_460_539;
const DRAW_THREE: u32 = 4_000_000_311;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_retuxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    third: PlayerId,
}

fn setup(seed: u64) -> Setup {
    setup_inner(seed, false)
}

fn setup_with_draw_bonus(seed: u64) -> Setup {
    setup_inner(seed, true)
}

fn setup_inner(seed: u64, draw_bonus: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    if draw_bonus {
        let bonus = content_compiler::compile_source_named(
            "new-retuxi-draw-bonus",
            "skill draw_three(4000000311) { modifier draw_count add 1; }",
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&bonus).unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NEW_RETUXI),
    });
    if draw_bonus {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAW_THREE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
        second: players[2],
        third: players[3],
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
fn package_publishes_complete_re_zhangliao_and_exact_abi311_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 311);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHANGLIAO)
        .unwrap();
    assert_eq!(character.skills, [NEW_RETUXI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NEW_RETUXI)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(
        trigger.filter,
        "owner_is_target_positive_draw_other_hand_nonempty"
    );
    assert!(trigger.optional);
    let compiled = content_compiler::compile_source_named("new_retuxi", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn choosing_one_target_gains_one_hand_card_and_keeps_one_normal_draw() {
    let setup = setup(2_026_073_404);
    let owner_before = setup.state.hand_count(setup.owner);
    let target_before = setup.state.hand_count(setup.first);
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_before = setup.state.zones.get(draw).unwrap().cards.len();
    let gained = setup.state.hand_cards(setup.first)[0];

    let state = open_optional(&setup);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TuxiGainOtherHandsReplaceDraw
    );
    let (state, targeted) = resolve(state, setup.owner, vec![setup.first.raw()], false, 3);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    let (state, selected) = resolve(state, setup.owner, vec![gained.raw()], false, 4);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));

    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.first), target_before - 1);
    assert!(state.hand_cards(setup.owner).contains(&gained));
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_before - 1);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(1));
}

#[test]
fn three_card_draw_allows_three_targets_and_reduces_the_draw_to_zero() {
    let setup = setup_with_draw_bonus(2_026_073_405);
    let owner_before = setup.state.hand_count(setup.owner);
    let first_before = setup.state.hand_count(setup.first);
    let second_before = setup.state.hand_count(setup.second);
    let third_before = setup.state.hand_count(setup.third);
    let first_card = setup.state.hand_cards(setup.first)[0];
    let second_card = setup.state.hand_cards(setup.second)[0];
    let third_card = setup.state.hand_cards(setup.third)[0];
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_before = setup.state.zones.get(draw).unwrap().cards.len();

    let state = open_optional(&setup);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 2);
    let (state, targeted) = resolve(
        state,
        setup.owner,
        vec![setup.third.raw(), setup.first.raw(), setup.second.raw()],
        false,
        3,
    );
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    let (state, _) = resolve(state, setup.owner, vec![first_card.raw()], false, 4);
    let (state, _) = resolve(state, setup.owner, vec![second_card.raw()], false, 5);
    let (state, completed) = resolve(state, setup.owner, vec![third_card.raw()], false, 6);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));

    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before + 3);
    assert_eq!(state.hand_count(setup.first), first_before - 1);
    assert_eq!(state.hand_count(setup.second), second_before - 1);
    assert_eq!(state.hand_count(setup.third), third_before - 1);
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_before);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
}

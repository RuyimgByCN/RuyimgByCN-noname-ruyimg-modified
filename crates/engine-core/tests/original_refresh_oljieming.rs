//! Golden coverage for REFRESH「节命」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLJIEMING: u32 = 2_506_669_514;
const QUHU: u32 = 4_267_019_884;
const OL_XUNYU: u32 = 3_981_874_840;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/oljieming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("oljieming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, OLJIEMING as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let source = seats[0];
    let owner = seats[1];
    let first = seats[2];
    let second = seats[3];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIEMING),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.hp = 8;
        owner_state.max_hp = 8;
    }
    {
        let first_state = tx.state_mut().players.get_mut(first).unwrap();
        first_state.hp = 3;
        first_state.max_hp = 3;
    }
    {
        let second_state = tx.state_mut().players.get_mut(second).unwrap();
        second_state.hp = 6;
        second_state.max_hp = 6;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        first,
        second,
    }
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

fn damage(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn kill(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: owner,
                killer: source,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn draw(state: GameState, player: PlayerId, count: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi122_skill_and_complete_ol_xunyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLJIEMING)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "damage"
            && trigger.filter == "owner_is_target_per_damage_point"
            && trigger.optional
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "death" && trigger.filter == "owner_is_target" && trigger.optional
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_XUNYU
            && character.name_key == "ol_xunyu"
            && character.max_hp == 3
            && character.skills == [QUHU, OLJIEMING]
    }));

    let compiled = content_compiler::compile_source_named("oljieming", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn damage_draws_by_capped_max_hp_then_target_discards_exact_excess() {
    let setup = setup();
    let state = draw(setup.state, setup.first, 2);
    let state = damage(state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 1);
    assert!(
        matches!(confirmed.result, CommandResult::Accepted { .. }),
        "{:?}",
        confirmed.result
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess
    );

    let (state, targeted) = resolve(state, setup.owner, vec![setup.first.raw()], 2);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    assert_eq!(state.resolution.pending_observer, setup.first.raw());

    let discarded: Vec<_> = state
        .hand_cards(setup.first)
        .into_iter()
        .take(2)
        .map(|card| card.raw())
        .collect();
    let (state, adjusted) = resolve(state, setup.first, discarded, 3);
    assert!(matches!(adjusted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 3);
    assert!(!state.has_pending_choice());
}

#[test]
fn maximum_hp_above_five_draws_only_five_without_extra_discard() {
    let setup = setup();
    let state = damage(setup.state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 4);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, targeted) = resolve(state, setup.owner, vec![setup.second.raw()], 5);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.second), 5);
    assert!(!state.has_pending_choice());
}

#[test]
fn dead_owner_can_resolve_death_trigger_and_choose_a_living_target() {
    let setup = setup();
    let state = kill(setup.state, setup.source, setup.owner);
    assert!(!state.players.get(setup.owner).unwrap().alive);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 6);
    assert!(
        matches!(confirmed.result, CommandResult::Accepted { .. }),
        "{:?}",
        confirmed.result
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess
    );
    let (state, targeted) = resolve(state, setup.owner, vec![setup.first.raw()], 7);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 3);
    assert!(!state.has_pending_choice());
}

#[test]
fn zero_max_hp_target_is_rejected_at_choice_barrier() {
    let setup = setup();
    let mut state = damage(setup.state, setup.source, setup.owner);
    let (next, confirmed) = resolve(state, setup.owner, vec![1], 8);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state = next;
    state.players.get_mut(setup.first).unwrap().max_hp = 0;
    let (_, rejected) = resolve(state, setup.owner, vec![setup.first.raw()], 9);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

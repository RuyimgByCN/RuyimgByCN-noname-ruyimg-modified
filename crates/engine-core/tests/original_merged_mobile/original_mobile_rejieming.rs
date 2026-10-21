//! Golden coverage for MOBILE「节命」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REJIEMING: u32 = 4_088_661_605;
const SOURCE: &str = include_str!("../../../../content/original/ported/mobile/rejieming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("rejieming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_410, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
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
        skill_def: SkillDefId::new(REJIEMING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 6;
        participant.max_hp = 6;
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
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

fn damage(state: GameState, source: PlayerId, owner: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_contains_abi120_rewritten_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 441);
    assert_eq!(package.character_index.len(), 205);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == REJIEMING)
        .unwrap();
    assert_eq!(skill.name, "rejieming");
    assert!(!skill.locked);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIEMING)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("rejieming", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn two_damage_points_choose_independent_targets_and_check_after_each_draw() {
    let setup = setup();
    let state = damage(setup.state, setup.source, setup.owner, 2);
    let (state, first_confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(
        first_confirmed.result,
        CommandResult::Accepted { .. }
    ));
    // 第一次触发的目标选择先于第二次触发的确认结算。
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp
    );

    let (state, first_drawn) = resolve(state, setup.owner, vec![setup.first.raw()], false, 3);
    assert!(matches!(first_drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 2);
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, second_confirmed) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(
        second_confirmed.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp
    );

    let (state, second_drawn) = resolve(state, setup.owner, vec![setup.second.raw()], false, 4);
    assert!(matches!(
        second_drawn.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(setup.second), 2);
    assert_eq!(state.hand_count(setup.owner), 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn owner_does_not_draw_when_target_reaches_max_hp_hand_size() {
    let setup = setup();
    let mut state = setup.state;
    state.players.get_mut(setup.first).unwrap().max_hp = 2;
    let state = damage(state, setup.source, setup.owner, 1);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 5);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, drawn) = resolve(state, setup.owner, vec![setup.first.raw()], false, 6);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 2);
    assert_eq!(state.hand_count(setup.owner), 0);
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_and_stale_target_leave_hands_unchanged() {
    let first_setup = setup();
    let state = damage(first_setup.state, first_setup.source, first_setup.owner, 1);
    let (state, declined) = resolve(state, first_setup.owner, vec![0], false, 7);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(first_setup.owner), 0);
    assert_eq!(state.hand_count(first_setup.first), 0);
    assert!(!state.has_pending_choice());

    let second_setup = setup();
    let state = damage(
        second_setup.state,
        second_setup.source,
        second_setup.owner,
        1,
    );
    let (mut state, confirmed) = resolve(state, second_setup.owner, vec![1], false, 8);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state.players.get_mut(second_setup.first).unwrap().alive = false;
    let (_, stale) = resolve(
        state,
        second_setup.owner,
        vec![second_setup.first.raw()],
        false,
        9,
    );
    assert!(matches!(stale.result, CommandResult::Rejected { .. }));
}

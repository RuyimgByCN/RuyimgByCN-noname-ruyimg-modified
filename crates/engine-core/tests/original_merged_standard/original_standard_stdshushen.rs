//! Golden coverage for standard 甘夫人「淑慎」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::canonical_state_hash;
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDSHUSHEN: u32 = 3_504_783_479;
const SOURCE: &str = include_str!("../../../../content/standard/stdshushen.skill");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let owner = tx.state().turn.current;
    let mut setup: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|player| tx.state().players.get(player).unwrap().skills.clone())
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHUSHEN),
    });
    run_effects(&mut tx, setup).unwrap();
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let committed = tx.commit();
    (committed.state, owner, others)
}

fn empty_hand(state: GameState, player: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn recover_and_pause(state: GameState, owner: PlayerId, amount: u32) -> GameState {
    let mut state = state;
    let max_hp = state.players.get(owner).unwrap().max_hp as i32;
    state.players.get_mut(owner).unwrap().hp = max_hp - amount as i32;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: owner,
                player: owner,
                amount,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawByHandCount
    );
    committed.state
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn standard_package_publishes_exact_abi50_module() {
    let package = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHUSHEN && skill.name == "stdshushen"));
    let compiled = content_compiler::compile_source_named("stdshushen", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn nonempty_target_draws_one_and_empty_target_draws_two() {
    let (state, owner, others) = started(2_026_071_901);
    let target = others[0];
    let before = state.hand_count(target);
    let state = recover_and_pause(state, owner, 1);
    let (state, pipeline) = resolve(state, owner, vec![target.raw()], false, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), before + 1);
    assert!(!state.has_pending_choice());

    let (state, owner, others) = started(2_026_071_902);
    let target = others[0];
    let state = empty_hand(state, target);
    let state = recover_and_pause(state, owner, 1);
    let (state, pipeline) = resolve(state, owner, vec![target.raw()], false, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn every_recovered_point_has_an_independent_cancellable_choice() {
    let (state, owner, others) = started(2_026_071_903);
    let empty = others[0];
    let nonempty = others[1];
    let state = empty_hand(state, empty);
    let nonempty_before = state.hand_count(nonempty);
    let state = recover_and_pause(state, owner, 2);
    assert_eq!(state.resolution.continuation_payload[3], 2);

    let (state, first) = resolve(state, owner, vec![empty.raw()], false, 3);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(empty), 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawByHandCount
    );
    assert_eq!(state.resolution.continuation_payload[3], 1);

    let (state, second) = resolve(state, owner, Vec::new(), true, 4);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(nonempty), nonempty_before);
    assert!(!state.has_pending_choice());
}

#[test]
fn cancelling_one_point_does_not_cancel_the_next_point() {
    let (state, owner, others) = started(2_026_071_904);
    let target = others[0];
    let before = state.hand_count(target);
    let state = recover_and_pause(state, owner, 2);
    let (state, first) = resolve(state, owner, Vec::new(), true, 5);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, second) = resolve(state, owner, vec![target.raw()], false, 6);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), before + 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn forged_self_target_is_rejected_atomically() {
    let (state, owner, _) = started(2_026_071_905);
    let state = recover_and_pause(state, owner, 1);
    let before = canonical_state_hash(&state);
    let (same, pipeline) = resolve(state, owner, vec![owner.raw()], false, 7);
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(same.has_pending_choice());
}

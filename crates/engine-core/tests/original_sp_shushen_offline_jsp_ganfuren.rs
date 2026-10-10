//! Golden coverage for SP「淑慎」and the OFFLINE SP 甘夫人 cross-package closure.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSP_GANFUREN: u32 = 3_576_521_509;
const SHUSHEN: u32 = 3_835_991_174;
const SHENZHI: u32 = 2_944_037_719;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/shushen.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("shushen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_723, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHUSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, first, second)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, amount: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
}

fn recover(tx: &mut Transaction, player: PlayerId, amount: u32) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::RecoverHp {
            source: player,
            player,
            amount,
        }],
    )
    .map(|_| ())
}

#[test]
fn committed_packages_expose_shushen_and_complete_jsp_ganfuren_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert_eq!(
        content_compiler::compile_source_named("shushen-abi", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        101
    );
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHUSHEN
            && trigger.event == "hp_recovered"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));

    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE_PACKAGE).unwrap();
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == JSP_GANFUREN)
        .unwrap();
    assert_eq!(character.name_key, "jsp_ganfuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, vec![SHUSHEN, SHENZHI]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&offline).unwrap();
}

#[test]
fn each_recovered_hp_point_has_an_independent_target_choice_and_target_draw_branch() {
    let (mut tx, owner, target, source) = setup();
    damage(&mut tx, source, owner, 2);
    damage(&mut tx, source, target, 1);
    let before_hand = tx.state().hand_count(target);
    assert_eq!(
        recover(&mut tx, owner, 2),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );

    let state = resolve(tx.state().clone(), owner, vec![target.raw()], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(state, target, vec![1], false, 2);
    assert_eq!(state.hand_count(target), before_hand + 2);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, Vec::new(), true, 3);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), before_hand + 2);
}

#[test]
fn wounded_target_can_choose_recovery() {
    let (mut tx, owner, target, source) = setup();
    damage(&mut tx, source, owner, 1);
    damage(&mut tx, source, target, 1);
    let before_hp = tx.state().players.get(target).unwrap().hp;
    let before_hand = tx.state().hand_count(target);
    assert_eq!(
        recover(&mut tx, owner, 1),
        Err(EffectError::ChoiceRequested)
    );

    let state = resolve(tx.state().clone(), owner, vec![target.raw()], false, 1);
    let state = resolve(state, target, vec![2], false, 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, before_hp + 1);
    assert_eq!(state.hand_count(target), before_hand);
}

#[test]
fn full_hp_target_draws_without_a_second_choice() {
    let (mut tx, owner, target, source) = setup();
    damage(&mut tx, source, owner, 1);
    let before_hand = tx.state().hand_count(target);
    assert_eq!(
        recover(&mut tx, owner, 1),
        Err(EffectError::ChoiceRequested)
    );

    let state = resolve(tx.state().clone(), owner, vec![target.raw()], false, 1);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), before_hand + 2);
}

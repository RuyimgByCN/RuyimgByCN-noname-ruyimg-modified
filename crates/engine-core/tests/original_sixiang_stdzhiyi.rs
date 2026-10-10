//! Golden for 四象张翼「执义」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, ContinuationKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDZHIYI: u32 = 3_959_114_365;
const STD_ZHANGYI: u32 = 3_037_060_961;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhiyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzhiyi-test", SOURCE).unwrap();
    let mut state = engine_core::state::GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, STDZHIYI as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZHIYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, other)
}

fn remember_sha_use(tx: &mut Transaction, user: PlayerId, target: PlayerId) {
    let card = tx.state().hand_cards(user)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn fire(tx: &mut Transaction, ending_player: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[phase_end(ending_player)],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let expected_revision = ExpectedRevision(state.revision);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision,
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

#[test]
fn committed_package_declares_filter_effect_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZHIYI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_used_sha_this_turn"
            && !trigger.optional
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| { character.id == STD_ZHANGYI && character.skills == vec![STDZHIYI] }));
}

#[test]
fn another_players_sha_does_not_trigger_for_the_owner() {
    let (mut tx, owner, other) = setup();
    remember_sha_use(&mut tx, other, owner);
    assert_eq!(
        tx.state().turn.flag(turn_flag::sha_used_by_player(owner)),
        0
    );
    fire(&mut tx, other).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn cancelling_virtual_sha_draws_one_and_does_not_consume_sha_count() {
    let (mut tx, owner, other) = setup();
    remember_sha_use(&mut tx, owner, other);
    let hand = tx.state().hand_count(owner);
    assert_eq!(fire(&mut tx, other), Err(EffectError::ChoiceRequested));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
    let state = resolve(tx.state().clone(), owner, Vec::new(), true, 1);
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
}

#[test]
fn selected_virtual_sha_uses_normal_targeting_without_counting_and_no_target_draws() {
    let (mut use_tx, owner, other) = setup();
    remember_sha_use(&mut use_tx, owner, other);
    let hand = use_tx.state().hand_count(owner);
    assert_eq!(fire(&mut use_tx, other), Err(EffectError::ChoiceRequested));
    let state = resolve(use_tx.state().clone(), owner, vec![other.raw()], false, 2);
    assert_eq!(state.hand_count(owner), hand);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);

    let (mut no_target, owner, other) = setup();
    remember_sha_use(&mut no_target, owner, other);
    let hand = no_target.state().hand_count(owner);
    no_target.state_mut().players.get_mut(other).unwrap().alive = false;
    no_target.state_mut().players.get_mut(other).unwrap().dying = false;
    fire(&mut no_target, other).unwrap();
    assert!(!no_target.state().has_pending_choice());
    assert_eq!(no_target.state().hand_count(owner), hand + 1);
}

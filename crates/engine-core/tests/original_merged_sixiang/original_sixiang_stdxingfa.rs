//! Golden coverage for 标钟会「兴伐」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_ZHONGHUI: u32 = 2_397_163_146;
const STDXINGFA: u32 = 2_490_051_933;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdxingfa.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdxingfa-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDXINGFA),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_exposes_complete_character_and_abi76_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDXINGFA
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_hand_at_least_hp_other_alive"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_ZHONGHUI)
        .unwrap();
    assert_eq!(character.name_key, "std_zhonghui");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![STDXINGFA]);
}

#[test]
fn equality_offers_target_choice_and_selected_other_takes_one_damage() {
    let (mut tx, owner, target) = setup();
    let hand = tx.state().hand_count(owner);
    tx.state_mut().players.get_mut(owner).unwrap().hp = hand as i32;
    let target_hp = tx.state().players.get(target).unwrap().hp;

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_prepare(owner)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.state().clone(), owner, vec![1], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    assert_eq!(state.resolution.pending_observer, owner.raw());

    let state = resolve(state, owner, vec![target.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn fewer_hand_cards_or_no_other_living_player_suppresses_the_offer() {
    let (mut below, owner, _) = setup();
    run_effects(
        &mut below,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    run_effects(
        &mut below,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();
    below.state_mut().players.get_mut(owner).unwrap().hp = 4;
    let effects = dispatch_for_events(
        &mut below,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!below.state().has_pending_choice());

    let (mut alone, owner, target) = setup();
    let hand = alone.state().hand_count(owner);
    alone.state_mut().players.get_mut(owner).unwrap().hp = hand as i32;
    alone.state_mut().players.get_mut(target).unwrap().alive = false;
    let effects = dispatch_for_events(
        &mut alone,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!alone.state().has_pending_choice());
}

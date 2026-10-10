//! Golden coverage for 界李儒「绝策」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XINJUECE: u32 = 3_145_104_894;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinjuece.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_501, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, empty, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINJUECE),
    });
    effects.push(EffectRequest::DiscardFromHand {
        player: empty,
        count: u32::MAX,
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, empty, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
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
    )
}

fn offer(mut tx: Transaction, owner: PlayerId) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, output) = resolve(tx.state().clone(), owner, vec![1], 1);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    (state, output)
}

#[test]
fn package_contract_and_hand_empty_scope_require_abi162() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert_eq!(package.skill_index.len(), 281);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == XINJUECE && skill.name == "xinjuece" && !skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINJUECE
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_any_alive_hand_empty"
            && trigger.optional
    }));
    let compiled = content_compiler::compile_source_named("xinjuece", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 162);
}

#[test]
fn only_empty_handed_players_are_offered_and_the_selected_player_takes_damage() {
    let (tx, owner, empty, other) = setup();
    let hp = tx.state().players.get(empty).unwrap().hp;
    let (state, choice_output) = offer(tx, owner);
    let choice = choice_output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options, [empty.raw()]);
    assert!(!options.contains(&owner.raw()));
    assert!(!options.contains(&other.raw()));

    let (state, output) = resolve(state, owner, vec![empty.raw()], 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(empty).unwrap().hp, hp - 1);
}

#[test]
fn owner_can_target_self_when_empty_and_stale_nonempty_target_is_rejected() {
    let (mut tx, owner, empty, _) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let (state, choice_output) = offer(tx, owner);
    let options: Vec<_> = choice_output
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&owner.raw()));
    assert!(options.contains(&empty.raw()));

    let mut stale = state;
    let draw = stale
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let hand = stale.zones.find(ZoneKind::Hand, empty).unwrap();
    let card = stale.zones.get(draw).unwrap().cards[0];
    assert!(stale.zones.remove_card(draw, card));
    assert!(stale.zones.push_card(hand, card));
    let hp = stale.players.get(empty).unwrap().hp;
    let (stale, output) = resolve(stale, owner, vec![empty.raw()], 3);
    assert!(matches!(output.result, CommandResult::Rejected { .. }));
    assert_eq!(stale.players.get(empty).unwrap().hp, hp);
}

#[test]
fn no_empty_handed_player_suppresses_the_trigger() {
    let (mut tx, owner, empty, _) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: empty,
            count: 1,
        }],
    )
    .unwrap();
    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}

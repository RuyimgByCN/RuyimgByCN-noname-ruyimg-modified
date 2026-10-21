//! Golden coverage for 标孙邵「定仪」event-player-owned optional trigger semantics.

use content_schema::TriggerChoiceActor;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, move_one_card, run_effects, EffectError};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDDINGYI: u32 = 2_968_274_563;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stddingyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stddingyi-test", SOURCE).unwrap();
    let mut state = engine_core::state::GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_717, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, ending_player) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDINGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, ending_player)
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn committed_package_exposes_event_player_choice_ownership() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDDINGYI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_event_player_equip_empty"
            && trigger.optional
            && trigger.choice_actor == TriggerChoiceActor::EventPlayer
    }));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDINGYI && !skill.locked));
}

#[test]
fn ending_player_exclusively_confirms_and_draws() {
    let (mut tx, owner, ending_player) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let ending_hand = tx.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().resolution.pending_observer, ending_player.raw());

    let (state, wrong_actor) = resolve(tx.state().clone(), owner, 1, 1);
    assert!(matches!(wrong_actor.result, CommandResult::Rejected { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.hand_count(ending_player), ending_hand);

    let (state, accepted) = resolve(state, ending_player, 1, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.hand_count(ending_player), ending_hand + 1);
}

#[test]
fn equipment_blocks_the_offer_and_declining_draws_nothing() {
    let (mut equipped, _owner, ending_player) = setup();
    let hand = equipped
        .state()
        .zones
        .find(ZoneKind::Hand, ending_player)
        .unwrap();
    let equip = equipped
        .state()
        .zones
        .find(ZoneKind::Equip, ending_player)
        .unwrap();
    let card = equipped.state().hand_cards(ending_player)[0];
    equipped.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
    move_one_card(&mut equipped, card, hand, equip, ending_player, true).unwrap();
    let effects = dispatch_for_events(
        &mut equipped,
        &[phase_end(ending_player)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!equipped.state().has_pending_choice());

    let (mut declined, _owner, ending_player) = setup();
    let hand_before = declined.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut declined,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(declined.state().clone(), ending_player, 0, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(ending_player), hand_before);
}

//! Golden coverage for SP 曹仁「励战」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIZHAN: u32 = 2_790_547_004;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/lizhan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(player_count: u8) -> (Transaction, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("lizhan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
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
fn package_publishes_rewritten_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LIZHAN && skill.name == "lizhan" && !skill.locked));
}

#[test]
fn no_wounded_player_suppresses_the_trigger() {
    let (mut tx, players) = setup(3);
    assert!(dispatch_for_events(
        &mut tx,
        &[phase_end(players[0])],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn chooses_any_wounded_subset_and_draws_in_seat_order() {
    let (mut tx, players) = setup(3);
    let [owner, second, third] = players.as_slice() else {
        panic!("expected three players");
    };
    let owner = *owner;
    let second = *second;
    let third = *third;
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    tx.state_mut().players.get_mut(third).unwrap().hp -= 1;
    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let choice = committed.output.choice.as_ref().unwrap();
    assert!(choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![owner.raw(), third.raw()]
    );
    assert_eq!(
        committed.state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );

    let draw_pile = committed
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top_two: Vec<_> = committed
        .state
        .zones
        .get(draw_pile)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    let (state, result) = resolve(
        committed.state,
        owner,
        vec![third.raw(), owner.raw()],
        false,
        1,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(second), 0);
    assert_eq!(state.hand_count(third), 1);
    assert_eq!(state.cards.get(top_two[0]).unwrap().owner, owner);
    assert_eq!(state.cards.get(top_two[1]).unwrap().owner, third);
}

#[test]
fn target_choice_can_be_cancelled() {
    let (mut tx, players) = setup(2);
    let owner = players[0];
    tx.state_mut().players.get_mut(players[1]).unwrap().hp -= 1;
    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(committed.state.resolution.continuation_payload.len(), 6);
    let (state, result) = resolve(committed.state, owner, Vec::new(), true, 2);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(players[1]), 0);
}

//! Golden coverage for OL界李儒「绝策」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLJUECE: u32 = 2_978_424_418;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    eligible: PlayerId,
    ineligible: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let eligible = players[1];
    let ineligible = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: eligible,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: ineligible,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 2,
        },
        EffectRequest::DrawCards {
            player: eligible,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: ineligible,
            count: 3,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLJUECE),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    Setup {
        state: tx.commit().state,
        owner,
        eligible,
        ineligible,
    }
}

fn trigger(state: GameState, owner: PlayerId) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

fn accept_trigger(
    state: GameState,
    owner: PlayerId,
    key: u64,
) -> (GameState, engine_api::ChoiceRequest) {
    let (state, result) = resolve(state, owner, vec![1], key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, result.choice.unwrap())
}

#[test]
fn package_publishes_optional_hand_lte_trigger_at_abi338() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLJUECE)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target_other_hand_lte_owner");
    assert!(trigger.optional);
}

#[test]
fn only_other_players_with_no_more_hand_cards_are_selectable_and_damaged() {
    let setup = setup(2_026_073_102);
    assert_eq!(setup.state.hand_count(setup.owner), 2);
    assert_eq!(setup.state.hand_count(setup.eligible), 1);
    assert_eq!(setup.state.hand_count(setup.ineligible), 3);
    let (state, optional) = trigger(setup.state, setup.owner);
    assert_eq!(optional.kind, ChoiceKind::Confirm);
    let (state, choice) = accept_trigger(state, setup.owner, 1);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.eligible.raw()]
    );
    let hp_before = state.players.get(setup.eligible).unwrap().hp;
    let (state, result) = resolve(state, setup.owner, vec![setup.eligible.raw()], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.eligible).unwrap().hp, hp_before - 1);
}

#[test]
fn target_hand_growth_is_revalidated_before_damage() {
    let setup = setup(2_026_073_103);
    let (state, _) = trigger(setup.state, setup.owner);
    let (state, _) = accept_trigger(state, setup.owner, 3);
    let hp_before = state.players.get(setup.eligible).unwrap().hp;
    let mut state = state;
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let hand = state.zones.find(ZoneKind::Hand, setup.eligible).unwrap();
    for _ in 0..2 {
        let card = state.zones.get(draw).unwrap().cards[0];
        assert!(state.zones.remove_card(draw, card));
        assert!(state.zones.push_card(hand, card));
        let card_state = state.cards.get_mut(card).unwrap();
        card_state.zone = hand;
        card_state.owner = setup.eligible;
    }
    assert!(state.hand_count(setup.eligible) > state.hand_count(setup.owner));
    let (state, result) = resolve(state, setup.owner, vec![setup.eligible.raw()], 4);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(state.players.get(setup.eligible).unwrap().hp, hp_before);
    assert!(state.has_pending_choice());
}

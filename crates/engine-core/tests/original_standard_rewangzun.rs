//! Golden coverage for Standard 妄尊 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{GameState, Identity};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REWANGZUN: u32 = 3_817_957_374;
const REWANGZUN2: u32 = 2_737_456_606;
const SOURCE: &str = include_str!("../../../content/standard/rewangzun.skill");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = compile_source_named("standard-rewangzun-test", SOURCE).unwrap();
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == REWANGZUN)
            .count(),
        2
    );
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    assert_eq!(
        state
            .content
            .defs_for_skill(SkillDefId::new(REWANGZUN))
            .len(),
        2
    );
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_61, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let lord = players[0];
    let first = players[1];
    let second = players[2];
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
    effects.extend([first, second].map(|player| EffectRequest::AddSkill {
        player,
        skill_def: SkillDefId::new(REWANGZUN),
    }));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state()
            .content
            .defs_for_skill(SkillDefId::new(REWANGZUN))
            .len(),
        2
    );
    assert_eq!(
        tx.state()
            .triggers
            .iter()
            .filter(|trigger| trigger.skill_def.raw() == REWANGZUN)
            .count(),
        4
    );
    tx.state_mut().players.get_mut(lord).unwrap().identity = Identity::Lord;
    tx.state_mut().players.get_mut(lord).unwrap().hp = 4;
    for owner in [first, second] {
        tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    }
    tx.state_mut().turn.current = lord;
    (tx.commit().state, lord, first, second)
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == REWANGZUN2)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn higher_hp_lord_grants_two_draws_per_owner_and_stacks_penalty() {
    let (state, lord, first, second) = setup();
    let first_before = state.hand_count(first);
    let second_before = state.hand_count(second);
    let base_limit = hand_limit(&state, lord);
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: lord,
            phase: TurnPhaseTag::Prepare,
        },
    );
    assert_eq!(state.hand_count(first), first_before + 2);
    assert_eq!(state.hand_count(second), second_before + 2);
    assert_eq!(mark_count(&state, lord), 2);
    assert_eq!(hand_limit(&state, lord), base_limit - 2);
    assert_eq!(
        state
            .players
            .get(lord)
            .unwrap()
            .skills
            .iter()
            .filter(|skill| {
                state
                    .skills
                    .get(**skill)
                    .is_some_and(|skill| skill.def.raw() == REWANGZUN2)
            })
            .count(),
        1
    );

    let state = dispatch(
        state,
        DomainEvent::PhaseEnded {
            current: lord,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        },
    );
    assert_eq!(mark_count(&state, lord), 0);
    assert_eq!(hand_limit(&state, lord), base_limit);
}

#[test]
fn non_lord_gets_no_penalty_and_equal_hp_does_not_trigger() {
    let (mut state, target, first, second) = setup();
    state.players.get_mut(target).unwrap().identity = Identity::Rebel;
    let first_before = state.hand_count(first);
    let second_before = state.hand_count(second);
    state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Prepare,
        },
    );
    assert_eq!(state.hand_count(first), first_before + 1);
    assert_eq!(state.hand_count(second), second_before + 1);
    assert_eq!(mark_count(&state, target), 0);

    state.players.get_mut(target).unwrap().hp = 3;
    let first_before = state.hand_count(first);
    let second_before = state.hand_count(second);
    state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Prepare,
        },
    );
    assert_eq!(state.hand_count(first), first_before);
    assert_eq!(state.hand_count(second), second_before);
}

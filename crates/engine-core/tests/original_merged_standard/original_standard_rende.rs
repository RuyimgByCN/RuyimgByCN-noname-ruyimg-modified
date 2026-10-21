//! Golden coverage for standard 仁德 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RENDE: u32 = 11;
const SOURCE: &str = include_str!("../../../../content/standard/rende.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-rende-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_11, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RENDE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 3;
    participant.max_hp = 4;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == RENDE)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_publishes_abi172_repeatable_hand_gift() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == RENDE)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|target| (target.min, target.max, target.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active
            .selection
            .as_ref()
            .map(|selection| (&*selection.zone, selection.min, selection.max)),
        Some(("hand", 1, u8::MAX))
    );
    let compiled = compile_source_named("standard-rende", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn cumulative_second_card_recovers_once_after_gifts() {
    let setup = setup();
    let cards = setup.state.hand_cards(setup.owner);
    assert!(cards.len() >= 3);
    let (state, first) = activate(
        setup.state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[0]],
        1,
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&cards[0]));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_fire_count,
        1
    );

    let (mut state, second) = activate(
        state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[1]],
        2,
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&cards[1]));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_fire_count,
        2
    );

    state.players.get_mut(setup.owner).unwrap().hp = 3;
    let (state, third) = activate(
        state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[2]],
        3,
    );
    assert!(matches!(third.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_fire_count,
        3
    );
}

#[test]
fn two_cards_at_once_recover_and_forged_inputs_are_rejected() {
    let setup = setup();
    let cards = setup.state.hand_cards(setup.owner);
    let foreign = setup.state.hand_cards(setup.target)[0];
    let (unchanged, forged) = activate(
        setup.state.clone(),
        setup.owner,
        setup.skill,
        setup.target,
        vec![foreign],
        4,
    );
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, setup.state);

    let (state, accepted) = activate(
        setup.state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[0], cards[1]],
        5,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert!(cards[..2]
        .iter()
        .all(|card| state.hand_cards(setup.target).contains(card)));
}

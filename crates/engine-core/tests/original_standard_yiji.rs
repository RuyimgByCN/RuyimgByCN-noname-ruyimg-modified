//! Golden coverage for standard 遗计 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YIJI: u32 = 26;
const SOURCE: &str = include_str!("../../../content/standard/yiji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-yiji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_26, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        source,
        other,
    }
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
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

fn choice_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .expect("pending choice snapshot")
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_abi174_per_damage_point_distribution() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YIJI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);
    let compiled = compile_source_named("standard-yiji", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn two_damage_points_each_distribute_two_public_processing_cards() {
    let setup = setup();
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_before = setup.state.zones.get(draw).unwrap().cards.len();
    let owner_before = setup.state.hand_count(setup.owner);
    let source_before = setup.state.hand_count(setup.source);
    let other_before = setup.state.hand_count(setup.other);
    let (state, damage_result) = damage(setup.state, setup.source, setup.owner, 2);
    assert_eq!(damage_result, Err(EffectError::ChoiceRequested));

    let (state, first_confirm) = resolve(state, setup.owner, vec![1], 1);
    assert_eq!(
        first_confirm.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (state, second_confirm) = resolve(state, setup.owner, vec![1], 2);
    assert_eq!(
        second_confirm.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let first_pair = choice_tokens(&second_confirm);
    assert_eq!(first_pair.len(), 2);
    let processing = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(first_pair.iter().all(|raw| {
        state
            .cards
            .get(engine_api::CardId::new(*raw))
            .is_some_and(|card| card.zone == processing && card.face_up)
    }));

    let (state, first_subset) = resolve(state, setup.owner, vec![first_pair[0]], 3);
    assert_eq!(
        first_subset.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, first_recipient) = resolve(state, setup.owner, vec![setup.source.raw()], 4);
    assert_eq!(
        first_recipient.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets,
        "the lone remaining card skips a redundant subset choice"
    );
    let (state, second_recipient) = resolve(state, setup.owner, vec![setup.other.raw()], 5);
    assert_eq!(
        second_recipient.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards,
        "the second damage point opens a fresh two-card distribution"
    );
    let second_pair = choice_tokens(&second_recipient);
    assert_eq!(second_pair.len(), 2);

    let (state, second_subset) = resolve(state, setup.owner, second_pair, 6);
    assert_eq!(
        second_subset.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, completed) = resolve(state, setup.owner, vec![setup.owner.raw()], 7);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_before - 4);
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.source), source_before + 1);
    assert_eq!(state.hand_count(setup.other), other_before + 1);
}

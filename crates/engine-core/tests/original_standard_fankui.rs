//! Golden coverage for standard 反馈's once-per-damage-event card-taking contract.

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

const FANKUI: u32 = 5;
const SOURCE: &str = include_str!("../../../content/standard/fankui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("standard-fankui-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = old_skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANKUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
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
            nature: 0,
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
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_and_two_damage_offer_exactly_one_card_take() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FANKUI
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_alive_gainable_card"
            && trigger.optional
    }));

    let (state, owner, source) = setup();
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let (state, result) = damage(state, source, owner, 2);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, accepted) = resolve(state, owner, vec![1], 1);
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let card = accepted.choice.as_ref().unwrap().options[0].token;
    let (state, taken) = resolve(state, owner, vec![card], 2);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(source), source_hand - 1);
}

#[test]
fn self_damage_requires_equipment_and_never_exposes_hand() {
    let (state, owner, _) = setup();
    let (state, result) = damage(state, owner, owner, 1);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let card = state.hand_cards(owner)[0];
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let (state, result) = damage(tx.commit().state, owner, owner, 1);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, owner, vec![1], 3);
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, vec![card.raw()]);
    let (_, taken) = resolve(state, owner, vec![card.raw()], 4);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
}

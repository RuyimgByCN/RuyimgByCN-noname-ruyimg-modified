//! Golden for REFRESH 张飞「替身」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLTISHEN: u32 = 3_850_927_603;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn isolated(hp: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(OLTISHEN as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: OLTISHEN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLTISHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_oltishen(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == OLTISHEN)
}

#[test]
fn committed_package_exposes_optional_limited_prepare_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLTISHEN)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        ("phase_prepare", "owner_is_target_wounded", true)
    );
}

#[test]
fn accepting_recovers_to_max_draws_lost_hp_and_consumes_the_skill() {
    let (mut tx, owner) = isolated(2);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let hand = tx.state().hand_count(owner);
    assert_eq!(offer(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&tx, owner, true, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp as i32);
    assert_eq!(state.hand_count(owner), hand + max_hp - 2);
    assert!(!has_oltishen(&state, owner));
}

#[test]
fn declining_preserves_state_and_healthy_owner_is_not_offered() {
    let (mut wounded, owner) = isolated(2);
    let hp = wounded.state().players.get(owner).unwrap().hp;
    let hand = wounded.state().hand_count(owner);
    assert_eq!(
        offer(&mut wounded, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&wounded, owner, false, 2);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(owner), hand);
    assert!(has_oltishen(&state, owner));

    let (mut healthy, owner) = isolated(4);
    let max_hp = healthy.state().players.get(owner).unwrap().max_hp as i32;
    healthy.state_mut().players.get_mut(owner).unwrap().hp = max_hp;
    assert_eq!(offer(&mut healthy, owner), Ok(Vec::new()));
    assert!(!healthy.state().has_pending_choice());
    assert!(has_oltishen(healthy.state(), owner));
}

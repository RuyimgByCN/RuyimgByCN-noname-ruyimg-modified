//! Golden for REFRESH 郭淮「精忠」rewritten from the legacy JavaScript semantics.

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

const JIEZHONG: u32 = 2_811_249_808;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn isolated(max_hp: u32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JIEZHONG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: JIEZHONG as u64,
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
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIEZHONG),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = max_hp;
    player.hp = player.hp.min(max_hp as i32);
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
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

fn has_jiezhong(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == JIEZHONG)
}

#[test]
fn committed_package_exposes_optional_limited_play_phase_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIEZHONG)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        ("phase_play", "owner_is_target_hand_below_max_hp", true)
    );
}

#[test]
fn accepting_draws_to_max_hp_and_consumes_the_skill() {
    let (mut tx, owner) = isolated(3);
    assert_eq!(offer(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&tx, owner, true, 1);
    assert_eq!(state.hand_count(owner), 3);
    assert!(!has_jiezhong(&state, owner));
}

#[test]
fn draw_is_capped_at_five_and_declining_does_not_consume() {
    let (mut capped, owner) = isolated(8);
    assert_eq!(offer(&mut capped, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&capped, owner, true, 2);
    assert_eq!(state.hand_count(owner), 5);
    assert!(!has_jiezhong(&state, owner));

    let (mut declined, owner) = isolated(4);
    assert_eq!(
        offer(&mut declined, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&declined, owner, false, 3);
    assert_eq!(state.hand_count(owner), 0);
    assert!(has_jiezhong(&state, owner));
}

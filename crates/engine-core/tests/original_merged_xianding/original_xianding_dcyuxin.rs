//! Golden for XIANDING「御心」的目标、非负体力与限定技语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DCYUXIN: u32 = 4_174_094_056;
const DCKUNLI: u32 = 4_133_766_049;
const DCKUICHI: u32 = 3_373_286_891;
const DCSHANGJUE: u32 = 2_632_217_594;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn isolated_state(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == skill_def));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn offer(tx: &mut Transaction, dying: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
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

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn yuxin_recovers_self_to_one_and_other_to_owner_hp() {
    let (mut self_dying, owner, _) = isolated_state(DCYUXIN);
    self_dying.state_mut().players.get_mut(owner).unwrap().hp = -2;
    assert_eq!(
        offer(&mut self_dying, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let self_saved = resolve(&self_dying, owner, true, 1);
    assert_eq!(self_saved.players.get(owner).unwrap().hp, 1);
    assert!(!has_skill(&self_saved, owner, DCYUXIN));

    let (mut other_dying, owner, other) = isolated_state(DCYUXIN);
    other_dying.state_mut().players.get_mut(owner).unwrap().hp = 3;
    other_dying.state_mut().players.get_mut(other).unwrap().hp = -1;
    assert_eq!(
        offer(&mut other_dying, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let other_saved = resolve(&other_dying, owner, true, 2);
    assert_eq!(other_saved.players.get(other).unwrap().hp, 3);
    assert!(!has_skill(&other_saved, owner, DCYUXIN));
}

#[test]
fn declined_and_non_improving_other_target_do_not_consume_yuxin() {
    let (mut tx, owner, other) = isolated_state(DCYUXIN);
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 3);
    assert_eq!(declined.players.get(other).unwrap().hp, 0);
    assert!(has_skill(&declined, owner, DCYUXIN));

    let mut boundary = Transaction::begin(declined);
    boundary.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert!(offer(&mut boundary, other).unwrap().is_empty());
    assert!(!boundary.state().has_pending_choice());
}

#[test]
fn nonnegative_owner_hp_matches_legacy_get_hp_clamping() {
    let (mut tx, owner, other) = isolated_state(DCYUXIN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = -2;
    tx.state_mut().players.get_mut(other).unwrap().hp = -1;
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 4);
    assert_eq!(accepted.players.get(other).unwrap().hp, 0);
    assert!(!has_skill(&accepted, owner, DCYUXIN));
}

#[test]
fn kunli_recovers_before_gaining_max_hp_and_removes_kuichi() {
    let (mut tx, owner, _) = isolated_state(DCKUNLI);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCKUICHI),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = offer(&mut tx, owner).unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
    assert!(!has_skill(tx.state(), owner, DCKUNLI));
    assert!(!has_skill(tx.state(), owner, DCKUICHI));

    let (mut capped, owner, _) = isolated_state(DCKUNLI);
    let player = capped.state_mut().players.get_mut(owner).unwrap();
    player.hp = 0;
    player.max_hp = 1;
    let effects = offer(&mut capped, owner).unwrap();
    run_effects(&mut capped, effects).unwrap();
    assert_eq!(capped.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(capped.state().players.get(owner).unwrap().max_hp, 2);
}

#[test]
fn shangjue_recovers_then_gains_max_hp_grants_kunli_and_records_upgrade() {
    let (mut tx, owner, _) = isolated_state(DCSHANGJUE);
    tx.state_mut().players.get_mut(owner).unwrap().hp = -1;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = offer(&mut tx, owner).unwrap();
    run_effects(&mut tx, effects).unwrap();

    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(player.hp, 1);
    assert_eq!(player.max_hp, max_hp + 1);
    assert_eq!(
        player
            .marks
            .iter()
            .find(|(mark, _)| *mark == DCSHANGJUE)
            .map_or(0, |(_, count)| *count),
        1
    );
    assert!(has_skill(tx.state(), owner, DCKUNLI));
    assert!(!has_skill(tx.state(), owner, DCSHANGJUE));
}

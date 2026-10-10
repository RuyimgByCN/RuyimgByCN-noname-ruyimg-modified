//! Golden for XIANDING（限定包）批次十五：
//! 连枝（dclianzhi，②）、杜君（dcdujun，②）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCLIANZHI: u32 = 2_871_252_462;
const DCDUJUN: u32 = 3_784_866_267;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch15_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let lianzhi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCLIANZHI)
        .expect("dclianzhi trigger");
    assert_eq!(lianzhi.event, "dying");
    assert!(lianzhi.optional);

    let dujun = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUJUN)
        .expect("dcdujun trigger");
    assert_eq!(dujun.event, "damage");
    assert_eq!(
        dujun.filter,
        "owner_source_or_target_first_damage_this_turn"
    );
    assert!(dujun.optional);
}

#[test]
fn dclianzhi_recovers_and_draws_when_owner_enters_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_834);
    add_skill(&mut tx, owner, DCLIANZHI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn dcdujun_draws_two_after_first_damage_of_turn() {
    let (mut tx, owner, first, _second) = setup(20_260_835);
    add_skill(&mut tx, owner, DCDUJUN);
    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn dcdujun_only_triggers_on_first_damage_of_turn() {
    let (mut tx, owner, first, _second) = setup(20_260_836);
    add_skill(&mut tx, owner, DCDUJUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let mut tx = Transaction::begin(state);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Ok(_)), "second damage must not fire");
}

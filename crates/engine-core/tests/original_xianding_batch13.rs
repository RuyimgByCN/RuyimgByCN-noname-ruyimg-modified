//! Golden for XIANDING（限定包）批次十三：
//! 金玠（dcjinjie，杨彪）。

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

const DCJINJIE: u32 = 2_966_946_872;
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
fn package_exposes_batch13_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJINJIE)
        .expect("dcjinjie trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive_once_per_round");
    assert!(trigger.optional);
}

#[test]
fn dcjinjie_draws_declared_count_then_discards_same_count_to_recover() {
    let (mut tx, owner, first, _second) = setup(20_260_830);
    add_skill(&mut tx, owner, DCJINJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(first).unwrap().hp;
    let hand_before = tx.state().hand_count(first);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
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
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume
    );
    let state = resolve(state, owner, vec![2], false, 2);
    assert_eq!(state.hand_count(first), hand_before + 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill
    );
    let payment: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .take(2)
        .map(|card| card.raw())
        .collect();
    let state = resolve(state, owner, payment, false, 3);
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp_before + 1);
}

#[test]
fn dcjinjie_zero_branch_can_recover_owner_without_draw_or_discard() {
    let (mut tx, owner, _first, _second) = setup(20_260_831);
    add_skill(&mut tx, owner, DCJINJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![0], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrRecover
    );
    let state = resolve(state, owner, vec![2], false, 3);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
}

#[test]
fn dcjinjie_may_cancel_equal_discard_without_recovery() {
    let (mut tx, owner, first, _second) = setup(20_260_832);
    add_skill(&mut tx, owner, DCJINJIE);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 2,
            },
            EffectRequest::DealDamage {
                source: owner,
                target: first,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            },
        ],
    )
    .unwrap();
    let hp_before = tx.state().players.get(first).unwrap().hp;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![2], false, 2);
    let state = resolve(state, owner, Vec::new(), true, 3);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp_before);
}

#[test]
fn dcjinjie_respects_once_per_round_limit() {
    let (mut tx, owner, first, _second) = setup(20_260_833);
    add_skill(&mut tx, owner, DCJINJIE);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![0], false, 2);
    let state = resolve(state, owner, Vec::new(), true, 3);

    let mut tx = Transaction::begin(state);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(dispatched, Ok(_)),
        "second dying in same round must not fire"
    );
}

//! Golden for XIANDING（限定包）批次十八：
//! 忿肆（fensi）、典财（diancai）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FENSI: u32 = 4_137_633_788;
const DIANCAI: u32 = 3_029_901_387;
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
fn package_exposes_batch18_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let fensi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FENSI)
        .expect("fensi trigger");
    assert_eq!(fensi.event, "phase_begin");
    assert!(fensi.optional);

    let diancai = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DIANCAI)
        .expect("diancai trigger");
    assert_eq!(diancai.event, "phase_end");
    assert_eq!(diancai.filter, "owner_alive_lost_hand_this_turn");
    assert!(diancai.optional);
}

#[test]
fn fensi_damages_other_player_at_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_842);
    add_skill(&mut tx, owner, FENSI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
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
        ContinuationKind::ChooseOtherDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}

#[test]
fn diancai_draws_to_max_hp_after_other_phase_end_with_hand_loss() {
    let (mut tx, owner, first, _second) = setup(20_260_843);
    add_skill(&mut tx, owner, DIANCAI);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::Play;

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    );
    assert!(err.is_ok());

    tx.state_mut().turn.phase = Phase::End;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![owner.raw()], false, 2);
    let hand_after = state.hand_count(owner);
    assert!(hand_after >= hand_before);
    assert!(
        hand_after <= max_hp as u32,
        "hand {} should be capped at max hp {}",
        hand_after,
        max_hp
    );
}

#[test]
fn diancai_does_not_trigger_on_own_phase_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_844);
    add_skill(&mut tx, owner, DIANCAI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)), "own phase end must not fire");
}

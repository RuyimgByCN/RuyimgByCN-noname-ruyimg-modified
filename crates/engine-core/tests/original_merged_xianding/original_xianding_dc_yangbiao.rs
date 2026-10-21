//! Golden for XIANDING 杨彪【举讹】（dcjue，ABI 583）。

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJUE: u32 = 2_849_188_079;
const YANGBIAO: u32 = 3_963_716_631;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCJUE),
        }],
    )
    .unwrap();
    tx.state_mut().turn.round = 1;
    tx.state_mut().revision = 1;
    (tx, owner, first, second)
}

fn add_hand_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_001),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn discard(tx: &mut Transaction, player: PlayerId, card: CardId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let pile = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![card],
            to: pile,
            from: Some(hand),
            reason: CardMoveReason::Discard,
        }],
    )
    .unwrap();
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(1),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

#[test]
fn package_closes_dc_yangbiao_with_exact_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YANGBIAO)
        .expect("dc_yangbiao");
    assert_eq!(character.name_key, "dc_yangbiao");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [2_818_087_075, 2_966_946_872, DCJUE]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJUE)
        .expect("dcjue trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_global_phase_end_discarded_cards_available"
    );
    assert!(!trigger.optional);
}

#[test]
fn dcjue_tracks_only_discard_reason_cards_that_remain_in_discard() {
    let (mut tx, owner, first, _) = setup(20_260_803);
    let discarded = add_hand_card(&mut tx, owner, CardKind::Sha);
    let used = add_hand_card(&mut tx, owner, CardKind::Tao);
    discard(&mut tx, owner, discarded);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let pile = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![used],
            to: pile,
            from: Some(hand),
            reason: CardMoveReason::UseOrRespond,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state()
            .turn
            .discarded_cards
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [discarded]
    );

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![discarded],
            to: hand,
            from: Some(pile),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    let result = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(result, Ok(events) if events.is_empty()));
}

#[test]
fn dcjue_confirms_fixed_target_and_starts_authoritative_random_loop_once_per_round() {
    let (mut tx, owner, first, _) = setup(20_260_804);
    let card = add_hand_card(&mut tx, owner, CardKind::Sha);
    discard(&mut tx, owner, card);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    let calls = tx.state().rng.call_count;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, effects);
    assert!(
        matches!(dispatched, Err(EffectError::ChoiceRequested)),
        "dispatch={dispatched:?}, discarded={:?}",
        tx.state().turn.discarded_cards
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );

    let state = resolve(tx.state().clone(), owner, vec![1], 1);
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCJUE)
        .unwrap();
    assert_eq!(skill.round_fire_round, state.turn.round);
    assert_eq!(
        state.rng.call_count,
        calls + 2,
        "three-way shuffle uses two draws"
    );
}

#[test]
fn dcjue_own_end_phase_uses_single_other_target_choice() {
    let (mut tx, owner, first, _) = setup(20_260_805);
    let card = add_hand_card(&mut tx, owner, CardKind::Sha);
    discard(&mut tx, owner, card);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    let state = resolve(tx.state().clone(), owner, vec![first.raw()], 2);
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCJUE)
        .unwrap();
    assert_eq!(skill.round_fire_round, state.turn.round);
}

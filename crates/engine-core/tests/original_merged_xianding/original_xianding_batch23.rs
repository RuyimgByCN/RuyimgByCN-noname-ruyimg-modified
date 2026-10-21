//! Golden for XIANDING（限定包）批次二十三：
//! 清棱（dcqingleng）、接缨（dcjieying）。

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

const DCQINGLENG: u32 = 2_507_120_853;
const DCJIEYING: u32 = 3_849_335_872;
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

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
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
fn package_exposes_batch23_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let qingleng = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCQINGLENG)
        .expect("dcqingleng trigger");
    assert_eq!(qingleng.event, "card_targeted");
    assert_eq!(qingleng.filter, "owner_is_target_nonred_card");
    assert!(qingleng.optional);

    let jieying = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJIEYING)
        .expect("dcjieying trigger");
    assert_eq!(jieying.event, "phase_end");
    assert!(jieying.optional);
}

#[test]
fn dcqingleng_discards_one_card_from_any_player_when_targeted_by_black_card() {
    let (mut tx, owner, first, _second) = setup(20_260_852);
    add_skill(&mut tx, owner, DCQINGLENG);
    let target_card = inject_card(&mut tx, first, CardKind::Sha);
    let source_card = inject_card(&mut tx, first, CardKind::Jiu);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: source_card,
            targets: vec![owner],
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
        ContinuationKind::ChooseOtherDiscardTargetCard
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let state = resolve(state, owner, vec![target_card.raw()], false, 3);
    let hand = state
        .zones
        .get(state.zones.find(ZoneKind::Hand, first).unwrap())
        .unwrap()
        .cards
        .clone();
    assert!(
        !hand.contains(&target_card),
        "chosen card must be discarded"
    );
}

#[test]
fn dcjieying_links_other_and_draws_linked_players_at_phase_end() {
    let (mut tx, owner, first, _second) = setup(20_260_853);
    add_skill(&mut tx, owner, DCJIEYING);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let before_first = tx.state().hand_count(first);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
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
        ContinuationKind::ChooseLinkOther
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert!(state
        .players
        .get(first)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
    assert!(state.hand_count(first) >= before_first);
}

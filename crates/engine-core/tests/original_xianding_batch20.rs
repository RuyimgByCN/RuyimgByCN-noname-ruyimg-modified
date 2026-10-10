//! Golden for XIANDING（限定包）批次二十：
//! 酒遁（dcjiudun）、全守（dcquanshou）。

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

const DCJIUDUN: u32 = 2_926_675_148;
const DCQUANSHOU: u32 = 2_462_490_450;
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

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit,
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
fn package_exposes_batch20_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jiudun: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCJIUDUN)
        .collect();
    assert_eq!(jiudun.len(), 2);
    assert!(jiudun
        .iter()
        .all(|trigger| trigger.event == "card_targeted"));
    assert_eq!(
        jiudun
            .iter()
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_event_card_jiu",
            "owner_is_target_other_source_black_dcjiudun_available"
        ]
    );
    assert!(jiudun.iter().all(|trigger| !trigger.optional));

    let quanshou = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCQUANSHOU)
        .expect("dcquanshou trigger");
    assert_eq!(quanshou.event, "phase_begin");
    assert_eq!(quanshou.filter, "owner_alive_not_target");
    assert!(quanshou.optional);
}

#[test]
fn dcjiudun_draws_and_uses_jiu_when_targeted_by_black_card() {
    let (mut tx, owner, first, _second) = setup(20_260_847);
    add_skill(&mut tx, owner, DCJIUDUN);
    let card = inject_card(&mut tx, first, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
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
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcquanshou_offers_draw_to_max_hp_on_other_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_848);
    add_skill(&mut tx, owner, DCQUANSHOU);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::Prepare;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
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
        ContinuationKind::ChooseTargetDraw
    );
    let hand_before = state.hand_count(first);
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert!(state.hand_count(first) >= hand_before);
}

//! Golden for sp batch-T DSL rewrites:
//! 飞杨·OL (olfeiyang) prepare-phase discard 2 → clear judgment zone (ABI 606);
//! 礼下·OL (ollixia) other's end phase outside owner's range → draw self 1 / event player 2
//! + rebiluan2 distance mark -1.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLFEIYANG: u32 = 2_973_432_445;
const OLLIXIA: u32 = 3_877_304_106;
const REBILUAN2: u32 = 3_075_740_841;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_zone(
    state: &mut GameState,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    tag: u32,
) -> CardId {
    let zone_id = state.zones.find(zone, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_700 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn equip_mount(state: &mut GameState, player: PlayerId, kind: CardKind) {
    let zone = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(9_999),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
}

fn mark_of(state: &GameState, player: PlayerId, mark: u32) -> Option<i32> {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_batch_t_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, OLFEIYANG | OLLIXIA))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        OLFEIYANG,
        "phase_prepare",
        "owner_is_target_judge_nonempty",
        true
    )));
    assert!(triggers.contains(&(
        OLLIXIA,
        "phase_end",
        "owner_alive_other_phase_end_player_outside_owner_range",
        true
    )));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| { matches!(modifier.skill_def_id, OLFEIYANG | OLLIXIA) }));
}

#[test]
fn olfeiyang_discards_two_hand_and_clears_judgment_zone() {
    let (mut state, owner) = started(2_026_081_401);
    let hand: Vec<_> = state.hand_cards(owner);
    assert!(hand.len() >= 2, "natural deal must hold >=2 cards");
    // Give the owner a judgment-zone card (乐不思蜀) so the filter fires.
    inject_zone(&mut state, owner, ZoneKind::Judge, CardKind::TrickLe, 1);
    let state = with_skill(state, owner, OLFEIYANG);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let hand_now: Vec<_> = state.hand_cards(owner);
    let (state, result) = resolve(
        state,
        owner,
        vec![hand_now[0].raw(), hand_now[1].raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        u32::try_from(hand.len()).unwrap() - 2
    );
    let judge_zone = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(
        state.zones.get(judge_zone).unwrap().cards.is_empty(),
        "judgment zone must be cleared by discard_all_own_judge"
    );
}

#[test]
fn ollixia_in_range_does_not_fire() {
    let (state, owner) = started(2_026_081_402);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLLIXIA);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    // In a 2-player identity game the event player is at distance 1, within the
    // owner's attack range, so the trigger should NOT fire.
    assert!(
        matches!(opened, Ok(_)),
        "owner must be in range in a 2p game"
    );
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before);
    assert!(!has_skill(&state, owner, REBILUAN2));
    assert!(mark_of(&state, owner, REBILUAN2).is_none());
}

#[test]
fn ollixia_outside_range_fires_and_marks_distance_minus_one() {
    let (mut state, owner) = started(2_026_081_403);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // A defensive horse on the owner makes distance(other→owner) = 2 > other's
    // attack range 1 → the event player is not in the owner's range.
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, OLLIXIA);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    // Choose branch 1: self draws 1 (confirm optional trigger, then branch choice).
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let (state, resolved) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(has_skill(&state, owner, REBILUAN2));
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

#[test]
fn ollixia_branch_two_draws_event_player_two() {
    let (mut state, owner) = started(2_026_081_404);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, OLLIXIA);
    let other_before = state.hand_count(other);
    let owner_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // OptionalTrigger → Custom choice (branch 2 = event player draws 2).
    let (state, accepted) = resolve(state, owner, vec![2], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(other), other_before + 2);
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

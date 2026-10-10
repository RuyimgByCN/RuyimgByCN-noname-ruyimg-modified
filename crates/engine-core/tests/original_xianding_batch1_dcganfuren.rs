//! Golden for XIANDING（限定包）DC 甘夫人（dc_ganfuren）与批次技能：
//! 淑慎（dcshushen）、神智（dcshenzhi）、诋敌（dcjidi）、制略（dczhijue）、
//! 闺秀（dcguixiu）、昭汉（dczhaohan）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const DC_GANFUREN: u32 = 3_184_320_853;
const DCSHUSHEN: u32 = 2_888_857_366;
const DCSHENZHI: u32 = 2_981_440_225;
const DCJIDI: u32 = 4_136_170_419;
const DCZHIJUE: u32 = 2_161_260_760;
const DCGUIXIU: u32 = 2_370_422_642;
const DCZHAOHAN: u32 = 2_818_087_075;
const DCZHAOHAN_CHOOSE: u32 = 3_308_549_699;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    let mut tx = Transaction::begin(state);
    // Bootstrap with the embedded standard roster first: the committed
    // xianding roster carries game_started choice skills (e.g. dcmingjie),
    // which would make bootstrap itself request a choice. The package loads
    // afterwards; setup removes every booted skill right below.
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Prepare;
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 3;
        participant.max_hp = 3;
    }
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

fn phase_event(current: PlayerId, phase: TurnPhaseTag) -> DomainEvent {
    DomainEvent::TurnStateChanged { current, phase }
}

fn phase_ended_event(current: PlayerId, phase: TurnPhaseTag) -> DomainEvent {
    DomainEvent::PhaseEnded {
        current,
        phase,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
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

fn dispatch_and_apply(tx: &mut Transaction, event: DomainEvent) {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
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

fn damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
}

#[test]
fn package_closes_dc_ganfuren_with_exact_skills() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_GANFUREN)
        .expect("dc_ganfuren character index entry");
    assert_eq!(character.name_key, "dc_ganfuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [DCSHUSHEN, DCSHENZHI]);

    let shushen = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHUSHEN)
        .expect("dcshushen trigger");
    assert_eq!(shushen.event, "hp_recovered");
    assert_eq!(shushen.filter, "owner_is_target");
    assert!(shushen.optional);

    let shenzhi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHENZHI)
        .expect("dcshenzhi trigger");
    assert_eq!(shenzhi.event, "phase_prepare");
    assert_eq!(shenzhi.filter, "owner_is_target_hand_above_hp");
    assert!(shenzhi.optional);

    let jidi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJIDI)
        .expect("dcjidi trigger");
    assert_eq!(jidi.event, "damage_before");
    assert_eq!(jidi.filter, "owner_is_target_source_alive");
    assert!(!jidi.optional);
}

#[test]
fn dcshushen_offers_other_player_draw_or_recover_choice_per_recover_point() {
    let (mut tx, owner, first, second) = setup();
    add_skill(&mut tx, owner, DCSHUSHEN);
    damage(&mut tx, first, owner, 1).unwrap();
    damage(&mut tx, owner, first, 1).unwrap();
    let before_hand = tx.state().hand_count(first);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    let state = resolve(state, first, vec![1], false, 3);
    assert_eq!(state.hand_count(first), before_hand + 2);
    assert_eq!(state.hand_count(second), 0);
}

#[test]
fn dcshenzhi_discards_one_hand_then_recovers_when_hand_above_hp() {
    let (mut tx, owner, _first, _second) = setup();
    add_skill(&mut tx, owner, DCSHENZHI);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 4,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);
    assert!(hand_before > hp_before as u32);

    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Prepare)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert_eq!(state.hand_count(owner), hand_before - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
}

#[test]
fn dcjidi_source_with_more_hp_loses_one_and_more_hand_discards_two() {
    let (mut tx, owner, first, _second) = setup();
    add_skill(&mut tx, owner, DCJIDI);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let owner_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: first,
            count: owner_hand + 2,
        }],
    )
    .unwrap();
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let first_hand = tx.state().hand_count(first);
    damage(&mut tx, first, owner, 1).unwrap();
    assert_eq!(tx.state().players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(tx.state().hand_count(first), first_hand - 2);
}

#[test]
fn dczhijue_shortens_distance_from_owner_by_two_and_draws_to_max_hp_on_damage() {
    let (mut tx, owner, first, _second) = setup();
    add_skill(&mut tx, owner, DCZHIJUE);

    let distance = match query(
        tx.state(),
        &QueryRequest::Distance {
            from: owner,
            to: first,
        },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("distance query failed: {other:?}"),
    };
    assert_eq!(distance, 1);

    let hand_before = tx.state().hand_count(owner);
    damage(&mut tx, owner, first, 1).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    assert!(tx.state().hand_count(owner) > hand_before);
    assert_eq!(tx.state().hand_count(owner), max_hp);
}

#[test]
fn dcguixiu_draws_two_at_phase_begin_once() {
    let (mut tx, owner, _first, _second) = setup();
    add_skill(&mut tx, owner, DCGUIXIU);
    let before = tx.state().hand_count(owner);
    dispatch_and_apply(&mut tx, phase_event(owner, TurnPhaseTag::Prepare));
    assert_eq!(tx.state().hand_count(owner), before + 2);
    let after_first = tx.state().hand_count(owner);
    dispatch_and_apply(&mut tx, phase_event(owner, TurnPhaseTag::Prepare));
    assert_eq!(tx.state().hand_count(owner), after_first);
}

#[test]
fn dczhaohan_increases_current_draw_count_by_two() {
    let (mut tx, owner, _first, _second) = setup();
    add_skill(&mut tx, owner, DCZHAOHAN);
    tx.state_mut().turn.phase = engine_core::state::Phase::Draw;
    let before = tx.state().hand_count(owner);
    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 4);
    assert_eq!(
        state
            .turn
            .flag(engine_core::state::turn_flag::CURRENT_DRAW_COUNT_BONUS),
        2
    );
    assert_eq!(state.hand_count(owner), before);
    assert!(has_skill(&state, owner, DCZHAOHAN_CHOOSE));
}

fn open_dczhaohan_draw_end(mut state: GameState, owner: PlayerId) -> GameState {
    state.turn.phase = engine_core::state::Phase::Play;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_ended_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    ));
    tx.state().clone()
}

#[test]
fn dczhaohan_gives_exactly_two_hand_cards_to_a_hand_empty_player() {
    let (mut tx, owner, first, _second) = setup();
    add_skill(&mut tx, owner, DCZHAOHAN);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 4,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Draw;
    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 10);
    let state = open_dczhaohan_draw_end(state, owner);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DrawThenChooseGiveOrDiscardHand
    );
    let state = resolve(state, owner, vec![1], false, 11);
    let state = resolve(state, owner, vec![first.raw()], false, 12);
    let cards: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .take(2)
        .map(CardId::raw)
        .collect();
    let state = resolve(state, owner, cards, false, 13);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(first), 2);
}

#[test]
fn dczhaohan_forces_two_discards_when_no_player_is_hand_empty() {
    let (mut tx, owner, first, second) = setup();
    add_skill(&mut tx, owner, DCZHAOHAN);
    for (player, count) in [(owner, 4), (first, 1), (second, 1)] {
        run_effects(&mut tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    }
    tx.state_mut().turn.phase = engine_core::state::Phase::Draw;
    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 20);
    let state = open_dczhaohan_draw_end(state, owner);
    let cards: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .take(2)
        .map(CardId::raw)
        .collect();
    let state = resolve(state, owner, cards, false, 21);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(first), 1);
    assert_eq!(state.hand_count(second), 1);
}

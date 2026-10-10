//! Golden coverage for 巧变（shenhua 张郃，ABI 559）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QIAOBIAN: u32 = 3_287_917_228;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhanghe.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhanghe-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QIAOBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

/// Dispatch a phase-begin event and return the resolved state; the composite's
/// first choice (discard) is expected to open, or the dispatch may complete.
fn dispatch_phase(state: GameState, actor: PlayerId, tag: engine_api::TurnPhaseTag) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: actor,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// Accept the optional trigger confirm, then answer the discard choice.
fn accept_and_pay(state: GameState, owner: PlayerId, card: Option<CardId>) -> GameState {
    // Optional trigger confirm: options [1].
    let state = resolve(state, owner, vec![1], false, 9_000);
    assert!(
        state.has_pending_choice(),
        "巧变 discard prompt after accept"
    );
    let selections = card.map(|card| vec![card.raw()]).unwrap_or_default();
    resolve(state, owner, selections, card.is_none(), 9_001)
}

#[test]
fn package_exposes_qiaobian_contract() {
    let compiled = content_compiler::compile_source_named("zhanghe", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 559);
    assert_eq!(compiled.skill_index.len(), 1);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == QIAOBIAN)
        .collect();
    assert_eq!(triggers.len(), 4);
    for trigger in triggers {
        assert!(trigger.optional);
    }
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == QIAOBIAN)
        .map(|t| t.event.as_str())
        .collect();
    for event in ["phase_judge", "phase_draw", "phase_play", "phase_discard"] {
        assert!(events.contains(&event), "missing trigger {event}");
    }
}

#[test]
fn qiaobian_judge_discards_and_skips_judge_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Judge;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_500,
        Suit::Spade,
        2,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Judge);
    assert!(state.has_pending_choice(), "巧变 judge discard prompt");
    let state = accept_and_pay(state, setup.owner, Some(card));
    // The judge phase was skipped: the turn advanced to Draw, whose 巧变
    // prompt now opens (cancelled below).
    assert_eq!(state.turn.phase, Phase::Draw, "judge phase skipped");
    assert!(
        !state.hand_cards(setup.owner).contains(&card),
        "card discarded"
    );
    assert!(
        state.has_pending_choice(),
        "draw-phase 巧变 prompt after skip"
    );
    let state = resolve(state, setup.owner, Vec::new(), true, 9_002);
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_draw_takes_one_card_from_each_of_two_targets() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Draw;
    let owner_hand = hand_zone(&tx, setup.owner);
    let other_hand = hand_zone(&tx, setup.other);
    let far_hand = hand_zone(&tx, setup.far);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_501,
        Suit::Spade,
        3,
    );
    let o1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_502,
        Suit::Heart,
        2,
    );
    let o2 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_503,
        Suit::Heart,
        3,
    );
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Sha,
        96_504,
        Suit::Club,
        4,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Draw);
    let state = accept_and_pay(state, setup.owner, Some(card));
    assert!(state.has_pending_choice(), "draw 巧变 target prompt");
    // Choose both targets: each gives one card.
    let state = resolve(
        state,
        setup.owner,
        vec![setup.other.raw(), setup.far.raw()],
        false,
        9_010,
    );
    assert!(state.has_pending_choice(), "first target's give prompt");
    let state = resolve(state, setup.other, vec![o2.raw()], false, 9_011);
    assert!(state.has_pending_choice(), "second target's give prompt");
    let state = resolve(state, setup.far, vec![f1.raw()], false, 9_012);
    // The draw phase was skipped: the play-phase 巧变 prompt now opens.
    assert_eq!(state.turn.phase, Phase::Play, "draw phase skipped");
    assert!(
        state.hand_cards(setup.owner).contains(&o2),
        "owner gained o2"
    );
    assert!(
        state.hand_cards(setup.owner).contains(&f1),
        "owner gained f1"
    );
    assert!(!state.hand_cards(setup.other).contains(&o2));
    assert!(!state.hand_cards(setup.far).contains(&f1));
    let _ = (o1, o2, f1);
    let state = resolve(state, setup.owner, Vec::new(), true, 9_013);
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_play_moves_field_equip_to_another_player() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_505,
        Suit::Spade,
        5,
    );
    // A weapon on the field owned by `other`.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, setup.other).unwrap();
    let weapon = alloc_card(
        &mut tx,
        setup.other,
        equip_zone,
        CardKind::Weapon,
        96_506,
        Suit::Diamond,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Play);
    let state = accept_and_pay(state, setup.owner, Some(card));
    assert!(state.has_pending_choice(), "play 巧变 field-card prompt");
    let state = resolve(state, setup.owner, vec![weapon.raw()], false, 9_020);
    assert!(state.has_pending_choice(), "destination prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_021);
    // The play phase was skipped: the discard-phase 巧变 prompt opens.
    assert_eq!(state.turn.phase, Phase::Discard, "play phase skipped");
    let far_equip = state.zones.find(ZoneKind::Equip, setup.far).unwrap();
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        far_equip,
        "weapon moved"
    );
    let other_equip = state.zones.find(ZoneKind::Equip, setup.other).unwrap();
    assert!(!state
        .zones
        .get(other_equip)
        .unwrap()
        .cards
        .contains(&weapon));
    let state = resolve(state, setup.owner, Vec::new(), true, 9_022);
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_discard_skips_discard_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Discard;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_507,
        Suit::Spade,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Discard);
    assert!(state.has_pending_choice(), "巧变 discard prompt");
    let state = accept_and_pay(state, setup.owner, Some(card));
    assert_eq!(state.turn.phase, Phase::End, "discard phase skipped");
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_cancel_keeps_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Judge;
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Judge);
    assert!(state.has_pending_choice());
    // Cancel the optional trigger confirm: the composite never runs.
    let state = resolve(state, setup.owner, Vec::new(), true, 9_030);
    // Cancelled: the judge phase proceeds normally.
    assert_eq!(state.turn.phase, Phase::Judge);
    assert!(!state.has_pending_choice());
}

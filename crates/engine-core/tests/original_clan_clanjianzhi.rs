//! Golden coverage for clan 谏直 (clanjianzhi): locked clan skill that, at the end of
//! the owner's turn, counts 惬诣 (clanqieyi) fires this turn via the cross-skill state
//! query (turn_fire_count of the 惬诣 instance), offers up to that many raw judgments
//! (top-pile reveals), assigns matching-suit judge cards to chosen players, and deals
//! 1 thunder damage when any judge card's suit was not used this play phase.
//!
//! Play-phase suit mask: Spade=1, Heart=2, Club=4, Diamond=8 (see turn.rs).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANJIANZHI: u32 = 2_209_164_637;
/// 惬诣 — 谏直 counts its fires this turn.
const CLANQIEYI: u32 = 3_347_941_832;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_813,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_skills(
    tx: &mut Transaction,
    owner: PlayerId,
    defs: &[u32],
) -> Vec<engine_api::SkillId> {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    for def in defs {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*def),
        });
    }
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .filter(|skill| skill.owner == owner && defs.contains(&skill.def.raw()))
        .map(|skill| skill.id)
        .collect()
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_pile_top(
    state: &mut engine_core::state::GameState,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn dispatch_phase_end(tx: &mut Transaction, owner: PlayerId) {
    let fired = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("phase_end dispatch error: {e:?}"),
    }
}

fn resolve_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    idempotency_key: IdempotencyKey,
    selections: Vec<u32>,
) -> (engine_core::state::GameState, CommandResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key,
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    (state, pipeline.result)
}

#[test]
fn committed_package_publishes_clanjianzhi_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANJIANZHI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn phase_end_without_qieyi_fires_is_noop() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_skills(&mut tx, owner, &[CLANJIANZHI]);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    dispatch_phase_end(&mut tx, owner);
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn matching_judge_cards_are_assigned_and_non_matching_deals_thunder() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = strip_and_add_skills(&mut tx, owner, &[CLANJIANZHI, CLANQIEYI]);
    let qieyi = skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CLANQIEYI)
        .unwrap();
    // 惬诣 fired twice this turn (the dispatcher normally increments this).
    tx.state_mut().skills.get_mut(qieyi).unwrap().turn_fire_count = 2;
    // This play phase the owner used a spade and a heart card.
    tx.state_mut().turn.remember_play_phase_card_suit_used(owner, Suit::Spade);
    tx.state_mut().turn.remember_play_phase_card_suit_used(owner, Suit::Heart);
    // Judge 1: spade (matching); judge 2: club (non-matching).
    let spade_judge = inject_pile_top(tx.state_mut(), CardKind::Sha, Suit::Spade);
    let club_judge = inject_pile_top(tx.state_mut(), CardKind::Shan, Suit::Club);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    dispatch_phase_end(&mut tx, owner);
    assert!(
        tx.state().has_pending_choice(),
        "judgment count choice must open"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::JianzhiCountChoice
    );

    // Choose 2 judgments.
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(51),
        vec![2],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard,
        "assignment loop opens with the matching spade card"
    );
    let options: Vec<_> = state
        .resolution
        .continuation_payload
        .get(4..)
        .unwrap_or(&[])
        .to_vec();
    assert_eq!(options, vec![spade_judge.raw()], "only the spade card matches");
    // The non-matching club judge card left the pile for the discard pile (judgments
    // remove their card from the pile; the engine has no limbo zone).
    assert!(
        !state
            .zones
            .get(
                state
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .unwrap()
            )
            .unwrap()
            .cards
            .contains(&club_judge)
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&club_judge));

    // Assign the spade card to the other player: stage 0 selects the card, stage 1 the
    // recipient.
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(52),
        vec![spade_judge.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(53),
        vec![other.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(
        state.hand_cards(other).contains(&spade_judge),
        "matching judge card given to the chosen player"
    );
    assert!(
        !state.hand_cards(owner).contains(&spade_judge),
        "owner no longer holds the judge card"
    );
    // The loop ended and the non-matching club judge dealt 1 thunder damage.
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp - 1,
        "non-matching judge card dealt thunder damage"
    );
}

#[test]
fn assignment_loop_covers_multiple_matching_cards() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = strip_and_add_skills(&mut tx, owner, &[CLANJIANZHI, CLANQIEYI]);
    let qieyi = skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CLANQIEYI)
        .unwrap();
    tx.state_mut().skills.get_mut(qieyi).unwrap().turn_fire_count = 2;
    tx.state_mut().turn.remember_play_phase_card_suit_used(owner, Suit::Spade);
    // Both judge cards are spades → both match.
    let spade_judge_1 = inject_pile_top(tx.state_mut(), CardKind::Sha, Suit::Spade);
    let spade_judge_2 = inject_pile_top(tx.state_mut(), CardKind::Shan, Suit::Spade);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    dispatch_phase_end(&mut tx, owner);
    let (after_count, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(61),
        vec![2],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        after_count.resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard
    );
    // First assignment: stage 0 selects a card (first in the payload), stage 1 a target.
    let tx = Transaction::begin(
        resolve_choice(
            after_count,
            owner,
            IdempotencyKey(62),
            vec![spade_judge_1.raw()],
        )
        .0,
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard
    );
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(63),
        vec![owner.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&spade_judge_1));
    // The loop re-opened for the remaining card.
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard
    );
    assert_eq!(
        state
            .resolution
            .continuation_payload
            .get(4..)
            .unwrap_or(&[])
            .to_vec(),
        vec![spade_judge_2.raw()]
    );
    // Assign the second card; all matched → no damage.
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(64),
        vec![spade_judge_2.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(65),
        vec![other.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(other).contains(&spade_judge_2));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}

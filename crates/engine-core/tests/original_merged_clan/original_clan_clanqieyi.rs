//! Golden coverage for clan 惬诣 (clanqieyi): locked clan skill whose play-phase start
//! resets per-suit first-use storage and privately views the top two draw-pile cards;
//! whose card-used path (first use of a suit this play phase) reveals the top draw-pile
//! card and gains it when it shares color or kind with the used card, else opens the
//! put-one-card-on-top choice.
//!
//! The draw pile's top is its last element; a move into the draw zone lands on top.
//! Storage keys 10..13 hold the per-suit first-use flags (suit tag offset).

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

const CLANQIEYI: u32 = 3_347_941_832;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_812,
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

fn strip_and_add_clanqieyi(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANQIEYI),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANQIEYI)
        .unwrap()
        .id
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_phase_play(tx: &mut Transaction, owner: PlayerId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId) {
    let fired = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("card_used dispatch error: {e:?}"),
    }
}

#[test]
fn committed_package_publishes_clanqieyi_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let triggers: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANQIEYI)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "phase_play" && trigger.filter == "owner_is_target"));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "card_used" && trigger.filter == "owner_source"));
}

#[test]
fn phase_start_resets_suit_gate_and_reveals_top_two() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanqieyi(&mut tx, owner);
    // Simulate stale suit flags from a previous play phase.
    tx.state_mut().skills.storage_set(skill, 10, 1);
    tx.state_mut().skills.storage_set(skill, 12, 1);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top_two: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    assert_eq!(top_two.len(), 2);

    dispatch_phase_play(&mut tx, owner);
    // Suit flags cleared.
    assert_eq!(tx.state().skills.storage_get(skill, 10), 0);
    assert_eq!(tx.state().skills.storage_get(skill, 12), 0);
    // Top two cards still on the pile (view only, no reorder).
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let after: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    assert_eq!(after, top_two);
}

#[test]
fn first_suit_use_gains_same_kind_top_card() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanqieyi(&mut tx, owner);
    let owner_hand = tx.state().hand_cards(owner);
    let owner_count = owner_hand.len();
    dispatch_phase_play(&mut tx, owner);

    // Force the draw pile's top card to be a 杀: move the top card out, inject a 杀
    // directly on top, and remember the pile's second card for the used card.
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top_sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_101),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(top_sha);
    // Owner uses a 杀 (same kind) — the revealed top card must be gained.
    let used_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    dispatch_card_used(&mut tx, owner, used_sha);
    assert!(
        tx.state().hand_cards(owner).contains(&top_sha),
        "same-kind top card gained"
    );
    // Original hand + the injected used 杀 + the gained top 杀.
    assert_eq!(tx.state().hand_cards(owner).len(), owner_count + 2);
    // The suit gate is now closed for spade.
    assert_eq!(tx.state().skills.storage_get(skill, 10), 1);
    // The other player's use does not disturb the owner's gate.
    dispatch_card_used(&mut tx, other, used_sha);
    assert_eq!(tx.state().skills.storage_get(skill, 10), 1);
}

#[test]
fn non_matching_reveal_offers_put_card_on_top() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanqieyi(&mut tx, owner);
    dispatch_phase_play(&mut tx, owner);

    // Top card: 桃 (heart, basic). Owner uses a spade 杀 → neither same color nor kind.
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top_tao = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_102),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(top_tao);
    let used_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let hand_card = inject_hand(tx.state_mut(), owner, CardKind::Shan);
    dispatch_card_used(&mut tx, owner, used_sha);
    // 桃 stays on the pile; the put-on-top choice opened over the owner's hand cards.
    assert!(!tx.state().hand_cards(owner).contains(&top_tao));
    assert!(
        tx.state().has_pending_choice(),
        "put-on-top choice must open"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::QieyiPutTopChoice
    );

    // Stage 0: put the 闪 on top of the draw pile.
    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(41),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![hand_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&hand_card));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = state.zones.get(draw).unwrap().cards.last().copied();
    assert_eq!(
        top,
        Some(hand_card),
        "selected card lands on top of the pile"
    );
}

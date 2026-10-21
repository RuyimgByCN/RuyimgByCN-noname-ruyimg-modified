//! Golden coverage for clan 英襄 (clanxsyingxiang): locked clan skill derived from 清绝.
//! When a card carrying the clanxsyingxiang gaintag is consumed (leaves its holder) this turn
//! and a card is used, the owner draws. Exercises the gaintag_map foundation primitive:
//!   - `turn.lost_card_tags` tracking (cards' content tags recorded at loss),
//!   - `owner_alive_tagged_card_lost_this_turn` trigger filter.
//! Per-turn approximation of upstream's per-useEvent `hasHistory("lose", gaintag_map)`
//! correlation (the engine has no event-parent ids; see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANXSYINGXIANG: u32 = 3_188_595_142;
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
            rng_seed: 20_260_810,
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

fn inject_hand(state: &mut engine_core::state::GameState, player: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
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

fn owner_with_skill(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANXSYINGXIANG),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == CLANXSYINGXIANG)
        .unwrap()
        .id
}

#[test]
fn committed_package_publishes_clanxsyingxiang_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANXSYINGXIANG
            && trigger.event == "card_used"
            && trigger.filter == "owner_alive_tagged_card_lost_this_turn"
    }));
}

#[test]
fn tagged_card_consumed_this_turn_fires_cross_player_draw() {
    // The owner holds clanxsyingxiang. A tagged card was consumed this turn. When ANOTHER player
    // uses a card targeting the owner, the skill fires draw_all_event_targets(1): the user AND
    // the owner (a target) each draw 1. Verifies the cross-player draw branch.
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);

    // Record a tagged-card loss so the filter passes.
    tx.state_mut().turn.record_lost_card_tag(CLANXSYINGXIANG);
    assert!(tx.state().turn.lost_card_tag(CLANXSYINGXIANG));

    let used_card = inject_hand(tx.state_mut(), other, 170_001);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card: used_card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // draw_all_event_targets(1) is emitted as one DrawAllEventTargets request whose player list
    // is the event player + the declared targets; run_effects expands it to per-player draws.
    let draw_all = fired.iter().find_map(|e| match e {
        EffectRequest::DrawAllEventTargets { players, count } => Some((players.clone(), *count)),
        _ => None,
    });
    let (mut players, count) = draw_all.expect("draw_all_event_targets(1) effect emitted");
    players.sort();
    let mut expected = vec![other, owner];
    expected.sort();
    assert_eq!(
        players, expected,
        "draws the event player and the owner (target)"
    );
    assert_eq!(count, 1);
    let owner_before = tx.state().hand_count(owner);
    let other_before = tx.state().hand_count(other);
    run_effects(&mut tx, fired).unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert_eq!(tx.state().hand_count(other), other_before + 1);
}

#[test]
fn no_tagged_card_lost_blocks_the_trigger() {
    // Symmetric negative case: without a tagged-card loss, the filter blocks the trigger.
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);
    let used_card = inject_hand(tx.state_mut(), other, 170_001);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card: used_card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let owner_draws = fired
        .iter()
        .filter(|e| matches!(e, EffectRequest::DrawCards { player, .. } | EffectRequest::DrawCardsFromSkill { player, .. } if *player == owner))
        .count();
    assert_eq!(owner_draws, 0, "no tagged card lost → no draw");
}

#[test]
fn gaintag_map_primitive_records_and_reports_lost_tags() {
    // State-layer verification of the gaintag_map foundation primitive (independent of the
    // runtime card_used dispatch path): recording a lost tag makes it queryable, and the
    // turn-scoped set clears on rollover.
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _ = owner;
    // Initially no tags are recorded as lost.
    assert!(!tx.state().turn.lost_card_tag(CLANXSYINGXIANG));
    // Recording the tag makes it present.
    tx.state_mut().turn.record_lost_card_tag(CLANXSYINGXIANG);
    assert!(tx.state().turn.lost_card_tag(CLANXSYINGXIANG));
    // Round/turn rollover clears the set.
    tx.state_mut().turn.clear_lost_card_tags();
    assert!(!tx.state().turn.lost_card_tag(CLANXSYINGXIANG));
}

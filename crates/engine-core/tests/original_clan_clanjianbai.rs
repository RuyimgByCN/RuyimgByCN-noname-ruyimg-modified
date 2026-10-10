//! Golden coverage for clan 坚白 (clanjianbai): locked clan skill that tracks each used card
//! category (kind tag) in keyed storage and draws 1 per use (a verifiable subset of upstream's
//! per-category-first-use + chooseControl + recast + gaintag-counter behavior).
//!
//! Verifies the storage-map primitive keyed by card kind and the `owner_source` card_used filter.
//! The full upstream semantics (per-category-first gating, suit keep/recast, gaintag accumulating
//! counters, round-end give+draw) need additional primitives not yet ported; see the .skill comment.

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

const CLANJIANBAI: u32 = 3_207_476_732;
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

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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
        skill_def: SkillDefId::new(CLANJIANBAI),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == CLANJIANBAI)
        .unwrap()
        .id
}

fn used_card(tx: &mut Transaction, owner: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanjianbai_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANJIANBAI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source"
    }));
}

#[test]
fn card_use_records_kind_in_storage_and_first_per_kind_draws() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = owner_with_skill(&mut tx, owner);

    // First 杀 use (kind=1): records key=1, draws 1 (first per-category).
    let sha = inject_hand(tx.state_mut(), owner, CardKind::Sha, 170_301);
    let hand_before = tx.state().hand_count(owner);
    used_card(&mut tx, owner, sha);
    let storage = &tx.state().skills.get(skill).unwrap().content_storage;
    assert!(storage.contains_key(&(CardKind::Sha as u32)), "kind key recorded");
    assert_eq!(tx.state().hand_count(owner), hand_before + 1, "first per-kind → draw 1");

    // Second 杀 use (same kind=1): key already recorded → does NOT draw.
    let sha2 = inject_hand(tx.state_mut(), owner, CardKind::Sha, 170_303);
    let hand_before2 = tx.state().hand_count(owner);
    used_card(&mut tx, owner, sha2);
    assert_eq!(tx.state().hand_count(owner), hand_before2, "repeat same kind → no draw");

    // First 桃 use (kind=2): different key, first for this kind → draws 1.
    let tao = inject_hand(tx.state_mut(), owner, CardKind::Tao, 170_302);
    let hand_before3 = tx.state().hand_count(owner);
    used_card(&mut tx, owner, tao);
    let storage = &tx.state().skills.get(skill).unwrap().content_storage;
    assert_eq!(storage.len(), 2, "two distinct kind keys recorded");
    assert_eq!(tx.state().hand_count(owner), hand_before3 + 1, "first per new kind → draw 1");
}

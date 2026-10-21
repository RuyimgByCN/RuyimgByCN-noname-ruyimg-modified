//! Golden coverage for clan 百出 (clanbaichu): locked clan skill that records each suit+category
//! combination of used ordinary tricks in keyed storage, and draws 1 when a combination repeats.
//!
//! Verifies the storage-map primitive (`storage_get`/`storage_set`) used to key the suit+kind
//! map and the `owner_source_ordinary_trick` filter. This is a verifiable subset of upstream
//! `clanbaichu` (the chooseButton name-recording and chooseDrawRecover recover-branch and the
//! dynamic 奇策 acquisition are not yet ported; see the .skill comment).

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

const CLANBAICHU: u32 = 4_194_820_394;
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

fn inject_trick_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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
        skill_def: SkillDefId::new(CLANBAICHU),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == CLANBAICHU)
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
fn committed_package_publishes_clanbaichu_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANBAICHU
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_ordinary_trick"
    }));
}

#[test]
fn first_trick_use_records_and_second_repeat_draws() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = owner_with_skill(&mut tx, owner);

    // First ordinary-trick use (黑桃普通锦囊 TrickWuzhong): records the suit+kind key, NO draw.
    let trick1 = inject_trick_hand(
        tx.state_mut(),
        owner,
        CardKind::TrickWuzhong,
        Suit::Spade,
        170_201,
    );
    let hand_before_first = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick1);
    let any_recorded = !tx
        .state()
        .skills
        .get(skill)
        .unwrap()
        .content_storage
        .is_empty();
    assert!(any_recorded, "first trick use records the suit+kind key");
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before_first,
        "first use does NOT draw (key not yet recorded)"
    );

    // Second use of a trick with the SAME suit+kind: key already recorded → draws 1.
    let trick2 = inject_trick_hand(
        tx.state_mut(),
        owner,
        CardKind::TrickWuzhong,
        Suit::Spade,
        170_202,
    );
    let hand_before_second = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick2);
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before_second + 1,
        "repeated suit+kind → draw 1"
    );
}

#[test]
fn different_suit_records_distinct_keys() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = owner_with_skill(&mut tx, owner);

    // Use a spade trick (records key A, no draw), then a heart trick (different key B, no draw).
    let trick1 = inject_trick_hand(
        tx.state_mut(),
        owner,
        CardKind::TrickWuzhong,
        Suit::Spade,
        170_211,
    );
    let hand_before = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick1);
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "first use → no draw"
    );

    let trick2 = inject_trick_hand(
        tx.state_mut(),
        owner,
        CardKind::TrickWuzhong,
        Suit::Heart,
        170_212,
    );
    let hand_before2 = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick2);
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before2,
        "new suit+kind → no draw (first for that key)"
    );

    // Two distinct keys recorded (spade vs heart same kind).
    let storage = &tx.state().skills.get(skill).unwrap().content_storage;
    assert_eq!(storage.len(), 2, "two distinct suit+kind keys recorded");
}

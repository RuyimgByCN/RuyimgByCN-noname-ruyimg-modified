//! Golden coverage for sp 内伐·用 (neifa_use): locked sub-skill that, after its
//! owner uses an equipment card, draws as many cards as the owner's current
//! 「内伐·无基本」mark count (upstream `player.draw(player.countMark(...))`).
//!
//! Exercises the body-level equipment-kind check (`card_attribute(event_card, 0)`
//! in the 10..=14 tag range) and the `mark_count` query feeding `draw_cards`.
//! The upstream `usable: 2` per-turn limit has no corresponding filter primitive;
//! the ported .skill documents this approximation.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NEIFA_USE: u32 = 2_442_423_886;
const NEIFA_NOBASIC: u32 = 3_552_802_810;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815,
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

fn strip_and_add_neifa_use(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(NEIFA_USE),
    });
    run_effects(tx, effects).unwrap();
}

fn set_marks(tx: &mut Transaction, player: PlayerId, count: i32) {
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player,
            mark: MarkDefId::new(NEIFA_NOBASIC),
            delta: count,
        }],
    )
    .unwrap();
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
fn committed_package_publishes_neifa_use_equip_draw_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NEIFA_USE
            && trigger.event == "card_used"
            && trigger.filter == "owner_source"
    }));
}

#[test]
fn using_equip_with_marks_draws_marks_count() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_neifa_use(&mut tx, owner);
    set_marks(&mut tx, owner, 3);
    let equip = inject_hand(tx.state_mut(), owner, CardKind::Weapon);
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, equip);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 3,
        "equip use with 3 marks draws 3"
    );
}

#[test]
fn using_equip_without_marks_draws_nothing() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_neifa_use(&mut tx, owner);
    let equip = inject_hand(tx.state_mut(), owner, CardKind::Armor);
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, equip);

    assert_eq!(tx.state().hand_count(owner), hand_before, "0 marks → no draw");
}

#[test]
fn using_non_equip_with_marks_draws_nothing() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_neifa_use(&mut tx, owner);
    set_marks(&mut tx, owner, 2);
    let sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, sha);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "non-equip card → no draw despite marks"
    );
}

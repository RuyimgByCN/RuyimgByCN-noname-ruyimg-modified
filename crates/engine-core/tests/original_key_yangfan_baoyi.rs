//! Golden coverage for KEY 扬帆 / 爆衣 rewritten from honest noops.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const KAMOME_YANGFAN: u32 = 3_575_287_643;
const RYOICHI_BAOYI: u32 = 2_396_712_868;
const YANGFAN_SRC: &str =
    include_str!("../../../content/original/ported/key/kamome_yangfan.skill");
const BAOYI_SRC: &str = include_str!("../../../content/original/ported/key/ryoichi_baoyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(engine_api::Command {
        actor: PlayerId::NONE,
        expected_revision: engine_api::ExpectedRevision(0),
        idempotency_key: engine_api::IdempotencyKey(20_260_815_50),
        kind: engine_api::CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_50,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn move_equip_to_discard(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(357_501),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 1,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    card
}

#[test]
fn yangfan_source_and_npack_publish_lose_equip_draw() {
    let compiled = content_compiler::compile_source_named("kamome_yangfan", YANGFAN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KAMOME_YANGFAN)
        .expect("yangfan trigger");
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_from_equip");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KAMOME_YANGFAN && trigger.filter == "owner_from_equip"
    }));
}

#[test]
fn yangfan_losing_equip_draws_two() {
    let (mut tx, owner) = setup(YANGFAN_SRC, "yangfan-test", KAMOME_YANGFAN);
    let before = tx.state().hand_count(owner);
    move_equip_to_discard(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), before + 2);
}

#[test]
fn baoyi_source_and_npack_publish_lose_equip_draw() {
    let compiled = content_compiler::compile_source_named("ryoichi_baoyi", BAOYI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RYOICHI_BAOYI)
        .expect("baoyi trigger");
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_from_equip");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == RYOICHI_BAOYI && trigger.filter == "owner_from_equip"
    }));
}

#[test]
fn baoyi_losing_equip_draws_one() {
    let (mut tx, owner) = setup(BAOYI_SRC, "baoyi-test", RYOICHI_BAOYI);
    let before = tx.state().hand_count(owner);
    move_equip_to_discard(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

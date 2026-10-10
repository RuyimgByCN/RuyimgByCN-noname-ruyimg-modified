//! Golden coverage for Standard 急救 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{CardDefId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::dying_save_card_for;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIJIU: u32 = 2_961_041_949;
const SOURCE: &str = include_str!("../../../content/standard/jijiu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_600 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_closes_huatuo_with_response_only_jijiu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JIJIU)
        .unwrap();
    assert_eq!(view.as_kind, "tao");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "owner_outside_turn_red");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    let huatuo = package
        .character_index
        .iter()
        .find(|character| character.name_key == "huatuo")
        .unwrap();
    assert!(huatuo.skills.contains(&JIJIU));
}

#[test]
fn red_hand_or_equipment_card_is_tao_only_outside_owner_turn() {
    let package = compile_source_named("standard-jijiu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_44, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    for skill in tx.state().players.get(owner).unwrap().skills.clone() {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let old_hand = tx.state().hand_cards(owner);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: old_hand,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIJIU),
        }],
    )
    .unwrap();
    let black = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let red = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        8,
    );

    tx.state_mut().turn.current = other;
    assert_eq!(dying_save_card_for(tx.state(), owner, other), Some(red));
    tx.state_mut().turn.current = owner;
    assert_eq!(dying_save_card_for(tx.state(), owner, other), None);
    assert_ne!(black, red);
}

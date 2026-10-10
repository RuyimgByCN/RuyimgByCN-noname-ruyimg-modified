//! Golden for SHIJI 骆统「勤政」rewritten from legacy JavaScript semantics.

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit, ZoneId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QINZHENG: u32 = 3_099_323_686;
const LUOTONG: u32 = 3_283_005_586;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, QINZHENG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(QINZHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1])
}

fn add_card(tx: &mut Transaction, zone: ZoneId, kind: CardKind, def: u32) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn emit_action(
    tx: &mut Transaction,
    player: PlayerId,
    other: PlayerId,
    responded: bool,
) -> Vec<DomainEvent> {
    let event = if responded {
        DomainEvent::CardResponded {
            player,
            respond_to: other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }
    } else {
        DomainEvent::CardUsed {
            player,
            card: CardId::NONE,
            targets: vec![other],
        }
    };
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap()
}

#[test]
fn package_publishes_abi198_skill_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 198);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == QINZHENG && skill.name == "qinzheng" && skill.locked
    }));
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LUOTONG)
            .unwrap()
            .skills,
        [QINZHENG]
    );
}

#[test]
fn action_120_gains_topmost_three_groups_in_one_batch() {
    let (mut tx, owner, other) = setup();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let older_shan = add_card(&mut tx, draw, CardKind::Shan, 91_000);
    let wuzhong = add_card(&mut tx, draw, CardKind::TrickWuzhong, 91_001);
    let jiu = add_card(&mut tx, draw, CardKind::Jiu, 91_002);
    let fire_sha = add_card(&mut tx, draw, CardKind::ShaFire, 91_003);
    tx.state_mut().card_action_totals.insert(owner, 119);

    let events = emit_action(&mut tx, owner, other, true);

    assert_eq!(tx.state().card_action_totals.get(&owner), Some(&120));
    let hand = tx.state().hand_cards(owner);
    assert!(hand.contains(&fire_sha) && hand.contains(&jiu) && hand.contains(&wuzhong));
    assert!(!hand.contains(&older_shan));
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved { cards, from, .. }
            if *from == draw && cards == &[fire_sha, jiu, wuzhong]
    )));
}

#[test]
fn matching_discard_card_is_not_a_fallback() {
    let (mut tx, owner, other) = setup();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    tx.state_mut().zones.get_mut(draw).unwrap().cards.clear();
    add_card(&mut tx, draw, CardKind::Weapon, 91_010);
    let sha = add_card(&mut tx, discard, CardKind::Sha, 91_011);
    tx.state_mut().card_action_totals.insert(owner, 2);
    let before = tx.state().hand_count(owner);

    emit_action(&mut tx, owner, other, false);

    assert_eq!(tx.state().hand_count(owner), before);
    assert!(tx.state().zones.get(discard).unwrap().cards.contains(&sha));
}

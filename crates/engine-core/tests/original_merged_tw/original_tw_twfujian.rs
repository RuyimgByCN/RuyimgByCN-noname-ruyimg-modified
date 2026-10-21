//! Golden coverage for TW 夏侯恩「负剑」.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, EquipSlot, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFUJIAN: u32 = 2_940_701_334;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twfujian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

#[test]
fn equips_one_weapon_and_loses_hp_only_when_it_leaves_outside_the_turn() {
    let package = content_compiler::compile_source_named("twfujian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 366);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWFUJIAN));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_106, "identity-standard").unwrap();
    // Roster-independent: the standard roster gained choice-bearing characters
    // (gongsunzan 义从 / old_re_lidian 询询+忘隙) whose booted presence changes
    // range/hand-limit/trigger answers; this test grants its own skills.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }

    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let injected = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_366_001),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 2,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(injected);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUJIAN),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::EquipWeaponFromDrawPile { player: owner }],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx
        .state()
        .zones
        .get(equip)
        .unwrap()
        .cards
        .iter()
        .copied()
        .find(|card| {
            tx.state().cards.get(*card).unwrap().kind.equip_slot() == Some(EquipSlot::Weapon)
        })
        .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().turn.current = other;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

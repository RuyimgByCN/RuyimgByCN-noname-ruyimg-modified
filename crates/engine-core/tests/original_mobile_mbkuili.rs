//! Golden coverage for MOBILE 玄毌丘俭「溃离」rewritten from legacy JavaScript.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{EquipSlot, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBKUILI: u32 = 2_909_034_781;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mbkuili.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MBKUILI),
        }],
    )
    .unwrap();
    (tx, owner, source)
}

#[test]
fn package_exposes_locked_damage_source_weapon_restore_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    assert_eq!(package.trigger_index.len(), 198);
    assert_eq!(package.modifier_index.len(), 225);
    assert_eq!(package.active_index.len(), 47);
    assert_eq!(package.view_as_index.len(), 8);
    assert_eq!(package.modules.len(), 245);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBKUILI && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBKUILI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_is_target_source_alive_disabled_weapon"
    );
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("mbkuili", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 161);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "mbkuili_damage"),
        compiled.modules.first()
    );
}

#[test]
fn damage_restores_exactly_one_disabled_weapon_slot_with_owner_attribution() {
    let (mut tx, owner, source) = setup(2_026_072_501);
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .disabled_equip_slots[EquipSlot::Weapon.index()] = 2;

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();

    assert_eq!(
        tx.state().players.get(source).unwrap().disabled_equip_slots[EquipSlot::Weapon.index()],
        1
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::EquipSlotEnabled {
                source: event_source,
                player,
                slot: 1
            } if *event_source == owner && *player == source
        )
    }));
}

#[test]
fn enabled_weapon_slot_or_unattributed_damage_does_not_fire() {
    let (mut tx, owner, source) = setup(2_026_072_502);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::EquipSlotEnabled { .. })));

    let (mut tx, owner, source) = setup(2_026_072_503);
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .disabled_equip_slots[EquipSlot::Weapon.index()] = 1;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().players.get(source).unwrap().disabled_equip_slots[EquipSlot::Weapon.index()],
        1
    );
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::EquipSlotEnabled { .. })));
}

//! Golden coverage for OL界黄忠「没矢」rewritten from legacy JavaScript semantics.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_card_flag, EffectRequest};

const OL_HUANGZHONG: u32 = 3_398_640_656;
const XINLIEGONG: u32 = 3_272_111_610;
const REMOSHI: u32 = 3_096_399_491;
const REMOSHI_STUCK: u32 = 4_090_725_192;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/remoshi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    equipment: CardId,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: CardDefId,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def,
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(equipment_kind: CardKind, physical_sha: bool) -> Setup {
    let package = content_compiler::compile_source_named("remoshi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_730, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REMOSHI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::Processing,
        CardKind::Sha,
        if physical_sha {
            CardDefId::new(99_318)
        } else {
            CardDefId::NONE
        },
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        equipment_kind,
        CardDefId::new(99_319),
    );
    Setup {
        state: tx.commit().state,
        owner,
        target,
        sha,
        equipment,
    }
}

fn damage(mut setup: Setup, card_flags: u8) -> Setup {
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.owner,
            target: setup.target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: setup.sha,
            card_kind: CardKind::Sha.tag(),
            card_flags,
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    setup
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_complete_character_and_abi318_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 318);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == OL_HUANGZHONG)
            .unwrap()
            .skills,
        [XINLIEGONG, REMOSHI]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == REMOSHI_STUCK && skill.name == "remoshi_stuck" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REMOSHI
            && trigger.event == "damage"
            && trigger.filter
                == "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REMOSHI
            && trigger.event == "cards_moved"
            && trigger.filter
                == "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile"
            && !trigger.optional
    }));
}

#[test]
fn physical_sha_is_stored_then_returned_when_target_loses_armor() {
    let setup = damage(setup(CardKind::Armor, true), 0);
    assert_eq!(
        setup.state.skill_pile_cards(setup.target, REMOSHI),
        [setup.sha]
    );
    assert!(has_skill(&setup.state, setup.target, REMOSHI_STUCK));

    let equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.target)
        .unwrap();
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![setup.equipment],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skill_pile_cards(setup.target, REMOSHI)
        .is_empty());
    assert!(tx.state().hand_cards(setup.owner).contains(&setup.sha));
}

#[test]
fn multi_card_view_as_sha_stores_every_physical_source() {
    let mut setup = setup(CardKind::Armor, true);
    let mut tx = Transaction::begin(setup.state);
    let extra = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::TrickWuzhong,
        CardDefId::new(99_321),
    );
    tx.state_mut().resolution.register_sha_resolution(
        setup.owner,
        setup.sha,
        &[setup.target],
        false,
    );
    assert!(tx.state_mut().resolution.register_sha_source_cards(
        setup.owner,
        setup.sha,
        &[setup.sha, extra]
    ));
    setup.state = tx.commit().state;

    let setup = damage(setup, 0);
    assert_eq!(
        setup.state.skill_pile_cards(setup.target, REMOSHI),
        [setup.sha, extra]
    );
}

#[test]
fn losing_a_weapon_does_not_return_the_stuck_sha() {
    let mut setup = damage(setup(CardKind::Armor, true), 0);
    let mut tx = Transaction::begin(setup.state);
    let weapon = inject(
        &mut tx,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        CardDefId::new(99_320),
    );
    setup.state = tx.commit().state;
    let equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.target)
        .unwrap();
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().skill_pile_cards(setup.target, REMOSHI),
        [setup.sha]
    );
}

#[test]
fn chain_damage_virtual_sha_and_weapon_only_target_do_not_place_cards() {
    for setup in [
        damage(
            setup(CardKind::Armor, true),
            damage_card_flag::CHAIN_PROPAGATED,
        ),
        damage(setup(CardKind::Armor, false), 0),
        damage(setup(CardKind::Weapon, true), 0),
    ] {
        assert!(setup
            .state
            .skill_pile_cards(setup.target, REMOSHI)
            .is_empty());
        assert_eq!(
            setup.state.cards.get(setup.sha).unwrap().zone,
            setup
                .state
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .unwrap()
        );
    }
}

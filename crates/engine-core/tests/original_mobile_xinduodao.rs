//! Golden coverage for MOBILE 界潘璋马忠「夺刀」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINDUODAO: u32 = 3_614_910_583;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinduodao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINDUODAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source)
}

fn add_equipment(tx: &mut Transaction, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
}

fn confirm(
    state: GameState,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_skill_and_now_publishes_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == XINDUODAO && skill.name == "xinduodao" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINDUODAO
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_has_weapon"
            && trigger.optional
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "xin_panzhangmazhong"));

    let compiled = content_compiler::compile_source_named("xinduodao", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn accepting_gains_every_current_weapon_and_leaves_other_equipment() {
    let (state, owner, source) = setup(2_026_072_404);
    let mut tx = Transaction::begin(state);
    let weapon_a = add_equipment(&mut tx, source, XINDUODAO + 1, CardKind::Weapon);
    let weapon_b = add_equipment(&mut tx, source, XINDUODAO + 2, CardKind::Weapon);
    let armor = add_equipment(&mut tx, source, XINDUODAO + 3, CardKind::Armor);
    let (state, result) = damage(tx.commit().state, source, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert!(state.has_pending_choice());

    let (state, accepted) = confirm(state, owner, true, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let source_equip = state.zones.find(ZoneKind::Equip, source).unwrap();
    for weapon in [weapon_a, weapon_b] {
        assert_eq!(state.cards.get(weapon).unwrap().zone, owner_hand);
        assert_eq!(state.cards.get(weapon).unwrap().owner, owner);
    }
    assert_eq!(state.cards.get(armor).unwrap().zone, source_equip);
    assert_eq!(state.cards.get(armor).unwrap().owner, source);
}

#[test]
fn no_weapon_opens_no_choice_and_declining_keeps_weapon_equipped() {
    let (state, owner, source) = setup(2_026_072_405);
    let mut tx = Transaction::begin(state);
    add_equipment(&mut tx, source, XINDUODAO + 4, CardKind::Armor);
    let (state, result) = damage(tx.commit().state, source, owner);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    let weapon = add_equipment(&mut tx, source, XINDUODAO + 5, CardKind::Weapon);
    let source_equip = tx.state().zones.find(ZoneKind::Equip, source).unwrap();
    let (state, result) = damage(tx.commit().state, source, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, declined) = confirm(state, owner, false, 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(weapon).unwrap().zone, source_equip);
    assert_eq!(state.cards.get(weapon).unwrap().owner, source);
}

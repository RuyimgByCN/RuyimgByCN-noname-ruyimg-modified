//! Golden coverage for MOBILE equipment skill「霹雳车」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LY_PILICHE: u32 = 2_383_388_842;
const SOURCE: &str = include_str!("../../../../content/original/ported/mobile/ly_piliche.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LY_PILICHE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target)
}

fn add_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Diamond,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
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
fn package_contains_equipment_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 441);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LY_PILICHE && skill.name == "ly_piliche"));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LY_PILICHE
            && trigger.event == "damage"
            && trigger.filter == "owner_source_other_target_alive_has_discardable_hand_equip"
            && trigger.optional
    }));

    let compiled = content_compiler::compile_source_named("ly_piliche", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn accepting_discards_all_equipment_and_never_discards_target_hand() {
    let (state, owner, target) = setup(2_026_072_406);
    let mut tx = Transaction::begin(state);
    let weapon = add_card(
        &mut tx,
        target,
        ZoneKind::Equip,
        LY_PILICHE + 1,
        CardKind::Weapon,
    );
    let armor = add_card(
        &mut tx,
        target,
        ZoneKind::Equip,
        LY_PILICHE + 2,
        CardKind::Armor,
    );
    let hand = add_card(
        &mut tx,
        target,
        ZoneKind::Hand,
        LY_PILICHE + 3,
        CardKind::TrickWuzhong,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let (state, result) = damage(tx.commit().state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = confirm(state, owner, true, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    assert_eq!(state.cards.get(armor).unwrap().zone, discard);
    assert_eq!(state.cards.get(hand).unwrap().zone, target_hand);
    assert_eq!(state.cards.get(hand).unwrap().owner, target);
}

#[test]
fn hand_only_target_can_confirm_but_effect_is_a_noop() {
    let (state, owner, target) = setup(2_026_072_407);
    let mut tx = Transaction::begin(state);
    let hand = add_card(
        &mut tx,
        target,
        ZoneKind::Hand,
        LY_PILICHE + 4,
        CardKind::TrickWuzhong,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let (state, result) = damage(tx.commit().state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = confirm(state, owner, true, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(hand).unwrap().zone, target_hand);
}

#[test]
fn self_damage_and_empty_other_target_do_not_offer_the_skill() {
    let (state, owner, target) = setup(2_026_072_408);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let (state, other_result) = damage(tx.commit().state, owner, target);
    assert!(other_result.is_ok());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    add_card(
        &mut tx,
        owner,
        ZoneKind::Equip,
        LY_PILICHE + 5,
        CardKind::Armor,
    );
    let (state, self_result) = damage(tx.commit().state, owner, owner);
    assert!(self_result.is_ok());
    assert!(!state.has_pending_choice());
}

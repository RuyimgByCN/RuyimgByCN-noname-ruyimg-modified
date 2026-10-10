//! Golden coverage for REFRESH「趫猛」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{apply_effect, bootstrap_match, move_one_card, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIAOMENG: u32 = 2_724_752_546;
const REQIAOMENG: u32 = 2_397_519_504;
const REYICONG: u32 = 3_426_755_583;
const RE_GONGSUNZAN: u32 = 3_363_210_554;
const GONGSUNZAN: u32 = 3_561_346_810;
const XINYICONG: u32 = 3_328_413_812;
const XIN_GONGSUNZAN: u32 = 574_905_297;
const PROTECT: u32 = 91_847;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/qiaomeng.skill"),
    include_str!("../../../content/original/ported/refresh/reqiaomeng.skill"),
    "\nskill protect(91847) locked { modifier card_discard_protected(equip) override 1; }\n"
);
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(
    damage_kind: CardKind,
    damage_suit: Suit,
    equipment_kind: CardKind,
    protected: bool,
) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("qiaomeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(QIAOMENG),
    });
    if protected {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(PROTECT),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let damage_card = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        damage_kind,
        damage_suit,
        92_000,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        equipment_kind,
        Suit::Club,
        92_001,
    );
    (tx.commit().state, owner, target, damage_card, equipment)
}

fn setup_reqiaomeng(
    damage_kind: CardKind,
    target_zone: ZoneKind,
    target_card_kind: CardKind,
) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("reqiaomeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_719, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(REQIAOMENG),
    });
    run_effects(&mut tx, effects).unwrap();
    let damage_card = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        damage_kind,
        Suit::Heart,
        92_100,
    );
    let target_card = inject(
        &mut tx,
        target,
        target_zone,
        target_card_kind,
        Suit::Diamond,
        92_101,
    );
    (tx.commit().state, owner, target, damage_card, target_card)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
) -> (GameState, Result<Vec<DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn open_equipment_choice(equipment_kind: CardKind) -> (GameState, PlayerId, PlayerId, CardId) {
    let (state, owner, target, sha, equipment) =
        setup(CardKind::Sha, Suit::Spade, equipment_kind, false);
    let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardTargetCardGainMount
    );
    assert_eq!(
        accepted
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![equipment.raw()]
    );
    (state, owner, target, equipment)
}

#[test]
fn packages_publish_exact_skill_character_and_dependency_contract() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    assert!(refresh.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == QIAOMENG
            && trigger.event == "damage"
            && trigger.filter == "owner_source_black_sha_target_alive_has_discardable_equip"
            && trigger.optional
    }));
    assert!(refresh.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REQIAOMENG
            && trigger.event == "damage"
            && trigger.filter == "owner_source_sha_target_alive_has_discardable_region_card"
            && trigger.optional
    }));
    let re_gongsunzan = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_GONGSUNZAN)
        .unwrap();
    assert_eq!(re_gongsunzan.skills, vec![REQIAOMENG, REYICONG]);
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    for package_id in ["refresh", "yijiang"] {
        assert!(mobile.manifest.dependencies.iter().any(|dependency| {
            dependency.namespace == "official"
                && dependency.package_id == package_id
                && dependency.version == "0.1.0"
        }));
    }
    let character = mobile
        .character_index
        .iter()
        .find(|character| character.id == XIN_GONGSUNZAN)
        .unwrap();
    assert_eq!(character.skills, vec![XINYICONG, QIAOMENG]);
}

#[test]
fn filter_requires_a_black_logical_sha_and_discardable_equipment() {
    for (kind, suit, protected) in [
        (CardKind::Sha, Suit::Heart, false),
        (CardKind::TrickHuogong, Suit::Spade, false),
        (CardKind::Sha, Suit::Club, true),
    ] {
        let (state, owner, target, card, _) = setup(kind, suit, CardKind::Weapon, protected);
        let (state, result) = damage(state, owner, target, card, kind);
        assert!(result.is_ok());
        assert!(!state.has_pending_choice());
    }
}

#[test]
fn declining_optional_trigger_leaves_the_equipment_untouched() {
    let (state, owner, target, sha, equipment) =
        setup(CardKind::Sha, Suit::Club, CardKind::Weapon, false);
    let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let original_zone = state.cards.get(equipment).unwrap().zone;
    let (state, declined) = resolve(state, owner, vec![0], false, 10);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(equipment).unwrap().zone, original_zone);
}

#[test]
fn forced_choice_rejects_cancel_and_revalidates_the_equipment() {
    let (state, owner, target, equipment) = open_equipment_choice(CardKind::Weapon);
    let (_, cancelled) = resolve(state.clone(), owner, Vec::new(), true, 20);
    assert!(matches!(cancelled.result, CommandResult::Rejected { .. }));

    let mut reprotected = Transaction::begin(state.clone());
    apply_effect(
        &mut reprotected,
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(PROTECT),
        },
    )
    .unwrap();
    let (_, protected) = resolve(
        reprotected.commit().state,
        owner,
        vec![equipment.raw()],
        false,
        21,
    );
    assert!(matches!(protected.result, CommandResult::Rejected { .. }));

    let mut moved = Transaction::begin(state);
    let equip = moved.state().cards.get(equipment).unwrap().zone;
    let discard = moved
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    move_one_card(&mut moved, equipment, equip, discard, PlayerId::NONE, true).unwrap();
    let (state, forged) = resolve(
        moved.commit().state,
        owner,
        vec![equipment.raw()],
        false,
        22,
    );
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
}

#[test]
fn non_mount_is_discarded_once_but_mount_is_discarded_then_gained() {
    let (state, owner, _, weapon) = open_equipment_choice(CardKind::Weapon);
    let (state, discarded) = resolve(state, owner, vec![weapon.raw()], false, 30);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    assert_eq!(
        discarded
            .events
            .iter()
            .filter(|event| matches!(event, DomainEvent::CardsMoved { .. }))
            .count(),
        1
    );

    let (state, owner, _, mount) = open_equipment_choice(CardKind::HorseOff);
    let (state, gained) = resolve(state, owner, vec![mount.raw()], false, 31);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(mount).unwrap().zone, hand);
    let moves: Vec<_> = gained
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsMoved { to, .. } => Some(*to),
            _ => None,
        })
        .collect();
    assert_eq!(moves, vec![discard, hand]);
}

#[test]
fn reqiaomeng_accepts_red_sha_and_discards_each_region_zone() {
    for (index, (zone, card_kind)) in [
        (ZoneKind::Hand, CardKind::Shan),
        (ZoneKind::Equip, CardKind::Weapon),
        (ZoneKind::Judge, CardKind::TrickLe),
    ]
    .into_iter()
    .enumerate()
    {
        let (state, owner, target, sha, target_card) =
            setup_reqiaomeng(CardKind::Sha, zone, card_kind);
        let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        let (state, accepted) = resolve(state, owner, vec![1], false, 100 + index as u64 * 2);
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::DiscardTargetCardGainMount
        );
        assert!(accepted
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .any(|option| option.token == target_card.raw()));
        let (state, discarded) = resolve(
            state,
            owner,
            vec![target_card.raw()],
            false,
            101 + index as u64 * 2,
        );
        assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(target_card).unwrap().zone, discard);
    }
}

#[test]
fn reqiaomeng_rejects_non_sha_but_gains_a_discarded_mount() {
    let (state, owner, target, trick, _) =
        setup_reqiaomeng(CardKind::TrickHuogong, ZoneKind::Hand, CardKind::Shan);
    let (state, result) = damage(state, owner, target, trick, CardKind::TrickHuogong);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let (state, owner, target, sha, mount) =
        setup_reqiaomeng(CardKind::Sha, ZoneKind::Equip, CardKind::HorseDef);
    let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, owner, vec![1], false, 120);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let (state, gained) = resolve(state, owner, vec![mount.raw()], false, 121);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(mount).unwrap().zone, hand);
}

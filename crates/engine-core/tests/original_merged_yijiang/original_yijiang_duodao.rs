//! Golden coverage for 潘璋马忠「夺刀」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DUODAO: u32 = 4_208_671_631;
const ANJIAN: u32 = 3_402_374_234;
const PANZHANGMAZHONG: u32 = 3_150_630_034;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/duodao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    cost: CardId,
    weapon: Option<CardId>,
    armor: CardId,
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
        suit: Suit::Club,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage_with_sha_and_pause(seed: u64, include_weapon: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;

    let mut setup: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|player| tx.state().players.get(player).unwrap().skills.clone())
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    setup.extend([
        EffectRequest::DiscardFromHand {
            player: source,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DUODAO),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let cost = add_card(
        &mut tx,
        owner,
        ZoneKind::Hand,
        DUODAO + 1,
        CardKind::TrickWuzhong,
    );
    let sha = add_card(&mut tx, source, ZoneKind::Hand, DUODAO + 2, CardKind::Sha);
    let weapon = include_weapon.then(|| {
        add_card(
            &mut tx,
            source,
            ZoneKind::Equip,
            DUODAO + 3,
            CardKind::Weapon,
        )
    });
    let armor = add_card(
        &mut tx,
        source,
        ZoneKind::Equip,
        DUODAO + 4,
        CardKind::Armor,
    );

    let (state, pipeline) = apply_command(
        tx.commit().state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed + 1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 9);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardOneThenGainTargetEquipment
    );
    assert_eq!(
        state.resolution.continuation_payload,
        vec![owner.raw(), source.raw(), 1]
    );
    Setup {
        state,
        source,
        owner,
        cost,
        weapon,
        armor,
    }
}

fn resolve(
    setup: Setup,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, CommandResult) {
    let choice_id = ChoiceId::new(setup.state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    (state, pipeline.result)
}

#[test]
fn package_publishes_complete_character_in_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DUODAO && skill.name == "duodao"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == PANZHANGMAZHONG)
        .unwrap();
    assert_eq!(character.skills, vec![DUODAO, ANJIAN]);
    let compiled = content_compiler::compile_source_named("duodao", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn valid_cost_is_discarded_then_weapon_is_gained_but_armor_stays() {
    let setup = damage_with_sha_and_pause(2_026_071_921, true);
    let owner = setup.owner;
    let source = setup.source;
    let cost = setup.cost;
    let weapon = setup.weapon.unwrap();
    let armor = setup.armor;
    let (state, result) = resolve(setup, vec![cost.raw()], false, 1);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let source_equip = state.zones.find(ZoneKind::Equip, source).unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_hand);
    assert_eq!(state.cards.get(weapon).unwrap().owner, owner);
    assert_eq!(state.cards.get(armor).unwrap().zone, source_equip);
    assert!(!state.has_pending_choice());
}

#[test]
fn cancelling_keeps_cost_and_weapon_in_place() {
    let setup = damage_with_sha_and_pause(2_026_071_922, true);
    let owner_hand = setup.state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let source_equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.source)
        .unwrap();
    let cost = setup.cost;
    let weapon = setup.weapon.unwrap();
    let (state, result) = resolve(setup, Vec::new(), true, 2);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(cost).unwrap().zone, owner_hand);
    assert_eq!(state.cards.get(weapon).unwrap().zone, source_equip);
    assert!(!state.has_pending_choice());
}

#[test]
fn paying_without_a_weapon_still_discards_the_cost() {
    let setup = damage_with_sha_and_pause(2_026_071_923, false);
    let cost = setup.cost;
    let armor = setup.armor;
    let source_equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.source)
        .unwrap();
    let (state, result) = resolve(setup, vec![cost.raw()], false, 3);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    assert_eq!(state.cards.get(armor).unwrap().zone, source_equip);
}

#[test]
fn forged_foreign_cost_is_rejected_atomically() {
    let setup = damage_with_sha_and_pause(2_026_071_924, true);
    let forged = setup.weapon.unwrap();
    let before = canonical_state_hash(&setup.state);
    let owner = setup.owner;
    let choice_id = ChoiceId::new(setup.state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        setup.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![forged.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    assert!(state.has_pending_choice());
}

//! Golden for SP 霍峻「穷守」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const QIONGSHOU: u32 = 2_792_246_343;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/qiongshou.skill");

fn package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("sp-qiongshou", SOURCE).unwrap()
}

fn isolated() -> (Transaction, PlayerId) {
    let package = package();
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.load_package(&package).unwrap();
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
        skill_def: SkillDefId::new(QIONGSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn package_declares_locked_game_start_rule_modifier_and_abi18() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 18);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == QIONGSHOU)
        .unwrap();
    assert!(skill.locked);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(package.trigger_index[0].event, "game_started");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_has_enabled_equip_slot"
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == QIONGSHOU
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value == 4
    }));
}

#[test]
fn game_start_disables_all_slots_discards_equipment_draws_four_and_raises_limit() {
    let (mut tx, owner) = isolated();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(weapon);
    let before_hand = tx.state().hand_count(owner);
    let before_limit = hand_limit(tx.state(), owner);
    assert_eq!(before_limit, tx.state().players.get(owner).unwrap().hp + 4);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.len() >= 2, "{effects:?}");
    assert!(
        matches!(
            &effects[..2],
            [
                EffectRequest::DisableEquipSlots { player, slots },
                EffectRequest::DrawCardsFromSkill { player: draw_player, count: 4, .. }
            ] if *player == owner && slots == &[1, 2, 3, 4, 5] && *draw_player == owner
        ),
        "{effects:?}"
    );
    let events = run_effects(&mut tx, effects).unwrap();

    assert_eq!(tx.state().hand_count(owner), before_hand + 4);
    assert_eq!(hand_limit(tx.state(), owner), before_limit);
    assert_eq!(
        tx.state().players.get(owner).unwrap().disabled_equip_slots,
        [1; 5]
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(weapon).unwrap().zone, discard);
    let disabled: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::EquipSlotDisabled { player, slot } if *player == owner => Some(*slot),
            _ => None,
        })
        .collect();
    assert_eq!(disabled, vec![1, 2, 3, 4, 5]);
}

#[test]
fn fully_disabled_owner_does_not_draw_again() {
    let (mut tx, owner) = isolated();
    tx.state_mut()
        .players
        .get_mut(owner)
        .unwrap()
        .disabled_equip_slots = [1; 5];
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(tx.state().hand_count(owner), before);
}

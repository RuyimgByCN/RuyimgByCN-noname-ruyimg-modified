//! Golden for OFFLINE「伏骑」response locking and per-far-target draws.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const YYFUQI: u32 = 3_281_236_781;
const JIAOZI: u32 = 2_415_897_133;
const YY_QUYI: u32 = 4_120_930_757;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn loaded() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(YYFUQI as u64),
        kind: CommandKind::StartGame {
            player_count: 6,
            rng_seed: YYFUQI as u64 + 1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let removals = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YYFUQI),
        }],
    )
    .unwrap();
    (tx, owner, seats)
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(91_001),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_locked_response_modifier_and_per_target_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let quyi = package
        .character_index
        .iter()
        .find(|character| character.id == YY_QUYI)
        .expect("yy_quyi must be published with a complete cross-package closure");
    assert_eq!(quyi.name_key, "yy_quyi");
    assert_eq!(quyi.max_hp, 4);
    assert_eq!(quyi.skills, [YYFUQI, JIAOZI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YYFUQI && skill.locked));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YYFUQI
            && modifier.target == "card_use_unrespondable_within_responder_to_owner_distance"
            && modifier.value == 1
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YYFUQI
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_source_target_distance_gt_one"
    }));
}

#[test]
fn near_targets_are_locked_and_each_far_target_draws_once() {
    let (mut tx, owner, seats) = loaded();
    let other: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let near: Vec<_> = other
        .iter()
        .copied()
        .filter(|target| distance(&tx, owner, *target) <= 1)
        .collect();
    let far: Vec<_> = other
        .iter()
        .copied()
        .filter(|target| distance(&tx, owner, *target) > 1)
        .collect();
    assert!(!near.is_empty());
    assert!(far.len() > 1);
    for target in &near {
        assert!(engine_core::query::card_use_unrespondable_within_distance(
            tx.state(),
            owner,
            *target
        ));
    }
    for target in &far {
        assert!(!engine_core::query::card_use_unrespondable_within_distance(
            tx.state(),
            owner,
            *target
        ));
    }

    let card = inject(&mut tx, owner);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + far.len() as u32);
}

#[test]
fn another_players_card_does_not_draw_for_the_owner() {
    let (mut tx, owner, seats) = loaded();
    let source = seats.into_iter().find(|player| *player != owner).unwrap();
    let card = inject(&mut tx, source);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}

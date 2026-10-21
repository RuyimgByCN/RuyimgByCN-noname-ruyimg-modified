//! Golden for OFFLINE equipment skill「白马骁骑」and its equipment-count tiers.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const BAIMAXIAOQI: u32 = 3_935_591_090;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn loaded() -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(BAIMAXIAOQI as u64),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: BAIMAXIAOQI as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    for player in tx.state().players.alive_by_seat() {
        let skills = tx.state().players.get(player).unwrap().skills.clone();
        for skill in skills {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BAIMAXIAOQI),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn int_query(tx: &Transaction, request: QueryRequest) -> i32 {
    match query(tx.state(), &request, &mut QueryContext::new()) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    int_query(tx, QueryRequest::AttackRange { player: owner })
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    int_query(tx, QueryRequest::Distance { from, to })
}

fn equip(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_000 + kind as u32),
        kind,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_all_four_equipment_count_modifiers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == BAIMAXIAOQI)
        .collect();
    assert_eq!(modifiers.len(), 4);
    assert!(modifiers.iter().any(|modifier| {
        modifier.target == "attack_range"
            && modifier.value_source.as_deref() == Some("owner_equip_count")
    }));
    for (target, threshold) in [
        ("sha_use_limit", 2),
        ("distance_from", 3),
        ("draw_count", 4),
    ] {
        assert!(modifiers.iter().any(|modifier| {
            modifier.target == target
                && modifier.value_source.as_deref()
                    == Some(format!("owner_equip_count_floor_{threshold}").as_str())
        }));
    }
}

#[test]
fn equipment_count_applies_each_legacy_threshold_without_early_bonus() {
    let (mut tx, owner) = loaded();
    let seats = tx.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let base_range = attack_range(&tx, owner);
    let base_sha = engine_core::query::sha_use_limit_for(tx.state(), owner);
    let base_distance = distance(&tx, owner, far);
    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);
    assert_eq!(attack_range(&tx, owner), base_range);
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        base_sha
    );
    assert_eq!(distance(&tx, owner, far), base_distance);
    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        base_draw
    );

    for (index, kind) in [
        CardKind::Weapon,
        CardKind::Armor,
        CardKind::HorseOff,
        CardKind::HorseDef,
    ]
    .into_iter()
    .enumerate()
    {
        equip(&mut tx, owner, kind);
        let count = index as i32 + 1;
        assert_eq!(attack_range(&tx, owner), base_range + count);
        assert_eq!(
            engine_core::query::sha_use_limit_for(tx.state(), owner),
            base_sha + if count >= 2 { count as u32 } else { 0 }
        );
        assert_eq!(
            distance(&tx, owner, far),
            if count >= 3 {
                (base_distance - count).max(1)
            } else {
                base_distance
            }
        );
        assert_eq!(
            engine_core::query::draw_count_for(tx.state(), owner),
            base_draw + if count >= 4 { count as u32 } else { 0 }
        );
    }
}

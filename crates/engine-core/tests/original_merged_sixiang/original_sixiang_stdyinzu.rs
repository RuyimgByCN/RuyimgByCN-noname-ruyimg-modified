//! Golden for 四象「荫族」按存活拥有者叠加的全场攻击范围光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STDYINZU: u32 = 3_812_447_605;
const STDDUANBING: u32 = 2_170_694_873;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDYINZU && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDYINZU as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDYINZU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDYINZU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn attack_range(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn equip_long_weapon(tx: &mut Transaction, player: PlayerId) {
    let weapon = tx.state().hand_cards(player)[0];
    {
        let card = tx.state_mut().cards.get_mut(weapon).unwrap();
        card.kind = CardKind::Weapon;
        card.weapon_range = 5;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn every_player_uses_their_own_hand_hp_condition() {
    let (mut tx, owner, equal, _) = isolated();
    let owner_hand = tx.state().hand_count(owner) as i32;
    let equal_hand = tx.state().hand_count(equal) as i32;
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_hand - 1;
    tx.state_mut().players.get_mut(equal).unwrap().hp = equal_hand;

    assert_eq!(attack_range(&tx, owner), 2);
    assert_eq!(attack_range(&tx, equal), 0);

    equip_long_weapon(&mut tx, equal);
    assert_eq!(attack_range(&tx, equal), 4);
}

#[test]
fn living_owners_stack_and_dead_owners_stop_contributing() {
    let (mut tx, first_owner, target, second_owner) = isolated();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(STDYINZU),
        }],
    )
    .unwrap();
    let target_hand = tx.state().hand_count(target) as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = target_hand - 1;

    assert_eq!(attack_range(&tx, target), 3);

    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(attack_range(&tx, target), 2);

    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(attack_range(&tx, target), 1);
}

#[test]
fn global_additions_compose_after_local_attack_range_override() {
    let (mut tx, _, target, second_owner) = isolated();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: second_owner,
                skill_def: SkillDefId::new(STDYINZU),
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(STDDUANBING),
            },
        ],
    )
    .unwrap();
    let target_hand = tx.state().hand_count(target) as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = target_hand - 1;

    assert_eq!(attack_range(&tx, target), 3);

    tx.state_mut().players.get_mut(target).unwrap().hp = target_hand;
    assert_eq!(attack_range(&tx, target), 0);
}

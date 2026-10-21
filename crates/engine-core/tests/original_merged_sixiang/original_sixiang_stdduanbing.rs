//! Golden for 四象「短兵」固定攻击范围与每回合首次【杀】伤害语义。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent_with_kind, DamageIntent};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STDDUANBING: u32 = 2_170_694_873;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDUANBING && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDDUANBING as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDDUANBING as u64,
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
        skill_def: SkillDefId::new(STDDUANBING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.max_hp = 10;
        state.hp = 10;
    }
    (tx, players[0], players[1], players[2])
}

fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn resolved_sha_amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    resolve_damage_intent_with_kind(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
        true,
    )
    .amount
}

fn deal_sha(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card: CardId::NONE,
            amount: 1,
            nature: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
}

#[test]
fn attack_range_remains_one_even_with_a_long_weapon() {
    let (mut tx, owner, _, _) = isolated();
    let weapon = tx.state().hand_cards(owner)[0];
    {
        let card = tx.state_mut().cards.get_mut(weapon).unwrap();
        card.kind = CardKind::Weapon;
        card.weapon_range = 5;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();

    assert_eq!(attack_range(&tx, owner), 1);
}

#[test]
fn only_the_owners_first_sha_damage_each_turn_gets_the_bonus() {
    let (mut tx, owner, target, other) = isolated();

    deal_sha(&mut tx, other, target);
    assert_eq!(resolved_sha_amount(&tx, owner, target), 2);

    deal_sha(&mut tx, owner, target);
    assert_eq!(resolved_sha_amount(&tx, owner, target), 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: 1,
        }],
    )
    .unwrap();
    assert_eq!(resolved_sha_amount(&tx, owner, target), 2);
}

#[test]
fn non_sha_damage_neither_gains_nor_consumes_the_bonus() {
    let (mut tx, owner, target, _) = isolated();
    let ordinary = resolve_damage_intent_with_kind(
        tx.state(),
        DamageIntent {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
        false,
    );
    assert_eq!(ordinary.amount, 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(resolved_sha_amount(&tx, owner, target), 2);
}

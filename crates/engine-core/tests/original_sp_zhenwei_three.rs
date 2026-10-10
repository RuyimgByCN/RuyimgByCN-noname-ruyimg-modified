//! Golden for SP 统率三军「镇卫」的阵营距离光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::Identity;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZHENWEI_THREE: u32 = 3_450_139_382;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHENWEI_THREE && skill.locked));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == ZHENWEI_THREE
            && modifier.target == "global_distance_to"));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(ZHENWEI_THREE as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: ZHENWEI_THREE as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(players[0]).unwrap().identity = Identity::Lord;
    tx.state_mut().players.get_mut(players[1]).unwrap().identity = Identity::Loyalist;
    tx.state_mut().players.get_mut(players[2]).unwrap().identity = Identity::Loyalist;
    tx.state_mut().players.get_mut(players[3]).unwrap().identity = Identity::Rebel;
    (tx, players[0], players[1], players[2], players[3])
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected distance integer, got {other:?}"),
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHENWEI_THREE),
        }],
    )
    .unwrap();
}

#[test]
fn living_allied_owners_stack_only_against_enemy_sources() {
    let (mut tx, protected, first_owner, second_owner, enemy) = isolated();
    let base = distance(&tx, enemy, protected);
    let ally_base = distance(&tx, first_owner, protected);
    add_skill(&mut tx, first_owner);
    assert_eq!(distance(&tx, enemy, protected), base + 1);
    assert_eq!(distance(&tx, first_owner, protected), ally_base);

    add_skill(&mut tx, second_owner);
    assert_eq!(distance(&tx, enemy, protected), base + 2);
    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(distance(&tx, enemy, protected), base + 1);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(distance(&tx, enemy, protected), base);
}

#[test]
fn an_owner_does_not_protect_self_but_is_protected_by_another_owner() {
    let (mut tx, protected, first_owner, second_owner, enemy) = isolated();
    let self_base = distance(&tx, enemy, first_owner);
    add_skill(&mut tx, first_owner);
    assert_eq!(distance(&tx, enemy, first_owner), self_base);

    add_skill(&mut tx, second_owner);
    assert_eq!(distance(&tx, enemy, first_owner), self_base + 1);

    let protected_base = distance(&tx, enemy, protected);
    add_skill(&mut tx, protected);
    assert_eq!(distance(&tx, enemy, protected), protected_base);
}

#[test]
fn enemy_owned_skill_does_not_protect_the_opposing_team() {
    let (mut tx, protected, _, _, enemy) = isolated();
    let base = distance(&tx, enemy, protected);
    add_skill(&mut tx, enemy);
    assert_eq!(distance(&tx, enemy, protected), base);
}

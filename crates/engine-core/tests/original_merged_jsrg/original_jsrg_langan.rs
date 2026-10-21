//! Golden for JSRG 诸葛诞「阑干」的死亡后回复、摸牌和攻击范围递减语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const LANGAN: u32 = 3_013_648_901;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == LANGAN)
        .map_or(0, |(_, count)| *count)
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

#[test]
fn other_deaths_recover_draw_and_reduce_range_with_a_three_mark_cap() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LANGAN
            && trigger.event == "death"
            && trigger.filter == "owner_alive"
            && !trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(LANGAN as u64),
            kind: CommandKind::StartGame {
                player_count: 5,
                rng_seed: LANGAN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let victims: Vec<_> = engine
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
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
        skill_def: SkillDefId::new(LANGAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(owner);
    let base_range = attack_range(&tx, owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victims[0],
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(mark_count(&tx, owner), 1);
    assert_eq!(attack_range(&tx, owner), (base_range - 1).max(0));

    for victim in victims.iter().skip(1) {
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: *victim,
                killer: owner,
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(&tx, owner), 3);
    assert_eq!(tx.state().hand_count(owner), hand + 8);
}

//! Golden for DIY「弑君」的伤害增加与结算后失去体力语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NSSHJUN: u32 = 3_960_662_326;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/diy/diy.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSSHJUN && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
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
        skill_def: SkillDefId::new(NSSHJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn hp(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn shijun_adds_one_to_damage_then_owner_loses_one_hp() {
    let (mut tx, owner, other) = isolated_state();
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    let owner_hp = hp(tx.state(), owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(hp(tx.state(), other), 8);
    assert_eq!(hp(tx.state(), owner), owner_hp - 1);
}

#[test]
fn lose_hp_is_neither_increased_nor_followed_by_shijun_hp_loss() {
    let (mut tx, owner, other) = isolated_state();
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    let owner_hp = hp(tx.state(), owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(hp(tx.state(), other), 9);
    assert_eq!(hp(tx.state(), owner), owner_hp);
}

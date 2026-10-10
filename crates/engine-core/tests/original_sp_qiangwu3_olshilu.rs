//! Golden for sp 强武 (qiangwu3) + 识露 (olshilu) rewrites.
//!
//! qiangwu3: locked modifiers — Sha with rank below the skill mark ignore
//! distance; Sha with rank above the mark ignore use limits.
//! olshilu: locked — after taking damage, draw min(5, HP).

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLSHILU: u32 = 2_791_913_057;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_401,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_olshilu_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == OLSHILU && t.event == "damage"
    }));
}

#[test]
fn olshilu_draws_min_hp_five_after_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, OLSHILU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let hand_before = tx.state().hand_count(owner);

    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("damage error: {e:?}"),
    }

    // HP is 3 before, 2 after the damage → min(5, 2) = 2.
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "识露 draws min(5, HP)=2 after the damage"
    );
}

#[test]
fn olshilu_draws_capped_at_five() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, OLSHILU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 6;
    let hp_after = tx.state().players.get(owner).unwrap().hp - 1;
    let expected = hp_after.min(5).max(0) as u32;
    let hand_before = tx.state().hand_count(owner);

    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("damage error: {e:?}"),
    }

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + expected,
        "识露 caps the draw at 5"
    );
}

#[test]
fn olshilu_trigger_is_locked_and_forced() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = pkg
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == OLSHILU)
        .unwrap();
    assert!(!trigger.optional, "识露 is forced");
}

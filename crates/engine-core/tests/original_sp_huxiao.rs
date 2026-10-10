//! Golden coverage for sp 虎啸 (huxiao): locked skill that, after the owner deals
//! fire damage, makes the damaged player draw 1 card.
//!
//! Exercises the `owner_source_fire` damage filter and the `event_player` draw
//! target. The upstream huxiao3 temp-skill (no card-use limit against the damaged
//! player this turn) is not ported; see the .skill comment.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HUXIAO: u32 = 3_796_089_056;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_818,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
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

fn add_huxiao(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(HUXIAO),
    });
    run_effects(tx, effects).unwrap();
}

fn deal_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature,
        }],
    )
    .unwrap_or_else(|e| panic!("damage error: {e:?}"));
}

#[test]
fn committed_package_publishes_huxiao_fire_damage_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HUXIAO
            && trigger.event == "damage"
            && trigger.filter == "owner_source_fire"
    }));
}

#[test]
fn fire_damage_makes_the_damaged_player_draw_one() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_huxiao(&mut tx, owner);
    let other_hand_before = tx.state().hand_count(other);

    deal_damage(&mut tx, owner, other, 1, 1 /* fire */);

    assert_eq!(
        tx.state().hand_count(other),
        other_hand_before + 1,
        "fire-damaged player draws 1"
    );
}

#[test]
fn non_fire_damage_draws_nothing() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_huxiao(&mut tx, owner);
    let other_hand_before = tx.state().hand_count(other);

    deal_damage(&mut tx, owner, other, 1, 0 /* normal */);

    assert_eq!(
        tx.state().hand_count(other),
        other_hand_before,
        "normal damage does not trigger 虎啸"
    );
}

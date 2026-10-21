//! Golden coverage for sp2 藏储 (cangchu): locked skill that grants 3 「粮」 marks
//! at game start and removes one mark per point of fire damage the owner takes.
//!
//! Exercises the `game_started` event trigger and the `add_mark`/`remove_mark`
//! effects with the damage event amount. The upstream custom `cangchuAwaken`
//! event (wired to 良营) is not ported; see the .skill comment.

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CANGCHU: u32 = 2_861_531_568;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_816,
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

fn add_cangchu(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(CANGCHU),
    });
    run_effects(tx, effects).unwrap();
}

fn marks_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == CANGCHU))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

fn fire_game_started(tx: &mut Transaction, player_count: u8) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::GameStarted { player_count }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn deal_damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, amount: u32, nature: u8) {
    let events = run_effects(
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
    // Damage triggers fire through the damage event the engine already dispatched;
    // the trigger pipeline ran inside run_effects, so no manual dispatch is needed.
    let _ = events;
}

#[test]
fn committed_package_publishes_cangchu_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let events: Vec<&str> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CANGCHU)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(events.contains(&"game_started"));
    assert!(events.contains(&"damage"));
}

#[test]
fn game_start_grants_three_marks_and_fire_damage_removes_them() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_cangchu(&mut tx, owner);

    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner), 3, "game start grants 3 marks");

    // Fire damage 1 → 3 - 1 = 2.
    deal_damage(&mut tx, other, owner, 1, 1 /* fire */);
    assert_eq!(marks_of(&tx, owner), 2, "1 fire damage removes 1 mark");

    // Fire damage 2 → marks exhausted.
    deal_damage(&mut tx, other, owner, 2, 1);
    assert_eq!(marks_of(&tx, owner), 0, "remaining marks all removed");
}

#[test]
fn non_fire_damage_does_not_remove_marks() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_cangchu(&mut tx, owner);

    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner), 3);

    deal_damage(&mut tx, other, owner, 1, 0 /* normal */);
    assert_eq!(
        marks_of(&tx, owner),
        3,
        "normal damage does not touch 粮 marks"
    );
}

//! Golden for sp 雍嗣·摸 (yongsi1) + 三哭 (sanku) rewrites.
//!
//! yongsi1: locked — during the draw phase, draw `alive_group_count()` extra cards.
//! sanku: locked — when entering dying, lose 1 max HP then recover to max HP.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YONGSI1: u32 = 3_190_419_615;
const SANKU: u32 = 2_924_465_879;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start(player_count: u8) -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count,
            rng_seed: 20_261_501,
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

fn end_phase(state: engine_core::state::GameState, actor: PlayerId, key: u64) -> (engine_core::state::GameState, engine_api::CommandResult) {
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    );
    (state, pipe.result)
}

#[test]
fn committed_package_publishes_yongsi1_and_sanku_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == YONGSI1 && t.event == "phase_draw"
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == SANKU && t.event == "dying"
    }));
}

#[test]
fn yongsi1_draws_alive_group_count_extra_cards() {
    let (engine, owner, _other) = start(4);
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, YONGSI1);
    tx.state_mut().turn.current = owner;
    // Start at Judge so EndPhase transitions INTO the draw phase (where the bonus applies).
    tx.state_mut().turn.phase = Phase::Judge;
    let before = tx.state().hand_count(owner);
    // Standard 4-player start: four distinct factions (wei/shu/wu/qun).
    let expected_extra = engine_core::query::alive_group_count(tx.state()) as u32;
    assert!(expected_extra >= 3, "at least three distinct factions at a 4-player start");

    let (state, result) = end_phase(tx.commit().state, owner, 1);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        before + 2 + expected_extra,
        "base 2 + alive_group_count extra cards drawn"
    );
}

#[test]
fn sanku_reduces_max_hp_and_recovers_to_max_when_dying() {
    let (engine, owner, other) = start(2);
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, SANKU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    assert!(max_hp_before >= 3);

    // Deal 1 damage → owner enters dying → sanku loses 1 max HP and recovers fully.
    let damage_result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage_result {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("unexpected damage error: {e:?}"),
    }

    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(
        player.max_hp,
        max_hp_before - 1,
        "sanku loses 1 max HP on dying"
    );
    assert_eq!(
        player.hp,
        player.max_hp as i32,
        "sanku recovers the owner to full HP"
    );
    assert!(player.alive, "recovered from dying");
    assert!(!player.dying, "no longer dying after recovery");
}

#[test]
fn sanku_recovers_after_heavy_damage_through_dying() {
    let (engine, owner, other) = start(2);
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, SANKU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;

    let damage_result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage_result {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("unexpected damage error: {e:?}"),
    }

    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(player.max_hp, max_hp_before - 1);
    assert_eq!(player.hp, player.max_hp as i32, "healed back to the reduced max HP");
    assert!(player.alive);
}

//! Golden for sp 凶祸 (xinfu_xionghuo) + 神甲 (olshanjia) rewrites.
//!
//! xinfu_xionghuo: 3 「暴戾」 marks at game start; may give one to another player
//! (play phase); +1 damage to marked players.
//! olshanjia: 3 「损」 marks at game start; play-phase draw 3 (once per turn).

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XIONGHUO: u32 = 3_787_720_298;
const SHANJIA: u32 = 3_933_707_186;
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
            rng_seed: 20_261_801,
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

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
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
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn fire_game_started(tx: &mut Transaction, count: u8) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::GameStarted { player_count: count }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn marks_of(tx: &Transaction, player: PlayerId, def: u32) -> i32 {
    tx.state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == def))
        .map(|(_, c)| *c)
        .unwrap_or(0)
}

#[test]
fn committed_package_publishes_both_skills() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == XIONGHUO && t.event == "game_started"
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == XIONGHUO && t.event == "damage_before"
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == SHANJIA && t.event == "game_started"
    }));
}

#[test]
fn xionghuo_grants_gives_and_boosts_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_skill(&mut tx, owner, XIONGHUO);
    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner, XIONGHUO), 3, "开局 3 枚暴戾");

    // Give one mark to the other player.
    tx.state_mut().turn.phase = Phase::Play;
    let (state, gave) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(gave.result, CommandResult::Accepted { .. }),
        "give-mark accepted: {:?}",
        gave.result
    );
    let mut tx = Transaction::begin(state);
    assert_eq!(marks_of(&tx, owner, XIONGHUO), 2);
    assert_eq!(marks_of(&tx, other, XIONGHUO), 1);

    // Damage the marked other: +1.
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
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
        tx.state().players.get(other).unwrap().hp,
        other_hp - 2,
        "marked target takes +1 damage"
    );
}

#[test]
fn xionghuo_does_not_boost_unmarked_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = add_skill(&mut tx, owner, XIONGHUO);
    fire_game_started(&mut tx, 2);
    let other_hp = tx.state().players.get(other).unwrap().hp;

    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
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
        tx.state().players.get(other).unwrap().hp,
        other_hp - 1,
        "unmarked target takes normal damage"
    );
}

#[test]
fn shanjia_grants_marks_and_draws_three() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_skill(&mut tx, owner, SHANJIA);
    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner, SHANJIA), 3, "开局 3 枚损");

    tx.state_mut().turn.phase = Phase::Play;
    let hand_before = tx.state().hand_count(owner);
    let (state, drew) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(drew.result, CommandResult::Accepted { .. }),
        "神甲 draw accepted: {:?}",
        drew.result
    );
    assert_eq!(state.hand_count(owner), hand_before + 3, "摸三张");
}

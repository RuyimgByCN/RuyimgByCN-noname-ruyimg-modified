//! Golden coverage for JSRG 孙策「猘横」「诈死」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGZHIHENG: u32 = 2_952_364_897;
const JSRGZHASI: u32 = 2_253_086_418;
const ZHIHENG_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgzhiheng.skill");
const ZHASI_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgzhasi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn zhiheng_source_and_npack_publish_outgoing_plus_one() {
    let compiled = content_compiler::compile_source_named("jsrgzhiheng", ZHIHENG_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHIHENG
            && trigger.event == "damage_before"
            && trigger.filter == "owner_source_other_damage_before"
            && !trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHIHENG && trigger.event == "damage_before"
    }));
}

#[test]
fn zhiheng_increases_outgoing_damage_by_one() {
    let (tx, owner, other) = bootstrap(ZHIHENG_SRC, "jsrgzhiheng-test", JSRGZHIHENG);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
}

#[test]
fn zhasi_source_and_npack_publish_optional_prevent() {
    let compiled = content_compiler::compile_source_named("jsrgzhasi", ZHASI_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHASI && trigger.event == "damage_before" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHASI && trigger.event == "damage_before"
    }));
}

#[test]
fn zhasi_can_prevent_incoming_damage_once() {
    let (tx, owner, other) = bootstrap(ZHASI_SRC, "jsrgzhasi-test", JSRGZHASI);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}

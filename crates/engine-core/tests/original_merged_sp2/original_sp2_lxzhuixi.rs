//! Golden for SP2「追袭」按伤害双方翻面状态双向加伤的语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const LXZHUIXI: u32 = 2_387_115_210;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LXZHUIXI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(LXZHUIXI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: LXZHUIXI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    // Roster-independent: remove every booted skill, draining any
    // optional-trigger choices deal-dependent skills leave behind, so later
    // DealDamage calls in these tests are choice-free.
    let mut key = 9_000u64;
    loop {
        if !tx.state().skills.sorted_live().is_empty() {
            let effects: Vec<_> = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
                .collect();
            match run_effects(&mut tx, effects) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("effect error: {e:?}"),
            }
        }
        if tx.state().resolution.pending_choice_id == 0 {
            break;
        }
        let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
        let (next, _) = apply_command(
            tx.state().clone(),
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(tx.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        tx = Transaction::begin(next);
        key += 1;
    }
    (tx, players[0], players[1], players[2])
}

fn add_lxzhuixi(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LXZHUIXI),
        }],
    )
    .unwrap();
}

fn resolved_amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    resolve_damage_intent(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
    .amount
}

fn dealt_amount(tx: &mut Transaction, source: PlayerId, target: PlayerId) -> i32 {
    let before = tx.state().players.get(target).unwrap().hp;
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    before - tx.state().players.get(target).unwrap().hp
}

#[test]
fn source_side_bonus_requires_different_turned_over_states() {
    let (mut tx, source, target, _) = isolated_state();
    add_lxzhuixi(&mut tx, source);

    assert_eq!(resolved_amount(&tx, source, target), 1);
    tx.state_mut().players.get_mut(target).unwrap().turned_over = true;
    assert_eq!(resolved_amount(&tx, source, target), 2);
}

#[test]
fn target_side_bonus_does_not_apply_to_unrelated_or_sourceless_damage() {
    let (mut tx, source, target, third) = isolated_state();
    add_lxzhuixi(&mut tx, target);
    tx.state_mut().players.get_mut(source).unwrap().turned_over = true;
    // Fixed HP so the two-damage hits never open a dying window, independent
    // of the booted roster's max_hp.
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(target).unwrap().hp = 5;

    assert_eq!(dealt_amount(&mut tx, source, target), 2);
    assert_eq!(dealt_amount(&mut tx, source, third), 1);
    assert_eq!(dealt_amount(&mut tx, PlayerId::NONE, target), 1);
}

#[test]
fn source_and_target_owners_each_contribute_once() {
    let (mut tx, source, target, _) = isolated_state();
    add_lxzhuixi(&mut tx, source);
    add_lxzhuixi(&mut tx, target);
    tx.state_mut().players.get_mut(target).unwrap().turned_over = true;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(target).unwrap().hp = 5;

    assert_eq!(dealt_amount(&mut tx, source, target), 3);
}

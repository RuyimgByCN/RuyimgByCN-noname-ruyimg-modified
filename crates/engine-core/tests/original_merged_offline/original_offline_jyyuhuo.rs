//! Golden for OFFLINE「浴火」按存活拥有者叠加的横置角色最终受伤光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{damage_taken_amount_for, damage_taken_final_amount_for};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const JYYUHUO: u32 = 2_185_491_588;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYYUHUO && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JYYUHUO as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: JYYUHUO as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(JYYUHUO),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn set_linked(tx: &mut Transaction, player: PlayerId, linked: bool) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    if linked {
        marks.push((engine_core::content::MARK_CHAIN, 1));
        marks.sort_by_key(|(mark, _)| *mark);
    }
}

fn final_amount(tx: &Transaction, target: PlayerId, nature: u8, base: u32) -> u32 {
    damage_taken_final_amount_for(tx.state(), target, nature, base)
}

#[test]
fn linked_other_players_gain_elemental_and_lose_normal_damage() {
    let (mut tx, owner, target, _) = isolated();
    assert_eq!(final_amount(&tx, target, damage_nature::NORMAL, 2), 2);
    set_linked(&mut tx, target, true);

    assert_eq!(final_amount(&tx, target, damage_nature::NORMAL, 2), 1);
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 2);
    assert_eq!(final_amount(&tx, target, damage_nature::THUNDER, 1), 2);

    set_linked(&mut tx, owner, true);
    assert_eq!(final_amount(&tx, owner, damage_nature::FIRE, 1), 1);
}

#[test]
fn living_owners_stack_and_dead_owners_stop_contributing() {
    let (mut tx, first_owner, target, second_owner) = isolated();
    set_linked(&mut tx, target, true);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(JYYUHUO),
        }],
    )
    .unwrap();

    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 3);
    assert_eq!(final_amount(&tx, target, damage_nature::NORMAL, 2), 0);
    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 2);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 1);
}

#[test]
fn aura_runs_only_in_the_final_incoming_damage_stage() {
    let (mut tx, _, target, source) = isolated();
    set_linked(&mut tx, target, true);

    assert_eq!(
        damage_taken_amount_for(tx.state(), source, target, damage_nature::FIRE, 1),
        1
    );
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 2);
}

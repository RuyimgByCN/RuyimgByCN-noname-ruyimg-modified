//! Golden for 四象「祸水」按存活拥有者叠加的判定区受伤光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::damage_taken_amount_for;
use engine_core::state::ZoneKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const STDHUOSHUI: u32 = 3_517_180_722;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDHUOSHUI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDHUOSHUI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDHUOSHUI as u64,
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
        skill_def: SkillDefId::new(STDHUOSHUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn add_judge_card(tx: &mut Transaction, player: PlayerId) {
    let card = tx.state().hand_cards(player)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: judge,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    damage_taken_amount_for(tx.state(), source, target, damage_nature::NORMAL, 1)
}

#[test]
fn judge_zone_condition_applies_to_other_targets_and_sourceless_damage() {
    let (mut tx, owner, target, source) = isolated();
    assert_eq!(amount(&tx, source, target), 1);

    add_judge_card(&mut tx, target);
    assert_eq!(amount(&tx, source, target), 2);
    assert_eq!(amount(&tx, PlayerId::NONE, target), 2);

    add_judge_card(&mut tx, owner);
    assert_eq!(amount(&tx, source, owner), 1);
}

#[test]
fn living_owners_stack_and_dead_owners_stop_contributing() {
    let (mut tx, first_owner, target, second_owner) = isolated();
    add_judge_card(&mut tx, target);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(STDHUOSHUI),
        }],
    )
    .unwrap();

    assert_eq!(amount(&tx, PlayerId::NONE, target), 3);
    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(amount(&tx, PlayerId::NONE, target), 2);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(amount(&tx, PlayerId::NONE, target), 1);
}

#[test]
fn aura_is_in_the_ordinary_incoming_damage_stage() {
    let (mut tx, _, target, source) = isolated();
    add_judge_card(&mut tx, target);

    assert_eq!(amount(&tx, source, target), 2);
    assert_eq!(
        engine_core::query::damage_taken_final_amount_for(
            tx.state(),
            target,
            damage_nature::NORMAL,
            1,
        ),
        1
    );
}

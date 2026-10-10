//! Golden for OFFLINE「策算」的伤害防止与体力上限联动。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSCESUAN: u32 = 3_127_862_270;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn prepared(full_hp: bool, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let owner = tx.state().turn.current;
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 4;
    player.hp = if full_hp { 4 } else { 2 };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSCESUAN),
        }],
    )
    .unwrap();
    (tx, owner, source)
}

#[test]
fn package_exposes_locked_damage_before_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSCESUAN && skill.locked));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NSCESUAN
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
}

#[test]
fn prevents_damage_and_draws_only_when_current_hp_falls_with_max_hp() {
    for (full_hp, seed, expected_hp, expected_draw) in
        [(true, 2_026_071_602, 3, 1), (false, 2_026_071_603, 2, 0)]
    {
        let (mut tx, owner, source) = prepared(full_hp, seed);
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
        let player = tx.state().players.get(owner).unwrap();
        assert_eq!((player.max_hp, player.hp), (3, expected_hp));
        assert_eq!(tx.state().hand_count(owner), hand + expected_draw);
    }
}

//! Golden for 一将成名「矢北」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHIBEI: u32 = 2_648_030_702;
const JIANYING: u32 = 3_679_198_172;
const YJ_JUSHOU: u32 = 3_182_131_819;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

#[test]
fn package_contains_locked_first_and_later_damage_branches() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YJ_JUSHOU)
        .expect("committed 沮授 character");
    assert_eq!(character.name_key, "yj_jushou");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.initial_hp, 0, "omitted means start at max_hp");
    assert_eq!(character.skills, [JIANYING, SHIBEI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SHIBEI && skill.locked));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SHIBEI)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_first_damage",
            "owner_is_target_after_first_damage"
        ]
    );
}

#[test]
fn first_damage_recovers_and_later_damage_loses_hp_in_the_same_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716_20),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716_20,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHIBEI),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 6;
        player.hp = 5;
    }

    let damage = || EffectRequest::DealDamage {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: 0,
    };
    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);

    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SHIBEI)
        .unwrap();
    assert_eq!(skill.turn_fire_count, 2);
}

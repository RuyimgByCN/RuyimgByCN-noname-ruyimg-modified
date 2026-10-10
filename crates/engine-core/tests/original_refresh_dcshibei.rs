//! Golden for 界限突破「矢北」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCSHIBEI: u32 = 2_817_420_663;
const RE_JUSHOU: u32 = 3_303_423_936;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn package_contains_locked_first_and_second_damage_branches_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DCSHIBEI && skill.locked));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == DCSHIBEI)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_first_damage",
            "owner_is_target_second_damage"
        ]
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_JUSHOU && character.skills == [2_862_591_465, DCSHIBEI]
    }));
}

#[test]
fn only_first_and_second_damage_events_have_follow_up_effects() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716_21),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716_21,
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
        skill_def: SkillDefId::new(DCSHIBEI),
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

    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCSHIBEI)
        .unwrap();
    assert_eq!(skill.turn_fire_count, 2);
}

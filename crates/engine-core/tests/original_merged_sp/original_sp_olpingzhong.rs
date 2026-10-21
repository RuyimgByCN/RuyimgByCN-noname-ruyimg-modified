//! Golden coverage for SP OL 杨阜「评忠」(olpingzhong).
//! After taking damage the owner draws 1.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLPINGZHONG: u32 = 2_762_208_431;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/olpingzhong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olpingzhong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_12, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
        skill_def: SkillDefId::new(OLPINGZHONG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn compiled_source_and_npack_publish_the_damage_draw() {
    let compiled = content_compiler::compile_source_named("olpingzhong", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == OLPINGZHONG && skill.name == "olpingzhong" && skill.locked
    }));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLPINGZHONG)
        .expect("olpingzhong trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLPINGZHONG)
        .expect("npack olpingzhong trigger");
    assert_eq!(pack.event, "damage");
    assert_eq!(pack.filter, "owner_is_target");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for olpingzhong");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn incoming_damage_draws_one() {
    let (state, owner, other) = setup();
    let before = state.hand_count(owner);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

//! Golden coverage for SP 司马徽「隐士」(xinfu_yinshi).
//! Incoming damage_before prevents the current damage.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_YINSHI: u32 = 3_413_758_271;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/xinfu_yinshi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("xinfu_yinshi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_04, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(XINFU_YINSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn compiled_source_and_npack_publish_the_prevent_window() {
    let compiled = content_compiler::compile_source_named("xinfu_yinshi", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == XINFU_YINSHI && skill.name == "xinfu_yinshi" && skill.locked
    }));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_YINSHI)
        .expect("xinfu_yinshi trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_YINSHI)
        .expect("npack xinfu_yinshi trigger");
    assert_eq!(pack_trigger.event, "damage_before");
    assert_eq!(pack_trigger.filter, "owner_is_target");
    let actual = committed
        .modules
        .iter()
        .find(|module| compiled.modules.iter().any(|donor| donor.name == module.name))
        .expect("npack module for xinfu_yinshi");
    let compiled_module = compiled
        .modules
        .iter()
        .find(|module| module.name == actual.name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled_module.logic_hash);
}

#[test]
fn incoming_damage_is_prevented() {
    let (state, owner, other) = setup();
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let events = run_effects(
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
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamagePrevented {
            target,
            ..
        } if *target == owner
    )));
}

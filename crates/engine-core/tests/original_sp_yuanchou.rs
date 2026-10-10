//! Golden coverage for SP 范疆张达「远筹」(yuanchou).
//! Locked ignore_armor: the owner's cards bypass the target's armor.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{apply_int_modifiers_for_state, ModifierTarget};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YUANCHOU: u32 = 3_016_188_154;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/yuanchou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("yuanchou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_02, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(YUANCHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn source_ignores_armor(state: &GameState, source: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::IgnoreArmor);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

#[test]
fn compiled_source_and_npack_publish_ignore_armor() {
    let compiled = content_compiler::compile_source_named("yuanchou", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == YUANCHOU && skill.name == "yuanchou" && skill.locked
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YUANCHOU && modifier.target == "ignore_armor"
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YUANCHOU && skill.name == "yuanchou"));
    let pack = committed
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == YUANCHOU)
        .expect("npack yuanchou modifier");
    assert_eq!(pack.target, "ignore_armor");
    assert_eq!(pack.mode, "override");
    assert_eq!(pack.value, 1);
}

#[test]
fn owner_ignores_armor_other_does_not() {
    let (state, owner, other) = setup();
    assert!(source_ignores_armor(&state, owner));
    assert!(!source_ignores_armor(&state, other));
}

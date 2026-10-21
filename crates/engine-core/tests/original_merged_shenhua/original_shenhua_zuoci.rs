//! Golden coverage for 化身/新生（shenhua 左慈，ABI 591，文档化近似：元数据登记）。

use engine_api::SkillDefId;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HUASHEN: u32 = 1_234_567_909;
const XINSHENG: u32 = 1_234_567_910;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/huashen.skill");

#[test]
fn package_exposes_zuoci_contract() {
    let compiled = content_compiler::compile_source_named("zuoci", SOURCE).unwrap();
    // 元数据技能不携带任何效果门控 → 基线 ABI。
    assert_eq!(compiled.manifest.rule_vm_abi_version, 3);
    assert_eq!(compiled.skill_index.len(), 2);
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&HUASHEN)));
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&XINSHENG)));
}

#[test]
fn zuoci_skills_are_metadata_only() {
    let package = content_compiler::compile_source_named("zuoci-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUASHEN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINSHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    // 无运行时行为：技能可添加，且不破坏任何查询。
    assert!(tx
        .state()
        .players
        .get(owner)
        .is_some_and(|p| p.skills.iter().any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == HUASHEN)
        })));
    let state = tx.commit().state;
    let _ = Transaction::begin(state);
    let _ = EffectError::ChoiceRequested;
}

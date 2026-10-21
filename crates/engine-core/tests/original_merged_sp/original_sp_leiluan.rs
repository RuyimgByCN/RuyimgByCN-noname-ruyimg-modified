//! Golden coverage for SP 孔淑「累卵」(leiluan).
//! Hand/equip may be used as Sha, Shan, or Tao.

use engine_api::SkillDefId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LEILUAN: u32 = 3_861_314_381;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/leiluan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

#[test]
fn compiled_source_and_npack_publish_sha_shan_tao() {
    let compiled = content_compiler::compile_source_named("leiluan", SOURCE).unwrap();
    let mut kinds: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == LEILUAN)
        .map(|view| view.as_kind.as_str())
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["sha", "shan", "tao"]);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut pack: Vec<_> = committed
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == LEILUAN)
        .map(|view| view.as_kind.as_str())
        .collect();
    pack.sort();
    assert_eq!(pack, vec!["sha", "shan", "tao"]);
}

#[test]
fn skill_attaches_from_compiled_source() {
    let compiled = content_compiler::compile_source_named("leiluan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_34, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LEILUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LEILUAN));
}

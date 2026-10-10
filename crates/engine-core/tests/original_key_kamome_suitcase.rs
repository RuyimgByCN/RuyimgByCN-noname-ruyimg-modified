//! Golden for KEY 装备技能「旅行箱」的阶段与翻面取消语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::query::{skip_discard_phase, skip_judge_phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SUITCASE: u32 = 3_677_089_645;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

#[test]
fn suitcase_skips_both_phases_and_cancels_both_turn_over_directions() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 261);
    assert_eq!(package.modifier_index.len(), 172);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SUITCASE
            && modifier.target == "turn_over_cancel"
            && modifier.mode == "override"
            && modifier.value == 1
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(SUITCASE as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: SUITCASE as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(SUITCASE),
    });
    run_effects(&mut tx, effects).unwrap();

    assert!(skip_judge_phase(tx.state(), owner));
    assert!(skip_discard_phase(tx.state(), owner));
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: owner }]).unwrap();
    assert!(!tx.state().players.get(owner).unwrap().turned_over);

    tx.state_mut().players.get_mut(owner).unwrap().turned_over = true;
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: owner }]).unwrap();
    assert!(tx.state().players.get(owner).unwrap().turned_over);

    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == SUITCASE)
        .unwrap()
        .id;
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: owner }]).unwrap();
    assert!(!tx.state().players.get(owner).unwrap().turned_over);
}

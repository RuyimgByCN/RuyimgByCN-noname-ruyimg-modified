//! Golden for sp 骄巍 (oljiaowei): locked — black cards are excluded from the
//! owner's hand limit.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLJIAOWEI: u32 = 2_764_079_934;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_501,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

#[test]
fn package_declares_black_hand_limit_ignored_modifier() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.modifier_index.iter().any(|m| {
        m.skill_def_id == OLJIAOWEI
            && m.target == "card_ignored_hand_limit"
            && m.selector.as_deref() == Some("black")
            && m.mode == "override"
            && m.value == 1
    }));
}

#[test]
fn adding_oljiaowei_keeps_the_skill_locked() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIAOWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == OLJIAOWEI)
        .map(|skill| skill.id)
        .expect("oljiaowei installed");
    let skill = tx.state().skills.get(skill_id).unwrap();
    assert_eq!(skill.owner, owner);
    assert!(skill.enabled);
}

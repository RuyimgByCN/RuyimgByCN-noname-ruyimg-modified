//! Golden for sp batch-AE DSL rewrite:
//! 凉州 (pxliangzhou)：将一张黑色牌当【杀】使用。上游 (sp/skill.js:1572)。

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;

const PXLIANGZHOU: u32 = 2_433_321_161;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

#[test]
fn package_publishes_pxliangzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == PXLIANGZHOU)
        .expect("pxliangzhou black-sha view_as must be registered");
    assert_eq!(view_as.as_kind, "sha");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.filter, "black");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));
    // 凉州不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXLIANGZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxliangzhou_package_loads_in_engine() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2_026_081_901),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 2_026_081_901,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
}

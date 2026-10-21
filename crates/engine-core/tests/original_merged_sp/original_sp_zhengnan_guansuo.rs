//! Golden coverage for SP「征南」及关索武将闭包。

const ZHENGNAN: u32 = 3_435_011_424;
const XIEFANG: u32 = 3_450_775_931;
const GUANSUO: u32 = 3_505_703_155;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

#[test]
fn zhengnan_draws_before_missing_skill_choice_and_closes_guansuo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "onlyOL"));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHENGNAN)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    let draw = module
        .code
        .windows(2)
        .position(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    rule_vm::bytecode::effect_tag::DRAW_CARDS as u8,
                ]
        })
        .unwrap();
    let choose = module
        .code
        .windows(2)
        .position(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    rule_vm::bytecode::effect_tag::CHOOSE_ADD_MISSING_SKILL as u8,
                ]
        })
        .unwrap();
    assert!(draw < choose);
    assert!(package.character_index.iter().any(|character| {
        character.id == GUANSUO && character.skills == vec![ZHENGNAN, XIEFANG]
    }));
}

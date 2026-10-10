//! Cross-package character closure for HUICUI 孟优 and 界阚泽.

const MENGYOU: u32 = 3_321_725_449;
const MANYI: u32 = 3_456_707_526;
const DCMANZHI: u32 = 4_194_306_664;
const RE_KANZE: u32 = 3_301_308_945;
const XIASHU: u32 = 2_873_874_515;
const REKUANSHI: u32 = 3_423_208_328;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

#[test]
fn package_declares_dependencies_and_complete_skill_lists() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let dependencies: Vec<_> = package
        .manifest
        .dependencies
        .iter()
        .map(|dependency| dependency.package_id.as_str())
        .collect();
    assert!(dependencies.contains(&"xianding"));
    assert!(dependencies.contains(&"sp"));
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == MENGYOU)
            .unwrap()
            .skills,
        [MANYI, DCMANZHI]
    );
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == RE_KANZE)
            .unwrap()
            .skills,
        [XIASHU, REKUANSHI]
    );
}

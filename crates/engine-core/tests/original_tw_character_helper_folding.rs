//! Source-exact TW character skill lists without exposed implementation helpers.

const TW_ZHOUCHU: u32 = 3_263_858_015;
const TWGUOYI: u32 = 3_002_953_620;
const TWCHUHAI: u32 = 3_782_702_310;
const TW_QIAORUI: u32 = 3_792_078_394;
const TWXIAWEI: u32 = 3_007_668_730;
const TWQIONGJI: u32 = 2_723_606_029;
const OLD_TWCHUHAI_HELPER: u32 = 3_469_371_130;
const OLD_TWQIONGJI_HELPER: u32 = 3_469_371_140;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

#[test]
fn package_folds_helpers_into_source_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let skills = |id| {
        package
            .character_index
            .iter()
            .find(|character| character.id == id)
            .unwrap()
            .skills
            .as_slice()
    };
    assert_eq!(skills(TW_ZHOUCHU), [TWGUOYI, TWCHUHAI]);
    assert_eq!(skills(TW_QIAORUI), [TWXIAWEI, TWQIONGJI]);
    assert!(!package
        .skill_index
        .iter()
        .any(|skill| [OLD_TWCHUHAI_HELPER, OLD_TWQIONGJI_HELPER].contains(&skill.skill_def_id)));
}

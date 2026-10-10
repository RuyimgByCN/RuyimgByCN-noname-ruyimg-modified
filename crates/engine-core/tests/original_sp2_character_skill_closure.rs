//! Exact source character closure for three previously partial SP2 entries.

const DC_JSP_GUANYU: u32 = 4_047_577_994;
const NEW_REWUSHENG: u32 = 3_038_912_803;
const DCDANJI: u32 = 3_851_102_612;
const ZHANGWEN: u32 = 2_318_285_269;
const SONGSHU: u32 = 3_706_775_979;
const SIBIAN: u32 = 3_021_138_179;
const ZHUJUN: u32 = 2_614_075_935;
const GONGJIAN: u32 = 3_309_202_552;
const KUIMANG: u32 = 2_972_733_766;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

#[test]
fn package_preserves_all_source_character_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    let skills = |id| {
        package
            .character_index
            .iter()
            .find(|character| character.id == id)
            .unwrap()
            .skills
            .as_slice()
    };
    assert_eq!(skills(DC_JSP_GUANYU), [NEW_REWUSHENG, DCDANJI]);
    assert_eq!(skills(ZHANGWEN), [SONGSHU, SIBIAN]);
    assert_eq!(skills(ZHUJUN), [GONGJIAN, KUIMANG]);
}

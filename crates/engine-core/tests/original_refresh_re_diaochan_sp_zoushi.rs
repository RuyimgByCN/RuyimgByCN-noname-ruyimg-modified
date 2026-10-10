//! Character closure for REFRESH 界貂蝉 and SP 邹氏.

const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn committed_packages_publish_exact_legacy_character_definitions() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let diaochan = refresh
        .character_index
        .iter()
        .find(|character| character.id == 2_743_786_675)
        .expect("re_diaochan must be published");
    assert_eq!(diaochan.name_key, "re_diaochan");
    assert_eq!(diaochan.sex, "female");
    assert_eq!(diaochan.group, "qun");
    assert_eq!(diaochan.max_hp, 3);
    assert_eq!(diaochan.skills, [2_292_478_631, 4_285_757_447]);

    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let zoushi = sp
        .character_index
        .iter()
        .find(|character| character.id == 2_442_478_990)
        .expect("zoushi must be published");
    assert_eq!(zoushi.name_key, "zoushi");
    assert_eq!(zoushi.sex, "female");
    assert_eq!(zoushi.group, "qun");
    assert_eq!(zoushi.max_hp, 3);
    assert!(zoushi.skills.is_empty());
}

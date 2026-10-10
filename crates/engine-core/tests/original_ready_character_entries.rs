//! Golden coverage for extension characters whose complete skill dependencies are already ported.

#[test]
fn old_yujin_and_offline_lord_lvbu_keep_exact_character_metadata() {
    let old = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/old/old.npack.json"
    ))
    .unwrap();
    let yujin = old
        .character_index
        .iter()
        .find(|character| character.id == 3_388_457_974)
        .expect("毅重于禁 should be packaged");
    assert_eq!(yujin.name_key, "re_yujin");
    assert_eq!((yujin.sex.as_str(), yujin.group.as_str()), ("male", "wei"));
    assert_eq!(yujin.max_hp, 4);
    assert_eq!(yujin.skills, vec![2_156_296_205]);
    assert!(!yujin.is_lord);

    let offline = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/offline/offline.npack.json"
    ))
    .unwrap();
    let lvbu = offline
        .character_index
        .iter()
        .find(|character| character.id == 2_640_290_591)
        .expect("君吕布 should be packaged");
    assert_eq!(lvbu.name_key, "jun_lvbu");
    assert_eq!((lvbu.sex.as_str(), lvbu.group.as_str()), ("male", "qun"));
    assert_eq!(lvbu.max_hp, 4);
    assert_eq!(lvbu.skills, vec![18, 4_273_235_044]);
    assert!(lvbu.is_lord);
}

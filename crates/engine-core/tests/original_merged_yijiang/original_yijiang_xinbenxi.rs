//! Golden for 界吴懿「奔袭」rewritten from legacy JavaScript semantics.

const RE_WUYI: u32 = 3_817_550_013;
const XINBENXI: u32 = 3_506_100_401;
const XINBENXI_EFFECT: u32 = 4_250_961_579;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/xinbenxi.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn packages_publish_xinbenxi_policy_and_complete_re_wuyi() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(yijiang.character_index.len(), 72);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINBENXI && skill.locked));
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINBENXI_EFFECT));
    assert!(yijiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINBENXI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_single_target_sha_or_ordinary_trick_current_turn"
    }));
    assert!(yijiang.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINBENXI
            && modifier.target == "distance_from"
            && modifier.value_source.as_deref() == Some("owner_current_turn_card_use_count")
            && modifier.value == -1
    }));
    assert!(yijiang.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINBENXI_EFFECT
            && modifier.target == "ignore_armor_for_skill_card_policy"
    }));
    assert!(yijiang.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINBENXI_EFFECT
            && modifier.target == "card_use_unrespondable_for_skill_card_policy"
    }));
    assert!(yijiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINBENXI_EFFECT
            && trigger.event == "damage"
            && trigger.filter == "owner_source_event_card_policy_draw"
    }));

    let compiled = content_compiler::compile_source_named("xinbenxi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = yijiang
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    assert!(refresh.skill_index.len() >= 76);
    let character = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_WUYI)
        .unwrap();
    assert_eq!(character.skills, [XINBENXI]);
}

//! 第二百八十八批：yixiang 义襄随机获得族状态机契约测试（ABI 734 / tag 755，
//! 单发无续体）。

const YIXIANG: u32 = 2_667_775_149;
const MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn package_publishes_yixiang_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YIXIANG)
        .expect("yixiang trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_is_target_source_hp_gt_owner_per_role_turn"
    );
}

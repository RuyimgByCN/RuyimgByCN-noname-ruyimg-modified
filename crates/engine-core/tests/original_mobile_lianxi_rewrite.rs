//! Golden coverage for mobile 连袭 partial rewrite (2026-08-21 placeholder-header batch):
//! 【杀】不计入手牌上限 via `modifier sha_ignored_hand_limit override 1`.
//! 第二句（杀被弃置后视为使用无距无次杀）无对应过滤器，未建模（近似注记在 .skill 头）。

const LIANXI: u32 = 289_965_7484;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn lianxi_source_and_npack_publish_sha_ignored_hand_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == LIANXI
            && modifier.target == "sha_ignored_hand_limit"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
}

//! Golden coverage for diy 包全量登记（66 将 / ABI 478）。

use content_schema::ContentPackage;

#[test]
fn package_exposes_diy_contract() {
    let committed = ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/diy/diy.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.character_index.len(), 66);
    // 本包技能 147（历史 179+ 中 32 项在引用重映射/去重后归并或移至他包，
    // 全 store 零悬空核验通过）；下限随当前内容钉住，防静默缩水。
    assert!(committed.skill_index.len() >= 147);
    // 代表性技能在包内。
    for def in [
        3_934_300_131u32,
        3_934_300_098u32,
        3_934_300_003u32,
        3_934_300_088u32,
    ] {
        assert!(
            committed
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == def),
            "skill {def} must exist"
        );
    }
    // 角色覆盖：南唐/斗地主系主要武将。
    for name in [
        "ns_caocao",
        "ns_zuoci",
        "ns_zhangxiu",
        "ns_yanliang",
        "junk_guanyu",
        "junk_xuyou",
        "zhangren",
        "ol_maliang",
        "old_bulianshi",
        "noname",
    ] {
        assert!(
            committed
                .character_index
                .iter()
                .any(|character| character.name_key == name),
            "character {name} must be registered"
        );
    }
}

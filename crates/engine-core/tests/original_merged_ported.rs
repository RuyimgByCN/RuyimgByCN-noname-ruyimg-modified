//! O2 全量合并第二批（2026-10-14）：家族 ported 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_ported_character_closure {
//! Ensures already-rewritten skills ship with their complete character entries.

use content_schema::ContentPackage;

fn assert_pack_closure(package: &[u8], dependencies: &[&[u8]]) {
    let package = ContentPackage::from_json_bytes(package).unwrap();
    let dependencies: Vec<_> = dependencies
        .iter()
        .map(|bytes| ContentPackage::from_json_bytes(bytes).unwrap())
        .collect();
    let local_ids: std::collections::BTreeSet<_> = package
        .skill_index
        .iter()
        .map(|entry| entry.skill_def_id)
        .collect();
    let dep_ids: std::collections::BTreeSet<_> = dependencies
        .iter()
        .flat_map(|package| package.skill_index.iter().map(|entry| entry.skill_def_id))
        .collect();
    for character in &package.character_index {
        for skill in &character.skills {
            assert!(
                local_ids.contains(skill) || dep_ids.contains(skill),
                "character {} references unavailable skill {}",
                character.name_key,
                skill
            );
        }
    }
}

#[test]
fn packages_close_all_local_rewritten_character_skill_sets() {
    let bingshi = include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");
    assert_pack_closure(bingshi, &[]);
    let clan = include_bytes!("../../../content/original/ported/clan/clan.npack.json");
    assert_pack_closure(clan, &[]);
    let collab = include_bytes!("../../../content/original/ported/collab/collab.npack.json");
    assert_pack_closure(collab, &[include_bytes!("../../../content/original/ported/mobile/mobile.npack.json"), include_bytes!("../../../content/original/ported/offline/offline.npack.json"), include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json")]);
    let ddd = include_bytes!("../../../content/original/ported/ddd/ddd.npack.json");
    assert_pack_closure(ddd, &[]);
    let diy = include_bytes!("../../../content/original/ported/diy/diy.npack.json");
    assert_pack_closure(diy, &[include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/sp2/sp2.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let extra = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
    assert_pack_closure(extra, &[include_bytes!("../../../content/standard/standard.npack.json")]);
    let huanle = include_bytes!("../../../content/original/ported/huanle/huanle.npack.json");
    assert_pack_closure(huanle, &[]);
    let huicui = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");
    assert_pack_closure(huicui, &[include_bytes!("../../../content/original/ported/mobile/mobile.npack.json"), include_bytes!("../../../content/original/ported/shiji/shiji.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let jiange = include_bytes!("../../../content/original/ported/jiange/jiange.npack.json");
    assert_pack_closure(jiange, &[]);
    let jsrg = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
    assert_pack_closure(jsrg, &[include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/standard/standard.npack.json")]);
    let key = include_bytes!("../../../content/original/ported/key/key.npack.json");
    assert_pack_closure(key, &[include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/sp2/sp2.npack.json")]);
    let mobile = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
    assert_pack_closure(mobile, &[include_bytes!("../../../content/original/ported/extra/extra.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/sp2/sp2.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let newjiang = include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json");
    assert_pack_closure(newjiang, &[]);
    let offline = include_bytes!("../../../content/original/ported/offline/offline.npack.json");
    assert_pack_closure(offline, &[include_bytes!("../../../content/original/ported/extra/extra.npack.json"), include_bytes!("../../../content/original/ported/huicui/huicui.npack.json"), include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json"), include_bytes!("../../../content/original/ported/mobile/mobile.npack.json"), include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json"), include_bytes!("../../../content/original/ported/old/old.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/sb/sb.npack.json"), include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json"), include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json")]);
    let old = include_bytes!("../../../content/original/ported/old/old.npack.json");
    assert_pack_closure(old, &[include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json"), include_bytes!("../../../content/original/ported/extra/extra.npack.json"), include_bytes!("../../../content/original/ported/mobile/mobile.npack.json"), include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json"), include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json")]);
    let onlyOL = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
    assert_pack_closure(onlyOL, &[include_bytes!("../../../content/original/ported/diy/diy.npack.json"), include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let refresh = include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
    assert_pack_closure(refresh, &[include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let sb = include_bytes!("../../../content/original/ported/sb/sb.npack.json");
    assert_pack_closure(sb, &[include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json")]);
    let shahai = include_bytes!("../../../content/original/ported/shahai/shahai.npack.json");
    assert_pack_closure(shahai, &[]);
    let shenhua = include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
    assert_pack_closure(shenhua, &[include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json")]);
    let shiji = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");
    assert_pack_closure(shiji, &[]);
    let sixiang = include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");
    assert_pack_closure(sixiang, &[include_bytes!("../../../content/original/ported/shiji/shiji.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let sp = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
    assert_pack_closure(sp, &[include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json"), include_bytes!("../../../content/original/ported/shiji/shiji.npack.json"), include_bytes!("../../../content/original/ported/sp2/sp2.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json"), include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json")]);
    let sp2 = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
    assert_pack_closure(sp2, &[include_bytes!("../../../content/original/ported/diy/diy.npack.json"), include_bytes!("../../../content/original/ported/mobile/mobile.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json"), include_bytes!("../../../content/original/ported/xianding/xianding.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let sxrm = include_bytes!("../../../content/original/ported/sxrm/sxrm.npack.json");
    assert_pack_closure(sxrm, &[]);
    let tw = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
    assert_pack_closure(tw, &[include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json"), include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json"), include_bytes!("../../../content/original/ported/mobile/mobile.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/original/ported/sb/sb.npack.json"), include_bytes!("../../../content/original/ported/shiji/shiji.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/sp2/sp2.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json")]);
    let wandianlunsha = include_bytes!("../../../content/original/ported/wandianlunsha/wandianlunsha.npack.json");
    assert_pack_closure(wandianlunsha, &[]);
    let xianding = include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");
    assert_pack_closure(xianding, &[include_bytes!("../../../content/original/ported/huicui/huicui.npack.json"), include_bytes!("../../../content/original/ported/sp/sp.npack.json"), include_bytes!("../../../content/original/ported/sp2/sp2.npack.json")]);
    let yijiang = include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
    assert_pack_closure(yijiang, &[include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json"), include_bytes!("../../../content/original/ported/refresh/refresh.npack.json"), include_bytes!("../../../content/standard/standard.npack.json"), include_bytes!("../../../content/original/ported/tw/tw.npack.json")]);
    let yingbian = include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json");
    assert_pack_closure(yingbian, &[]);
    let yxs = include_bytes!("../../../content/original/ported/yxs/yxs.npack.json");
    assert_pack_closure(yxs, &[]);
}
}

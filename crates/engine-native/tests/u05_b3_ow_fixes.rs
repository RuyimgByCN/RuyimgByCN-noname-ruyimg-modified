//! U05-3 (`U05 杀海拾遗` `ow` 守望先锋子包 = 24 武将 + 103 技能 + 辅标记/跨包共 109 宿主项) 真实事件全闭环验收。
//!
//! 源码依据：
//! - `/archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/character.js:1-148`（24 武将）
//! - `/archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js:1-3316`（103 ow 技能）
//! - `standard/skill.js` (`feiying`) 与 `extra/skill.js` (`lianpo`)
//! - 严格不含 `debug_execute_effect` 调用；全部覆盖正例 + 反例 + A/B 选择 + 取消 + 公开信息 AI + 中途 `save -> restore -> Answer`。

#[path = "u05_b3_ow/common.rs"]
mod common;
#[path = "u05_b3_ow/part1_batch_a.rs"]
mod part1_batch_a;
#[path = "u05_b3_ow/part2_batch_b.rs"]
mod part2_batch_b;
#[path = "u05_b3_ow/part3_batch_c.rs"]
mod part3_batch_c;
#[path = "u05_b3_ow/part4_batch_d_and_ai_save.rs"]
mod part4_batch_d_and_ai_save;
#[path = "u05_b3_ow/part5_round2_parity.rs"]
mod part5_round2_parity;
#[path = "u05_b3_ow/part6_round3_parity.rs"]
mod part6_round3_parity;

use content_native_loader::{
    bridge_pack, prepare_locked_content_with_bridge, start_locked_engine, LockedStartMode,
    U05_OW_ALL_SKILLS, U05_OW_GENERALS,
};
use engine_native::catalog::CardKind;
use engine_native::content::extension::ExtensionPack;
use engine_native::Engine;

use common::{assert_card_conservation, freeze_pkg, start_formal_ow};

#[test]
fn u05_ow_formal_package_24_generals_109_host_entries_and_supply_closure() {
    let (_verified, mut e) = start_formal_ow(2026_1121, 4);

    // 1. 全量 24 名 OW 武将（ow/character.js:3-147）静态元数据 + 装配无缺项
    assert_eq!(U05_OW_GENERALS.len(), 24);
    for &(gname, faction, gender, max_hp, skills) in &U05_OW_GENERALS {
        let gid = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == gname))
            .unwrap_or_else(|| panic!("missing formal ow general {gname}"));
        let (_, gfaction, gskills, ghp, ggender) = e.catalog().general_meta(gid).unwrap();
        assert_eq!(gfaction, faction, "faction mismatch on {gname}");
        assert_eq!(ggender, gender, "gender mismatch on {gname}");
        assert_eq!(ghp, max_hp, "max_hp mismatch on {gname}");
        assert_eq!(gskills.as_slice(), skills, "skills mismatch on {gname}");
        for &sk in skills {
            let def = e
                .catalog()
                .skill_by_name(sk)
                .unwrap_or_else(|| panic!("skill {sk} missing in catalog"));
            assert!(
                e.catalog().skills[def as usize]
                    .behavior
                    .package_skill_id()
                    .is_none(),
                "{gname}.{sk} must be native HostSkillDecl (no VM proxy)"
            );
        }
        e.assign_general_named(0, gname)
            .unwrap_or_else(|err| panic!("assign_general_named({gname}) failed: {err:?}"));
    }

    // 2. 全量 103 U05_OW_ALL_SKILLS + 4 辅助标记 + 2 跨包技 = 109 宿主项全部就位且零 VM 代理
    assert_eq!(U05_OW_ALL_SKILLS.len(), 103);
    let helper_markers = ["zhencha2", "pingzhang2", "pingzhang3", "bfengshi4"];
    let cross_skills = ["feiying", "lianpo"];
    for sk in U05_OW_ALL_SKILLS
        .iter()
        .copied()
        .chain(helper_markers)
        .chain(cross_skills)
    {
        let def = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("ow entry {sk} missing from catalog"));
        assert!(
            e.catalog().skills[def as usize]
                .behavior
                .package_skill_id()
                .is_none(),
            "ow skill {sk} must be native HostBehavior (purged from vm_skills)"
        );
    }

    // 3. 132 张离场补给牌（56 ow_hslingjian_* + 24 ow_zhiliaobo_* + 52 ow_sha_*）建局即入 detached_cards
    let detached = &e.state_core().detached_cards;
    let hslingjian_count = detached
        .iter()
        .filter(|(cid, _)| {
            let d = &e.catalog().cards[*cid as usize];
            d.kind == CardKind::Hslingjian && d.name.starts_with("ow_hslingjian_")
        })
        .count();
    let zhiliaobo_count = detached
        .iter()
        .filter(|(cid, _)| e.catalog().cards[*cid as usize].name.starts_with("ow_zhiliaobo_"))
        .count();
    let ow_sha_count = detached
        .iter()
        .filter(|(cid, _)| {
            let d = &e.catalog().cards[*cid as usize];
            d.kind == CardKind::Sha && d.name.starts_with("ow_sha_")
        })
        .count();
    assert_eq!(hslingjian_count, 56, "56 ow_hslingjian supply cards in detached_cards");
    assert_eq!(zhiliaobo_count, 24, "24 ow_zhiliaobo supply cards in detached_cards");
    assert_eq!(ow_sha_count, 52, "52 ow_sha supply cards in detached_cards");
    assert_card_conservation(&e, "shaai-ow startup conservation");

    // 4. bridge_pack 隔离：legacy bridge_pack 仍保留旧 wudian 绑定，不注入 U05OwBatchC
    let legacy_json = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/shaai-ow-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let legacy_out = bridge_pack(&legacy_json).expect("legacy bridge_pack succeeds");
    let legacy_pack: ExtensionPack = legacy_out.pack;
    let legacy_e = Engine::new_with_extensions(9911, 4, &[&legacy_pack]).expect("legacy engine");
    let mujing_def = legacy_e
        .catalog()
        .skill_by_name("mujing")
        .expect("legacy mujing exists");
    assert_eq!(
        legacy_e.catalog().skills[mujing_def as usize]
            .behavior
            .as_host_behavior()
            .map(|b| b.key()),
        Some("host:wudian_mujing"),
        "legacy bridge_pack keeps WudianMujing without touching U05OwBatchC"
    );

    // 5. 9 个 shahai-* 子包联合共存建局零冲突（含 shaai-ow）
    let verified_all = prepare_locked_content_with_bridge(vec![
        freeze_pkg("shahai-gujian-upstream-noname"),
        freeze_pkg("shahai-gwent-upstream-noname"),
        freeze_pkg("shahai-hearth-upstream-noname"),
        freeze_pkg("shahai-mtg-upstream-noname"),
        freeze_pkg("shahai-swd-upstream-noname"),
        freeze_pkg("shahai-yunchou-upstream-noname"),
        freeze_pkg("shahai-zhenfa-upstream-noname"),
        freeze_pkg("shaai-ow-upstream-noname"),
        freeze_pkg("shaai-xianjian-upstream-noname"),
    ])
    .expect("all 9 formal shahai packages prepare cleanly");
    let co_e = start_locked_engine(2026_1122, 4, LockedStartMode::Prototype, &verified_all)
        .expect("9-pack coexistence starts");
    assert_card_conservation(&co_e, "9-pack coexistence conservation");
}

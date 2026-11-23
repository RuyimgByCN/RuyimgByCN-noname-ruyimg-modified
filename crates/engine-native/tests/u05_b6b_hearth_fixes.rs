//! Batch U05-6b (`杀海拾遗` `hearth` Part 2: 107 inventory skills + 4 subskills + 110 generals)
//! Comprehensive upstream-faithful integration test suite (`hearth` 383/383 = 100%).
//!
//! Upstream references:
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/character.js:3-677` (110 generals)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/skill.js:4185-7278` (106 inventory + 4 companions)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/main/content.js:397-412` (`stone_chenshui`)
//!
//! All tests use the formal `packages/extensions/shahai-hearth-upstream-noname` engine,
//! real `GameAction::{PlayCard, ActivateSkill, Answer, NextPhase}` events, ZERO `debug_execute_effect`,
//! strict card conservation across all zones, `BoundedPlanner` public-info checks, and `NGSA 107` save/restore.

#[path = "u05_b6b_hearth/common.rs"]
mod common;
#[path = "u05_b6b_hearth/part1_generals_and_coverage.rs"]
mod part1_generals_and_coverage;
#[path = "u05_b6b_hearth/part2_batch_a.rs"]
mod part2_batch_a;
#[path = "u05_b6b_hearth/part3_batch_b.rs"]
mod part3_batch_b;
#[path = "u05_b6b_hearth/part4_batch_c.rs"]
mod part4_batch_c;
#[path = "u05_b6b_hearth/part5_batch_d.rs"]
mod part5_batch_d;
#[path = "u05_b6b_hearth/part6_batch_e_f.rs"]
mod part6_batch_e_f;
#[path = "u05_b6b_hearth/part7_round2_fixes.rs"]
mod part7_round2_fixes;
#[path = "u05_b6b_hearth/part8_round3_fixes.rs"]
mod part8_round3_fixes;

use common::fresh_proto;
use content_native_loader::U05_HEARTH_ALL_SKILLS;
use engine_native::content::host::HostBehavior;

const U05_6B_ALL_SKILL_KEYS: &[(&str, HostBehavior)] = &[
    // Part 2 Batch A (17)
    ("xieneng", HostBehavior::U05Hearth2BatchA),
    ("fbeifa", HostBehavior::U05Hearth2BatchA),
    ("oldfbeifa", HostBehavior::U05Hearth2BatchA),
    ("yufa", HostBehavior::U05Hearth2BatchA),
    ("yufa2", HostBehavior::U05Hearth2BatchA),
    ("yufa3", HostBehavior::U05Hearth2BatchA),
    ("bingyan", HostBehavior::U05Hearth2BatchA),
    ("shifa", HostBehavior::U05Hearth2BatchA),
    ("oldshifa", HostBehavior::U05Hearth2BatchA),
    ("yuanzheng", HostBehavior::U05Hearth2BatchA),
    ("yulu", HostBehavior::U05Hearth2BatchA),
    ("oldyulu", HostBehavior::U05Hearth2BatchA),
    ("duzhang", HostBehavior::U05Hearth2BatchA),
    ("duzhang2", HostBehavior::U05Hearth2BatchA),
    ("hannu", HostBehavior::U05Hearth2BatchA),
    ("chuidiao", HostBehavior::U05Hearth2BatchA),
    ("hhudun", HostBehavior::U05Hearth2BatchA),
    // Part 2 Batch B (25)
    ("fenlie", HostBehavior::U05Hearth2BatchB),
    ("oldfenlie", HostBehavior::U05Hearth2BatchB),
    ("nianfu", HostBehavior::U05Hearth2BatchB),
    ("xiaorong", HostBehavior::U05Hearth2BatchB),
    ("shixu", HostBehavior::U05Hearth2BatchB),
    ("jixuan", HostBehavior::U05Hearth2BatchB),
    ("qianghua", HostBehavior::U05Hearth2BatchB),
    ("qianghua2", HostBehavior::U05Hearth2BatchB),
    ("biri", HostBehavior::U05Hearth2BatchB),
    ("stuxi", HostBehavior::U05Hearth2BatchB),
    ("stuxi2", HostBehavior::U05Hearth2BatchB),
    ("bingdong", HostBehavior::U05Hearth2BatchB),
    ("luoshi", HostBehavior::U05Hearth2BatchB),
    ("ronghuo", HostBehavior::U05Hearth2BatchB),
    ("ronghuo2", HostBehavior::U05Hearth2BatchB),
    ("fushi", HostBehavior::U05Hearth2BatchB),
    ("oldfushi", HostBehavior::U05Hearth2BatchB),
    ("moyao", HostBehavior::U05Hearth2BatchB),
    ("jiaohui", HostBehavior::U05Hearth2BatchB),
    ("bimeng", HostBehavior::U05Hearth2BatchB),
    ("zhoujiang", HostBehavior::U05Hearth2BatchB),
    ("liehun", HostBehavior::U05Hearth2BatchB),
    ("xjumo", HostBehavior::U05Hearth2BatchB),
    ("malymowang", HostBehavior::U05Hearth2BatchB),
    ("oldmalymowang", HostBehavior::U05Hearth2BatchB),
    // Part 2 Batch C (18)
    ("lingzhou", HostBehavior::U05Hearth2BatchC),
    ("mieshi", HostBehavior::U05Hearth2BatchC),
    ("xmojian", HostBehavior::U05Hearth2BatchC),
    ("xshixin", HostBehavior::U05Hearth2BatchC),
    ("enze", HostBehavior::U05Hearth2BatchC),
    ("oldenze", HostBehavior::U05Hearth2BatchC),
    ("chongsheng", HostBehavior::U05Hearth2BatchC),
    ("guozai", HostBehavior::U05Hearth2BatchC),
    ("guozai2", HostBehavior::U05Hearth2BatchC),
    ("guozaix", HostBehavior::U05Hearth2BatchC),
    ("guozaix2", HostBehavior::U05Hearth2BatchC),
    ("hanshuang", HostBehavior::U05Hearth2BatchC),
    ("oldhanshuang", HostBehavior::U05Hearth2BatchC),
    ("bingshi", HostBehavior::U05Hearth2BatchC),
    ("bingshi2", HostBehavior::U05Hearth2BatchC),
    ("huanwu", HostBehavior::U05Hearth2BatchC),
    ("fengnu", HostBehavior::U05Hearth2BatchC),
    ("shengdun", HostBehavior::U05Hearth2BatchC),
    // Part 2 Batch D (20)
    ("jingmeng", HostBehavior::U05Hearth2BatchD),
    ("hswuji", HostBehavior::U05Hearth2BatchD),
    ("bingshuang", HostBehavior::U05Hearth2BatchD),
    ("yanshu", HostBehavior::U05Hearth2BatchD),
    ("oldyanshu", HostBehavior::U05Hearth2BatchD),
    ("shengyan", HostBehavior::U05Hearth2BatchD),
    ("shengyan2", HostBehavior::U05Hearth2BatchD),
    ("liechao", HostBehavior::U05Hearth2BatchD),
    ("qingliu", HostBehavior::U05Hearth2BatchD),
    ("zhongjia", HostBehavior::U05Hearth2BatchD),
    ("dunji", HostBehavior::U05Hearth2BatchD),
    ("fengxing", HostBehavior::U05Hearth2BatchD),
    ("fengxing2", HostBehavior::U05Hearth2BatchD),
    ("fengxian", HostBehavior::U05Hearth2BatchD),
    ("zhanhou", HostBehavior::U05Hearth2BatchD),
    ("oldzhanhou", HostBehavior::U05Hearth2BatchD),
    ("shijie", HostBehavior::U05Hearth2BatchD),
    ("shengguang", HostBehavior::U05Hearth2BatchD),
    ("xinci", HostBehavior::U05Hearth2BatchD),
    ("anying", HostBehavior::U05Hearth2BatchD),
    // Part 2 Batch E (15)
    ("huanjue", HostBehavior::U05Hearth2BatchE),
    ("bingjia", HostBehavior::U05Hearth2BatchE),
    ("bingjia2", HostBehavior::U05Hearth2BatchE),
    ("bianxing2", HostBehavior::U05Hearth2BatchE),
    ("zuzhou", HostBehavior::U05Hearth2BatchE),
    ("mdzhoufu", HostBehavior::U05Hearth2BatchE),
    ("mdzhoufu2", HostBehavior::U05Hearth2BatchE),
    ("xianzhi", HostBehavior::U05Hearth2BatchE),
    ("jingxiang", HostBehavior::U05Hearth2BatchE),
    ("wlianji", HostBehavior::U05Hearth2BatchE),
    ("mengun", HostBehavior::U05Hearth2BatchE),
    ("mengun2", HostBehavior::U05Hearth2BatchE),
    ("jianren", HostBehavior::U05Hearth2BatchE),
    ("jihuo", HostBehavior::U05Hearth2BatchE),
    ("stone_chenshui", HostBehavior::U05Hearth2BatchE),
    // Part 2 Batch F (16)
    ("tuteng_s", HostBehavior::U05Hearth2BatchF),
    ("s_tuteng", HostBehavior::U05Hearth2BatchF),
    ("tuteng", HostBehavior::U05Hearth2BatchF),
    ("zuling", HostBehavior::U05Hearth2BatchF),
    ("huanfeng", HostBehavior::U05Hearth2BatchF),
    ("tuteng_h", HostBehavior::U05Hearth2BatchF),
    ("tuteng_lose", HostBehavior::U05Hearth2BatchF),
    ("tuteng1", HostBehavior::U05Hearth2BatchF),
    ("tuteng2", HostBehavior::U05Hearth2BatchF),
    ("tuteng3", HostBehavior::U05Hearth2BatchF),
    ("tuteng4", HostBehavior::U05Hearth2BatchF),
    ("tuteng5", HostBehavior::U05Hearth2BatchF),
    ("tuteng6", HostBehavior::U05Hearth2BatchF),
    ("tuteng7", HostBehavior::U05Hearth2BatchF),
    ("tuteng8", HostBehavior::U05Hearth2BatchF),
    ("hongxi", HostBehavior::U05Hearth2BatchF),
];

#[test]
fn u05_b6b_all_217_hearth_skills_unshadowed_and_tokens_present() {
    assert_eq!(U05_HEARTH_ALL_SKILLS.len(), 217, "217 hearth inventory skill rows");
    assert_eq!(U05_6B_ALL_SKILL_KEYS.len(), 111, "107 inventory + 4 subskills = 111");

    let e = fresh_proto(10_701, &["hs_jaina", "hs_lrexxar"]);
    let cat = e.catalog();

    // Every single U05_HEARTH_ALL_SKILLS entry (all 217!) is promoted to HostBehavior (0 left on VM)
    for sk in U05_HEARTH_ALL_SKILLS {
        let def = cat
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U05_HEARTH_ALL_SKILLS `{sk}` missing in Catalog"));
        let s = &cat.skills[def as usize];
        assert!(
            s.behavior.package_skill_id().is_none(),
            "hearth skill `{sk}` must NOT remain on VM"
        );
        assert!(
            s.behavior.as_host_behavior().is_some(),
            "hearth skill `{sk}` must be a HostBehavior"
        );
    }

    // All 111 Part-2 skills + subskills bound to exact U05Hearth2BatchA..F tags 1250..=1255
    for &(sk, expected_beh) in U05_6B_ALL_SKILL_KEYS {
        let def = cat
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U05-6b skill `{sk}` missing in Catalog"));
        let s = &cat.skills[def as usize];
        assert_eq!(
            s.behavior.as_host_behavior(),
            Some(expected_beh),
            "U05-6b skill `{sk}` wrong HostBehavior"
        );
    }

    // Extra off-deck tokens exist in detached_cards and NOT in live deck
    for prefix in [
        "fulei",
        "chuansongmen",
        "liuxinghuoyu",
        "hslingjian_jinjilengdong",
        "hs_huogong",
        "hs_horse_plus",
        "hs_horse_minus",
    ] {
        let off_deck_found = e
            .state_core()
            .detached_cards
            .iter()
            .any(|&(cid, _)| cat.cards[cid as usize].name.starts_with(&format!("{prefix}_")));
        assert!(off_deck_found, "missing detached supply for `{prefix}`");
        let leaked_to_deck = e
            .state_core()
            .deck
            .iter()
            .any(|&cid| cat.cards[cid as usize].name.starts_with(&format!("{prefix}_")));
        assert!(!leaked_to_deck, "token `{prefix}` leaked into draw deck");
    }
}

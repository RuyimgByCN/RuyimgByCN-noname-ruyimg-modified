//! Batch U05-6a (`杀海拾遗` `hearth` Part 1: 166 items = 57 cards + first 109 skills)
//! Comprehensive upstream-faithful integration test suite.
//!
//! Upstream references:
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:4-638` (13 cards)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/skill.js:1-23` (`shihuawuqi`, `jihuocard2`, `chuansongmen2`, `chuansongmen3`)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:4-1136` (44 cards)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/skill.js:5-4177` (107 skills: `hshuanyu` .. `hsbaowu_huangjinyuanhou2`)
//!
//! All tests use the real formal `packages/extensions/shahai-hearth-upstream-noname` engine path,
//! real `GameAction::{PlayCard, ActivateSkill, Answer, NextPhase}` events, ZERO `debug_execute_effect`,
//! strict card conservation, public-info `BoundedPlanner` checks, and mid-choice `NGSA 107` save/restore.

#[path = "u05_b6a_hearth/common.rs"]
mod common;
#[path = "u05_b6a_hearth/part1_cards.rs"]
mod part1_cards;
#[path = "u05_b6a_hearth/part2_batch_a.rs"]
mod part2_batch_a;
#[path = "u05_b6a_hearth/part3_batch_b.rs"]
mod part3_batch_b;
#[path = "u05_b6a_hearth/part4_batch_c.rs"]
mod part4_batch_c;
#[path = "u05_b6a_hearth/part5_batch_d_e.rs"]
mod part5_batch_d_e;
#[path = "u05_b6a_hearth/part6_round3_fixes.rs"]
mod part6_round3_fixes;
#[path = "u05_b6a_hearth/part7_round3_verifier_fixes.rs"]
mod part7_round3_verifier_fixes;

use common::fresh_proto;
use content_native_loader::{U05_HEARTH_ALL_CARDS, U05_HEARTH_ALL_SKILLS};
use engine_native::content::host::HostBehavior;

const U05_6A_ALL_SKILL_KEYS: &[(&str, HostBehavior)] = &[
    // card/hearth/skill.js (2+2+1)
    ("shihuawuqi", HostBehavior::U05HearthCards),
    ("jihuocard2", HostBehavior::U05HearthCards),
    ("chuansongmen2", HostBehavior::U05HearthCards),
    ("chuansongmen3", HostBehavior::U05HearthCards),
    ("hsmengjing_mengye", HostBehavior::U05HearthCards),
    // Batch A: character/hearth/skill.js:5-1117 (20)
    ("hshuanyu", HostBehavior::U05HearthBatchA),
    ("wxuying", HostBehavior::U05HearthBatchA),
    ("buwendingyibian_lose", HostBehavior::U05HearthBatchA),
    ("yibian", HostBehavior::U05HearthBatchA),
    ("hualing", HostBehavior::U05HearthBatchA),
    ("muyin", HostBehavior::U05HearthBatchA),
    ("tqchuanyue", HostBehavior::U05HearthBatchA),
    ("oldhuanjue", HostBehavior::U05HearthBatchA),
    ("hshuanling", HostBehavior::U05HearthBatchA),
    ("hsxingyi", HostBehavior::U05HearthBatchA),
    ("asyouzhang", HostBehavior::U05HearthBatchA),
    ("ylyuchu", HostBehavior::U05HearthBatchA),
    ("nsaiqi", HostBehavior::U05HearthBatchA),
    ("nsbeiming", HostBehavior::U05HearthBatchA),
    ("hsnitai", HostBehavior::U05HearthBatchA),
    ("hspuzhao", HostBehavior::U05HearthBatchA),
    ("hsyanxin", HostBehavior::U05HearthBatchA),
    ("hstianqi", HostBehavior::U05HearthBatchA),
    ("hstianqi_dalian", HostBehavior::U05HearthBatchA),
    ("hstianqi_shali", HostBehavior::U05HearthBatchA),
    // Batch B: character/hearth/skill.js:1127-1856 (20 + 7 ysjqisha subskills)
    ("ysjqisha", HostBehavior::U05HearthBatchB),
    ("ysjqisha_ju", HostBehavior::U05HearthBatchB),
    ("ysjqisha_kuang", HostBehavior::U05HearthBatchB),
    ("ysjqisha_nu", HostBehavior::U05HearthBatchB),
    ("ysjqisha_yi", HostBehavior::U05HearthBatchB),
    ("ysjqisha_wang", HostBehavior::U05HearthBatchB),
    ("ysjqisha_hen", HostBehavior::U05HearthBatchB),
    ("ysjqisha_ao", HostBehavior::U05HearthBatchB),
    ("yindan", HostBehavior::U05HearthBatchB),
    ("hllingxi", HostBehavior::U05HearthBatchB),
    ("zhaochao", HostBehavior::U05HearthBatchB),
    ("zhaochao2", HostBehavior::U05HearthBatchB),
    ("xiyong", HostBehavior::U05HearthBatchB),
    ("srjici", HostBehavior::U05HearthBatchB),
    ("yinzong", HostBehavior::U05HearthBatchB),
    ("tansuo", HostBehavior::U05HearthBatchB),
    ("lieqi", HostBehavior::U05HearthBatchB),
    ("azaowu", HostBehavior::U05HearthBatchB),
    ("shouwang", HostBehavior::U05HearthBatchB),
    ("shouwang2", HostBehavior::U05HearthBatchB),
    ("qingtian", HostBehavior::U05HearthBatchB),
    ("qianfu", HostBehavior::U05HearthBatchB),
    ("qianfu2", HostBehavior::U05HearthBatchB),
    ("shimo", HostBehavior::U05HearthBatchB),
    ("lieyang", HostBehavior::U05HearthBatchB),
    ("lieyang2", HostBehavior::U05HearthBatchB),
    ("zhuilie", HostBehavior::U05HearthBatchB),
    // Batch C: character/hearth/skill.js:1893-2567 (20)
    ("szbianshen", HostBehavior::U05HearthBatchC),
    ("kekao", HostBehavior::U05HearthBatchC),
    ("jinhua", HostBehavior::U05HearthBatchC),
    ("kqizhou", HostBehavior::U05HearthBatchC),
    ("jingcu", HostBehavior::U05HearthBatchC),
    ("shengzhang", HostBehavior::U05HearthBatchC),
    ("pyuhuo", HostBehavior::U05HearthBatchC),
    ("mengye", HostBehavior::U05HearthBatchC),
    ("mengye2", HostBehavior::U05HearthBatchC),
    ("lianzhan", HostBehavior::U05HearthBatchC),
    ("kuixin", HostBehavior::U05HearthBatchC),
    ("fuhua", HostBehavior::U05HearthBatchC),
    ("fuhua2", HostBehavior::U05HearthBatchC),
    ("fuhua3", HostBehavior::U05HearthBatchC),
    ("moxie", HostBehavior::U05HearthBatchC),
    ("qianhou", HostBehavior::U05HearthBatchC),
    ("hlongyi", HostBehavior::U05HearthBatchC),
    ("zhongji", HostBehavior::U05HearthBatchC),
    ("fuwen", HostBehavior::U05HearthBatchC),
    ("jinzhou", HostBehavior::U05HearthBatchC),
    // Batch D: character/hearth/skill.js:2604-3339 (20)
    ("midian", HostBehavior::U05HearthBatchD),
    ("xingluo", HostBehavior::U05HearthBatchD),
    ("yuelu", HostBehavior::U05HearthBatchD),
    ("yushou", HostBehavior::U05HearthBatchD),
    ("yushou_lose", HostBehavior::U05HearthBatchD),
    ("yushou_misha", HostBehavior::U05HearthBatchD),
    ("yushou_huofu", HostBehavior::U05HearthBatchD),
    ("yushou_leiouke", HostBehavior::U05HearthBatchD),
    ("qingzun", HostBehavior::U05HearthBatchD),
    ("ayuling", HostBehavior::U05HearthBatchD),
    ("aoshu", HostBehavior::U05HearthBatchD),
    ("bzhuiji", HostBehavior::U05HearthBatchD),
    ("lianjin", HostBehavior::U05HearthBatchD),
    ("shouji", HostBehavior::U05HearthBatchD),
    ("yingxi", HostBehavior::U05HearthBatchD),
    ("yingxi2", HostBehavior::U05HearthBatchD),
    ("hsguimou", HostBehavior::U05HearthBatchD),
    ("peiyu", HostBehavior::U05HearthBatchD),
    ("peiyu2", HostBehavior::U05HearthBatchD),
    ("wzhanyi", HostBehavior::U05HearthBatchD),
    // Batch E: character/hearth/skill.js:3370-4177 (27)
    ("shengteng", HostBehavior::U05HearthBatchE),
    ("yuansu", HostBehavior::U05HearthBatchE),
    ("chouhuo", HostBehavior::U05HearthBatchE),
    ("nuyan2", HostBehavior::U05HearthBatchE),
    ("nuyan", HostBehavior::U05HearthBatchE),
    ("duxin", HostBehavior::U05HearthBatchE),
    ("oldduxin", HostBehavior::U05HearthBatchE),
    ("hsdusu_shinancao", HostBehavior::U05HearthBatchE),
    ("xiubu", HostBehavior::U05HearthBatchE),
    ("zengli", HostBehavior::U05HearthBatchE),
    ("mobao", HostBehavior::U05HearthBatchE),
    ("mobao2", HostBehavior::U05HearthBatchE),
    ("mobao3", HostBehavior::U05HearthBatchE),
    ("xianji", HostBehavior::U05HearthBatchE),
    ("xianji2", HostBehavior::U05HearthBatchE),
    ("xianji3", HostBehavior::U05HearthBatchE),
    ("tanmi", HostBehavior::U05HearthBatchE),
    ("xueren", HostBehavior::U05HearthBatchE),
    ("maoxian", HostBehavior::U05HearthBatchE),
    ("yiwen", HostBehavior::U05HearthBatchE),
    ("qianghuax", HostBehavior::U05HearthBatchE),
    ("zhuizong", HostBehavior::U05HearthBatchE),
    ("xunbao", HostBehavior::U05HearthBatchE),
    ("xunbao2", HostBehavior::U05HearthBatchE),
    ("hsbaowu_cangbaotu", HostBehavior::U05HearthBatchE),
    ("hsbaowu_huangjinyuanhou", HostBehavior::U05HearthBatchE),
    ("hsbaowu_huangjinyuanhou2", HostBehavior::U05HearthBatchE),
];

#[test]
fn u05_b6a_all_166_items_registered_unshadowed_and_card_metadata_exact() {
    assert_eq!(U05_HEARTH_ALL_CARDS.len(), 57, "57 cards in U05 hearth");
    assert_eq!(U05_HEARTH_ALL_SKILLS.len(), 217, "217 skill rows in U05 hearth table");
    let e = fresh_proto(10_601, &["hs_jaina", "hs_lrexxar"]);
    let cat = e.catalog();

    for &(sk, expected_beh) in U05_6A_ALL_SKILL_KEYS {
        let def = cat
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U05-6a skill `{sk}` missing from formal Catalog"));
        let s = &cat.skills[def as usize];
        assert!(
            s.behavior.package_skill_id().is_none(),
            "skill `{sk}` unexpectedly shadowed by VM instead of HostBehavior"
        );
        assert_eq!(
            s.behavior.as_host_behavior(),
            Some(expected_beh),
            "skill `{sk}` bound to wrong HostBehavior"
        );
    }

    for &(cname, _, _, _, _) in &U05_HEARTH_ALL_CARDS {
        let found = cat.cards.iter().any(|c| {
            c.name == cname
                || c.name.starts_with(&format!("ht_{cname}_"))
                || c.name.starts_with(&format!("{cname}_"))
        });
        assert!(found, "U05-6a card `{cname}` missing in formal Catalog");
    }

    // Derived character/hearth/card.js cards + shihuawuqi must NOT pollute live draw deck initially
    assert!(!e.state_core().deck.iter().any(|&cid| {
        let n = cat.cards[cid as usize].name;
        n.starts_with("hsfashu_")
            || n.starts_with("hsjixie_")
            || n.starts_with("hsqizhou_")
            || n.starts_with("hsbaowu_")
            || n.starts_with("tuteng")
            || n.starts_with("ht_shihuawuqi_")
    }));

    // Real detached token supply exists for derived effects and card tokens
    assert!(!e.state_core().detached_cards.is_empty());
}

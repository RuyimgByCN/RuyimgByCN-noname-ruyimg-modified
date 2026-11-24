//! Track `P3-E2` (`HostBehavior::L11VmFixB`, tag 1393):
//! Strips the 34 broken/incomplete legacy `taixu-upstream-liuli/package.npack.json` VM skills
//! (plus their VM companion sub-skills) ONLY when loading `("upstream-liuli", "taixu")`,
//! and attaches genuine native `HostBehavior::L11VmFixB` `SkillDef`s + off-deck `"tiaojiyanmei"` cards.
//! (`rebiyue` stays on `taixu-upstream-liuli/package.npack.json` VM.)

use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use std::collections::BTreeSet;

/// All 34 `P3-E2` primary skills stripped from `taixu` VM and bound to `HostBehavior::L11VmFixB`.
pub const L11_VM_FIX_B_PRIMARY_SKILLS: [&str; 34] = [
    "qiangzhi",
    "qingxi",
    "quhu",
    "rebeige",
    "refanjian",
    "rejiangchi",
    "rejianxiong",
    "rejieming",
    "releiji",
    "reluanwu",
    "repolu",
    "reshejian",
    "reweimu",
    "rexianzhen",
    "rezhiyi",
    "sbyangwei",
    "shanxie",
    "shenfen",
    "spchijie",
    "spdengli",
    "sphuangen",
    "tianyi",
    "tiaoxin",
    "twshengxi",
    "xinfu_qinguo",
    "xinliegong",
    "yinghun",
    "yiyong",
    "zaiqi",
    "zaoxian",
    "zhaohuo",
    "zhengjian",
    "zhiyi",
    "zhuandui",
];

/// Companion runtime sub-skills / temp modifier skills bound to `HostBehavior::L11VmFixB`.
pub const L11_VM_FIX_B_SUB_SKILLS: [&str; 13] = [
    "jiangchi2",
    "qiangzhi_draw",
    "rejiangchi3",
    "rexianzhen2",
    "rexianzhen3",
    "rexianzhen4",
    "sbyangwei_counter",
    "sbyangwei_effect",
    "shanxie_exclude",
    "tianyi2",
    "tianyi3",
    "xinfu_qinguo_recover",
    "zhengjian_mark",
];

/// Legacy VM skill/sub-skill names stripped from `taixu-upstream-liuli/package.npack.json`.
pub(crate) fn is_stripped_vm_skill(name: &str) -> bool {
    L11_VM_FIX_B_PRIMARY_SKILLS.contains(&name)
        || matches!(
            name,
            "quhu_win"
                | "quhu_lose"
                | "rejiangchi3"
                | "rexianzhen2"
                | "rexianzhen3"
                | "sbyangwei_effect"
                | "tianyi2"
                | "tianyi3"
        )
}

/// Attach `HostBehavior::L11VmFixB` skills for `taixu`.
pub(crate) fn attach_l11_vm_fix_b_skills(
    extra_host_skills: &mut Vec<HostSkillDecl>,
    seen_skills: &mut BTreeSet<String>,
) {
    for &sk in L11_VM_FIX_B_PRIMARY_SKILLS
        .iter()
        .chain(L11_VM_FIX_B_SUB_SKILLS.iter())
    {
        crate::host_skill_bindings::push_host_skill_if_unseen(
            sk,
            HostBehavior::L11VmFixB,
            extra_host_skills,
            seen_skills,
        );
    }
}

/// Attach outside-the-game `"tiaojiyanmei"` cards for `taixu`.
pub(crate) fn attach_l11_vm_fix_b_cards(
    extra_cards: &mut Vec<CardDef>,
    seen_cards: &mut BTreeSet<String>,
) {
    for (suit, cname) in [
        (Suit::Spade, "tiaojiyanmei"),
        (Suit::Heart, "tiaojiyanmei_heart"),
        (Suit::Club, "tiaojiyanmei_club"),
        (Suit::Diamond, "tiaojiyanmei_diamond"),
    ] {
        if seen_cards.insert(cname.to_string()) {
            let mut def = CardDef::basic(cname, CardKind::Wuzhong).with_suit_rank(suit, 6);
            def.off_deck = true;
            extra_cards.push(def);
        }
    }
}

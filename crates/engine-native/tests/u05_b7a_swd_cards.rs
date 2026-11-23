//! Batch U05-7a (`杀海拾遗 · swd` 39 卡 + 67 卡牌技 = 106 项) 真实战局与审修回归测试。

#[path = "u05_b7a_swd_cards/common.rs"]
mod common;
#[path = "u05_b7a_swd_cards/part1_cards.rs"]
mod part1_cards;
#[path = "u05_b7a_swd_cards/part2_card_skills.rs"]
mod part2_card_skills;
#[path = "u05_b7a_swd_cards/part3_forging_and_save.rs"]
mod part3_forging_and_save;
#[path = "u05_b7a_swd_cards/part4_round3_fixes.rs"]
mod part4_round3_fixes;

use common::*;
use content_native_loader::{U05_SWD_ALL_CARDS, U05_SWD_CARD_SKILLS};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;

#[test]
fn u05_b7a_all_39_cards_and_67_skills_bound_without_vm_stubs() {
    let (_verified, e) = start_shahai_swd_only(9201, 2);
    let cat = e.catalog();

    let first_39 = &U05_SWD_ALL_CARDS[..39];
    assert_eq!(first_39.last().map(|r| r.0), Some("jiguanyuan"));
    for &(cname, ..) in first_39 {
        let found = cat.cards.iter().any(|c| {
            c.name == cname
                || c.name == format!("swd_{cname}")
                || c.name.starts_with(&format!("swd_{cname}"))
        });
        assert!(found, "U05-7a card missing from formal Catalog: {cname}");
    }

    let first_67 = &U05_SWD_CARD_SKILLS[..67];
    assert_eq!(first_67.first().copied(), Some("qiankundai"));
    assert_eq!(first_67.last().copied(), Some("kongdongyin"));
    for &sk in first_67 {
        let def = cat
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U05-7a skill missing from formal Catalog: {sk}"));
        let s = &cat.skills[def as usize];
        assert!(
            s.behavior.package_skill_id().is_none(),
            "U05-7a skill {sk} still uses VM stub instead of native HostBehavior"
        );
    }

    assert_eq!(CardKind::from_key("dujian"), Some(CardKind::Dujian));
    assert_eq!(CardKind::Dujian.key(), "dujian");
    assert_eq!(CardKind::from_key("pangufu"), Some(CardKind::Pangufu));
    assert_eq!(CardKind::Pangufu.key(), "pangufu");
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("decode snapshot");
    assert_eq!(decoded.core.players.len(), 2);
}

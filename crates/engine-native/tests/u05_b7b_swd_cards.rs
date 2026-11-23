//! Batch U05-7b (`杀海拾遗 · swd` 40 卡 + 68 卡牌技 = 108 项) 真实战局与审修回归测试。

#[path = "u05_b7b_swd_cards/common.rs"]
mod common;
#[path = "u05_b7b_swd_cards/part1_cards.rs"]
mod part1_cards;
#[path = "u05_b7b_swd_cards/part2_lingjian.rs"]
mod part2_lingjian;
#[path = "u05_b7b_swd_cards/part3_equip_skills.rs"]
mod part3_equip_skills;
#[path = "u05_b7b_swd_cards/part4_r2_fixes.rs"]
mod part4_r2_fixes;
#[path = "u05_b7b_swd_cards/part5_r3_fixes.rs"]
mod part5_r3_fixes;
#[path = "u05_b7b_swd_cards/part6_n04e_snapshot_parity.rs"]
mod part6_n04e_snapshot_parity;

use common::*;
use content_native_loader::{U05_SWD_ALL_CARDS, U05_SWD_CARD_SKILLS};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, EquipSlot};
use engine_native::content::host::yangpijuan_class;

#[test]
fn u05_b7b_all_40_cards_and_68_skills_bound_without_vm_stubs() {
    let (_verified, e) = start_shahai_swd_only(9301, 2);
    let cat = e.catalog();

    // 后 40 张 `card/swd/card.js:1027..2259` 全部入库且牌种精准
    let rest_40 = &U05_SWD_ALL_CARDS[39..];
    assert_eq!(rest_40.len(), 40);
    assert_eq!(rest_40.first().map(|r| r.0), Some("shenmiguo"));
    assert_eq!(rest_40.last().map(|r| r.0), Some("qiankundai"));
    for &(cname, ..) in rest_40 {
        let found = cat.cards.iter().any(|c| {
            c.name == cname
                || c.name == format!("swd_{cname}")
                || c.name.starts_with(&format!("swd_{cname}"))
                || cname
                    .strip_prefix("hslingjian_")
                    .is_some_and(|stripped| c.name == format!("swd_{stripped}1"))
        });
        assert!(found, "U05-7b card missing from formal Catalog: {cname}");
    }

    // 7 个新增牌种键与 `yangpijuan_class` / 槽位 / 装备技检查
    let expected_kinds = [
        ("qinglianxindeng", CardKind::Qinglianxindeng, Some(EquipSlot::Armor), &["qinglianxindeng"][..], Some(2)),
        ("baihupifeng", CardKind::Baihupifeng, Some(EquipSlot::Armor), &["baihupifeng"][..], Some(2)),
        ("guangshatianyi", CardKind::Guangshatianyi, Some(EquipSlot::Armor), &["guangshatianyi"][..], Some(2)),
        ("xiangyuye", CardKind::Xiangyuye, None, &[][..], Some(0)),
        ("huanpodan", CardKind::Huanpodan, None, &[][..], Some(0)),
        ("qipoguyu", CardKind::Qipoguyu, Some(EquipSlot::Treasure), &[][..], Some(2)),
        ("guiyanfadao", CardKind::Guiyanfadao, Some(EquipSlot::Weapon), &["guiyanfadao"][..], Some(2)),
    ];
    for (key, kind, slot, eq_skills, ypj) in expected_kinds {
        assert_eq!(CardKind::from_key(key), Some(kind));
        assert_eq!(kind.key(), key);
        assert_eq!(kind.equip_slot(), slot);
        assert_eq!(yangpijuan_class(kind), ypj);
        let cd = cat
            .cards
            .iter()
            .find(|c| c.kind == kind)
            .unwrap_or_else(|| panic!("Catalog card missing for {kind:?}"));
        assert_eq!(cd.equip_skills, eq_skills, "equip_skills mismatch on {key}");
    }
    assert_eq!(CardKind::Guiyanfadao.named_weapon_range(), Some(2));
    assert!(CardKind::Xiangyuye.is_basic_kind());
    assert!(CardKind::Huanpodan.is_basic_kind());

    // 确认 `qipoguyu` 不再误挂 `wuxingpan_skill`
    let qpg = cat
        .cards
        .iter()
        .find(|c| c.name == "qipoguyu")
        .expect("qipoguyu card");
    assert_eq!(qpg.kind, CardKind::Qipoguyu);
    assert!(qpg.equip_skills.is_empty());

    // 全部 68 个 U05-7b 卡牌技均无 VM 桩，全部绑定正式 HostBehavior
    let rest_68 = &U05_SWD_CARD_SKILLS[67..];
    assert_eq!(rest_68.len(), 68);
    assert_eq!(rest_68.first().copied(), Some("nvwashi"));
    assert_eq!(rest_68.last().copied(), Some("tianxianjiu"));
    for &sk in rest_68 {
        let def = cat
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U05-7b skill missing from formal Catalog: {sk}"));
        let s = &cat.skills[def as usize];
        assert!(
            s.behavior.package_skill_id().is_none(),
            "U05-7b skill {sk} still uses VM stub instead of native HostBehavior"
        );
    }

    let snap = e.save();
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("decode snapshot");
    assert_eq!(decoded.core.players.len(), 2);
}

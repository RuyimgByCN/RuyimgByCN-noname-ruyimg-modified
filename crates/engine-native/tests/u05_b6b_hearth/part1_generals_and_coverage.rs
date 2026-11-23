//! U05-6b Part 1: ALL 110 `hearth` generals (`hs_jaina` .. `hs_duyaxinshi`)
//! Static parity against `character/hearth/character.js:3-677`, full runtime catalog assembly,
//! cross-subpack skills (`mashu`, `qiangxi`, `busi`, `huopu`, `luanji`, `rejieming`, `xinliegong`, `xinwuyan`),
//! and representative cross-skill runtime verification.

use super::common::*;
use content_native_loader::U05_HEARTH_GENERALS;
use engine_native::catalog::CardKind;
use engine_native::choice::ChoiceOption;
use engine_native::content::host::HostBehavior;
use engine_native::event::Phase;

#[test]
fn u05_b6b_all_110_hearth_generals_static_and_runtime_assembled() {
    assert_eq!(U05_HEARTH_GENERALS.len(), 110, "110 hearth generals");

    // 1) If upstream character.js exists in workspace parent, machine-check every single row!
    let upstream_js = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/character.js");
    if upstream_js.exists() {
        let src = std::fs::read_to_string(&upstream_js).expect("read upstream hearth character.js");
        let mut js_count = 0usize;
        for &(id, _, _, hp, skills) in &U05_HEARTH_GENERALS {
            let needle = format!("\t{id}: {{");
            assert!(src.contains(&needle), "upstream character.js missing `{id}`");
            let rest = src.split(&needle).nth(1).unwrap();
            let block = rest.split("\n\t},").next().unwrap();
            assert!(
                block.contains(&format!("hp: {hp},")),
                "hp mismatch for `{id}` in character.js"
            );
            for sk in skills {
                assert!(
                    block.contains(&format!("\"{sk}\"")),
                    "skill `{sk}` missing for `{id}` in character.js"
                );
            }
            js_count += 1;
        }
        assert_eq!(js_count, 110);
    }

    // 2) Runtime Catalog & Engine assignment across ALL 110 generals + cross-subpack checks
    let e0 = fresh_proto(10_702, &["hs_jaina", "hs_lrexxar"]);
    let cat = e0.catalog();

    for chunk in U05_HEARTH_GENERALS.chunks(2) {
        let g0 = chunk[0].0;
        let g1 = chunk.get(1).map_or("hs_jaina", |x| x.0);
        let e = fresh_proto(10_703, &[g0, g1]);
        for (seat, entry) in chunk.iter().enumerate() {
            let s = seat as u8;
            let p = e.state_core().player(s);
            assert!(p.alive, "general {} failed alive", entry.0);
            assert_eq!(p.max_hp, entry.3, "general {} max_hp mismatch", entry.0);
            assert_eq!(p.hp, entry.3 as i32, "general {} hp mismatch", entry.0);
            for &sk in entry.4 {
                if sk == "mashu" {
                    assert!(
                        has_skill(&e, s, "马术") || has_skill(&e, s, "mashu"),
                        "general {} missing standard mashu",
                        entry.0
                    );
                } else if matches!(sk, "busi" | "huopu" | "luanji") {
                    let ref_sk = format!("shahai-hearth__ref_{sk}");
                    assert!(
                        has_skill(&e, s, &ref_sk) || has_skill(&e, s, sk),
                        "general {} missing ref skill `{sk}` at runtime",
                        entry.0
                    );
                } else {
                    assert!(
                        has_skill(&e, s, sk),
                        "general {} missing skill `{sk}` at runtime",
                        entry.0
                    );
                }
            }
        }
    }

    // 3) Explicitly verify the 8 cross-subpack skills on hearth generals are cleanly resolved
    let qiangxi_def = cat.skill_by_name("qiangxi").expect("qiangxi");
    assert_eq!(
        cat.skills[qiangxi_def as usize].behavior.as_host_behavior(),
        Some(HostBehavior::U08YxsBatchD)
    );
    for (ref_name, raw) in [
        ("shahai-hearth__ref_busi", "busi"),
        ("shahai-hearth__ref_huopu", "huopu"),
        ("shahai-hearth__ref_luanji", "luanji"),
    ] {
        let def = cat
            .skill_by_name(ref_name)
            .or_else(|| cat.skill_by_name(raw))
            .unwrap_or_else(|| panic!("cross-ref skill `{raw}` missing"));
        assert!(
            cat.skills[def as usize].behavior.as_host_behavior().is_some(),
            "cross-ref `{raw}` must resolve to HostBehavior"
        );
    }
    for vm_sk in ["rejieming", "xinliegong", "xinwuyan"] {
        assert!(
            cat.skill_by_name(vm_sk).is_some(),
            "VM hearth general skill `{vm_sk}` must resolve"
        );
    }
}

#[test]
fn u05_b6b_general_cross_skill_play_verification() {
    // 1) hs_jgarrosh (`zhanhou` + `qiangxi`): equip armor -> discard via `zhanhou` -> +1 hujia -> activate `qiangxi`
    let mut e = fresh_proto(10_704, &["hs_jgarrosh", "hs_wuther"]);
    let armor = give_free_card_kind_suit(&mut e, 0, CardKind::Armor, None);
    advance_to_phase(&mut e, 0, Phase::Play);
    play_card(&mut e, 0, armor, vec![]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).equips.len(), 1);
    let _ = activate(&mut e, 0, "zhanhou");
    answer(&mut e, 0, vec![ChoiceOption::Card(armor)]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1);

    // Trigger `qiangxi` on hs_wuther (lose 1 hp, deal 1 dmg to seat 1)
    let _ = activate(&mut e, 0, "qiangxi");
    if let Some(ch) = e.open_choice().cloned() {
        let sel = ch
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Player(1)))
            .copied()
            .unwrap_or_else(|| ch.spec.options[0]);
        answer(&mut e, 0, vec![sel]);
    }
    if let Some(ch) = e.open_choice().cloned() {
        let cost = ch
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Tag(1)))
            .copied()
            .unwrap_or_else(|| ch.spec.options[0]);
        answer(&mut e, 0, vec![cost]);
    }
    drain_auto(&mut e);
    assert!(e.state_core().player(0).hp < e.state_core().player(0).max_hp as i32);

    // 2) hs_anduin (`shengguang` + `shijie`) + `hs_zhishigushu` (`jiaohui`): Play heal ally + End no-damage `jiaohui` + steal `shijie`
    let ezs = fresh_proto(10_706, &["hs_zhishigushu", "hs_jgarrosh"]);
    assert!(has_skill(&ezs, 0, "jiaohui"));
    let mut e2 = fresh_proto(10_705, &["hs_anduin", "hs_jgarrosh"]);
    grant_host_skill(&mut e2, 0, "jiaohui");
    let card_before = total_tracked_cards(&e2);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    let red = give_free_card_kind_suit(
        &mut e2,
        0,
        CardKind::Sha,
        Some(engine_native::catalog::Suit::Heart),
    );
    let enemy_h = give_free_card_kind_suit(&mut e2, 1, CardKind::Shan, None);
    e2.state_core_mut().player_mut(0).hp = 1;
    advance_to_phase(&mut e2, 0, Phase::Play);
    let _ = activate(&mut e2, 0, "shengguang");
    answer(&mut e2, 0, vec![ChoiceOption::Card(red)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(0)]);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hp, 2);

    let ch = advance_until_open_prompt(&mut e2, "hs_jiaohui_target", 12)
        .expect("jiaohui trigger when no damage dealt");
    assert_eq!(ch.observer, 0);
    answer(&mut e2, 0, vec![ChoiceOption::Player(0)]);
    let c2 = e2.open_choice().expect("jiaohui choice");
    assert_eq!(c2.spec.prompt, "hs_jiaohui_choice");
    answer(&mut e2, 0, vec![ChoiceOption::Tag(1)]); // recover +1 -> hp=3
    // `shijie` triggers on same End phase because seat 1 has `enemy_h`
    if let Some(ch_shijie) = e2.open_choice().cloned() {
        assert_eq!(ch_shijie.spec.prompt, "hs_shijie_target");
        answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    }
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hp, 3);
    assert!(e2.state_core().player(0).hand.contains(&enemy_h));
    assert_eq!(total_tracked_cards(&e2), card_before);
}

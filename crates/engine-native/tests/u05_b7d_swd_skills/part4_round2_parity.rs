//! U05-7d 复审 R2 全部 8 项上游等价与反例回归测试（零 `debug_execute_effect`）。

use super::common::*;
use content_native_loader::{
    bridge_pack, decision34_pack9_extension_pack, prepare_locked_content_with_bridge,
    start_locked_engine, validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::content::extension::ExtensionPack;
use std::collections::{BTreeMap, BTreeSet};

fn load_swd_with_pack9(seed: u64, seats: u8) -> Engine {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(
        ext_root.join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("read swd cards.json")
    .replace("\r\n", "\n");
    let mut swd_pack = bridge_pack(&json).expect("bridge swd").pack;
    let p9 = decision34_pack9_extension_pack();
    let existing_skills: BTreeSet<&str> = p9
        .host_skills
        .iter()
        .map(|s| s.name)
        .chain(p9.vm_skills.iter().map(|s| s.name.as_str()))
        .collect();
    let existing_generals: BTreeSet<&str> = p9.generals.iter().map(|g| g.name).collect();
    swd_pack
        .host_skills
        .retain(|s| !existing_skills.contains(s.name));
    swd_pack
        .vm_skills
        .retain(|s| !existing_skills.contains(s.name.as_str()));
    swd_pack
        .generals
        .retain(|g| !existing_generals.contains(g.name));
    let packs: [&ExtensionPack; 2] = [&p9, &swd_pack];
    Engine::new_with_extensions(seed, usize::from(seats), &packs).expect("swd+pack9")
}

fn load_swd_gujian_mtg(seed: u64, seats: u8) -> (VerifiedLockedContent, Engine) {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let mut frozen_pkgs = Vec::new();
    for dir_name in [
        "shahai-swd-upstream-noname",
        "shahai-gujian-upstream-noname",
        "shahai-mtg-upstream-noname",
    ] {
        let pkg_dir = ext_root.join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .expect("read package.npack.json")
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        frozen_pkgs.push(validate_and_freeze_raw_package(raw_json, assets).expect("freeze"));
    }
    let verified = prepare_locked_content_with_bridge(frozen_pkgs).expect("prepare multi packs");
    let engine = start_locked_engine(seed, usize::from(seats), LockedStartMode::Prototype, &verified)
        .expect("start multi");
    (verified, engine)
}

#[test]
fn u05_b7d_r2_moyan_cancel_refund_xuanzhou_filter_and_tianshu_unique() {
    // 1. Moyan cancel refund at moyan_cards & moyan_targets + effective_card_suit (xinhongyan).
    let mut em = load_swd_with_pack9(7951, 3);
    for p in 0..3 {
        em.debug_remove_general(p);
        em.debug_clear_hand(p);
    }
    em.debug_grant(0, "moyan");
    advance_to_play(&mut em, 0);
    em.debug_clear_hand(0);
    em.debug_set_hp(0, 2);
    let sp_card = place_matching_card(&mut em, 0, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let moyan_def = em.catalog().skill_by_name("moyan").unwrap();
    // Raw Spade alone must NOT pass activation!
    assert!(matches!(
        submit(&mut em, 0, GameAction::ActivateSkill { skill: moyan_def }),
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ), "moyan must reject raw non-red cards before xinhongyan");
    // Grant xinhongyan -> Spade turns into effective Heart -> activation succeeds!
    em.debug_grant(0, "xinhongyan");
    assert!(matches!(
        submit(&mut em, 0, GameAction::ActivateSkill { skill: moyan_def }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ), "moyan must allow effective red cards via xinhongyan");
    let c_cards = em.open_choice().expect("moyan_cards").clone();
    assert!(c_cards.spec.options.contains(&ChoiceOption::Card(sp_card)));
    // Cancel at moyan_cards -> once_used refunded!
    submit(&mut em, 0, GameAction::Answer { choice_id: c_cards.id, selection: vec![] });
    assert!(em.open_choice().is_none());
    assert!(matches!(
        submit(&mut em, 0, GameAction::ActivateSkill { skill: moyan_def }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ), "cancelling moyan_cards must refund once_used");
    let c_cards2 = em.open_choice().expect("moyan_cards retry").clone();
    submit(&mut em, 0, GameAction::Answer { choice_id: c_cards2.id, selection: vec![ChoiceOption::Card(sp_card)] });
    let c_tgts = em.open_choice().expect("moyan_targets").clone();
    // Cancel at moyan_targets -> once_used refunded!
    submit(&mut em, 0, GameAction::Answer { choice_id: c_tgts.id, selection: vec![] });
    assert!(matches!(
        submit(&mut em, 0, GameAction::ActivateSkill { skill: moyan_def }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ), "cancelling moyan_targets must refund once_used");
    let c_cards3 = em.open_choice().expect("moyan_cards final").clone();
    submit(&mut em, 0, GameAction::Answer { choice_id: c_cards3.id, selection: vec![ChoiceOption::Card(sp_card)] });
    let c_tgts3 = em.open_choice().expect("moyan_targets final").clone();
    let hp1_before = em.debug_hp(1);
    submit(&mut em, 0, GameAction::Answer { choice_id: c_tgts3.id, selection: vec![ChoiceOption::Player(1)] });
    settle_confirm(&mut em);
    assert_eq!(em.debug_hp(1), hp1_before - 1, "moyan deals 1 fire damage after full commit");
    assert!(matches!(
        submit(&mut em, 0, GameAction::ActivateSkill { skill: moyan_def }),
        CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
    ), "moyan consumes once_used only after committing");

    // 2. Xuanzhou type === "trick" strict gate + 6. SwdTianshu type === ["trick","delay"] & unique list.
    let (_v2, mut ex) = load_swd_gujian_mtg(7952, 3);
    for p in 0..3 {
        ex.debug_remove_general(p);
        ex.debug_clear_hand(p);
    }
    ex.debug_grant(0, "xuanzhou");
    ex.debug_grant(0, "swdtianshu");
    advance_to_play(&mut ex, 0);
    ex.debug_clear_hand(0);
    let _c_le = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Le);
    let _c_bing = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Bing);
    let _c_shandian = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::ShanDian);
    let _c_caoyao = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Caoyao);
    let _c_land = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Longlushanfeng);
    let _c_lingjian = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Hslingjian);
    let _c_food = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Tanhuadong);
    let xz_def = ex.catalog().skill_by_name("xuanzhou").unwrap();
    // None of delay / Land / Caoyao / Hslingjian / food may pass xuanzhou!
    assert!(matches!(
        submit(&mut ex, 0, GameAction::ActivateSkill { skill: xz_def }),
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ), "xuanzhou must reject delay & non-trick special cards");
    // Now add an ordinary trick (Wuzhong) -> xuanzhou_trick must contain ONLY c_wuzhong!
    let c_wuzhong = place_matching_card(&mut ex, 0, |_, d| d.kind == CardKind::Wuzhong);
    assert!(matches!(
        submit(&mut ex, 0, GameAction::ActivateSkill { skill: xz_def }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let ask_xz = ex.open_choice().expect("xuanzhou_trick").clone();
    assert_eq!(ask_xz.spec.options, vec![ChoiceOption::Card(c_wuzhong)]);
    submit(&mut ex, 0, GameAction::Answer { choice_id: ask_xz.id, selection: vec![ChoiceOption::Card(c_wuzhong)] });
    let ask_xzt = ex.open_choice().expect("xuanzhou_target").clone();
    submit(&mut ex, 0, GameAction::Answer { choice_id: ask_xzt.id, selection: vec![ChoiceOption::Player(1)] });
    let ask_xzk = ex.open_choice().expect("xuanzhou_delay_kind").clone();
    // Includes Guiyoujie (4) & HongShui (5) alongside 1..=3!
    assert!(ask_xzk.spec.options.contains(&ChoiceOption::Tag(4)));
    assert!(ask_xzk.spec.options.contains(&ChoiceOption::Tag(5)));
    submit(&mut ex, 0, GameAction::Answer { choice_id: ask_xzk.id, selection: vec![ChoiceOption::Tag(4)] });
    settle_confirm(&mut ex);
    assert_eq!(ex.state_core().declared_kinds.get(&c_wuzhong).copied(), Some(CardKind::Guiyoujie));

    // SwdTianshu checks trick+delay (rejects Land/Caoyao/Hslingjian/food) and unique filter!
    let (_v3, mut et) = load_swd_gujian_mtg(7953, 3);
    for p in 0..3 {
        et.debug_remove_general(p);
        et.debug_clear_hand(p);
    }
    et.debug_grant(0, "swdtianshu");
    let ts_def = et.catalog().skill_by_name("swdtianshu").unwrap();
    let mut check_ts = |general_name: &str, expected_skills: &[&str], pick_idx: usize| {
        et.debug_remove_general(1);
        et.assign_general_named(1, general_name).expect(general_name);
        advance_to_play(&mut et, 0);
        et.debug_clear_hand(0);
        let _ = place_matching_card(&mut et, 0, |_, d| d.kind == CardKind::Caoyao);
        let _ = place_matching_card(&mut et, 0, |_, d| d.kind == CardKind::Longlushanfeng);
        let _ = place_matching_card(&mut et, 0, |_, d| d.kind == CardKind::Hslingjian);
        let _ = place_matching_card(&mut et, 0, |_, d| d.kind == CardKind::Tanhuadong);
        let c_pay = place_matching_card(&mut et, 0, |_, d| d.kind == CardKind::Le);
        assert!(matches!(
            submit(&mut et, 0, GameAction::ActivateSkill { skill: ts_def }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        let ask_tsc = et.open_choice().expect("swdtianshu_card").clone();
        assert_eq!(ask_tsc.spec.options, vec![ChoiceOption::Card(c_pay)]);
        submit(&mut et, 0, GameAction::Answer { choice_id: ask_tsc.id, selection: vec![ChoiceOption::Card(c_pay)] });
        let ask_tst = et.open_choice().expect("swdtianshu_target").clone();
        submit(&mut et, 0, GameAction::Answer { choice_id: ask_tst.id, selection: vec![ChoiceOption::Player(1)] });
        let ask_tss = et.open_choice().expect("swdtianshu_skill").clone();
        let expected_tags: Vec<ChoiceOption> = (0..expected_skills.len())
            .map(|i| ChoiceOption::Tag(i as u8))
            .collect();
        assert_eq!(
            ask_tss.spec.options, expected_tags,
            "{general_name} should expose exact non-unique skill set {expected_skills:?}"
        );
        let chosen = expected_skills[pick_idx];
        submit(&mut et, 0, GameAction::Answer { choice_id: ask_tss.id, selection: vec![ChoiceOption::Tag(pick_idx as u8)] });
        settle_confirm(&mut et);
        assert!(et.debug_has_skill(0, chosen), "{general_name} tag {pick_idx} must grant {chosen}");
    };
    // Check both huanyin (index 0) & xuanzhou (index 2) on swd_qiner (none are unique).
    check_ts("swd_qiner", &["huanyin", "tianhuo", "xuanzhou"], 2);
    check_ts("swd_qiner", &["huanyin", "tianhuo"], 0);
    // Unique skills excluded: tianlun / mailun / (miles_xueyi, mohua2).
    check_ts("swd_jiangziya", &["mingfu"], 0);
    check_ts("swd_jialanduo", &["xianyin"], 0);
    check_ts("swd_miles", &["aojian"], 0);
}

#[test]
fn u05_b7d_r2_huajian_targetenabled_physical_and_dunxing_huanyin_multitarget() {
    // 3. SwdHuajian: kongcheng 0-hand excluded + real card Suit triggers Renwang!
    let (_v, mut eh) = load_formal_swd_only(7953, 4);
    for p in 0..4 {
        eh.debug_remove_general(p);
        eh.debug_clear_hand(p);
    }
    eh.debug_grant(0, "huajian");
    eh.debug_grant(1, "空城"); // p1 has 0 hand cards -> Sha targetEnabled must exclude p1!
    let rw_id = place_matching_card(&mut eh, 2, |_, d| d.kind == CardKind::Renwang);
    eh.state_core_mut().players[2].hand.retain(|&x| x != rw_id);
    eh.state_core_mut().players[2].equips.push(rw_id);
    let _ = eh.debug_place(2, "tao"); // keep hand non-empty
    let _ = eh.debug_place(3, "tao");
    advance_to_play(&mut eh, 0);
    eh.debug_clear_hand(0);
    let blk_cost = place_matching_card(&mut eh, 0, |_, d| d.suit == Suit::Spade && d.kind == CardKind::Sha);
    // Advance out of Play (Play -> Discard) to trigger huajian at Phase::End/phaseUseEnd:
    submit(&mut eh, 0, GameAction::NextPhase);
    let ask_hc = eh.open_choice().expect("huajian_discard (black)").clone();
    assert_eq!(ask_hc.spec.prompt, "huajian_discard");
    submit(&mut eh, 0, GameAction::Answer { choice_id: ask_hc.id, selection: vec![ChoiceOption::Card(blk_cost)] });
    let ask_ht = eh.open_choice().expect("huajian_target").clone();
    assert!(!ask_ht.spec.options.contains(&ChoiceOption::Player(1)), "kongcheng 0-hand p1 excluded");
    assert!(ask_ht.spec.options.contains(&ChoiceOption::Player(2)));
    let hp2_before = eh.debug_hp(2);
    submit(&mut eh, 0, GameAction::Answer { choice_id: ask_ht.id, selection: vec![ChoiceOption::Player(2)] });
    settle_confirm(&mut eh);
    assert_eq!(eh.debug_hp(2), hp2_before, "Renwang blocks black huajian Sha (card identity preserved)");
    assert!(eh.state_core().discard.contains(&blk_cost), "black huajian card discarded normally");

    // Now test red card through Renwang -> deals 1 damage!
    advance_to_play(&mut eh, 0);
    eh.debug_clear_hand(0);
    let red_cost = place_matching_card(&mut eh, 0, |_, d| d.suit == Suit::Heart && d.kind == CardKind::Sha);
    submit(&mut eh, 0, GameAction::NextPhase);
    let ask_hc2 = eh.open_choice().expect("huajian_discard (red)").clone();
    submit(&mut eh, 0, GameAction::Answer { choice_id: ask_hc2.id, selection: vec![ChoiceOption::Card(red_cost)] });
    let ask_ht2 = eh.open_choice().expect("huajian_target (red)").clone();
    submit(&mut eh, 0, GameAction::Answer { choice_id: ask_ht2.id, selection: vec![ChoiceOption::Player(2)] });
    settle_confirm(&mut eh);
    assert_eq!(eh.debug_hp(2), hp2_before - 1, "Red huajian card passes through Renwang and deals 1 dmg");

    // 4a. SwdDunxing against multi-target Chain [1, 2]: p1 cancels self ONLY, p2 still linked, card discarded.
    let (_v2, mut ed) = load_formal_swd_only(7954, 4);
    for p in 0..4 {
        ed.debug_remove_general(p);
        ed.debug_clear_hand(p);
    }
    ed.debug_grant(1, "dunxing");
    advance_to_play(&mut ed, 0);
    for p in 0..4 {
        ed.debug_clear_hand(p);
    }
    let dx_cost = ed.debug_place(1, "tao");
    let chain = place_matching_card(&mut ed, 0, |_, d| d.kind == CardKind::Chain);
    set_deck_top_matching(&mut ed, |d| d.suit == Suit::Spade);
    submit(&mut ed, 0, GameAction::PlayCard { card: chain, targets: vec![1, 2] });
    for _ in 0..8 {
        let Some(c) = ed.open_choice().cloned() else { break };
        let sel = match c.spec.prompt {
            "trigger_confirm" => vec![ChoiceOption::Confirm],
            "dunxing_discard" => vec![ChoiceOption::Card(dx_cost)],
            _ => vec![ChoiceOption::Pass],
        };
        submit(&mut ed, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(!ed.debug_linked(1), "dunxing removes p1 from multi-target Chain");
    assert!(ed.debug_linked(2), "other target p2 remains linked under dunxing");
    assert!(!ed.debug_hand(0).contains(&chain) && ed.state_core().discard.contains(&chain));

    // 4b. SwdHuanyin against SavageAttack [1, 2, 3]: p1 Spade removes p1 ONLY, p2 & p3 still take 1 dmg!
    let (_v3, mut ehy) = load_formal_swd_only(7955, 4);
    for p in 0..4 {
        ehy.debug_remove_general(p);
        ehy.debug_clear_hand(p);
    }
    ehy.debug_grant(1, "huanyin");
    advance_to_play(&mut ehy, 0);
    for p in 0..4 {
        ehy.debug_clear_hand(p);
    }
    let aoe = place_matching_card(&mut ehy, 0, |_, d| d.kind == CardKind::SavageAttack);
    set_deck_top_matching(&mut ehy, |d| d.suit == Suit::Spade);
    let (h1, h2, h3) = (ehy.debug_hp(1), ehy.debug_hp(2), ehy.debug_hp(3));
    submit(&mut ehy, 0, GameAction::PlayCard { card: aoe, targets: vec![] });
    settle_confirm(&mut ehy);
    assert_eq!(ehy.debug_hp(1), h1, "huanyin Spade removes p1 from SavageAttack");
    assert_eq!(ehy.debug_hp(2), h2 - 1, "huanyin only removes owner: p2 still damaged");
    assert_eq!(ehy.debug_hp(3), h3 - 1, "huanyin only removes owner: p3 still damaged");
}

#[test]
fn u05_b7d_r2_tanlin_zero_dmg_jifeng_tricks_and_daofa_xielv_he_suit() {
    // 5. SwdTanlinDefence ignores d.amount == 0 damage.
    let (v, mut et) = load_formal_swd_only(7956, 3);
    for p in 0..3 {
        et.debug_remove_general(p);
        et.debug_clear_hand(p);
    }
    et.debug_grant(1, "tanlin_defence");
    let c0 = et.debug_place(0, "tao");
    let c1 = et.debug_place(1, "tao");
    et.debug_deal_damage(Some(0), 1, 0, engine_native::event::DamageNature::Normal);
    assert!(et.open_choice().is_none(), "tanlin_defence must not trigger on 0 damage");
    assert!(et.debug_hand(0).contains(&c0) && et.debug_hand(1).contains(&c1));

    // 7. Jifeng expands [1,1] ordinary tricks (Dismantle/Snatch), keeps delay single-target, per-target miss!
    let mut saw_both = false;
    let mut saw_partial = false;
    let mut saved_and_restored = false;
    for seed in 7960..8020u64 {
        let (_vs, mut ej) = load_formal_swd_only(seed, 4);
        for p in 0..4 {
            ej.debug_remove_general(p);
            ej.debug_clear_hand(p);
        }
        ej.debug_grant(0, "jifeng");
        advance_to_play(&mut ej, 0);
        for p in 0..4 {
            ej.debug_clear_hand(p);
        }
        let le_c = place_matching_card(&mut ej, 0, |_, d| d.kind == CardKind::Le);
        assert!(matches!(
            submit(&mut ej, 0, GameAction::PlayCard { card: le_c, targets: vec![1, 3] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ), "jifeng must NOT expand delay cards like Le");
        let t1 = ej.debug_place(1, "tao");
        let t3 = ej.debug_place(3, "shan");
        let snatch = place_matching_card(&mut ej, 0, |_, d| d.kind == CardKind::Snatch);
        assert!(matches!(
            submit(&mut ej, 0, GameAction::PlayCard { card: snatch, targets: vec![1, 3] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ), "jifeng must allow multi-target Snatch at distance 1");
        if !saved_and_restored && ej.open_choice().is_some() {
            saved_and_restored = true;
            ej = roundtrip_bytes(&v, &ej);
        }
        settle_confirm(&mut ej);
        let got1 = ej.debug_hand(0).contains(&t1);
        let got3 = ej.debug_hand(0).contains(&t3);
        if got1 && got3 {
            saw_both = true;
        } else if got1 != got3 {
            saw_partial = true;
        }
        if saw_both && saw_partial && saved_and_restored {
            break;
        }
    }
    assert!(saw_both, "jifeng multi-target Snatch resolves against both targets");
    assert!(saw_partial, "jifeng per-target miss keeps the surviving target on Snatch");

    // 8a. SwdDaofa discards equip when damage source has 0 hand cards & >= 1 equip.
    let (_vd, mut edf) = load_formal_swd_only(7958, 3);
    for p in 0..3 {
        edf.debug_remove_general(p);
        edf.debug_clear_hand(p);
    }
    edf.debug_grant(2, "daofa");
    let eq0 = place_matching_card(&mut edf, 0, |_, d| d.kind == CardKind::Weapon);
    edf.state_core_mut().players[0].hand.clear();
    edf.state_core_mut().players[0].equips.push(eq0);
    edf.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let ask_df = edf.open_choice().expect("daofa confirm when source has equip only").clone();
    assert_eq!(ask_df.observer, 2);
    submit(&mut edf, 2, GameAction::Answer { choice_id: ask_df.id, selection: vec![ChoiceOption::Confirm] });
    settle_confirm(&mut edf);
    assert!(edf.state_core().players[0].equips.is_empty(), "daofa discarded source equip");
    assert!(edf.state_core().discard.contains(&eq0), "discarded equip landed in discard pile");

    // 8b. SwdXielv honors effective_card_suit (xinhongyan Spade -> Heart makes mixed-raw hand all red).
    let mut exl = load_swd_with_pack9(7959, 3);
    for p in 0..3 {
        exl.debug_remove_general(p);
        exl.debug_clear_hand(p);
    }
    exl.debug_grant(0, "xielv");
    exl.debug_grant(0, "xinhongyan");
    advance_to_play(&mut exl, 0);
    exl.debug_clear_hand(0);
    let _ht = place_matching_card(&mut exl, 0, |_, d| d.suit == Suit::Heart);
    let _sp = place_matching_card(&mut exl, 0, |_, d| d.suit == Suit::Spade);
    exl.debug_set_hp(0, 4); // hand_len=2 <= max_hp=4 -> skips discard ask, goes right to End!
    submit(&mut exl, 0, GameAction::NextPhase); // Play -> Discard -> End!
    // Advance until xielv confirm or turn passes:
    for _ in 0..6 {
        if exl.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            break;
        }
        let cur = exl.turn().current;
        if cur != 0 {
            break;
        }
        submit(&mut exl, 0, GameAction::NextPhase);
    }
    let ask_xl = exl.open_choice().expect("xielv triggers via effective red suits").clone();
    assert_eq!(ask_xl.spec.prompt, "trigger_confirm");
}

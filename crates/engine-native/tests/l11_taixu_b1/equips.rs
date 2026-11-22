use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::engine::RejectReason;
use engine_native::event::DamageNature;
use engine_native::rvm_bridge::modifiers::{eval_card_target_canceled, txminguangkai_blocks_link};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension_card.js:8, 2238-2285` (`txhj_xingtianpojunfu`, `txhj_noda_axe`, `txhj_noda_axe2`),
/// `22, 1554` (`txhj_zhuge`), `108, 1673` & `extension_skill.js:856` (`txhj_qilingong`, `qilin_skill`).
#[test]
fn l11_equips_axe_zhuge_and_qilingong() {
    // 1. txhj_xingtianpojunfu: Sha in Play discards 2 (excluding axe itself) -> seals target (`txhj_noda_axe2`), cancel keeps cards
    let mut e = start_taixu(0x1201, 2);
    let axe = equip_named(&mut e, 0, "txhj_xingtianpojunfu");
    let sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let d1 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let d2 = place_kind_suit(&mut e, 0, CardKind::Jiu, Suit::Club);
    let shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Diamond);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    let c = expect_pending(&e, "txhj_xingtianpojunfu:discard_cards");
    assert!(!c.spec.options.contains(&ChoiceOption::Card(axe)), "must not offer axe itself");
    // AI / cancel check: cancelling does not seal target, target CAN play Shan
    let mut e_cancel = e.clone();
    let _ = answer(&mut e_cancel, &c, Vec::new());
    let c_shan = e_cancel.open_choice().expect("target Shan choice").clone();
    assert!(c_shan.spec.options.contains(&ChoiceOption::Card(shan1)));
    let _ = answer(&mut e_cancel, &c_shan, vec![ChoiceOption::Card(shan1)]);
    drain(&mut e_cancel);
    assert_eq!(e_cancel.debug_hp(1), 6);

    // Confirming discard seals target (`txhj_noda_axe2`), target cannot play Shan and takes damage!
    let e_ok = save_restore_and_compare_pending(&e, &c, vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)]);
    assert!(e_ok.debug_hand(0).is_empty());
    assert!(e_ok.debug_has_skill(1, "txhj_noda_axe2"));
    assert_eq!(e_ok.debug_hp(1), 5, "sealed target could not respond Shan");
    assert!(e_ok.debug_hand(1).contains(&shan1));

    // Also test granted `txhj_noda_axe` when weapon NOT equipped, clearing `txhj_noda_axe2` at turn end
    let mut e_raw = start_taixu(0x1202, 2);
    e_raw.debug_grant(0, "txhj_noda_axe");
    let sha_r = place_kind_suit(&mut e_raw, 0, CardKind::Sha, Suit::Spade);
    let r1 = place_kind(&mut e_raw, 0, CardKind::Shan);
    let r2 = place_kind(&mut e_raw, 0, CardKind::Shan);
    let _ = play_card(&mut e_raw, 0, sha_r, vec![1]);
    let c_raw = expect_pending(&e_raw, "txhj_noda_axe:discard_cards");
    let _ = answer(&mut e_raw, &c_raw, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    drain(&mut e_raw);
    assert!(e_raw.debug_has_skill(1, "txhj_noda_axe2"));
    // Advance out of seat 0's turn -> `txhj_noda_axe2` clears
    e_raw.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_raw);
    drain(&mut e_raw);
    assert!(!e_raw.debug_has_skill(1, "txhj_noda_axe2"));

    // 2. txhj_zhuge: +3 Sha limit (4 Sha total in Play phase, 5th rejected)
    let mut e_zg = start_taixu(0x1203, 2);
    let _ = equip_named(&mut e_zg, 0, "txhj_zhuge");
    for _ in 0..4 {
        let s = place_kind_suit(&mut e_zg, 0, CardKind::Sha, Suit::Club);
        let out = play_card(&mut e_zg, 0, s, vec![1]);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
        drain(&mut e_zg);
    }
    let s5 = place_kind_suit(&mut e_zg, 0, CardKind::Sha, Suit::Spade);
    assert!(matches!(
        play_card(&mut e_zg, 0, s5, vec![1]),
        CommandOutcome::Rejected(RejectReason::ShaLimit)
    ));

    // 3. txhj_qilingong & qilin_skill: Sha damage discards target's Treasure; counterexample without Treasure
    let mut e_ql = start_taixu(0x1204, 2);
    let _ = equip_named(&mut e_ql, 0, "txhj_qilingong");
    let tr = equip_named(&mut e_ql, 1, "txhj_dinglanyemingzhu");
    let s_ql = place_kind_suit(&mut e_ql, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_ql, 0, s_ql, vec![1]);
    let c_ql = answer_pass_until(&mut e_ql, "txhj_qilingong:discard_horse");
    assert_eq!(ai_plan(&e_ql, 0), GameAction::Answer {
        choice_id: c_ql.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let e_ql_done = save_restore_and_compare_pending(&e_ql, &c_ql, vec![ChoiceOption::Confirm]);
    assert!(e_ql_done.debug_equips(1).is_empty(), "target Treasure {tr} discarded");

    // `qilin_skill` granted directly without weapon
    let mut e_qs = start_taixu(0x1205, 2);
    e_qs.debug_grant(0, "qilin_skill");
    let tr_qs = equip_named(&mut e_qs, 1, "txhj_xuwangzhimian");
    let s_qs = place_kind_suit(&mut e_qs, 0, CardKind::Sha, Suit::Diamond);
    let _ = play_card(&mut e_qs, 0, s_qs, vec![1]);
    let c_qs = answer_pass_until(&mut e_qs, "qilin_skill:discard_horse");
    let _ = answer(&mut e_qs, &c_qs, vec![ChoiceOption::Card(tr_qs)]);
    drain(&mut e_qs);
    assert!(e_qs.debug_equips(1).is_empty());
}

/// Upstream: `extension_card.js:171, 1639` (`txhj_fangtianhuaji`), `379, 1700` (`txhj_xuwangzhimian`), `438, 1716` (`txhj_piliche`).
#[test]
fn l11_equips_fangtian_xuwang_and_piliche() {
    // 1. txhj_fangtianhuaji: first Play-phase Sha adds up to 2 extra targets; 2nd Sha does NOT prompt
    let mut e = start_taixu(0x1206, 4);
    let _ = equip_named(&mut e, 0, "txhj_fangtianhuaji");
    e.debug_grant(0, "咆哮");
    let s1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Diamond);
    let _ = play_card(&mut e, 0, s1, vec![1]);
    let c_ft = expect_pending(&e, "txhj_fangtianhuaji:extra_targets");
    assert!(!c_ft.spec.allow_cancel, "fangtianhuaji target pick is forced when alive >= 3");
    assert_eq!(c_ft.spec.max, 2);
    let _ = answer(&mut e, &c_ft, vec![ChoiceOption::Player(2), ChoiceOption::Player(3)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5);
    assert_eq!(e.debug_hp(2), 5);
    assert_eq!(e.debug_hp(3), 5);
    // 2nd Sha must NOT open `txhj_fangtianhuaji:extra_targets`
    let s2 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e, 0, s2, vec![1]);
    assert!(e.open_choice().map(|c| c.spec.prompt) != Some("txhj_fangtianhuaji:extra_targets"));
    drain(&mut e);

    // 2. txhj_xuwangzhimian: Draw phase draws +2 (total 4), hand limit reduced by 1
    let mut e_xw = start_taixu(0x1207, 2);
    let _ = equip_named(&mut e_xw, 0, "txhj_xuwangzhimian");
    e_xw.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e_xw);
    drain(&mut e_xw);
    assert_eq!(e_xw.debug_hand(0).len(), 4, "Draw phase 2 + xuwang 2 = 4 cards");
    assert_eq!(e_xw.state_core().hand_limit(e_xw.catalog(), 0), 5, "max_hp 6 - 1 = 5");

    // 3. txhj_piliche: DamageEnd reduces target's max_hp by damage amount (cancel keeps max_hp)
    let mut e_pl = start_taixu(0x1208, 2);
    let plc = equip_named(&mut e_pl, 0, "txhj_piliche");
    let s_pl = place_kind_suit(&mut e_pl, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_pl, 0, s_pl, vec![1]);
    let c_pl = expect_pending(&e_pl, "txhj_piliche:confirm");
    assert_eq!(ai_plan(&e_pl, 0), GameAction::Answer {
        choice_id: c_pl.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let mut e_pl_cancel = e_pl.clone();
    let _ = answer(&mut e_pl_cancel, &c_pl, Vec::new());
    drain(&mut e_pl_cancel);
    assert_eq!(e_pl_cancel.state_core().player(1).max_hp, 6);

    let e_pl_ok = save_restore_and_compare_pending(&e_pl, &c_pl, vec![ChoiceOption::Confirm]);
    assert_eq!(e_pl_ok.state_core().player(1).max_hp, 5);
    // Unequipping txhj_piliche removes it without entering discard
    let mut e_rm = start_taixu(0x1209, 2);
    let plc2 = equip_named(&mut e_rm, 0, "txhj_piliche");
    let _ = equip_kind(&mut e_rm, 0, CardKind::Zhuge);
    assert!(!e_rm.state_core().discard.contains(&plc2));
    assert!(!e_rm.debug_equips(0).contains(&plc2));
    let _ = plc;
}

/// Upstream: `extension_card.js:501, 1733` (`txhj_liulongcanjia`), `642, 1781` (`txhj_juechenjinge`),
/// `1263` (`txhj_baihu`), `580, 1743` (`txhj_leijimu`).
#[test]
fn l11_equips_mounts_and_leijimu() {
    // 1. Mount distances & liulong replacement:
    let mut e = start_versus(0x120a, &[&[0, 1], &[2, 3]], &[Faction::Wei, Faction::Shu]);
    let base_02 = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(base_02, 2);

    // `txhj_liulongcanjia` on seat 0 replaces both HorsePlus & HorseMinus, gives from-1 and to+1
    let _h_plus = equip_kind(&mut e, 0, CardKind::HorsePlus);
    let _h_minus = equip_kind(&mut e, 0, CardKind::HorseMinus);
    let ll = equip_named(&mut e, 0, "txhj_liulongcanjia");
    assert_eq!(e.debug_equips(0), vec![ll], "liulong must replace both existing mounts");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1, "from -1");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 0), 3, "to +1");
    let extra_mount = place_kind(&mut e, 0, CardKind::HorseMinus);
    assert!(eval_card_target_canceled(
        e.state_core(),
        e.catalog(),
        0,
        0,
        extra_mount,
        CardKind::HorseMinus,
        DamageNature::Normal,
    ));

    // `txhj_baihu`: to +2 (not from -1)
    let _ = equip_named(&mut e, 2, "txhj_baihu");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 3, 2), 3, "baihu +2 to seat 2");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 3), 1, "baihu no -1 from seat 2");

    // `txhj_juechenjinge`: to +2 self, +1 for alive friend (seat 1 gets +1 from seat 0!)
    let mut e_jc = start_versus(0x120b, &[&[0, 1], &[2, 3]], &[Faction::Wei, Faction::Shu]);
    let _ = equip_named(&mut e_jc, 0, "txhj_juechenjinge");
    assert_eq!(e_jc.state_core().computed_distance(e_jc.catalog(), 2, 0), 4, "2 base + 2 juechen");
    assert_eq!(e_jc.state_core().computed_distance(e_jc.catalog(), 2, 1), 2, "1 base + 1 ally aura");

    // 2. txhj_leijimu: Normal Sha -> confirm turns into LeiSha (Thunder), triggers chain!
    let mut e_lm = start_taixu(0x120c, 3);
    let _ = equip_named(&mut e_lm, 0, "txhj_leijimu");
    e_lm.state_core_mut().player_mut(1).linked = true;
    e_lm.state_core_mut().player_mut(2).linked = true;
    let norm_sha = place_kind_suit(&mut e_lm, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_lm, 0, norm_sha, vec![1]);
    let c_lm = expect_pending(&e_lm, "txhj_leijimu:confirm");
    let _ = answer(&mut e_lm, &c_lm, vec![ChoiceOption::Confirm]);
    drain(&mut e_lm);
    assert_eq!(e_lm.debug_hp(1), 5);
    assert_eq!(e_lm.debug_hp(2), 5, "Thunder Sha from leijimu must propagate chain");

    // Counterexample: FireSha must NOT trigger txhj_leijimu
    let mut e_lm2 = start_taixu(0x120d, 2);
    let _ = equip_named(&mut e_lm2, 0, "txhj_leijimu");
    let fire_sha = place_kind(&mut e_lm2, 0, CardKind::FireSha);
    let _ = play_card(&mut e_lm2, 0, fire_sha, vec![1]);
    assert!(e_lm2.open_choice().map(|c| c.spec.prompt) != Some("txhj_leijimu:confirm"));
    drain(&mut e_lm2);
}

/// Upstream: `extension_card.js:821, 1798` (`txhj_guofengyupao`), `880, 1815` (`txhj_feilong`),
/// `923, 1850` (`txhj_dinglanyemingzhu`), `1013, 1868` (`txhj_chiyanzhenhunqin`), `1123, 1880` (`txhj_lingbaoxianhu`),
/// `1137, 2339-2381` (`txhj_chongyingshenfu`), `1152, 1593-1637` (`txminguangkai`), `1201, 2429` (`txhj_taijifuchen`).
#[test]
fn l11_equips_armor_weapons_feilong_mingzhu_and_taijifuchen() {
    // 1. txhj_guofengyupao: cancels ordinary trick from another seat; sealed by txhj_noda_axe2 (`is_target_unequip2`)
    let mut e_gf = start_taixu(0x120e, 2);
    let _ = equip_named(&mut e_gf, 1, "txhj_guofengyupao");
    let guohe = place_kind_suit(&mut e_gf, 0, CardKind::Dismantle, Suit::Spade);
    assert!(eval_card_target_canceled(
        e_gf.state_core(),
        e_gf.catalog(),
        0,
        1,
        guohe,
        CardKind::Dismantle,
        DamageNature::Normal,
    ));
    let _axe_gf = equip_named(&mut e_gf, 0, "txhj_xingtianpojunfu");
    let sha_gf = place_kind(&mut e_gf, 0, CardKind::Sha);
    let gfa = place_kind(&mut e_gf, 0, CardKind::Tao);
    let gfb = place_kind(&mut e_gf, 0, CardKind::Jiu);
    let _ = play_card(&mut e_gf, 0, sha_gf, vec![1]);
    let c_gf = expect_pending(&e_gf, "txhj_xingtianpojunfu:discard_cards");
    let _ = answer(&mut e_gf, &c_gf, vec![ChoiceOption::Card(gfa), ChoiceOption::Card(gfb)]);
    drain(&mut e_gf);
    assert!(!eval_card_target_canceled(
        e_gf.state_core(),
        e_gf.catalog(),
        0,
        1,
        guohe,
        CardKind::Dismantle,
        DamageNature::Normal,
    ));

    // 2. txhj_feilong & txhj_feilong_skill & txhj_feilong_skill_blank:
    // Kills non-last enemy (in 3-seat versus [0] vs [1,2]), revives victim at 3/3 as Wei with locked blank
    let mut e_fl = start_versus(0x120f, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    let _ = equip_named(&mut e_fl, 0, "txhj_feilong");
    e_fl.debug_set_hp(1, 1);
    let s_kill = place_kind_suit(&mut e_fl, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_fl, 0, s_kill, vec![1]);
    let c_fl = answer_pass_until(&mut e_fl, "txhj_feilong:confirm");
    let e_fl_done = save_restore_and_compare_pending(&e_fl, &c_fl, vec![ChoiceOption::Confirm]);
    assert!(e_fl_done.debug_alive(1), "txhj_feilong revives victim");
    assert_eq!(e_fl_done.debug_hp(1), 3);
    assert_eq!(e_fl_done.state_core().player(1).max_hp, 3);
    assert!(e_fl_done.debug_has_skill(1, "txhj_feilong_skill_blank"));
    assert!(e_fl_done.state_core().is_friend_of(e_fl_done.catalog(), 0, 1));
    // Taking damage clears `txhj_feilong_skill_blank`:
    let mut e_fl_dmg = e_fl_done;
    e_fl_dmg.debug_grant(0, "咆哮");
    let s_clear = place_kind_suit(&mut e_fl_dmg, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_fl_dmg, 0, s_clear, vec![1]);
    drain(&mut e_fl_dmg);
    assert!(!e_fl_dmg.debug_has_skill(1, "txhj_feilong_skill_blank"));

    // Counterexample: killing last enemy does NOT prompt txhj_feilong
    let mut e_last = start_versus(0x1210, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    let _ = equip_named(&mut e_last, 0, "txhj_feilong");
    e_last.debug_set_hp(1, 1);
    let s_end = place_kind_suit(&mut e_last, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_last, 0, s_end, vec![1]);
    drain(&mut e_last);
    assert!(!e_last.debug_alive(1));

    // 3. txhj_dinglanyemingzhu: zhiheng up to max_hp (treasure excluded); once per turn
    let mut e_mz = start_taixu(0x1211, 2);
    let mz = equip_named(&mut e_mz, 0, "txhj_dinglanyemingzhu");
    let a = place_kind_suit(&mut e_mz, 0, CardKind::Sha, Suit::Spade);
    let b = place_kind_suit(&mut e_mz, 0, CardKind::Shan, Suit::Heart);
    let _ = activate_skill(&mut e_mz, 0, "txhj_dinglanyemingzhu");
    let c_mz = expect_pending(&e_mz, "txhj_dinglanyemingzhu:discard_cards");
    assert!(!c_mz.spec.options.contains(&ChoiceOption::Card(mz)));
    let _ = answer(&mut e_mz, &c_mz, vec![ChoiceOption::Card(a), ChoiceOption::Card(b)]);
    drain(&mut e_mz);
    assert_eq!(e_mz.debug_hand(0).len(), 2);
    assert!(matches!(
        activate_skill(&mut e_mz, 0, "txhj_dinglanyemingzhu"),
        CommandOutcome::Rejected(_)
    ));

    // 4. txhj_chiyanzhenhunqin + txhj_lingbaoxianhu + txhj_chongyingshenfu + txminguangkai + txhj_taijifuchen
    let mut e_combo = start_taixu(0x1212, 3);
    e_combo.debug_set_hp(0, 4); // injured
    let _ = equip_named(&mut e_combo, 0, "txhj_chiyanzhenhunqin"); // turns damage to Fire
    let _ = equip_named(&mut e_combo, 0, "txhj_lingbaoxianhu"); // replaces weapon! test separately:
    // First test lingbaoxianhu on death & >1 dmg:
    e_combo.state_core_mut().player_mut(1).linked = true;
    e_combo.debug_set_hp(2, 1);
    let jiu0 = place_kind_suit(&mut e_combo, 0, CardKind::Jiu, Suit::Spade);
    let _ = play_card(&mut e_combo, 0, jiu0, Vec::new());
    let s_lb = place_kind_suit(&mut e_combo, 0, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e_combo, 0, s_lb, vec![2]);
    drain(&mut e_combo);
    // Damage 2 to seat 2 (>1 dmg -> +1 max_hp +1 hp) AND death of seat 2 (+1 max_hp +1 hp) -> 6+2=8, 4+2=6!
    assert_eq!(e_combo.state_core().player(0).max_hp, 8);
    assert_eq!(e_combo.debug_hp(0), 6);

    // Chiyanzhenhunqin Fire damage triggers chain on seat 1 + 2!
    let mut e_cy = start_taixu(0x1213, 3);
    let _ = equip_named(&mut e_cy, 0, "txhj_chiyanzhenhunqin");
    e_cy.state_core_mut().player_mut(1).linked = true;
    e_cy.state_core_mut().player_mut(2).linked = true;
    let s_cy = place_kind_suit(&mut e_cy, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_cy, 0, s_cy, vec![1]);
    drain(&mut e_cy);
    assert_eq!(e_cy.debug_hp(1), 5);
    assert_eq!(e_cy.debug_hp(2), 5, "chiyanzhenhunqin turns damage Fire and conducts chains");

    // Chongyingshenfu: 1st Sha takes full 1 damage and records `sha`; 2nd Sha reduced by 1 -> 0!
    let mut e_cs = start_taixu(0x1214, 2);
    let _ = equip_named(&mut e_cs, 1, "txhj_chongyingshenfu");
    e_cs.debug_grant(0, "咆哮");
    let s_c1 = place_kind_suit(&mut e_cs, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_cs, 0, s_c1, vec![1]);
    drain(&mut e_cs);
    assert_eq!(e_cs.debug_hp(1), 5);
    assert!(e_cs.debug_has_skill(1, "txhj_chongyingshenfu_effect"));
    let s_c2 = place_kind_suit(&mut e_cs, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_cs, 0, s_c2, vec![1]);
    drain(&mut e_cs);
    assert_eq!(e_cs.debug_hp(1), 5, "2nd damage from same card name `sha` prevented");

    // Txminguangkai: cancels FireSha; blocks linking when injured (full HP CAN still link)
    let mut e_mk = start_taixu(0x1215, 2);
    let _ = equip_named(&mut e_mk, 1, "txminguangkai");
    assert!(!txminguangkai_blocks_link(e_mk.state_core(), e_mk.catalog(), 1));
    e_mk.debug_set_hp(1, 5);
    assert!(txminguangkai_blocks_link(e_mk.state_core(), e_mk.catalog(), 1));
    let fsha = place_kind(&mut e_mk, 0, CardKind::FireSha);
    let _ = play_card(&mut e_mk, 0, fsha, vec![1]);
    drain(&mut e_mk);
    assert_eq!(e_mk.debug_hp(1), 5, "FireSha cancelled by txminguangkai");

    // Txhj_taijifuchen & gx_taijifuchen: target discards same suit -> owner gains it; Pass -> unrespondable
    let mut e_tj = start_taixu(0x1216, 2);
    let _ = equip_named(&mut e_tj, 0, "txhj_taijifuchen");
    let s_tj = place_kind_suit(&mut e_tj, 0, CardKind::Sha, Suit::Heart);
    let t_heart = place_kind_suit(&mut e_tj, 1, CardKind::Tao, Suit::Heart);
    let _ = play_card(&mut e_tj, 0, s_tj, vec![1]);
    let c_tj = expect_pending(&e_tj, "txhj_taijifuchen:target_discard");
    let mut e_tj_pass = e_tj.clone();
    let _ = answer(&mut e_tj_pass, &c_tj, vec![ChoiceOption::Pass]);
    drain(&mut e_tj_pass);
    assert_eq!(e_tj_pass.debug_hp(1), 5, "passing taijifuchen leaves Sha unrespondable");
    let _ = answer(&mut e_tj, &c_tj, vec![ChoiceOption::Card(t_heart)]);
    drain(&mut e_tj);
    assert!(e_tj.debug_hand(0).contains(&t_heart), "same-suit discarded card gained by Sha owner");

    // gx_taijifuchen standalone sub-skill test
    let mut e_gx = start_taixu(0x1217, 2);
    e_gx.debug_grant(0, "gx_taijifuchen");
    let s_gx = place_kind_suit(&mut e_gx, 0, CardKind::Sha, Suit::Club);
    let shan_gx = place_kind_suit(&mut e_gx, 1, CardKind::Shan, Suit::Diamond);
    let _ = play_card(&mut e_gx, 0, s_gx, vec![1]);
    let c_gx = expect_pending(&e_gx, "gx_taijifuchen:target_discard");
    let _ = answer(&mut e_gx, &c_gx, Vec::new());
    drain(&mut e_gx);
    assert_eq!(e_gx.debug_hp(1), 5, "cancelled gx_taijifuchen makes Sha unrespondable despite Shan");
    assert!(e_gx.debug_hand(1).contains(&shan_gx));
}

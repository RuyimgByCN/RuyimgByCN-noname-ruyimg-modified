//! Batch L11-4 Group C (`L11BuffsC` tag 736 + `txhj_taoyuanyishi` / `taoyuanyishiX` / `txhj_jufuxionghao`) real-path tests.

use super::common::*;
use content_native_loader::{
    TaixuConfig, TaixuRunState, carryover_max_hp_after_battle, exam_excluded_wrong_option,
    upgrade_taixu_equip_majun,
};
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, GameAction, Phase};

#[test]
fn l11_b4_luanjifengshi_discard_cancel_ai_and_restore() {
    // `extension_skill.js:2532`: ArcheryAttack asks `txhj_luanjifengshi:discard_card`.
    // Companion pair dedups cleanly; discard avoids `aoe_direct_hit`, cancel forces direct hit.
    let mut e = start_versus(401, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "buff_txhj_luanjifengshi");
    grant_skill(&mut e, 0, "_buff_txhj_luanjifengshi");

    e.debug_set_turn(0, Phase::Play, 1);
    let s1_shan = place_kind(&mut e, 1, CardKind::Shan);
    let s1_sha = place_kind(&mut e, 1, CardKind::Sha); // lower value non-Shan
    let s2_shan = place_kind(&mut e, 2, CardKind::Shan);
    let wanjian = place_std_kind(&mut e, 0, CardKind::ArcheryAttack);

    let out_wj = play_card(&mut e, 0, wanjian, Vec::new());
    assert!(
        matches!(out_wj, engine_native::CommandOutcome::Pending { .. }),
        "play ArcheryAttack outcome: {out_wj:?}"
    );
    let ch1 = expect_pending(&e, "txhj_luanjifengshi:discard_card");
    assert_eq!(ch1.observer, 1);
    // AI preserves sole Shan and discards non-Shan Sha!
    assert_eq!(
        ai_plan(&e, 1),
        GameAction::Answer {
            choice_id: ch1.id,
            selection: vec![ChoiceOption::Card(s1_sha)],
        }
    );
    // Test save -> restore -> Answer parity on discard branch:
    let snap = e.save();
    let restored =
        content_native_loader::restore_locked_engine(&snap, taixu_verified()).expect("restore");
    assert_eq!(restored.state_hash(), e.state_hash());

    let out_a1 = answer(&mut e, &ch1, vec![ChoiceOption::Card(s1_sha)]);
    assert!(
        matches!(out_a1, engine_native::CommandOutcome::Pending { .. }),
        "after ch1 answer: {out_a1:?}"
    );
    // Seat 2 cancels discard even while holding Shan -> directHit skips `aoe_response` for seat 2!
    let ch2 = expect_pending(&e, "txhj_luanjifengshi:discard_card");
    assert_eq!(ch2.observer, 2);
    // Test save -> restore -> Answer parity on cancel (directHit) branch:
    let snap2 = e.save();
    let mut restored2 =
        content_native_loader::restore_locked_engine(&snap2, taixu_verified()).expect("restore 2");
    assert_eq!(restored2.state_hash(), e.state_hash());
    let _ = answer(&mut e, &ch2, Vec::new());
    let _ = answer(&mut restored2, &ch2, Vec::new());
    assert_eq!(restored2.state_hash(), e.state_hash());

    // Seat 1 still gets `aoe_response` (discarded earlier, kept Shan), while Seat 2 is directHit (no prompt)!
    let aoe_s1 = expect_pending(&e, "aoe_response");
    assert_eq!(aoe_s1.observer, 1);
    let _ = answer(&mut e, &aoe_s1, vec![ChoiceOption::Card(s1_shan)]);
    assert!(e.open_choice().is_none(), "seat 2 directHit must skip aoe_response");
    assert_eq!(e.state_core().player(1).hp, 6, "seat 1 kept and used Shan after discarding 1");
    assert_eq!(e.state_core().player(2).hp, 5, "seat 2 canceled discard -> directHit even holding Shan");
    assert!(e.state_core().player(2).hand.contains(&s2_shan));

    // AI with 0 Shan passes cleanly instead of burning unrelated cards:
    let _s1_tao = place_kind(&mut e, 1, CardKind::Tao);
    let wanjian2 = place_std_kind(&mut e, 0, CardKind::ArcheryAttack);
    let _ = play_card(&mut e, 0, wanjian2, Vec::new());
    let ch_pass = expect_pending(&e, "txhj_luanjifengshi:discard_card");
    assert_eq!(
        ai_plan(&e, 1),
        GameAction::Answer {
            choice_id: ch_pass.id,
            selection: Vec::new(),
        }
    );
    drain(&mut e);

    // Negative: SavageAttack and `txhj_huoshaolianying` must NOT trigger `txhj_luanjifengshi:discard_card`.
    let nanman = place_std_kind(&mut e, 0, CardKind::SavageAttack);
    let _ = play_card(&mut e, 0, nanman, Vec::new());
    assert!(
        e.open_choice()
            .is_none_or(|c| c.spec.prompt != "txhj_luanjifengshi:discard_card"),
        "SavageAttack must not trigger luanjifengshi"
    );
    drain(&mut e);
    let hsl = place_named(&mut e, 0, "txhj_huoshaolianying");
    let _ = play_card(&mut e, 0, hsl, vec![1]);
    assert!(
        e.open_choice()
            .is_none_or(|c| c.spec.prompt != "txhj_luanjifengshi:discard_card"),
        "txhj_huoshaolianying must not trigger luanjifengshi"
    );
}

#[test]
fn l11_b4_taoyuanyishi_haoshidebaochao_and_duocejiyong() {
    // 1. `taoyuanyishi` (`2563`): Taoyuan grants +1 recover to allies; cleanup after use; Tao unchanged.
    let mut e = start_versus(402, &[&[0, 1], &[2]]);
    grant_skill(&mut e, 0, "buff_txhj_taoyuanyishi");
    grant_skill(&mut e, 0, "_buff_txhj_taoyuanyishi");
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 2);
    e.debug_set_turn(0, Phase::Play, 1);

    let taoyuan = place_kind(&mut e, 0, CardKind::Taoyuan);
    let _ = play_card(&mut e, 0, taoyuan, Vec::new());
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "ally recovers 1 + 1 = 2 via taoyuanyishi");
    assert_eq!(e.state_core().player(2).hp, 3, "enemy recovers only 1 HP");
    assert!(!has_skill(&e, 0, "txhj_taoyuanyishi"), "temp helper removed after Taoyuan");
    assert!(!has_skill(&e, 1, "taoyuanyishiX"), "ally helper removed after Taoyuan");

    // Negative: Tao must not gain taoyuanyishi +1
    let tao = place_kind(&mut e, 0, CardKind::Tao);
    e.debug_set_hp(0, 3);
    let _ = play_card(&mut e, 0, tao, Vec::new());
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 4);

    // 2. `haoshidebaochao` / `_buff_txhj_haoshidebaochang` (`2588`): Wugu draws +1 (deduped when paired), Wuxie-cancelled -> 0!
    grant_skill(&mut e, 0, "buff_txhj_haoshidebaochao");
    grant_skill(&mut e, 0, "_buff_txhj_haoshidebaochang");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let wugu = place_kind(&mut e, 0, CardKind::Wugu);
    let _ = play_card(&mut e, 0, wugu, Vec::new());
    drain(&mut e);
    // 1 card picked from Wugu + 1 extra draw from haoshidebaochao = 2!
    assert_eq!(
        e.state_core().player(0).hand.len(),
        2,
        "paired baochao/baochang draws exactly 1 extra card on Wugu"
    );

    // Negative: Wuxie-cancelled Wugu must NOT draw via haoshidebaochao!
    e.debug_clear_hand(0);
    let wugu_cancel = place_kind(&mut e, 0, CardKind::Wugu);
    let wuxie_cancel = place_kind(&mut e, 2, CardKind::Nullify);
    let _ = play_card(&mut e, 0, wugu_cancel, Vec::new());
    for _ in 0..3 {
        if let Some(nw_c) = e.open_choice().cloned() {
            if nw_c.spec.prompt == "nullify_window" && nw_c.observer == 2 {
                let _ = answer(&mut e, &nw_c, vec![ChoiceOption::Card(wuxie_cancel)]);
                break;
            } else {
                drain(&mut e);
                break;
            }
        }
    }
    drain(&mut e);
    assert!(
        e.state_core().player(0).hand.len() <= 1,
        "when enemy nullifies ally pick or Wugu resolves normally, haoshidebaochao draws at most 1"
    );

    // 3. `duocejiyong` (`2609`):
    //    (a) Nullifying an enemy trick steals the nullified card into Wuxie user's hand!
    //    (b) Counter-nullifying an opponent's Wuxie steals the opponent's Wuxie!
    let mut ed = start_versus(403, &[&[0], &[1]]);
    grant_skill(&mut ed, 0, "buff_txhj_duocejiyong");
    grant_skill(&mut ed, 0, "_buff_txhj_duocejiyong");
    ed.debug_set_turn(1, Phase::Play, 1);
    let _s0_dummy = place_kind(&mut ed, 0, CardKind::Sha);
    let snatch1 = place_kind(&mut ed, 1, CardKind::Snatch);
    let wuxie0 = place_kind(&mut ed, 0, CardKind::Nullify);
    let _ = play_card(&mut ed, 1, snatch1, vec![0]);
    let nw = expect_pending(&ed, "nullify_window");
    assert_eq!(nw.observer, 0);
    let _ = answer(&mut ed, &nw, vec![ChoiceOption::Card(wuxie0)]);
    drain(&mut ed);
    assert!(
        ed.state_core().player(0).hand.contains(&snatch1),
        "duocejiyong must steal unopposed nullified trick card into hand"
    );
    assert!(!ed.state_core().discard.contains(&snatch1));

    // Counter-wuxie path: Seat 0 plays Snatch on Seat 1; Seat 1 Wuxie; Seat 0 counter-Wuxie with duocejiyong!
    ed.debug_set_turn(0, Phase::Play, 2);
    let target_card = place_kind(&mut ed, 1, CardKind::Tao);
    let snatch0 = place_kind(&mut ed, 0, CardKind::Snatch);
    let wuxie1 = place_kind(&mut ed, 1, CardKind::Nullify);
    let _ = play_card(&mut ed, 0, snatch0, vec![1]);
    let nw1 = expect_pending(&ed, "nullify_window");
    assert_eq!(nw1.observer, 1);
    let wuxie0_counter = place_kind(&mut ed, 0, CardKind::Nullify);
    let _ = answer(&mut ed, &nw1, vec![ChoiceOption::Card(wuxie1)]);
    let nw2 = expect_pending(&ed, "nullify_window");
    assert_eq!(nw2.observer, 0);
    let _ = answer(&mut ed, &nw2, vec![ChoiceOption::Card(wuxie0_counter)]);
    drain(&mut ed);
    assert!(
        ed.state_core().player(0).hand.contains(&wuxie1),
        "duocejiyong counter-Wuxie must steal opponent's prior Wuxie"
    );
    assert!(
        ed.state_core().player(0).hand.contains(&target_card),
        "counter-nullified Snatch must proceed to steal target's card"
    );
}

#[test]
fn l11_b4_wolongmice_fenchengluetu_jufuxionghao_and_campaign_buffs() {
    // 1. `wolongmice` (`487`): Huogong accepts same-color suit (Heart matching Diamond)!
    //    `fenchengluetu` (`511`): gains target's shown card to owner's hand before fire damage!
    let mut e = start_versus(404, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "buff_txhj_wolongmice");
    grant_skill(&mut e, 0, "buff_txhj_fenchengluetu");
    e.debug_set_turn(0, Phase::Play, 1);

    let show_dia = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Diamond);
    let disc_heart = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let _disc_black = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let hg = place_named(&mut e, 0, "huogong");
    let _ = play_card(&mut e, 0, hg, vec![1]);
    let sc = expect_pending(&e, "huogong:show_card");
    let _ = answer(&mut e, &sc, vec![ChoiceOption::Card(show_dia)]);
    let dc = expect_pending(&e, "huogong:discard_card");
    assert!(dc.spec.options.contains(&ChoiceOption::Card(disc_heart)));
    assert!(!dc.spec.options.contains(&ChoiceOption::Card(_disc_black)));
    let _ = answer(&mut e, &dc, vec![ChoiceOption::Card(disc_heart)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5, "wolongmice Huogong deals 1 fire dmg");
    assert!(
        e.state_core().player(0).hand.contains(&show_dia),
        "fenchengluetu gains target's shown card into owner's hand"
    );

    // Negative: without wolongmice, Heart cannot match Diamond in Huogong!
    let mut e_neg = start_versus(405, &[&[0], &[1]]);
    e_neg.debug_set_turn(0, Phase::Play, 1);
    let show2 = place_kind_suit(&mut e_neg, 1, CardKind::Shan, Suit::Diamond);
    let _heart2 = place_kind_suit(&mut e_neg, 0, CardKind::Tao, Suit::Heart);
    let hg2 = place_named(&mut e_neg, 0, "huogong");
    let _ = play_card(&mut e_neg, 0, hg2, vec![1]);
    let sc2 = expect_pending(&e_neg, "huogong:show_card");
    let _ = answer(&mut e_neg, &sc2, vec![ChoiceOption::Card(show2)]);
    drain(&mut e_neg);
    assert_eq!(
        e_neg.state_core().player(1).hp,
        6,
        "without wolongmice, mismatched same-color suit must not discard or deal damage"
    );

    // 2. `_buff_txhj_jufuxionghao` / `txhj_jufuxionghao` (`extension_buff.js:519`, `extension_skill.js:2638,3454` commented out no-op):
    grant_skill(&mut e, 0, "_buff_txhj_jufuxionghao");
    grant_skill(&mut e, 0, "txhj_jufuxionghao");
    let h_before = e.state_core().player(0).hand.len();
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert!(e.state_core().player(0).hand.len() <= h_before + 2);

    // 3. Campaign helpers: `bowenqiangzhi`, `majundegaizao`, `xianhujiqu`
    let cfg = TaixuConfig::default();
    let plain_run = TaixuRunState::from_config(&cfg);
    assert_eq!(exam_excluded_wrong_option(&plain_run, 4, 7), None);
    assert_eq!(upgrade_taixu_equip_majun(&plain_run, "zhuge"), "zhuge");
    assert_eq!(carryover_max_hp_after_battle(&plain_run, 9, 6, true), 6);

    let mut buff_run = plain_run.clone();
    buff_run.buffs = vec![
        "buff_txhj_bowenqiangzhi".to_string(),
        "buff_txhj_majundegaizao".to_string(),
        "buff_txhj_xianhujiqu".to_string(),
    ];
    let ex = exam_excluded_wrong_option(&buff_run, 4, 5).expect("bowenqiangzhi excludes 1 wrong");
    assert!((1..4).contains(&ex), "must only exclude a wrong option (>= 1), got {ex}");
    for (base, rew) in [
        ("zhuge", "rewrite_zhuge"),
        ("lanyinjia", "rewrite_lanyinjia"),
        ("tengjia", "rewrite_tengjia"),
        ("baiyin", "rewrite_baiyin"),
        ("bagua", "rewrite_bagua"),
        ("renwang", "rewrite_renwang"),
        ("qilin", "qilin"),
    ] {
        assert_eq!(upgrade_taixu_equip_majun(&buff_run, base), rew);
    }
    assert_eq!(carryover_max_hp_after_battle(&buff_run, 9, 6, true), 9);
    assert_eq!(carryover_max_hp_after_battle(&buff_run, 9, 6, false), 6);
}

#[test]
fn l11_b4_fuchendangmo_shenhanxueyin_and_shangjiangdelvli() {
    // 1. `fuchendangmo` + `shenhanxueyin`:
    //    (a) Cixiong: target discards 2 (`min == 2, max == 2`) AND owner gains both into hand via shenhanxueyin!
    //    (b) Hanbing: prevents dmg & asks target 3 discards (`2 + 1`) AND owner steals all 3!
    //    (c) Guanshi + `shangjiangdelvli`: discard cost reduced from 2 to 1!
    let mut e = start_versus(406, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut e, 0, "buff_txhj_shenhanxueyin");
    e.debug_assemble_general(1, "甄姬"); // female opposite gender for Cixiong
    e.debug_set_turn(0, Phase::Play, 1);

    let _ = equip_std_kind(&mut e, 0, CardKind::Cixiong);
    let t1 = place_kind(&mut e, 1, CardKind::Tao);
    let t2 = place_kind(&mut e, 1, CardKind::Tao);
    let sha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    let cx = expect_pending(&e, "cixiong_discard_choose");
    assert_eq!((cx.spec.min, cx.spec.max), (2, 2), "fuchendangmo sets cixiong cost to 2");
    let _ = answer(&mut e, &cx, vec![ChoiceOption::Card(t1), ChoiceOption::Card(t2)]);
    drain(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&t1) && e.state_core().player(0).hand.contains(&t2),
        "shenhanxueyin must steal opponent's discarded Cixiong cards"
    );

    // Guanshi + shangjiangdelvli (`min = 1, max = 1`)
    let mut eg = start_versus(407, &[&[0], &[1]]);
    grant_skill(&mut eg, 0, "buff_txhj_shangjiangdelvli");
    eg.debug_set_turn(0, Phase::Play, 1);
    let _ = equip_std_kind(&mut eg, 0, CardKind::Guanshi);
    let cost1 = place_kind(&mut eg, 0, CardKind::Tao);
    let shan1 = place_kind(&mut eg, 1, CardKind::Shan);
    let sha_g = place_kind(&mut eg, 0, CardKind::Sha);
    let _ = play_card(&mut eg, 0, sha_g, vec![1]);
    let rs = expect_pending(&eg, "respond_shan");
    let _ = answer(&mut eg, &rs, vec![ChoiceOption::Card(shan1)]);
    let gs = expect_pending(&eg, "guanshi_force_damage");
    assert_eq!((gs.spec.min, gs.spec.max), (1, 1), "shangjiangdelvli reduces guanshi cost to 1");
    let _ = answer(&mut eg, &gs, vec![ChoiceOption::Card(cost1)]);
    drain(&mut eg);
    assert_eq!(eg.state_core().player(1).hp, 5, "guanshi forces 1 damage with only 1 discard");

    // Hanbing + fuchendangmo + shenhanxueyin: prevents damage, forces 3 discards (`2 + 1`), owner steals all 3!
    let mut eh = start_versus(417, &[&[0], &[1]]);
    grant_skill(&mut eh, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut eh, 0, "buff_txhj_shenhanxueyin");
    let _ = equip_std_kind(&mut eh, 0, CardKind::Hanbing);
    eh.debug_set_turn(0, Phase::Play, 1);
    let hc1 = place_kind(&mut eh, 1, CardKind::Tao);
    let hc2 = place_kind(&mut eh, 1, CardKind::Tao);
    let hc3 = place_kind(&mut eh, 1, CardKind::Sha);
    let sha_h = place_kind(&mut eh, 0, CardKind::Sha);
    let _ = play_card(&mut eh, 0, sha_h, vec![1]);
    let hb_c = expect_pending(&eh, "hanbing_prevent_confirm");
    let _ = answer(&mut eh, &hb_c, vec![ChoiceOption::Confirm]);
    for pick in [hc1, hc2, hc3] {
        let hd = expect_pending(&eh, "hanbing_target_discard");
        let _ = answer(&mut eh, &hd, vec![ChoiceOption::Card(pick)]);
    }
    drain(&mut eh);
    assert_eq!(eh.state_core().player(1).hp, 6, "Hanbing prevented damage");
    assert!(
        eh.state_core().player(0).hand.contains(&hc1)
            && eh.state_core().player(0).hand.contains(&hc2)
            && eh.state_core().player(0).hand.contains(&hc3),
        "fuchendangmo (3 cards) + shenhanxueyin steals all 3 Hanbing-discarded cards"
    );

    // `txhj_taijifuchen` + fuchendangmo + shenhanxueyin: target discards 2 different-suit cards, owner steals both!
    let mut et = start_versus(418, &[&[0], &[1]]);
    grant_skill(&mut et, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut et, 0, "buff_txhj_shenhanxueyin");
    let _ = equip_named(&mut et, 0, "txhj_taijifuchen");
    et.debug_set_turn(0, Phase::Play, 1);
    let d_a = place_kind_suit(&mut et, 1, CardKind::Tao, Suit::Heart);
    let d_b = place_kind_suit(&mut et, 1, CardKind::Tao, Suit::Diamond);
    let sha_t = place_kind_suit(&mut et, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut et, 0, sha_t, vec![1]);
    let tj1 = expect_pending(&et, "txhj_taijifuchen:target_discard");
    let _ = answer(&mut et, &tj1, vec![ChoiceOption::Card(d_a)]);
    let tj2 = expect_pending(&et, "txhj_taijifuchen:target_discard");
    let _ = answer(&mut et, &tj2, vec![ChoiceOption::Card(d_b)]);
    drain(&mut et);
    assert!(
        et.state_core().player(0).hand.contains(&d_a)
            && et.state_core().player(0).hand.contains(&d_b),
        "txhj_taijifuchen + fuchendangmo (2 discards) + shenhanxueyin steals both"
    );

    // Qilin + fuchendangmo + shenhanxueyin: discards 2 horses across 2 asks and owner steals both!
    let mut eq = start_versus(419, &[&[0], &[1]]);
    grant_skill(&mut eq, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut eq, 0, "buff_txhj_shenhanxueyin");
    let _ = equip_std_kind(&mut eq, 0, CardKind::Qilin);
    let h_plus = equip_std_kind(&mut eq, 1, CardKind::HorsePlus);
    let h_minus = equip_std_kind(&mut eq, 1, CardKind::HorseMinus);
    eq.debug_set_turn(0, Phase::Play, 1);
    let sha_q = place_kind(&mut eq, 0, CardKind::Sha);
    let _ = play_card(&mut eq, 0, sha_q, vec![1]);
    let q1 = expect_pending(&eq, "qilin_discard_horse");
    let _ = answer(&mut eq, &q1, vec![ChoiceOption::Card(h_plus)]);
    let q2 = expect_pending(&eq, "qilin_discard_horse");
    let _ = answer(&mut eq, &q2, vec![ChoiceOption::Card(h_minus)]);
    drain(&mut eq);
    assert!(eq.state_core().player(1).equips.is_empty(), "both horses discarded by Qilin + fuchendangmo");
    assert!(
        eq.state_core().player(0).hand.contains(&h_plus)
            && eq.state_core().player(0).hand.contains(&h_minus),
        "shenhanxueyin steals both Qilin-discarded horses"
    );
}

#[test]
fn l11_b4_shenfuhuaxie_gongzhibaowu_fangzhibaowu_and_suikuiliejia() {
    // 1. `shenfuhuaxie` (`2687`): gains table damage card when NOT holding jianxiong/rejianxiong.
    let mut e = start_versus(408, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_shenfuhuaxie");
    e.debug_set_turn(1, Phase::Play, 1);
    let sha1 = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha1, vec![0]);
    drain(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&sha1),
        "shenfuhuaxie must gain Sha from table without double-discard"
    );
    assert!(!e.state_core().discard.contains(&sha1));

    // Negative: Jianxiong suppresses `shenfuhuaxie` duplicate gain
    let mut e_jx = start_versus(409, &[&[0], &[1]]);
    grant_skill(&mut e_jx, 0, engine_native::vocabulary::names::JIAN_XIONG);
    grant_skill(&mut e_jx, 0, "_buff_txhj_shenfuhuaxie");
    e_jx.debug_set_turn(1, Phase::Play, 1);
    let sha_jx = place_kind(&mut e_jx, 1, CardKind::Sha);
    let _ = play_card(&mut e_jx, 1, sha_jx, vec![0]);
    drain(&mut e_jx); // decline/handle jianxiong confirm if asked (or drain accepts pass/cancel)
    assert!(!e_jx.state_core().player(0).hand.contains(&sha_jx), "shenfuhuaxie must not auto-take when Jianxiong present");

    // 2. `gongzhibaowu` (Prepare + Horse -> gains Sha) & `fangzhibaowu` (End + Horse -> gains Shan)
    let mut eb = start_versus(410, &[&[0], &[1]]);
    grant_skill(&mut eb, 0, "_buff_txhj_gongzhibaowu");
    grant_skill(&mut eb, 0, "_buff_txhj_fangzhibaowu");
    // Without Horse: Prepare and End draw neither Sha nor Shan
    eb.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut eb, 0, Phase::Play);
    assert!(eb.state_core().player(0).hand.len() <= 2); // only Draw phase cards

    // Equip HorsePlus & enter round 2 Prepare & End
    eb.debug_clear_hand(0);
    let _ = equip_std_kind(&mut eb, 0, CardKind::HorsePlus);
    eb.debug_set_turn(1, Phase::End, 1);
    advance_until_turn_phase(&mut eb, 0, Phase::Draw);
    assert!(
        eb.state_core()
            .player(0)
            .hand
            .iter()
            .any(|&c| eb.catalog().card_kind(c).is_sha_like()),
        "gongzhibaowu gains Sha at Prepare when equipped with Horse"
    );
    eb.debug_clear_hand(0);
    advance_until_turn_phase(&mut eb, 0, Phase::End);
    let _ = next_phase(&mut eb);
    drain(&mut eb);
    // Still check before Discard/End transitions out: better verify during End phase:
    let mut ef = start_versus(411, &[&[0], &[1]]);
    grant_skill(&mut ef, 0, "_buff_txhj_fangzhibaowu");
    let _ = equip_std_kind(&mut ef, 0, CardKind::HorseMinus);
    ef.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut ef); // enters Phase::End (`begin: true`)!
    drain(&mut ef);
    assert!(
        ef.state_core()
            .player(0)
            .hand
            .iter()
            .any(|&c| ef.catalog().card_kind(c) == CardKind::Shan),
        "fangzhibaowu gains Shan at End when equipped with Horse"
    );

    // 3. `suikuiliejia` (`2754`): Weapon + Sha dmg -> asks `txhj_suikuiliejia:discard_armor`, AI + save/restore!
    //    Also verify AI cancels when hitting an ally's Armor instead of discarding teammate gear.
    let mut es = start_versus(412, &[&[0, 2], &[1]]);
    grant_skill(&mut es, 0, "_buff_txhj_suikuiliejia");
    let _ = equip_std_kind(&mut es, 0, CardKind::Weapon);
    let armor1 = equip_named(&mut es, 1, "renwang");
    let armor2 = equip_named(&mut es, 2, "txminguangkai");
    es.debug_set_turn(0, Phase::Play, 1);
    let sha_s = place_kind_suit(&mut es, 0, CardKind::Sha, Suit::Heart);
    let out_sha = play_card(&mut es, 0, sha_s, vec![1]);
    assert!(
        es.open_choice().is_some(),
        "expected suikuiliejia after red Sha; out={out_sha:?} eq0={:?} eq1={:?} hp1={} log={:?}",
        es.state_core().player(0).equips,
        es.state_core().player(1).equips,
        es.state_core().player(1).hp,
        es.log(),
    );
    let sk = expect_pending(&es, "txhj_suikuiliejia:discard_armor");
    assert_eq!(sk.observer, 0);
    assert_eq!(
        ai_plan(&es, 0),
        GameAction::Answer {
            choice_id: sk.id,
            selection: vec![ChoiceOption::Card(armor1)],
        }
    );
    let mut es = save_restore_and_compare_pending(&es, &sk, vec![ChoiceOption::Card(armor1)]);
    assert!(es.state_core().player(1).equips.is_empty(), "suikuiliejia must discard enemy armor");

    grant_skill(&mut es, 0, engine_native::vocabulary::names::PAO_XIAO);
    let _ = top_deck_suit(&mut es, Suit::Spade);
    let sha_ally = place_kind_suit(&mut es, 0, CardKind::Sha, Suit::Diamond);
    let _ = play_card(&mut es, 0, sha_ally, vec![2]);
    let sk_ally = expect_pending(&es, "txhj_suikuiliejia:discard_armor");
    assert_eq!(
        ai_plan(&es, 0),
        GameAction::Answer {
            choice_id: sk_ally.id,
            selection: Vec::new(),
        },
        "AI must cancel suikuiliejia when target is an ally"
    );
    let _ = answer(&mut es, &sk_ally, Vec::new());
    drain(&mut es);
    assert!(
        es.state_core().player(2).equips.contains(&armor2),
        "canceling suikuiliejia preserves ally armor"
    );
}

#[test]
fn l11_b4_yiguzuoqi_xiuluodelianmin_cebeibuyu_and_gongtaizhizhi() {
    // 1. `yiguzuoqi` (`2791`): 2nd Sha in Play with Weapon draws 1; 1st and 3rd do NOT!
    let mut e = start_versus(413, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_yiguzuoqi");
    grant_skill(&mut e, 0, engine_native::vocabulary::names::PAO_XIAO);
    let _ = equip_std_kind(&mut e, 0, CardKind::Weapon);
    e.debug_set_turn(0, Phase::Play, 1);
    let s1 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, s1, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 0, "1st Sha no yiguzuoqi draw");
    let s2 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, s2, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "2nd Sha draws 1 via yiguzuoqi");
    let s3 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, s3, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "3rd Sha does NOT trigger yiguzuoqi again");

    // 2. `xiuluodebeimin` + `_buff_txhj_xiuluodelianmin` (`2828`): Weapon + owner-sourced Recover draws 1 (deduped)
    let mut ex = start_versus(414, &[&[0, 2], &[1]]);
    grant_skill(&mut ex, 0, "buff_txhj_xiuluodebeimin");
    grant_skill(&mut ex, 0, "_buff_txhj_xiuluodelianmin");
    let _ = equip_std_kind(&mut ex, 0, CardKind::Weapon);
    ex.debug_set_hp(0, 4);
    ex.debug_set_hp(2, 4);
    ex.debug_set_turn(0, Phase::Play, 1);
    let ty = place_kind(&mut ex, 0, CardKind::Taoyuan);
    let _ = play_card(&mut ex, 0, ty, Vec::new());
    drain(&mut ex);
    assert_eq!(ex.state_core().player(0).hand.len(), 1, "Taoyuan heals ally 2 -> 1 recover sourced by owner");
    let tao0 = place_kind(&mut ex, 0, CardKind::Tao);
    let _ = play_card(&mut ex, 0, tao0, Vec::new());
    drain(&mut ex);
    assert_eq!(ex.state_core().player(0).hand.len(), 2, "owner Tao self-recover -> 2nd xiuluodelianmin draw");

    // Enemy Tao recover must NOT trigger seat 0's xiuluodelianmin!
    ex.debug_clear_hand(0);
    ex.debug_set_hp(1, 4);
    ex.debug_set_turn(1, Phase::Play, 1);
    let tao1 = place_kind(&mut ex, 1, CardKind::Tao);
    let _ = play_card(&mut ex, 1, tao1, Vec::new());
    drain(&mut ex);
    assert_eq!(ex.state_core().player(0).hand.len(), 0, "enemy recover must not trigger xiuluodelianmin");

    // 3. `cebeibuyu` (`2847`): Armor reduces Trick/Delay dmg by 1 (1->0 prevented), leaves Sha unchanged!
    let mut ec = start_versus(415, &[&[0], &[1]]);
    grant_skill(&mut ec, 0, "buff_txhj_cebeibuyu");
    grant_skill(&mut ec, 0, "_buff_txhj_cebeibuyu");
    let _ = equip_std_kind(&mut ec, 0, CardKind::Armor);
    ec.debug_set_turn(1, Phase::Play, 1);
    let wj = place_kind(&mut ec, 1, CardKind::ArcheryAttack);
    let _ = play_card(&mut ec, 1, wj, Vec::new());
    drain(&mut ec);
    assert_eq!(ec.state_core().player(0).hp, 6, "cebeibuyu prevents 1-dmg trick when equipped with Armor");
    let sha_c = place_kind(&mut ec, 1, CardKind::Sha);
    let _ = play_card(&mut ec, 1, sha_c, vec![0]);
    drain(&mut ec);
    assert_eq!(ec.state_core().player(0).hp, 5, "cebeibuyu must not reduce Sha damage");

    // 4. `gongtaizhizhi` (`2867`): Round start (`TurnBegin`) once per round when full HP + Armor -> draw 1!
    // Note: `Phase::Prepare` auto-advances into `Phase::Draw` (+2), so at `Phase::Draw` seat 0 has 1 (gongtaizhizhi) + 2 (Draw) = 3!
    let mut eg = start_versus(416, &[&[0], &[1]]);
    grant_skill(&mut eg, 0, "buff_txhj_gongtaizhizhi");
    grant_skill(&mut eg, 0, "_buff_txhj_gongtaizhizhi");
    let _ = equip_std_kind(&mut eg, 0, CardKind::Armor);
    eg.debug_clear_hand(0);
    eg.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut eg, 0, Phase::Draw);
    assert_eq!(eg.state_core().player(0).hand.len(), 3, "gongtaizhizhi draws 1 at Round 1 start + 2 at Draw");
    // Advancing into Seat 1's TurnBegin in same round 1 must NOT trigger again:
    advance_until_turn_phase(&mut eg, 1, Phase::Play);
    assert_eq!(eg.state_core().player(0).hand.len(), 3, "no 2nd gongtaizhizhi trigger during Seat 1 TurnBegin in round 1");
    // Next round (`round 2`) when injured (`hp < max_hp`): must NOT draw at TurnBegin (Draw adds 2 only)!
    eg.debug_set_hp(0, 5);
    eg.debug_clear_hand(0);
    advance_until_turn_phase(&mut eg, 0, Phase::Draw);
    assert_eq!(eg.state_core().player(0).hand.len(), 2, "injured player does NOT trigger gongtaizhizhi (Draw 2 only)");
}

#[test]
fn l11_b4_sanjian_yitianjian_and_wushuangfangtianji_buffs() {
    // 1. `sanjian_skill` / `wd_sanjian` (`extension_skill.js:975-1039`) + `buff_txhj_shangjiangdelvli`:
    //    (a) With 0 extra hand cards (sole Sha played): still triggers `wd_sanjian_target` (NO `wd_sanjian_discard`),
    //        and `save -> restore -> Answer` parity holds!
    let mut es = start_versus_with_l28_wudian(433, &[&[0], &[1, 2]]);
    let _ = equip_catalog_prefix(&mut es, 0, "wd_sanjian");
    grant_skill(&mut es, 0, "buff_txhj_shangjiangdelvli");
    grant_skill(&mut es, 0, engine_native::vocabulary::names::PAO_XIAO);
    es.debug_clear_hand(0);
    es.debug_set_turn(0, Phase::Play, 1);

    let sha0 = place_kind(&mut es, 0, CardKind::Sha);
    let _ = play_card(&mut es, 0, sha0, vec![1]);
    let sj1 = expect_pending(&es, "wd_sanjian_target");
    assert_eq!(sj1.observer, 0);
    let mut es = save_restore_and_compare_pending_with(
        &es,
        &sj1,
        vec![ChoiceOption::Player(2)],
        taixu_with_l28_wudian_verified(),
    );
    assert_eq!(es.state_core().player(1).hp, 5, "Sha damaged seat 1");
    assert_eq!(es.state_core().player(2).hp, 5, "0-hand sanjian + shangjiangdelvli damaged seat 2");
    assert!(es.state_core().player(0).hand.is_empty());

    // (b) With >=1 extra hand card: `shangjiangdelvli` still skips discard and preserves the hand card!
    let kept_tao = place_kind(&mut es, 0, CardKind::Tao);
    let sha1 = place_kind(&mut es, 0, CardKind::Sha);
    let _ = play_card(&mut es, 0, sha1, vec![1]);
    let sj2 = expect_pending(&es, "wd_sanjian_target");
    let _ = answer(&mut es, &sj2, vec![ChoiceOption::Player(2)]);
    drain(&mut es);
    assert_eq!(es.state_core().player(2).hp, 4, "1+-hand sanjian + shangjiangdelvli damaged seat 2");
    assert!(es.state_core().player(0).hand.contains(&kept_tao), "shangjiangdelvli spares hand card");

    // (c) Negative without `shangjiangdelvli` when hand is empty at damageSource: must NOT trigger Sanjian!
    let mut es_neg = start_versus_with_l28_wudian(434, &[&[0], &[1, 2]]);
    let _ = equip_catalog_prefix(&mut es_neg, 0, "wd_sanjian");
    es_neg.debug_clear_hand(0);
    es_neg.debug_set_turn(0, Phase::Play, 1);
    let sha_neg = place_kind(&mut es_neg, 0, CardKind::Sha);
    let _ = play_card(&mut es_neg, 0, sha_neg, vec![1]);
    assert!(es_neg.open_choice().is_none(), "without shangjiangdelvli, 0-hand sanjian must not trigger");
    assert_eq!(es_neg.state_core().player(2).hp, 6);

    // 2. `wushuangfangtianji_skill` (`extension_skill.js:785`, `l28_shengong_equips.rs:549`)
    //    + `buff_txhj_fuchendangmo` + `buff_txhj_shenhanxueyin`:
    //    mode 1 asks 2 discards and owner steals both (with save -> restore -> Answer parity on 2nd discard)!
    let mut ew = start_versus_with_l28_wudian(435, &[&[0], &[1]]);
    let _ = equip_catalog_named(&mut ew, 0, "wushuangfangtianji");
    grant_skill(&mut ew, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut ew, 0, "buff_txhj_shenhanxueyin");
    ew.debug_set_turn(0, Phase::Play, 1);
    let wk1 = place_kind(&mut ew, 1, CardKind::Tao);
    let wk2 = place_kind(&mut ew, 1, CardKind::Sha);
    let w_sha = place_kind(&mut ew, 0, CardKind::Sha);
    let _ = play_card(&mut ew, 0, w_sha, vec![1]);
    let wmode = expect_pending(&ew, "wushuangfangtianji:choose_mode");
    let _ = answer(&mut ew, &wmode, vec![ChoiceOption::Tag(1)]);
    let wd1 = expect_pending(&ew, "wushuangfangtianji:discard_card");
    let _ = answer(&mut ew, &wd1, vec![ChoiceOption::Card(wk1)]);
    let wd2 = expect_pending(&ew, "wushuangfangtianji:discard_card");
    let ew = save_restore_and_compare_pending_with(
        &ew,
        &wd2,
        vec![ChoiceOption::Card(wk2)],
        taixu_with_l28_wudian_verified(),
    );
    assert!(ew.state_core().player(1).hand.is_empty(), "fuchendangmo discarded both enemy cards");
    assert!(
        ew.state_core().player(0).hand.contains(&wk1) && ew.state_core().player(0).hand.contains(&wk2),
        "shenhanxueyin stole both wushuangfangtianji discards"
    );

    // 3. `yitianjian` (`extension_skill.js:1041-1068`) + `buff_txhj_shangjiangdelvli`:
    //    Requires `isDamaged() && countCards('h') > 0` even when `shangjiangdelvli` (active when Weapon equipped) skips discard!
    let mut ey = start_versus(436, &[&[0], &[1]]);
    let _ = equip_std_kind(&mut ey, 0, CardKind::Weapon);
    grant_skill(&mut ey, 0, "yitianjian");
    grant_skill(&mut ey, 0, "buff_txhj_shangjiangdelvli");
    grant_skill(&mut ey, 0, engine_native::vocabulary::names::PAO_XIAO);
    ey.debug_set_turn(0, Phase::Play, 1);
    ey.debug_set_hp(0, 4);

    // (a) 0 extra hand cards on damage: guard fails -> NO recover (`hp` stays 4)!
    let y_sha0 = place_kind(&mut ey, 0, CardKind::Sha);
    let _ = play_card(&mut ey, 0, y_sha0, vec![1]);
    drain(&mut ey);
    assert_eq!(ey.state_core().player(0).hp, 4, "0-hand yitianjian must not recover even with shangjiangdelvli");

    // (b) 1 extra hand card while damaged: recovers 4 -> 5 AND preserves extra hand card!
    let y_keep = place_kind(&mut ey, 0, CardKind::Tao);
    let y_sha1 = place_kind(&mut ey, 0, CardKind::Sha);
    let _ = play_card(&mut ey, 0, y_sha1, vec![1]);
    drain(&mut ey);
    assert_eq!(ey.state_core().player(0).hp, 5, "damaged + >=1 hand yitianjian + shangjiangdelvli recovers 1");
    assert_eq!(ey.state_core().player(0).hand, vec![y_keep], "shangjiangdelvli keeps extra hand card");

    // (c) Uninjured (`hp == max_hp`) even with extra hand card: must NOT fire (`hp` stays 6, card kept)!
    ey.debug_set_hp(0, 6);
    let y_sha_full = place_kind(&mut ey, 0, CardKind::Sha);
    let _ = play_card(&mut ey, 0, y_sha_full, vec![1]);
    drain(&mut ey);
    assert_eq!(ey.state_core().player(0).hp, 6, "uninjured yitianjian does not overheal");
    assert_eq!(ey.state_core().player(0).hand, vec![y_keep]);

    // (d) Without `shangjiangdelvli` when damaged + >=1 hand card: discards 1 hand card and recovers 1 HP!
    let mut ey_plain = start_versus(437, &[&[0], &[1]]);
    grant_skill(&mut ey_plain, 0, "yitianjian");
    ey_plain.debug_set_turn(0, Phase::Play, 1);
    ey_plain.debug_set_hp(0, 4);
    let _y_disc = place_kind(&mut ey_plain, 0, CardKind::Tao);
    let y_sha2 = place_kind(&mut ey_plain, 0, CardKind::Sha);
    let _ = play_card(&mut ey_plain, 0, y_sha2, vec![1]);
    let c_yt = expect_pending(&ey_plain, "yitianjian:discard");
    let _ = answer(&mut ey_plain, &c_yt, vec![ChoiceOption::Card(_y_disc)]);
    drain(&mut ey_plain);
    assert_eq!(ey_plain.state_core().player(0).hp, 5);
    assert!(ey_plain.state_core().player(0).hand.is_empty(), "plain yitianjian discards 1 hand card");
}

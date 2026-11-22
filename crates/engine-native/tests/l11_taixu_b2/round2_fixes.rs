use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Findings #1, #4 (`charlotte`), and #9: `txhj_suzhi`, `ljxuechi`, `zglkongcheng`, `txnianrui`,
/// `txhj_aleSkill1..2`, `txhj_yueerSkill1`, and `txhj_xiaoxiaoSkill1` ignore `is_tieji_sealed`.
#[test]
fn l11_b2_r2_1_4_9_locked_and_charlotte_bypass_tieji_seal() {
    // 1. txhj_suzhi (#1) + txnianrui (#9) + ljxuechi (#9) under fz_new_tieji_target_sealed:
    let mut e = start_taixu(0x2301, 4);
    e.debug_grant(0, "txhj_suzhi");
    e.debug_grant(0, "txnianrui");
    e.debug_grant(0, "ljxuechi");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "fz_new_tieji_target_sealed".into()), 1);

    // txnianrui under tieji seal still draws +2 (total 4) in Draw phase:
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e); // -> Draw
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 4, "locked txnianrui draws +2 despite tieji seal");

    // ljxuechi under tieji seal still blocks seat 1's Tao when seat 0 has < 3 cards:
    while e.debug_hand(0).len() > 2 {
        let c = e.debug_hand(0)[0];
        let _ = e.debug_discard_card(0, c);
    }
    e.debug_set_hp(1, 3);
    e.debug_set_turn(1, Phase::Play, 1);
    let tao1 = place_kind(&mut e, 1, CardKind::Tao);
    let _ = play_card(&mut e, 1, tao1, Vec::new());
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3, "locked ljxuechi blocks recover despite tieji seal");

    // txhj_suzhi: under tieji seal, marks < 3 grants unlimited Snatch distance; at 3 marks it stops!
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "fz_new_tieji_target_sealed".into()), 1);
    let _ = place_kind(&mut e, 2, CardKind::Sha);
    let _ = place_kind(&mut e, 2, CardKind::Sha);
    let sn1 = place_kind(&mut e, 0, CardKind::Snatch);
    assert!(
        !matches!(play_card(&mut e, 0, sn1, vec![2]), CommandOutcome::Rejected(_)),
        "txhj_suzhi (<3 marks) grants unlimited trick range despite tieji seal"
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "skill_turn:l11_suzhi_marks"), 1);

    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "skill_turn:l11_suzhi_marks".into()), 3);
    let sn2 = place_kind(&mut e, 0, CardKind::Snatch);
    assert!(
        matches!(play_card(&mut e, 0, sn2, vec![2]), CommandOutcome::Rejected(_)),
        "txhj_suzhi stops unlimited range once l11_suzhi_marks reaches 3"
    );

    // 2. charlotte servant skills (txhj_aleSkill1, txhj_aleSkill2, txhj_yueerSkill1, txhj_xiaoxiaoSkill1) bypass tieji seal:
    let mut ec = start_taixu(0x2302, 2);
    ec.debug_grant(1, "txhj_aleSkill1");
    ec.debug_grant(1, "txhj_aleSkill2");
    ec.debug_grant(1, "txhj_yueerSkill1");
    ec.debug_grant(1, "txhj_xiaoxiaoSkill1");
    ec.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "fz_new_tieji_target_sealed".into()), 1);
    ec.debug_set_hp(1, 3);
    let shan1 = place_kind(&mut ec, 1, CardKind::Shan);
    let _ = shan1;
    ec.debug_set_turn(0, Phase::Play, 1);
    let sha0 = place_kind(&mut ec, 0, CardKind::Sha);
    let _ = play_card(&mut ec, 0, sha0, vec![1]);
    let c_sh_al = expect_pending(&ec, "respond_shan");
    let _ = answer(&mut ec, &c_sh_al, vec![ChoiceOption::Card(shan1)]);
    drain(&mut ec); // dodged Sha -> aleSkill1 draws 1 despite tieji seal!
    assert_eq!(ec.debug_hand(1).len(), 1, "charlotte txhj_aleSkill1 draws despite tieji seal");

    // Delayed or ordinary trick from seat 0 triggers charlotte txhj_xiaoxiaoSkill1 even under tieji seal:
    let le0 = place_kind(&mut ec, 0, CardKind::Le);
    let _ = play_card(&mut ec, 0, le0, vec![1]);
    let c_xx = expect_pending(&ec, "txhj_xiaoxiaoSkill1:confirm");
    let _ = answer(&mut ec, &c_xx, vec![ChoiceOption::Confirm]);
    drain(&mut ec);
    assert_eq!(ec.debug_hp(1), 4, "charlotte txhj_xiaoxiaoSkill1 triggers on Le despite tieji seal");

    // Prepare phase of seat 1 under tieji seal -> charlotte txhj_aleSkill2 recovers +1 (to 5)
    ec.debug_set_hp(0, 6);
    ec.debug_set_turn(1, Phase::Prepare, 1);
    ec.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "fz_new_tieji_target_sealed".into()), 1);
    // Enter Prepare via Turn Begin / next_phase from previous End:
    ec.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut ec); // -> Turn 1 Prepare
    ec.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "fz_new_tieji_target_sealed".into()), 1);
    drain(&mut ec);
    // Note next_phase into Turn 1 already creates Prepare (before test inserted flag post-Prepare); verify Prepare directly:
    let mut ec2 = start_taixu(0x2303, 2);
    ec2.debug_grant(1, "txhj_aleSkill2");
    ec2.debug_set_hp(1, 2);
    ec2.debug_set_turn(0, Phase::End, 1);
    ec2.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "fz_new_tieji_target_sealed".into()), 1);
    let _ = next_phase(&mut ec2);
    drain(&mut ec2);
    assert_eq!(ec2.debug_hp(1), 3, "charlotte txhj_aleSkill2 recovers in Prepare despite tieji seal");

    // male damage -> charlotte txhj_yueerSkill1 still recovers & draws under tieji seal:
    let mut ec3 = start_taixu(0x2304, 2);
    ec3.debug_grant(1, "txhj_yueerSkill1");
    ec3.debug_set_hp(1, 3);
    ec3.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "fz_new_tieji_target_sealed".into()), 1);
    ec3.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    drain(&mut ec3);
    assert_eq!(ec3.debug_hp(1), 4);
    assert_eq!(ec3.debug_hand(1).len(), 1);
}

/// Finding #2: `_slUseCard`, `txhj_ruiSkill3`, `txhj_yanSkill1`, `txnangce` match ordinary tricks ONLY (not delayed),
/// while `txhj_xuanwuSkill3` and `txhj_suzhi` still include delayed tricks.
#[test]
fn l11_b2_r2_2_ordinary_tricks_exclude_delayed_for_sl_rui3_yan1_nangce() {
    let mut e = start_taixu(0x2305, 2);
    e.debug_grant(0, "_slUseCard");
    e.debug_grant(0, "txnangce");
    e.debug_grant(0, "txhj_suzhi");
    e.debug_grant(1, "txhj_ruiSkill3");
    e.debug_grant(1, "txhj_yanSkill1");
    e.debug_set_turn(0, Phase::Play, 1);

    let le = place_kind(&mut e, 0, CardKind::Le);
    let _ = play_card(&mut e, 0, le, vec![1]);
    // Must NOT trigger txnangce:confirm, ruiSkill3, yanSkill1, or _slUseCard; MUST trigger txhj_suzhi draw (+1):
    assert!(
        e.open_choice().is_none(),
        "Le must NOT open txnangce/ruiSkill3/yanSkill1"
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_usecard_anim"), 0, "_slUseCard ignores Le");
    assert_eq!(flag(&e, 0, "skill_turn:l11_suzhi_marks"), 1, "txhj_suzhi triggers on Le");
    assert_eq!(e.debug_hand(0).len(), 1, "txhj_suzhi drew 1 on Le");

    // Counter-check: ordinary trick (Dismantle) triggers _slUseCard, txnangce, and ruiSkill3/yanSkill1:
    let gh = place_kind(&mut e, 0, CardKind::Dismantle);
    let _ = play_card(&mut e, 0, gh, vec![1]);
    let c_nc = expect_pending(&e, "txnangce:confirm");
    let _ = answer(&mut e, &c_nc, vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_usecard_anim"), 1, "_slUseCard fires on ordinary trick");

    // Verify txhj_xuanwuSkill3 STILL accepts delayed tricks:
    let mut ex = start_taixu(0x2306, 2);
    ex.debug_grant(1, "txhj_xuanwuSkill3");
    for _ in 0..13 {
        let _ = place_kind(&mut ex, 1, CardKind::Sha);
    }
    ex.debug_set_turn(0, Phase::Play, 1);
    let le2 = place_kind(&mut ex, 0, CardKind::Le);
    let _ = play_card(&mut ex, 0, le2, vec![1]);
    drain(&mut ex);
    assert!(ex.state_core().player(1).judge.is_empty(), "txhj_xuanwuSkill3 still cancels Le");
}

/// Findings #3, #4 (multi-target), #5, #6, #7, #8a, #8b, #8c: servant accuracy + equip-loss/discard events.
#[test]
fn l11_b2_r2_3_4_5_6_7_8_servants_and_equip_discard_events() {
    // 1. #3 txhj_luluSkill2: 1st Sha with gained < hp does NOT empower 2nd Sha after drawing >= hp:
    let mut el = start_taixu(0x2307, 2);
    el.debug_grant(0, "txhj_luluSkill2");
    el.debug_set_hp(0, 2);
    el.debug_set_turn(0, Phase::Play, 1);
    let sha1 = place_kind(&mut el, 0, CardKind::Sha);
    let _ = play_card(&mut el, 0, sha1, vec![1]);
    drain(&mut el);
    assert_eq!(el.debug_hp(1), 5, "1st Sha without gained>=hp deals 1 dmg");
    let wz = el.debug_move_named_card_to_hand(0, "wuzhong");
    let _ = play_card(&mut el, 0, wz, Vec::new()); // draws 2 -> gained == hp!
    drain(&mut el);
    el.state_core_mut().turn.sha_used_play_phase = false;
    let sha2 = place_kind(&mut el, 0, CardKind::Sha);
    let shan1 = place_kind(&mut el, 1, CardKind::Shan);
    let _ = play_card(&mut el, 0, sha2, vec![1]);
    let c_sh = expect_pending(&el, "respond_shan"); // 2nd Sha is NOT directHit!
    let _ = answer(&mut el, &c_sh, vec![ChoiceOption::Card(shan1)]);
    drain(&mut el);
    assert_eq!(el.debug_hp(1), 5, "2nd Sha does not get luluSkill2 boost/directHit");

    // 2. #4 txhj_aleSkill1 multi-target Sha: target 1 dodges while target 2 takes dmg -> target 1 still draws 1:
    let mut ea = start_taixu(0x2308, 3);
    ea.debug_grant(1, "txhj_aleSkill1");
    ea.debug_grant(0, "txhj_fangtianhuaji");
    ea.debug_set_turn(0, Phase::Play, 1);
    let shan_1 = place_kind(&mut ea, 1, CardKind::Shan);
    let sha_a = place_kind(&mut ea, 0, CardKind::Sha); // last card for fangtian
    let _ = play_card(&mut ea, 0, sha_a, vec![1, 2]);
    let c_sh1 = expect_pending(&ea, "respond_shan");
    assert_eq!(c_sh1.observer, 1);
    let _ = answer(&mut ea, &c_sh1, vec![ChoiceOption::Card(shan_1)]);
    drain(&mut ea); // target 2 takes dmg, target 1 did NOT take dmg -> aleSkill1 draws 1!
    assert_eq!(ea.debug_hp(2), 5);
    assert_eq!(ea.debug_hand(1).len(), 1, "aleSkill1 draws 1 even when co-target took damage");

    // 3. #5 txhj_ruiSkill2: odd hand with 0 other friends falls back to discarding 1 enemy card (never draws for self!):
    let mut er = start_taixu(0x2309, 2);
    er.debug_grant(0, "txhj_ruiSkill2");
    let c0 = place_kind(&mut er, 0, CardKind::Sha);
    let _ = place_kind(&mut er, 1, CardKind::Sha);
    er.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut er); // TurnBegin(0), odd hand 1, 0 other friends -> enemy 1 loses card, seat 0 hand stays [c0]
    drain(&mut er);
    assert_eq!(er.debug_hand(0), vec![c0], "ruiSkill2 must NOT count self as friend");
    assert!(er.debug_hand(1).is_empty(), "ruiSkill2 odd hand with 0 other friends discards enemy card");

    // 4. #6 txhj_manmanSkill1_damage: Play-phase hand Nanman draws 1 per hit; TurnBegin non-Nanman does NOT draw:
    let mut em = start_taixu(0x230A, 2);
    em.debug_grant(0, "txhj_manmanSkill1");
    em.debug_set_turn(0, Phase::Play, 1);
    let nm = place_kind(&mut em, 0, CardKind::SavageAttack);
    let _ = play_card(&mut em, 0, nm, Vec::new());
    drain(&mut em);
    assert_eq!(em.debug_hand(0).len(), 1, "manmanSkill1_damage triggers on Play-phase Nanman");
    let sha_m = place_kind(&mut em, 0, CardKind::Sha);
    let _ = play_card(&mut em, 0, sha_m, vec![1]);
    drain(&mut em);
    assert_eq!(em.debug_hand(0).len(), 1, "manmanSkill1_damage does NOT trigger on Sha");

    // 5. #7 + #8c: equip-only discard (hgkurou) & txmanzhen must NOT trigger txhj_xuanwuSkill2, while 2 hand + 1 equip counts hs only (=1 draw):
    let mut ex2 = start_taixu(0x230B, 2);
    ex2.debug_grant(0, "txhj_xuanwuSkill2");
    ex2.debug_grant(0, "hgkurou");
    let _ = equip_kind(&mut ex2, 0, CardKind::Zhuge);
    ex2.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut ex2, 0, "hgkurou");
    let c_hk = expect_pending(&ex2, "hgkurou:choose_card");
    let eq_hk = ex2.debug_equips(0)[0];
    let _ = answer(&mut ex2, &c_hk, vec![ChoiceOption::Card(eq_hk)]);
    drain(&mut ex2);
    assert_eq!(ex2.debug_hand(0).len(), 3, "hgkurou draws 3 only; xuanwuSkill2 must NOT trigger on equip-only discard");
    assert_eq!(ex2.debug_hp(1), 6, "xuanwuSkill2 must NOT deal damage on equip-only discard");
    assert_eq!(flag(&ex2, 0, "skill_turn:l11_xuanwu2_used"), 0);

    // txmanzhen viewAs must NOT trigger xuanwuSkill2 either:
    let mut emz = start_taixu(0x230C, 2);
    emz.debug_grant(0, "txhj_xuanwuSkill2");
    emz.debug_grant(0, "txmanzhen");
    emz.debug_set_hp(0, 2);
    let sha_mz = place_kind(&mut emz, 0, CardKind::Sha);
    emz.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut emz, 0, "txmanzhen");
    let c_mc = expect_pending(&emz, "txmanzhen:choose_cards");
    let _ = answer(&mut emz, &c_mc, vec![ChoiceOption::Card(sha_mz)]);
    let c_mt = expect_pending(&emz, "txmanzhen:choose_targets");
    let _ = answer(&mut emz, &c_mt, vec![ChoiceOption::Player(1)]);
    drain(&mut emz);
    assert!(emz.debug_hand(0).is_empty(), "txmanzhen viewAs must NOT emit CardsDiscarded / trigger xuanwuSkill2");

    // Mixed discard (2 hand + 1 equip via gdsanchen) counts hs ONLY: ceil(2/2) = 1 draw & 1 enemy dmg (NOT ceil(3/2) = 2)!
    let mut emix = start_versus(0x231F, &[&[0], &[1, 2, 3]], &[]);
    emix.debug_grant(0, "txhj_xuanwuSkill2");
    emix.debug_grant(1, "gdsanchen");
    let mh1 = place_kind(&mut emix, 0, CardKind::Sha);
    let mh2 = place_kind(&mut emix, 0, CardKind::Shan);
    let meq = equip_kind(&mut emix, 0, CardKind::Zhuge);
    emix.debug_set_turn(1, Phase::Play, 1);
    let _ = activate_skill(&mut emix, 1, "gdsanchen");
    let c_gst = expect_pending(&emix, "gdsanchen:choose_target");
    let _ = answer(&mut emix, &c_gst, vec![ChoiceOption::Player(0)]);
    let c_gsd = expect_pending(&emix, "gdsanchen:discard_cards");
    let _ = answer(
        &mut emix,
        &c_gsd,
        vec![
            ChoiceOption::Card(mh1),
            ChoiceOption::Card(mh2),
            ChoiceOption::Card(meq),
        ],
    );
    drain(&mut emix);
    assert_eq!(
        emix.debug_hand(0).len(),
        4,
        "mixed 2 hand + 1 equip discard after gdsanchen +3 draws ceil(2/2)=1 (total 4, not 5)"
    );
    let hp_before = emix.debug_hp(2);
    assert_eq!(
        emix.debug_hp(1),
        hp_before - 1,
        "xuanwuSkill2 deals 1 dmg to 1st enemy"
    );
    assert_eq!(
        emix.debug_hp(3),
        hp_before,
        "xuanwuSkill2 counts hs ONLY (ceil(2/2)=1, must NOT damage 2nd/3rd enemy)"
    );

    // 6. #8a equip replacement triggers txhj_aHaoSkill2 + #8b Guohe equip triggers txyinli & txhj_suzhi_gain (NOT xuanwuSkill2!):
    let mut eah = start_taixu(0x230D, 3);
    eah.debug_grant(0, "txhj_aHaoSkill2");
    let _ = equip_kind(&mut eah, 0, CardKind::Zhuge);
    eah.debug_set_turn(0, Phase::Play, 1);
    let qg = place_kind(&mut eah, 0, CardKind::Qinggang);
    let _ = play_card(&mut eah, 0, qg, Vec::new());
    drain(&mut eah);
    assert_eq!(eah.debug_hp(1), 5, "replacing equip triggers aHaoSkill2 -> enemy 1 dmg");

    let mut egh = start_taixu(0x230E, 2);
    egh.debug_grant(0, "txhj_suzhi");
    egh.debug_grant(1, "txhj_xuanwuSkill2");
    egh.debug_grant(0, "txyinli");
    let _ = equip_kind(&mut egh, 1, CardKind::Renwang);
    let _ = equip_kind(&mut egh, 1, CardKind::Zhuge);
    egh.debug_set_turn(0, Phase::Play, 1);
    let gh = place_kind(&mut egh, 0, CardKind::Dismantle);
    let _ = play_card(&mut egh, 0, gh, vec![1]);
    let c_sz = expect_pending(&egh, "txhj_suzhi:choose_he");
    let _ = answer(&mut egh, &c_sz, vec![c_sz.spec.options[0]]);
    drain(&mut egh);
    assert_eq!(
        flag(&egh, 0, "skill_turn:l11_suzhi_marks"),
        2,
        "Guohe equip emits CardsDiscarded -> triggers suzhi_gain (1 trick + 1 discard = 2 marks)"
    );
    assert!(
        egh.debug_hand(1).is_empty(),
        "Guohe equip discard must NOT trigger victim xuanwuSkill2"
    );
    egh.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut egh); // TurnEnded(0) triggers txyinli for seat 0? Wait seat 0 == owner skips! Test in enemy turn:
    let mut egh2 = start_taixu(0x230F, 3);
    egh2.debug_grant(0, "txyinli");
    let _ = equip_kind(&mut egh2, 2, CardKind::Renwang);
    let eq_2 = egh2.debug_equips(2)[0];
    egh2.debug_set_turn(1, Phase::Play, 1);
    let gh2 = place_kind(&mut egh2, 1, CardKind::Dismantle);
    let _ = play_card(&mut egh2, 1, gh2, vec![2]);
    drain(&mut egh2);
    egh2.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut egh2);
    let c_yl = expect_pending(&egh2, "txyinli:choose_cards");
    let _ = answer(&mut egh2, &c_yl, vec![ChoiceOption::Card(eq_2)]);
    assert!(egh2.debug_hand(0).contains(&eq_2), "Guohe equip discard recovers via txyinli at TurnEnded");
}

/// Findings #10..#15: `txmanjia1/2` + `Qinggang`, `zylongdan`, `txjuxiang`, `txchengyuan` cancel/reset,
/// `txdaoji` order, `txhj_mouqiang` order, `txbenji` self+cancel, and `txyicheng` per-target confirm.
#[test]
fn l11_b2_r2_10_11_12_13_14_15_general_fixes_and_ordering() {
    // 1. #10 txmanjia1 vs txmanjia2 split & Qinggang bypass:
    let mut emj1 = start_taixu(0x2310, 2);
    emj1.debug_grant(1, "txmanjia1");
    emj1.debug_set_turn(0, Phase::Play, 1);
    let fsha = place_kind(&mut emj1, 0, CardKind::FireSha);
    let _ = play_card(&mut emj1, 0, fsha, vec![1]);
    drain(&mut emj1);
    assert_eq!(emj1.debug_hp(1), 5, "txmanjia1 alone must NOT take +1 Fire damage");

    let mut emj2 = start_taixu(0x2311, 2);
    emj2.debug_grant(1, "txmanjia2");
    emj2.debug_set_turn(0, Phase::Play, 1);
    let nsha = place_kind(&mut emj2, 0, CardKind::Sha);
    let _ = play_card(&mut emj2, 0, nsha, vec![1]);
    drain(&mut emj2);
    assert_eq!(emj2.debug_hp(1), 5, "txmanjia2 alone must NOT zero normal Sha");
    emj2.state_core_mut().turn.sha_used_play_phase = false;
    let fsha2 = place_kind(&mut emj2, 0, CardKind::FireSha);
    let _ = play_card(&mut emj2, 0, fsha2, vec![1]);
    drain(&mut emj2);
    assert_eq!(emj2.debug_hp(1), 3, "txmanjia2 adds +1 Fire dmg");

    let mut eqg = start_taixu(0x2312, 2);
    eqg.debug_grant(1, "txmanjia");
    let _ = equip_kind(&mut eqg, 0, CardKind::Qinggang);
    eqg.debug_set_turn(0, Phase::Play, 1);
    let sha_qg = place_kind(&mut eqg, 0, CardKind::Sha);
    let _ = play_card(&mut eqg, 0, sha_qg, vec![1]);
    drain(&mut eqg);
    assert_eq!(eqg.debug_hp(1), 5, "Qinggang ignores unarmored txmanjia damage prevention");

    // 2. #11 zylongdan: Wanjian with Sha-as-Shan does NOT trigger recover_target; Shan-as-Sha respond vs Duel draws with shouyue:
    let mut eld = start_taixu(0x2313, 3);
    eld.debug_grant(1, "zylongdan");
    eld.debug_set_hp(2, 3); // wounded seat 2 exists!
    eld.debug_set_turn(0, Phase::Play, 1);
    let wj = place_kind(&mut eld, 0, CardKind::ArcheryAttack);
    let sha_1 = place_kind(&mut eld, 1, CardKind::Sha);
    let _ = play_card(&mut eld, 0, wj, Vec::new());
    let c_aoe = expect_pending(&eld, "aoe_response");
    let _ = answer(&mut eld, &c_aoe, vec![ChoiceOption::Card(sha_1)]);
    assert!(eld.open_choice().is_none(), "Wanjian Sha-as-Shan must NOT trigger zylongdan:recover_target");

    // Now test Shan-as-Sha respond vs Duel when shouyue active -> draws 1:
    eld.state_core_mut().vm_aux.flags.insert((2, "shouyue_active".into()), 1);
    let shan_1 = place_kind(&mut eld, 1, CardKind::Shan);
    let dl = place_kind(&mut eld, 0, CardKind::Duel);
    let _ = play_card(&mut eld, 0, dl, vec![1]);
    let c_dl = expect_pending(&eld, "duel_sha");
    let _ = answer(&mut eld, &c_dl, vec![ChoiceOption::Card(shan_1)]);
    drain(&mut eld);
    assert_eq!(eld.debug_hand(1).len(), 1, "shouyue zylongdan draws 1 on Shan-as-Sha respond");

    // 3. #12a txjuxiang prefers Spade 7 / Club 9 over earlier fallback Nanman, & #12b cancelled red trick sets chengyuan color + Prepare resets:
    let mut ejp = start_taixu(0x2314, 2);
    ejp.debug_grant(0, "txjuxiang");
    ejp.debug_grant(0, "txchengyuan");
    ejp.debug_grant(1, "txhj_xuanwuSkill3"); // cancels Dismantle (`rank <= hand.len()`) during useCard!
    for _ in 0..13 {
        let _ = place_kind(&mut ejp, 1, CardKind::Sha);
    }
    // Put a fallback Nanman on top/front, and Spade 7 deeper in deck:
    let s7_nm = ejp.catalog().cards.iter().enumerate().find(|(_, c)| c.kind == CardKind::SavageAttack && c.suit == Suit::Spade && c.rank == 7).unwrap().0 as u32;
    let other_nm = ejp.catalog().cards.iter().enumerate().find(|(_, c)| c.kind == CardKind::SavageAttack && !(c.suit == Suit::Spade && c.rank == 7) && !(c.suit == Suit::Club && c.rank == 9)).unwrap().0 as u32;
    ejp.state_core_mut().deck.retain(|&c| c != s7_nm && c != other_nm);
    ejp.state_core_mut().discard.retain(|&c| c != s7_nm && c != other_nm);
    ejp.state_core_mut().deck.insert(0, other_nm);
    ejp.state_core_mut().deck.push(s7_nm);

    // Cancelled red Dismantle still records Red color for txchengyuan (#12b)!
    ejp.debug_set_turn(0, Phase::Play, 1);
    let red_gh = place_kind_suit(&mut ejp, 0, CardKind::Dismantle, Suit::Heart);
    let _ = play_card(&mut ejp, 0, red_gh, vec![1]);
    drain(&mut ejp);
    assert_eq!(flag(&ejp, 0, "skill_turn:l11_chengyuan_used") & 1, 1, "cancelled red card records txchengyuan color");

    ejp.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut ejp); // End -> txjuxiang picks Spade 7 (not other_nm)!
    let c_jx2 = expect_pending(&ejp, "txjuxiang:choose_target");
    let _ = answer(&mut ejp, &c_jx2, vec![ChoiceOption::Player(0)]);
    drain(&mut ejp);
    assert!(ejp.debug_hand(0).contains(&s7_nm), "txjuxiang prefers Spade 7 over fallback SavageAttack");

    // Turn 2 Prepare(0) resets l11_chengyuan_used to 0 (#15):
    ejp.state_core_mut().vm_aux.flags.insert((0, "skill_turn:l11_chengyuan_used".into()), 3);
    ejp.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut ejp); // -> Turn 2 Prepare(0)
    drain(&mut ejp);
    assert_eq!(flag(&ejp, 0, "skill_turn:l11_chengyuan_used"), 0, "Prepare resets l11_chengyuan_used");

    // 4. #13 txdaoji: equip resolves BEFORE damage (`loot` is already in equips when Damage triggers `反馈`)!
    let mut edj = start_taixu(0x2315, 2);
    edj.debug_grant(0, "txdaoji");
    edj.debug_grant(1, "反馈");
    let pay_tr = place_kind(&mut edj, 0, CardKind::Wuzhong);
    let dummy = place_kind(&mut edj, 0, CardKind::Sha);
    let loot = equip_kind(&mut edj, 1, CardKind::Renwang);
    edj.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut edj, 0, "txdaoji");
    let c_d1 = expect_pending(&edj, "txdaoji:discard_card");
    let _ = answer(&mut edj, &c_d1, vec![ChoiceOption::Card(pay_tr)]);
    let c_d2 = expect_pending(&edj, "txdaoji:choose_target");
    let _ = answer(&mut edj, &c_d2, vec![ChoiceOption::Player(1)]);
    let c_d3 = expect_pending(&edj, "txdaoji:choose_he");
    let _ = answer(&mut edj, &c_d3, vec![ChoiceOption::Card(loot)]);
    // Equips first -> Damage -> 反馈 sees `loot` ALREADY equipped in equip slot (`hand == [dummy]`)!
    let c_fk = expect_pending(&edj, "trigger_confirm");
    assert!(
        edj.debug_equips(0).contains(&loot),
        "txdaoji must complete equip BEFORE Damage triggers fire"
    );
    assert_eq!(edj.debug_hand(0), vec![dummy]);
    let _ = answer(&mut edj, &c_fk, vec![ChoiceOption::Confirm]);
    drain(&mut edj);
    assert_eq!(edj.debug_hp(1), 5, "txdaoji equip+fankui resolves without panic");
    assert!(edj.debug_hand(1).contains(&dummy));

    // 5. #14 txhj_mouqiang: target damage + Dying fires BEFORE owner recovers +1 HP:
    let mut emq = start_taixu(0x2316, 3);
    emq.debug_grant(0, "txhj_mouqiang");
    emq.debug_set_hp(1, 1);
    let b1 = place_kind(&mut emq, 1, CardKind::Sha);
    let tao1 = place_kind(&mut emq, 1, CardKind::Tao);
    let _ = tao1;
    emq.debug_set_hp(0, 4);
    emq.debug_deal_damage(Some(2), 0, 2, DamageNature::Normal); // 4 -> 2 HP
    let c_mqt = expect_pending(&emq, "txhj_mouqiang:choose_target");
    let _ = answer(&mut emq, &c_mqt, vec![ChoiceOption::Player(1)]);
    let c_mqc = expect_pending(&emq, "txhj_mouqiang:choose_he");
    let _ = answer(&mut emq, &c_mqc, vec![ChoiceOption::Card(b1)]);
    // Seat 1 took 1 dmg (1 -> 0 HP) and enters Dying AskTao BEFORE seat 0 recovers!
    let c_save = expect_pending(&emq, "dying_save");
    assert_eq!(emq.debug_hp(0), 2, "txhj_mouqiang owner must NOT recover until target damage settles");
    let _ = answer(&mut emq, &c_save, vec![ChoiceOption::Card(tao1)]);
    drain(&mut emq);
    assert_eq!(emq.debug_hp(0), 3, "txhj_mouqiang owner recovers +1 after target damage settles");

    // 6. #15 txchengyuan cancel + txbenji self/cancel + txyicheng per-target confirm (with save/restore):
    let mut ecy = start_taixu(0x2317, 2);
    ecy.debug_grant(0, "txchengyuan");
    let r1 = place_kind_suit(&mut ecy, 0, CardKind::Sha, Suit::Heart);
    let b1 = place_kind_suit(&mut ecy, 0, CardKind::Sha, Suit::Spade);
    let r2 = place_kind_suit(&mut ecy, 0, CardKind::Sha, Suit::Diamond);
    ecy.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "l11_txchengyuan".into()), vec![r1, b1]);
    ecy.debug_set_turn(0, Phase::Play, 1);
    let _ = next_phase(&mut ecy); // -> Discard prompts txchengyuan
    let c_cy_c = expect_pending(&ecy, "txchengyuan:choose_card");
    let _ = answer(&mut ecy, &c_cy_c, vec![ChoiceOption::Card(r2)]);
    let c_cy_t = expect_pending(&ecy, "txchengyuan:choose_target");
    assert!(c_cy_t.spec.allow_cancel, "txchengyuan:choose_target must allow cancel");
    let _ = answer(&mut ecy, &c_cy_t, Vec::new()); // cancel target choice
    assert_eq!(ecy.debug_hp(1), 6);

    let mut ebj = start_taixu(0x2318, 2);
    ebj.debug_grant(0, "txbenji");
    ebj.debug_set_hp(0, 3);
    ebj.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut ebj); // -> Prepare(0) triggers txbenji
    let c_bj = expect_pending(&ebj, "txbenji:choose_target");
    assert!(c_bj.spec.allow_cancel, "txbenji:choose_target must be cancelable");
    assert!(c_bj.spec.options.contains(&ChoiceOption::Player(0)), "txbenji must allow targeting self");
    assert_eq!(
        ai_plan(&ebj, 0),
        GameAction::Answer {
            choice_id: c_bj.id,
            selection: vec![ChoiceOption::Player(1)],
        }
    );
    let _ = answer(&mut ebj, &c_bj, Vec::new()); // cancel -> no dmg, no loseHp!
    assert_eq!(ebj.debug_hp(0), 3);

    // txyicheng per-target confirm: multi-target Sha prompts confirm independently per target (with save/restore)!
    let mut eyc = start_taixu(0x2319, 3);
    eyc.debug_grant(1, "txyicheng");
    eyc.debug_grant(0, "txhj_fangtianhuaji");
    eyc.debug_set_turn(0, Phase::Play, 1);
    let sha_yc = place_kind(&mut eyc, 0, CardKind::Sha);
    let _ = play_card(&mut eyc, 0, sha_yc, vec![1, 2]);
    let c_yc1 = expect_pending(&eyc, "txyicheng:confirm");
    let snap = eyc.save();
    let mut eyc_restored =
        content_native_loader::restore_locked_engine(&snap, taixu_verified()).expect("restore txyicheng pending");
    assert_eq!(eyc.state_hash(), eyc_restored.state_hash());
    let _ = answer(&mut eyc, &c_yc1, vec![ChoiceOption::Confirm]);
    let _ = answer(&mut eyc_restored, &c_yc1, vec![ChoiceOption::Confirm]);
    assert_eq!(eyc.state_hash(), eyc_restored.state_hash());
    // Target 1 confirmed -> draws 1 & discards 1:
    let c_yc_dis = expect_pending(&eyc_restored, "txyicheng:discard_card");
    let d_card = eyc_restored.debug_hand(1)[0];
    let _ = answer(&mut eyc_restored, &c_yc_dis, vec![ChoiceOption::Card(d_card)]);
    // Target 2 prompts a separate txyicheng:confirm -> decline!
    let c_yc2 = expect_pending(&eyc_restored, "txyicheng:confirm");
    let _ = answer(&mut eyc_restored, &c_yc2, vec![ChoiceOption::Pass]);
    drain(&mut eyc_restored);
    assert!(
        eyc_restored.debug_hand(2).is_empty(),
        "declining second txyicheng:confirm leaves target 2 untouched"
    );
}

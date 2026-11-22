use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension_servant.js:24-210` (`txhj_daiyuan`, `txhj_daiyuan2`, `txhj_daiyuanA`, `txhj_daiyuanB`).
#[test]
fn l11_servants_daiyuan_family_modes_ai_and_restore() {
    // Seat 0 has `txhj_daiyuan`, seat 2 also has `txhj_daiyuan` (both injured) -> seat 1 gets `txhj_daiyuan2`
    let mut e = start_versus(0x1301, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txhj_daiyuan");
    e.debug_grant(2, "txhj_daiyuan");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e); // Draw -> triggers Turn/Phase -> grants txhj_daiyuan2 to seat 1
    drain(&mut e);
    assert!(e.debug_has_skill(1, "txhj_daiyuan2"));

    // Seat 1's Play phase: give Red Trick (`wuzhong` Heart), Black Basic (`sha` Spade), Red Basic (`tao` Heart)
    e.debug_set_turn(1, Phase::Play, 1);
    let red_basic = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Heart);
    let blk_basic = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    let red_trick = place_kind_suit(&mut e, 1, CardKind::Wuzhong, Suit::Heart);

    let out = activate_skill(&mut e, 1, "txhj_daiyuan2");
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let c_mode = expect_pending(&e, "txhj_daiyuan:choose_mode");
    assert_eq!(c_mode.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_eq!(
        ai_plan(&e, 1),
        GameAction::Answer {
            choice_id: c_mode.id,
            selection: vec![ChoiceOption::Tag(0)],
        }
    );
    // Mode 0 (different colors): pick same-color first (re-prompts!), then different color -> grants A & heals target
    let _ = answer(&mut e, &c_mode, vec![ChoiceOption::Tag(0)]);
    let c_cards = expect_pending(&e, "txhj_daiyuan:choose_cards");
    let _ = answer(
        &mut e,
        &c_cards,
        vec![ChoiceOption::Card(red_basic), ChoiceOption::Card(red_trick)],
    );
    let c_re = expect_pending(&e, "txhj_daiyuan:choose_cards");
    let _ = answer(
        &mut e,
        &c_re,
        vec![ChoiceOption::Card(red_basic), ChoiceOption::Card(blk_basic)],
    );
    let c_tgt = expect_pending(&e, "txhj_daiyuan:choose_target");
    assert_eq!(
        ai_plan(&e, 1),
        GameAction::Answer {
            choice_id: c_tgt.id,
            selection: vec![ChoiceOption::Player(0)], // ally over enemy!
        }
    );
    let mut e2 = save_restore_and_compare_pending(&e, &c_tgt, vec![ChoiceOption::Player(0)]);
    assert!(e2.debug_has_skill(1, "txhj_daiyuanA"));
    assert_eq!(e2.debug_hp(0), 5);
    assert_eq!(e2.debug_hp(2), 4);

    // Mode 1 (different type2): only mode B left -> skips choose_mode directly to choose_cards!
    let blk_basic2 = place_kind_suit(&mut e2, 1, CardKind::Sha, Suit::Club);
    let out_b = activate_skill(&mut e2, 1, "txhj_daiyuan2");
    assert!(matches!(out_b, CommandOutcome::Pending { .. }));
    let c_cb = expect_pending(&e2, "txhj_daiyuan:choose_cards");
    let _ = answer(
        &mut e2,
        &c_cb,
        vec![ChoiceOption::Card(blk_basic2), ChoiceOption::Card(red_trick)],
    );
    let c_tb = expect_pending(&e2, "txhj_daiyuan:choose_target");
    let _ = answer(&mut e2, &c_tb, vec![ChoiceOption::Player(0)]);
    drain(&mut e2);
    assert!(e2.debug_has_skill(1, "txhj_daiyuanB"));
    assert_eq!(e2.debug_hp(0), 6);

    // Both A & B learned -> cannot activate `txhj_daiyuan2` again
    assert!(matches!(
        activate_skill(&mut e2, 1, "txhj_daiyuan2"),
        CommandOutcome::Rejected(_)
    ));
}

/// Upstream: `extension_buff.js` (`txhj_ruiSkill1`, `txhj_dundunSkill2`, `txhj_jiuweiSkill1..3`).
#[test]
fn l11_servants_rui_dundun_and_jiuwei() {
    // 1. txhj_ruiSkill1: Play phase end deals 1 Fire to enemies with hand.len() <= owner.hand.len()
    let mut e = start_versus(0x1302, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txhj_ruiSkill1");
    e.debug_set_hp(0, 1);
    e.debug_set_hp(1, 5);
    e.debug_set_hp(2, 1);
    for _ in 0..3 {
        let _ = place_kind(&mut e, 0, CardKind::Sha);
    }
    let _ = place_kind(&mut e, 1, CardKind::Sha); // hand 1 < 3 -> triggers & takes 1 Fire
    for _ in 0..5 {
        let _ = place_kind(&mut e, 2, CardKind::Shan); // hand 5 > 3 -> spared despite low hp
    }
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 4);
    assert_eq!(e.debug_hp(2), 1);

    // 2. txhj_dundunSkill2: damage > 1 reduced by 1; damage == 1 untouched
    let mut e_dd = start_taixu(0x1303, 2);
    e_dd.debug_grant(1, "txhj_dundunSkill2");
    e_dd.debug_grant(0, "咆哮");
    let jiu = place_kind_suit(&mut e_dd, 0, CardKind::Jiu, Suit::Spade);
    let _ = play_card(&mut e_dd, 0, jiu, Vec::new());
    let s2 = place_kind_suit(&mut e_dd, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_dd, 0, s2, vec![1]);
    drain(&mut e_dd);
    assert_eq!(e_dd.debug_hp(1), 5, "dundunSkill2 reduces 2 dmg to 1");
    let s1 = place_kind_suit(&mut e_dd, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_dd, 0, s1, vec![1]);
    drain(&mut e_dd);
    assert_eq!(e_dd.debug_hp(1), 4, "1 dmg stays 1");

    // 3. txhj_jiuweiSkill1 / _disable / 2 / 3
    let mut e_jw = start_taixu(0x1304, 2);
    e_jw.debug_grant(1, "txhj_jiuweiSkill1");
    e_jw.debug_grant(1, "txhj_jiuweiSkill2");
    e_jw.debug_grant(1, "txhj_jiuweiSkill3");
    e_jw.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_jw); // Seat 1 End -> Seat 0 TurnBegin attaches jiuweiSkill1_disable to seat 0
    drain(&mut e_jw);
    assert!(e_jw.debug_has_skill(0, "txhj_jiuweiSkill1_disable"));
    e_jw.debug_set_turn(0, Phase::Play, 2);
    // Single-target trick from seat 0 onto seat 1 is cancelled!
    let ghe = place_kind_suit(&mut e_jw, 0, CardKind::Dismantle, Suit::Spade);
    let _ = place_kind_suit(&mut e_jw, 1, CardKind::Tao, Suit::Heart);
    let _ = play_card(&mut e_jw, 0, ghe, vec![1]);
    drain(&mut e_jw);
    assert_eq!(e_jw.debug_hand(1).len(), 1, "jiuweiSkill1_disable cancelled Dismantle");

    // Seat 1's turn: seat 1 has jiuweiSkill2; when seat 0 uses first non-trick card (dying Tao) during seat 1's turn, jiuweiSkill2 cancels it!
    e_jw.debug_clear_hand(1);
    e_jw.debug_set_turn(1, Phase::Play, 2);
    e_jw.debug_set_hp(0, 1);
    let sha_jw = place_kind_suit(&mut e_jw, 1, CardKind::Sha, Suit::Club);
    let tao1 = place_kind(&mut e_jw, 0, CardKind::Tao);
    let tao2 = place_kind(&mut e_jw, 0, CardKind::Tao);
    let _ = play_card(&mut e_jw, 1, sha_jw, vec![0]);
    let c_tao1 = answer_pass_until(&mut e_jw, "dying_save");
    let _ = answer(&mut e_jw, &c_tao1, vec![ChoiceOption::Card(tao1)]);
    // 1st Tao was cancelled by jiuweiSkill2, so seat 0 is still dying at 0 HP and prompted again for tao2:
    let c_tao2 = answer_pass_until(&mut e_jw, "dying_save");
    assert_eq!(e_jw.debug_hp(0), 0, "jiuweiSkill2 cancelled first non-owner Basic useCard during owner's turn");
    let _ = answer(&mut e_jw, &c_tao2, vec![ChoiceOption::Card(tao2)]);
    drain(&mut e_jw);
    assert_eq!(e_jw.debug_hp(0), 1, "2nd Tao succeeded after jiuweiSkill2 once-per-turn cap");

    // Seat 0 deals 1 damage to seat 1 -> jiuweiSkill3 draws 9 cards on first hit (`9 - 0`)
    e_jw.debug_set_turn(0, Phase::Play, 3);
    e_jw.state_core_mut().turn.sha_used_play_phase = false;
    let hand_before = e_jw.debug_hand(1).len();
    let sha_hit = place_kind_suit(&mut e_jw, 0, CardKind::Sha, Suit::Spade);
    let out_hit = play_card(&mut e_jw, 0, sha_hit, vec![1]);
    assert!(
        matches!(out_hit, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "sha_hit rejected: {out_hit:?}"
    );
    drain(&mut e_jw);
    assert_eq!(e_jw.debug_hp(1), 5);
    assert_eq!(e_jw.debug_hand(1).len(), hand_before + 9, "jiuweiSkill3 draws 9 on first damage");
}

/// Upstream: `extension_buff.js` (`txhj_tengsheSkill1..3`, `txhj_yayaSkill1..2_protected`, `txhj_youyouSkill1..2`).
#[test]
fn l11_servants_tengshe_yaya_and_youyou() {
    // 1. txhj_tengsheSkill1 + 2 + 3
    let mut e = start_versus(0x1305, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(1, "txhj_tengsheSkill1");
    e.debug_grant(1, "txhj_tengsheSkill3");
    e.debug_set_hp(1, 5);
    let s0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e, 0, s0, vec![1]); // normal dmg off-turn -> tengshe1 uses LeiSha on 0 -> tengshe3 heals 1 & draws 2!
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 5, "tengsheSkill1 counter-hit with LeiSha");
    assert_eq!(e.debug_hp(1), 5, "tengsheSkill1 damaged -> -1 to 4 then tengsheSkill3 +1 = 5");
    assert_eq!(e.debug_hand(1).len(), 2, "tengsheSkill3 drew 2 on Thunder damage");

    // txhj_tengsheSkill2: owner TurnEndWindow after -> 2 Thunder if enemies.len() == 1, 1 Thunder if 2+ enemies alive
    let mut e_ts2 = start_versus(0x1306, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e_ts2.debug_grant(0, "txhj_tengsheSkill2");
    e_ts2.debug_set_hp(1, 4); // 1 injured enemy out of 2 alive enemies -> 1 Thunder each
    e_ts2.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_ts2); // End -> TurnEnd after
    drain(&mut e_ts2);
    assert_eq!(e_ts2.debug_hp(1), 3, "2 alive enemies -> 1 Thunder even when only 1 was damaged");
    assert_eq!(e_ts2.debug_hp(2), 5, "all enemies take 1 Thunder");

    // 2. txhj_yayaSkill1 + 2 + protected
    let mut e_yy = start_versus(0x1307, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_yy.debug_grant(1, "txhj_yayaSkill1");
    e_yy.debug_grant(1, "txhj_yayaSkill2");
    e_yy.debug_grant(0, "咆哮");
    e_yy.debug_set_hp(1, 1);
    for _ in 0..4 {
        let rs = place_kind_suit(&mut e_yy, 0, CardKind::Sha, Suit::Heart);
        let sh = place_kind_suit(&mut e_yy, 1, CardKind::Shan, Suit::Diamond);
        let _ = play_card(&mut e_yy, 0, rs, vec![1]);
        let cs = expect_pending(&e_yy, "respond_shan");
        let _ = answer(&mut e_yy, &cs, vec![ChoiceOption::Card(sh)]);
        drain(&mut e_yy);
    }
    // 4 red card triggers -> yaya1 drew 4 total and dealt 1 damage to enemy seat 0!
    assert_eq!(e_yy.debug_hand(1).len(), 4);
    assert_eq!(e_yy.debug_hp(0), 5);
    // Hit seat 1 (`hp 1` -> dying) with black Sha -> yayaSkill2 revives to 1 HP & protects from further damage until turn 1!
    let bs1 = place_kind_suit(&mut e_yy, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_yy, 0, bs1, vec![1]);
    drain(&mut e_yy);
    assert_eq!(e_yy.debug_hp(1), 1);
    assert!(e_yy.debug_has_skill(1, "txhj_yayaSkill2_protected"));
    let bs2 = place_kind_suit(&mut e_yy, 0, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e_yy, 0, bs2, vec![1]);
    drain(&mut e_yy);
    assert_eq!(e_yy.debug_hp(1), 1, "yayaSkill2_protected prevents subsequent damage");
    // Advance to seat 1's TurnBegin -> clears protection
    e_yy.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_yy);
    drain(&mut e_yy);
    assert!(!e_yy.debug_has_skill(1, "txhj_yayaSkill2_protected"));

    // 3. txhj_youyouSkill1 (off-turn dmg -> draw 2) & txhj_youyouSkill2 (non-DrawPhase >1 gain -> heal friend 1)
    let mut e_uu = start_versus(0x1308, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_uu.debug_grant(1, "txhj_youyouSkill1");
    e_uu.debug_grant(1, "txhj_youyouSkill2");
    let sha_u = place_kind_suit(&mut e_uu, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_uu, 0, sha_u, vec![1]);
    drain(&mut e_uu);
    // Seat 1 dropped from 6 to 5 -> off-turn damage triggers youyou1 (+2 cards outside DrawPhase!) -> triggers youyou2 (+1 HP back to 6)!
    assert_eq!(e_uu.debug_hand(1).len(), 2);
    assert_eq!(e_uu.debug_hp(1), 6, "youyouSkill2 healed 1 HP on >1 card gain outside DrawPhase");

    // Counterexample: DrawPhase 2 cards must NOT trigger youyouSkill2
    let mut e_u2 = start_taixu(0x1309, 2);
    e_u2.debug_grant(0, "txhj_youyouSkill2");
    e_u2.debug_set_hp(0, 4);
    e_u2.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e_u2);
    drain(&mut e_u2);
    assert_eq!(e_u2.debug_hp(0), 4, "DrawPhase must NOT trigger txhj_youyouSkill2");
}

/// Upstream: `extension_buff.js` (`txhj_qilinSkill1..3`, `txhj_minminSkill1..2`, `txhj_ditingSkill1..3`, `txhj_qiaoqiaoSkill1..2`).
#[test]
fn l11_servants_qilin_minmin_diting_and_qiaoqiao() {
    // 1. txhj_qilinSkill1..3
    let mut e_ql = start_versus(0x130a, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_ql.debug_grant(0, "txhj_qilinSkill1");
    e_ql.debug_grant(0, "txhj_qilinSkill2");
    e_ql.debug_grant(0, "txhj_qilinSkill3");
    e_ql.debug_set_hp(0, 4);
    let _ = place_kind_suit(&mut e_ql, 1, CardKind::Sha, Suit::Heart);
    e_ql.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e_ql); // Judge -> Draw (+2 + qilin1 +2 = 4)
    drain(&mut e_ql);
    assert!(e_ql.debug_hand(0).len() >= 4, "qilinSkill1 doubled draw phase gain");
    // Enter Play phase -> qilinSkill3 uses virtual huogong on enemy 1 -> qilinSkill2 steals & heals on Fire dmg!
    let _ = next_phase(&mut e_ql);
    drain(&mut e_ql);
    assert!(e_ql.debug_hp(0) >= 4);

    // Explicitly verify txhj_qilinSkill2 steal + heal + save/restore via real FireSha
    let mut e_q2 = start_taixu(0x130b, 2);
    e_q2.debug_grant(0, "txhj_qilinSkill2");
    e_q2.debug_set_hp(0, 4);
    let fsha = place_kind(&mut e_q2, 0, CardKind::FireSha);
    let e1a = place_kind_suit(&mut e_q2, 1, CardKind::Shan, Suit::Diamond);
    let _e1b = place_kind_suit(&mut e_q2, 1, CardKind::Tao, Suit::Diamond);
    let _ = play_card(&mut e_q2, 0, fsha, vec![1]);
    let c_pass = expect_pending(&e_q2, "respond_shan");
    let _ = answer(&mut e_q2, &c_pass, vec![ChoiceOption::Pass]);
    let c_q2 = expect_pending(&e_q2, "txhj_qilinSkill2:choose_card");
    let e_q2_ok = save_restore_and_compare_pending(&e_q2, &c_q2, vec![ChoiceOption::Card(e1a)]);
    assert_eq!(e_q2_ok.debug_hp(0), 5);
    assert!(e_q2_ok.debug_hand(0).contains(&e1a));

    // 2. txhj_minminSkill1 & 2
    let mut e_mm = start_taixu(0x130c, 2);
    e_mm.debug_grant(0, "txhj_minminSkill1");
    e_mm.debug_grant(0, "txhj_minminSkill2");
    e_mm.debug_set_turn(1, Phase::Play, 1);
    let s1 = place_kind_suit(&mut e_mm, 1, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_mm, 1, s1, vec![0]);
    drain(&mut e_mm);
    // Damage 1 drops seat 0 to 5 -> minmin1 draws 1 (note < 2 so minmin2 doesn't heal on 1-card draw)
    assert_eq!(e_mm.debug_hand(0).len(), 1);
    assert_eq!(e_mm.debug_hp(0), 5);
    // Drawing 2 via Judge -> DrawPhase triggers minmin2 healing injured seat 0!
    e_mm.debug_set_turn(0, Phase::Judge, 2);
    let _ = next_phase(&mut e_mm);
    drain(&mut e_mm);
    assert_eq!(e_mm.debug_hp(0), 6, "minminSkill2 healed on 2-card draw");

    // 3. txhj_ditingSkill1, 2, 3
    let mut e_dt = start_taixu(0x130d, 2);
    e_dt.debug_grant(0, "txhj_ditingSkill1");
    e_dt.debug_grant(1, "txhj_ditingSkill2");
    e_dt.debug_grant(0, "txhj_ditingSkill3");
    e_dt.debug_set_hp(0, 4);
    let s_dt = place_kind_suit(&mut e_dt, 0, CardKind::Sha, Suit::Spade); // rank != others
    let _ = play_card(&mut e_dt, 0, s_dt, vec![1]);
    drain(&mut e_dt);
    assert_eq!(e_dt.debug_hp(0), 5, "ditingSkill1 recovered 1 HP on damage");
    assert_eq!(e_dt.state_core().player(1).max_hp, 7, "ditingSkill2 +1 max_hp on taking dmg");
    let h_before = e_dt.debug_hand(0).len();
    e_dt.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_dt); // End -> ditingSkill3 draws 1 (1 distinct rank) & deals 1 to enemy 1!
    drain(&mut e_dt);
    assert_eq!(e_dt.debug_hand(0).len(), h_before + 1);
    assert_eq!(e_dt.debug_hp(1), 4, "ditingSkill3 dealt 1 damage");

    // 4. txhj_qiaoqiaoSkill1 & 2
    let mut e_qq = start_taixu(0x130e, 2);
    e_qq.debug_grant(0, "txhj_qiaoqiaoSkill1");
    e_qq.debug_grant(0, "txhj_qiaoqiaoSkill2");
    e_qq.debug_set_hp(0, 4);
    let le = place_kind_suit(&mut e_qq, 0, CardKind::Le, Suit::Heart);
    let _ = play_card(&mut e_qq, 0, le, vec![1]); // places delayed trick on seat 1 -> qiaoqiaoSkill1 drains 1 HP from enemy 1!
    drain(&mut e_qq);
    assert_eq!(e_qq.debug_hp(1), 5, "qiaoqiaoSkill1 triggered on addJudge");
    e_qq.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_qq); // End -> qiaoqiaoSkill2 fills hand up to min(hp=4, 5) = 4
    drain(&mut e_qq);
    assert_eq!(e_qq.debug_hand(0).len(), 4);
}

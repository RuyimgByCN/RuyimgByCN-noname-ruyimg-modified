use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::engine::RejectReason;
use engine_native::state::GameMode;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension_card.js:233` (`txhj_zhijizhibi`), `305` (`txhj_yuanjiao`), `951` (`txhj_diaohu`).
#[test]
fn l11_cards_zhibi_yuanjiao_diaohu_and_wuxie() {
    // 1. txhj_zhijizhibi: reveal target hand -> confirm -> draw 1; wuxie nullifies
    let mut e = start_taixu(0x1101, 2);
    let zb = e.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
    let opp_card = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    let _out = play_card(&mut e, 0, zb, vec![1]);
    drain(&mut e); // no wuxie -> advances to confirm? Wait: pass_until confirm!
    let mut e_obs = start_taixu(0x1101, 2);
    let zb2 = e_obs.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
    let _ = place_kind_suit(&mut e_obs, 1, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_obs, 0, zb2, vec![1]);
    let c_conf = answer_pass_until(&mut e_obs, "txhj_zhijizhibi:confirm");
    let log_before = e_obs.log().len();
    assert!(log_has_reveal(&e_obs.log()[..log_before], 0));
    let e_after = save_restore_and_compare_pending(&e_obs, &c_conf, vec![ChoiceOption::Confirm]);
    assert_eq!(e_after.debug_hand(0).len(), 1);
    assert!(e_after.state_core().table.is_empty());
    let _ = opp_card;

    // Counterexample: wuxie cancels zhibi reveal & draw
    let mut e_wx = start_taixu(0x1102, 2);
    let zb_wx = e_wx.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
    let _ = place_kind_suit(&mut e_wx, 1, CardKind::Sha, Suit::Spade);
    let wx = place_kind_suit(&mut e_wx, 1, CardKind::Nullify, Suit::Spade);
    let _ = play_card(&mut e_wx, 0, zb_wx, vec![1]);
    let cw = e_wx.open_choice().expect("wuxie choice").clone();
    let _ = answer(&mut e_wx, &cw, vec![ChoiceOption::Card(wx)]);
    drain(&mut e_wx);
    assert_eq!(e_wx.debug_hand(0).len(), 0);
    assert!(!log_has_reveal(e_wx.log(), 0));

    // chongzhu: true recast on PlayCard with 0 targets discards & draws 1 without peeking
    let mut e_emp = start_taixu(0x1103, 2);
    let zb_e = e_emp.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
    let _ = play_card(&mut e_emp, 0, zb_e, Vec::new());
    drain(&mut e_emp);
    assert_eq!(e_emp.debug_hand(0).len(), 1);
    assert!(e_emp.state_core().discard.contains(&zb_e));
    assert_eq!(flag(&e_emp, 0, "l11_zhijizhibi_peeked"), 0);

    // 2. txhj_yuanjiao: target draws 1, owner draws 3; wuxie cancels
    let mut e_yj = start_taixu(0x1104, 2);
    let yj = e_yj.debug_move_named_card_to_hand(0, "txhj_yuanjiao");
    let _ = play_card(&mut e_yj, 0, yj, vec![1]);
    drain(&mut e_yj);
    assert_eq!(e_yj.debug_hand(1).len(), 1);
    assert_eq!(e_yj.debug_hand(0).len(), 3);

    let mut e_yj_wx = start_taixu(0x1105, 2);
    let yj2 = e_yj_wx.debug_move_named_card_to_hand(0, "txhj_yuanjiao");
    let wx2 = place_kind(&mut e_yj_wx, 1, CardKind::Nullify);
    let _ = play_card(&mut e_yj_wx, 0, yj2, vec![1]);
    let cw2 = e_yj_wx.open_choice().expect("wuxie").clone();
    let _ = answer(&mut e_yj_wx, &cw2, vec![ChoiceOption::Card(wx2)]);
    drain(&mut e_yj_wx);
    assert_eq!(e_yj_wx.debug_hand(1).len(), 0);
    assert_eq!(e_yj_wx.debug_hand(0).len(), 0);

    // 3. txhj_diaohu: flip target, owner draws 1, target draws 1; wuxie cancels
    let mut e_dh = start_taixu(0x1106, 2);
    let dh = e_dh.debug_move_named_card_to_hand(0, "txhj_diaohu");
    let _ = play_card(&mut e_dh, 0, dh, vec![1]);
    drain(&mut e_dh);
    assert!(e_dh.debug_face_down(1));
    assert_eq!(e_dh.debug_hand(0).len(), 1);
    assert_eq!(e_dh.debug_hand(1).len(), 1);

    let mut e_dh_wx = start_taixu(0x1107, 2);
    let dh2 = e_dh_wx.debug_move_named_card_to_hand(0, "txhj_diaohu");
    let wx3 = place_kind(&mut e_dh_wx, 1, CardKind::Nullify);
    let _ = play_card(&mut e_dh_wx, 0, dh2, vec![1]);
    let cw3 = e_dh_wx.open_choice().expect("wuxie").clone();
    let _ = answer(&mut e_dh_wx, &cw3, vec![ChoiceOption::Card(wx3)]);
    drain(&mut e_dh_wx);
    assert!(!e_dh_wx.debug_face_down(1));
    assert_eq!(e_dh_wx.debug_hand(0).len(), 0);
}

/// Upstream: `extension_card.js:334` (`txhj_yiyi`: allies draw 2 discard 2, per-target nullify).
#[test]
fn l11_card_yiyi_friends_per_target_nullify_ai_and_restore() {
    let mut e = start_versus(0x1108, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    let yiyi = e.debug_move_named_card_to_hand(0, "txhj_yiyi");
    let _init1 = place_kind(&mut e, 1, CardKind::Sha); // 1 + 2 = 3 > 2 -> prompts discard_cards
    let wx = place_kind_suit(&mut e, 2, CardKind::Nullify, Suit::Spade);
    let _ = play_card(&mut e, 0, yiyi, Vec::new());

    // Seat 2 nullifies seat 0 ONLY; seat 1 still resolves
    let c_wx0 = e.open_choice().expect("wuxie for seat 0").clone();
    assert_eq!(c_wx0.observer, 2);
    let _ = answer(&mut e, &c_wx0, vec![ChoiceOption::Card(wx)]);

    // Seat 1 has no wuxie counter-ask from enemies and draws 2 -> discard 2 prompt
    let c_disc1 = answer_pass_until(&mut e, "txhj_yiyi:discard_cards");
    assert_eq!(c_disc1.observer, 1);
    assert_eq!(c_disc1.spec.min, 2);
    assert_eq!(c_disc1.spec.max, 2);
    assert!(!c_disc1.spec.allow_cancel);

    // AI chooses 2 cards cleanly using public info, and save/restore agrees
    let ai_act = ai_plan(&e, 1);
    let sel = match ai_act {
        GameAction::Answer { choice_id, selection } => {
            assert_eq!(choice_id, c_disc1.id);
            assert_eq!(selection.len(), 2);
            selection
        }
        other => panic!("expected Answer from AI, got {other:?}"),
    };
    let e_done = save_restore_and_compare_pending(&e, &c_disc1, sel);
    assert_eq!(e_done.debug_hand(0).len(), 0, "nullified seat 0 neither drew nor discarded");
    assert_eq!(e_done.debug_hand(1).len(), 1, "ally seat 1 kept 1 after drawing 2 and discarding 2");
    assert_eq!(e_done.debug_hand(2).len(), 0, "enemy seat 2 was not targeted");
}

/// Upstream: `extension_card.js:704` (`txhj_jiedao`: use Sha on victim OR lose 1 HP).
#[test]
fn l11_card_jiedao_use_sha_vs_lose_hp_and_no_weapon_reject() {
    // 1. Holder declines -> LoseHp(1), weapon NOT transferred
    let mut e = start_taixu(0x1109, 3);
    let _wpn = equip_kind(&mut e, 1, CardKind::Zhuge);
    let jd = e.debug_move_named_card_to_hand(0, "txhj_jiedao");
    let _sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e, 0, jd, vec![1, 2]);
    let c = answer_pass_until(&mut e, "txhj_jiedao:choose_sha");
    assert_eq!(c.observer, 1);
    assert!(c.spec.allow_cancel);
    let _ = answer(&mut e, &c, vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5, "declining txhj_jiedao causes holder to lose 1 HP");
    assert_eq!(e.debug_hp(2), 6);
    assert_eq!(e.debug_equips(1).len(), 1, "weapon must NOT be transferred on decline");
    assert!(e.debug_equips(0).is_empty());

    // 2. Holder uses Sha -> deals 1 damage to victim, holder keeps full HP
    let mut e2 = start_taixu(0x110a, 3);
    let _wpn2 = equip_kind(&mut e2, 1, CardKind::Zhuge);
    let jd2 = e2.debug_move_named_card_to_hand(0, "txhj_jiedao");
    let sha2 = place_kind_suit(&mut e2, 1, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e2, 0, jd2, vec![1, 2]);
    let c2 = answer_pass_until(&mut e2, "txhj_jiedao:choose_sha");
    let _ = answer(&mut e2, &c2, vec![ChoiceOption::Card(sha2)]);
    drain(&mut e2);
    assert_eq!(e2.debug_hp(1), 6);
    assert_eq!(e2.debug_hp(2), 5, "victim takes Sha damage");

    // 3. Negative: victim out of holder's attack range in 4-seat circle is rejected (`IllegalTargets`)
    let mut e3 = start_taixu(0x110b, 4);
    let jd3 = e3.debug_move_named_card_to_hand(0, "txhj_jiedao");
    assert!(matches!(
        play_card(&mut e3, 0, jd3, vec![1, 3]),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
}

/// Upstream: `extension_card.js:775` (`txhj_huoshaolianying`: fire chain across next friend, cycle guard).
#[test]
fn l11_card_huoshaolianying_friend_chain_wuxie_and_cycle_guard() {
    // 1. 4-seat [0:Wei, 1:Shu, 2:Shu, 3:Wei] -> hits 1, chains to friend 2, stops before 3
    let mut e = start_versus(
        0x110c,
        &[&[0, 3], &[1, 2]],
        &[Faction::Wei, Faction::Shu],
    );
    let hs = e.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let _ = play_card(&mut e, 0, hs, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5);
    assert_eq!(e.debug_hp(2), 5);
    assert_eq!(e.debug_hp(3), 6, "chain stops when next alive is not target's friend");

    // 2. Wuxie on seat 1 stops the entire huoshaolianying chain (neither seat 1 nor seat 2 takes damage)
    let mut e_wx = start_versus(
        0x110d,
        &[&[0, 3], &[1, 2]],
        &[Faction::Wei, Faction::Shu],
    );
    let hs_w = e_wx.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let wx = place_kind_suit(&mut e_wx, 1, CardKind::Nullify, Suit::Club);
    let _ = play_card(&mut e_wx, 0, hs_w, Vec::new());
    let c = e_wx.open_choice().expect("wuxie on seat 1").clone();
    let _ = answer(&mut e_wx, &c, vec![ChoiceOption::Card(wx)]);
    drain(&mut e_wx);
    assert_eq!(e_wx.debug_hp(1), 6, "seat 1 nullified");
    assert_eq!(e_wx.debug_hp(2), 6, "nullifying seat 1 prevents chain to seat 2");

    // 3. All-friend 3-seat ring terminates cleanly due to visited guard
    let mut e_cyc = start_versus(0x110e, &[&[0, 1, 2]], &[Faction::Wei]);
    let hs_c = e_cyc.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let _ = play_card(&mut e_cyc, 0, hs_c, Vec::new());
    drain(&mut e_cyc);
    assert_eq!(e_cyc.debug_hp(1), 5);
    assert_eq!(e_cyc.debug_hp(2), 5);
    assert_eq!(e_cyc.debug_hp(0), 5);
}

/// Upstream: `extension_card.js:1076` (`txhj_caomu`), `1287` (`txhj_wangmeizhike`), `1356` (`txhj_shengdong`).
#[test]
fn l11_cards_caomu_wangmeizhike_and_shengdong() {
    // 1. txhj_caomu on seat 1 in 4-seat game: seat 1 discards 1; dist<=1 seats (0, 2) draw 1; seat 3 (dist=2) draws 0
    let mut e = start_taixu(0x110f, 4);
    let cm = e.debug_move_named_card_to_hand(0, "txhj_caomu");
    let c1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e, 0, cm, vec![1]);
    let c = answer_pass_until(&mut e, "txhj_caomu:discard_card");
    assert_eq!(c.observer, 1);
    let _ = answer(&mut e, &c, vec![ChoiceOption::Card(c1)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0);
    assert_eq!(e.debug_hand(0).len(), 1);
    assert_eq!(e.debug_hand(2).len(), 1);
    assert_eq!(e.debug_hand(3).len(), 0, "dist > 1 seat must NOT draw");

    // 2. txhj_wangmeizhike: Club recovers 1 HP; non-Club draws 1 card
    let mut e_wm = start_taixu(0x1110, 2);
    e_wm.debug_set_hp(0, 4);
    let wm1 = e_wm.debug_move_named_card_to_hand(0, "txhj_wangmeizhike");
    let _ = top_deck_suit(&mut e_wm, Suit::Club);
    let _ = play_card(&mut e_wm, 0, wm1, vec![0]);
    drain(&mut e_wm);
    assert_eq!(e_wm.debug_hp(0), 5, "Club judge recovers 1 HP");
    assert_eq!(e_wm.debug_hand(0).len(), 0);

    let wm2 = e_wm.debug_move_named_card_to_hand(0, "txhj_wangmeizhike");
    let _ = top_deck_suit(&mut e_wm, Suit::Heart);
    let _ = play_card(&mut e_wm, 0, wm2, vec![0]);
    drain(&mut e_wm);
    assert_eq!(e_wm.debug_hp(0), 5);
    assert_eq!(e_wm.debug_hand(0).len(), 1, "non-Club judge draws 1");

    // 3. txhj_shengdong: give 1 to t1, then t1 gives 2 to t2 (+ mid-choice save/restore)
    let mut e_sd = start_taixu(0x1111, 3);
    let sd = e_sd.debug_move_named_card_to_hand(0, "txhj_shengdong");
    let give0 = place_kind_suit(&mut e_sd, 0, CardKind::Sha, Suit::Diamond);
    let h1a = place_kind_suit(&mut e_sd, 1, CardKind::Shan, Suit::Heart);
    let h1b = place_kind_suit(&mut e_sd, 1, CardKind::Tao, Suit::Heart);
    let _ = play_card(&mut e_sd, 0, sd, vec![1, 2]);
    let c_g1 = answer_pass_until(&mut e_sd, "txhj_shengdong:give_card1");
    let _ = answer(&mut e_sd, &c_g1, vec![ChoiceOption::Card(give0)]);
    let c_g2 = expect_pending(&e_sd, "txhj_shengdong:give_card2");
    assert_eq!(c_g2.observer, 1);
    let e_sd_done = save_restore_and_compare_pending(
        &e_sd,
        &c_g2,
        vec![ChoiceOption::Card(h1a), ChoiceOption::Card(h1b)],
    );
    assert_eq!(e_sd_done.debug_hand(0).len(), 0);
    assert_eq!(e_sd_done.debug_hand(1).len(), 1);
    assert_eq!(e_sd_done.debug_hand(2).len(), 2);
}

/// Upstream: `extension_card.js:1437` (`txhj_shuiyan`), `1897` (`txhj_jiuchiroulin`), `extension_skill.js:458` (`huogong`).
#[test]
fn l11_cards_shuiyan_jiuchiroulin_and_huogong() {
    // 1a. txhj_shuiyan with equip rank sum < 6 -> immediate 1 Thunder damage
    let mut e_lt = start_taixu(0x1112, 2);
    let _zhuge = equip_named(&mut e_lt, 1, "txhj_zhuge"); // rank 1 < 6
    let sy = e_lt.debug_move_named_card_to_hand(0, "txhj_shuiyan");
    let _ = play_card(&mut e_lt, 0, sy, vec![1]);
    drain(&mut e_lt);
    assert_eq!(e_lt.debug_hp(1), 5);
    assert_eq!(e_lt.debug_equips(1).len(), 1);

    // 1b. txhj_shuiyan with equip rank sum >= 6 -> partial pick re-prompts, full sum >= 6 discards, cancel deals damage
    let mut e_ge = start_taixu(0x1113, 2);
    let eq_a = equip_named(&mut e_ge, 1, "txhj_chongyingshenfu"); // rank 2
    let eq_b = equip_named(&mut e_ge, 1, "txhj_juechenjinge"); // rank 5 -> sum 7 >= 6
    let sy2 = e_ge.debug_move_named_card_to_hand(0, "txhj_shuiyan");
    let _ = play_card(&mut e_ge, 0, sy2, vec![1]);
    let c_ask = answer_pass_until(&mut e_ge, "txhj_shuiyan:discard_equips");
    assert_eq!(ai_plan(&e_ge, 1), GameAction::Answer {
        choice_id: c_ask.id,
        selection: vec![ChoiceOption::Card(eq_a), ChoiceOption::Card(eq_b)],
    });
    // Insufficient rank (<6) re-prompts cleanly:
    let _ = answer(&mut e_ge, &c_ask, vec![ChoiceOption::Card(eq_a)]);
    let c_re = expect_pending(&e_ge, "txhj_shuiyan:discard_equips");
    let e_disc = save_restore_and_compare_pending(
        &e_ge,
        &c_re,
        vec![ChoiceOption::Card(eq_a), ChoiceOption::Card(eq_b)],
    );
    assert_eq!(e_disc.debug_hp(1), 6);
    assert!(e_disc.debug_equips(1).is_empty());

    // Cancel falls back to 1 Thunder damage
    let _ = answer(&mut e_ge, &c_re, Vec::new());
    drain(&mut e_ge);
    assert_eq!(e_ge.debug_hp(1), 5);
    assert_eq!(e_ge.debug_equips(1).len(), 2);

    // 2. txhj_jiuchiroulin: reverse order (2 then 1); Jiu negates, Pass damages
    let mut e_jc = start_taixu(0x1114, 3);
    let jc = e_jc.debug_move_named_card_to_hand(0, "txhj_jiuchiroulin");
    let jiu1 = place_kind(&mut e_jc, 1, CardKind::Jiu);
    let _jiu2 = place_kind(&mut e_jc, 2, CardKind::Jiu);
    let _ = play_card(&mut e_jc, 0, jc, Vec::new());
    let c_t2 = answer_pass_until(&mut e_jc, "txhj_jiuchiroulin:respond_jiu");
    assert_eq!(c_t2.observer, 2, "reverse AOE must ask seat 2 before seat 1");
    let _ = answer(&mut e_jc, &c_t2, vec![ChoiceOption::Pass]);
    let c_t1 = answer_pass_until(&mut e_jc, "txhj_jiuchiroulin:respond_jiu");
    assert_eq!(c_t1.observer, 1);
    let _ = answer(&mut e_jc, &c_t1, vec![ChoiceOption::Card(jiu1)]);
    drain(&mut e_jc);
    assert_eq!(e_jc.debug_hp(2), 5);
    assert_eq!(e_jc.debug_hp(1), 6);
    assert!(e_jc.debug_hand(1).is_empty());

    // 3. huogong: show card -> discard same suit -> Fire 1; mismatch -> 0 damage
    let mut e_hg = start_taixu(0x1115, 2);
    let hg = e_hg.debug_move_named_card_to_hand(0, "huogong");
    let own_spade = place_kind_suit(&mut e_hg, 0, CardKind::Sha, Suit::Spade);
    let tgt_spade = place_kind_suit(&mut e_hg, 1, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_hg, 0, hg, vec![1]);
    let c_show = answer_pass_until(&mut e_hg, "huogong:show_card");
    let _ = answer(&mut e_hg, &c_show, vec![ChoiceOption::Card(tgt_spade)]);
    let c_disc = expect_pending(&e_hg, "huogong:discard_card");
    let e_hg_ok = save_restore_and_compare_pending(&e_hg, &c_disc, vec![ChoiceOption::Card(own_spade)]);
    assert_eq!(e_hg_ok.debug_hp(1), 5);
    assert!(e_hg_ok.debug_hand(0).is_empty());
}

/// Upstream: `extension_card.js:1977-2085` (`txzhaoshu_skill`, `txzhaoshu_global`).
#[test]
fn l11_card_txzhaoshu_skill_and_global_pile_suits() {
    let mut e = start_versus(0x1116, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    let zs = e.debug_move_named_card_to_hand(0, "txzhaoshu_skill");
    let _ = play_card(&mut e, 0, zs, Vec::new());
    drain(&mut e);
    assert!(e.debug_has_skill(0, "txzhaoshu_skill"));
    assert!(e.debug_has_skill(1, "txzhaoshu_global"));
    assert!(e.state_core().vm_aux.piles.contains_key(&(0, "txzhaoshu_skill".to_string())));

    // Undamaged seat 0 places 1 Spade; Counterexample: 1 suit is NOT enough to trigger `txzhaoshu_skill`
    let s1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let out0 = activate_skill(&mut e, 0, "txzhaoshu_global");
    assert!(matches!(out0, CommandOutcome::Pending { .. }));
    let c_g0 = expect_pending(&e, "txzhaoshu_global:give_cards");
    assert_eq!(c_g0.spec.max, 1);
    let _ = answer(&mut e, &c_g0, vec![ChoiceOption::Card(s1)]);
    drain(&mut e);
    assert!(matches!(
        activate_skill(&mut e, 0, "txzhaoshu_skill"),
        CommandOutcome::Rejected(_)
    ));

    // Damaged friend seat 1 enters Play phase and places a Heart + Club (2 suits in pile!)
    e.debug_set_hp(1, 4);
    e.debug_set_turn(1, Phase::Play, 1);
    let h1 = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Heart);
    let _ = activate_skill(&mut e, 1, "txzhaoshu_global");
    let c_g1 = expect_pending(&e, "txzhaoshu_global:give_cards");
    let _ = answer(&mut e, &c_g1, vec![ChoiceOption::Card(h1)]);
    drain(&mut e);

    // Now owner 0 has >= 2 distinct suits in `txzhaoshu_cards` -> activates `txzhaoshu_skill` cleanly!
    e.debug_set_turn(0, Phase::Play, 2);
    let out_act = activate_skill(&mut e, 0, "txzhaoshu_skill");
    assert!(matches!(out_act, CommandOutcome::Accepted { .. }), "{out_act:?}");
    assert!(
        !e.state_core().vm_aux.piles.contains_key(&(0, "txzhaoshu_cards".to_string())),
        "txzhaoshu_cards pile must be discarded after activation"
    );
    assert_eq!(e.debug_hand(0).len(), 1, "owner gains one of the G1 tricks or draws 1");
    assert!(matches!(e.state_core().mode, GameMode::Versus { .. }));
}

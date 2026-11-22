use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension.js:4380-4550` (`oldfuman`, `oldfuman2`, `txhuao`, `hgkurou`, `hgkurou_effect`, `gdsanchen`).
#[test]
fn l11_b2_skills_d_oldfuman_txhuao_hgkurou_gdsanchen() {
    // 1. oldfuman & oldfuman2: Play gives Sha to other -> target gains oldfuman2; target using Sha draws 1 for owner; clears at TurnEnded.
    let mut e = start_versus(0x2101, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "oldfuman");
    e.debug_set_turn(0, Phase::Play, 1);
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _ = activate_skill(&mut e, 0, "oldfuman");
    let c_card = expect_pending(&e, "oldfuman:choose_card");
    let _ = answer(&mut e, &c_card, vec![ChoiceOption::Card(sha0)]);
    let c_tgt = expect_pending(&e, "oldfuman:choose_target");
    assert_eq!(ai_plan(&e, 0), GameAction::Answer {
        choice_id: c_tgt.id,
        selection: vec![ChoiceOption::Player(1)], // ally over enemy!
    });
    let _ = answer(&mut e, &c_tgt, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert!(e.debug_hand(1).contains(&sha0));
    assert!(e.debug_has_skill(1, "oldfuman2"));

    // Target 1 uses Sha in own turn -> owner 0 draws 1; TurnEnded removes oldfuman2.
    e.debug_set_turn(1, Phase::Play, 2);
    let _ = play_card(&mut e, 1, sha0, vec![2]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "oldfuman2 draws 1 for giver on Sha use");
    e.debug_set_turn(1, Phase::End, 2);
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert!(!e.debug_has_skill(1, "oldfuman2"), "oldfuman2 clears at TurnEnded");

    // 2. txhuao: TurnBegin draws bottom Sha from deck
    let mut e_ha = start_taixu(0x2102, 2);
    e_ha.debug_grant(0, "txhuao");
    e_ha.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_ha); // TurnBegin(0)
    drain(&mut e_ha);
    assert_eq!(e_ha.debug_hand(0).len(), 1);
    assert!(e_ha.catalog().card_kind(e_ha.debug_hand(0)[0]).is_sha_like());

    // 3. hgkurou & hgkurou_effect: cancel refunds once-per-turn; confirm discards 1, loses 1 HP, draws 3, +1 Sha & +3 hand limit
    let mut e_hg = start_taixu(0x2103, 2);
    e_hg.debug_grant(0, "hgkurou");
    e_hg.debug_set_turn(0, Phase::Play, 1);
    let k0 = place_kind(&mut e_hg, 0, CardKind::Shan);
    let _ = activate_skill(&mut e_hg, 0, "hgkurou");
    let c_hg = expect_pending(&e_hg, "hgkurou:choose_card");
    let _ = answer(&mut e_hg, &c_hg, Vec::new()); // cancel refunds once-per-turn!
    drain(&mut e_hg);
    assert_eq!(e_hg.debug_hp(0), 6);
    assert_eq!(e_hg.debug_hand(0).len(), 1);

    // Re-activate hgkurou -> confirm:
    let _ = activate_skill(&mut e_hg, 0, "hgkurou");
    let c_hg2 = expect_pending(&e_hg, "hgkurou:choose_card");
    let e_hg2 = save_restore_and_compare_pending(&e_hg, &c_hg2, vec![ChoiceOption::Card(k0)]);
    assert_eq!(e_hg2.debug_hp(0), 5);
    assert_eq!(e_hg2.debug_hand(0).len(), 3);
    assert!(e_hg2.debug_has_skill(0, "hgkurou_effect"));
    // Verify +1 Sha limit (can play 2 Shas):
    let mut e_hg_sha = e_hg2;
    let sa = place_kind(&mut e_hg_sha, 0, CardKind::Sha);
    let sb = place_kind(&mut e_hg_sha, 0, CardKind::Sha);
    assert!(matches!(play_card(&mut e_hg_sha, 0, sa, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e_hg_sha);
    assert!(matches!(play_card(&mut e_hg_sha, 0, sb, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e_hg_sha);

    // 4. gdsanchen: 3 distinct card types resets once-per-turn; same-type 3 cards does NOT reset!
    let mut e_sc = start_versus(0x2104, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_sc.debug_grant(0, "gdsanchen");
    e_sc.debug_set_turn(0, Phase::Play, 1);
    let b1 = place_kind(&mut e_sc, 1, CardKind::Sha);
    let t1 = place_kind(&mut e_sc, 1, CardKind::Wuzhong);
    let eq1 = place_kind(&mut e_sc, 1, CardKind::Zhuge);
    let extra = place_kind(&mut e_sc, 1, CardKind::Shan);
    let _ = extra;
    let _ = activate_skill(&mut e_sc, 0, "gdsanchen");
    let c_s_tgt = expect_pending(&e_sc, "gdsanchen:choose_target");
    assert_eq!(ai_plan(&e_sc, 0), GameAction::Answer {
        choice_id: c_s_tgt.id,
        selection: vec![ChoiceOption::Player(0)],
    });
    let _ = answer(&mut e_sc, &c_s_tgt, vec![ChoiceOption::Player(1)]);
    let c_s_dis = expect_pending(&e_sc, "gdsanchen:discard_cards");
    // AI picks 3 distinct categories:
    let ai_sel = ai_plan(&e_sc, 1);
    assert!(matches!(ai_sel, GameAction::Answer { .. }));
    let mut e_sc2 = save_restore_and_compare_pending(
        &e_sc,
        &c_s_dis,
        vec![ChoiceOption::Card(b1), ChoiceOption::Card(t1), ChoiceOption::Card(eq1)],
    );
    // Reset happened -> can activate gdsanchen a 2nd time on another target (AI prefers ally 0 or 2? 0 is done? No, 1 is done so AI picks self 0 over enemy 2!):
    let s2a = place_kind(&mut e_sc2, 0, CardKind::Sha);
    let s2b = place_kind(&mut e_sc2, 0, CardKind::Sha);
    let s2c = place_kind(&mut e_sc2, 0, CardKind::Sha);
    let out2 = activate_skill(&mut e_sc2, 0, "gdsanchen");
    assert!(matches!(out2, CommandOutcome::Pending { .. }), "distinct 3-type gdsanchen must reset");
    let c_s_tgt2 = expect_pending(&e_sc2, "gdsanchen:choose_target");
    let _ = answer(&mut e_sc2, &c_s_tgt2, vec![ChoiceOption::Player(0)]);
    let c_s_dis2 = expect_pending(&e_sc2, "gdsanchen:discard_cards");
    let _ = answer(
        &mut e_sc2,
        &c_s_dis2,
        vec![ChoiceOption::Card(s2a), ChoiceOption::Card(s2b), ChoiceOption::Card(s2c)],
    );
    drain(&mut e_sc2);
    // Same-type 3 Shas did NOT reset -> 3rd activation rejected:
    let out3 = activate_skill(&mut e_sc2, 0, "gdsanchen");
    assert!(matches!(out3, CommandOutcome::Rejected(_)), "non-distinct gdsanchen must not reset");
}

/// Upstream: `extension.js:4555-4695` (`txlveming`, `txdaoxi`, `txlangxi`, `txkuangyi`).
#[test]
fn l11_b2_skills_d_txlveming_txdaoxi_txlangxi_txkuangyi() {
    // 1. txlveming: target has more equips -> hit deals 3 dmg; miss steals 1 + deals 1 dmg!
    let mut e = start_versus(0x2105, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txlveming");
    let eq_1 = equip_kind(&mut e, 1, CardKind::Renwang);
    let _ = eq_1;
    let h1 = place_kind(&mut e, 1, CardKind::Tao);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut e, 0, "txlveming");
    let c_lm_t = expect_pending(&e, "txlveming:choose_target");
    assert_eq!(c_lm_t.spec.options, vec![ChoiceOption::Player(1)]); // seat 2 has 0 equips!
    let _ = answer(&mut e, &c_lm_t, vec![ChoiceOption::Player(1)]);
    let c_lm_r = expect_pending(&e, "txlveming:choose_rank");
    // Branch A (hit): deck top rank 5, target picks tag 4 (rank 5) -> 3 dmg!
    let mut e_hit = e.clone();
    let _ = top_deck_rank(&mut e_hit, 5);
    let e_hit = save_restore_and_compare_pending(&e_hit, &c_lm_r, vec![ChoiceOption::Tag(4)]);
    assert_eq!(e_hit.debug_hp(1), 3, "txlveming hit deals 3 dmg");

    // Branch B (miss): deck top rank 9, target picks tag 4 -> steal 1 + deal 1 dmg!
    let mut e_miss = e;
    let _ = top_deck_rank(&mut e_miss, 9);
    let _ = answer(&mut e_miss, &c_lm_r, vec![ChoiceOption::Tag(4)]);
    drain(&mut e_miss);
    assert_eq!(e_miss.debug_hp(1), 5, "txlveming miss deals 1 dmg");
    assert_eq!(e_miss.debug_hand(0).len(), 1, "txlveming miss steals 1");
    let _ = h1;

    // 2. txdaoxi: usable:3 Red card -> Snatch! Black or 4th Red rejected!
    let mut e_dx = start_taixu(0x2106, 2);
    e_dx.debug_grant(0, "txdaoxi");
    e_dx.debug_set_turn(0, Phase::Play, 1);
    let _ = place_kind(&mut e_dx, 1, CardKind::Tao);
    let blk_null = place_kind_suit(&mut e_dx, 0, CardKind::Nullify, Suit::Spade);
    assert!(
        matches!(play_card(&mut e_dx, 0, blk_null, vec![1]), CommandOutcome::Rejected(_)),
        "txdaoxi must reject Black Nullify"
    );
    let mut snatched = 0usize;
    for i in 0..4 {
        let _p1_loot = place_kind(&mut e_dx, 1, CardKind::Tao);
        let red_shan = place_kind_suit(&mut e_dx, 0, CardKind::Shan, Suit::Diamond);
        let out = play_card(&mut e_dx, 0, red_shan, vec![1]);
        if i < 3 {
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "txdaoxi #{i} Red Shan as Snatch must be accepted: {out:?}"
            );
            drain(&mut e_dx);
            snatched += 1;
        } else {
            assert!(
                matches!(out, CommandOutcome::Rejected(_)),
                "txdaoxi 4th Red Shan must be rejected (usable:3)"
            );
        }
    }
    assert_eq!(snatched, 3);

    // 3. txlangxi & txkuangyi:
    let mut e_lk = start_versus(0x2107, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_lk.debug_grant(0, "txlangxi");
    e_lk.debug_grant(0, "txkuangyi");
    e_lk.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_lk); // Turn 1 begins -> next Prepare triggers txlangxi
    e_lk.debug_set_turn(0, Phase::End, 1);
    // Actually trigger Seat 0 Prepare from Turn 1 End:
    let mut e_l = start_versus(0x2108, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_l.debug_grant(0, "txlangxi");
    e_l.debug_grant(0, "txkuangyi");
    e_l.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_l); // -> Turn 0 Prepare: txlangxi
    let c_lx = expect_pending(&e_l, "txlangxi:choose_target");
    assert_eq!(ai_plan(&e_l, 0), GameAction::Answer {
        choice_id: c_lx.id,
        selection: vec![ChoiceOption::Player(1)],
    });
    let _ = answer(&mut e_l, &c_lx, vec![ChoiceOption::Player(1)]);
    drain(&mut e_l);
    assert!((2..=4).contains(&(6 - e_l.debug_hp(1))), "txlangxi deals 2..=4 dmg");

    // txkuangyi: Play basic card (Sha) -> confirm flips link & retrieves Sha once per turn!
    e_l.debug_set_turn(0, Phase::Play, 2);
    let sha_ky = place_kind(&mut e_l, 0, CardKind::Sha);
    let _ = play_card(&mut e_l, 0, sha_ky, vec![1]);
    let c_ky = expect_pending(&e_l, "txkuangyi:confirm");
    let e_l2 = save_restore_and_compare_pending(&e_l, &c_ky, vec![ChoiceOption::Confirm]);
    assert!(e_l2.debug_linked(0), "txkuangyi links owner");
    assert!(e_l2.debug_hand(0).contains(&sha_ky), "txkuangyi retrieves basic card");
    let _ = e_lk;
}

/// Upstream: `extension.js:4697-4800` (`txtanbei`, `ljxuechi`, `txzhuxin`, `txyicheng`, `txnianrui`).
#[test]
fn l11_b2_skills_d_txtanbei_ljxuechi_txzhuxin_txyicheng_txnianrui() {
    // 1. txtanbei: mode 0 steals + grants effect2 (owner CAN still target!) & effect3; mode 1 grants effect1 (seal) & effect3 (unlimited range/uses); cancel refunds.
    let mut e = start_taixu(0x2109, 3);
    e.debug_grant(0, "txtanbei");
    let loot1 = place_kind(&mut e, 1, CardKind::Tao);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut e, 0, "txtanbei");
    let c_tb_t = expect_pending(&e, "txtanbei:choose_target");
    let _ = answer(&mut e, &c_tb_t, Vec::new()); // cancel refunds once-per-turn!
    drain(&mut e);
    assert!(matches!(activate_skill(&mut e, 0, "txtanbei"), CommandOutcome::Pending { .. }));
    let c_tb_t2 = expect_pending(&e, "txtanbei:choose_target");
    let _ = answer(&mut e, &c_tb_t2, vec![ChoiceOption::Player(1)]);
    let c_tb_m = expect_pending(&e, "txtanbei:choose_mode");
    // Mode 0 branch: steals card + adds txtanbei_effect2; verify owner can STILL play Sha on seat 1!
    let mut e_m0 = save_restore_and_compare_pending(&e, &c_tb_m, vec![ChoiceOption::Tag(0)]);
    assert!(e_m0.debug_hand(0).contains(&loot1), "txtanbei mode 0 steals");
    assert!(e_m0.debug_has_skill(1, "txtanbei_effect2"));
    assert!(e_m0.debug_has_skill(0, "txtanbei_effect3"));
    let sha_m0 = place_kind(&mut e_m0, 0, CardKind::Sha);
    assert!(
        matches!(play_card(&mut e_m0, 0, sha_m0, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "txtanbei_effect2 must NOT forbid owner targeting seat 1 (extension.js:4741)"
    );
    drain(&mut e_m0);

    // Mode 1 branch: target grants txtanbei_effect1 (seals non-locked skills) & owner unlimited Sha vs seat 1!
    let mut e_m1 = e;
    e_m1.debug_grant(1, "txleizhou"); // non-locked skill sealed by effect1
    let _ = answer(&mut e_m1, &c_tb_m, vec![ChoiceOption::Tag(1)]);
    drain(&mut e_m1);
    assert!(e_m1.debug_has_skill(1, "txtanbei_effect1"));
    let s1 = place_kind(&mut e_m1, 0, CardKind::Sha);
    let s2 = place_kind(&mut e_m1, 0, CardKind::Sha);
    let _ = play_card(&mut e_m1, 0, s1, vec![1]);
    drain(&mut e_m1);
    assert!(
        matches!(play_card(&mut e_m1, 0, s2, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "txtanbei_effect3 allows unlimited Sha against target"
    );
    drain(&mut e_m1);

    // 2. ljxuechi, txzhuxin, txyicheng, txnianrui:
    let mut e_lc = start_taixu(0x210A, 3);
    e_lc.debug_grant(0, "ljxuechi");
    e_lc.debug_grant(0, "txzhuxin");
    e_lc.debug_grant(0, "txnianrui");
    e_lc.debug_grant(1, "txyicheng");
    // ljxuechi with 0 hand (<3): seat 2 Tao recover is blocked and seat 0 draws 1!
    e_lc.debug_set_hp(2, 4);
    e_lc.debug_set_turn(2, Phase::Play, 1);
    let tao2 = place_kind(&mut e_lc, 2, CardKind::Tao);
    let _ = play_card(&mut e_lc, 2, tao2, Vec::new());
    drain(&mut e_lc);
    assert_eq!(e_lc.debug_hp(2), 4, "ljxuechi (<3 hand) blocks recover");
    assert_eq!(e_lc.debug_hand(0).len(), 1, "ljxuechi owner draws 1");
    // Give seat 0 >= 3 hand cards -> Tao recover now succeeds!
    let _ = place_kind(&mut e_lc, 0, CardKind::Sha);
    let _ = place_kind(&mut e_lc, 0, CardKind::Shan);
    let tao2b = place_kind(&mut e_lc, 2, CardKind::Tao);
    let _ = play_card(&mut e_lc, 2, tao2b, Vec::new());
    drain(&mut e_lc);
    assert_eq!(e_lc.debug_hp(2), 5, "ljxuechi (>=3 hand) allows recover");

    // txnianrui: Draw phase draws 2 + 2 = 4!
    let h0_before = e_lc.debug_hand(0).len();
    e_lc.debug_set_turn(0, Phase::Judge, 2);
    let _ = next_phase(&mut e_lc); // -> Draw
    drain(&mut e_lc);
    assert_eq!(e_lc.debug_hand(0).len(), h0_before + 4, "txnianrui adds +2 draw");

    // txyicheng: when Sha is used -> confirm draws 1 then discards 1 per target!
    e_lc.debug_set_turn(0, Phase::Play, 2);
    let sha_yc = place_kind(&mut e_lc, 0, CardKind::Sha);
    let _ = play_card(&mut e_lc, 0, sha_yc, vec![1]);
    let c_yc = expect_pending(&e_lc, "txyicheng:confirm");
    let _ = answer(&mut e_lc, &c_yc, vec![ChoiceOption::Confirm]);
    let c_yc_dis = expect_pending(&e_lc, "txyicheng:discard_card");
    let d1 = e_lc.debug_hand(1)[0];
    let _ = answer(&mut e_lc, &c_yc_dis, vec![ChoiceOption::Card(d1)]);
    drain(&mut e_lc);

    // txzhuxin: dying caused by owner directly kills target even if target holds Tao!
    e_lc.debug_set_hp(2, 1);
    let _ = place_kind(&mut e_lc, 2, CardKind::Tao);
    let sha_zx = place_kind(&mut e_lc, 0, CardKind::Sha);
    e_lc.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e_lc, 0, sha_zx, vec![2]);
    // Auto-drain if txyicheng confirm prompts -> pass txyicheng
    drain(&mut e_lc);
    assert!(!e_lc.debug_alive(2), "txzhuxin directly kills dying target");
}

use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension.js:4801-4950` (`txmengtai`, `txshouyi`, `txrenxing`, `txyinli`, `txzhulian`, `txkeji`, `txkeji_add`).
#[test]
fn l11_b2_skills_e_mengtai_shouyi_renxing_yinli_zhulian_keji() {
    // 1. txmengtai: Prepare sets no_draw & skip_discard; when Bing skips Draw -> End draws 3; when Le skips Play -> Discard skipped!
    let mut e = start_taixu(0x2201, 2);
    e.debug_grant(0, "txmengtai");
    let _ = e.debug_place_judge(0, "bing");
    let _ = top_deck_suit(&mut e, Suit::Spade); // non-Club -> Bing skips Draw!
    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e); // TurnBegin(0) -> Prepare -> sets no_draw=1 & skip_discard=true
    let _ = next_phase(&mut e); // Prepare -> Judge (Bing skips Draw!)
    let _ = next_phase(&mut e); // Judge -> Play (Draw skipped so no_draw stays 1; Play sets skip_discard=false)
    let _ = next_phase(&mut e); // Play -> Discard
    let _ = next_phase(&mut e); // Discard -> End (no_draw == 1 -> draws 3!)
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 3, "txmengtai draws 3 at End after Draw was skipped");

    // 2. txshouyi: unlimited range for cards (Snatch at distance 2 in 4-player game!)
    let mut e_sy = start_taixu(0x2202, 4);
    let _p2_loot = place_kind(&mut e_sy, 2, CardKind::Tao);
    let sn_far = place_kind(&mut e_sy, 0, CardKind::Snatch);
    e_sy.debug_set_turn(0, Phase::Play, 1);
    assert!(
        matches!(play_card(&mut e_sy, 0, sn_far, vec![2]), CommandOutcome::Rejected(_)),
        "without txshouyi Snatch at distance 2 must be rejected"
    );
    e_sy.debug_grant(0, "txshouyi");
    assert!(
        matches!(play_card(&mut e_sy, 0, sn_far, vec![2]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "txshouyi grants unlimited range for Snatch at distance 2"
    );
    drain(&mut e_sy);

    // 3. txrenxing: damage/recover outside own turn draws 1 for owner!
    let mut e_rx = start_taixu(0x2203, 2);
    e_rx.debug_grant(0, "txrenxing");
    e_rx.debug_set_turn(1, Phase::Play, 1);
    e_rx.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain(&mut e_rx);
    assert_eq!(e_rx.debug_hand(0).len(), 1, "txrenxing drew 1 on off-turn damage");

    // 4. txyinli: real Discard phase discarding Equips in another's turn recovers equips at TurnEnded!
    let mut e_yl = start_taixu(0x2204, 2);
    e_yl.debug_grant(0, "txyinli");
    e_yl.debug_set_max_hp(1, 1);
    e_yl.debug_set_hp(1, 1);
    let eq1 = place_kind(&mut e_yl, 1, CardKind::Zhuge);
    let eq2 = place_kind(&mut e_yl, 1, CardKind::Renwang);
    let sha_keep = place_kind(&mut e_yl, 1, CardKind::Sha);
    let _ = sha_keep;
    e_yl.debug_set_turn(1, Phase::Play, 1);
    let _ = next_phase(&mut e_yl); // Play -> Discard: 3 cards > hp 1 -> prompts discard 2
    let c_dis1 = expect_pending(&e_yl, "discard_to_limit");
    let _ = answer(&mut e_yl, &c_dis1, vec![ChoiceOption::Card(eq1), ChoiceOption::Card(eq2)]);
    let _ = next_phase(&mut e_yl); // Discard -> End
    let _ = next_phase(&mut e_yl); // End -> TurnEnded(1) triggers txyinli:choose_cards!
    let c_yl = expect_pending(&e_yl, "txyinli:choose_cards");
    let e_yl2 = save_restore_and_compare_pending(
        &e_yl,
        &c_yl,
        vec![ChoiceOption::Card(eq1), ChoiceOption::Card(eq2)],
    );
    assert!(e_yl2.debug_hand(0).contains(&eq1));
    assert!(e_yl2.debug_hand(0).contains(&eq2));

    // 5. txzhulian: TurnBegin links owner; Damage links alive others within attack range!
    let mut e_zl = start_taixu(0x2205, 4);
    e_zl.debug_grant(0, "txzhulian");
    e_zl.debug_set_turn(3, Phase::End, 1);
    let _ = next_phase(&mut e_zl); // TurnBegin(0) -> links seat 0!
    drain(&mut e_zl);
    assert!(e_zl.debug_linked(0), "txzhulian links owner at TurnBegin");
    e_zl.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal); // Damage triggers linking seat 1 & seat 3 (range 1), skipping seat 2 (dist 2)!
    drain(&mut e_zl);
    assert!(e_zl.debug_linked(1) && e_zl.debug_linked(3));
    assert!(!e_zl.debug_linked(2), "seat 2 outside attack range must stay unlinked");

    // 6. txkeji & txkeji_add: Discard begin grants txkeji_add (+unlimited hand / effect) when <=1 Play color used!
    let mut e_kj = start_taixu(0x2206, 2);
    e_kj.debug_grant(0, "txkeji");
    e_kj.debug_set_turn(0, Phase::Play, 1);
    let _ = next_phase(&mut e_kj); // -> Discard begin -> grants txkeji_add
    drain(&mut e_kj);
    assert!(e_kj.debug_has_skill(0, "txkeji_add"));
}

/// Upstream: `extension.js:4951-5220` (`txnangce`, `txduanzui`, `txyingyang`, `mdqianxi7`, `mdqianxi72`, `mdqianxi`).
#[test]
fn l11_b2_skills_e_nangce_duanzui_yingyang_qianxi() {
    // 1. txnangce: using a trick prompts confirm -> current turn player draws 1; AI only confirms on self/allies, declines on enemies!
    let mut e = start_versus(0x2207, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txnangce");
    e.debug_set_turn(2, Phase::Play, 1); // Enemy 2 turn! Seat 0 plays Nullify on enemy's Dismantle -> AI Pass!
    let _ = place_kind(&mut e, 1, CardKind::Sha);
    let gh2 = place_kind(&mut e, 2, CardKind::Dismantle);
    let wx0 = place_kind(&mut e, 0, CardKind::Nullify);
    let _ = play_card(&mut e, 2, gh2, vec![1]);
    let c_wx = expect_pending(&e, "nullify_window");
    let _ = answer(&mut e, &c_wx, vec![ChoiceOption::Card(wx0)]);
    let c_nc_e = expect_pending(&e, "txnangce:confirm");
    assert_eq!(ai_plan(&e, 0), GameAction::Answer {
        choice_id: c_nc_e.id,
        selection: vec![ChoiceOption::Pass], // enemy turn -> do NOT draw for enemy!
    });
    let _ = answer(&mut e, &c_nc_e, vec![ChoiceOption::Pass]);
    drain(&mut e);
    // Own turn Wuzhong -> AI Confirm -> draws +1 extra!
    e.debug_set_turn(0, Phase::Play, 2);
    let wz0 = e.debug_move_named_card_to_hand(0, "wuzhong");
    let res_wz = play_card(&mut e, 0, wz0, Vec::new());
    assert!(
        !matches!(res_wz, CommandOutcome::Rejected(_)),
        "wuzhong rejected: {res_wz:?}"
    );
    let c_nc = expect_pending(&e, "txnangce:confirm");
    assert_eq!(ai_plan(&e, 0), GameAction::Answer {
        choice_id: c_nc.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let _ = answer(&mut e, &c_nc, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 3, "txnangce (+1) + Wuzhong (+2) = 3");

    // 2. txduanzui: TurnEnded deals (3 - hand) Fire dmg to others with 1..=2 hand; AI declines when ally would be burned!
    let mut e_dz = start_versus(0x2208, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_dz.debug_grant(0, "txduanzui");
    let _ = place_kind(&mut e_dz, 1, CardKind::Sha); // 1 hand (<3 -> 2 Fire dmg!)
    e_dz.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_dz);
    let c_dz = expect_pending(&e_dz, "txduanzui:confirm");
    assert_eq!(ai_plan(&e_dz, 0), GameAction::Answer {
        choice_id: c_dz.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let e_dz2 = save_restore_and_compare_pending(&e_dz, &c_dz, vec![ChoiceOption::Confirm]);
    assert_eq!(e_dz2.debug_hp(1), 4, "txduanzui dealt 3 - 1 = 2 Fire dmg");

    // 3. txyingyang: real ChooseHandCompare Pindian (qiaoshui) + +3/-3/cancel!
    let mut e_yy = start_taixu(0x2209, 2);
    e_yy.debug_grant(0, "qiaoshui");
    e_yy.debug_grant(0, "txyingyang");
    let _ = place_kind_rank(&mut e_yy, 0, CardKind::Sha, 5); // 5 vs 7 normally loses; with +3 = 8 > 7 wins!
    let _ = place_kind_rank(&mut e_yy, 1, CardKind::Sha, 7);
    e_yy.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_yy); // -> Play triggers qiaoshui!
    let c_qs_opt = expect_pending(&e_yy, "trigger_confirm");
    let _ = answer(&mut e_yy, &c_qs_opt, vec![ChoiceOption::Confirm]);
    let c_qs_t = expect_pending(&e_yy, "rvm_choice");
    let _ = answer(&mut e_yy, &c_qs_t, vec![ChoiceOption::Player(1)]);
    // Single-card hands auto-pick into Pindian -> txyingyang:choose_mode (+3 makes 5+3=8 > 7 win!).
    let c_yy = expect_pending(&e_yy, "txyingyang:choose_mode");
    assert_eq!(ai_plan(&e_yy, 0), GameAction::Answer {
        choice_id: c_yy.id,
        selection: vec![ChoiceOption::Tag(0)],
    });
    let e_yy_done = save_restore_and_compare_pending(&e_yy, &c_yy, vec![ChoiceOption::Tag(0)]);
    assert_eq!(
        flag(&e_yy_done, 0, "compare_win_turn"),
        1,
        "txyingyang +3 turns rank 5 vs 7 into a Pindian win"
    );

    // 4. mdqianxi7, mdqianxi72, mdqianxi:
    let mut e_qx = start_taixu(0x220A, 2);
    e_qx.debug_grant(0, "mdqianxi7");
    let red_d = place_kind_suit(&mut e_qx, 0, CardKind::Sha, Suit::Heart);
    let red_play = place_kind_suit(&mut e_qx, 1, CardKind::Sha, Suit::Diamond);
    let blk_play = place_kind_suit(&mut e_qx, 1, CardKind::Sha, Suit::Spade);
    e_qx.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_qx); // Prepare(0)
    let c_q7 = expect_pending(&e_qx, "mdqianxi7:confirm");
    let _ = answer(&mut e_qx, &c_q7, vec![ChoiceOption::Confirm]);
    let c_qd = expect_pending(&e_qx, "mdqianxi7:discard_card");
    let _ = answer(&mut e_qx, &c_qd, vec![ChoiceOption::Card(red_d)]);
    let c_qt = expect_pending(&e_qx, "mdqianxi7:choose_target");
    let _ = answer(&mut e_qx, &c_qt, vec![ChoiceOption::Player(1)]);
    drain(&mut e_qx);
    assert!(e_qx.debug_has_skill(1, "mdqianxi72"));
    e_qx.debug_set_turn(1, Phase::Play, 2);
    assert!(
        matches!(play_card(&mut e_qx, 1, red_play, vec![0]), CommandOutcome::Rejected(_)),
        "mdqianxi72 must ban playing Red cards"
    );
    assert!(
        matches!(play_card(&mut e_qx, 1, blk_play, vec![0]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "mdqianxi72 allows Black cards"
    );
    drain(&mut e_qx);

    // Permanent mdqianxi (judge-based):
    let mut e_qx2 = start_taixu(0x220B, 2);
    e_qx2.debug_grant(0, "mdqianxi");
    let _ = top_deck_suit(&mut e_qx2, Suit::Spade); // Black
    e_qx2.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_qx2); // Prepare(0)
    let c_qc = expect_pending(&e_qx2, "mdqianxi:confirm");
    let _ = answer(&mut e_qx2, &c_qc, vec![ChoiceOption::Confirm]);
    let c_qt2 = expect_pending(&e_qx2, "mdqianxi:choose_target");
    let _ = answer(&mut e_qx2, &c_qt2, vec![ChoiceOption::Player(1)]);
    drain(&mut e_qx2);
    let blk1 = place_kind_suit(&mut e_qx2, 1, CardKind::Sha, Suit::Club);
    e_qx2.debug_set_turn(1, Phase::Play, 2);
    assert!(
        matches!(play_card(&mut e_qx2, 1, blk1, vec![0]), CommandOutcome::Rejected(_)),
        "mdqianxi Black judge permanently bans Black cards on target"
    );
}

/// Upstream: `extension.js:5221-5501` (`zylongdan`, `zglkongcheng`, `txjuxiang`, `txdaoji`).
#[test]
fn l11_b2_skills_e_longdan_kongcheng_juxiang_daoji() {
    // 1. zylongdan: Shan as Sha (dodged -> damage other target + shouyue draws 1!); Sha as Shan (recovers other + shouyue draws 1!)
    let mut e = start_versus(0x220C, &[&[0, 1], &[2, 3]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "zylongdan");
    e.state_core_mut().vm_aux.flags.insert((1, "shouyue_active".into()), 1); // active shouyue!
    e.debug_set_hp(1, 4);
    let shan0 = place_kind(&mut e, 0, CardKind::Shan);
    let shan3 = place_kind(&mut e, 3, CardKind::Shan);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut e, 0, shan0, vec![3]); // Shan used as Sha against distance-1 enemy 3!
    let c_sh = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &c_sh, vec![ChoiceOption::Card(shan3)]);
    let c_dmg = expect_pending(&e, "zylongdan:damage_target");
    assert_eq!(ai_plan(&e, 0), GameAction::Answer {
        choice_id: c_dmg.id,
        selection: vec![ChoiceOption::Player(2)], // enemy 2 over ally 1 (3 excluded as Sha target)!
    });
    let _ = save_restore_and_compare_pending(&e, &c_dmg, vec![ChoiceOption::Player(2)]);

    // 2. zglkongcheng & txjuxiang:
    // zglkongcheng blocks Sha/Duel when hand == 0, intercepts external `give_hand_cards` (e.g. oldfuman) to pile until Draw begin,
    // while txjuxiang (`giver = player`, not `source`) places Nanman directly into target's hand (NOT zglkongcheng pile)!
    let mut e_zk = start_taixu(0x220D, 2);
    e_zk.debug_grant(0, "txjuxiang");
    e_zk.debug_grant(0, "oldfuman");
    e_zk.debug_grant(1, "zglkongcheng");
    e_zk.debug_set_turn(0, Phase::Play, 1);
    let sha_0 = place_kind(&mut e_zk, 0, CardKind::Sha);
    assert!(
        matches!(play_card(&mut e_zk, 0, sha_0, vec![1]), CommandOutcome::Rejected(_)),
        "zglkongcheng (0 hand) cannot be targeted by Sha"
    );
    // oldfuman gives Sha from seat 0 to empty-hand seat 1 -> intercepted into seat 1's l11_zglkongcheng pile!
    let _ = activate_skill(&mut e_zk, 0, "oldfuman");
    let c_of_c = expect_pending(&e_zk, "oldfuman:choose_card");
    let _ = answer(&mut e_zk, &c_of_c, vec![ChoiceOption::Card(sha_0)]);
    let c_of_t = expect_pending(&e_zk, "oldfuman:choose_target");
    let _ = answer(&mut e_zk, &c_of_t, vec![ChoiceOption::Player(1)]);
    drain(&mut e_zk);
    assert!(e_zk.debug_hand(1).is_empty(), "off-turn external give_hand_cards goes to zglkongcheng pile, hand stays empty");
    assert!(
        e_zk.state_core()
            .vm_aux
            .piles
            .get(&(1, "l11_zglkongcheng".to_string()))
            .is_some_and(|p| p.len() == 1)
    );
    // txjuxiang: Phase::End(0) spawns SavageAttack from deck/discard to target 1 -> lands directly in seat 1 hand (NOT zglkongcheng pile)!
    e_zk.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut e_zk); // Phase::End(0) triggers txjuxiang
    let c_jx = expect_pending(&e_zk, "txjuxiang:choose_target");
    let _ = answer(&mut e_zk, &c_jx, vec![ChoiceOption::Player(1)]);
    drain(&mut e_zk);
    assert_eq!(e_zk.debug_hand(1).len(), 1, "txjuxiang (.giver = player) enters target hand directly, bypassing zglkongcheng_gain");
    // Seat 1 Draw begin retrieves zglkongcheng pile (1) + draws normally (2) => 1 + 1 + 2 = 4!
    e_zk.debug_set_turn(1, Phase::Judge, 2);
    let _ = next_phase(&mut e_zk); // Judge -> Draw
    drain(&mut e_zk);
    assert_eq!(e_zk.debug_hand(1).len(), 4, "zglkongcheng retrieves pile (1) + txjuxiang (1) + Draw (2) = 4");

    // 3. txdaoji: cancel refunds once-per-turn; steal Basic draws 1; steal Equip equips & deals 1 dmg!
    let mut e_dj = start_taixu(0x220E, 2);
    e_dj.debug_grant(0, "txdaoji");
    let cost_c = place_kind(&mut e_dj, 0, CardKind::Wuzhong);
    let loot_eq = equip_kind(&mut e_dj, 1, CardKind::Zhuge);
    e_dj.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut e_dj, 0, "txdaoji");
    let c_d1 = expect_pending(&e_dj, "txdaoji:discard_card");
    let _ = answer(&mut e_dj, &c_d1, Vec::new()); // cancel!
    drain(&mut e_dj);
    // Can re-activate after cancel:
    let _ = activate_skill(&mut e_dj, 0, "txdaoji");
    let c_d2 = expect_pending(&e_dj, "txdaoji:discard_card");
    let _ = answer(&mut e_dj, &c_d2, vec![ChoiceOption::Card(cost_c)]);
    let c_d_t = expect_pending(&e_dj, "txdaoji:choose_target");
    let _ = answer(&mut e_dj, &c_d_t, vec![ChoiceOption::Player(1)]);
    let c_d_h = expect_pending(&e_dj, "txdaoji:choose_he");
    let e_dj2 = save_restore_and_compare_pending(&e_dj, &c_d_h, vec![ChoiceOption::Card(loot_eq)]);
    assert!(e_dj2.debug_equips(0).contains(&loot_eq), "txdaoji equips stolen equip on owner");
    assert_eq!(e_dj2.debug_hp(1), 5, "txdaoji deals 1 dmg after equip steal");
}

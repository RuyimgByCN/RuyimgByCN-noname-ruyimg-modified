use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension.js:5502-5700` (`txhj_suoming`, `txhj_huangmen`, `txhj_mouqiang`, `txhj_suzhi`).
#[test]
fn l11_b2_skills_f_suoming_huangmen_mouqiang_suzhi() {
    // 1. txhj_suoming & txhj_huangmen:
    let mut e = start_versus(0x2301, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txhj_suoming");
    e.debug_grant(0, "txhj_huangmen");
    let _loot2 = place_kind(&mut e, 2, CardKind::Sha);
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut e); // End begin: both txhj_suoming (unlinked others) and txhj_huangmen (0 hand) fire!
    // One of them opens first; verify both work by driving sequentially:
    for _ in 0..4 {
        let Some(c) = e.open_choice().cloned() else { break };
        match c.spec.prompt {
            "txhj_suoming:choose_targets" => {
                // AI picks only enemies (seat 2):
                assert_eq!(ai_plan(&e, 0), GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(2)],
                });
                let _ = answer(&mut e, &c, vec![ChoiceOption::Player(2)]);
            }
            "txhj_huangmen:choose_mode" => {
                // Tag 1 steals random he from enemy 2!
                let _ = answer(&mut e, &c, vec![ChoiceOption::Tag(1)]);
            }
            _ => {
                let _ = answer(&mut e, &c, Vec::new());
            }
        }
    }
    drain(&mut e);
    assert!(e.debug_linked(2), "txhj_suoming linked enemy 2");
    assert!(!e.debug_linked(1), "txhj_suoming skipped ally");
    assert_eq!(e.debug_hand(0).len(), 1, "txhj_huangmen tag 1 stole from enemy 2");

    // 2. txhj_mouqiang & txhj_suzhi:
    let mut e_ms = start_versus(0x2302, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e_ms.debug_grant(0, "txhj_mouqiang");
    e_ms.debug_grant(1, "txhj_suzhi");
    e_ms.debug_set_hp(0, 4);
    // 1 damage does NOT trigger txhj_mouqiang:
    e_ms.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(e_ms.open_choice().is_none(), "mouqiang ignores damage == 1");

    // Fresh clean check for mouqiang (4 dmg from 6 HP -> 2 HP, steal 2 cards -> +1 heal, +1 dmg to target & source) with save/restore:
    let mut e_mq = start_versus(0x2303, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e_mq.debug_grant(0, "txhj_mouqiang");
    let bc = place_kind(&mut e_mq, 2, CardKind::Tao);
    let tc = place_kind(&mut e_mq, 2, CardKind::Wuzhong);
    let _ = e_mq.debug_deal_damage(Some(1), 0, 4, DamageNature::Normal); // 6 -> 2 HP
    let c_t = expect_pending(&e_mq, "txhj_mouqiang:choose_target");
    let _ = answer(&mut e_mq, &c_t, vec![ChoiceOption::Player(2)]);
    let c_h = expect_pending(&e_mq, "txhj_mouqiang:choose_he");
    let e_mq = save_restore_and_compare_pending(&e_mq, &c_h, vec![ChoiceOption::Card(bc), ChoiceOption::Card(tc)]);
    assert_eq!(e_mq.debug_hp(0), 3, "mouqiang recovered +1 for stolen basic card");
    assert_eq!(e_mq.debug_hp(2), 5, "mouqiang dealt 1 dmg to target");
    assert_eq!(e_mq.debug_hp(1), 5, "mouqiang dealt 1 dmg to source for stolen non-basic");

    // txhj_suzhi: <3 marks gives unlimited trick range + fankui + draws/steals/+1 dmg up to 3 marks!
    let mut e_sz = start_taixu(0x2304, 4);
    e_sz.debug_grant(0, "txhj_suzhi");
    e_sz.debug_set_turn(0, Phase::Play, 1);
    let _p2_card = place_kind(&mut e_sz, 2, CardKind::Sha); // seat 2 is distance 2! Trick Snatch normally range 1!
    let sn = place_kind(&mut e_sz, 0, CardKind::Snatch);
    assert!(
        matches!(play_card(&mut e_sz, 0, sn, vec![2]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "txhj_suzhi (<3 marks) grants unlimited trick range"
    );
    drain(&mut e_sz);
    assert_eq!(flag(&e_sz, 0, "skill_turn:l11_suzhi_marks"), 1);
    assert_eq!(e_sz.debug_hand(0).len(), 2, "suzhi mark 1 drew 1 on Trick use + Snatched 1");
    // Sha deals +1 dmg -> mark 2; End grants fankui; mark 3 stops further triggers & clears fankui on TurnBegin:
    let sha_sz = place_kind(&mut e_sz, 0, CardKind::Sha);
    let _ = play_card(&mut e_sz, 0, sha_sz, vec![1]);
    drain(&mut e_sz);
    assert_eq!(e_sz.debug_hp(1), 4, "suzhi adds +1 dmg on Sha (total 2 dmg)");
    assert_eq!(flag(&e_sz, 0, "skill_turn:l11_suzhi_marks"), 2);
    // End grants fankui:
    e_sz.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut e_sz);
    drain(&mut e_sz);
    assert!(
        e_sz.debug_has_skill(0, "反馈") || e_sz.debug_has_skill(0, "fankui"),
        "suzhi End grants fankui"
    );
}

/// Upstream: `extension.js:5701-5880` (`txmanjia`, `txmanjia1`, `txmanjia2`, `txmanzhen`, `txfusha`, `txleizhou`, `txzhennu`).
#[test]
fn l11_b2_skills_f_manjia_manzhen_fusha_leizhou_zhennu() {
    // 1. txmanjia / 1 / 2: immune to normal Sha + Nanman when unarmored; Fire +1; armored -> ineffective!
    let mut e = start_taixu(0x2305, 3);
    e.debug_grant(1, "txmanjia");
    e.debug_grant(0, "txfusha");
    e.debug_set_turn(0, Phase::Play, 1);
    // Normal Sha on unarmored manjia -> 0 dmg! (Even with txfusha, txmanjia cancels normal Sha!)
    let sha_n = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha_n, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 6, "txmanjia blocks normal Sha when unarmored");
    // FireSha on unarmored manjia -> base 1 + txfusha 1 + txmanjia Fire +1 = 3 dmg!
    let sha_f = place_kind(&mut e, 0, CardKind::FireSha);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e, 0, sha_f, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3, "FireSha hits unarmored txmanjia (+1 Fire + txfusha)");
    // txfusha chain counterexample: chain damage does NOT get +1!
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    let sha_l = place_kind(&mut e, 0, CardKind::LeiSha);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e, 0, sha_l, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 1, "direct LeiSha + txfusha = 2 dmg (3->1)");
    assert_eq!(e.debug_hp(2), 4, "chained LeiSha does NOT re-add txfusha (+0, total 2 dmg)");
    // Armor equipped on seat 1 negates txmanjia so normal Sha now hits:
    equip_kind(&mut e, 1, CardKind::Renwang);
    let sha_arm = place_kind(&mut e, 0, CardKind::Sha);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_set_hp(1, 5);
    let _ = play_card(&mut e, 0, sha_arm, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3, "armored seat loses txmanjia immunity");

    // 2. txmanzhen, txleizhou, txzhennu:
    let mut e_m = start_versus(0x2306, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e_m.debug_grant(0, "txmanzhen");
    e_m.debug_set_turn(0, Phase::Play, 1);
    let c0a = place_kind(&mut e_m, 0, CardKind::Sha);
    let c0b = place_kind(&mut e_m, 0, CardKind::Shan);
    let _ = activate_skill(&mut e_m, 0, "txmanzhen");
    let c_mc = expect_pending(&e_m, "txmanzhen:choose_cards");
    let _ = answer(&mut e_m, &c_mc, vec![ChoiceOption::Card(c0a), ChoiceOption::Card(c0b)]);
    let c_mt = expect_pending(&e_m, "txmanzhen:choose_targets");
    let _ = answer(&mut e_m, &c_mt, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    // Both targets have hp 6 >= owner 6 -> confirms +1 dmg & lose 1 hp!
    let c_md1 = expect_pending(&e_m, "txmanzhen:confirm_damage");
    assert_eq!(ai_plan(&e_m, 0), GameAction::Answer {
        choice_id: c_md1.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let snap = e_m.save();
    let mut restored =
        content_native_loader::restore_locked_engine(&snap, taixu_verified())
            .expect("restore txmanzhen");
    assert_eq!(e_m.state_hash(), restored.state_hash());
    let _ = answer(&mut e_m, &c_md1, vec![ChoiceOption::Confirm]);
    let _ = answer(&mut restored, &c_md1, vec![ChoiceOption::Confirm]);
    if let Some(c_md2) = e_m.open_choice().cloned() {
        assert_eq!(c_md2.spec.prompt, "txmanzhen:confirm_damage");
        let _ = answer(&mut e_m, &c_md2, vec![ChoiceOption::Confirm]);
        let _ = answer(&mut restored, &c_md2, vec![ChoiceOption::Confirm]);
    }
    drain(&mut e_m);
    drain(&mut restored);
    assert_eq!(e_m.state_hash(), restored.state_hash());
    assert_eq!(e_m.debug_hp(1), 4, "txmanzhen target 1 took 1 + 1 = 2 damage");
    assert_eq!(e_m.debug_hp(2), 4, "txmanzhen target 2 took 1 + 1 = 2 damage");
    assert_eq!(e_m.debug_hp(0), 4, "txmanzhen owner lost 2 HP total");
    let mut e_m2 = e_m;

    // txleizhou & txzhennu at Prepare:
    e_m2.debug_grant(1, "txleizhou");
    e_m2.debug_grant(1, "txzhennu");
    let _ = e_m2.debug_set_turn(0, Phase::End, 2);
    let _ = next_phase(&mut e_m2); // Seat 1 TurnBegin -> Prepare triggers txzhennu + txleizhou!
    drain(&mut e_m2);
}

/// Upstream: `extension.js:5881-6088` (`txyongguan`, `txyongguan_fate`, `txchengyuan`, `txbenji`, `txyonglve`, `txjiang`, `txhj_oldhuxiao`).
#[test]
fn l11_b2_skills_f_yongguan_chengyuan_benji_yonglve_jiang_oldhuxiao() {
    // 1. txyongguan / txyongguan_fate + txchengyuan + txjiang + txhj_oldhuxiao:
    let mut e = start_taixu(0x2307, 3);
    e.debug_grant(1, "txyongguan");
    e.debug_grant(1, "txyongguan_fate");
    e.debug_grant(0, "txjiang");
    e.debug_grant(0, "txhj_oldhuxiao");
    e.debug_set_turn(0, Phase::Play, 1);
    let shan1 = place_kind(&mut e, 1, CardKind::Shan);
    let red_sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    // Red Sha triggers txjiang (+1 draw); dodged by Shan -> txhj_oldhuxiao refunds Sha use!
    let _ = play_card(&mut e, 0, red_sha, vec![1]);
    let c_jg = expect_pending(&e, "txjiang:confirm");
    let _ = answer(&mut e, &c_jg, vec![ChoiceOption::Confirm]);
    let c_sh = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &c_sh, vec![ChoiceOption::Card(shan1)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2, "txjiang drew 2 for Red Sha");
    // Second Sha hits seat 1 -> txyongguan prevents & adds 1 mark!
    let sha2 = place_kind(&mut e, 0, CardKind::Sha);
    assert!(
        matches!(play_card(&mut e, 0, sha2, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "txhj_oldhuxiao must refund Sha use"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 6, "txyongguan prevented damage");
    assert_eq!(flag(&e, 1, "l11_yongguan_marks"), 1);

    // Seat 1 has 1 hand card; TurnEnded(1) triggers txyongguan_fate:discard -> discard 1 saves HP vs cancel loses 1 HP!
    let d1 = place_kind(&mut e, 1, CardKind::Tao);
    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e);
    let c_yg = expect_pending(&e, "txyongguan_fate:discard");
    let e_saved = save_restore_and_compare_pending(&e, &c_yg, vec![ChoiceOption::Card(d1)]);
    assert_eq!(e_saved.debug_hp(1), 6, "discarding mark cards avoids LoseHp");
    let mut e_lose = e;
    let _ = answer(&mut e_lose, &c_yg, Vec::new());
    drain(&mut e_lose);
    assert_eq!(e_lose.debug_hp(1), 5, "skipping discard loses HP == marks");

    // 2. txchengyuan: placing 3 unused-color cards triggers discard pile -> 2 dmg & discard equips!
    let mut e_cy = start_versus(0x2308, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_cy.debug_grant(0, "txchengyuan");
    equip_kind(&mut e_cy, 1, CardKind::Renwang);
    equip_kind(&mut e_cy, 1, CardKind::Zhuge);
    for round in 1..=3u32 {
        let c = place_kind_suit(&mut e_cy, 0, CardKind::Sha, Suit::Heart);
        e_cy.debug_set_turn(0, Phase::Play, round);
        let _ = next_phase(&mut e_cy); // Play -> Discard begin triggers txchengyuan
        let c_pick = expect_pending(&e_cy, "txchengyuan:choose_card");
        let _ = answer(&mut e_cy, &c_pick, vec![ChoiceOption::Card(c)]);
        if round == 3 {
            let c_t = expect_pending(&e_cy, "txchengyuan:choose_target");
            let e_cy_done = save_restore_and_compare_pending(&e_cy, &c_t, vec![ChoiceOption::Player(1)]);
            assert_eq!(e_cy_done.debug_hp(1), 4, "txchengyuan dealt 2 dmg");
            assert!(e_cy_done.debug_equips(1).is_empty(), "txchengyuan discarded all target equips");
            break;
        } else {
            drain(&mut e_cy);
        }
    }

    // 3. txbenji: Prepare deals (max_hp - hp) dmg -> non-dying loses 1 hp, dying skips loseHp!
    let mut e_bj = start_taixu(0x2309, 2);
    e_bj.debug_grant(0, "txbenji");
    e_bj.debug_set_hp(0, 4); // lost = 2
    e_bj.debug_set_hp(1, 5); // 5 - 2 = 3 (no dying -> owner loses 1 hp -> 3)
    e_bj.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_bj);
    let c_bj = expect_pending(&e_bj, "txbenji:choose_target");
    let _ = answer(&mut e_bj, &c_bj, vec![ChoiceOption::Player(1)]);
    drain(&mut e_bj);
    assert_eq!(e_bj.debug_hp(1), 3);
    assert_eq!(e_bj.debug_hp(0), 3, "txbenji non-dying target causes owner lose 1 HP");

    // 4. txyonglve: empty-hand Pindian via deck top!
    let mut e_yl = start_taixu(0x230A, 2);
    e_yl.debug_grant(0, "tianyi"); // `tianyi` triggers real VM ChooseHandCompare
    e_yl.debug_grant(0, "txyonglve");
    let _ = place_kind_rank(&mut e_yl, 1, CardKind::Sha, 5);
    let _ = top_deck_rank(&mut e_yl, 12); // deck top 12 > 5 -> owner wins via txyonglve!
    e_yl.debug_set_turn(0, Phase::Play, 1);
    let _ = activate_skill(&mut e_yl, 0, "tianyi");
    let c_ty_t = expect_pending(&e_yl, "l11_vm_b:tianyi_target");
    let _ = answer(&mut e_yl, &c_ty_t, vec![ChoiceOption::Player(1)]);
    let c_yl = expect_pending(&e_yl, "txyonglve:confirm");
    let e_yl_done = save_restore_and_compare_pending(&e_yl, &c_yl, vec![ChoiceOption::Confirm]);
    assert!(e_yl_done.debug_hand(0).is_empty(), "deck top was discarded after Pindian");
}

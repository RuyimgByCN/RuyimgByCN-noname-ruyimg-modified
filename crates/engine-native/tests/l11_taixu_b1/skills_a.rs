use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension.js:3207-3365` (`tx_mojian`, `tx_yushou`, `tx_moyany`, `tx_danshu`, `txyirang`).
#[test]
fn l11_skills_a_mojian_yushou_moyany_danshu_yirang() {
    // 1. tx_mojian: Play phase begin prompts optional confirm -> emits ArcheryAttack vs enemies (allies untouched)
    let mut e_mj = start_versus(0x1401, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_mj.debug_grant(0, "tx_mojian");
    e_mj.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_mj); // Draw -> Play begin triggers tx_mojian
    let c_mj = expect_pending(&e_mj, "tx_mojian:confirm");
    let _ = answer(&mut e_mj, &c_mj, vec![ChoiceOption::Confirm]);
    drain(&mut e_mj);
    assert_eq!(e_mj.debug_hp(1), 6, "ally must not be targeted by tx_mojian");
    assert_eq!(e_mj.debug_hp(2), 5, "enemy takes ArcheryAttack");

    // 2. tx_yushou: Play phase begin prompts optional confirm -> emits SavageAttack vs enemies (allies untouched)
    let mut e_ys = start_versus(0x1402, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_ys.debug_grant(0, "tx_yushou");
    e_ys.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_ys);
    let c_ys = expect_pending(&e_ys, "tx_yushou:confirm");
    let _ = answer(&mut e_ys, &c_ys, vec![ChoiceOption::Confirm]);
    drain(&mut e_ys);
    assert_eq!(e_ys.debug_hp(1), 6);
    assert_eq!(e_ys.debug_hp(2), 5);

    // 3. tx_moyany & tx_danshu: losing card outside own turn judges -> Red deals 2 Fire / heals 1; Black no effect
    let mut e_md = start_versus(0x1403, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_md.debug_grant(0, "tx_moyany");
    e_md.debug_grant(0, "tx_danshu");
    e_md.debug_set_hp(0, 4);
    e_md.debug_set_turn(1, Phase::Play, 1);
    let c0a = place_kind_suit(&mut e_md, 0, CardKind::Sha, Suit::Spade);
    let c0b = place_kind_suit(&mut e_md, 0, CardKind::Shan, Suit::Diamond);
    // First CardsLost with Spade judge -> Black: no Fire dmg, no heal!
    let _ = top_deck_suit(&mut e_md, Suit::Spade);
    let _ = e_md.debug_discard_card(0, c0a);
    drain(&mut e_md);
    assert_eq!(e_md.debug_hp(0), 4);
    assert_eq!(e_md.debug_hp(2), 6);
    // Next CardsLost with Heart judge -> moyany deals 2 Fire to chosen target 2, danshu recovers +1 HP!
    let _ = top_deck_suit(&mut e_md, Suit::Heart);
    let _ = top_deck_suit(&mut e_md, Suit::Diamond);
    let _ = e_md.debug_discard_card(0, c0b);
    let c_my = expect_pending(&e_md, "tx_moyany:choose_target");
    assert_eq!(ai_plan(&e_md, 0), GameAction::Answer {
        choice_id: c_my.id,
        selection: vec![ChoiceOption::Player(2)], // enemy over ally 1!
    });
    let _ = answer(&mut e_md, &c_my, vec![ChoiceOption::Player(2)]);
    drain(&mut e_md);
    assert_eq!(e_md.debug_hp(2), 4, "tx_moyany dealt 2 Fire damage");
    assert_eq!(e_md.debug_hp(0), 5, "tx_danshu recovered 1 HP");

    // 4. txyirang: Play begin gives non-basic cards to target with max_hp > own & copies max_hp
    let mut e_yr = start_taixu(0x1404, 2);
    e_yr.debug_grant(0, "txyirang");
    e_yr.debug_set_max_hp(0, 3);
    e_yr.debug_set_hp(0, 3);
    e_yr.debug_set_max_hp(1, 6);
    let basic_s = place_kind_suit(&mut e_yr, 0, CardKind::Sha, Suit::Spade);
    let trick_w = place_kind_suit(&mut e_yr, 0, CardKind::Wuzhong, Suit::Heart);
    e_yr.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_yr);
    let c_yr = expect_pending(&e_yr, "txyirang:choose_target");
    let _ = answer(&mut e_yr, &c_yr, vec![ChoiceOption::Player(1)]);
    drain(&mut e_yr);
    assert_eq!(e_yr.state_core().player(0).max_hp, 6);
    assert!(e_yr.debug_hand(0).contains(&basic_s), "basic stays");
    assert!(e_yr.debug_hand(1).contains(&trick_w), "non-basic moved");
}

/// Upstream: `extension.js:3366-3465` (`txayboji`, `txayhanji`, `tx_dqtianzi`, `xinmingjian`, `xinmingjian2`, `txmazhan`, `txlieji`).
#[test]
fn l11_skills_a_boji_hanji_dqtianzi_mingjian_mazhan_lieji() {
    // 1. txayboji & txayhanji & txmazhan
    let mut e = start_taixu(0x1405, 4);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);
    e.debug_grant(0, "txmazhan"); // from -2, to +1
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1, "txmazhan from -2 clamps to 1");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 0), 3, "txmazhan to +1");

    e.debug_grant(0, "txayboji");
    e.debug_grant(0, "txayhanji");
    let sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let disc = place_kind(&mut e, 0, CardKind::Shan);
    let _extra = place_kind(&mut e, 0, CardKind::Tao);
    let _t1a = place_kind(&mut e, 1, CardKind::Tao);
    let _t1b = place_kind(&mut e, 1, CardKind::Jiu);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    let c_bj = expect_pending(&e, "txayboji:confirm");
    let _ = answer(&mut e, &c_bj, vec![ChoiceOption::Confirm]);
    let c_bd = expect_pending(&e, "txayboji:discard_card");
    let _ = answer(&mut e, &c_bd, vec![ChoiceOption::Card(disc)]);
    drain(&mut e);
    assert!(e.debug_hand(1).is_empty(), "txayboji stripped target's entire hand");
    assert_eq!(e.debug_hand(0).len(), 2, "kept extra card + drew 1 = 2");
    assert_eq!(e.debug_hp(1), 4, "txayhanji added +1 damage on first hit -> 2 damage total");

    // 2. tx_dqtianzi: Diamond skips next Play phase; Black steals hand cards
    let mut e_dq = start_taixu(0x1406, 2);
    e_dq.debug_grant(0, "tx_dqtianzi");
    let t_hand = place_kind_suit(&mut e_dq, 1, CardKind::Tao, Suit::Heart);
    // Play begin triggers dqtianzi -> judge Diamond -> seat 1 skips next Play phase
    let _ = top_deck_suit(&mut e_dq, Suit::Diamond);
    e_dq.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_dq);
    let c_dq1 = expect_pending(&e_dq, "tx_dqtianzi:choose_target");
    let _ = answer(&mut e_dq, &c_dq1, vec![ChoiceOption::Player(1)]);
    drain(&mut e_dq);
    // End seat 0 turn and step seat 1 Judge -> skips Play straight to Discard!
    e_dq.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut e_dq);
    drain(&mut e_dq);
    e_dq.debug_set_turn(1, Phase::Judge, 2);
    let _ = next_phase(&mut e_dq); // Judge -> Draw
    drain(&mut e_dq);
    let _ = next_phase(&mut e_dq); // Draw -> should skip Play!
    drain(&mut e_dq);
    assert_ne!(e_dq.turn().phase, Phase::Play, "Diamond judge skips target's Play phase");

    // Damage triggers dqtianzi -> judge Spade steals cards via choose_cards
    e_dq.debug_set_turn(1, Phase::Play, 2);
    let _ = top_deck_suit(&mut e_dq, Suit::Spade);
    let s_hit = place_kind_suit(&mut e_dq, 1, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e_dq, 1, s_hit, vec![0]);
    let c_dq2 = expect_pending(&e_dq, "tx_dqtianzi:choose_target");
    let _ = answer(&mut e_dq, &c_dq2, vec![ChoiceOption::Player(1)]);
    let c_dq_c = expect_pending(&e_dq, "tx_dqtianzi:choose_cards");
    assert!(c_dq_c.spec.options.contains(&ChoiceOption::Card(t_hand)));
    let _ = answer(&mut e_dq, &c_dq_c, vec![ChoiceOption::Card(t_hand)]);
    drain(&mut e_dq);
    assert!(e_dq.debug_hand(0).contains(&t_hand));

    // 3. xinmingjian + xinmingjian2 (<= 2 per turn, grants unlimited Sha & hand limit +1 until TurnEnded)
    let mut e_mj = start_taixu(0x1407, 2);
    e_mj.debug_grant(0, "xinmingjian");
    let g1 = place_kind_suit(&mut e_mj, 0, CardKind::Sha, Suit::Heart);
    let out_mj = activate_skill(&mut e_mj, 0, "xinmingjian");
    assert!(matches!(out_mj, CommandOutcome::Pending { .. }));
    let c_mc = expect_pending(&e_mj, "xinmingjian:choose_cards");
    let _ = answer(&mut e_mj, &c_mc, vec![ChoiceOption::Card(g1)]);
    let c_mt = expect_pending(&e_mj, "xinmingjian:choose_target");
    let _ = answer(&mut e_mj, &c_mt, vec![ChoiceOption::Player(1)]);
    drain(&mut e_mj);
    assert!(e_mj.debug_has_skill(1, "xinmingjian2"));
    assert!(
        e_mj.state_core().hand_limit(e_mj.catalog(), 1) >= 100,
        "xinmingjian2 grants unlimited hand limit"
    );
    e_mj.debug_set_turn(1, Phase::Play, 1);
    for _ in 0..3 {
        let sx = place_kind_suit(&mut e_mj, 1, CardKind::Sha, Suit::Spade);
        let out_s = play_card(&mut e_mj, 1, sx, vec![0]);
        assert!(matches!(out_s, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
        drain(&mut e_mj);
    }
    e_mj.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_mj);
    drain(&mut e_mj);
    assert!(!e_mj.debug_has_skill(1, "xinmingjian2"), "xinmingjian2 clears on TurnEnded");

    // 4. txlieji: single-target trick/Sha -> direct (no confirm prompt) draw 1 & replay once!
    let mut e_lj = start_taixu(0x1408, 2);
    e_lj.debug_grant(0, "txlieji");
    let s_lj = place_kind_suit(&mut e_lj, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_lj, 0, s_lj, vec![1]);
    drain(&mut e_lj);
    assert_eq!(e_lj.debug_hand(0).len(), 1, "txlieji drew 1 directly");
    assert_eq!(e_lj.debug_hp(1), 4, "Sha resolved twice for 2 total damage");
}

/// Upstream: `extension.js:3466-3580` (`diycuidu`, `diychunxiao`, `diyzhongdu`).
#[test]
fn l11_skills_a_diy_cuidu_chunxiao_zhongdu() {
    let mut e = start_versus(0x1409, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "diycuidu");
    e.debug_grant(0, "diychunxiao");
    e.debug_set_hp(0, 4);
    let s_cd = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e, 0, s_cd, vec![1]);
    drain(&mut e);
    assert!(e.debug_has_skill(1, "diyzhongdu"), "diycuidu inflicted diyzhongdu on enemy 1");
    assert_eq!(e.debug_hand(0).len(), 2, "diycuidu drew 2");

    // Seat 1 Prepare with Club judge -> takes 1 damage AND removes `diyzhongdu`
    let _ = top_deck_suit(&mut e, Suit::Club);
    e.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e); // TurnEnd 0 -> Seat 1 TurnBegin -> Prepare
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 4);
    assert!(!e.debug_has_skill(1, "diyzhongdu"));

    // Counterexample: Spade judge on `diyzhongdu` deals 1 dmg AND keeps `diyzhongdu`; Heart removes with 0 dmg
    let mut e_zd = start_taixu(0x140a, 2);
    e_zd.debug_grant(1, "diyzhongdu");
    let _ = top_deck_suit(&mut e_zd, Suit::Spade);
    e_zd.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_zd);
    drain(&mut e_zd);
    assert_eq!(e_zd.debug_hp(1), 5);
    assert!(e_zd.debug_has_skill(1, "diyzhongdu"));
    let _ = top_deck_suit(&mut e_zd, Suit::Heart);
    e_zd.debug_set_turn(0, Phase::End, 2);
    let _ = next_phase(&mut e_zd);
    drain(&mut e_zd);
    assert_eq!(e_zd.debug_hp(1), 5, "Heart judge deals 0 dmg");
    assert!(!e_zd.debug_has_skill(1, "diyzhongdu"), "Heart removes diyzhongdu");

    // `diychunxiao` at End phase recovers to max_hp on 50% RNG (verify across two seeds)
    let mut recovered_any = false;
    for seed in 0x1410u64..0x1418 {
        let mut ex = start_taixu(seed, 2);
        ex.debug_grant(0, "diychunxiao");
        ex.debug_set_hp(0, 3);
        ex.debug_set_turn(0, Phase::Discard, 1);
        let _ = next_phase(&mut ex);
        drain(&mut ex);
        if ex.debug_hp(0) == 6 {
            recovered_any = true;
            break;
        }
    }
    assert!(recovered_any, "diychunxiao must recover to max_hp when RNG==0");
}

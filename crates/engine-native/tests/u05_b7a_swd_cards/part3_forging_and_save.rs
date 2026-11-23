use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b7a_all_6_forging_families_equip1_to_5_and_held_guards() {
    let (v, mut e) = start_shahai_swd_only(9230, 3);
    let init_set = total_card_set(&e);

    // Verify all 6 g_* held skills + all 30 *_equip1..5 forging skills map cleanly.
    let families = [
        "cangchizhibi",
        "huanglinzhicong",
        "xuanwuzhihuang",
        "zhuquezhizhang",
        "baishouzhihu",
        "qinglongzhigui",
    ];
    for fam in families {
        let held = format!("g_{fam}");
        let held_def = e.catalog().skill_by_name(&held).expect("held skill");
        assert!(e.catalog().skills[held_def as usize].behavior.package_skill_id().is_none());
        for slot in 1..=5 {
            let sk = format!("{fam}_equip{slot}");
            let def = e.catalog().skill_by_name(&sk).expect("forging skill");
            assert!(e.catalog().skills[def as usize].behavior.package_skill_id().is_none());
        }
    }

    // 1. qinglongzhigui_equip1 -> TurnEnd draws 1.
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_grant(0, "qinglongzhigui_equip1");
    let h0 = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "qinglongzhigui_equip1 drew 1 at TurnEnd");

    // 2. cangchizhibi_equip2 -> TurnEnd toggles linked (save/restore across choice).
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    let q_def = e.catalog().skill_by_name("qinglongzhigui_equip1").unwrap();
    e.state_core_mut().skills.remove(&(0, q_def));
    e.debug_grant(0, "cangchizhibi_equip2");
    assert!(!e.state_core().player(1).linked);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_cangchizhibi_forging_target");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(1).linked, "cangchizhibi_equip2 linked seat 1");

    // 3. huanglinzhicong_equip3 -> black card -> +1 hujia; blocked if hujia > 0.
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    let c_def = e.catalog().skill_by_name("cangchizhibi_equip2").unwrap();
    e.state_core_mut().skills.remove(&(0, c_def));
    e.debug_grant(0, "huanglinzhicong_equip3");
    let blk = find_kind_suit(&e, CardKind::Sha, |s| matches!(s, Suit::Spade | Suit::Club)).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, blk);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_huanglinzhicong_forging_discard");
    answer_open(&mut e, vec![ChoiceOption::Card(blk)]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1, "huanglinzhicong_equip3 gave +1 hujia");

    // 4. xuanwuzhihuang_equip4 -> red card -> recover 1 when wounded; skipped at full HP.
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    let h_def = e.catalog().skill_by_name("huanglinzhicong_equip3").unwrap();
    e.state_core_mut().skills.remove(&(0, h_def));
    e.debug_grant(0, "xuanwuzhihuang_equip4");
    e.debug_set_hp(0, 2);
    let red = find_kind_suit(&e, CardKind::Shan, |s| matches!(s, Suit::Heart | Suit::Diamond)).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, red);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_xuanwuzhihuang_forging_discard");
    answer_open(&mut e, vec![ChoiceOption::Card(red)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(0), 3, "xuanwuzhihuang_equip4 recovered 1 HP");

    // 5. zhuquezhizhang_equip5 -> red card -> target (hp >= owner.hp) -> Fire damage 1 (roundtrip).
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    let x_def = e.catalog().skill_by_name("xuanwuzhihuang_equip4").unwrap();
    e.state_core_mut().skills.remove(&(0, x_def));
    e.debug_grant(0, "zhuquezhizhang_equip5");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(2, 4);
    let red2 = find_kind_suit(&e, CardKind::Sha, |s| matches!(s, Suit::Heart | Suit::Diamond)).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, red2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_zhuquezhizhang_forging_card");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(red2)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_zhuquezhizhang_forging_target");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(2), 3, "zhuquezhizhang_equip5 dealt 1 Fire damage");

    // 6. baishouzhihu_equip1 -> target -> discards random he card from target.
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    let z_def = e.catalog().skill_by_name("zhuquezhizhang_equip5").unwrap();
    e.state_core_mut().skills.remove(&(0, z_def));
    e.debug_grant(0, "baishouzhihu_equip1");
    let t_card = find_kind(&e, CardKind::Duel);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 1, t_card);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_baishouzhihu_forging_target");
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e);
    assert!(!e.debug_hand(1).contains(&t_card), "baishouzhihu_equip1 discarded target he card");
    assert_eq!(total_card_set(&e), init_set, "strict conservation across forging suite");
}

#[test]
fn u05_b7a_ai_choice_planner_covers_real_suspended_asks() {
    let (v, mut e) = start_shahai_swd_only(9231, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // Trigger real dujian prompts and let BoundedPlanner answer both own_card & target_card.
    let dujian = find_kind(&e, CardKind::Dujian);
    let s0_card = find_kind(&e, CardKind::Sha);
    let s1_card = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 0, dujian);
    force_move_card_to_hand(&mut e, &v, 0, s0_card);
    force_move_card_to_hand(&mut e, &v, 1, s1_card);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dujian,
            targets: vec![1],
        },
    );
    let planner = BoundedPlanner::new(64);
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let obs = c.observer;
        let plan = e.plan_action(obs, &planner);
        let out = e.apply_plan(&plan).expect("ai apply plan");
        assert!(matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
    }

    // Trigger real cangchizhibi_equip1 TurnEnd prompt and let BoundedPlanner answer it.
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_grant(0, "cangchizhibi_equip1");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_cangchizhibi_forging_target");
    let plan = e.plan_action(0, &planner);
    let out = e.apply_plan(&plan).expect("ai cangchizhibi");
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(e.open_choice().is_none());
    assert_eq!(total_card_set(&e), init_set);
}

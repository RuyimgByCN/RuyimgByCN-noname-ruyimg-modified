use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::CardKind;
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b7b_kongxin_pindian_win_sha_and_lose_discard_with_save_restore() {
    let (v, mut e) = start_shahai_swd_only(9331, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let kx_def = grant_skill(&mut e, &v, 0, "kongxin");
    // Ensure high rank for owner win first
    let hi = (0..e.catalog().cards.len() as u32)
        .find(|&id| e.catalog().cards[id as usize].rank >= 12)
        .expect("rank >= 12");
    let lo = (0..e.catalog().cards.len() as u32)
        .find(|&id| e.catalog().cards[id as usize].rank <= 3)
        .expect("rank <= 3");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, hi);
    force_move_card_to_hand(&mut e, &v, 1, lo);

    // 1. Pindian Win -> seat 1 uses virtual Sha on seat 2 (`t2 != owner`)!
    let hp2_before = e.state_core().player(2).hp;
    clear_hand_to_discard(&mut e, &v, 2);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: kx_def });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_target");
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_pindian_own");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(hi)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_pindian_target");
    answer_open(&mut e, vec![ChoiceOption::Card(lo)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_sha_target");
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(2).hp, hp2_before - 1, "kongxin win triggers target Sha on seat 2");

    // 2. Pindian Lose next turn -> seat 1 discards 1 he card from seat 0
    step_full_turn_to(&mut e, 0);
    let own_spare = find_kind(&e, CardKind::Tao);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, lo);
    force_move_card_to_hand(&mut e, &v, 0, own_spare);
    force_move_card_to_hand(&mut e, &v, 1, hi);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: kx_def });
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    answer_open(&mut e, vec![ChoiceOption::Card(lo)]);
    answer_open(&mut e, vec![ChoiceOption::Card(hi)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_lose_discard");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    answer_open(&mut e, vec![ChoiceOption::Card(own_spare)]);
    assert!(!e.state_core().player(0).hand.contains(&own_spare));

    // 3. `kongxin2` Dying cleanup removes `kongxin2`
    let kx2_def = grant_skill(&mut e, &v, 1, "kongxin2");
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain_pass_all(&mut e);
    assert!(!e.state_core().has_skill(1, kx2_def));
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_chilongya2_chilongfengxue_old_longfan_lingjiandai_and_standalone_skills() {
    let (v, mut e) = start_shahai_swd_only(9332, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. `lingjiandai` real PlayCard fetches up to 3 random `Hslingjian` from deck/discard with strict conservation
    let ljd = find_card_name(&e, "lingjiandai");
    force_move_card_to_hand(&mut e, &v, 0, ljd);
    let hs_before = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == CardKind::Hslingjian)
        .count();
    let out_ljd = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ljd,
            targets: vec![0],
        },
    );
    assert!(matches!(out_ljd, CommandOutcome::Accepted { .. }), "{out_ljd:?}");
    let hs_after = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == CardKind::Hslingjian)
        .count();
    assert_eq!(hs_after, hs_before + 3, "lingjiandai gains 3 Hslingjian");

    // 2. `old_longfan` PhaseUse once_per_turn
    let olf = grant_skill(&mut e, &v, 0, "old_longfan");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: olf });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_old_longfan_target");
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e);

    // 3. Standalone `xingjunyan` + `xixue` + `chilongya2` + `chilongfengxue`
    grant_skill(&mut e, &v, 0, "xingjunyan");
    grant_skill(&mut e, &v, 0, "xixue");
    grant_skill(&mut e, &v, 0, "chilongya2");
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 4);
    clear_hand_to_discard(&mut e, &v, 1);
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    drain_pass_all(&mut e);
    assert!(e.state_core().player(0).hp > 2, "standalone xixue healed after Sha damage");
    assert!(e.state_core().player(1).hp <= 2, "standalone xingjunyan boosted Sha damage");

    // 4. Equipping `Xingjunyan` card simultaneously does NOT double-fire +1 damage!
    step_full_turn_to(&mut e, 0);
    let xjy_card = find_kind(&e, CardKind::Xingjunyan);
    force_move_card_to_hand(&mut e, &v, 0, xjy_card);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: xjy_card, targets: vec![] });
    remove_skill(&mut e, &v, 0, "chilongya2");
    e.debug_set_hp(1, 4);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, 2, "standalone + equipped Xingjunyan must not double-boost");

    // 5. `nigong` + `nigong2` + `nigong3` standalone
    grant_skill(&mut e, &v, 1, "nigong");
    grant_skill(&mut e, &v, 1, "nigong2");
    let n3_def = grant_skill(&mut e, &v, 1, "nigong3");
    e.debug_set_hp(1, 4);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.state_core().player(1).nigong_lv, 1);
    advance_to_play(&mut e, 1);
    let n3_out = submit(&mut e, 1, GameAction::ActivateSkill { skill: n3_def });
    assert!(matches!(n3_out, CommandOutcome::Accepted { .. }), "{n3_out:?}");
    assert_eq!(e.state_core().player(1).nigong_lv, 0);

    // `sadengjinhuan` standalone triggers judge confirm on ShaDodged
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let sd_def = grant_skill(&mut e, &v, 0, "sadengjinhuan");
    let _ = sd_def;
    let shan = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    force_move_card_to_hand(&mut e, &v, 1, shan);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    while e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
        answer_open(&mut e, vec![ChoiceOption::Pass]);
    }
    answer_open(&mut e, vec![ChoiceOption::Card(shan)]); // seat 1 dodges
    assert_eq!(e.open_choice().unwrap().spec.prompt, "sadengjinhuan_judge_confirm");
    answer_open(&mut e, Vec::new()); // decline judge

    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_ai_planner_handles_all_u05_7b_prompts() {
    let (v, mut e) = start_shahai_swd_only(9333, 4);
    advance_to_play(&mut e, 0);
    let planner = BoundedPlanner::new(64);

    let xyy = find_kind(&e, CardKind::Xiangyuye);
    let black_h = (0..e.catalog().cards.len() as u32)
        .find(|&id| {
            e.catalog().card_kind(id) == CardKind::Sha
                && matches!(e.catalog().cards[id as usize].suit, engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club)
        })
        .unwrap();
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 0, xyy);
    force_move_card_to_hand(&mut e, &v, 2, black_h);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: xyy, targets: vec![2] });
    let ch = e.open_choice().cloned().expect("xiangyuye choice");
    let act = e.plan_action(ch.observer, &planner).action;
    assert!(matches!(act, GameAction::Answer { .. }));
    let _ = submit(&mut e, ch.observer, act);
    drain_pass_all(&mut e);

    // Also test AI planner on `u05_guiyanfadao_confirm` & `u05_old_longfan_target` & `u05_baihupifeng_recover`
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 2);
    let gy = find_kind(&e, CardKind::Guiyanfadao);
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, gy);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: gy, targets: vec![] });
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    while e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
        answer_open(&mut e, vec![ChoiceOption::Pass]);
    }
    let ch_gy = e.open_choice().cloned().expect("guiyanfadao choice");
    assert_eq!(ch_gy.spec.prompt, "u05_guiyanfadao_confirm");
    let act_gy = e.plan_action(ch_gy.observer, &planner).action;
    assert!(matches!(act_gy, GameAction::Answer { .. }));
    let _ = submit(&mut e, ch_gy.observer, act_gy);
    drain_pass_all(&mut e);

    let olf = grant_skill(&mut e, &v, 0, "old_longfan");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: olf });
    let ch_olf = e.open_choice().cloned().expect("old_longfan choice");
    assert_eq!(ch_olf.spec.prompt, "u05_old_longfan_target");
    let act_olf = e.plan_action(ch_olf.observer, &planner).action;
    assert!(matches!(act_olf, GameAction::Answer { .. }));
    let _ = submit(&mut e, ch_olf.observer, act_olf);
    drain_pass_all(&mut e);

    let bhp = find_kind(&e, CardKind::Baihupifeng);
    force_move_card_to_hand(&mut e, &v, 0, bhp);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bhp, targets: vec![] });
    e.debug_set_hp(0, 1);
    clear_hand_to_discard(&mut e, &v, 0);
    advance_to_phase(&mut e, 0, engine_native::event::Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let ch_bhp = e.open_choice().cloned().expect("baihupifeng choice");
    assert_eq!(ch_bhp.spec.prompt, "u05_baihupifeng_recover");
    let act_bhp = e.plan_action(ch_bhp.observer, &planner).action;
    assert!(matches!(act_bhp, GameAction::Answer { .. }));
    let _ = submit(&mut e, ch_bhp.observer, act_bhp);
    drain_pass_all(&mut e);
}

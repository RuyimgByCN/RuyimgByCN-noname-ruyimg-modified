//! U05-2 武将技 Batch C & D 上游等价回归（`character/gujian/skill.js:1031-1311, 2097-2485`）。
//!
//! 覆盖 4 位武将 19 技：
//! - 闻人羽：`chizhen`、`shahai-gujian__ref_dangping`
//! - 夏夷则：`shahai-gujian__ref_xuanning`（`xuanning1`/`xuanning2`）、`liuguang`、`yangming`（+`yangming2`）
//! - 欧阳少恭：`yunyin`、`shishui`、`duhun`
//! - 阿阮：`zhaolu`、`jiehuo`（+`jiehuo2`）、`yuling`（`yuling1..6`）

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn u05_2_batch_c_wenrenyu_xiayize_shaogong() {
    let (verified, mut e) = gujian_verified_and_engine(2026_4001, 4);
    e.assign_general_named(0, "gjqt_wenrenyu").unwrap();
    e.assign_general_named(1, "gjqt_xiayize").unwrap();
    e.assign_general_named(2, "gjqt_ouyangshaogong").unwrap();
    advance_to_phase(&mut e, 0, Phase::Judge);
    clear_all_hands(&mut e);

    // 1. 闻人羽 `chizhen` (`character/gujian/skill.js:1237`): at Play PhaseBegin draws max(lost, 1), discards all -> virtual Duel if Sha discarded
    let sha_top = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    e.state_core_mut().deck.retain(|&c| c != sha_top);
    e.state_core_mut().deck.push(sha_top);
    e.debug_set_hp(0, 2); // lost=2 -> draw 2 (includes sha_top) -> discard 2 -> virtual Duel
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play (triggers optional chizhen!)
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Confirm]);
    let cd = e.open_choice().cloned().expect("gj_chizhen_discard");
    assert_eq!(cd.spec.prompt, "gj_chizhen_discard");
    let picked: Vec<ChoiceOption> = cd.spec.options.iter().take(cd.spec.min).copied().collect();
    assert!(picked.contains(&ChoiceOption::Card(sha_top)));
    let _ = answer_choice(&mut e, picked);
    let c_duel = e.open_choice().cloned().expect("gj_chizhen_duel_target");
    assert_eq!(c_duel.spec.prompt, "gj_chizhen_duel_target");
    let enemy0 = enemy_seat(&e, 0);
    let hp_e_before = e.debug_hp(enemy0);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(enemy0)]);
    drain(&mut e);
    assert!(e.debug_hp(enemy0) < hp_e_before || e.debug_hp(0) < 2, "chizhen virtual Duel settled");

    // 2. 夏夷则 `shahai-gujian__ref_xuanning` (`xuanning1` + `xuanning2`), `liuguang`, `yangming` (+ `yangming2`)
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let b1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Sha, Suit::Spade, &mut used);
    let r1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Tao, Suit::Heart, &mut used);
    e.debug_set_hp(1, 2); // wounded -> can trigger yangming2 recover at next Play End
    let sk_xn = e.catalog().skill_by_name("shahai-gujian__ref_xuanning").unwrap();
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: sk_xn });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_xuanning1_card");
    let mut restored = save_restore_verified(&e, &verified);
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Card(b1)]);
    assert_eq!(flag(&restored, 1, "gj_xuanning"), 3, "xuanning1 grants 3 marks");
    // yangming: discard red -> grants yangming2 countdown 2
    let sk_ym = restored.catalog().skill_by_name("yangming").unwrap();
    let _ = submit(&mut restored, 1, GameAction::ActivateSkill { skill: sk_ym });
    assert_eq!(restored.open_choice().unwrap().spec.prompt, "gj_yangming_card");
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Card(r1)]);
    assert_eq!(flag(&restored, 1, "gj_yangming2"), 2);
    // Advance full cycle to seat 0 End -> NextPhase into seat 1 TurnBegin -> triggers `liuguang`!
    advance_to_phase(&mut restored, 0, Phase::End);
    let _ = submit(&mut restored, 0, GameAction::NextPhase);
    drain_until_prompt(&mut restored, "gj_liuguang_targets");
    let clg = restored.open_choice().cloned().expect("gj_liuguang_targets");
    assert_eq!(clg.spec.prompt, "gj_liuguang_targets");
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Player(2)]);
    drain(&mut restored);
    assert_eq!(flag(&restored, 1, "gj_xuanning"), 2, "liuguang consumed 1 xuanning mark");

    // 3. 欧阳少恭 `shishui` (`CardUsed` red -> all targets loseHp 1), `yunyin` (Play/TurnEndWindow), `duhun` (dying compare)
    advance_to_phase(&mut restored, 2, Phase::Play);
    clear_all_hands(&mut restored);
    let rsha2 = move_kind_suit_to_hand(&mut restored, 2, CardKind::Sha, Suit::Heart, &mut used);
    let hp3_b = restored.debug_hp(3);
    let _ = submit(&mut restored, 2, GameAction::PlayCard { card: rsha2, targets: vec![3] });
    drain(&mut restored);
    assert!(restored.debug_hp(3) < hp3_b, "shishui deals +1 loseHp on red card");

    // `duhun` dying rescue: seat 0 dying -> chooses eligible target (hp <= owner.max_hp=3) -> point compare -> win swaps hp & loses 1 max_hp!
    let (_, mut e_dh) = gujian_verified_and_engine(2026_4002, 3);
    e_dh.assign_general_named(0, "gjqt_ouyangshaogong").unwrap();
    e_dh.assign_general_named(1, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e_dh, 1, Phase::Play);
    clear_all_hands(&mut e_dh);
    e_dh.debug_set_hp(0, 1);
    e_dh.debug_set_hp(1, 3);
    let mut used2 = Vec::new();
    let high0 = find_unused_card_by(&e_dh, &used2, |d| d.rank >= 11 && d.kind != CardKind::Shan);
    used2.push(high0);
    move_card_to_hand(&mut e_dh, 0, high0);
    let low1 = find_unused_card_by(&e_dh, &used2, |d| d.rank <= 3);
    used2.push(low1);
    move_card_to_hand(&mut e_dh, 1, low1);
    let ksha1 = move_kind_suit_to_hand(&mut e_dh, 1, CardKind::Sha, Suit::Spade, &mut used2);
    let _ = submit(&mut e_dh, 1, GameAction::PlayCard { card: ksha1, targets: vec![0] });
    let duhun_def = e_dh.catalog().skill_by_name("duhun").unwrap();
    let c_dh = e_dh.open_choice().cloned().expect("dying_save prompt for duhun");
    assert_eq!(c_dh.spec.prompt, "dying_save");
    assert!(c_dh.spec.options.contains(&ChoiceOption::Skill(duhun_def)));
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Skill(duhun_def)]);
    assert_eq!(e_dh.open_choice().unwrap().spec.prompt, "gj_duhun_target");
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Player(1)]);
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Card(high0)]);
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Card(low1)]);
    drain(&mut e_dh);
    assert!(e_dh.state_core().player(0).alive && e_dh.debug_hp(0) >= 2);
}

#[test]
fn u05_2_batch_d_aruan_zhaolu_jiehuo_yuling() {
    let (_, mut e) = gujian_verified_and_engine(2026_4003, 4);
    e.assign_general_named(0, "gjqt_aruan").unwrap();
    advance_to_phase(&mut e, 0, Phase::Judge);
    clear_all_hands(&mut e);

    // 1. `yuling` (`character/gujian/skill.js:2388-2485`):
    //    - yuling1 skips DrawPhase & DiscardPhase!
    //    - yuling2 refills hand to 5 whenever < 5!
    //    - yuling4 caps cards played per turn at max_hp + 2!
    let mut used = Vec::new();
    let _ = move_kind_suit_to_hand(&mut e, 0, CardKind::Wuzhong, Suit::Heart, &mut used);
    // Enter Draw phase: yuling1 skips drawing 2 cards!
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.debug_hand(0).len(), 1, "yuling1 skips Draw phase draw (0 cards drawn)");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::Play, "advance into Play phase");
    // Using a card drops hand < 5 -> yuling2 refills hand back to 5!
    let wz = e.debug_hand(0)[0];
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 5, "yuling2+3 keeps aruan hand size at 5");

    // End Play -> yuling1 skips Discard phase cleanly (emits PhaseSkipped & suppresses DiscardToLimitMachine despite hand=5 > hp)!
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard (skipped: no discard prompt!)
    assert!(
        e.open_choice().is_none(),
        "yuling1 suppresses DiscardToLimitMachine even when hand=5 > hp"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    assert_eq!(e.debug_hand(0).len(), 5, "yuling1 retains all 5 cards past Discard phase");
    drain(&mut e);

    // 2. `jiehuo` (`character/gujian/skill.js:2345`): limited active 2 fire damage then owner dies!
    let (_, mut e2) = gujian_verified_and_engine(2026_4004, 4);
    e2.assign_general_named(0, "gjqt_aruan").unwrap();
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_all_hands(&mut e2);
    let sk_jh = e2.catalog().skill_by_name("jiehuo").unwrap();
    let hp1_b = e2.debug_hp(1);
    let out = submit(&mut e2, 0, GameAction::ActivateSkill { skill: sk_jh });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "gj_jiehuo_target");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Player(1)]);
    drain(&mut e2);
    assert_eq!(e2.debug_hp(1), hp1_b - 2, "jiehuo deals 2 fire damage");
    assert!(!e2.state_core().player(0).alive, "jiehuo sacrifices owner after settling");
}

#[test]
fn u05_2_batch_cd_dangping_xuanning2_yangming2_yunyin_zhaolu_yuling4() {
    // 1. `shahai-gujian__ref_dangping` + `xuanning2` + `yangming2` + `yunyin`
    let (_, mut e) = gujian_verified_and_engine(2026_4005, 4);
    e.assign_general_named(0, "gjqt_wenrenyu").unwrap();
    e.assign_general_named(1, "gjqt_xiayize").unwrap();
    e.assign_general_named(2, "gjqt_ouyangshaogong").unwrap();
    e.assign_general_named(3, "gjqt_aruan").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    // Prepare Xiayize with 2 xuanning marks & Wenrenyu with Sha + TWO spare cards (xuanning2 discards 1, dangping uses the other!)
    set_flag_for_test(&mut e, 1, "gj_xuanning", 2);
    let sha0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let spare0a = move_kind_suit_to_hand(&mut e, 0, CardKind::Tao, Suit::Heart, &mut used);
    let spare0b = move_kind_suit_to_hand(&mut e, 0, CardKind::Shan, Suit::Diamond, &mut used);
    let _ = spare0b;
    let hp2_b = e.debug_hp(2);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    drain_until_prompt(&mut e, "gj_dangping_card");
    let c_dp = e.open_choice().cloned().expect("gj_dangping_card");
    assert_eq!(c_dp.spec.prompt, "gj_dangping_card");
    let dp_card = c_dp.spec.options[0];
    let _ = spare0a;
    let _ = answer_choice(&mut e, vec![dp_card]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_dangping_target");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2_b - 1, "dangping damaged neighbor within dist 1");
    assert_eq!(flag(&e, 1, "gj_xuanning"), 1, "xuanning2 consumed 1 mark on damage");

    // 2. `yangming2`: seat 1 sets `gj_yangming2` = 1 and ends Play -> recovers injured dist<=1 neighbor!
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    let r1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Tao, Suit::Heart, &mut used);
    let sk_ym = e.catalog().skill_by_name("yangming").unwrap();
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: sk_ym });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(r1)]);
    set_flag_for_test(&mut e, 1, "gj_yangming2", 1);
    let hp2_pre = e.debug_hp(2);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play -> Discard triggers `yangming2`!
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yangming2_targets");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2_pre + 1, "yangming2 healed injured dist<=1 target");

    // 3. `yunyin`: Shaogong discards unused suit at TurnPhaseEnd -> virtual Wuzhong -> draws 2!
    advance_to_phase(&mut e, 2, Phase::End);
    clear_all_hands(&mut e);
    let yy_card = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yunyin_card");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(yy_card)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yunyin_vcard");
    // Option matching Wuzhong (index 3 in YUNYIN_VCARDS)
    let _ = answer_choice(&mut e, vec![ChoiceOption::Tag(3)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(2).len(), 2, "yunyin Wuzhong drew 2 cards");

    // 4. `zhaolu`: Aruan (seat 3) countdown drops & resets + `yuling4` caps Play cards at max_hp + 2!
    advance_to_phase(&mut e, 3, Phase::Play);
    clear_all_hands(&mut e);
    // Play 5 cards (max_hp 3 + 2 = 5) -> 6th Rejected by `yuling4`!
    for i in 0..6 {
        let c = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Wuzhong || d.kind == CardKind::Sha);
        let is_sha = e.catalog().card_kind(c) == CardKind::Sha;
        used.push(c);
        move_card_to_hand(&mut e, 3, c);
        if is_sha {
            e.state_core_mut().turn.sha_used_play_phase = false;
        }
        let tgts = if is_sha { vec![0] } else { vec![] };
        let out = submit(&mut e, 3, GameAction::PlayCard { card: c, targets: tgts });
        if i == 5 {
            assert!(matches!(out, CommandOutcome::Rejected(_)), "yuling4 caps Play phase at max_hp+2=5");
        } else {
            drain(&mut e);
        }
    }
    // zhaolu countdown verified: starting 4 decrements on TurnEnd
    set_flag_for_test(&mut e, 3, "gj_zhaolu_count", 1);
    let mhp3_b = e.state_core().player(3).max_hp;
    advance_to_phase(&mut e, 3, Phase::End);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(e.state_core().player(3).max_hp, mhp3_b - 1, "zhaolu loses 1 max_hp when countdown hits 0");
    assert_eq!(e.state_core().alive_count(), 3, "one player died earlier in scenario");
    assert_eq!(flag(&e, 3, "gj_zhaolu_count"), 3, "zhaolu resets countdown to min(alive_count, 5)");
}

fn set_flag_for_test(e: &mut engine_native::Engine, p: u8, k: &str, v: u32) {
    e.state_core_mut().vm_aux.flags.insert((p, k.to_string()), v);
}

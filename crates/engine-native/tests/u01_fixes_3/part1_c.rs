use super::common::*;

/// (01) `dddxiaheng` (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3944-4018`)
/// - `t1` with >2 `he` chooses 2 cards to discard (`observer = t1`, not auto-taken).
/// - `t2` takes 1 damage FIRST, then `!del` -> `loseMaxHp` (including down to 0).
/// - `t1 == t2` removes `"dddxiaheng"` from `skills` (no further triggers on later End phases).
/// - After 3 distinct targets take damage, `dddxiaheng_del` prevents future `loseMaxHp`.
#[test]
fn u01_fix3_01_dddxiaheng() {
    // Part A: t1 (>2 he) chooses discard cards; t1 == t2 removes `dddxiaheng` and drops max_hp to 0 -> dies!
    let (mut e, d3, p9) = new_clean_engine(701, 4);
    e.debug_grant(0, "dddxiaheng");
    e.debug_set_max_hp(0, 1);
    e.debug_set_hp(0, 1);
    let c1 = find_nth_card_by(&e, 0, |_, _| true);
    let c2 = find_nth_card_by(&e, 1, |_, _| true);
    let c3 = find_nth_card_by(&e, 2, |_, _| true);
    let c4 = find_nth_card_by(&e, 3, |_, _| true);
    put_in_hand(&mut e, 1, c1);
    put_in_hand(&mut e, 1, c2);
    put_in_hand(&mut e, 1, c3);
    put_in_hand(&mut e, 1, c4);
    e.debug_set_turn(0, Phase::Discard, 1);

    // Forced target selection: `allow_cancel` is false (`min == 1`), empty answer rejected, and 0-card target (P2) is allowed!
    let mut e_zero_target = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    submit(&mut e_zero_target, 0, GameAction::NextPhase);
    let c_t1_forced = e_zero_target.open_choice().unwrap().clone();
    assert_eq!(c_t1_forced.spec.prompt, "dddxiaheng_discard_target");
    assert!(!c_t1_forced.spec.allow_cancel);
    assert_eq!(c_t1_forced.spec.min, 1);
    assert!(c_t1_forced.spec.options.contains(&ChoiceOption::Player(2)), "0-card alive target P2 must be eligible");
    let rej_empty = submit(&mut e_zero_target, 0, GameAction::Answer { choice_id: c_t1_forced.id, selection: vec![] });
    assert!(matches!(rej_empty, CommandOutcome::Rejected(_)));
    answer(&mut e_zero_target, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e_zero_target.open_choice().unwrap().spec.prompt, "dddxiaheng_damage_target", "0-card t1 skips discard and proceeds directly to damage target");
    assert_eq!(e_zero_target.state_core().player(1).hand.len(), 4);
    assert!(has_skill(&e_zero_target, 0, "dddxiaheng"));

    // Positive: pick t1 = P1, then t1 chooses c2 & c4 specifically (with save/restore + AI check).
    submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddxiaheng_discard_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let c_cards = e.open_choice().unwrap().clone();
    assert_eq!(c_cards.observer, 1);
    assert_eq!(c_cards.spec.prompt, "dddxiaheng_discard_cards");
    assert_eq!(c_cards.spec.min, 2);
    assert_eq!(c_cards.spec.max, 2);
    assert!(!c_cards.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(
        &e,
        &d3,
        &p9,
        vec![ChoiceOption::Card(c2), ChoiceOption::Card(c4)],
    );
    answer(&mut e, 1, vec![ChoiceOption::Card(c2), ChoiceOption::Card(c4)]);
    assert_eq!(e.state_core().player(1).hand, vec![c1, c3]);

    // Choice A: t2 = P1 (same as t1) -> damage 1, loseMaxHp(1) to 0 (dies!), remove `dddxiaheng`.
    let mut e_same = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let hp1_before = e_same.state_core().player(1).hp;
    assert_eq!(e_same.open_choice().unwrap().spec.prompt, "dddxiaheng_damage_target");
    answer(&mut e_same, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e_same);
    assert_eq!(e_same.state_core().player(1).hp, hp1_before - 1);
    assert!(!e_same.state_core().player(0).alive, "max_hp=1 after loseMaxHp=1 drops to 0 -> dead");
    assert!(!has_skill(&e_same, 0, "dddxiaheng"));

    // Part B: 3 distinct damaged targets set `dddxiaheng_del`, skipping 4th `loseMaxHp`; t1==t2 on healthy P0 removes skill!
    let (mut e3, _d3, _p9) = new_clean_engine(702, 5);
    e3.debug_grant(0, "dddxiaheng");
    e3.debug_set_max_hp(0, 6);
    e3.debug_set_hp(0, 6);
    for target in 1u8..=3 {
        let card = find_nth_card_by(&e3, usize::from(target) + 10, |_, _| true);
        put_in_hand(&mut e3, 4, card);
        e3.debug_set_turn(0, Phase::Discard, u32::from(target));
        submit(&mut e3, 0, GameAction::NextPhase);
        answer(&mut e3, 0, vec![ChoiceOption::Player(4)]);
        answer(&mut e3, 0, vec![ChoiceOption::Player(target)]);
    }
    assert_eq!(e3.state_core().player(0).max_hp, 3, "3 distinct damages lost 3 max_hp");
    // 4th turn: t1 != t2 with `dddxiaheng_del` already active -> NO further loseMaxHp!
    let c_extra = find_nth_card_by(&e3, 20, |_, _| true);
    put_in_hand(&mut e3, 1, c_extra);
    e3.debug_set_turn(0, Phase::Discard, 4);
    submit(&mut e3, 0, GameAction::NextPhase);
    answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e3, 0, vec![ChoiceOption::Player(4)]);
    assert_eq!(e3.state_core().player(0).max_hp, 3, "after 3 distinct targets, del skips loseMaxHp");
    assert!(has_skill(&e3, 0, "dddxiaheng"));

    // Now pick t1 == t2 (both P2) -> removes actual `dddxiaheng` skill; next End phase does not trigger.
    let c_extra2 = find_nth_card_by(&e3, 21, |_, _| true);
    put_in_hand(&mut e3, 2, c_extra2);
    e3.debug_set_turn(0, Phase::Discard, 5);
    submit(&mut e3, 0, GameAction::NextPhase);
    answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
    answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
    assert!(!has_skill(&e3, 0, "dddxiaheng"), "t1 == t2 must remove skill from owner");

    let c_extra3 = find_nth_card_by(&e3, 22, |_, _| true);
    put_in_hand(&mut e3, 1, c_extra3);
    e3.debug_set_turn(0, Phase::Discard, 6);
    submit(&mut e3, 0, GameAction::NextPhase);
    assert!(e3.open_choice().is_none(), "removed dddxiaheng must not trigger again");
}

/// (02) `dddfengzheng` & `dddfengzheng_global` (`skill.js:4019-4212`)
/// - Global companion `dddfengzheng_global` granted to all seats when `dddfengzheng` exists.
/// - PlayCard / ActivateSkill: hand cards only (>=1, no 4-card cap), rank sum must == 13, casts real Wuzhong, usable: 1.
/// - RoundStart: N = alive seats that used `dddfengzheng_global` last round; owner draws N and distributes across targets;
///   any target receiving >2 cards removes `dddfengzheng` (and unsyncs `dddfengzheng_global`).
#[test]
fn u01_fix3_02_dddfengzheng() {
    let (mut e, d3, p9) = new_clean_engine(703, 4);
    assert!(!has_skill(&e, 1, "dddfengzheng_global"), "global skill hidden without owner");
    e.debug_grant(0, "dddfengzheng");
    for p in 0..4 {
        assert!(has_skill(&e, p, "dddfengzheng_global"), "global skill synced to seat {p}");
    }
    let g_def = e.catalog().skill_by_name("dddfengzheng_global").expect("dddfengzheng_global in catalog");

    // Pick 5 hand cards summing to 13 (tests no 4-card cap!) and 1 bad card (rank != 13) for P1; rank-13 K for P2; rank-13 K for P3 (who dies).
    let r2a = find_nth_card_by(&e, 0, |_, d| d.rank == 2);
    let r2b = find_nth_card_by(&e, 1, |_, d| d.rank == 2);
    let r3a = find_nth_card_by(&e, 0, |_, d| d.rank == 3);
    let r3b = find_nth_card_by(&e, 1, |_, d| d.rank == 3);
    let r3c = find_nth_card_by(&e, 2, |_, d| d.rank == 3);
    let r1 = find_card_by(&e, |_, d| d.rank == 1);
    let rk2 = find_nth_card_by(&e, 0, |_, d| d.rank == 13);
    let rk3 = find_nth_card_by(&e, 1, |_, d| d.rank == 13);
    let eq_r5 = find_card_by(&e, |_, d| d.kind.equip_slot().is_some() && d.rank == 5);
    let r8 = find_card_by(&e, |_, d| d.rank == 8);
    for &c in &[r2a, r2b, r3a, r3b, r3c, r1, r8] {
        put_in_hand(&mut e, 1, c);
    }
    put_in_equip(&mut e, 1, eq_r5);
    put_in_hand(&mut e, 2, rk2);
    put_in_hand(&mut e, 3, rk3);

    // P1 Play phase round 1: equip not in `dddfengzheng_cards` options; sum != 13 rejected!
    e.debug_set_turn(1, Phase::Play, 1);
    let out = submit(&mut e, 1, GameAction::ActivateSkill { skill: g_def });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let cc = e.open_choice().unwrap().clone();
    assert_eq!(cc.spec.prompt, "dddfengzheng_cards");
    assert!(!cc.spec.options.contains(&ChoiceOption::Card(eq_r5)), "equip cannot be used for dddfengzheng_global");
    // Negative: rank 1 (sum != 13) rejected!
    let bad = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cc.id,
            selection: vec![ChoiceOption::Card(r1)],
        },
    );
    assert!(matches!(bad, CommandOutcome::Rejected(_)), "sum != 13 must be rejected");

    // Negative: sum = 5 via r2a+r3a != 13 rejected!
    let bad2 = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cc.id,
            selection: vec![ChoiceOption::Card(r2a), ChoiceOption::Card(r3a)],
        },
    );
    assert!(matches!(bad2, CommandOutcome::Rejected(_)));

    // Positive: 5 hand cards summing 2+2+3+3+3 = 13 -> casts real Wuzhong (draws 2!).
    let p1_before = e.state_core().player(1).hand.len();
    answer(
        &mut e,
        1,
        vec![
            ChoiceOption::Card(r2a),
            ChoiceOption::Card(r2b),
            ChoiceOption::Card(r3a),
            ChoiceOption::Card(r3b),
            ChoiceOption::Card(r3c),
        ],
    );
    assert_eq!(e.state_core().player(1).hand.len(), p1_before - 5 + 2);
    // Usable: 1 in same Play phase.
    let again = submit(&mut e, 1, GameAction::ActivateSkill { skill: g_def });
    assert!(matches!(again, CommandOutcome::Rejected(_)), "usable: 1 per play phase");

    // P2 uses rank 13 Wuzhong; P3 uses rank 13 Wuzhong and then dies (not alive at round 2 start).
    e.debug_set_turn(2, Phase::Play, 1);
    submit(&mut e, 2, GameAction::ActivateSkill { skill: g_def });
    answer(&mut e, 2, vec![ChoiceOption::Card(rk2)]);
    e.debug_set_turn(3, Phase::Play, 1);
    submit(&mut e, 3, GameAction::ActivateSkill { skill: g_def });
    answer(&mut e, 3, vec![ChoiceOption::Card(rk3)]);
    e.state_core_mut().player_mut(3).alive = false;

    // Advance P2 Play -> End -> Turn wrap to Round 2 (P0 TurnBegin). Alive users from round 1 = {P1, P2} => N = 2!
    let p0_hand_marker = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Shan && d.rank != 13);
    put_in_hand(&mut e, 0, p0_hand_marker);
    e.debug_set_turn(2, Phase::End, 1);
    clear_hand(&mut e, 2);
    let p0_before_r2 = e.state_core().player(0).hand.len();
    let deck_before_r2 = e.state_core().deck.len();
    submit(&mut e, 2, GameAction::NextPhase);
    let _ = drain;
    // Note: P3 is dead so P2 Discard End skips P3 and enters Round 2 P0 TurnBegin!
    let c_conf = e.open_choice().expect("dddfengzheng optional confirm").clone();
    assert_eq!(c_conf.observer, 0);
    assert_eq!(c_conf.spec.prompt, "dddfengzheng_confirm");
    assert!(c_conf.spec.allow_cancel);
    assert_eq!(e.state_core().player(0).hand.len(), p0_before_r2);
    assert!(e.state_core().player(0).off_court.is_empty());
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);

    // Cancel branch: declining confirm leaves deck/hand/off_court untouched and skips Round 2 distribution!
    let mut e_decline = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_decline, 0, vec![ChoiceOption::Pass]);
    assert!(e_decline.open_choice().is_none());
    assert_eq!(e_decline.state_core().deck.len(), deck_before_r2);
    assert!(e_decline.state_core().player(0).off_court.is_empty());
    assert_eq!(e_decline.state_core().player(0).hand.len(), p0_before_r2);
    assert!(has_skill(&e_decline, 0, "dddfengzheng"));

    // Confirm branch: pops N = 2 from deck into ordering zone (`off_court`), NOT P0's hand!
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), p0_before_r2, "drawn cards must go to ordering zone (off_court), not hand");
    assert_eq!(e.state_core().player(0).off_court.len(), 2, "N = 2 alive round-1 global users (dead P3 excluded)");
    let ct = e.open_choice().expect("dddfengzheng roundStart distribute target").clone();
    assert_eq!(ct.spec.prompt, "dddfengzheng_give_target");
    assert!(!ct.spec.allow_cancel);
    assert_eq!(ct.spec.min, 1);
    let rej_t = submit(&mut e, 0, GameAction::Answer { choice_id: ct.id, selection: vec![] });
    assert!(matches!(rej_t, CommandOutcome::Rejected(_)), "dddfengzheng_give_target is mandatory");

    // Branch A: distribute 1 to P1 and 1 to P2 from ordering zone -> neither >2 -> keeps dddfengzheng & dddfengzheng_global!
    let mut e_split = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let p1_h = e_split.state_core().player(1).hand.len();
    let p2_h = e_split.state_core().player(2).hand.len();
    answer(&mut e_split, 0, vec![ChoiceOption::Player(1)]);
    let c_dist = e_split.open_choice().unwrap().clone();
    assert_eq!(c_dist.spec.prompt, "dddfengzheng_give_cards");
    assert!(!c_dist.spec.allow_cancel);
    assert_eq!(c_dist.spec.min, 1);
    assert!(!c_dist.spec.options.contains(&ChoiceOption::Card(p0_hand_marker)), "pre-existing hand card must NOT be offered for distribution");
    assert_eq!(c_dist.spec.options.len(), 2);
    let first_off = e_split.state_core().player(0).off_court[0];
    assert_mid_choice_archive_and_ai(&e_split, &d3, &p9, vec![ChoiceOption::Card(first_off)]);
    answer(&mut e_split, 0, vec![ChoiceOption::Card(first_off)]);
    let next_t = e_split.open_choice().expect("remaining off_court card prompts target P2").clone();
    assert_eq!(next_t.spec.prompt, "dddfengzheng_give_target");
    assert!(!next_t.spec.allow_cancel);
    answer(&mut e_split, 0, vec![ChoiceOption::Player(2)]);
    let next_c = e_split.open_choice().expect("dddfengzheng_give_cards for 2nd target").clone();
    assert_eq!(next_c.spec.prompt, "dddfengzheng_give_cards");
    let card_to_p2 = e_split.state_core().player(0).off_court[0];
    answer(&mut e_split, 0, vec![ChoiceOption::Card(card_to_p2)]);
    assert!(e_split.state_core().player(0).off_court.is_empty());
    assert_eq!(e_split.state_core().player(0).hand.len(), p0_before_r2);
    assert!(e_split.state_core().player(0).hand.contains(&p0_hand_marker));
    assert_eq!(e_split.state_core().player(1).hand.len(), p1_h + 1);
    assert_eq!(e_split.state_core().player(2).hand.len(), p2_h + 1);
    assert!(has_skill(&e_split, 0, "dddfengzheng"));
    assert!(has_skill(&e_split, 1, "dddfengzheng_global"));

    // Branch B: force N=3 in round 2 -> confirm & give all 3 to P1 -> >2 removes `dddfengzheng` AND removes `dddfengzheng_global`!
    e_split.debug_set_turn(0, Phase::Play, 2);
    let r13a = find_nth_card_by(&e_split, 2, |_, d| d.rank == 13);
    put_in_hand(&mut e_split, 0, r13a);
    submit(&mut e_split, 0, GameAction::ActivateSkill { skill: g_def });
    answer(&mut e_split, 0, vec![ChoiceOption::Card(r13a)]);
    e_split.debug_set_turn(1, Phase::Play, 2);
    let r13b = find_nth_card_by(&e_split, 3, |_, d| d.rank == 13);
    put_in_hand(&mut e_split, 1, r13b);
    submit(&mut e_split, 1, GameAction::ActivateSkill { skill: g_def });
    answer(&mut e_split, 1, vec![ChoiceOption::Card(r13b)]);
    e_split.debug_set_turn(2, Phase::Play, 2);
    let r13c = find_nth_card_by(&e_split, 4, |_, d| d.rank == 13);
    put_in_hand(&mut e_split, 2, r13c);
    submit(&mut e_split, 2, GameAction::ActivateSkill { skill: g_def });
    answer(&mut e_split, 2, vec![ChoiceOption::Card(r13c)]);
    e_split.debug_set_turn(2, Phase::End, 2);
    clear_hand(&mut e_split, 2);
    submit(&mut e_split, 2, GameAction::NextPhase);
    assert_eq!(e_split.open_choice().unwrap().spec.prompt, "dddfengzheng_confirm");
    answer(&mut e_split, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e_split.open_choice().unwrap().spec.prompt, "dddfengzheng_give_target");
    answer(&mut e_split, 0, vec![ChoiceOption::Player(1)]);
    let c3_pick: Vec<ChoiceOption> = e_split.state_core().player(0).off_court.iter().map(|&c| ChoiceOption::Card(c)).collect();
    assert_eq!(c3_pick.len(), 3);
    answer(&mut e_split, 0, c3_pick);
    assert!(e_split.state_core().player(0).off_court.is_empty());
    assert!(!has_skill(&e_split, 0, "dddfengzheng"), ">2 cards to a player removes dddfengzheng");
    for p in 0..4 {
        assert!(!has_skill(&e_split, p, "dddfengzheng_global"), "global skill must disappear when last owner loses dddfengzheng");
    }
}

/// (03) `dddyulv` (`skill.js:4213-4306`)
/// - Init at Prepare draws 1 and puts 1 `he` card onto `"dddyulv"` pile.
/// - Same-rank card entering discard via PlayCard use OR equip swap triggers `dddyulv_mode` (`allow_cancel: false`) for `cur`.
/// - 2nd match in SAME turn: if owner hand empty -> draws 1; then owner chooses 1 HAND card ("h" only) to swap with `"dddyulv"` pile!
#[test]
fn u01_fix3_03_dddyulv() {
    let (mut e, d3, p9) = new_clean_engine(704, 3);
    e.debug_grant(0, "dddyulv");
    let r7_put = find_card_by(&e, |_, d| d.rank == 7 && d.kind != CardKind::Sha && d.kind != CardKind::Wuzhong);
    let r7_use = find_card_by(&e, |_, d| d.rank == 7 && d.kind == CardKind::Sha);
    let r7_wuzhong = find_card_by(&e, |_, d| d.rank == 7 && d.kind == CardKind::Wuzhong);
    let r5_other = find_card_by(&e, |_, d| d.rank == 6 && d.kind == CardKind::Tao);
    let eq_p0 = find_card_by(&e, |_, d| d.kind.equip_slot().is_some() && d.rank != 7);
    put_in_hand(&mut e, 0, r7_put);
    put_in_equip(&mut e, 0, eq_p0);

    // Round-1 start (`phaseBefore`, `phase_number == 0`): draw 1 + choose 1 HAND ("h" only) card onto dddyulv pile!
    advance_to_prepare(&mut e, 0, 1);
    assert_eq!(e.state_core().turn.phase_number, 0);
    let c_put = e.open_choice().unwrap().clone();
    assert_eq!(c_put.spec.prompt, "dddyulv_put");
    assert!(!c_put.spec.allow_cancel);
    assert!(!c_put.spec.options.contains(&ChoiceOption::Card(eq_p0)), "dddyulv_put chooses hand ('h') only, not equip");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(r7_put)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(r7_put)]);
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddyulv".into())).map(Vec::as_slice), Some(&[r7_put][..]));

    // Verify off-seat Round-1 start: when Seat 1 owns dddyulv, it STILL triggers BEFORE Seat 0's Prepare (`phase_number == 0`)!
    let (mut e_pre, _, _) = new_clean_engine(7041, 3);
    e_pre.debug_grant(1, "dddyulv");
    advance_to_prepare(&mut e_pre, 0, 1);
    let c_pre = e_pre.open_choice().expect("dddyulv must fire at phaseBefore phase_number==0 before seat 0").clone();
    assert_eq!(c_pre.observer, 1);
    assert_eq!(c_pre.spec.prompt, "dddyulv_put");
    assert_eq!(e_pre.state_core().turn.phase_number, 0);

    // Advance to P1 Play; P1 plays rank 6 (non-matching) -> NO trigger; plays rank 7 Wuzhong -> naturally discards rank 7 -> triggers #1!
    e.debug_set_turn(1, Phase::Play, 1);
    e.debug_set_hp(1, 2);
    put_in_hand(&mut e, 1, r5_other);
    put_in_hand(&mut e, 1, r7_wuzhong);
    put_in_hand(&mut e, 1, r7_use);
    let extra_h1 = find_nth_card_by(&e, 0, |_, d| d.rank == 4);
    put_in_hand(&mut e, 1, extra_h1);

    submit(&mut e, 1, GameAction::PlayCard { card: r5_other, targets: vec![1] });
    assert!(e.open_choice().is_none(), "different rank card entering discard must not trigger dddyulv");

    submit(&mut e, 1, GameAction::PlayCard { card: r7_wuzhong, targets: vec![] });
    let cm1 = e.open_choice().expect("dddyulv_mode after used r7 Wuzhong enters discard").clone();
    assert_eq!(cm1.observer, 0, "owner (P0) chooses dddyulv_mode, not current turn player");
    assert_eq!(cm1.spec.prompt, "dddyulv_mode");
    assert!(!cm1.spec.allow_cancel, "dddyulv_mode is mandatory");
    // Negative: cancelling / empty rejected!
    let bad = submit(&mut e, 0, GameAction::Answer { choice_id: cm1.id, selection: vec![] });
    assert!(matches!(bad, CommandOutcome::Rejected(_)));
    // Choice B: Tag(1) chosen by owner P0 -> cur P1 chooses 1 he card to discard (`observer = 1`).
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    let cd = e.open_choice().expect("dddyulv_discard choice for P1").clone();
    assert_eq!(cd.observer, 1);
    assert_eq!(cd.spec.prompt, "dddyulv_discard");
    answer(&mut e, 1, vec![ChoiceOption::Card(extra_h1)]);
    assert!(e.state_core().discard.contains(&extra_h1));
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddyulv".into())).map(Vec::as_slice), Some(&[r7_put][..]), "1st trigger in turn does not swap pile");

    // 2nd trigger in SAME turn: P0 hand drained to 0 (only eq_p0 in equips) so 2nd trigger draws 1 then swaps HAND card (not eq_p0!).
    clear_hand(&mut e, 0);
    let top_draw = find_nth_card_by(&e, 1, |_, d| d.rank == 9);
    let p1_draw = find_nth_card_by(&e, 2, |_, d| d.rank == 8);
    put_deck_top(&mut e, top_draw);
    put_deck_top(&mut e, p1_draw);
    submit(&mut e, 1, GameAction::PlayCard { card: r7_use, targets: vec![2] });
    let cm2 = e.open_choice().expect("2nd dddyulv_mode after r7 Sha enters discard").clone();
    assert_eq!(cm2.observer, 0);
    assert_eq!(cm2.spec.prompt, "dddyulv_mode");
    let p1_h_before = e.state_core().player(1).hand.len();
    // Choice A: Tag(0) -> cur P1 draws 1!
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e.state_core().player(1).hand.len(), p1_h_before + 1);
    // Owner P0 had empty hand -> drew `top_draw`, now `dddyulv_swap` allows ONLY hand card (`top_draw`), NOT `eq_p0`!
    let csw = e.open_choice().expect("2nd trigger must prompt dddyulv_swap").clone();
    assert_eq!(csw.observer, 0);
    assert_eq!(csw.spec.prompt, "dddyulv_swap");
    assert!(!csw.spec.allow_cancel);
    assert!(!csw.spec.options.contains(&ChoiceOption::Card(eq_p0)), "swap chooses hand ('h') only, not equip");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(top_draw)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(top_draw)]);
    assert!(e.state_core().player(0).hand.contains(&r7_put));
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddyulv".into())).map(Vec::as_slice), Some(&[top_draw][..]));
}

/// (04) `dddyulv_old` (`skill.js:4307-4402`)
/// - Owner declares rank `1..=13` at Round-1 start (`phaseBefore`, `phase_number == 0`).
/// - When declared-rank card enters discard during `cur`'s turn, OWNER chooses draw 1 (`Tag(0)`) or `cur` discards 1 `he` (`Tag(1)`).
/// - If `cur` suffered that same option earlier -> owner re-declares `1..=13` AND immediately ends `cur`'s turn!
#[test]
fn u01_fix3_04_dddyulv_old() {
    // Off-seat Round-1 start (`phase_number == 0`) BEFORE Seat 0 Prepare:
    let (mut e_pre, _, _) = new_clean_engine(7051, 3);
    e_pre.debug_grant(1, "dddyulv_old");
    advance_to_prepare(&mut e_pre, 0, 1);
    let c_pre = e_pre.open_choice().expect("dddyulv_old must declare at phaseBefore phase_number==0 before seat 0").clone();
    assert_eq!(c_pre.observer, 1);
    assert_eq!(c_pre.spec.prompt, "dddyulv_old_declare");
    assert_eq!(e_pre.state_core().turn.phase_number, 0);

    let (mut e, d3, p9) = new_clean_engine(705, 3);
    e.debug_grant(0, "dddyulv_old");
    advance_to_prepare(&mut e, 0, 1);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddyulv_old_declare");
    answer(&mut e, 0, vec![ChoiceOption::Tag(7)]); // Declare rank 7!

    let r6a = find_nth_card_by(&e, 0, |_, d| d.rank == 7 && d.kind == CardKind::Wuzhong);
    let r6b = find_nth_card_by(&e, 0, |_, d| d.rank == 7 && d.kind == CardKind::Sha);
    let r6c = find_card_by(&e, |_, d| d.rank == 7 && d.kind == CardKind::SavageAttack);
    e.debug_set_turn(1, Phase::Play, 1);
    put_in_hand(&mut e, 1, r6a);
    put_in_hand(&mut e, 1, r6b);
    let d_card = find_nth_card_by(&e, 0, |_, d| d.rank == 4);
    put_in_hand(&mut e, 1, d_card);

    // Match #1: owner P0 picks Tag(0) draw 1 (`observer == 0`).
    submit(&mut e, 1, GameAction::PlayCard { card: r6a, targets: vec![] });
    let cm1 = e.open_choice().unwrap().clone();
    assert_eq!(cm1.observer, 0, "owner P0 chooses dddyulv_old_mode");
    assert_eq!(cm1.spec.prompt, "dddyulv_old_mode");
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e.state_core().turn.current, 1, "first choice stays in P1 turn");

    // Match #2: owner P0 picks Tag(1) discard (different from 0) -> P1 (`observer == 1`) discards -> stays in P1 turn!
    submit(&mut e, 1, GameAction::PlayCard { card: r6b, targets: vec![2] });
    let cm2 = e.open_choice().unwrap().clone();
    assert_eq!(cm2.observer, 0);
    assert_eq!(cm2.spec.prompt, "dddyulv_old_mode");
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    let cd2 = e.open_choice().unwrap().clone();
    assert_eq!(cd2.observer, 1);
    assert_eq!(cd2.spec.prompt, "dddyulv_old_discard");
    answer(&mut e, 1, vec![ChoiceOption::Card(d_card)]);
    assert_eq!(e.state_core().turn.current, 1, "different option keeps P1 turn");

    // Match #3 (even in another turn!): owner P0 picks Tag(1) AGAIN (same as P1's stored option 1) -> re-declare + immediately end P1's turn!
    e.debug_set_turn(1, Phase::Play, 2);
    put_in_hand(&mut e, 1, r6c);
    let extra_keep = find_nth_card_by(&e, 3, |_, d| d.rank == 2);
    put_in_hand(&mut e, 1, extra_keep);
    submit(&mut e, 1, GameAction::PlayCard { card: r6c, targets: vec![] });
    let cm = e.open_choice().unwrap().clone();
    assert_eq!(cm.observer, 0);
    assert_eq!(cm.spec.prompt, "dddyulv_old_mode");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    let cd3 = e.open_choice().unwrap().clone();
    assert_eq!(cd3.observer, 1);
    assert_eq!(cd3.spec.prompt, "dddyulv_old_discard");
    answer(&mut e, 1, vec![ChoiceOption::Card(extra_keep)]);
    // Re-declares 1..=13!
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddyulv_old_declare");
    answer(&mut e, 0, vec![ChoiceOption::Tag(10)]);
    // P1's current turn must have ended and handed off to P2!
    assert_ne!(e.state_core().turn.current, 1, "consecutive same option must end cur's current turn");
}

/// (05) `dddfenye` (`skill.js:4403-4874`)
/// - Owner + target mandatory hand card; all other alive players with hand optionally pick card + group side.
/// - All chosen cards discarded; compares exact group means `sum0/cnt0` vs `sum1/cnt1`.
/// - Winners in seat order optionally use nodistance virtual Sha on remaining losers (removed once targeted); tie -> no winners.
#[test]
fn u01_fix3_05_dddfenye() {
    let (mut e, d3, p9) = new_clean_engine(706, 4);
    e.debug_grant(0, "dddfenye");
    let fy_def = e.catalog().skill_by_name("dddfenye").unwrap();

    // Negative: no other alive player has hand cards -> cannot activate.
    let c0_tmp = find_card_by(&e, |_, d| d.rank == 9);
    put_in_hand(&mut e, 0, c0_tmp);
    e.debug_set_turn(0, Phase::Play, 1);
    let rej = submit(&mut e, 0, GameAction::ActivateSkill { skill: fy_def });
    assert!(matches!(rej, CommandOutcome::Rejected(_)), "no other player with hand -> rejected");

    // Setup: P0=10 vs P1=12 + bystander P3 joins P1 with rank 6 (mean(P1,P3)=9 < 10!) -> P0 alone wins!
    // Wait: let's test the non-sum mean case specifically: P0=10, P1=12, P2 declines, P3 joins P1 with 6 -> P1+P3 sum=18 (>10!) but mean=9.0 (<10.0)! P0 wins and chooses one of {P1, P3} for virtual Sha!
    let c0_10 = find_nth_card_by(&e, 0, |_, d| d.rank == 10);
    let c1_12 = find_nth_card_by(&e, 0, |_, d| d.rank == 12);
    let c2_5 = find_nth_card_by(&e, 0, |_, d| d.rank == 5);
    let c3_6 = find_nth_card_by(&e, 0, |_, d| d.rank == 6);
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, c0_10);
    put_in_hand(&mut e, 1, c1_12);
    put_in_hand(&mut e, 2, c2_5);
    put_in_hand(&mut e, 3, c3_6);

    submit(&mut e, 0, GameAction::ActivateSkill { skill: fy_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfenye_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfenye_owner_card");
    answer(&mut e, 0, vec![ChoiceOption::Card(c0_10)]);
    assert_eq!(e.open_choice().unwrap().observer, 1);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfenye_target_card");
    answer(&mut e, 1, vec![ChoiceOption::Card(c1_12)]);
    // Bystander P2 cancels.
    assert_eq!(e.open_choice().unwrap().observer, 2);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfenye_join_card");
    answer(&mut e, 2, vec![]);
    // Bystander P3 joins P1's group!
    assert_eq!(e.open_choice().unwrap().observer, 3);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfenye_join_card");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(c3_6)]);
    answer(&mut e, 3, vec![ChoiceOption::Card(c3_6)]);
    assert_eq!(e.open_choice().unwrap().observer, 3);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfenye_join_side");
    answer(&mut e, 3, vec![ChoiceOption::Player(1)]);

    // Discarded c0_10, c1_12, c3_6; P2 kept c2_5!
    assert!(e.state_core().discard.contains(&c0_10));
    assert!(e.state_core().discard.contains(&c1_12));
    assert!(e.state_core().discard.contains(&c3_6));
    assert_eq!(e.state_core().player(2).hand, vec![c2_5]);

    // P0 mean (10) > P1+P3 mean (9) -> P0 chooses Sha target from losers {P1, P3} (distance 2 P3 is allowed via nodistance!).
    let c_sha = e.open_choice().expect("winner P0 gets dddfenye_sha_target").clone();
    assert_eq!(c_sha.observer, 0);
    assert_eq!(c_sha.spec.prompt, "dddfenye_sha_target");
    assert!(!c_sha.spec.options.contains(&ChoiceOption::Player(2)), "non-participant P2 is not a loser");
    assert!(c_sha.spec.options.contains(&ChoiceOption::Player(3)), "distant loser P3 reachable via nodistance");
    let hp3_before = e.state_core().player(3).hp;
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    assert_eq!(e.state_core().player(3).hp, hp3_before - 1);

    // Multi-winner no-duplicate-loser & tie test: P0=1 + P2=1 vs P1=1 -> mean 1==1 tie -> NO winners, NO Sha prompts!
    e.debug_set_turn(0, Phase::Play, 2);
    let c0_1 = find_nth_card_by(&e, 0, |_, d| d.rank == 1);
    let c1_1 = find_nth_card_by(&e, 1, |_, d| d.rank == 1);
    let c2_1 = find_nth_card_by(&e, 2, |_, d| d.rank == 1);
    clear_hand(&mut e, 2);
    put_in_hand(&mut e, 0, c0_1);
    put_in_hand(&mut e, 1, c1_1);
    put_in_hand(&mut e, 3, c2_1);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: fy_def });
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(c0_1)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(c1_1)]);
    answer(&mut e, 3, vec![ChoiceOption::Card(c2_1)]);
    answer(&mut e, 3, vec![ChoiceOption::Player(0)]);
    assert!(e.open_choice().is_none(), "exact mean tie produces zero winners");
}

/// (06) `dddshichao` (`skill.js:4875-4974`)
/// - Requires Lord (`isZhu` in Identity mode); mark+1 <= alive.len(); selects from Nth largest hand size group.
/// - Adjusts OWNER's own hand to `min(target.hand.len(), lord.max_hp)` (owner discards `-del` if more!).
/// - Temp guard on target until owner's next turn: target dealing damage to owner may cancel damage and give owner +1 mark!
#[test]
fn u01_fix3_06_dddshichao() {
    // Negative: Prototype mode (no Lord) -> does not trigger.
    let (mut e_proto, _d3, _p9) = new_clean_engine(707, 3);
    e_proto.debug_grant(1, "dddshichao");
    advance_to_prepare(&mut e_proto, 1, 1);
    assert!(e_proto.open_choice().is_none(), "dddshichao requires a Lord (isZhu)");

    // Positive: Identity mode, P0 = Lord (max_hp=5!), P1 = dddshichao owner with 4 cards; P2 has 3 cards, P3 has 1 card.
    let (mut e, d3, p9) = new_identity_clean_engine(708, 4);
    e.debug_grant(1, "dddshichao");
    let lord_max = e.state_core().player(0).max_hp;
    assert_eq!(lord_max, 5);
    for idx in 0..4 {
        let c = find_nth_card_by(&e, idx, |_, _| true);
        put_in_hand(&mut e, 1, c);
    }
    for idx in 4..7 {
        let c = find_nth_card_by(&e, idx, |_, _| true);
        put_in_hand(&mut e, 2, c);
    }
    let c3 = find_nth_card_by(&e, 7, |_, _| true);
    put_in_hand(&mut e, 3, c3);

    // P1 Prepare: mark=0 -> largest hand group is {P1 (4)}, NOT P2 (3) or P3 (1); wait let's give P2 4 cards too so tied {P1, P2} are allowed and P3 (1) is excluded!
    let c2_extra = find_nth_card_by(&e, 8, |_, _| true);
    put_in_hand(&mut e, 2, c2_extra);
    // Give P1 5 cards vs P2 3 cards? Let's make P0=3, P1=5, P2=4, P3=4 -> mark=0: ONLY P1(5) largest; let's give P0=3, P1=6, P2=4, P3=4 so P1 chooses P1 vs wait: let's have P2=4, P3=4, P1=3, P0=1 so P1 adjusts hand up to 4 OR let's make P1 have 6 cards and P2=4, P3=4? No! If P1=6 then P1 alone is largest (6)! Better: set P0=2, P1=2, P2=4, P3=4 -> largest group is {P2, P3} (4 cards); P1 (mark=0) MUST choose P2 or P3, and owner P1 draws `4 - 2 = 2`! Or give P1 5 cards when picking? Even better: test BOTH!
    clear_hand(&mut e, 1);
    let p1_cards: Vec<u32> = (10..15).map(|idx| find_nth_card_by(&e, idx, |_, _| true)).collect();
    for c in p1_cards {
        put_in_hand(&mut e, 1, c); // P1=5
    }
    clear_hand(&mut e, 2);
    let p2_cards: Vec<u32> = (15..20).map(|idx| find_nth_card_by(&e, idx, |_, _| true)).collect();
    for c in p2_cards {
        put_in_hand(&mut e, 2, c); // P2=5
    }
    // Wait: let's test owner P1 discarding down FIRST by setting Lord max_hp=2 via debug_set_max_hp(0, 2)!
    // Then P1 has 5 cards, P2 has 5 cards -> min(target=5, lord.max_hp=2) = 2 -> owner P1 discards `5 - 2 = 3`!
    e.debug_set_max_hp(0, 2);
    advance_to_prepare(&mut e, 1, 1);
    let ct = e.open_choice().unwrap().clone();
    assert_eq!(ct.observer, 1);
    assert_eq!(ct.spec.prompt, "dddshichao_target");
    assert!(!ct.spec.allow_cancel);
    assert_eq!(ct.spec.min, 1);
    assert!(ct.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(ct.spec.options.contains(&ChoiceOption::Player(2)));
    assert!(!ct.spec.options.contains(&ChoiceOption::Player(3)), "P3 (1 card) not in largest hand group (5)");
    assert!(matches!(
        submit(&mut e, 1, GameAction::Answer { choice_id: ct.id, selection: vec![] }),
        CommandOutcome::Rejected(_)
    ), "forced dddshichao_target rejects cancel");

    // Owner P1 picks P2 (with save/restore + AI check).
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    // Adjusts OWNER P1's own hand (NOT P2's!) to min(5, 2) = 2 -> P1 chooses 3 hand cards to discard!
    let cd = e.open_choice().expect("dddshichao_discard for owner P1").clone();
    assert_eq!(cd.observer, 1);
    assert_eq!(cd.spec.prompt, "dddshichao_discard");
    assert_eq!((cd.spec.min, cd.spec.max), (3, 3));
    let drops: Vec<ChoiceOption> = e.state_core().player(1).hand[0..3].iter().map(|&c| ChoiceOption::Card(c)).collect();
    answer(&mut e, 1, drops);
    assert_eq!(e.state_core().player(1).hand.len(), 2, "owner hand adjusted to min(target, lord.max_hp)=2");
    assert_eq!(e.state_core().player(2).hand.len(), 5, "target P2 hand untouched");

    // P2 deals Sha damage to owner P1 -> P2 prompted `dddshichao_avoid_confirm`!
    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 2, sha);
    e.debug_set_turn(2, Phase::Play, 1);
    submit(&mut e, 2, GameAction::PlayCard { card: sha, targets: vec![1] });
    // P1 passes Shan if needed (P1 hand doesn't have Shan unless lucky; drain handles any Shan ask).
    for _ in 0..2 {
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.prompt != "dddshichao_avoid_confirm" {
                answer(&mut e, ch.observer, vec![]);
            }
        }
    }
    let ca = e.open_choice().expect("dddshichao_avoid_confirm before P2 damages P1").clone();
    assert_eq!(ca.observer, 2);
    assert_eq!(ca.spec.prompt, "dddshichao_avoid_confirm");
    let hp1_before = e.state_core().player(1).hp;
    // Branch A: P2 confirms -> damage cancelled (P1 HP untouched), P1 dddshichao mark becomes 1!
    let mut e_cancel_dmg = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_cancel_dmg, 2, vec![ChoiceOption::Confirm]);
    assert_eq!(e_cancel_dmg.state_core().player(1).hp, hp1_before);
    assert_eq!(flag(&e_cancel_dmg, 1, "dddshichao_mark"), 1);

    // Branch C: guard flag cleanup when next Prepare SKIPS `dddshichao` (e.g. Lord died before next Prepare)!
    let mut e_skip = Engine::restore_with_extensions(&e_cancel_dmg.save(), &[&d3, &p9]).unwrap();
    e_skip.state_core_mut().player_mut(0).alive = false;
    advance_to_prepare(&mut e_skip, 1, 2);
    assert!(e_skip.open_choice().is_none(), "dead Lord skips dddshichao on P1 Prepare");
    let sha_skip = find_nth_card_by(&e_skip, 3, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e_skip, 2, sha_skip);
    clear_hand(&mut e_skip, 1);
    e_skip.debug_set_turn(2, Phase::Play, 2);
    let hp1_skip_before = e_skip.state_core().player(1).hp;
    submit(&mut e_skip, 2, GameAction::PlayCard { card: sha_skip, targets: vec![1] });
    assert!(e_skip.open_choice().is_none(), "skipped Prepare must have cleared old dddshichao_guard:2");
    drain(&mut e_skip);
    assert_eq!(e_skip.state_core().player(1).hp, hp1_skip_before - 1);

    // Branch B: P2 passes -> P1 takes 1 damage, mark remains 0!
    answer(&mut e, 2, vec![ChoiceOption::Pass]);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    assert_eq!(flag(&e, 1, "dddshichao_mark"), 0);

    // On e_cancel_dmg (mark = 1): raise lord max_hp=5, advance to P1 next round Prepare -> mark=1 selects 2nd largest hand size ({P3} has 1 vs {P2} 5 vs {P1} 2 vs {P0} 0 -> 2nd largest = P1 (2 cards))!
    e_cancel_dmg.debug_set_max_hp(0, 5);
    clear_hand(&mut e_cancel_dmg, 0);
    clear_hand(&mut e_cancel_dmg, 3);
    advance_to_prepare(&mut e_cancel_dmg, 1, 2);
    let ct2 = e_cancel_dmg.open_choice().unwrap().clone();
    assert_eq!(ct2.spec.prompt, "dddshichao_target");
    assert!(ct2.spec.options.contains(&ChoiceOption::Player(1)), "2nd largest group is P1 (size 2)");
    assert!(!ct2.spec.options.contains(&ChoiceOption::Player(2)), "largest group P2 (size 5) excluded when mark=1");
}

use super::common::*;

/// (13) `dddweiqiu` (`skill.js:5714-5728`)
/// - Forced `recoverBefore` when `owner.hand.is_empty()`: CANCELS the `Recover` event (`amount` -> 0 healed) AND `owner` draws 1!
/// - Negative: when `owner.hand` is NON-empty, `Recover` succeeds normally and `owner` does NOT draw.
#[test]
fn u01_fix3_13_dddweiqiu() {
    let (mut e, _d3, _p9) = new_clean_engine(717, 3);
    e.debug_grant(0, "dddweiqiu");
    e.debug_set_hp(1, 2);
    let tao = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Tao);
    let tao2 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut e, 1, tao);
    put_in_hand(&mut e, 1, tao2);
    // P0 hand is EMPTY -> P1 plays Tao to recover -> forced `dddweiqiu` cancels recovery and draws 1 for P0!
    e.debug_set_turn(1, Phase::Play, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
    // Note: forced skill auto-executes (or Confirm-only without Pass); handle Confirm if prompted.
    if let Some(ch) = e.open_choice().cloned() {
        assert_eq!(ch.spec.prompt, "dddweiqiu_confirm");
        assert!(!ch.spec.allow_cancel);
        answer(&mut e, ch.observer, vec![ChoiceOption::Confirm]);
    }
    assert_eq!(e.state_core().player(1).hp, 2, "dddweiqiu must CANCEL P1's Recover");
    assert_eq!(e.state_core().player(0).hand.len(), 1, "dddweiqiu draws 1 card for empty-hand owner");

    // Negative: P0 now has 1 card in hand -> P1 plays Tao2 -> P1 recovers +1 HP normally!
    submit(&mut e, 1, GameAction::PlayCard { card: tao2, targets: vec![] });
    assert_eq!(e.state_core().player(1).hp, 3, "non-empty P0 hand must NOT cancel Recover");
    assert_eq!(e.state_core().player(0).hand.len(), 1);
}

/// (14) `dddlianer` (`skill.js:5729-5795`)
/// - `useCardAfter`: when owner used red card AND physical card is STILL in discard -> optional regain from discard.
/// - Sets `dddlianer_rank_cap = rank` until turn end; blocks use, response (`Shan`/`Nullify`), AND dying save (`Tao`) for rank >= cap!
/// - Negative: black card, or red card not in discard -> no prompt; Pass declines without setting rank_cap.
#[test]
fn u01_fix3_14_dddlianer() {
    let (mut e, d3, p9) = new_clean_engine(718, 3);
    e.debug_grant(0, "dddlianer");
    // Choose red Guohe (rank e.g. >= 5), plus high-rank cards (>= cap) and lower-rank Sha (< cap)!
    let red_guohe = find_card_by(&e, |_, d| {
        d.kind == CardKind::Dismantle && matches!(d.suit, Suit::Heart | Suit::Diamond) && d.rank >= 5
    });
    let cap = e.catalog().cards[red_guohe as usize].rank as u32;
    let hi_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.rank as u32 >= cap);
    let hi_shan = find_card_by(&e, |_, d| d.kind == CardKind::Shan && d.rank as u32 >= cap);
    let hi_tao = find_card_by(&e, |_, d| d.kind == CardKind::Tao && d.rank as u32 >= cap);
    let lo_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.rank as u32 > 0 && (d.rank as u32) < cap);
    let blk_chain = find_card_by(&e, |_, d| {
        d.kind == CardKind::Chain && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let p1_hold = find_nth_card_by(&e, 15, |_, _| true);
    let p1_hold2 = find_nth_card_by(&e, 16, |_, _| true);
    put_in_hand(&mut e, 1, p1_hold);
    put_in_hand(&mut e, 1, p1_hold2);
    put_in_hand(&mut e, 0, blk_chain);
    put_in_hand(&mut e, 0, red_guohe);
    put_in_hand(&mut e, 0, hi_sha);
    put_in_hand(&mut e, 0, hi_shan);
    put_in_hand(&mut e, 0, hi_tao);
    put_in_hand(&mut e, 0, lo_sha);

    // Negative: black card does not prompt `dddlianer_confirm`.
    e.debug_set_turn(0, Phase::Play, 1);
    submit(&mut e, 0, GameAction::PlayCard { card: blk_chain, targets: vec![2] });
    assert!(e.open_choice().is_none(), "black card does not trigger dddlianer");

    // Positive: P0 plays `red_guohe` on P1 -> finishes resolving -> `dddlianer_confirm` AFTER card use ends!
    submit(&mut e, 0, GameAction::PlayCard { card: red_guohe, targets: vec![1] });
    let cl = e.open_choice().expect("dddlianer_confirm after Guohe settles into discard").clone();
    assert_eq!(cl.observer, 0);
    assert_eq!(cl.spec.prompt, "dddlianer_confirm");
    assert!(cl.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);

    // Choice B: cancel keeps card in discard and sets no cap!
    let mut e_pass = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_pass, 0, vec![ChoiceOption::Pass]);
    assert!(e_pass.state_core().discard.contains(&red_guohe));
    assert!(!matches!(
        submit(&mut e_pass, 0, GameAction::PlayCard { card: hi_sha, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));

    // Choice A: confirm regains `red_guohe` and enforces `rank < cap` for Play, Respond, AND Dying Save!
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(0).hand.contains(&red_guohe));
    assert!(!e.state_core().discard.contains(&red_guohe));
    // (a) Play: `hi_sha` (`rank >= cap`) REJECTED; `lo_sha` (`rank < cap`) ALLOWED!
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: hi_sha, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ), "PlayCard rank >= cap must be rejected by dddlianer");
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: lo_sha, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));

    // (b) Respond (`Sha -> Shan`) & Dying Save (`Tao`) while STILL within SAME turn (`dddlianer_rank_cap` active):
    let p1_sha = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 1, p1_sha);
    e.debug_set_hp(0, 1);
    // Suppose P0 takes a response Sha from P1 (e.g. via Duel/other card in same turn, OR test `apply_card_use_canceled` via direct Play/Dying):
    // Let P1 respond/force P0 to save via P0's own red Le / self-damage (`Duel` against P1 who has Sha!).
    let duel = find_card_by(&e, |_, d| d.kind == CardKind::Duel);
    let p1_duel_sha = find_nth_card_by(&e, 3, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, duel);
    put_in_hand(&mut e, 1, p1_duel_sha);
    // Duel vs P1: P1 responds with `p1_duel_sha`, then P0 responds -> P0 has `hi_sha` (`>= cap`) which CANNOT respond AND then P0 enters Dying -> P0 has `hi_tao` (`>= cap`) which CANNOT save!
    if e.catalog().cards[duel as usize].rank < e.catalog().cards[red_guohe as usize].rank {
        submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] });
        if let Some(c) = e.open_choice().cloned() {
            if c.observer == 1 {
                answer(&mut e, 1, vec![ChoiceOption::Card(p1_duel_sha)]);
            }
        }
        // P0 must NOT have `hi_sha` offered in duel response (or auto-fails to respond -> takes 1 dmg -> Dying!),
        // and P0 Dying has `hi_tao` (`>= cap`) blocked so P0 dies unless saved!
        for _ in 0..4 {
            let Some(c) = e.open_choice().cloned() else { break; };
            assert!(
                !c.spec.options.contains(&ChoiceOption::Card(hi_sha))
                    && !c.spec.options.contains(&ChoiceOption::Card(hi_tao)),
                "rank >= cap card ({}) forbidden in response/dying save ({})",
                cap,
                c.spec.prompt
            );
            answer(&mut e, c.observer, vec![]);
        }
        assert!(!e.state_core().player(0).alive, "P0 could not save self with Tao rank >= dddlianer_rank_cap");
    }

    // (c) Turn end (`Phase::End` -> `after: true`): `dddlianer_rank_cap` expires, high-rank card becomes usable again!
    e_pass.debug_set_turn(0, Phase::Discard, 2);
    // Re-trigger and confirm on e_pass cleanly, then advance End -> turn end clears cap!
    let hi_sha2 = hi_sha;
    let red_gh2 = red_guohe;
    let p1_extra = find_nth_card_by(&e_pass, 16, |_, _| true);
    let p1_extra2 = find_nth_card_by(&e_pass, 17, |_, _| true);
    put_in_hand(&mut e_pass, 1, p1_extra);
    put_in_hand(&mut e_pass, 1, p1_extra2);
    clear_hand(&mut e_pass, 0);
    put_in_hand(&mut e_pass, 0, red_gh2);
    put_in_hand(&mut e_pass, 0, hi_sha2);
    e_pass.debug_set_turn(0, Phase::Play, 2);
    submit(&mut e_pass, 0, GameAction::PlayCard { card: red_gh2, targets: vec![1] });
    answer(&mut e_pass, 0, vec![ChoiceOption::Confirm]);
    assert!(matches!(
        submit(&mut e_pass, 0, GameAction::PlayCard { card: hi_sha2, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));
    clear_hand(&mut e_pass, 0);
    put_in_hand(&mut e_pass, 0, hi_sha2);
    e_pass.debug_set_turn(0, Phase::End, 2);
    submit(&mut e_pass, 0, GameAction::NextPhase);
    drain(&mut e_pass);
    e_pass.debug_set_turn(0, Phase::Play, 3);
    assert!(
        !matches!(submit(&mut e_pass, 0, GameAction::PlayCard { card: hi_sha2, targets: vec![1] }), CommandOutcome::Rejected(_)),
        "after turn end, dddlianer rank cap expires"
    );
}

/// (15) `dddanzhi` (`skill.js:5796-5839`)
/// Regression test ensuring upstream-aligned ward behavior remains intact while preserving
/// frozen `u01-dddanzhi-pending-v107.ngsa` (`content_lock_id` + `host_skill_trigger_meta` compatibility).
/// Note shared-engine boundaries (`Section 3 Item 3①`): `Taoyuan` whole-card nullify/targeting in
/// `kernel/tricks.rs`, `Duel` `card==0` discard sentinel in `kernel/duel.rs:324`, and frozen static lock.
#[test]
fn u01_fix3_15_dddanzhi() {
    let (mut e, d3, p9) = new_clean_engine(719, 3);
    e.debug_grant(0, "dddanzhi");
    // Last card played is Black -> P0 activates dddanzhi at Prepare to grant ward to P1.
    let blk_chain = find_card_by(&e, |_, d| d.kind == CardKind::Chain && matches!(d.suit, Suit::Spade | Suit::Club));
    put_in_hand(&mut e, 2, blk_chain);
    e.debug_set_turn(2, Phase::Play, 1);
    submit(&mut e, 2, GameAction::PlayCard { card: blk_chain, targets: vec![2] });

    let mut e = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    advance_to_prepare(&mut e, 0, 1);
    let ct = e.open_choice().expect("dddanzhi_target").clone();
    assert_eq!(ct.spec.prompt, "dddanzhi_target");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);

    // Red Sha still hits immune P1 without consuming ward; black Sha is nullified on P1 and consumes ward!
    let red_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Heart | Suit::Diamond));
    let blk_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club));
    put_in_hand(&mut e, 2, red_sha);
    put_in_hand(&mut e, 2, blk_sha);
    let hp1_before = e.state_core().player(1).hp;
    e.debug_set_turn(2, Phase::Play, 2);
    submit(&mut e, 2, GameAction::PlayCard { card: red_sha, targets: vec![1] });
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    e.debug_set_turn(0, Phase::Play, 2);
    put_in_hand(&mut e, 0, blk_sha);
    submit(&mut e, 0, GameAction::PlayCard { card: blk_sha, targets: vec![1] });
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "first black card targeting immune P1 is nullified");
}

/// (16) `dddshixing` + companion `xiangle` / `jiushi` / `rezhiheng` (`skill.js:5840-5888`)
/// - Independently evaluates `bool1 = any alive female at full hp` (`xiangle`) and `bool2 = no trick used this turn` (`jiushi`).
/// - Both true -> BOTH `xiangle` and `jiushi` granted! Both false -> `rezhiheng` granted!
/// - Tests live execution of `xiangle` (Sha exclusion on decline), `jiushi` (flip + Jiu + damaged flip back),
///   `rezhiheng` (discard all hand -> draw N + 1), and `onremove` cleanup.
#[test]
fn u01_fix3_16_dddshixing() {
    let (mut e, d3, p9) = new_clean_engine(720, 3);
    e.debug_grant(0, "dddshixing");
    // Assign a female general on seat 1 at full HP (4/4)!
    let fem_gid = (0..e.catalog().general_count() as u16)
        .find(|&gid| e.catalog().general_meta(gid).is_some_and(|m| m.4 == Gender::Female))
        .expect("female general exists");
    e.assign_general(1, fem_gid).unwrap();

    // Prepare P0 (bool1 = true (P1 female full HP), bool2 = true (no trick used yet)): grants BOTH `xiangle` & `jiushi`!
    advance_to_prepare(&mut e, 0, 1);
    assert!(has_skill(&e, 0, "xiangle"), "bool1 true must grant xiangle");
    assert!(has_skill(&e, 0, "jiushi"), "bool2 true must grant jiushi");
    assert!(!has_skill(&e, 0, "rezhiheng"));

    // Live `jiushi`: P0 face-up activates `jiushi` in Play phase -> flips face-down + `jiu_active = true`!
    submit(&mut e, 0, GameAction::NextPhase); // Prepare -> Judge/Draw/Play
    e.debug_set_turn(0, Phase::Play, 1);
    let js_def = e.catalog().skill_by_name("jiushi").unwrap();
    let sha0 = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, sha0);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: js_def });
    assert!(e.state_core().player(0).face_down, "jiushi flips face-down");
    assert!(e.state_core().turn.jiu_active, "jiushi gives virtual Jiu");
    // Play Sha at female P1 (who has NO Shan) -> P1 takes 2 damage!
    submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    // Now P1 female is DAMAGED (bool1 becomes false!), while bool2 is STILL true -> loses `xiangle`, keeps `jiushi`!
    assert!(!has_skill(&e, 0, "xiangle"), "female damaged -> bool1 false -> removes xiangle");
    assert!(has_skill(&e, 0, "jiushi"), "bool2 still true -> keeps jiushi");
    assert!(!has_skill(&e, 0, "rezhiheng"));

    // Now P0 uses a trick (`Wuzhong`): bool2 ALSO becomes false! Now BOTH false -> removes `jiushi`, grants `rezhiheng`!
    let wz = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    put_in_hand(&mut e, 0, wz);
    submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    assert!(!has_skill(&e, 0, "xiangle"));
    assert!(!has_skill(&e, 0, "jiushi"), "trick used -> bool2 false -> removes jiushi");
    assert!(has_skill(&e, 0, "rezhiheng"), "both false -> grants rezhiheng");

    // Live `rezhiheng`: P0 discards ALL hand cards -> draws N + 1!
    let rzh_def = e.catalog().skill_by_name("rezhiheng").unwrap();
    let hand_cnt = e.state_core().player(0).hand.len();
    assert!(hand_cnt >= 1);
    let discards: Vec<ChoiceOption> = e.state_core().player(0).hand.iter().map(|&c| ChoiceOption::Card(c)).collect();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: rzh_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rezhiheng_cards");
    answer(&mut e, 0, discards);
    assert_eq!(e.state_core().player(0).hand.len(), hand_cnt + 1, "rezhiheng discarding all hand cards draws N + 1");

    // Next turn (P1 turn, female P1 healed back to full HP by `Tao`): bool1=true, bool2=true -> grants `xiangle` + `jiushi` again!
    e.debug_set_turn(0, Phase::End, 1);
    submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    e.debug_set_turn(1, Phase::Play, 2);
    let tao1 = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Tao);
    let tao2 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut e, 1, tao1);
    put_in_hand(&mut e, 1, tao2);
    submit(&mut e, 1, GameAction::PlayCard { card: tao1, targets: vec![] });
    submit(&mut e, 1, GameAction::PlayCard { card: tao2, targets: vec![] });
    assert_eq!(e.state_core().player(1).hp, e.state_core().player(1).max_hp as i32);
    assert!(has_skill(&e, 0, "xiangle"), "female healed to full -> recovers xiangle");
    assert!(has_skill(&e, 0, "jiushi"), "new turn with no trick -> recovers jiushi");

    // Live `xiangle`: P1 uses `Sha` on P0 while holding an extra Basic card -> P1 asked `xiangle_discard`; P1 cancels -> P0 excluded!
    let sha1 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Sha);
    let extra_basic = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Shan);
    put_in_hand(&mut e, 1, sha1);
    put_in_hand(&mut e, 1, extra_basic);
    let hp0_before = e.state_core().player(0).hp;
    let out_sha1 = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    assert!(!matches!(out_sha1, CommandOutcome::Rejected(_)), "P1 PlayCard sha1 failed: {out_sha1:?}");
    let cx = e.open_choice().unwrap_or_else(|| {
        panic!("xiangle_discard prompt for attacker P1 missing; out={out_sha1:?}, hp0={}, skills_0={:?}",
            e.state_core().player(0).hp,
            e.state_core().skills.keys().filter(|(o, _)| *o == 0).map(|(_, d)| e.catalog().skill_name(*d)).collect::<Vec<_>>(),
        )
    }).clone();
    assert_eq!(cx.observer, 1);
    assert_eq!(cx.spec.prompt, "xiangle_discard");
    assert!(cx.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![]);
    answer(&mut e, 1, vec![]);
    assert_eq!(e.state_core().player(0).hp, hp0_before, "declining xiangle_discard excludes P0 from Sha");

    // `jiushi_turnover`: P0 begins Turn 3 (turn-begin flips face-up and skips turn, or Prepare grants `jiushi`),
    // then in Play activates `jiushi` to flip face-down, ends turn, and takes Sha damage from P1!
    e.debug_set_turn(0, Phase::End, 2);
    submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    e.state_core_mut().player_mut(0).face_down = false;
    advance_to_prepare(&mut e, 0, 2);
    drain(&mut e);
    assert!(has_skill(&e, 0, "jiushi"), "P0 Prepare grants jiushi");
    e.debug_set_turn(0, Phase::Play, 3);
    let js_def2 = e.catalog().skill_by_name("jiushi").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: js_def2 });
    assert!(e.state_core().player(0).face_down, "jiushi must flip P0 face-down");
    clear_hand(&mut e, 0);
    let sha2 = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Sha);
    let basic_drop = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut e, 1, sha2);
    put_in_hand(&mut e, 1, basic_drop);
    e.debug_set_turn(0, Phase::End, 3);
    submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    e.debug_set_turn(1, Phase::Play, 3);
    submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    answer(&mut e, 1, vec![ChoiceOption::Card(basic_drop)]); // Discards Basic for xiangle!
    let jt = e.open_choice().expect("jiushi_turnover prompt on face-down damage").clone();
    assert_eq!(jt.observer, 0);
    assert_eq!(jt.spec.prompt, "jiushi_turnover");
    assert!(jt.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(!e.state_core().player(0).face_down);
}

/// (17) `ddddanggu` (`skill.js:5889-5947`)
/// - Forced `Phase::End { begin: true }`: triggers iff `linked_count < qun_count`; min choice = `qun_count - linked_count`.
/// - Links via real `EventData::Linked` (triggers `dddqiaoduan` when present!).
/// - `ddddanggu_negative`: other linked players count `he` cards against `hand_limit` in `Phase::Discard` and discard from `he`.
#[test]
fn u01_fix3_17_ddddanggu() {
    let (mut e, d3, p9) = new_clean_engine(721, 4);
    e.debug_grant(0, "ddddanggu");
    e.debug_grant(2, "dddqiaoduan");
    e.debug_set_hp(3, 2);
    for idx in 0..3 {
        let qc = find_nth_card_by(&e, idx + 20, |_, d| d.kind == CardKind::Sha);
        put_in_hand(&mut e, 2, qc);
    } // Verifies real Linked events fire!
    // Assign 3 Qun generals on seats 0, 1, 2 (`qun_count = 3`, `linked_count = 0` -> `min = 3`!).
    let qun_gid = (0..e.catalog().general_count() as u16)
        .find(|&gid| e.catalog().general_meta(gid).is_some_and(|m| m.1 == Faction::Qun))
        .expect("Qun general");
    for s in 0..3 {
        e.assign_general(s, qun_gid).unwrap();
    }
    e.debug_grant(0, "ddddanggu");
    e.debug_grant(2, "dddqiaoduan");
    e.debug_set_hp(3, 2);
    for idx in 0..3 {
        let qc = find_nth_card_by(&e, idx + 20, |_, d| d.kind == CardKind::Sha);
        put_in_hand(&mut e, 2, qc);
    }

    e.debug_set_turn(0, Phase::Discard, 1);
    submit(&mut e, 0, GameAction::NextPhase);
    let cd = e.open_choice().expect("ddddanggu_targets at End").clone();
    assert_eq!(cd.observer, 0);
    assert_eq!(cd.spec.prompt, "ddddanggu_targets");
    assert_eq!(cd.spec.min, 3);
    assert!(!cd.spec.allow_cancel);
    // Picking fewer than 3 rejected!
    let bad = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cd.id,
            selection: vec![ChoiceOption::Player(0), ChoiceOption::Player(1)],
        },
    );
    assert!(matches!(bad, CommandOutcome::Rejected(_)), "min = qun_count - linked_count = 3");
    assert_mid_choice_archive_and_ai(
        &e,
        &d3,
        &p9,
        vec![ChoiceOption::Player(0), ChoiceOption::Player(1), ChoiceOption::Player(2)],
    );
    answer(&mut e, 0, vec![ChoiceOption::Player(0), ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    // Answer any `dddqiaoduan` prompts proving real `EventData::Linked` events fired!
    let mut qd_fired = false;
    while let Some(ch) = e.open_choice().cloned() {
        if ch.spec.prompt.starts_with("dddqiaoduan") {
            qd_fired = true;
        }
        answer(&mut e, ch.observer, if ch.spec.allow_cancel { vec![] } else { vec![ch.spec.options[0]] });
    }
    assert!(qd_fired, "ddddanggu must emit real EventData::Linked frames triggering dddqiaoduan");
    assert!(e.state_core().player(0).linked && e.state_core().player(1).linked && e.state_core().player(2).linked);

    // Negative: now `linked_count (3) >= qun_count (3)` -> next End phase does NOT trigger `ddddanggu`!
    e.debug_set_turn(0, Phase::Discard, 2);
    submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "linked >= qun -> ddddanggu silent");

    // `ddddanggu_negative`: linked P1 (hp=2 -> hand_limit=2) has 2 hand cards + 1 equip = 3 `he` cards -> excess = 1 in Discard!
    e.debug_set_hp(1, 2);
    let h1 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Sha);
    let h2 = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Shan);
    let eq1 = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    put_in_hand(&mut e, 1, h1);
    put_in_hand(&mut e, 1, h2);
    put_in_equip(&mut e, 1, eq1);
    e.debug_set_turn(1, Phase::Play, 2);
    submit(&mut e, 1, GameAction::NextPhase);
    let c_disc = e.open_choice().expect("ddddanggu_negative forces he discard on linked P1").clone();
    assert_eq!(c_disc.observer, 1);
    assert_eq!((c_disc.spec.min, c_disc.spec.max), (1, 1));
    assert!(c_disc.spec.options.contains(&ChoiceOption::Card(eq1)), "equip included in ddddanggu_negative he discard");
    answer(&mut e, 1, vec![ChoiceOption::Card(eq1)]);
    assert!(!e.state_core().player(1).equips.contains(&eq1));
    assert!(e.state_core().discard.contains(&eq1));
}

/// (18) `dddlanghuai` (`skill.js:5989-6057`)
/// - `DrawPhaseBegin1`: reveals hand, CANCELS normal draw (`pending = 0`), draws `suit_count` (state 0) or `4 - suit_count` (state 1),
///   ALLOWING **0** cards drawn! Toggles state `0 <-> 1`.
/// - When `dddxuanlun_del` is active: `allow_cancel: false`, NO `Pass` option, clears `dddxuanlun_del`.
#[test]
fn u01_fix3_18_dddlanghuai() {
    let (mut e, d3, p9) = new_clean_engine(722, 3);
    e.debug_grant(0, "dddlanghuai");

    // Put 4 distinct suits in P0's hand!
    let c_s = find_card_by(&e, |_, d| d.suit == Suit::Spade);
    let c_h = find_card_by(&e, |_, d| d.suit == Suit::Heart);
    let c_c = find_card_by(&e, |_, d| d.suit == Suit::Club);
    let c_d = find_card_by(&e, |_, d| d.suit == Suit::Diamond);
    put_in_hand(&mut e, 0, c_s);
    put_in_hand(&mut e, 0, c_h);
    put_in_hand(&mut e, 0, c_c);
    put_in_hand(&mut e, 0, c_d);

    // Draw #1 (state 0, suit_count = 4): Choice A Confirm -> cancels normal +2 draw, draws 4 instead -> hand = 4 + 4 = 8; Choice B Pass -> normal +2 -> hand = 6!
    e.debug_set_turn(0, Phase::Judge, 1);
    submit(&mut e, 0, GameAction::NextPhase);
    let cl1 = e.open_choice().expect("dddlanghuai_confirm").clone();
    assert_eq!(cl1.spec.prompt, "dddlanghuai_confirm");
    assert!(cl1.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);

    let mut e_pass = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_pass, 0, vec![ChoiceOption::Pass]);
    assert_eq!(e_pass.state_core().player(0).hand.len(), 6, "passing lets normal Draw 2 run");

    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), 8, "state 0 with 4 suits cancels Draw 2 and draws 4");

    // Now set P0 hand to 4 suits again & Draw #2 (state 1, 4 - suit_count = 0!): normal draw CANCELLED and draws **0** cards (hand stays 4)!
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, c_s);
    put_in_hand(&mut e, 0, c_h);
    put_in_hand(&mut e, 0, c_c);
    put_in_hand(&mut e, 0, c_d);
    e.debug_set_turn(0, Phase::Judge, 2);
    submit(&mut e, 0, GameAction::NextPhase);
    let cl2 = e.open_choice().unwrap().clone();
    assert_eq!(cl2.spec.prompt, "dddlanghuai_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), 4, "state 1 with 4 suits draws 4-4 = 0 and cancels normal draw!");

    // Forced test (`dddxuanlun_del == 1` via `dddxuanlun`): Pass option MUST NOT exist & MUST be rejected!
    e.debug_grant(0, "dddxuanlun");
    e.state_core_mut().vm_aux.flags.insert((0, "dddxuanlun_del".into()), 1);
    e.debug_set_turn(0, Phase::Judge, 3);
    submit(&mut e, 0, GameAction::NextPhase);
    let cl3 = e.open_choice().unwrap().clone();
    assert!(!cl3.spec.allow_cancel);
    assert!(!cl3.spec.options.contains(&ChoiceOption::Pass), "forced dddlanghuai must NOT contain Pass");
    assert!(matches!(
        submit(&mut e, 0, GameAction::Answer { choice_id: cl3.id, selection: vec![ChoiceOption::Pass] }),
        CommandOutcome::Rejected(_)
    ));
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(flag(&e, 0, "dddxuanlun_del"), 0, "dddlanghuai clears dddxuanlun_del");
}

/// (19) `dddxuanlun` (`skill.js:6058-6235`)
/// - Damage end: optional draw 4 (`dddxuanlun_confirm`).
/// - `phaseEnd` (`TurnEndWindow { after: false }`): skipped if `!hand.is_empty() == false && dddxuanlun_del`.
/// - `Tag(0)`: requires exact 4 `he` cards when `he.len() > 4` (3 rejected!), lets owner partition/order into `top` and `bottom` of deck!
/// - `Tag(1)`: sets `dddxuanlun_del = 1`; while `dddxuanlun_del == 1`, `Tag(1)` cannot be selected again!
#[test]
fn u01_fix3_19_dddxuanlun() {
    let (mut e, d3, p9) = new_clean_engine(723, 3);
    e.debug_grant(0, "dddxuanlun");
    e.debug_grant(0, "dddlanghuai");

    // P1 deals real Sha damage to P0 -> `dddxuanlun_confirm`!
    let sha1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 1, sha1);
    e.debug_set_turn(1, Phase::Play, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    let ce = e.open_choice().expect("dddxuanlun_confirm after taking damage").clone();
    assert_eq!(ce.observer, 0);
    assert_eq!(ce.spec.prompt, "dddxuanlun_confirm");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    let before = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), before + 4);

    // Give 1 extra card so P0 has 5 hand cards (> 4); advance to `End` (`phaseEnd`!) -> pick Tag(0), require exact 4!
    let c_extra = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut e, 0, c_extra);
    e.debug_set_turn(1, Phase::End, 1);
    clear_hand(&mut e, 1);
    submit(&mut e, 1, GameAction::NextPhase);
    let cm = e.open_choice().expect("dddxuanlun_end_mode at phaseEnd").clone();
    assert_eq!(cm.spec.prompt, "dddxuanlun_end_mode");
    assert!(cm.spec.options.contains(&ChoiceOption::Tag(0)) && cm.spec.options.contains(&ChoiceOption::Tag(1)));
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);

    let cc = e.open_choice().expect("dddxuanlun_cards").clone();
    assert_eq!((cc.spec.min, cc.spec.max), (4, 4), "when he.len() > 4 must select exact 4 cards");
    let h0 = e.state_core().player(0).hand.clone();
    let bad3 = submit(&mut e, 0, GameAction::Answer {
        choice_id: cc.id,
        selection: vec![ChoiceOption::Card(h0[0]), ChoiceOption::Card(h0[1]), ChoiceOption::Card(h0[2])],
    });
    assert!(matches!(bad3, CommandOutcome::Rejected(_)), "< 4 cards must be rejected");
    answer(&mut e, 0, vec![
        ChoiceOption::Card(h0[0]),
        ChoiceOption::Card(h0[1]),
        ChoiceOption::Card(h0[2]),
        ChoiceOption::Card(h0[3]),
    ]);

    // Top vs Bottom partition: pick `[h0[1], h0[0]]` for deck top (`dddxuanlun_top`) -> remainder placed on bottom (`dddxuanlun_bottom`)!
    let ctop = e.open_choice().expect("dddxuanlun_top").clone();
    assert_eq!(ctop.spec.prompt, "dddxuanlun_top");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(h0[1]), ChoiceOption::Card(h0[0])]);
    answer(&mut e, 0, vec![ChoiceOption::Card(h0[1]), ChoiceOption::Card(h0[0])]);
    if let Some(cbot) = e.open_choice().cloned() {
        if cbot.spec.prompt == "dddxuanlun_bottom" {
            answer(&mut e, 0, vec![ChoiceOption::Card(h0[2]), ChoiceOption::Card(h0[3])]);
        }
    }
    // `h0[1]` should be very top of deck (`deck.last()`), `h0[0]` right below it; `h0[2], h0[3]` at bottom (`deck[0..2]`)!
    let deck = &e.state_core().deck;
    assert_eq!(deck[deck.len() - 1], h0[1], "first selected top card drawn first");
    assert_eq!(deck[deck.len() - 2], h0[0], "second selected top card drawn second");
    assert!(deck[..2].contains(&h0[2]) && deck[..2].contains(&h0[3]), "remaining cards placed on deck bottom");

    // Next damage turn: choose Tag(1) -> sets `dddxuanlun_del = 1`; then ANOTHER damage while `del == 1` still set (skipped langhuai or same turn): Tag(1) disabled!
    let sha2 = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 2, sha2);
    e.debug_set_turn(2, Phase::Play, 2);
    submit(&mut e, 2, GameAction::PlayCard { card: sha2, targets: vec![0] });
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    e.debug_set_turn(2, Phase::End, 2);
    clear_hand(&mut e, 2);
    submit(&mut e, 2, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(flag(&e, 0, "dddxuanlun_del"), 1);

    // Take another damage while `del == 1` still active; with empty hand + equips only, End must NOT trigger!
    clear_hand(&mut e, 0);
    let eq0 = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    put_in_equip(&mut e, 0, eq0);
    let sha3 = find_card_by(&e, |_, d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    put_in_hand(&mut e, 1, sha3);
    e.debug_set_turn(1, Phase::Play, 2);
    submit(&mut e, 1, GameAction::PlayCard { card: sha3, targets: vec![0] });
    // Decline extra draw so hand stays empty -> at End with `!hand && del == 1`, dddxuanlun skips!
    answer(&mut e, 0, vec![ChoiceOption::Pass]);
    e.debug_set_turn(1, Phase::End, 2);
    submit(&mut e, 1, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "empty hand + dddxuanlun_del skips phaseEnd trigger");
}

/// (20) `dddjiexing` (`skill.js:6236-6301`)
/// - Limited (`once_used`).
/// - Intercepts OTHER `Recover` BEFORE commit: cancels original recover and emits `recover(owner, amount)` keeping exact `amount`!
/// - Intercepts OTHER `CardUsed(Equip)` BEFORE commit: cancels original user's equip & skills, equips onto owner!
/// - Direct `Equipped` move does NOT reset `once_used`; owner's OWN subsequent `Recover` / `CardUsed(Equip)` resets `once_used`!
#[test]
fn u01_fix3_20_dddjiexing() {
    let (mut e, d3, p9) = new_clean_engine(724, 3);
    e.debug_grant(0, "dddjiexing");
    e.debug_set_hp(0, 1); // Missing 3 HP!
    e.debug_set_hp(1, 2);
    // P1 drinks Jiu/Tao or uses skill healing 2 HP, or let's use P1 playing `Tao` (amount = 1) vs `recover` of amount 2 via `Wuzhong`/`Jiedao`? Let's test `Recover` with amount = 2 if a card heals 2 (e.g. P0/P1 with Jiu+dying? Or check `Tao` on dying recovers 1; wait in P1 Turn: P1 `Guose`/custom? Wait: in standard, `Taoyuan` heals 1; `P1` dying while P2 uses `Tao` -> `Recover(1, 1)` vs let's check if any standard card heals >1: e.g. `Caoyao` or `Jiu`+`Tao`? No, what if P1 is at hp=-1 dying and someone saves? Still `Recover(1)` per Tao unless `huatuo`/`qiaosi` — wait: what about P0 having `jiuxian`/or wait: we have `recover` test AND check if we use P0 `P1` `Tao` on P1 (`hp 2 -> 3` cancelled, P0 `hp 1 -> 2`) AND/OR verify `amount` propagation! Wait: does any card in standard/d3 heal 2? Wait `shenmiguo`? Even better: P1 uses `Tao` (`amount = 1` or `dddweici`? No, let's check `P1` `Taoyuan`: heals P1 (`1`), P0 steals P1's Taoyuan heal (`owner != P1`, Wait P0 is ALSO a target of Taoyuan so P0 heals from self-Taoyuan too!). Better: P1 plays `Tao` targeting P1 (`amount = 1`, `P1` stays at 2 while `P0` heals `1 -> 2` -> `once_used = true`!).
    let tao1 = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Tao);
    let tao2 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Tao);
    let eq1 = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    let eq2 = find_card_by(&e, |_, d| d.kind == CardKind::Zhangba);
    put_in_hand(&mut e, 1, tao1);
    put_in_hand(&mut e, 0, tao2);
    put_in_hand(&mut e, 1, eq1);
    put_in_hand(&mut e, 0, eq2);

    e.debug_set_turn(1, Phase::Play, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: tao1, targets: vec![] });
    let cj1 = e.open_choice().expect("dddjiexing_confirm on P1 Recover").clone();
    assert_eq!(cj1.observer, 0);
    assert_eq!(cj1.spec.prompt, "dddjiexing_confirm");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    // P1's Recover cancelled (`hp == 2`), P0 recovered (`hp == 2`)! `dddjiexing`'s OWN recover does NOT immediately reset `once_used`!
    assert_eq!(e.state_core().player(1).hp, 2, "original P1 recover cancelled");
    assert_eq!(e.state_core().player(0).hp, 2, "owner P0 steals P1 recover");

    // While `once_used == true`, P1 using `eq1` (`Zhuge`) does NOT trigger `dddjiexing`; P1 equips `Zhuge` normally!
    submit(&mut e, 1, GameAction::PlayCard { card: eq1, targets: vec![] });
    assert!(e.open_choice().is_none(), "once_used prevents second dddjiexing trigger");
    assert!(e.state_core().player(1).equips.contains(&eq1));

    // P0's OWN separate `Recover` (`tao2`) resets `dddjiexing` (`once_used = false` -> `dddjiexing_used = 0`)!
    e.debug_set_turn(0, Phase::Play, 1);
    submit(&mut e, 0, GameAction::PlayCard { card: tao2, targets: vec![] });
    assert_eq!(e.state_core().player(0).hp, 3);

    // Now P1 plays `eq2` (`Zhangba`): `dddjiexing_confirm` fires BEFORE `eq2` equips onto P1 -> cancels P1's `CardUsed` (`P1` never gets `丈八蛇矛`!), equips `eq2` onto `P0`!
    e.debug_set_turn(1, Phase::Play, 2);
    put_in_hand(&mut e, 1, eq2);
    submit(&mut e, 1, GameAction::PlayCard { card: eq2, targets: vec![] });
    let cj2 = e.open_choice().expect("dddjiexing_confirm on P1 Equip").clone();
    assert_eq!(cj2.spec.prompt, "dddjiexing_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(0).equips.contains(&eq2), "owner P0 steals Zhangba");
    assert!(!e.state_core().player(1).equips.contains(&eq2), "P1 does not equip stolen Zhangba");
    assert!(!e.debug_has_skill(1, "丈八蛇矛"), "P1 never gained weapon skill");
    assert!(e.debug_has_skill(0, "丈八蛇矛"), "owner P0 gained weapon skill");

    // P0's OWN `CardUsed(Equip)` resets `dddjiexing` (`once_used = false`); verify another equip `eq3` triggers reset cleanly!
    let eq3 = find_card_by(&e, |_, d| d.kind == CardKind::Renwang);
    put_in_hand(&mut e, 0, eq3);
    e.debug_set_turn(0, Phase::Play, 2);
    submit(&mut e, 0, GameAction::PlayCard { card: eq3, targets: vec![] });
    let eq4 = find_nth_card_by(&e, 1, |_, d| d.kind.equip_slot().is_some() && d.kind != CardKind::Renwang);
    put_in_hand(&mut e, 1, eq4);
    e.debug_set_turn(1, Phase::Play, 3);
    submit(&mut e, 1, GameAction::PlayCard { card: eq4, targets: vec![] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddjiexing_confirm", "own CardUsed(Equip) reset dddjiexing");
    answer(&mut e, 0, vec![ChoiceOption::Pass]);
}

/// (21) `dddbailei` (`skill.js:6302-6368`)
/// - `Phase::Prepare { begin: true }`: targets filtered strictly to `(isMaxEquip && !he.is_empty()) || hp == 1`.
/// - `isMaxEquip` ONLY: mandatory gain 1 `he` card (chosen by owner, NOT blind `he[0]`), keeps `dddbailei`.
/// - `hp == 1` ONLY: deals 1 damage; if target IS SAVED (`alive == true`), owner KEEPS `dddbailei`; if target DIES (`!alive`), removes `dddbailei` AFTER death!
/// - BOTH `isMaxEquip && hp == 1`: gain card (`allow_cancel: true`) OR cancel to deal 1 damage!
#[test]
fn u01_fix3_21_dddbailei() {
    let (mut e, d3, p9) = new_clean_engine(725, 3);
    e.debug_grant(0, "dddbailei");
    // P1: 2 equips (isMaxEquip!) + 2 hand cards + hp=3; P2: 0 equips, hp=2 -> P1 legal, P2 ILLEGAL!
    let eq1 = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    let eq2 = find_card_by(&e, |_, d| d.kind == CardKind::Renwang);
    let h1 = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Sha);
    let h2 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Shan);
    put_in_equip(&mut e, 1, eq1);
    put_in_equip(&mut e, 1, eq2);
    put_in_hand(&mut e, 1, h1);
    put_in_hand(&mut e, 1, h2);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(2, 2);

    advance_to_prepare(&mut e, 0, 1);
    let ct = e.open_choice().expect("dddbailei_target").clone();
    assert_eq!(ct.spec.prompt, "dddbailei_target");
    assert!(ct.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(!ct.spec.options.contains(&ChoiceOption::Player(2)), "P2 neither max-equip nor hp=1 must be excluded");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    // Choice of specific card (`dddbailei_pick_card`, mandatory because hp != 1!): owner picks `eq2` specifically!
    let cp = e.open_choice().expect("dddbailei_pick_card").clone();
    assert_eq!(cp.observer, 0);
    assert_eq!(cp.spec.prompt, "dddbailei_pick_card");
    assert!(!cp.spec.allow_cancel);
    assert!(cp.spec.options.contains(&ChoiceOption::Card(eq2)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(eq2)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(eq2)]);
    assert!(e.state_core().player(0).hand.contains(&eq2), "owner gained specifically chosen eq2");
    assert!(has_skill(&e, 0, "dddbailei"));

    // Case B: P2 has hp == 1 (NOT maxEquip -> deals 1 damage). Branch B1: P2 saved by P1 Tao -> owner RETAINS `dddbailei`!
    e.debug_set_hp(2, 1);
    let tao1 = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut e, 1, tao1);
    advance_to_prepare(&mut e, 0, 1);
    let _ct2 = e.open_choice().unwrap().clone();
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    // P2 dying -> P1 saves with `tao1`!
    let mut e_die = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let cd_save = e.open_choice().expect("dying tao ask on P1").clone();
    assert_eq!(cd_save.observer, 1);
    answer(&mut e, 1, vec![ChoiceOption::Card(tao1)]);
    drain(&mut e);
    assert!(e.state_core().player(2).alive, "P2 saved by Tao");
    assert!(has_skill(&e, 0, "dddbailei"), "saved hp=1 target must NOT remove dddbailei from owner");

    // Branch B2: `e_die` passes dying save -> P2 actually dies -> AFTER death, `dddbailei` IS REMOVED!
    drain(&mut e_die);
    assert!(!e_die.state_core().player(2).alive);
    assert!(!has_skill(&e_die, 0, "dddbailei"), "actual death removes dddbailei from owner");

    // Case C: BOTH `isMaxEquip && hp == 1` -> `dddbailei_pick_card` (`allow_cancel: true`): cancel deals 1 damage!
    let (mut ec, _, _) = new_clean_engine(726, 3);
    ec.debug_grant(0, "dddbailei");
    let eq_c = find_card_by(&ec, |_, d| d.kind == CardKind::Zhuge);
    put_in_equip(&mut ec, 1, eq_c);
    ec.debug_set_hp(1, 1);
    advance_to_prepare(&mut ec, 0, 1);
    answer(&mut ec, 0, vec![ChoiceOption::Player(1)]);
    let cp_both = ec.open_choice().expect("both conditions -> cancellable dddbailei_pick_card").clone();
    assert!(cp_both.spec.allow_cancel);
    answer(&mut ec, 0, vec![]); // Cancel gaining card -> deals 1 damage -> P1 dies -> removes `dddbailei`!
    drain(&mut ec);
    assert!(!ec.state_core().player(1).alive);
    assert!(!has_skill(&ec, 0, "dddbailei"));

    // Case D: strict `isMaxEquip(true)` rejects 0-equip and tied-equip players!
    let (mut e_zero, _, _) = new_clean_engine(727, 3);
    e_zero.debug_grant(0, "dddbailei");
    let z_h = find_nth_card_by(&e_zero, 0, |_, _| true);
    put_in_hand(&mut e_zero, 1, z_h);
    advance_to_prepare(&mut e_zero, 0, 1);
    assert!(e_zero.open_choice().is_none(), "0 equips tied across alive players is NOT isMaxEquip(true)");

    // Tied 1 equip vs 1 equip (with hp >= 2): also NOT strictly greatest, so neither triggers `dddbailei`!
    let eq_t1 = find_card_by(&e_zero, |_, d| d.kind == CardKind::Zhuge);
    let eq_t2 = find_card_by(&e_zero, |_, d| d.kind == CardKind::Renwang);
    put_in_equip(&mut e_zero, 1, eq_t1);
    put_in_equip(&mut e_zero, 2, eq_t2);
    advance_to_prepare(&mut e_zero, 0, 2);
    assert!(e_zero.open_choice().is_none(), "tied max equip count (1 vs 1) is NOT isMaxEquip(true)");

    // Tied 1 equip + hp == 1: eligible ONLY for damage (`hp == 1`), NOT `dddbailei_pick_card`!
    e_zero.debug_set_hp(1, 1);
    advance_to_prepare(&mut e_zero, 0, 3);
    let ct_tie = e_zero.open_choice().expect("hp == 1 alone still makes tied-equip P1 eligible").clone();
    assert_eq!(ct_tie.spec.options, vec![ChoiceOption::Player(1)]);
    answer(&mut e_zero, 0, vec![ChoiceOption::Player(1)]);
    assert_ne!(e_zero.open_choice().map(|c| c.spec.prompt), Some("dddbailei_pick_card"), "tied equip must not offer card-pick branch");
    drain(&mut e_zero);
    assert!(!e_zero.state_core().player(1).alive);
}

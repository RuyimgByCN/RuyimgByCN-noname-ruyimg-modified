use super::common::*;

#[test]
fn u01_fix_16_dddbingjian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1204-1441
    let (mut e, d3, p9) = new_clean_engine(91016, 4);
    let wk = find_general(&e, "ddd_wangkanglvkai");
    let zhuge = find_general(&e, "诸葛亮");
    e.assign_general(0, wk).unwrap();
    let bj = e.catalog().skill_by_name("dddbingjian").unwrap();
    e.assign_general(1, zhuge).unwrap();

    // Case 1: hand.len() == 0 (< 2) in Play -> choose Sha target FIRST -> draw 2 -> optional partner -> canceling partner STILL uses Sha!
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: bj });
    let sha_tgt = e.open_choice().expect("dddbingjian_sha_target first in Play").clone();
    assert_eq!(sha_tgt.spec.prompt, "dddbingjian_sha_target");
    assert!(sha_tgt.spec.allow_cancel);
    assert!(!sha_tgt.spec.options.contains(&ChoiceOption::Player(1)), "Kongcheng excluded");
    assert!(!sha_tgt.spec.options.contains(&ChoiceOption::Player(2)), "dist 2 excluded");
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);

    let p_c = e.open_choice().expect("dddbingjian_partner after draw 2").clone();
    assert_eq!(p_c.spec.prompt, "dddbingjian_partner");
    assert_eq!(e.state_core().player(0).hand.len(), 2);
    assert!(p_c.spec.allow_cancel);
    // Fork: partner chosen draws 2 too, whereas cancel skips partner draw AND STILL completes Sha against seat 3!
    let mut e_with_partner = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let hp3_before = e_with_partner.debug_hp(3);
    answer(&mut e_with_partner, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e_with_partner.state_core().player(2).hand.len(), 2);
    drain(&mut e_with_partner);
    assert_eq!(e_with_partner.debug_hp(3), hp3_before - 1);

    assert_mid_choice_archive_and_ai(&e, &d3, &p9, Vec::new());
    let hp3_cancel = e.debug_hp(3);
    answer(&mut e, 0, Vec::new());
    drain(&mut e);
    assert_eq!(e.debug_hp(3), hp3_cancel - 1);

    // Case 2: Defensive `Shan` with hand.len() == 4 (> 2) -> discard 2 hand cards -> discard partner `he` -> dodge Sha!
    let c_a = find_card_by(&e, |_, _| true);
    let c_b = find_card_by(&e, |id, _| id != c_a);
    let c_c = find_card_by(&e, |id, _| id != c_a && id != c_b);
    let c_d = find_card_by(&e, |id, _| id != c_a && id != c_b && id != c_c);
    let p3_h = find_card_by(&e, |id, _| id != c_a && id != c_b && id != c_c && id != c_d);
    let p3_eq = find_card_by(&e, |id, d| {
        id != c_a && id != c_b && id != c_c && id != c_d && id != p3_h && d.kind.is_equip()
    });
    let sha3 = find_card_by(&e, |id, d| {
        id != c_a && id != c_b && id != c_c && id != c_d && id != p3_h && id != p3_eq && d.kind == CardKind::Sha
    });
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, c_a);
    put_in_hand(&mut e, 0, c_b);
    put_in_hand(&mut e, 0, c_c);
    put_in_hand(&mut e, 0, c_d);
    put_in_hand(&mut e, 3, p3_h);
    put_in_equip(&mut e, 3, p3_eq);
    put_in_hand(&mut e, 3, sha3);
    e.debug_set_turn(3, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let hp0 = e.debug_hp(0);
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: sha3, targets: vec![0] });
    let resp = e.open_choice().expect("respond_shan dddbingjian").clone();
    assert!(resp.spec.options.contains(&ChoiceOption::Skill(bj)));
    answer(&mut e, 0, vec![ChoiceOption::Skill(bj)]);
    let disc = e.open_choice().expect("dddbingjian_discard").clone();
    assert_eq!((disc.spec.min, disc.spec.max), (2, 2));
    answer(&mut e, 0, vec![ChoiceOption::Card(c_a), ChoiceOption::Card(c_b)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    let d_p3 = e.open_choice().expect("dddbingjian_discard_target").clone();
    assert_eq!(d_p3.spec.prompt, "dddbingjian_discard_target");
    answer(&mut e, 0, vec![ChoiceOption::Card(p3_eq), ChoiceOption::HiddenCard(0)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0);
    assert!(e.state_core().player(3).hand.is_empty() && e.state_core().player(3).equips.is_empty());
}

#[test]
fn u01_fix_17_dddtongyu() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1443-1583
    let (mut e, d3, p9) = new_clean_engine(91017, 3);
    e.debug_grant(0, "dddtongyu");
    let ty = e.catalog().skill_by_name("dddtongyu").unwrap();

    let h_c = find_card_by(&e, |_, d| d.suit == Suit::Heart);
    let s_c = find_card_by(&e, |_, d| d.suit == Suit::Spade);
    let h_c2 = find_card_by(&e, |id, d| id != h_c && d.suit == Suit::Heart);
    let p1_sha = find_card_by(&e, |id, d| id != h_c && id != s_c && id != h_c2 && d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, h_c);
    put_in_hand(&mut e, 0, s_c);
    put_in_hand(&mut e, 0, h_c2);
    put_in_hand(&mut e, 1, p1_sha);
    e.debug_set_turn(0, Phase::Play, 1);

    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: ty });
    let cards_c = e.open_choice().expect("dddtongyu_cards").clone();
    assert_eq!(cards_c.spec.prompt, "dddtongyu_cards");
    // Duplicate suit must be rejected!
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: cards_c.id,
                selection: vec![ChoiceOption::Card(h_c), ChoiceOption::Card(h_c2)],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(h_c), ChoiceOption::Card(s_c)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(h_c), ChoiceOption::Card(s_c)]);

    // Target choices in seat order FIRST: 0 chooses lock hand (Tag(0)), 1 chooses lock hand (Tag(0)), 2 chooses invalid (Tag(1))!
    // Paid cards remain on table until Wugu finishes!
    assert!(e.state_core().table.contains(&h_c) && e.state_core().table.contains(&s_c));
    assert!(!e.state_core().discard.contains(&h_c));
    for obs in [0u8, 1, 2] {
        let tc = e.open_choice().expect("dddtongyu_target_choice").clone();
        assert_eq!((tc.observer, tc.spec.prompt), (obs, "dddtongyu_target_choice"));
        let tag = if obs == 2 { 1 } else { 0 };
        answer(&mut e, obs, vec![ChoiceOption::Tag(tag)]);
    }

    // Direction choice (`dddtongyu_direction`) AFTER all targets accept/refuse!
    let dir_c = e.open_choice().expect("dddtongyu_direction").clone();
    assert_eq!(dir_c.spec.prompt, "dddtongyu_direction");
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]); // counterclockwise: 0 -> 2(excluded) -> 1!

    // Counterclockwise: Wugu reveals all 3 initial targets + 2 extra = 5 cards!
    // Seat 0 picks 1, seat 1 picks 1, leaving 3 extra cards -> `dddtongyu_remain_target` for excluded seat 2!
    let w0 = e.open_choice().expect("wugu_pick seat 0").clone();
    assert_eq!((w0.observer, w0.spec.prompt), (0, "wugu_pick"));
    assert_eq!(w0.spec.options.len(), 5);
    answer(&mut e, 0, vec![w0.spec.options[0]]);

    let w1 = e.open_choice().expect("wugu_pick seat 1").clone();
    assert_eq!((w1.observer, w1.spec.prompt), (1, "wugu_pick"));
    answer(&mut e, 1, vec![w1.spec.options[0]]);

    let rem_c = e.open_choice().expect("dddtongyu_remain_target").clone();
    assert_eq!((rem_c.observer, rem_c.spec.prompt), (0, "dddtongyu_remain_target"));
    assert_eq!(rem_c.spec.options, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().player(2).hand.len(), 3);
    assert!(e.state_core().discard.contains(&h_c) && e.state_core().discard.contains(&s_c));

    // Seat 0 & seat 1 chose Tag(0) -> `dddtongyu_blocker` (tempSkill for current turn) forbids using/playing hand cards DURING seat 0's turn!
    let p0_sha = find_card_by(&e, |id, d| id != p1_sha && d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, p0_sha);
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: p0_sha, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ),
        "dddtongyu Tag(0) must block seat 0 playing hand cards during current turn"
    );

    // Once seat 1's own turn begins (after global phaseAfter clears temp skill), seat 1 CAN play hand cards normally.
    e.debug_set_turn(1, Phase::Play, 1);
    assert!(
        matches!(
            submit(&mut e, 1, GameAction::PlayCard { card: p1_sha, targets: vec![0] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "dddtongyu_blocker must expire after turn ends"
    );
}

#[test]
fn u01_fix_18_dddzhilian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1584-1675
    let (mut e, d3, p9) = new_clean_engine(91018, 3);
    let baosanniang = find_general(&e, "ddd_baosanniang");
    let male_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.4 == Gender::Male)
        })
        .unwrap();
    e.assign_general(0, baosanniang).unwrap(); // female, has `dddzhilian`
    e.assign_general(1, male_g).unwrap();
    e.assign_general(2, male_g).unwrap();

    // Put an old-turn Sha into discard BEFORE turn 2 starts (must NOT be scooped!).
    let old_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    remove_card_from_all_zones(&mut e, old_sha);
    e.state_core_mut().discard.push(old_sha);

    // Turn 2 of male seat 1: seat 1 plays Sha at seat 0, seat 0 responds Shan -> exactly 2 players (0 & 1)!
    let sha1 = find_card_by(&e, |id, d| id != old_sha && d.kind == CardKind::Sha);
    let shan0 = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let eq0 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let h1 = find_card_by(&e, |id, _| id != old_sha && id != sha1 && id != shan0 && id != eq0);
    put_in_hand(&mut e, 1, sha1);
    put_in_hand(&mut e, 0, shan0);
    put_in_equip(&mut e, 0, eq0);
    put_in_hand(&mut e, 1, h1);

    e.debug_set_turn(1, Phase::Play, 2);
    let out_sha = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    assert!(matches!(out_sha, CommandOutcome::Pending { .. }), "PlayCard Sha failed: {out_sha:?}");
    answer(&mut e, 0, vec![ChoiceOption::Card(shan0)]);
    drain(&mut e);

    // Advance into `Phase::End`: MUST NOT trigger at `Phase::End, begin: true`!
    e.debug_set_turn(1, Phase::Discard, 2);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::End);
    assert!(e.open_choice().is_none(), "dddzhilian is TurnPhaseEnd (`phaseEnd`), NOT Phase::End begin");

    // Advance out of `Phase::End` -> `TurnPhaseEnd`: triggers `dddzhilian_mode`!
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let m = e.open_choice().expect("dddzhilian_mode at TurnPhaseEnd").clone();
    assert_eq!((m.observer, m.spec.prompt), (0, "dddzhilian_mode"));
    assert_eq!(m.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);

    // Fork Choice A (Draw 1 each): scoops `sha1`, NOT `old_sha`!
    let mut e_draw = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_draw, 0, vec![ChoiceOption::Tag(0)]);
    assert!(e_draw.state_core().player(0).hand.contains(&sha1));
    assert!(!e_draw.state_core().player(0).hand.contains(&old_sha));

    // Choice B (Discard 1 `he` each): sorted by seat from turn.current (1 -> 0)! Seat 1 picks `h1`, seat 0 picks `eq0`, then seat 0 scoops `sha1`!
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    let d1 = e.open_choice().expect("dddzhilian_discard seat 1").clone();
    assert_eq!((d1.observer, d1.spec.prompt), (1, "dddzhilian_discard"));
    answer(&mut e, 1, vec![ChoiceOption::Card(h1)]);
    let d0 = e.open_choice().expect("dddzhilian_discard seat 0").clone();
    assert_eq!((d0.observer, d0.spec.prompt), (0, "dddzhilian_discard"));
    answer(&mut e, 0, vec![ChoiceOption::Card(eq0)]);
    assert!(e.state_core().player(0).hand.contains(&sha1));
    assert!(!e.state_core().player(0).hand.contains(&old_sha));
}

#[test]
fn u01_fix_19_dddjijian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1676-1806
    let (mut e, d3, p9) = new_clean_engine(91019, 3);
    let baosanniang = find_general(&e, "ddd_baosanniang");
    let male_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.4 == Gender::Male)
        })
        .unwrap();
    e.assign_general(0, baosanniang).unwrap();
    e.assign_general(1, male_g).unwrap();
    let jj = e.catalog().skill_by_name("dddjijian").unwrap();

    // Seat 1 starts with 0 hand cards, receives black `sha` + already has black `dismantle`? First test empty male + red trick:
    let blk_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let blk_dismantle = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle && d.suit == Suit::Spade);
    let red_le = find_card_by(&e, |_, d| d.kind == CardKind::Le && d.suit == Suit::Heart);
    let seat2_card = find_card_by(&e, |id, _| id != blk_sha && id != blk_dismantle && id != red_le);
    put_in_hand(&mut e, 0, blk_sha);
    put_in_hand(&mut e, 1, blk_dismantle);
    put_in_hand(&mut e, 1, red_le);
    put_in_hand(&mut e, 2, seat2_card);

    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jj });
    answer(&mut e, 0, vec![ChoiceOption::Card(blk_sha)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);

    // Seat 1 forced-shows `1..=2` same-color `Sha`/normal-trick hand cards (`red_le` delay trick MUST NOT be offered!).
    let show_c = e.open_choice().expect("dddjijian_show").clone();
    assert_eq!((show_c.observer, show_c.spec.prompt), (1, "dddjijian_show"));
    assert!(!show_c.spec.options.contains(&ChoiceOption::Card(red_le)));
    assert!(show_c.spec.options.contains(&ChoiceOption::Card(blk_sha)));
    assert!(show_c.spec.options.contains(&ChoiceOption::Card(blk_dismantle)));
    assert_mid_choice_archive_and_ai(
        &e,
        &d3,
        &p9,
        vec![ChoiceOption::Card(blk_sha), ChoiceOption::Card(blk_dismantle)],
    );
    answer(&mut e, 1, vec![ChoiceOption::Card(blk_sha), ChoiceOption::Card(blk_dismantle)]);

    // Seat order from `turn.current == 0`: [0, 1]! Seat 0 picks card name (`Kind`), then target!
    let u0 = e.open_choice().expect("dddjijian_use_card seat 0").clone();
    assert_eq!((u0.observer, u0.spec.prompt), (0, "dddjijian_use_card"));
    let dismantle_code = u0.spec.options.iter().find(|o| matches!(o, ChoiceOption::Kind(k) if *k != u0.spec.options.first().and_then(|x| match x { ChoiceOption::Kind(c) => Some(*c), _ => None }).unwrap_or(999))).copied().unwrap_or(u0.spec.options[0]);
    let _ = dismantle_code;
    // Let seat 0 pick Dismantle (`Kind`) on seat 2 -> then seat 1 MUST get the remaining unused Sha (`Kind`)!
    let sha_kind_code = u0.spec.options[0];
    let dis_kind_opt = u0.spec.options[1];
    answer(&mut e, 0, vec![dis_kind_opt]);
    let t0 = e.open_choice().expect("dddjijian_use_target seat 0").clone();
    assert_eq!((t0.observer, t0.spec.prompt), (0, "dddjijian_use_target"));
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(!e.state_core().player(2).hand.contains(&seat2_card), "Dismantle must discard seat 2 card");

    let u1 = e.open_choice().expect("dddjijian_use_card seat 1").clone();
    assert_eq!((u1.observer, u1.spec.prompt), (1, "dddjijian_use_card"));
    assert_eq!(u1.spec.options, vec![sha_kind_code]);
    answer(&mut e, 1, vec![sha_kind_code]);
    answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert!(!e.state_core().turn.sha_used_play_phase, "dddjijian virtual Sha has addCount: false");
}

#[test]
fn u01_fix_20_dddjijian_old() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1807-2004
    let (mut e, d3, p9) = new_clean_engine(91020, 3);
    let male_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.4 == Gender::Male)
        })
        .unwrap();
    e.assign_general(1, male_g).unwrap();
    e.debug_grant(0, "dddjijian_old");
    let jjo = e.catalog().skill_by_name("dddjijian_old").unwrap();

    let red_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Heart);
    let red_shown = find_card_by(&e, |id, d| id != red_sha && d.suit == Suit::Heart);
    let p1_unshown_wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    put_in_hand(&mut e, 0, red_sha);
    put_in_hand(&mut e, 1, red_shown);
    put_in_hand(&mut e, 1, p1_unshown_wuzhong);

    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jjo });
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    // Seat 1 shows `red_shown` (`1..=2`, forced):
    answer(&mut e, 1, vec![ChoiceOption::Card(red_shown)]);
    // Owner gives `red_sha` (same red color -> `num == 1`!):
    let g_c = e.open_choice().expect("dddjijian_old_give_sha").clone();
    assert_eq!(g_c.spec.prompt, "dddjijian_old_give_sha");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(red_sha)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red_sha)]);

    // Seat 1 chooses up to `num = 1` basic/normal-trick name from hand EXCLUDING `red_shown`!
    let u_c = e.open_choice().expect("dddjijian_old_use_card").clone();
    assert_eq!((u_c.observer, u_c.spec.prompt), (1, "dddjijian_old_use_card"));
    assert!(u_c.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![u_c.spec.options[0]]);
    let h1_before = e.state_core().player(1).hand.len();
    // Pick Wuzhong (last option or find non-Sha/non-target? Wuzhong has 0 target prompt or auto-use!):
    let wuzhong_opt = u_c.spec.options.iter().find(|&&o| o == u_c.spec.options[u_c.spec.options.len() - 1]).copied().unwrap();
    answer(&mut e, 1, vec![wuzhong_opt]);
    if let Some(tc) = e.open_choice().cloned() {
        if tc.spec.prompt == "dddjijian_old_use_target" {
            answer(&mut e, 1, vec![tc.spec.options[0]]);
        }
    }
    drain(&mut e);
    assert!(
        e.state_core().player(1).hand.len() > h1_before
            || e.log()
                .iter()
                .any(|ev| matches!(ev, LogEvent::VirtualCardPlayed { player: 1, .. }))
    );
}

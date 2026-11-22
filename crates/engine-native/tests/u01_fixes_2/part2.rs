use super::common::*;

#[test]
fn u01_fix2_06_dddshijian() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2423-2518
    let (mut e, d3, p9) = new_clean_engine(92010, 3);
    let xujing = find_general(&e, "ddd_xujing");
    e.assign_general(0, xujing).unwrap();
    retain_only_skill(&mut e, 0, Some("dddshijian"));

    let le1 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let bing = find_card_by(&e, |_, d| d.kind == CardKind::Bing);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let give_c = find_card_by(&e, |id, _| id != le1 && id != bing && id != wuzhong);

    put_in_hand(&mut e, 0, give_c);
    put_in_hand(&mut e, 1, le1);
    put_in_hand(&mut e, 1, bing);
    put_in_hand(&mut e, 1, wuzhong);
    e.debug_set_turn(1, Phase::Play, 1);

    // 1st card of P1's Play phase: does NOT trigger `dddshijian`
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: le1, targets: vec![2] });
    assert!(e.open_choice().is_none());

    // 2nd card of P1's Play phase: triggers `dddshijian_give` AFTER `bing` finishes!
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: bing, targets: vec![2] });
    assert!(e.state_core().player(2).judge.contains(&bing));
    let cg = e.open_choice().expect("dddshijian_give").clone();
    assert_eq!((cg.observer, cg.spec.prompt), (0, "dddshijian_give"));
    assert!(cg.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(give_c)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(give_c)]);
    assert!(e.state_core().player(1).hand.contains(&give_c));

    // 3rd card of P1's Play phase (`wuzhong`): `dddshijian_mode` (`额外结算` vs `摸一张牌`)!
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: wuzhong, targets: Vec::new() });
    let cm = e.open_choice().expect("dddshijian_mode").clone();
    assert_eq!((cm.observer, cm.spec.prompt), (0, "dddshijian_mode"));
    assert!(!cm.spec.allow_cancel);
    assert_eq!(cm.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(0)]);

    // Fork Choice A (`Tag(0)`): Wuzhong settles TWICE -> P1 draws 4 cards instead of 2!
    let mut e_extra = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let p1_h_before = e_extra.state_core().player(1).hand.len();
    answer(&mut e_extra, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(
        e_extra.state_core().player(1).hand.len(),
        p1_h_before + 4,
        "extra effectCount must resolve Wuzhong twice (+4 cards)"
    );

    // Live Choice B (`Tag(1)`): owner P0 draws 1, P1 draws normal 2!
    let p0_h_before = e.state_core().player(0).hand.len();
    let p1_h_before = e.state_core().player(1).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.state_core().player(0).hand.len(), p0_h_before + 1);
    assert_eq!(e.state_core().player(1).hand.len(), p1_h_before + 2);
}

#[test]
fn u01_fix2_07_dddxuyu_old() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2519-2573
    let (mut e, d3, p9) = new_clean_engine(92011, 3);
    e.debug_grant(0, "dddxuyu_old");

    let le1 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let le2 = find_card_by(&e, |id, d| id != le1 && d.kind == CardKind::Le);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, le1);
    put_in_hand(&mut e, 0, wuzhong);
    put_in_hand(&mut e, 0, sha);
    put_in_hand(&mut e, 0, le2);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1st card of Play phase 1 (`le1`): `dddxuyu_old_confirm` triggers AFTER `le1` placed -> draw 2!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: le1, targets: vec![1] });
    assert!(e.state_core().player(1).judge.contains(&le1));
    let c1 = e.open_choice().expect("dddxuyu_old_confirm").clone();
    assert_eq!((c1.observer, c1.spec.prompt), (0, "dddxuyu_old_confirm"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    let h_before = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), h_before + 2);

    // 2nd card (`wuzhong`) does NOT trigger discard; 3rd card (`sha`) triggers `dddxuyu_old_discard` at `useCardAfter`!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wuzhong, targets: Vec::new() });
    assert!(e.open_choice().is_none(), "2nd card must not trigger dddxuyu_old");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    let cd = e.open_choice().expect("dddxuyu_old_discard on 3rd card useCardAfter").clone();
    assert_eq!((cd.observer, cd.spec.prompt), (0, "dddxuyu_old_discard"));
    assert!(!cd.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(le2)]);
    let h_pre_drop = e.state_core().player(0).hand.len();
    // Discard one non-le2 card so le2 remains for the 2nd Play phase check:
    let drop = e.state_core().player(0).hand.iter().copied().find(|&c| c != le2).unwrap();
    answer(&mut e, 0, vec![ChoiceOption::Card(drop)]);
    assert_eq!(e.state_core().player(0).hand.len(), h_pre_drop - 1);

    // Prepare 2nd Play phase in the SAME turn (counterexample check!):
    let mut e_extra_phase = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    e_extra_phase.state_core_mut().turn.extra_phases.push(2);
    let _ = submit(&mut e_extra_phase, 0, GameAction::NextPhase);
    assert!(e_extra_phase.open_choice().is_none(), "Discard phase skipped / extra Play phase enters cleanly");
    if e_extra_phase.turn().phase == Phase::Discard {
        let _ = submit(&mut e_extra_phase, 0, GameAction::NextPhase);
    }
    if e_extra_phase.turn().phase == Phase::End {
        // Make sure we are in the extra Play phase if NextPhase went to Discard->Play or directly Play
    }

    // Now in 2nd Play phase of SAME turn, playing `le2` is the 1st card of THIS phaseUse -> triggers `dddxuyu_old_confirm` again!
    assert_eq!(e_extra_phase.turn().phase, Phase::Play);
    let _ = submit(
        &mut e_extra_phase,
        0,
        GameAction::PlayCard {
            card: le2,
            targets: vec![2],
        },
    );
    while let Some(c) = e_extra_phase.open_choice().cloned() {
        if c.spec.prompt != "nullify_window" {
            break;
        }
        answer(&mut e_extra_phase, c.observer, vec![ChoiceOption::Pass]);
    }
    assert_eq!(
        e_extra_phase.open_choice().expect("2nd phaseUse triggers dddxuyu_old again").spec.prompt,
        "dddxuyu_old_confirm"
    );
}

#[test]
fn u01_fix2_08_dddshijian_old() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2574-2635
    let (mut e, d3, p9) = new_clean_engine(92012, 3);
    e.debug_grant(0, "dddshijian_old");

    let le1 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let bing = find_card_by(&e, |_, d| d.kind == CardKind::Bing);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let le2 = find_card_by(&e, |id, d| id != le1 && d.kind == CardKind::Le);
    put_in_hand(&mut e, 1, le1);
    put_in_hand(&mut e, 1, bing);
    put_in_hand(&mut e, 1, wuzhong);
    put_in_hand(&mut e, 1, sha);
    put_in_hand(&mut e, 1, le2);
    e.debug_set_turn(1, Phase::Play, 1);

    let _ = submit(&mut e, 1, GameAction::PlayCard { card: le1, targets: vec![2] });
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: bing, targets: vec![2] });
    assert!(e.open_choice().is_none());

    // 3rd card of P1's Play phase: `dddshijian_old_confirm` -> P1 draws 1!
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: wuzhong, targets: Vec::new() });
    let c3 = e.open_choice().expect("dddshijian_old_confirm").clone();
    assert_eq!((c3.observer, c3.spec.prompt), (0, "dddshijian_old_confirm"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    let p1_h = e.state_core().player(1).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(1).hand.len(), p1_h + 1);

    // Fork A: P1 plays NO 4th card (`NextPhase` -> P0 draws 0!)
    let mut e_3 = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let p0_h3 = e_3.state_core().player(0).hand.len();
    let _ = submit(&mut e_3, 1, GameAction::NextPhase);
    assert_eq!(e_3.state_core().player(0).hand.len(), p0_h3);

    // Live B: P1 uses 4th card (`sha`, while P1 still holds `le2` so `he > 0`) -> `useCardAfter` draws 2 for P0!
    let p0_h4 = e.state_core().player(0).hand.len();
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![2] });
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), p0_h4 + 2);
}

#[test]
fn u01_fix2_09_dddtaisi() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2636-2705
    let (mut e, d3, p9) = new_clean_engine(92013, 4);
    let caomao = find_general(&e, "ddd_caomao");
    e.assign_general(0, caomao).unwrap();
    retain_only_skill(&mut e, 0, Some("dddtaisi"));

    let old_card = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let turn_card1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let turn_card2 = find_card_by(&e, |id, d| id != turn_card1 && d.kind == CardKind::Sha);

    // Put an OLD-turn card directly into discard before turn events (must NOT be selectable!).
    remove_card_from_all_zones(&mut e, old_card);
    e.state_core_mut().discard.push(old_card);

    // Negative guard: turn_card1 enters discard this turn, P0 has NOT changed HP this turn -> no trigger at phaseEnd!
    e.debug_set_turn(1, Phase::Discard, 1);
    put_in_hand(&mut e, 1, turn_card1);
    // Advance P1 Discard (discards turn_card1 if hp==0? Give P1 hp=1 and 2 cards or debug discard -> use Phase::Play dismantle!)
    e.debug_set_turn(1, Phase::Play, 1);
    let dism = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle);
    put_in_hand(&mut e, 1, dism);
    put_in_hand(&mut e, 2, turn_card1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism, targets: vec![2] });
    e.debug_set_turn(1, Phase::End, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "no changeHp on P0 -> no dddtaisi at phaseEnd");

    // New turn 2: P1 deals 1 damage to P0 (first damage source!), THEN P2 deals 1 damage to P0 (second damage source!),
    // and `turn_card2` enters discard this turn -> at `TurnEndWindow { after: false }` (`phaseEnd` when advancing `Phase::End`):
    // `dddtaisi_target` opens; picking FIRST damage source `P1` STILL draws 2 cards for `P0`!
    let dism2 = find_card_by(&e, |id, d| id != dism && d.kind == CardKind::Dismantle);
    put_in_hand(&mut e, 1, dism2);
    put_in_hand(&mut e, 3, turn_card2);
    e.debug_set_turn(1, Phase::Play, 2);
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let _ = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism2, targets: vec![3] });

    e.debug_set_turn(1, Phase::End, 2);
    let out = submit(&mut e, 1, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let ct = e.open_choice().expect("dddtaisi_target").clone();
    assert_eq!((ct.observer, ct.spec.prompt), (0, "dddtaisi_target"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);

    let cc = e.open_choice().expect("dddtaisi_card").clone();
    assert_eq!((cc.observer, cc.spec.prompt), (0, "dddtaisi_card"));
    assert!(!cc.spec.options.contains(&ChoiceOption::Card(old_card)), "old discard must be excluded");
    assert!(cc.spec.options.contains(&ChoiceOption::Card(turn_card2)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(turn_card2)]);
    let p0_h = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Card(turn_card2)]);
    assert!(e.state_core().player(1).hand.contains(&turn_card2));
    assert_eq!(e.state_core().player(0).hand.len(), p0_h + 2, "P1 dealt damage to P0 earlier in turn even before P2!");
}

#[test]
fn u01_fix2_09b_dddtaisi_change_hp_and_non_damager() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2636-2705
    let (mut e, d3, p9) = new_clean_engine(92109, 4);
    let caomao = find_general(&e, "ddd_caomao");
    e.assign_general(0, caomao).unwrap();
    retain_only_skill(&mut e, 0, Some("dddtaisi"));

    let turn_card = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let dism = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle);

    // 1) Real LoseHp on P0 (via `kurou`, 0 damagers!) + card entering discard this turn -> `dddtaisi_target` MUST trigger and offer ALL alive players!
    e.debug_grant(0, "kurou");
    let kurou_def = e.catalog().skill_by_name("kurou").unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: kurou_def });
    put_in_hand(&mut e, 0, dism);
    put_in_hand(&mut e, 2, turn_card);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: dism, targets: vec![2] });
    e.debug_set_turn(0, Phase::End, 1);
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let ct = e.open_choice().expect("dddtaisi_target on sourceless LoseHp").clone();
    assert_eq!(ct.spec.prompt, "dddtaisi_target");
    assert_eq!(
        ct.spec.options,
        vec![
            ChoiceOption::Player(0),
            ChoiceOption::Player(1),
            ChoiceOption::Player(2),
            ChoiceOption::Player(3),
        ],
        "dddtaisi must allow selecting ANY alive player, not only damagers"
    );
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    let cc = e.open_choice().expect("dddtaisi_card").clone();
    assert!(cc.spec.options.contains(&ChoiceOption::Card(turn_card)));
    let p0_h = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Card(turn_card)]);
    assert!(e.state_core().player(2).hand.contains(&turn_card));
    assert_eq!(e.state_core().player(0).hand.len(), p0_h, "non-damager P2 draws 0 for P0");

    // 2) Recover on P0 -> also counts as changeHp (`turn_change_hp == 1`), plus choosing self (P0) gives card to self and draws 0!
    let dism2 = find_card_by(&e, |id, d| id != dism && d.kind == CardKind::Dismantle);
    let turn_card2 = find_card_by(&e, |id, d| id != turn_card && d.kind == CardKind::Sha);
    put_in_hand(&mut e, 1, dism2);
    put_in_hand(&mut e, 3, turn_card2);
    e.debug_set_turn(1, Phase::Play, 2);
    // P0 already at hp 3 from turn 1 `kurou`; trigger Recover via Taoyuan played by P1 in turn 2!
    let taoyuan = find_card_by(&e, |_, d| d.kind == CardKind::Taoyuan);
    put_in_hand(&mut e, 1, taoyuan);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: taoyuan, targets: Vec::new() });
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism2, targets: vec![3] });
    e.debug_set_turn(1, Phase::End, 2);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let ct2 = e.open_choice().expect("dddtaisi_target on Recover").clone();
    assert!(ct2.spec.options.contains(&ChoiceOption::Player(0)));
    answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
    let h0_pre = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Card(turn_card2)]);
    assert_eq!(e.state_core().player(0).hand.len(), h0_pre + 1, "gives turn_card2 to self (0 bonus draw)");
}

#[test]
fn u01_fix2_10_dddquche() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2706-2818
    let (mut e, d3, p9) = new_clean_engine(92014, 4);
    let caomao = find_general(&e, "ddd_caomao");
    e.assign_general(0, caomao).unwrap();
    let quche_def = e.catalog().skill_by_name("dddquche").unwrap();

    let red1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart) && d.kind == CardKind::Tao);
    let red2 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Diamond) && d.kind == CardKind::Shan);
    let blk1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade));
    let blk2 = find_card_by(&e, |id, d| id != blk1 && matches!(d.suit, Suit::Club));
    put_in_hand(&mut e, 0, red1);
    put_in_hand(&mut e, 0, red2);
    put_in_hand(&mut e, 0, blk1);
    put_in_hand(&mut e, 0, blk2);
    e.debug_set_turn(0, Phase::Play, 1);

    // P2 is at distance 2 initially!
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);

    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: quche_def });
    let c_r1 = e.open_choice().expect("dddquche_red_card 1").clone();
    assert_eq!(c_r1.spec.prompt, "dddquche_red_card");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(red1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red1)]);

    let c_g1 = e.open_choice().expect("dddquche_give_target 1").clone();
    assert_eq!(c_g1.spec.prompt, "dddquche_give_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.state_core().player(1).hand.contains(&red1));
    // +1 mark -> distance to P2 becomes 1!
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1);

    // Step 3: black card `blk1` as virtual Sha (`addCount: false` -> can choose P2 at distance 1, and does NOT consume Sha limit!)
    let c_b1 = e.open_choice().expect("dddquche_black_sha 1").clone();
    assert_eq!(c_b1.spec.prompt, "dddquche_black_sha");
    answer(&mut e, 0, vec![ChoiceOption::Card(blk1)]);
    let c_st1 = e.open_choice().expect("dddquche_sha_target 1").clone();
    assert_eq!(c_st1.spec.prompt, "dddquche_sha_target");
    assert!(c_st1.spec.options.contains(&ChoiceOption::Player(2)), "mark -1 brings P2 into Sha range 1");
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().player(2).hp, 3);

    // Loop back to Step 1! Give `red2` to P3, then use `blk2` as a SECOND virtual Sha on P2!
    let c_r2 = e.open_choice().expect("dddquche_red_card loop 2").clone();
    assert_eq!(c_r2.spec.prompt, "dddquche_red_card");
    answer(&mut e, 0, vec![ChoiceOption::Card(red2)]);
    let _c_g2 = e.open_choice().expect("dddquche_give_target 2").clone();
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    let _c_b2 = e.open_choice().expect("dddquche_black_sha 2").clone();
    answer(&mut e, 0, vec![ChoiceOption::Card(blk2)]);
    let _c_st2 = e.open_choice().expect("dddquche_sha_target 2").clone();
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().player(2).hp, 2);

    // Limited guard: cannot reactivate `dddquche` after loop completes
    put_in_hand(&mut e, 0, red1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: quche_def }),
        CommandOutcome::Rejected(_)
    ));
}

#[test]
fn u01_fix2_10b_dddquche_no_loop_and_marks_hongyan() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2706-2818
    let (mut e, d3, p9) = new_clean_engine(92110, 4);
    let caomao = find_general(&e, "ddd_caomao");
    e.assign_general(0, caomao).unwrap();
    let quche_def = e.catalog().skill_by_name("dddquche").unwrap();

    let red1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart) && d.kind == CardKind::Tao);
    let red2 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Diamond) && d.kind == CardKind::Shan);
    let spade1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade));
    let club1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Club));

    // (a) No black cards: giving 1 red card adds 1 mark and FINISHES without looping back to red2!
    put_in_hand(&mut e, 0, red1);
    put_in_hand(&mut e, 0, red2);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: quche_def });
    answer(&mut e, 0, vec![ChoiceOption::Card(red1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.open_choice().is_none(), "no black cards -> must NOT loop back to dddquche_red_card");
    assert!(e.state_core().player(0).hand.contains(&red2));
    assert_eq!(flag(&e, 0, "dddquche_effect_marks"), 1);

    // (b) Cancelling `dddquche_black_sha` stops immediately without looping red!
    let (mut e2, _, _) = new_clean_engine(92111, 4);
    e2.assign_general(0, caomao).unwrap();
    put_in_hand(&mut e2, 0, red1);
    put_in_hand(&mut e2, 0, red2);
    put_in_hand(&mut e2, 0, club1);
    e2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: quche_def });
    answer(&mut e2, 0, vec![ChoiceOption::Card(red1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    let cb = e2.open_choice().expect("dddquche_black_sha").clone();
    assert_eq!(cb.spec.prompt, "dddquche_black_sha");
    assert_mid_choice_archive_and_ai(&e2, &d3, &p9, vec![ChoiceOption::Card(club1)]);
    answer(&mut e2, 0, Vec::new());
    assert!(e2.open_choice().is_none(), "cancelling black Sha must finish without looping red");
    assert!(e2.state_core().player(0).hand.contains(&red2));
    assert_eq!(flag(&e2, 0, "dddquche_effect_marks"), 1);

    // (c) Using black Sha increments `dddquche_effect_marks` (1 -> 2), AND `xinhongyan` Spade is RED-only (NOT black)!
    let (mut e3, _, _) = new_clean_engine(92112, 4);
    e3.assign_general(0, caomao).unwrap();
    e3.debug_grant(0, "xinhongyan");
    put_in_hand(&mut e3, 0, red1);
    put_in_hand(&mut e3, 0, club1);
    put_in_hand(&mut e3, 0, spade1);
    e3.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: quche_def });
    let cr = e3.open_choice().expect("dddquche_red_card with xinhongyan").clone();
    assert!(cr.spec.options.contains(&ChoiceOption::Card(spade1)), "xinhongyan Spade is red");
    answer(&mut e3, 0, vec![ChoiceOption::Card(red1)]);
    answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
    let cb3 = e3.open_choice().expect("dddquche_black_sha with xinhongyan").clone();
    assert!(!cb3.spec.options.contains(&ChoiceOption::Card(spade1)), "xinhongyan Spade must NOT also count as black");
    assert!(cb3.spec.options.contains(&ChoiceOption::Card(club1)));
    answer(&mut e3, 0, vec![ChoiceOption::Card(club1)]);
    answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(flag(&e3, 0, "dddquche_effect_marks"), 2, "black Sha step increments mark to 2");
    assert_eq!(e3.state_core().computed_distance(e3.catalog(), 0, 2), 1);
}

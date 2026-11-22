use super::common::*;

/// (07) `dddyouxue` (`skill.js:4975-5017`)
/// - `global: "roundStart"` (`TurnBegin` once per round): shared GLOBAL `"dddyouxue"` mark across all owners!
/// - First time nobody has mark -> grants mark to `owner` FIRST, then picks alive `!hasMark` (excludes self!),
///   moves mark `source -> target`, draws `distance(source, target)`.
/// - Does NOT re-trigger inside the same round (`P1`, `P2`, `P3` turns); triggers again at next round `TurnBegin`!
#[test]
fn u01_fix3_07_dddyouxue() {
    let (mut e, d3, p9) = new_clean_engine(709, 4);
    e.debug_grant(0, "dddyouxue");
    e.debug_grant(2, "dddyouxue");
    // Ensure round wrap triggers TurnBegin -> advance from P3 Discard/End to Round 2 P0 TurnBegin!
    e.debug_set_turn(3, Phase::End, 1);
    submit(&mut e, 3, GameAction::NextPhase);

    // First owner (P0) runs `dddyouxue`: nobody had mark -> P0 acquires mark, MUST pick `!hasMark` (excludes P0!), non-cancelable!
    let c0 = e.open_choice().expect("dddyouxue_target for P0").clone();
    assert_eq!(c0.observer, 0);
    assert_eq!(c0.spec.prompt, "dddyouxue_target");
    assert!(!c0.spec.allow_cancel);
    assert_eq!(c0.spec.min, 1);
    assert!(!c0.spec.options.contains(&ChoiceOption::Player(0)), "first holder P0 cannot self-pick");
    let rej_cancel = submit(&mut e, 0, GameAction::Answer { choice_id: c0.id, selection: vec![] });
    assert!(matches!(rej_cancel, CommandOutcome::Rejected(_)), "forced dddyouxue_target rejects empty cancel");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(1)]);
    let p0_before = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    // dist(0, 1) == 1 -> P0 draws 1; global holder is now P1!
    assert_eq!(e.state_core().player(0).hand.len(), p0_before + 1);

    // Second owner (P2) in SAME roundStart triggers against shared global holder P1 (NOT P2!):
    let c2 = e.open_choice().expect("second owner P2 shares global dddyouxue mark").clone();
    assert_eq!(c2.observer, 2);
    assert_eq!(c2.spec.prompt, "dddyouxue_target");
    assert!(!c2.spec.allow_cancel);
    assert!(!c2.spec.options.contains(&ChoiceOption::Player(1)), "current global holder P1 excluded");
    assert!(c2.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(c2.spec.options.contains(&ChoiceOption::Player(2)));
    let p2_before = e.state_core().player(2).hand.len();
    // P2 picks P3 (dist(P1, P3) = 2 in 4-seat ring!) -> draws 2!
    answer(&mut e, 2, vec![ChoiceOption::Player(3)]);
    assert_eq!(e.state_core().player(2).hand.len(), p2_before + 2);

    // Advancing P0 -> P1 -> P2 -> P3 within Round 2 must NOT trigger roundStart `dddyouxue` again!
    for seat in 0u8..3 {
        e.debug_set_turn(seat, Phase::End, 2);
        submit(&mut e, seat, GameAction::NextPhase);
        assert!(e.open_choice().is_none(), "dddyouxue must not re-fire mid-round at seat {seat}");
    }
    // Round 3 wrap (P3 End -> P0 TurnBegin): triggers again and forces target selection (`allow_cancel: false`)!
    e.debug_set_turn(3, Phase::End, 2);
    submit(&mut e, 3, GameAction::NextPhase);
    let c_r3 = e.open_choice().expect("dddyouxue triggers again on round 3 start").clone();
    assert_eq!(c_r3.spec.prompt, "dddyouxue_target");
    assert!(!c_r3.spec.allow_cancel);
    assert!(!c_r3.spec.options.contains(&ChoiceOption::Player(3)), "current holder P3 excluded");
    let rej_r3 = submit(&mut e, c_r3.observer, GameAction::Answer { choice_id: c_r3.id, selection: vec![] });
    assert!(matches!(rej_r3, CommandOutcome::Rejected(_)));
    let first_r3 = c_r3.spec.options[0];
    answer(&mut e, c_r3.observer, vec![first_r3]);
    let c_r3b = e.open_choice().unwrap().clone();
    let first_r3b = c_r3b.spec.options[0];
    answer(&mut e, c_r3b.observer, vec![first_r3b]);
}

/// (08) `dddchengjing` (`skill.js:5018-5130`)
/// - Global `phaseEnd` on `dddyouxue` mark holder's turn: stores LAST physical single-card basic or instant trick still in discard.
/// - Excludes equips, delayed tricks, or virtual/moved cards no longer in discard.
/// - Active `dddchengjing_use` (`usable: 1`): disabled when no pile card; converts 1 `he` card as pile card's kind;
///   at `useCardAfter` (if physical card still in discard), optional give to `mark_holder.next_alive()`!
#[test]
fn u01_fix3_08_dddchengjing() {
    let (mut e, d3, p9) = new_clean_engine(710, 4);
    e.debug_grant(0, "dddyouxue");
    e.debug_grant(0, "dddchengjing");
    let cj_def = e.catalog().skill_by_name("dddchengjing").unwrap();

    // Negative: without any card stored in `dddchengjing` pile, activating fails (cannot view-as default Sha!).
    let sha0 = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, sha0);
    e.debug_set_turn(0, Phase::Play, 1);
    let rej = submit(&mut e, 0, GameAction::ActivateSkill { skill: cj_def });
    assert!(matches!(rej, CommandOutcome::Rejected(_)), "dddchengjing inactive when pile is empty");

    // Set global mark on P1 (`dddyouxue` Round 2 wrap).
    e.debug_set_turn(3, Phase::End, 1);
    submit(&mut e, 3, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);

    // P1's turn: P1 plays Equip (`Zhuge`), Delayed Trick (`Bing`), AND a converted card (`武圣` Red Tao -> Sha) -> MUST NOT store into `dddchengjing` at End!
    let eq = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    let bing = find_card_by(&e, |_, d| d.kind == CardKind::Bing);
    let red_tao = find_card_by(&e, |_, d| d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond));
    e.debug_grant(1, "武圣");
    e.debug_set_turn(1, Phase::Play, 2);
    put_in_hand(&mut e, 1, eq);
    put_in_hand(&mut e, 1, bing);
    put_in_hand(&mut e, 1, red_tao);
    submit(&mut e, 1, GameAction::PlayCard { card: eq, targets: vec![] });
    submit(&mut e, 1, GameAction::PlayCard { card: bing, targets: vec![2] });
    submit(&mut e, 1, GameAction::PlayCard { card: red_tao, targets: vec![2] });
    drain(&mut e);
    if let Some(ws_def) = e.catalog().skill_by_name("武圣") {
        e.state_core_mut().skills.remove(&(1, ws_def));
    }
    e.debug_set_turn(1, Phase::End, 2);
    submit(&mut e, 1, GameAction::NextPhase);
    assert!(e.state_core().vm_aux.piles.get(&(0, "dddchengjing".into())).is_none_or(Vec::is_empty), "equip/delay/converted card do not populate dddchengjing");

    // Next turn (P1 is STILL global mark holder!): P1 uses `Sha` then instant trick `Wuzhong` -> last qualifying card is `Wuzhong`!
    let sha1 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Sha);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    e.debug_set_turn(1, Phase::Play, 2);
    put_in_hand(&mut e, 1, sha1);
    put_in_hand(&mut e, 1, wuzhong);
    submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![2] });
    submit(&mut e, 1, GameAction::PlayCard { card: wuzhong, targets: vec![] });
    drain(&mut e);
    e.debug_set_turn(1, Phase::End, 2);
    submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddchengjing".into())).map(Vec::as_slice), Some(&[wuzhong][..]));

    // Now P0 Play phase: P0 activates `dddchengjing` using an EQUIP card (`eq_p0`) as `Wuzhong` (draws 2), then optional give to `mark_holder(P1).next_alive() = P2`!
    let eq_p0 = find_card_by(&e, |_, d| d.kind == CardKind::Zhangba);
    put_in_equip(&mut e, 0, eq_p0);
    e.debug_set_turn(0, Phase::Play, 2);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: cj_def });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "dddchengjing activate failed: {out:?}");
    let cc = e.open_choice().unwrap().clone();
    assert_eq!(cc.spec.prompt, "dddchengjing_card");
    assert!(cc.spec.options.contains(&ChoiceOption::Card(eq_p0)));
    let p0_before = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Card(eq_p0)]);
    // Wuzhong draws 2, then useCardAfter prompts `dddchengjing_give_confirm` to give `eq_p0` to `P1.next_alive() = P2`!
    assert_eq!(e.state_core().player(0).hand.len(), p0_before + 2);
    let cg = e.open_choice().expect("dddchengjing_give_confirm after card use").clone();
    assert_eq!(cg.observer, 0);
    assert_eq!(cg.spec.prompt, "dddchengjing_give_confirm");
    assert!(cg.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);

    // Choice A: confirm -> P2 gains eq_p0!
    let mut e_give = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_give, 0, vec![ChoiceOption::Confirm]);
    assert!(e_give.state_core().player(2).hand.contains(&eq_p0));

    // Choice B: cancel -> eq_p0 stays in discard!
    answer(&mut e, 0, vec![ChoiceOption::Pass]);
    assert!(e.state_core().discard.contains(&eq_p0));
    assert!(!e.state_core().player(2).hand.contains(&eq_p0));

    // Round 3: P1 uses physical `Jiedao` -> stored into `dddchengjing`, then P0 uses `dddchengjing` as 2-target `Jiedao` `[holder=P2, victim=P3]`!
    let jiedao = find_card_by(&e, |_, d| d.kind == CardKind::Jiedao);
    let eq_p2 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    put_in_equip(&mut e, 2, eq_p2);
    e.debug_set_hp(3, 4);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 2);
    clear_hand(&mut e, 3);
    put_in_hand(&mut e, 1, jiedao);
    e.debug_set_turn(1, Phase::Play, 3);
    let out_jd = submit(&mut e, 1, GameAction::PlayCard { card: jiedao, targets: vec![2, 3] });
    assert!(!matches!(out_jd, CommandOutcome::Rejected(_)), "P1 physical Jiedao failed: {out_jd:?}");
    while let Some(nw) = e.open_choice().filter(|c| c.spec.prompt == "nullify_window").cloned() {
        answer(&mut e, nw.observer, vec![]);
    }
    let cj_p1 = e.open_choice().expect("Jiedao asks P2 jiedao_sha_or_give").clone();
    assert_eq!((cj_p1.observer, cj_p1.spec.prompt), (2, "jiedao_sha_or_give"));
    answer(&mut e, 2, vec![ChoiceOption::Tag(0)]); // P2 hits P3 (4 -> 3) and keeps weapon eq_p2
    drain(&mut e);
    assert_eq!(e.state_core().player(3).hp, 3);
    e.debug_set_turn(1, Phase::End, 3);
    submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddchengjing".into())).map(Vec::as_slice), Some(&[jiedao][..]));
    let cy_r3 = e.open_choice().expect("Round 3 TurnBegin prompts dddyouxue_target").clone();
    assert_eq!(cy_r3.spec.prompt, "dddyouxue_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);

    // P0 activates `dddchengjing` as `Jiedao`: chooses weapon holder P2, then 2nd target victim P3 via `dddchengjing_jiedao_victim`!
    let cost_c = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, cost_c);
    e.debug_set_turn(0, Phase::Play, 3);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: cj_def });
    answer(&mut e, 0, vec![ChoiceOption::Card(cost_c)]);
    let ct1 = e.open_choice().expect("dddchengjing Jiedao weapon holder target").clone();
    assert_eq!(ct1.spec.prompt, "dddchengjing_target");
    assert!(ct1.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    let ct2 = e.open_choice().expect("dddchengjing_jiedao_victim 2nd target").clone();
    assert_eq!(ct2.spec.prompt, "dddchengjing_jiedao_victim");
    assert!(!ct2.spec.allow_cancel);
    assert_eq!(ct2.spec.min, 1);
    assert!(ct2.spec.options.contains(&ChoiceOption::Player(3)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(3)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    let cj_sha = e.open_choice().expect("resolved dddchengjing Jiedao forces holder P2 choice").clone();
    assert_eq!((cj_sha.observer, cj_sha.spec.prompt), (2, "jiedao_sha_or_give"));
    answer(&mut e, 2, vec![ChoiceOption::Tag(0)]); // P2 deals 1 damage to P3 (3 -> 2)!
    assert_eq!(e.state_core().player(3).hp, 2, "dddchengjing Jiedao must resolve against 2nd target victim P3");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddchengjing_give_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Pass]);
}

/// (09) `dddduanbing` (`skill.js:5131-5222`)
/// - NO `once_per_turn` limit! Reusable in Play phase whenever black basic/equip exists and `!has_judge(Bing)`.
/// - Puts black basic/equip as `Bing` in own judge, draws 2, optional nodistance virtual `Sha` (does NOT unlock normal Sha distance!).
/// - `DamageSource` ONLY when caused by a CARD (NOT non-card skill damage): transfers ONLY `Bing` judge cards (not `Le` / `ShanDian`), only if target has no `Bing`.
#[test]
fn u01_fix3_09_dddduanbing() {
    let (mut e, d3, p9) = new_clean_engine(711, 4);
    e.debug_grant(0, "dddduanbing");
    let db_def = e.catalog().skill_by_name("dddduanbing").unwrap();

    // Negative: red card or trick cannot be picked; only black basic/equip accepted!
    let red_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Heart | Suit::Diamond));
    let blk_sha = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club));
    let blk_sha2 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club));
    let blk_eq = find_card_by(&e, |_, d| d.kind.equip_slot().is_some() && matches!(d.suit, Suit::Spade | Suit::Club));
    let le = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    put_in_hand(&mut e, 0, red_sha);
    e.debug_set_turn(0, Phase::Play, 1);
    let rej_red = submit(&mut e, 0, GameAction::ActivateSkill { skill: db_def });
    assert!(matches!(rej_red, CommandOutcome::Rejected(_)), "only black basic/equip activates dddduanbing");

    // Put `Le` in P0's judge too — it must NEVER be transferred by `dddduanbing_effect`!
    put_in_judge(&mut e, 0, le);
    put_in_hand(&mut e, 0, blk_sha);
    put_in_equip(&mut e, 0, blk_eq);
    let top1 = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Jiu);
    let top2 = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Tao);
    put_deck_top(&mut e, top2);
    put_deck_top(&mut e, top1);

    // 1st activation: place blk_sha as `Bing` -> draw 2 -> optional virtual `Sha` at distant P2 (distance 2, nodistance!) -> deal Sha card damage -> transfer `Bing` (NOT `Le`!) to P2!
    submit(&mut e, 0, GameAction::ActivateSkill { skill: db_def });
    let cc = e.open_choice().unwrap().clone();
    assert_eq!(cc.spec.prompt, "dddduanbing_card");
    assert!(!cc.spec.options.contains(&ChoiceOption::Card(red_sha)));
    assert!(cc.spec.options.contains(&ChoiceOption::Card(blk_sha)));
    assert!(cc.spec.options.contains(&ChoiceOption::Card(blk_eq)));
    answer(&mut e, 0, vec![ChoiceOption::Card(blk_sha)]);
    assert!(e.state_core().player(0).judge.contains(&blk_sha));
    assert_eq!(e.state_core().declared_kinds.get(&blk_sha).copied(), Some(CardKind::Bing));
    assert!(e.state_core().player(0).hand.contains(&top1));

    let cs = e.open_choice().expect("optional nodistance virtual Sha target").clone();
    assert_eq!(cs.spec.prompt, "dddduanbing_target");
    assert!(cs.spec.allow_cancel);
    assert!(cs.spec.options.contains(&ChoiceOption::Player(2)), "virtual Sha has nodistance");
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    // P2 has no Shan -> Sha hits -> `dddduanbing_transfer` asks to move Bing to P2!
    let ct = e.open_choice().expect("dddduanbing_transfer after card Sha damage").clone();
    assert_eq!(ct.spec.prompt, "dddduanbing_transfer");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(2).judge.contains(&blk_sha), "Bing transferred to P2");
    assert!(e.state_core().player(0).judge.contains(&le), "Le in P0 judge must NOT transfer");

    // Normal PlayCard `Sha` at distant P2 STILL out of range (no permanent unlimited distance bug!)!
    put_in_hand(&mut e, 0, blk_sha2);
    let rej_dist = submit(&mut e, 0, GameAction::PlayCard { card: blk_sha2, targets: vec![2] });
    assert!(matches!(rej_dist, CommandOutcome::Rejected(_)), "normal Sha distance must remain restricted");

    // 2nd activation IN SAME PLAY PHASE now allowed because P0 no longer has `Bing` in judge and still has `blk_eq`!
    let reuse = submit(&mut e, 0, GameAction::ActivateSkill { skill: db_def });
    assert!(!matches!(reuse, CommandOutcome::Rejected(_)), "dddduanbing is not once_per_turn");
    answer(&mut e, 0, vec![ChoiceOption::Card(blk_eq)]);
    // Cancel optional Sha this time.
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddduanbing_target");
    answer(&mut e, 0, vec![]);

    // Cannot activate 3rd time while `blk_eq` (`Bing`) is STILL in P0 judge!
    let blk_sha3 = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club));
    put_in_hand(&mut e, 0, blk_sha3);
    let rej_has_bing = submit(&mut e, 0, GameAction::ActivateSkill { skill: db_def });
    assert!(matches!(rej_has_bing, CommandOutcome::Rejected(_)), "cannot activate while Bing in own judge");

    // Non-card skill damage (`dddxiaheng` at P0 End) MUST NOT trigger `dddduanbing_transfer`!
    e.debug_grant(0, "dddxiaheng");
    let extra = find_nth_card_by(&e, 5, |_, _| true);
    put_in_hand(&mut e, 1, extra);
    e.debug_set_turn(0, Phase::Discard, 1);
    submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    assert!(e.open_choice().is_none(), "non-card skill damage must NOT trigger dddduanbing_effect");
    assert!(e.state_core().player(0).judge.contains(&blk_eq));
}

/// (10) `dddxiaoxing` (`skill.js:5223-5295`)
/// - Opening `assign_general` (`gameDrawBegin`) draws +3 (4 -> 7).
/// - Mid-game `debug_grant` (`init`) immediately draws +3.
/// - Live `+3` attack_range & `+3` max_handcard while skill held & enabled.
/// - Dying trigger `dddxiaoxing_remove_confirm`: Confirm actually removes `"dddxiaoxing"` and reverts +3 modifiers; Pass keeps it.
#[test]
fn u01_fix3_10_dddxiaoxing() {
    let (d3, p9) = build_u01_packs();
    let mut e = Engine::new_with_extensions(712, 4, &[&d3, &p9]).unwrap();
    if let Some(gid) = find_general_with_skill(&e, "dddxiaoxing") {
        let h_before = e.state_core().player(0).hand.len();
        e.assign_general(0, gid).unwrap();
        assert_eq!(e.state_core().player(0).hand.len(), h_before + 3, "gameDrawBegin adds +3 on assign_general");
    }

    // Mid-game gain (`init`): debug_grant on seat 1 draws +3!
    let (mut e2, _d3_2, _p9_2) = new_clean_engine(713, 6);
    assert_eq!(e2.state_core().attack_range(e2.catalog(), 1), 1);
    e2.debug_grant(1, "dddxiaoxing");
    assert_eq!(e2.state_core().player(1).hand.len(), 3, "init draws +3 on grant");
    assert_eq!(e2.state_core().attack_range(e2.catalog(), 1), 4, "+3 attack range");
    assert_eq!(e2.state_core().hand_limit(e2.catalog(), 1), 7, "hp(4) + 3 hand limit");

    // Sha at distance 3 (P1 -> P4 in 6-seat ring) succeeds via attack_range=4; distant P4 accepts!
    let sha = find_card_by(&e2, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e2, 1, sha);
    e2.debug_set_turn(1, Phase::Play, 1);
    let ok_range = submit(&mut e2, 1, GameAction::PlayCard { card: sha, targets: vec![4] });
    assert!(!matches!(ok_range, CommandOutcome::Rejected(_)));
    drain(&mut e2);

    // Discard phase with 7 cards (`max_handcard == 7`) requires 0 discards!
    while e2.state_core().player(1).hand.len() < 7 {
        let c = e2.state_core_mut().deck.pop().unwrap();
        e2.state_core_mut().player_mut(1).hand.push(c);
    }
    e2.debug_set_turn(1, Phase::Play, 1);
    submit(&mut e2, 1, GameAction::NextPhase);
    assert!(e2.open_choice().is_none(), "hand=7 within 4+3 limit requires no discard");

    // Enter Dying: P2 kills/dying-hits P1 to 0 hp -> forced `dddxiaoxing` removal IMMEDIATELY (no confirm prompt), then dying_save!
    let (mut ed, _d3_d, _p9_d) = new_clean_engine(714, 4);
    ed.debug_grant(1, "dddxiaoxing");
    clear_hand(&mut ed, 1);
    let tao_save = find_card_by(&ed, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut ed, 1, tao_save);
    ed.debug_set_hp(1, 1);
    let sha_d = find_card_by(&ed, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut ed, 2, sha_d);
    ed.debug_set_turn(2, Phase::Play, 1);
    submit(&mut ed, 2, GameAction::PlayCard { card: sha_d, targets: vec![1] });
    assert!(!has_skill(&ed, 1, "dddxiaoxing"), "Dying must forcefully remove dddxiaoxing without confirm");
    assert_eq!(ed.state_core().attack_range(ed.catalog(), 1), 1, "removed dddxiaoxing reverts range immediately");
    let cd_save = ed.open_choice().expect("dying_save directly on Dying").clone();
    assert_eq!(cd_save.observer, 1);
    assert_eq!(cd_save.spec.prompt, "dying_save");
    answer(&mut ed, 1, vec![ChoiceOption::Card(tao_save)]);
    drain(&mut ed);
    assert!(ed.state_core().player(1).alive, "Tao saved dying P1");
    assert!(!has_skill(&ed, 1, "dddxiaoxing"), "even after being saved, dddxiaoxing remains removed");
    assert_eq!(ed.state_core().hand_limit(ed.catalog(), 1), 1, "hand limit reverted after dddxiaoxing removed");
}

/// (11) `dddfuyi` (`skill.js:5427-5592`, Coordinator ruling #162 Option A)
/// - Literal upstream `skill.js:5438`: `if (!event.source.countCards("he") < 2) return false;` always false -> `dieAfter` never triggers.
/// - Authentic `dddfuyi_sha`: in Identity mode with alive Lord holding enabled `dddfuyi`, Qun seats get `Sha` `usable + 1`.
#[test]
fn u01_fix3_11_dddfuyi() {
    let (mut e, _d3, _p9) = new_identity_clean_engine(715, 4);
    e.debug_grant(0, "dddfuyi"); // Seat 0 is Lord in Identity mode!
    let qun_gid = (0..e.catalog().general_count() as u16)
        .find(|&gid| e.catalog().general_meta(gid).is_some_and(|m| m.1 == Faction::Qun))
        .expect("at least one Qun general");
    let wei_gid = (0..e.catalog().general_count() as u16)
        .find(|&gid| e.catalog().general_meta(gid).is_some_and(|m| m.1 == Faction::Wei))
        .expect("at least one Wei general");
    e.assign_general(1, qun_gid).unwrap();
    e.assign_general(2, wei_gid).unwrap();

    // Seat 1 (Qun) can use 2 Shas in Play phase when Lord has `dddfuyi`!
    let s1a = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Sha);
    let s1b = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Sha);
    let s1c = find_nth_card_by(&e, 2, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 1, s1a);
    put_in_hand(&mut e, 1, s1b);
    put_in_hand(&mut e, 1, s1c);
    e.debug_set_turn(1, Phase::Play, 1);
    let out1a = submit(&mut e, 1, GameAction::PlayCard { card: s1a, targets: vec![2] });
    assert!(!matches!(out1a, CommandOutcome::Rejected(_)), "1st Sha failed: {out1a:?}");
    drain(&mut e);
    let out1b = submit(&mut e, 1, GameAction::PlayCard { card: s1b, targets: vec![2] });
    assert!(!matches!(out1b, CommandOutcome::Rejected(_)), "Qun gets +1 Sha usable from Lord dddfuyi: {out1b:?}");
    drain(&mut e);
    assert!(matches!(submit(&mut e, 1, GameAction::PlayCard { card: s1c, targets: vec![2] }), CommandOutcome::Rejected(_)), "3rd Sha still rejected");

    // Seat 2 (Wei, non-Qun) only gets normal 1 Sha! Advance P1's turn cleanly first so turn Sha flags reset.
    e.debug_set_turn(1, Phase::End, 1);
    submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    let s2a = find_nth_card_by(&e, 3, |_, d| d.kind == CardKind::Sha);
    let s2b = find_nth_card_by(&e, 4, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 2, s2a);
    put_in_hand(&mut e, 2, s2b);
    e.debug_set_turn(2, Phase::Play, 1);
    let out2a = submit(&mut e, 2, GameAction::PlayCard { card: s2a, targets: vec![1] });
    assert!(!matches!(out2a, CommandOutcome::Rejected(_)), "Wei 1st Sha failed: {out2a:?}");
    drain(&mut e);
    assert!(matches!(submit(&mut e, 2, GameAction::PlayCard { card: s2b, targets: vec![1] }), CommandOutcome::Rejected(_)), "Wei gets no dddfuyi_sha bonus");

    // Literal `skill.js:5438` (coordinator #162 Option A): killing someone with <2 cards does NOT trigger `dddfuyi` death recovery.
    e.debug_set_turn(2, Phase::End, 1);
    submit(&mut e, 2, GameAction::NextPhase);
    drain(&mut e);
    e.debug_set_hp(3, 1);
    e.debug_set_hp(0, 2);
    let kill_sha = find_nth_card_by(&e, 5, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, kill_sha);
    e.debug_set_turn(0, Phase::Play, 2);
    let out_kill = submit(&mut e, 0, GameAction::PlayCard { card: kill_sha, targets: vec![3] });
    assert!(!matches!(out_kill, CommandOutcome::Rejected(_)), "kill_sha failed: {out_kill:?}");
    drain(&mut e);
    assert!(!e.state_core().player(3).alive);
    assert!(e.open_choice().is_none(), "Option A: literal `(!n)<2` never triggers dddfuyi death prompt");
    assert_eq!(e.state_core().player(0).hp, 2);

    // Locked `dddfuyi`: `tieji` seal on Lord Seat 0 must NOT block Qun's +1 Sha usable!
    e.debug_set_turn(0, Phase::End, 2);
    submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    e.debug_set_hp(2, 4);
    e.state_core_mut().vm_aux.flags.insert((0, "fz_new_tieji_target_sealed".to_string()), 1);
    assert!(e.catalog().is_tieji_sealed(e.state_core(), 0));
    let s1d = find_nth_card_by(&e, 6, |_, d| d.kind == CardKind::Sha);
    let s1e = find_nth_card_by(&e, 7, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 1, s1d);
    put_in_hand(&mut e, 1, s1e);
    e.debug_set_turn(1, Phase::Play, 2);
    let res1d = submit(&mut e, 1, GameAction::PlayCard { card: s1d, targets: vec![2] });
    assert!(!matches!(res1d, CommandOutcome::Rejected(_)), "1st Sha failed: {res1d:?}");
    drain(&mut e);
    let res1e = submit(&mut e, 1, GameAction::PlayCard { card: s1e, targets: vec![2] });
    assert!(!matches!(res1e, CommandOutcome::Rejected(_)), "locked dddfuyi ignores tieji seal: {res1e:?}");
    drain(&mut e);

    // 2 alive Lords both holding `dddfuyi` stack to +2 Sha usable (3 Shas total for Qun Seat 1)!
    let (mut e_two, _, _) = new_identity_clean_engine(7152, 4);
    e_two.debug_set_identity_roles(vec![
        engine_native::Role::Lord,
        engine_native::Role::Rebel,
        engine_native::Role::Loyalist,
        engine_native::Role::Lord,
    ]);
    e_two.debug_grant(0, "dddfuyi");
    e_two.debug_grant(3, "dddfuyi");
    e_two.assign_general(1, qun_gid).unwrap();
    let qs: Vec<u32> = (0..4)
        .map(|i| find_nth_card_by(&e_two, i, |_, d| d.kind == CardKind::Sha))
        .collect();
    for &c in &qs {
        put_in_hand(&mut e_two, 1, c);
    }
    e_two.debug_set_turn(1, Phase::Play, 1);
    for (idx, &c) in qs[0..3].iter().enumerate() {
        let res = submit(&mut e_two, 1, GameAction::PlayCard { card: c, targets: vec![2] });
        assert!(!matches!(res, CommandOutcome::Rejected(_)), "Sha #{} failed with 2 dddfuyi Lords: {res:?}", idx + 1);
        drain(&mut e_two);
    }
    assert!(matches!(submit(&mut e_two, 1, GameAction::PlayCard { card: qs[3], targets: vec![2] }), CommandOutcome::Rejected(_)), "4th Sha rejected when base 1 + 2 Lords = 3");
}

/// (12) `dddzhuanshe` (`skill.js:5593-5713`)
/// - `phaseUseBegin` (`player != owner`): optionally give 1 hand card to `player`.
/// - `useCard2` when `player` uses same-name basic/instant trick in turn: optionally add 1 unselected legal target!
/// - `phaseEnd`: ONLY if `player` NEVER used a card of that given card's name this turn -> optional 1 damage to `player`!
#[test]
fn u01_fix3_12_dddzhuanshe() {
    let (mut e, d3, p9) = new_clean_engine(716, 4);
    e.debug_grant(0, "dddzhuanshe");
    let given_sha = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Sha);
    let given_tao = find_nth_card_by(&e, 0, |_, d| d.kind == CardKind::Tao);
    let given_fire_sha = find_card_by(&e, |_, d| d.kind == CardKind::FireSha);
    let play_sha = find_nth_card_by(&e, 1, |_, d| d.kind == CardKind::Sha);
    let play_lei_sha = find_card_by(&e, |_, d| d.kind == CardKind::LeiSha);
    put_in_hand(&mut e, 0, given_sha);
    put_in_hand(&mut e, 0, given_tao);
    put_in_hand(&mut e, 0, given_fire_sha);
    put_in_hand(&mut e, 1, play_sha);

    // Turn 1 (P1): P0 gives `given_sha` at `phaseUseBegin`.
    e.debug_set_turn(1, Phase::Draw, 1);
    submit(&mut e, 1, GameAction::NextPhase); // Draw -> Play (`phaseUseBegin`)
    let cg = e.open_choice().expect("dddzhuanshe_give at P1 phaseUseBegin").clone();
    assert_eq!(cg.observer, 0);
    assert_eq!(cg.spec.prompt, "dddzhuanshe_give");
    answer(&mut e, 0, vec![ChoiceOption::Card(given_sha)]);
    assert!(e.state_core().player(1).hand.contains(&given_sha));

    // P1 uses `play_sha` targeting P2 -> `dddzhuanshe_add_target` lets P0 add another legal target (e.g. P0 itself!)!
    submit(&mut e, 1, GameAction::PlayCard { card: play_sha, targets: vec![2] });
    let ca = e.open_choice().expect("dddzhuanshe_add_target when P1 uses matching Sha").clone();
    assert_eq!(ca.observer, 0);
    assert_eq!(ca.spec.prompt, "dddzhuanshe_add_target");
    assert!(!ca.spec.options.contains(&ChoiceOption::Player(2)), "existing target P2 excluded");
    assert!(ca.spec.options.contains(&ChoiceOption::Player(0)), "P0 within P1 range");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(0)]);
    let hp0_before = e.state_core().player(0).hp;
    let hp2_before = e.state_core().player(2).hp;
    answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
    assert_eq!(e.state_core().player(2).hp, hp2_before - 1);
    assert_eq!(e.state_core().player(0).hp, hp0_before - 1, "added extra target P0 also hit by Sha");

    // Advance P1 to `End` -> since P1 USED `Sha` this turn, `dddzhuanshe_damage` must NOT prompt!
    e.debug_set_turn(1, Phase::End, 1);
    clear_hand(&mut e, 1);
    submit(&mut e, 1, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "P1 used Sha this turn -> no dddzhuanshe_damage at End");

    // Turn 2 (P1): P0 gives `given_tao`, P1 NEVER uses `Tao` -> at `End` (`TurnEndWindow { after: false }`), `dddzhuanshe_damage` prompts!
    e.debug_set_turn(1, Phase::Draw, 2);
    submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Card(given_tao)]);
    e.debug_set_turn(1, Phase::End, 2);
    clear_hand(&mut e, 1);
    submit(&mut e, 1, GameAction::NextPhase);
    let cd = e.open_choice().expect("dddzhuanshe_damage when given Tao was unused all turn").clone();
    assert_eq!(cd.observer, 0);
    assert_eq!(cd.spec.prompt, "dddzhuanshe_damage");
    assert!(cd.spec.allow_cancel);
    let hp1_before = e.state_core().player(1).hp;
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);

    // Turn 3 (P1): FireSha/LeiSha/Sha normalize to `"sha"` (`Sha | FireSha | LeiSha` match same name)!
    put_in_hand(&mut e, 1, play_lei_sha);
    e.debug_set_turn(1, Phase::Draw, 3);
    submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Card(given_fire_sha)]);
    submit(&mut e, 1, GameAction::PlayCard { card: play_lei_sha, targets: vec![2] });
    let ca_elem = e.open_choice().expect("LeiSha must match given FireSha via Sha normalization").clone();
    assert_eq!(ca_elem.spec.prompt, "dddzhuanshe_add_target");
    assert!(ca_elem.spec.options.contains(&ChoiceOption::Player(0)));
    answer(&mut e, 0, vec![]);
    drain(&mut e);
    e.debug_set_turn(1, Phase::End, 3);
    clear_hand(&mut e, 1);
    submit(&mut e, 1, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "using LeiSha satisfies given FireSha -> no dddzhuanshe_damage at End");
}

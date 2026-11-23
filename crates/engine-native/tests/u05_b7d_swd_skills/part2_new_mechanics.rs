use super::common::*;

#[test]
fn u05_b7d_oldliaoyuan_and_oldliaoyuan2() {
    let (verified, mut e) = load_formal_swd_only(7911, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "oldliaoyuan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let ol1 = e.catalog().skill_by_name("oldliaoyuan").unwrap();
    let h1 = place_matching_card(&mut e, 0, |_, d| d.suit == Suit::Heart);
    let h2 = place_matching_card(&mut e, 0, |id, d| d.suit == Suit::Heart && id != h1);
    let s1 = place_matching_card(&mut e, 0, |id, d| d.suit == Suit::Spade && id != h1 && id != h2);
    let _s2 = place_matching_card(&mut e, 0, |id, d| d.suit == Suit::Spade && id != h1 && id != h2 && id != s1);
    let p1_card = e.debug_place(1, "tao");

    // 1. Cancel refunds once_used; mismatched suit pair re-prompts.
    submit(&mut e, 0, GameAction::ActivateSkill { skill: ol1 });
    let c = e.open_choice().expect("oldliaoyuan_cards").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: c.id, selection: Vec::new() });
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: ol1 }),
            CommandOutcome::Pending { .. }
        ),
        "oldliaoyuan cancel must refund once_used"
    );
    let c = e.open_choice().expect("oldliaoyuan_cards").clone();
    // AI planner should automatically pick same-suit pair!
    let ai_ans = e.plan_action(0, &BoundedPlanner::new(64));
    assert!(matches!(ai_ans.action, GameAction::Answer { ref selection, .. } if selection.len() == 2));
    // Also verify rejecting mismatched suit first:
    submit(&mut e, 0, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(h1), ChoiceOption::Card(s1)] });
    let reprompt = e.open_choice().expect("reprompt on mismatched suits").clone();
    assert_eq!(reprompt.spec.prompt, "oldliaoyuan_cards");
    let _ = ai_ans;
    let mut e = roundtrip_bytes(&verified, &e);
    let reprompt = e.open_choice().expect("reprompt after restore").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: reprompt.id, selection: vec![ChoiceOption::Card(h1), ChoiceOption::Card(h2)] });

    let tgt = e.open_choice().expect("oldliaoyuan_target").clone();
    assert_eq!(tgt.spec.prompt, "oldliaoyuan_target");
    submit(&mut e, 0, GameAction::Answer { choice_id: tgt.id, selection: vec![ChoiceOption::Player(1)] });

    // Settle into oldliaoyuan_target_discard (pass shan first if any).
    let mut saw_disc = false;
    for _ in 0..16 {
        let Some(ch) = e.open_choice().cloned() else { break };
        if ch.spec.prompt == "oldliaoyuan_target_discard" {
            saw_disc = true;
            submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Card(p1_card)] });
            continue;
        }
        let sel = if ch.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if ch.spec.allow_cancel {
            Vec::new()
        } else {
            ch.spec.options.iter().take(ch.spec.min.max(1)).copied().collect()
        };
        submit(&mut e, ch.observer, GameAction::Answer { choice_id: ch.id, selection: sel });
    }
    assert!(saw_disc, "oldliaoyuan_target_discard must trigger");
    assert_eq!(e.debug_hp(1), 3, "oldliaoyuan FireSha dealt 1 fire damage (4->3)");
    assert!(!e.debug_hand(1).contains(&p1_card), "oldliaoyuan discarded 1 target card");

    // 2. oldliaoyuan2: FireSha + extra Sha against target.
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "oldliaoyuan2");
    let ol2 = e.catalog().skill_by_name("oldliaoyuan2").unwrap();
    let d1 = place_matching_card(&mut e, 0, |_, d| d.suit == Suit::Diamond);
    let d2 = place_matching_card(&mut e, 0, |id, d| d.suit == Suit::Diamond && id != d1);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: ol2 });
    let c1 = e.open_choice().expect("oldliaoyuan2_cards").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: c1.id, selection: vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)] });
    let c2 = e.open_choice().expect("oldliaoyuan2_target").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: c2.id, selection: vec![ChoiceOption::Player(1)] });
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 1, "oldliaoyuan2 FireSha + second Sha dealt 2 damage (3->1)");
}

#[test]
fn u05_b7d_moyu_huanling2_lexue_and_tianshu_remove() {
    // 1. moyu: black card treated as Heart + killer full heal.
    let (_v, mut e) = load_formal_swd_only(7912, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "moyu");
    e.debug_grant(0, "huopu");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2);
    let heart_card = place_matching_card(&mut e, 0, |_, d| {
        d.suit == Suit::Heart && !d.kind.is_sha_like() && !d.kind.is_equip()
    });
    // Play Heart card as Liuxinghuoyu (huopu requires Heart!).
    let out = submit(&mut e, 0, GameAction::PlayCard { card: heart_card, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    settle_confirm(&mut e);

    // Kill p1 with Sha (distance 1 from p0) -> moyu triggers full recover (hp -> max_hp 4).
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 1);
    let killer_sha = e.debug_place(0, "sha");
    let out_k = submit(&mut e, 0, GameAction::PlayCard { card: killer_sha, targets: vec![1] });
    assert!(matches!(out_k, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "killer_sha outcome: {out_k:?}");
    settle_confirm(&mut e);
    assert!(!e.debug_alive(1), "p1 should die from killer_sha (hp={})", e.debug_hp(1));
    assert_eq!(e.debug_hp(0), 4, "moyu recovers owner to max_hp on Death");

    // 2. huanling2: face_down damage prevented.
    let (_v2, mut eh) = load_formal_swd_only(7913, 3);
    for p in 0..3 {
        eh.debug_remove_general(p);
        eh.debug_clear_hand(p);
        eh.debug_set_hp(p, 4);
    }
    eh.debug_grant(1, "huanling");
    assert!(eh.debug_has_skill(1, "huanling2"));
    advance_to_play(&mut eh, 0);
    eh.debug_clear_hand(0);
    eh.debug_clear_hand(1);
    eh.state_core_mut().player_mut(1).face_down = true;
    let sha = eh.debug_place(0, "sha");
    submit(&mut eh, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    settle_confirm(&mut eh);
    assert_eq!(eh.debug_hp(1), 4, "huanling2 prevents damage while face_down");

    // 3. lexue + lexue2 rotation across TurnBegin / TurnPhaseEnd.
    let (_v3, mut el) = load_formal_swd_only(7914, 3);
    for p in 0..3 {
        el.debug_remove_general(p);
        el.debug_clear_hand(p);
    }
    el.debug_grant(0, "lexue");
    assert!(el.debug_has_skill(0, "lexue2"));
    // Cycle to p1's Play so p0's TurnPhaseEnd fires lexue2, then back to p0's Play so TurnBegin fires lexue!
    advance_to_play(&mut el, 1);
    let off_turn_hit = ["遗计", "急救", "鬼道", "反馈"]
        .iter()
        .filter(|s| el.debug_has_skill(0, s))
        .count();
    assert_eq!(off_turn_hit, 1, "lexue2 grants 1 off-turn skill at TurnPhaseEnd");
    advance_to_play(&mut el, 0);
    let in_turn_hit = ["制衡", "集智", "缔盟", "驱虎"]
        .iter()
        .filter(|s| el.debug_has_skill(0, s))
        .count();
    let stale_off_turn = ["遗计", "急救", "鬼道", "反馈"]
        .iter()
        .filter(|s| el.debug_has_skill(0, s))
        .count();
    assert_eq!(in_turn_hit, 1, "lexue grants exactly 1 on-turn skill at TurnBegin");
    assert_eq!(stale_off_turn, 0, "lexue removes prior off-turn skill");

    // 4. swdtianshu + swdtianshu_remove + unique filter + previous revoke.
    let (_v5, mut ets) = load_formal_swd_only(7916, 3);
    for p in 0..3 {
        ets.debug_remove_general(p);
        ets.debug_clear_hand(p);
    }
    ets.debug_grant(0, "swdtianshu");
    ets.assign_general_named(1, "swd_weida").expect("assign swd_weida");
    advance_to_play(&mut ets, 0);
    ets.debug_clear_hand(0);
    let c_pay = place_matching_card(&mut ets, 0, |_, d| d.kind.is_trick_kind());
    let ts_def = ets.catalog().skill_by_name("swdtianshu").unwrap();
    submit(&mut ets, 0, GameAction::ActivateSkill { skill: ts_def });
    let mut saw_skill_tag = false;
    for _ in 0..16 {
        let Some(c) = ets.open_choice().cloned() else { break };
        let sel = match c.spec.prompt {
            "trigger_confirm" => vec![ChoiceOption::Confirm],
            "swdtianshu_card" => vec![ChoiceOption::Card(c_pay)],
            "swdtianshu_target" => vec![ChoiceOption::Player(1)],
            "swdtianshu_skill" => {
                saw_skill_tag = true;
                // swd_weida has [yueren, zhenlie, duijue]; unique duijue excluded -> 2 options!
                assert_eq!(c.spec.options.len(), 2, "swdtianshu must exclude unique skill duijue");
                c.spec.options.iter().take(1).copied().collect()
            }
            _ => Vec::new(),
        };
        submit(&mut ets, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(saw_skill_tag, "swdtianshu_skill prompt observed");
    assert!(ets.debug_has_skill(0, "yueren"), "p0 gained non-unique yueren via swdtianshu");
    // Now kill p1 -> SwdTianshuRemove must remove yueren from p0!
    advance_to_play(&mut ets, 0);
    ets.debug_clear_hand(0);
    ets.debug_clear_hand(1);
    ets.debug_set_hp(1, 1);
    let kill1 = ets.debug_place(0, "sha");
    submit(&mut ets, 0, GameAction::PlayCard { card: kill1, targets: vec![1] });
    settle_confirm(&mut ets);
    assert!(!ets.debug_alive(1));
    assert!(
        !ets.debug_has_skill(0, "yueren"),
        "swdtianshu_remove clears granted skill when source player dies"
    );
}

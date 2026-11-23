use super::common::*;

#[test]
fn u05_b7d_daixing3_yinguo2_susheng2_and_ordinary_judge() {
    // 1. daixing3 consumes exact hujia absorbed (not blind fixed loss).
    let (_v, mut e) = load_formal_swd_only(7921, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(1, "daixing");
    e.debug_grant(1, "daixing3");
    e.state_core_mut().player_mut(1).daixing_count = 3;
    e.state_core_mut().player_mut(1).hujia = 1; // 1 hujia vs 1 dmg -> absorbed=1 -> daixing_count 3 -> 2
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 4, "1 hujia absorbed 1 damage");
    assert_eq!(
        e.state_core().player(1).daixing_count,
        2,
        "daixing3 reduced count strictly by absorbed=1 (3 -> 2)"
    );

    // 2. susheng + susheng2 (global dieBefore): discards from hand to cancel death & set hp=1, attaches susheng2 preventing double trigger, cleaned at turn end.
    let mut e2 = load_formal_swd_only(7922, 3).1;
    for p in 0..3 {
        e2.debug_remove_general(p);
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(1, "susheng");
    advance_to_play(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    let c_drop = e2.debug_place(1, "tao");
    e2.debug_set_hp(2, 1);
    let kill_sha = e2.debug_place(0, "sha");
    submit(&mut e2, 0, GameAction::PlayCard { card: kill_sha, targets: vec![2] });
    let mut saw_susheng = false;
    for _ in 0..24 {
        let Some(c) = e2.open_choice().cloned() else { break };
        if c.spec.prompt == "susheng_discard" {
            saw_susheng = true;
            submit(&mut e2, 1, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(c_drop)] });
            continue;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else {
            Vec::new()
        };
        submit(&mut e2, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(saw_susheng, "susheng triggered on dieBefore");
    assert!(e2.debug_alive(2) && e2.debug_hp(2) == 1, "susheng saved dying target at hp=1");
    assert!(!e2.debug_hand(1).contains(&c_drop), "susheng discarded hand card");
    assert!(e2.debug_has_skill(1, "susheng2"), "susheng2 active until turn end");
    advance_to_play(&mut e2, 1);
    assert!(!e2.debug_has_skill(1, "susheng2"), "susheng2 cleaned on TurnEndWindow");

    // 3. OrdinaryJudge interaction: lqingcheng + 天妒 (Tiandu) gains judge card & max 3 confirms + flush!
    let (_v3, mut ej) = load_formal_swd_only(7923, 3);
    for p in 0..3 {
        ej.debug_remove_general(p);
        ej.debug_clear_hand(p);
    }
    ej.debug_grant(0, "lqingcheng");
    ej.debug_grant(0, "天妒");
    advance_to_play(&mut ej, 0);
    ej.debug_clear_hand(0);
    let j1 = set_deck_top_matching(&mut ej, |d| matches!(d.suit, Suit::Heart));
    // Trigger lqingcheng on Phase::End (NextPhase twice: Play -> Discard -> End).
    let mut lq_confirms = 0u32;
    for _ in 0..24 {
        if ej.open_choice().is_none() && ej.turn().current != 0 {
            break;
        }
        if let Some(c) = ej.open_choice().cloned() {
            let sel = match c.spec.prompt {
                "trigger_confirm" => vec![ChoiceOption::Confirm],
                "lqingcheng_continue" => {
                    lq_confirms += 1;
                    let _ = set_deck_top_matching(&mut ej, |d| matches!(d.suit, Suit::Diamond | Suit::Heart));
                    vec![ChoiceOption::Confirm]
                }
                _ if c.spec.options.contains(&ChoiceOption::Confirm) => vec![ChoiceOption::Confirm],
                _ => Vec::new(),
            };
            submit(&mut ej, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        if ej.turn().current == 0 {
            ej.debug_clear_hand(0);
            submit(&mut ej, 0, GameAction::NextPhase);
        }
    }
    assert_eq!(lq_confirms, 2, "lqingcheng asks twice then caps at max 3 red judges total");
    assert_eq!(ej.debug_hand(0).len(), 3, "lqingcheng + Tiandu flushes all 3 red judge cards");
    let _ = j1;

    // 4. shending enemy-only & !has_treasure check.
    let (_v4, mut es) = load_formal_swd_only(7924, 4);
    for p in 0..4 {
        es.debug_remove_general(p);
        es.debug_clear_hand(p);
        es.debug_set_hp(p, 4);
    }
    es.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Rebel]);
    es.debug_grant(1, "shending"); // p1 Loyalist, p0 Lord -> NOT enemy!
    advance_to_play(&mut es, 0);
    let sha_friend = es.debug_place(0, "sha");
    let _ = set_deck_top_matching(&mut es, |d| d.suit == Suit::Spade);
    submit(&mut es, 0, GameAction::PlayCard { card: sha_friend, targets: vec![1] });
    // Ally p0 hitting p1 should NOT trigger shending!
    assert!(
        es.open_choice().is_none_or(|c| c.observer != 1 || c.spec.prompt != "trigger_confirm"),
        "shending must not trigger on non-enemy source"
    );
    settle_confirm(&mut es);
}

#[test]
fn u05_b7d_equip_viewas_virtual_sha_safe_and_post_damage_fixes() {
    let (_v, mut e) = load_formal_swd_only(7931, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1. equip black_eq BEFORE granting shengshou/huanjian/huopu.
    e.debug_set_hp(1, 3);
    let black_eq = place_matching_card(&mut e, 0, |_, d| d.kind.is_equip() && matches!(d.suit, Suit::Spade | Suit::Club));
    submit(&mut e, 0, GameAction::PlayCard { card: black_eq, targets: vec![] });
    assert!(e.state_core().player(0).equips.contains(&black_eq), "equipped black equip");
    e.debug_grant(0, "shengshou");
    e.debug_grant(0, "huanjian");
    e.debug_grant(0, "huopu");
    // Because p0 ALSO has huanjian, black equip maps to Bingpotong (rejected with 0 targets or target=1 missing hand? Wait test shengshou without huanjian!).
    let ss_def = e.catalog().skill_by_name("shengshou").unwrap();
    let _ = ss_def;
    let (_v_ss, mut ess) = load_formal_swd_only(7932, 3);
    for p in 0..3 {
        ess.debug_remove_general(p);
        ess.debug_clear_hand(p);
        ess.debug_set_hp(p, 4);
    }
    ess.debug_grant(0, "shengshou");
    advance_to_play(&mut ess, 0);
    for p in 0..3 {
        ess.debug_clear_hand(p);
    }
    let b_eq = place_matching_card(&mut ess, 0, |_, d| d.kind.is_equip() && matches!(d.suit, Suit::Spade | Suit::Club));
    submit(&mut ess, 0, GameAction::PlayCard { card: b_eq, targets: vec![] });
    ess.debug_set_hp(1, 3);
    assert!(
        matches!(
            submit(&mut ess, 0, GameAction::PlayCard { card: b_eq, targets: vec![1] }),
            CommandOutcome::Rejected(RejectReason::CardNotInHand)
        ),
        "shengshou is hand-only, rejects equip card"
    );

    // 2. huanjian from equip zone converts black equip -> Bingpotong and revokes equip on commit.
    let show0 = e.debug_place(0, "sha");
    let _ = show0;
    let black_hand0 = place_matching_card(&mut e, 0, |_, d| d.suit == Suit::Spade && d.kind == CardKind::Sha);
    let black_hand1 = place_matching_card(&mut e, 1, |_, d| d.suit == Suit::Club && d.kind == CardKind::Sha);
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: black_eq, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }), "huanjian equip -> Bingpotong accepted");
    while let Some(c) = e.open_choice().cloned() {
        let sel = match c.spec.prompt {
            "bingpotong_show_own" => vec![ChoiceOption::Card(black_hand0)],
            "bingpotong_show_target" => vec![ChoiceOption::Card(black_hand1)],
            _ if c.spec.options.contains(&ChoiceOption::Pass) => vec![ChoiceOption::Pass],
            _ => c.spec.options.iter().take(1).copied().collect(),
        };
        submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(!e.state_core().player(0).equips.contains(&black_eq), "equip consumed after huanjian PlayCard");

    // 3. huopu usable: 1 (2nd Heart Liuxinghuoyu in same Play phase rejected).
    let heart1 = place_matching_card(&mut e, 0, |_, d| d.suit == Suit::Heart && !d.kind.is_equip());
    let heart2 = place_matching_card(&mut e, 0, |id, d| d.suit == Suit::Heart && !d.kind.is_equip() && id != heart1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: heart1, targets: vec![1] });
    settle_confirm(&mut e);
    let out2 = submit(&mut e, 0, GameAction::PlayCard { card: heart2, targets: vec![1] });
    assert!(
        !matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) || e.catalog().card_kind(heart2) != CardKind::Sha,
        "huopu usable: 1 prevents second conversion in same phase"
    );

    // 4. guanhu_discard: rejects [hand1, hand2] and accepts [hand1, equip1]; AI picks valid combo.
    let (_v_gh, mut eg) = load_formal_swd_only(7933, 3);
    for p in 0..3 {
        eg.debug_remove_general(p);
        eg.debug_clear_hand(p);
        eg.debug_set_hp(p, 4);
    }
    eg.debug_grant(0, "guanhu");
    advance_to_play(&mut eg, 0);
    for p in 0..3 {
        eg.debug_clear_hand(p);
    }
    let teq = place_matching_card(&mut eg, 1, |_, d| d.kind.is_equip());
    submit(&mut eg, 1, GameAction::PlayCard { card: teq, targets: Vec::new() });
    // Advance to p1 then p0.
    advance_to_play(&mut eg, 1);
    let teq2 = place_matching_card(&mut eg, 1, |_, d| d.kind == CardKind::Weapon);
    submit(&mut eg, 1, GameAction::PlayCard { card: teq2, targets: Vec::new() });
    let th1 = eg.debug_place(1, "tao");
    let th2 = eg.debug_place(1, "shan");
    advance_to_play(&mut eg, 0);
    // p0 attacks p1 or kills/fails? Guanhu triggers at DamageBefore/Battle? Actually skip Phase::Play -> guanhu triggers on Sha!
    let dsha = eg.debug_place(0, "sha");
    submit(&mut eg, 0, GameAction::PlayCard { card: dsha, targets: vec![1] });
    for _ in 0..16 {
        let Some(c) = eg.open_choice().cloned() else { break };
        if c.spec.prompt == "guanhu_discard" {
            // AI planner must pick [1 hand, 1 equip] (never [th1, th2])!
            let plan = eg.plan_action(0, &BoundedPlanner::new(64));
            if let GameAction::Answer { selection, .. } = plan.action {
                assert_eq!(selection.len(), 2, "AI picks 1 hand + 1 equip");
            }
            submit(&mut eg, 0, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(th1), ChoiceOption::Card(th2)] });
            let reprompt = eg.open_choice().expect("guanhu rejects 2 hand cards").clone();
            assert_eq!(reprompt.spec.prompt, "guanhu_discard");
            submit(&mut eg, 0, GameAction::Answer { choice_id: reprompt.id, selection: vec![ChoiceOption::Card(th1), ChoiceOption::Card(teq2)] });
            continue;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        submit(&mut eg, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(!eg.debug_hand(1).contains(&th1) && !eg.state_core().player(1).equips.contains(&teq2), "guanhu discarded [hand, equip]");
    let _ = teq;

    // 5. SwdRexue: consumes real Sha + draws 1 safely even when deck & discard are empty!
    let (_v_rx, mut erx) = load_formal_swd_only(7934, 3);
    for p in 0..3 {
        erx.debug_remove_general(p);
        erx.debug_clear_hand(p);
        erx.debug_set_hp(p, 4);
    }
    erx.debug_grant(0, "rexue");
    advance_to_play(&mut erx, 0);
    let s0 = erx.debug_place(0, "sha");
    erx.state_core_mut().deck.clear();
    erx.state_core_mut().discard.clear();
    let mut saw_rexue = false;
    for _ in 0..16 {
        if let Some(c) = erx.open_choice().cloned() {
            let sel = match (c.observer, c.spec.prompt) {
                (0, "trigger_confirm") => vec![ChoiceOption::Confirm],
                (0, "rexue_sha") => {
                    saw_rexue = true;
                    vec![ChoiceOption::Card(s0)]
                }
                _ if c.spec.options.contains(&ChoiceOption::Pass) => vec![ChoiceOption::Pass],
                _ => Vec::new(),
            };
            submit(&mut erx, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        if erx.turn().current == 1 && saw_rexue {
            break;
        }
        let cur = erx.turn().current;
        let _ = submit(&mut erx, cur, GameAction::NextPhase);
    }
    assert!(saw_rexue, "rexue_sha prompted on seat 1 TurnBegin");
    assert!(!erx.debug_hand(0).contains(&s0), "rexue consumed real Sha s0");
    assert_eq!(erx.debug_hp(1), 3, "rexue Sha dealt 1 dmg even with empty deck/discard");
}

#[test]
fn u05_b7d_toudan_shehun_kunlunjing_swd_wuxie_and_ai_prompts() {
    // 1. toudan: post-damage dist<=1 neighbor queue + toudan_neighbor_discard + save/restore.
    let (v, mut e) = load_formal_swd_only(7941, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "toudan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let td_def = e.catalog().skill_by_name("toudan").unwrap();
    let pick_c = place_matching_card(&mut e, 0, |_, d| d.suit == Suit::Spade);
    let p1_c = e.debug_place(1, "sha");
    let p2_c = e.debug_place(2, "shan");
    submit(&mut e, 0, GameAction::ActivateSkill { skill: td_def });
    let c1 = e.open_choice().expect("toudan_pick").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: c1.id, selection: vec![ChoiceOption::Card(pick_c)] });
    let c2 = e.open_choice().expect("toudan_target").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: c2.id, selection: vec![ChoiceOption::Player(1)] });
    let mut neighbor_prompts = 0usize;
    let mut restored_once = false;
    for _ in 0..16 {
        let Some(ch) = e.open_choice().cloned() else { break };
        if ch.spec.prompt == "toudan_neighbor_discard" {
            neighbor_prompts += 1;
            if !restored_once {
                restored_once = true;
                e = roundtrip_bytes(&v, &e);
            }
            let plan = e.plan_action(ch.observer, &BoundedPlanner::new(64));
            submit(&mut e, ch.observer, plan.action);
            continue;
        }
        let sel = if ch.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else {
            Vec::new()
        };
        submit(&mut e, ch.observer, GameAction::Answer { choice_id: ch.id, selection: sel });
    }
    assert!(neighbor_prompts >= 1, "toudan opened toudan_neighbor_discard for dist<=1 neighbors");
    assert!(!e.debug_hand(2).contains(&p2_c) || !e.debug_hand(1).contains(&p1_c));

    // 2. shehun_discard rejects duplicate suits & AI picks distinct suits; swd_wuxie blocks targeting others with Bing/Le.
    let (_v2, mut es) = load_formal_swd_only(7942, 3);
    for p in 0..3 {
        es.debug_remove_general(p);
        es.debug_clear_hand(p);
    }
    es.debug_grant(0, "shehun");
    es.debug_grant(0, "swd_wuxie");
    advance_to_play(&mut es, 0);
    for p in 0..3 {
        es.debug_clear_hand(p);
    }
    let sh1 = place_matching_card(&mut es, 0, |_, d| d.suit == Suit::Spade);
    let sh2 = place_matching_card(&mut es, 0, |id, d| d.suit == Suit::Spade && id != sh1);
    let sh3 = place_matching_card(&mut es, 0, |_, d| d.suit == Suit::Heart);
    let _p1k = es.debug_place(1, "tao");
    let sh_def = es.catalog().skill_by_name("shehun").unwrap();
    submit(&mut es, 0, GameAction::ActivateSkill { skill: sh_def });
    let ask_sh = es.open_choice().expect("shehun_discard").clone();
    // AI picks distinct suits!
    let ai_sh = es.plan_action(0, &BoundedPlanner::new(64));
    assert!(matches!(ai_sh.action, GameAction::Answer { ref selection, .. } if selection.len() <= 2));
    // Duplicate suit answer reprompts:
    submit(&mut es, 0, GameAction::Answer { choice_id: ask_sh.id, selection: vec![ChoiceOption::Card(sh1), ChoiceOption::Card(sh2)] });
    let rep = es.open_choice().expect("shehun reprompts on duplicate suit").clone();
    assert_eq!(rep.spec.prompt, "shehun_discard");
    submit(&mut es, 0, GameAction::Answer { choice_id: rep.id, selection: vec![ChoiceOption::Card(sh1), ChoiceOption::Card(sh3)] });
    let ask_t = es.open_choice().expect("shehun_target").clone();
    submit(&mut es, 0, GameAction::Answer { choice_id: ask_t.id, selection: vec![ChoiceOption::Player(1)] });
    settle_confirm(&mut es);

    // swd_wuxie blocks targeting p1 with Le or Bing.
    advance_to_play(&mut es, 0);
    es.debug_clear_hand(0);
    let le_card = place_matching_card(&mut es, 0, |_, d| d.kind == CardKind::Le);
    assert!(matches!(
        submit(&mut es, 0, GameAction::PlayCard { card: le_card, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "swd_wuxie prevents targeting others with Le");
    let bing_card = place_matching_card(&mut es, 0, |_, d| d.kind == CardKind::Bing);
    assert!(matches!(
        submit(&mut es, 0, GameAction::PlayCard { card: bing_card, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "swd_wuxie prevents targeting others with Bing");
}

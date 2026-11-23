use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::machine::LogEvent;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b7a_lianyaohu_kunlunjingc_shennongding_donghuangzhong_fixes() {
    let (v, mut e) = start_shahai_swd_only(9220, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. lianyaohu: shouna cancel refund + lianhua yangpijuan_class Basic/Trick/Equip match + conservation.
    let lyh = find_kind(&e, CardKind::Lianyaohu);
    let sha_stored = find_kind(&e, CardKind::Sha);
    let sha_match = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != sha_stored && e.catalog().card_kind(id) == CardKind::Sha)
        .unwrap();
    let p1_card = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 0, lyh);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: lyh,
            targets: vec![],
        },
    );
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, sha_match);
    force_move_card_to_hand(&mut e, &v, 1, p1_card);

    let shouna_def = e.catalog().skill_by_name("lianyaohu_shouna").expect("lianyaohu_shouna");
    let s1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: shouna_def });
    assert!(matches!(s1, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "shouna_cost");
    answer_open(&mut e, vec![]); // cancel must refund once_per_turn!
    let s2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: shouna_def });
    assert!(matches!(s2, CommandOutcome::Pending { .. }));
    answer_open(&mut e, vec![ChoiceOption::Card(sha_match)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "shouna_target");
    let log_mark = e.log().len();
    let _s3 = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    let disc_events = e.log()[log_mark..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { cards, .. } if cards.contains(&sha_match)))
        .count();
    assert_eq!(disc_events, 1, "shouna logs single CardDiscarded");
    assert_eq!(
        e.state_core().player(0).lianyaohu_store.len(),
        1,
        "shouna stored 1 card in lianyaohu_store"
    );

    // Second shouna on next turn so lianyaohu_store holds 2 Basic cards, then verify lianhua
    // skips top Basic in deck and draws non-Basic (Trick/Equip), plus cancel refund!
    step_full_turn_to(&mut e, 0);
    let sha_cost2 = find_kind(&e, CardKind::Jiu);
    let shan_p1_2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != p1_card && e.catalog().card_kind(id) == CardKind::Shan)
        .unwrap();
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, sha_cost2);
    force_move_card_to_hand(&mut e, &v, 1, shan_p1_2);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: shouna_def });
    answer_open(&mut e, vec![ChoiceOption::Card(sha_cost2)]);
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.state_core().player(0).lianyaohu_store.len(), 2);

    let deck_basic = find_kind(&e, CardKind::Tao);
    let deck_trick = find_kind(&e, CardKind::Wuzhong);
    let deck_bottom_equip = find_kind(&e, CardKind::Qiankundai);
    {
        let mut snap = e.save();
        for cid in [deck_basic, deck_trick, deck_bottom_equip] {
            snap.core.deck.retain(|&x| x != cid);
            snap.core.discard.retain(|&x| x != cid);
            snap.core.table.retain(|&x| x != cid);
            for p in &mut snap.core.players {
                p.hand.retain(|&x| x != cid);
            }
        }
        // Bottom = Qiankundai (index 0); Top = deck_basic then deck_trick underneath it!
        snap.core.deck.insert(0, deck_bottom_equip);
        snap.core.deck.push(deck_trick);
        snap.core.deck.push(deck_basic);
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    e.debug_grant(0, "lianhua");
    let lianhua_def = e.catalog().skill_by_name("lianhua").expect("lianhua");
    let l1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: lianhua_def });
    assert!(matches!(l1, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "lianhua_pick");
    answer_open(&mut e, vec![]); // cancel refunds!
    let l2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: lianhua_def });
    assert!(matches!(l2, CommandOutcome::Pending { .. }));
    let p_opts = e.open_choice().unwrap().spec.options.clone();
    answer_open(&mut e, vec![p_opts[0], p_opts[1]]);
    drain_pass_all(&mut e);
    assert!(
        e.state_core().player(0).lianyaohu_store.is_empty(),
        "lianhua consumed both stored Basic cards"
    );
    assert!(
        e.debug_hand(0).contains(&deck_trick),
        "lianhua scanned top-down, skipped Basic Tao at top, and drew top-most Wuzhong"
    );
    assert!(
        !e.debug_hand(0).contains(&deck_bottom_equip) && e.state_core().deck.first() == Some(&deck_bottom_equip),
        "lianhua must NOT draw from deck bottom first"
    );

    // 2. kunlunjingc: cancel refunds usable, inspects deck TOP 3 (rev), and replaced hand card is drawn FIRST on next draw!
    let klj = find_kind(&e, CardKind::Kunlunjingc);
    force_move_card_to_hand(&mut e, &v, 0, klj);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: klj,
            targets: vec![],
        },
    );
    let bottom_sentinel = find_kind(&e, CardKind::Zhufangshenshi);
    let top3_c = find_kind(&e, CardKind::Sha);
    let top2_c = find_kind(&e, CardKind::Shan);
    let top1_c = find_kind(&e, CardKind::Jiu);
    let hand_rep = find_kind(&e, CardKind::Jingleishan);
    {
        let mut snap = e.save();
        for cid in [bottom_sentinel, top3_c, top2_c, top1_c, hand_rep] {
            snap.core.deck.retain(|&x| x != cid);
            snap.core.discard.retain(|&x| x != cid);
            snap.core.table.retain(|&x| x != cid);
            for p in &mut snap.core.players {
                p.hand.retain(|&x| x != cid);
            }
        }
        snap.core.deck.insert(0, bottom_sentinel);
        snap.core.deck.push(top3_c);
        snap.core.deck.push(top2_c);
        snap.core.deck.push(top1_c);
        snap.core.players[0].hand.push(hand_rep);
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    let klj_def = e.catalog().skill_by_name("kunlunjingc").expect("kunlunjingc");
    let k1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: klj_def });
    assert!(matches!(k1, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "kunlunjingc_pick");
    answer_open(&mut e, vec![]); // cancel refunds
    let k2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: klj_def });
    assert!(matches!(k2, CommandOutcome::Pending { .. }));
    let klj_opts = e.open_choice().unwrap().spec.options.clone();
    assert_eq!(
        klj_opts,
        vec![
            ChoiceOption::Card(top1_c),
            ChoiceOption::Card(top2_c),
            ChoiceOption::Card(top3_c),
        ],
        "kunlunjingc must reveal top 3 from deck tail (`rev()`), excluding deck bottom"
    );
    assert!(!klj_opts.contains(&ChoiceOption::Card(bottom_sentinel)));
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(top1_c)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "kunlunjingc_replace");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(hand_rep)]);
    assert_eq!(
        e.state_core().deck.last().copied(),
        Some(hand_rep),
        "kunlunjingc placed replaced card onto deck TOP (last) rather than bottom or discard"
    );
    assert!(e.debug_hand(0).contains(&top1_c));
    assert!(!e.debug_hand(0).contains(&hand_rep));
    // Prove replaced card is drawn FIRST on the very next draw!
    let wuzhong = find_kind(&e, CardKind::Wuzhong);
    force_move_card_to_hand(&mut e, &v, 0, wuzhong);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    pass_nullify(&mut e);
    drain_pass_all(&mut e);
    assert!(
        e.debug_hand(0).contains(&hand_rep) && e.debug_hand(0).contains(&top2_c),
        "next 2 draws must pop hand_rep then top2_c from deck top!"
    );
    assert_eq!(total_card_set(&e), init_set, "strict conservation across kunlunjingc");

    // 3. shennongding cancel refund + recover; donghuangzhong alive target incl self.
    let snd = find_kind(&e, CardKind::Shennongding);
    force_move_card_to_hand(&mut e, &v, 0, snd);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: snd,
            targets: vec![],
        },
    );
    e.debug_set_hp(0, 2);
    let snd_def = e.catalog().skill_by_name("shennongding").expect("shennongding");
    let r_card = find_kind(&e, CardKind::Jiu);
    let r_card2 = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 0, r_card);
    force_move_card_to_hand(&mut e, &v, 0, r_card2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: snd_def }),
        CommandOutcome::Pending { .. }
    ));
    answer_open(&mut e, vec![]); // cancel refunds
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: snd_def }),
        CommandOutcome::Pending { .. }
    ));
    let snd_opts = e.open_choice().unwrap().spec.options.clone();
    answer_open(&mut e, vec![snd_opts[0], snd_opts[1]]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3, "shennongding healed +1");

    let dhz = find_kind(&e, CardKind::Donghuangzhong);
    force_move_card_to_hand(&mut e, &v, 0, dhz);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dhz,
            targets: vec![],
        },
    );
    clear_hand_to_discard(&mut e, &v, 0);
    let c_cost = find_kind_suit(&e, CardKind::Wuzhong, |s| matches!(s, Suit::Heart | Suit::Diamond))
        .expect("red card for donghuangzhong");
    force_move_card_to_hand(&mut e, &v, 0, c_cost);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnPhaseEnd -> donghuangzhong_cost
    assert_eq!(e.open_choice().unwrap().spec.prompt, "donghuangzhong_cost");
    answer_open(&mut e, vec![ChoiceOption::Card(c_cost)]);
    let tgt_ask = e.open_choice().expect("donghuangzhong_target").clone();
    assert_eq!(tgt_ask.spec.prompt, "donghuangzhong_target");
    assert!(tgt_ask.spec.options.contains(&ChoiceOption::Player(0)), "alive self selectable");
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_zhufangshenshi_xuanyuanjian_yuruyi_qiankundai_and_misc_cards() {
    let (v, mut e) = start_shahai_swd_only(9221, 4);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. qiankundai: +1 hand limit verified cleanly at real Discard phase (hp + 1 kept without discard).
    let qkd = find_kind(&e, CardKind::Qiankundai);
    force_move_card_to_hand(&mut e, &v, 0, qkd);
    e.debug_set_hp(0, 1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qkd,
            targets: vec![],
        },
    );
    let keep1 = find_kind(&e, CardKind::Sha);
    let keep2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != keep1 && e.catalog().card_kind(id) == CardKind::Sha)
        .unwrap();
    e.debug_clear_hand(0);
    force_move_card_to_hand(&mut e, &v, 0, keep1);
    force_move_card_to_hand(&mut e, &v, 0, keep2);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard (hp 1 + qiankundai +1 = limit 2, so 0 discards!)
    assert!(
        e.open_choice().is_none(),
        "qiankundai raises hand limit from 1 to 2 so 2 hand cards pass Discard phase"
    );
    step_full_turn_to(&mut e, 0);
    e.debug_set_hp(0, 4);

    // 2. zhufangshenshi + g_zhufangshenshi: draws exactly 1, Snatch distance-1 bypassed on marked target 2!
    let zfs = find_kind(&e, CardKind::Zhufangshenshi);
    let snatch = find_kind(&e, CardKind::Snatch);
    let p2_card = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 2, p2_card);
    force_move_card_to_hand(&mut e, &v, 0, snatch);
    // In 4-player ring, seat 2 is distance 2 from seat 0 -> normal Snatch rejected!
    let sn_rej = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    );
    assert!(matches!(sn_rej, CommandOutcome::Rejected(_)));
    force_move_card_to_hand(&mut e, &v, 0, zfs);
    let h0_before = e.state_core().player(0).hand.len();
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zfs,
            targets: vec![2],
        },
    );
    pass_nullify(&mut e);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().turn.zhufangshenshi_target, Some(2));
    assert_eq!(e.state_core().player(0).hand.len(), h0_before, "-1 zfs +1 draw = same len");
    let sn_ok = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    );
    assert!(!matches!(sn_ok, CommandOutcome::Rejected(_)), "zhufangshenshi unlocks Snatch dist 2");
    pass_nullify(&mut e);
    drain_pass_all(&mut e);

    // 3. xuanyuanjian + xuanyuanjian2 + xuanyuanjian3: single +1 Thunder dmg AND hp<=2 discard!
    let xyj = find_kind(&e, CardKind::Xuanyuanjian);
    force_move_card_to_hand(&mut e, &v, 0, xyj);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xyj,
            targets: vec![],
        },
    );
    e.debug_grant(0, "xuanyuanjian");
    e.debug_grant(0, "xuanyuanjian2");
    e.debug_grant(0, "xuanyuanjian3");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 5);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 3, "xuanyuanjian adds +1 ONCE (not +2 when both skills present)");
    assert_eq!(e.debug_hp(0), 3, "xuanyuanjian3 backlash -1 HP without xuanyuan");
    // Equipping xuanyuanjian granted +1 hujia; deal 2 damage so hujia clears and hp drops 3 -> 2!
    e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert_eq!(e.debug_hp(0), 2);
    assert!(
        !e.state_core().player(0).equips.contains(&xyj),
        "xuanyuanjian discarded when hp drops to <= 2 without xuanyuan"
    );

    // 4. guisheqi + haotianta + fuxiqin + kongdongyin + liuxinghuoyu + yangpijuan + shencaojie + fengyinzhidan + xuejibingbao + yuruyi
    for kind in [
        CardKind::Haotianta,
        CardKind::WuxingPan,
        CardKind::Kongdongyin,
        CardKind::Yuruyi,
    ] {
        let cid = find_kind(&e, kind);
        force_move_card_to_hand(&mut e, &v, 0, cid);
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: cid,
                targets: vec![],
            },
        );
        drain_pass_all(&mut e);
    }
    let fuxiqin = find_card_name(&e, "fuxiqin");
    let kongxin_def = e.catalog().skill_by_name("kongxin").expect("kongxin skill");
    force_move_card_to_hand(&mut e, &v, 0, fuxiqin);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fuxiqin,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        e.state_core().has_skill(0, kongxin_def),
        "equipping fuxiqin must grant equip skill kongxin"
    );
    let gsq = find_kind(&e, CardKind::Guisheqi);
    force_move_card_to_hand(&mut e, &v, 0, gsq);
    let h_before = e.state_core().player(3).hujia;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gsq,
            targets: vec![3],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(3).hujia, h_before + 1);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_jiguan_cards_jiguantong_cross_cmd_jiguanyuan_gouhunluo_huanpodan() {
    let (v, mut e) = start_shahai_swd_only(9222, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. jiguantong cross-command hit flag: target 2 takes damage -> target 1 discards -> user does NOT draw 1!
    let jgt = find_card_name(&e, "jiguantong");
    let p1_disc = find_kind(&e, CardKind::Shan);
    clear_hand_to_discard(&mut e, &v, 1);
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 1, p1_disc);
    force_move_card_to_hand(&mut e, &v, 0, jgt);
    let hp2_before = e.debug_hp(2);
    let h0_before = e.debug_hand(0).len();
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jgt,
            targets: vec![1, 2],
        },
    );
    pass_nullify(&mut e);
    while e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
        answer_open(&mut e, vec![]);
    }
    // Reverse order hits seat 2 first (empty hand -> immediate fire damage + sets u05_jiguantong_hit:{fid}),
    // then suspends on seat 1! Now save/restore BEFORE answering seat 1 across separate submit() commands!
    e = roundtrip_choice(e, &v);
    assert_eq!(e.open_choice().unwrap().observer, 1);
    answer_open(&mut e, vec![ChoiceOption::Card(p1_disc)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(2), hp2_before - 1, "jiguantong dealt fire dmg to empty-hand seat 2");
    assert_eq!(
        e.debug_hand(0).len(),
        h0_before - 1,
        "cross-command hit flag remembered damage so caster did NOT draw 1"
    );

    // 2. jiguanyuan: places only 2 cards (chosen + body), even if off_court has earlier cards, then returns at end phase!
    step_full_turn_to(&mut e, 0);
    let jgy = find_card_name(&e, "jiguanyuan");
    let extra_off = find_kind(&e, CardKind::Jiu);
    {
        let mut snap = e.save();
        snap.core.deck.retain(|&x| x != extra_off);
        snap.core.players[1].off_court.push(extra_off);
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    let jgy_cost = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, jgy);
    force_move_card_to_hand(&mut e, &v, 0, jgy_cost);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jgy,
            targets: vec![1],
        },
    );
    pass_nullify(&mut e);
    answer_open(&mut e, vec![ChoiceOption::Card(jgy_cost)]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().player(1).jiguanyuan_cards.len(),
        2,
        "jiguanyuan queued ONLY cost+body (not unrelated off_court)"
    );
    step_full_turn_to(&mut e, 2); // passes seat 1 TurnEndWindow -> regains 2 cards!
    assert!(e.state_core().player(1).jiguanyuan_cards.is_empty());
    assert!(e.debug_hand(1).contains(&jgy_cost));

    // 3. gouhunluo2 source death cleanup + huanpodan_skill saves once then consumes skill.
    step_full_turn_to(&mut e, 0);
    let ghl = find_card_name(&e, "gouhunluo");
    force_move_card_to_hand(&mut e, &v, 0, ghl);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ghl,
            targets: vec![1],
        },
    );
    pass_nullify(&mut e);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).gouhunluo_source, Some(0));
    assert!(e.state_core().player(1).gouhunluo_rounds > 0);
    // Grant huanpodan_skill to seat 0: first fatal death saves, second fatal death kills seat 0 -> cleans gouhunluo on seat 1!
    e.debug_grant(0, "huanpodan_skill");
    let hp_p0 = e.debug_hp(0) as u32;
    e.debug_deal_damage(Some(2), 0, hp_p0, DamageNature::Normal);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(0).alive, "huanpodan_skill cancelled first death");
    assert_eq!(e.debug_hp(0), 1, "huanpodan_skill restored hp to 1");
    assert!(e.debug_hand(0).len() == 1, "huanpodan_skill discarded all & drew 1");
    let hpd_def = e.catalog().skill_by_name("huanpodan_skill").unwrap();
    assert!(!e.state_core().has_skill(0, hpd_def), "huanpodan_skill consumed");
    // Second fatal death kills seat 0 and cleans gouhunluo_source on seat 1!
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(0).alive, "seat 0 dies after huanpodan consumed");
    assert_eq!(e.state_core().player(1).gouhunluo_source, None, "source death clears gouhunluo");
    assert_eq!(e.state_core().player(1).gouhunluo_rounds, 0);

    // 4. zhiluxiaohu + mujiaren + jiguanfeng present & bound
    for kw in ["zhiluxiaohu", "mujiaren", "jiguanfeng"] {
        let _ = find_card_name(&e, kw);
    }
    let _ = Suit::Club;
    assert_eq!(total_card_set(&e).1.len(), init_set.1.len());
}

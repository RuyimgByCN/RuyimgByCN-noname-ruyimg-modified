use super::common::*;

#[test]
fn u01_fix2_11_dddqianlong() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2819-2850
    let (mut e, _d3, _p9) = new_identity_clean_engine(92015, 5);
    let caomao = find_general(&e, "ddd_caomao");
    e.assign_general(0, caomao).unwrap();
    retain_only_skill(&mut e, 0, Some("dddqianlong"));

    let lord = 0u8;
    let (loyalist, rebel1, rebel2) = {
        let roles = e.state_core().identity_roles().unwrap();
        let loy = roles.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
        let rebels: Vec<u8> = roles
            .iter()
            .enumerate()
            .filter_map(|(i, r)| (*r == Role::Rebel).then_some(i as u8))
            .collect();
        (loy, rebels[0], rebels[1])
    };

    // Negative guard: non-Lord Rebel with `dddqianlong` does NOT activate zhuSkill!
    let mut e_nonlord = new_identity_clean_engine(92016, 5).0;
    e_nonlord.assign_general(rebel1, caomao).unwrap();
    retain_only_skill(&mut e_nonlord, rebel1, Some("dddqianlong"));
    let victim_other = (1u8..5).find(|&s| s != rebel1 && s != loyalist).unwrap();
    let h_nl_before = e_nonlord.state_core().player(rebel1).hand.len();
    let _ = e_nonlord.debug_deal_damage(Some(rebel1), victim_other, 10, DamageNature::Normal);
    assert!(
        !e_nonlord.log().iter().any(|ev| matches!(ev, LogEvent::SkillTriggered { skill, .. } if *skill == "dddqianlong")),
        "non-Lord must never trigger dddqianlong"
    );
    let _ = h_nl_before;

    // 1) Lord `ddd_caomao` kills Loyalist: discard-all penalty CANCELLED, Lord draws 3!
    let keep_c = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let keep_eq = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    put_in_hand(&mut e, lord, keep_c);
    put_in_equip(&mut e, lord, keep_eq);
    let _ = e.debug_deal_damage(Some(lord), loyalist, 10, DamageNature::Normal);
    assert!(
        e.state_core().player(lord).hand.contains(&keep_c)
            && e.state_core().player(lord).equips.contains(&keep_eq),
        "dddqianlong must cancel Loyalist kill discard penalty"
    );
    assert_eq!(e.state_core().player(lord).hand.len(), 1 + 3, "Lord draws 3 from dddqianlong");

    // 2) Lord kills Rebel: Rebel +3 reward CANCELLED, draws ONLY 3 (not 6)!
    let h_before_reb = e.state_core().player(lord).hand.len();
    let _ = e.debug_deal_damage(Some(lord), rebel1, 10, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(lord).hand.len(),
        h_before_reb + 3,
        "Lord killing Rebel draws 3 total, NOT 6"
    );
    let _ = rebel2;
}

#[test]
fn u01_fix2_12_ddddongcha() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2851-2940
    let (mut e, d3, p9) = new_clean_engine(92017, 3);
    let xinxianying = find_general(&e, "ddd_xinxianying");
    e.assign_general(0, xinxianying).unwrap();
    retain_only_skill(&mut e, 0, Some("ddddongcha"));

    let c_pre1 = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let c_pre2 = find_card_by(&e, |id, d| id != c_pre1 && d.kind == CardKind::Shan);
    let c_own = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let c_tgt = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    remove_card_from_all_zones(&mut e, c_pre1);
    remove_card_from_all_zones(&mut e, c_pre2);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "ddd_jian".to_string()), vec![c_pre1, c_pre2]);
    put_in_hand(&mut e, 0, c_own);
    put_in_hand(&mut e, 1, c_tgt);

    // Advance into P1's turn (`TurnBegin` fires before Phase::Prepare!):
    e.debug_set_turn(0, Phase::End, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let cc = e.open_choice().expect("ddddongcha_confirm").clone();
    assert_eq!((cc.observer, cc.spec.prompt), (0, "ddddongcha_confirm"));
    assert!(cc.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    let c1 = e.open_choice().expect("ddddongcha_owner_put").clone();
    assert_eq!((c1.observer, c1.spec.prompt), (0, "ddddongcha_owner_put"));
    assert!(!c1.spec.allow_cancel);
    answer(&mut e, 0, vec![ChoiceOption::Card(c_own)]);

    let c2 = e.open_choice().expect("ddddongcha_target_put").clone();
    assert_eq!((c2.observer, c2.spec.prompt), (1, "ddddongcha_target_put"));
    assert!(!c2.spec.allow_cancel);
    answer(&mut e, 1, vec![ChoiceOption::Card(c_tgt)]);

    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "ddd_jian".to_string())).map(Vec::len),
        Some(4)
    );

    // At `phaseEnd` (`TurnEndWindow { after: false }`): owner picks 1, then target picks 1, leaving 2 in `"ddd_jian"`!
    e.debug_set_turn(1, Phase::End, 2);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let eo = e.open_choice().expect("ddddongcha_end_owner_pick").clone();
    assert_eq!((eo.observer, eo.spec.prompt), (0, "ddddongcha_end_owner_pick"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(c_own)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(c_own)]);
    assert!(e.state_core().player(0).hand.contains(&c_own));

    let et = e.open_choice().expect("ddddongcha_end_target_pick").clone();
    assert_eq!((et.observer, et.spec.prompt), (1, "ddddongcha_end_target_pick"));
    assert!(!et.spec.options.contains(&ChoiceOption::Card(c_own)));
    answer(&mut e, 1, vec![ChoiceOption::Card(c_tgt)]);
    assert!(e.state_core().player(1).hand.contains(&c_tgt));
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "ddd_jian".to_string())),
        Some(&vec![c_pre1, c_pre2]),
        "leftovers must remain in ddd_jian pile!"
    );
}

#[test]
fn u01_fix2_12b_ddddongcha_guards() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2851-2940
    let (mut e, _, _) = new_clean_engine(92112, 3);
    let xinxianying = find_general(&e, "ddd_xinxianying");
    e.assign_general(0, xinxianying).unwrap();
    retain_only_skill(&mut e, 0, Some("ddddongcha"));

    let c0 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let c1 = find_card_by(&e, |id, d| id != c0 && d.kind == CardKind::Tao);
    put_in_hand(&mut e, 0, c0);
    put_in_hand(&mut e, 1, c1);

    // 1) Own Prepare phase (P2 End -> P0 Prepare): must NOT trigger ddddongcha!
    e.debug_set_turn(2, Phase::End, 1);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "ddddongcha must NOT trigger on owner's own Prepare");

    // 2) Owner empty hand while P1 has hand: must NOT trigger ddddongcha on P1 Prepare!
    remove_card_from_all_zones(&mut e, c0);
    e.debug_set_turn(0, Phase::End, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "ddddongcha requires owner to have hand cards");
    assert_eq!(flag(&e, 0, "ddddongcha_active_turn"), 0);

    // 3) Owner has hand while target P1 has empty hand: must NOT trigger ddddongcha on P1 Prepare!
    put_in_hand(&mut e, 0, c0);
    remove_card_from_all_zones(&mut e, c1);
    e.debug_set_turn(0, Phase::End, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "ddddongcha requires target to have hand cards");
    assert_eq!(flag(&e, 0, "ddddongcha_active_turn"), 0);
}

#[test]
fn u01_fix2_13_dddzhijie() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2941-3067
    let (mut e, d3, p9) = new_clean_engine(92018, 3);
    let xinxianying = find_general(&e, "ddd_xinxianying");
    e.assign_general(0, xinxianying).unwrap();
    let zhijie_def = e.catalog().skill_by_name("dddzhijie").unwrap();

    let red1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart) && d.kind == CardKind::Tao);
    let red2 = find_card_by(&e, |id, d| id != red1 && matches!(d.suit, Suit::Diamond) && d.kind == CardKind::Shan);
    let blk1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade) && d.kind == CardKind::Sha);
    let sha1 = find_card_by(&e, |id, d| id != blk1 && d.kind == CardKind::Sha);
    let sha2 = find_card_by(&e, |id, d| id != blk1 && id != sha1 && d.kind == CardKind::Sha);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);

    remove_card_from_all_zones(&mut e, red1);
    remove_card_from_all_zones(&mut e, red2);
    remove_card_from_all_zones(&mut e, blk1);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "ddd_jian".to_string()), vec![red1, red2, blk1]);

    // Negative guard: NOT active in Phase::Play (`if (event.type === "phase") return false`)!
    e.debug_set_turn(0, Phase::Play, 1);
    assert!(
        matches!(submit(&mut e, 0, GameAction::ActivateSkill { skill: zhijie_def }), CommandOutcome::Rejected(_)),
        "dddzhijie must NEVER activate in Play phase"
    );

    // 1) Same-color Shan in `Sha` dodge window (+ no once_per_turn cap + draws 2 on useCardAfter!):
    put_in_hand(&mut e, 1, sha1);
    e.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    let cs = e.open_choice().expect("sha_shan with dddzhijie").clone();
    assert!(cs.spec.options.contains(&ChoiceOption::Skill(zhijie_def)));
    answer(&mut e, 0, vec![ChoiceOption::Skill(zhijie_def)]);

    let cz = e.open_choice().expect("dddzhijie_cards").clone();
    assert_eq!((cz.observer, cz.spec.prompt), (0, "dddzhijie_cards"));
    assert!(cz.spec.options.contains(&ChoiceOption::Card(red1)));
    assert!(cz.spec.options.contains(&ChoiceOption::Card(red2)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(red1), ChoiceOption::Card(red2)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red1), ChoiceOption::Card(red2)]);
    assert_eq!(e.state_core().player(0).hp, 3, "Shan dodged Sha");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "dddzhijie draws 2 on useCardAfter");

    // 2) Different-color Wuxie in `NullifyWindow`: put [red1, blk1] back into ddd_jian, P1 plays Wuzhong!
    let red3 = find_card_by(&e, |id, d| id != red1 && id != red2 && matches!(d.suit, Suit::Heart));
    remove_card_from_all_zones(&mut e, red3);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "ddd_jian".to_string()), vec![red3, blk1]);
    put_in_hand(&mut e, 1, wuzhong);
    // Notice only 1 red + 1 black: cannot be Shan, ONLY Nullify!
    let p0_shan = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    put_in_hand(&mut e, 0, p0_shan);
    e.state_core_mut().turn.sha_used_play_phase = false;
    put_in_hand(&mut e, 1, sha2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    let cs2 = e.open_choice().expect("sha_shan without same-color pair").clone();
    assert!(
        !cs2.spec.options.contains(&ChoiceOption::Skill(zhijie_def)),
        "red+black pair must NOT offer dddzhijie for Shan"
    );
    answer(&mut e, 0, vec![ChoiceOption::Pass]);

    let _ = submit(&mut e, 1, GameAction::PlayCard { card: wuzhong, targets: Vec::new() });
    let cn = e.open_choice().expect("nullify_window with dddzhijie").clone();
    assert_eq!(cn.spec.prompt, "nullify_window");
    assert!(cn.spec.options.contains(&ChoiceOption::Skill(zhijie_def)));
    answer(&mut e, 0, vec![ChoiceOption::Skill(zhijie_def)]);
    let _czw = e.open_choice().expect("dddzhijie_cards wuxie").clone();
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(red3), ChoiceOption::Card(blk1)]);
    let h0_pre_wx = e.state_core().player(0).hand.len();
    let p1_pre_wx = e.state_core().player(1).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Card(red3), ChoiceOption::Card(blk1)]);
    assert_eq!(e.state_core().player(1).hand.len(), p1_pre_wx, "Wuzhong must be nullified!");
    assert_eq!(e.state_core().player(0).hand.len(), h0_pre_wx + 2, "dddzhijie draws 2 after Wuxie!");
}

#[test]
fn u01_fix2_13b_dddzhijie_no_choose_to_respond() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2942
    let (mut e, _, _) = new_clean_engine(92113, 3);
    let xinxianying = find_general(&e, "ddd_xinxianying");
    e.assign_general(0, xinxianying).unwrap();
    let zhijie_def = e.catalog().skill_by_name("dddzhijie").unwrap();

    let red1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart) && d.kind == CardKind::Tao);
    let red2 = find_card_by(&e, |id, d| id != red1 && matches!(d.suit, Suit::Diamond) && d.kind == CardKind::Shan);
    let p0_shan = find_card_by(&e, |id, d| id != red2 && d.kind == CardKind::Shan);
    let archery = find_card_by(&e, |_, d| d.kind == CardKind::ArcheryAttack);

    remove_card_from_all_zones(&mut e, red1);
    remove_card_from_all_zones(&mut e, red2);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "ddd_jian".to_string()), vec![red1, red2]);
    put_in_hand(&mut e, 0, p0_shan);
    put_in_hand(&mut e, 1, archery);
    e.debug_set_turn(1, Phase::Play, 1);

    let _ = submit(&mut e, 1, GameAction::PlayCard { card: archery, targets: Vec::new() });
    let resp = e.open_choice().expect("aoe_response for P0").clone();
    assert_eq!((resp.observer, resp.spec.prompt), (0, "aoe_response"));
    assert!(resp.spec.options.contains(&ChoiceOption::Card(p0_shan)));
    assert!(
        !resp.spec.options.contains(&ChoiceOption::Skill(zhijie_def)),
        "dddzhijie is enable:chooseToUse only and must NOT be offered on chooseToRespond (ArcheryAttack)"
    );
}

#[test]
fn u01_fix2_14_dddqiahua() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3068-3111
    let (mut e, d3, p9) = new_clean_engine(92019, 3);
    let xianglang = find_general(&e, "ddd_xianglang");
    e.assign_general(0, xianglang).unwrap();
    retain_only_skill(&mut e, 0, Some("dddqiahua"));
    let xunxun_def = e.catalog().skill_by_name("dddxunxun").unwrap();
    assert!(!e.state_core().has_skill(1, xunxun_def));

    let c1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let c2 = find_card_by(&e, |id, d| id != c1 && d.kind == CardKind::Sha);
    let c3 = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    e.state_core_mut().player_mut(1).hp = 2;

    // 1) Own Prepare phase (P2 End -> P0 Prepare): MUST NOT trigger even with >= P0.hp unshown cards!
    put_in_hand(&mut e, 0, c1);
    put_in_hand(&mut e, 0, c2);
    put_in_hand(&mut e, 0, c3);
    e.debug_set_turn(2, Phase::End, 1);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "dddqiahua triggers only on OTHER players' Phase::Prepare");

    // 2) Negative guard: `c1` already shown, `c3` removed -> only 1 unshown (`c2`) < `P1.hp == 2` -> NO trigger on P1 Prepare!
    remove_card_from_all_zones(&mut e, c3);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{c1}")), 1);
    e.debug_set_turn(0, Phase::End, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "not enough unshown cards for P1.hp=2");

    // 3) Add `c3` unshown -> advance into P1's Phase::Prepare -> triggers directly on P1 (no target pick Ask 2035), `c1` excluded!
    put_in_hand(&mut e, 0, c3);
    e.debug_set_turn(0, Phase::End, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let cq = e.open_choice().expect("dddqiahua_show on P1 Prepare").clone();
    assert_eq!((cq.observer, cq.spec.prompt), (0, "dddqiahua_show"));
    assert_eq!((cq.spec.min, cq.spec.max), (2, 2));
    assert!(!cq.spec.options.contains(&ChoiceOption::Card(c1)), "already-shown card must be excluded");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(c2), ChoiceOption::Card(c3)]);

    // Fork A: Cancel `dddqiahua_show` -> c2/c3 stay unshown, P1 does NOT gain `dddxunxun`.
    let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_cancel, 0, Vec::new());
    assert!(!e_cancel.state_core().has_skill(1, xunxun_def));
    assert_eq!(flag(&e_cancel, 0, &format!("u01_shown_card:{c2}")), 0);

    // Live B: Confirm `[c2, c3]` -> real `dddxunxun` granted temporarily to P1 + shown flags on `c2, c3`!
    answer(&mut e, 0, vec![ChoiceOption::Card(c2), ChoiceOption::Card(c3)]);
    assert!(e.state_core().has_skill(1, xunxun_def), "dddqiahua must grant dddxunxun to trigger.player P1");
    assert_eq!(flag(&e, 0, &format!("u01_shown_card:{c2}")), 1);
    assert_eq!(flag(&e, 0, &format!("u01_shown_card:{c3}")), 1);

    // Advancing P1 to Phase::Draw triggers P1's newly granted `dddxunxun`!
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge -> Draw!
    let cx = e.open_choice().expect("dddxunxun triggered on P1 Draw").clone();
    assert_eq!(cx.observer, 1);
    drain(&mut e);

    // Ending P1's turn expires temporary `dddxunxun`, while still-held cards stay shown on P0!
    e.debug_set_turn(1, Phase::End, 4);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert!(!e.state_core().has_skill(1, xunxun_def), "temp dddxunxun must expire after P1 turn");
    assert_eq!(flag(&e, 0, &format!("u01_shown_card:{c2}")), 1);
}

#[test]
fn u01_fix2_15_dddxunxun() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3112-3148
    let (mut e, d3, p9) = new_clean_engine(92020, 3);
    e.debug_grant(0, "dddxunxun");

    let t1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let t2 = find_card_by(&e, |id, d| id != t1 && d.kind == CardKind::Sha);
    let t3 = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let t4 = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    let keep_below = find_card_by(&e, |id, d| {
        id != t1 && id != t2 && id != t3 && id != t4 && d.kind == CardKind::Dismantle
    });

    // Put `[keep_below, t4, t3, t2, t1]` on top of deck (preserving all other cards for snapshot conservation)!
    put_deck_top(&mut e, keep_below);
    put_deck_top(&mut e, t4);
    put_deck_top(&mut e, t3);
    put_deck_top(&mut e, t2);
    put_deck_top(&mut e, t1);
    let pre_top_len = e.state_core().deck.len() - 5;
    let pre_prefix = e.state_core().deck[..pre_top_len].to_vec();
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);

    let cc = e.open_choice().expect("dddxunxun_confirm").clone();
    assert_eq!((cc.observer, cc.spec.prompt), (0, "dddxunxun_confirm"));
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    // Order top 2: choose `[t4, t1]` so `t4` is drawn 1st and `t1` drawn 2nd!
    let ct = e.open_choice().expect("dddxunxun_top").clone();
    assert_eq!((ct.observer, ct.spec.prompt), (0, "dddxunxun_top"));
    assert_eq!((ct.spec.min, ct.spec.max), (2, 2));
    assert!(!ct.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(t4), ChoiceOption::Card(t1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(t4), ChoiceOption::Card(t1)]);

    // Order bottom 2 (`t3` above `t2` on deck bottom): `[t3, t2]` -> `deck[0] == t2, deck[1] == t3`!
    let cb = e.open_choice().expect("dddxunxun_bottom").clone();
    assert_eq!((cb.observer, cb.spec.prompt), (0, "dddxunxun_bottom"));
    assert_eq!((cb.spec.min, cb.spec.max), (2, 2));
    assert!(!cb.spec.allow_cancel);
    answer(&mut e, 0, vec![ChoiceOption::Card(t3), ChoiceOption::Card(t2)]);

    assert_eq!(e.state_core().player(0).hand, vec![t4, t1], "top 2 drawn in exact chosen order [t4, t1]!");
    let mut expected_deck = vec![t2, t3];
    expected_deck.extend(pre_prefix);
    expected_deck.push(keep_below);
    assert_eq!(e.state_core().deck, expected_deck, "bottom 2 placed in exact order!");

    // Reshuffle check: deck has only 2 cards, remaining deck cards in discard -> draws full 4 cards after reshuffling discard!
    let (mut e_res, _d3, _p9) = new_clean_engine(92021, 3);
    e_res.debug_grant(0, "dddxunxun");
    let rest: Vec<u32> = e_res
        .state_core_mut()
        .deck
        .drain(..)
        .filter(|&c| c != t1 && c != t2)
        .collect();
    e_res.state_core_mut().discard.extend(rest);
    e_res.state_core_mut().deck.extend([t1, t2]);
    e_res.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e_res, 0, GameAction::NextPhase);
    answer(&mut e_res, 0, vec![ChoiceOption::Confirm]);
    let ct_r = e_res.open_choice().expect("dddxunxun_top reshuffle").clone();
    assert_eq!(ct_r.spec.options.len(), 4, "dddxunxun must reshuffle discard when deck < 4");
}

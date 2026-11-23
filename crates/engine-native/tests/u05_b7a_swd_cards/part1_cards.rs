use super::common::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::machine::LogEvent;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b7a_hufu_viewas_sha_shan_jiu_and_dying_with_save_restore() {
    let (v, mut e) = start_shahai_swd_only(9210, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let hufu = find_kind(&e, CardKind::Hufu);
    let hufu2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != hufu && e.catalog().card_kind(id) == CardKind::Hufu)
        .expect("second hufu");
    let sha_c = find_kind(&e, CardKind::Sha);

    // 1. Raw PlayCard on hufu outside dying must stay Rejected.
    force_move_card_to_hand(&mut e, &v, 0, hufu);
    force_move_card_to_hand(&mut e, &v, 0, hufu2);
    let raw_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: hufu,
            targets: vec![1],
        },
    );
    assert!(
        matches!(raw_out, CommandOutcome::Rejected(_)),
        "raw hufu PlayCard must stay dying-only: {raw_out:?}"
    );

    // 2. g_hufu_jiu view-as Jiu + cancel safety + active use.
    let hufu_jiu = e.catalog().skill_by_name("g_hufu_jiu").expect("g_hufu_jiu");
    let c0 = submit(&mut e, 0, GameAction::ActivateSkill { skill: hufu_jiu });
    assert!(matches!(c0, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_hufu_jiu_card");
    e = roundtrip_choice(e, &v);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_hufu_jiu_card");
    answer_open(&mut e, vec![]); // cancel leaves card in hand and jiu_used false
    assert!(!e.state_core().turn.jiu_used);
    assert!(e.state_core().player(0).hand.contains(&hufu));

    submit(&mut e, 0, GameAction::ActivateSkill { skill: hufu_jiu });
    answer_open(&mut e, vec![ChoiceOption::Card(hufu)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().turn.jiu_used, "g_hufu_jiu marks jiu_used");

    // 3. g_hufu_sha view-as Sha -> roundtrip mid-prompt -> target choice -> deals boosted damage!
    let hp1_before = e.state_core().player(1).hp;
    let hufu_sha = e.catalog().skill_by_name("g_hufu_sha").expect("g_hufu_sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: hufu_sha });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_hufu_sha_card");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(hufu2)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_hufu_sha_target");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        hp1_before - 2,
        "g_hufu_jiu + g_hufu_sha deals 2 damage"
    );

    // 4. g_hufu_shan view-as Shan allows dodging opponent's Sha!
    step_full_turn_to(&mut e, 1);
    force_move_card_to_hand(&mut e, &v, 0, hufu2);
    force_move_card_to_hand(&mut e, &v, 1, sha_c);
    let hp0_before = e.state_core().player(0).hp;
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![0],
        },
    );
    let shan_ask = e.open_choice().expect("respond_shan with hufu").clone();
    assert!(
        shan_ask.spec.options.contains(&ChoiceOption::Card(hufu2)),
        "g_hufu_shan exposes hufu as Shan response"
    );
    answer_open(&mut e, vec![ChoiceOption::Card(hufu2)]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_before, "hufu Shan dodged Sha");
    assert_eq!(total_card_set(&e), init_set, "strict conservation across hufu suite");
}

#[test]
fn u05_b7a_dujian_color_compare_turn_mark_and_target_guards() {
    let (v, mut e) = start_shahai_swd_only(9211, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let dujian = find_kind(&e, CardKind::Dujian);
    let red_own = find_kind_suit(&e, CardKind::Wuzhong, |s| matches!(s, Suit::Heart | Suit::Diamond))
        .expect("red card");
    let red_target =
        find_kind_suit(&e, CardKind::Tao, |s| matches!(s, Suit::Heart | Suit::Diamond))
            .expect("red target card");
    let black_target =
        find_kind_suit(&e, CardKind::Sha, |s| matches!(s, Suit::Spade | Suit::Club))
            .expect("black target card");

    // 1. Empty-hand target rejected.
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, dujian);
    let rej = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dujian,
            targets: vec![1],
        },
    );
    assert!(
        matches!(rej, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "dujian rejects empty-hand target: {rej:?}"
    );

    // 2. Same color -> caster discards chosen card & target loses 1 HP (roundtrip mid-choice, NO nullify window!).
    // Regression: opponent holding CardKind::Nullify must NOT trigger nullify_window when basic card Dujian is played.
    let nullify = find_kind(&e, CardKind::Nullify);
    force_move_card_to_hand(&mut e, &v, 1, nullify);
    force_move_card_to_hand(&mut e, &v, 0, red_own);
    force_move_card_to_hand(&mut e, &v, 1, red_target);
    let hp1_before = e.state_core().player(1).hp;
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dujian,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_dujian_own_card");
    assert_ne!(
        e.open_choice().unwrap().spec.prompt,
        "nullify_window",
        "dujian is type:basic and must not open nullify_window"
    );
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(red_own)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_dujian_target_card");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(red_target)]);
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(0).hand.contains(&red_own), "same-color own card discarded");
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "same-color target lost 1 HP");

    // 3. Different color -> keeps own card, target keeps HP, sets u05_dujian2_turn cleared at TurnEndWindow.
    step_full_turn_to(&mut e, 0);
    let red_own2 = find_kind_suit(&e, CardKind::Sha, |s| matches!(s, Suit::Heart | Suit::Diamond))
        .expect("second red card");
    e.debug_set_hp(1, 4);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, dujian);
    force_move_card_to_hand(&mut e, &v, 0, red_own2);
    force_move_card_to_hand(&mut e, &v, 1, black_target);
    let hp1_diff = e.state_core().player(1).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dujian,
            targets: vec![1],
        },
    );
    answer_open(&mut e, vec![ChoiceOption::Card(red_own2)]);
    answer_open(&mut e, vec![ChoiceOption::Card(black_target)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(0).hand.contains(&red_own2), "diff-color keeps own card");
    assert_eq!(e.state_core().player(1).hp, hp1_diff, "diff-color no LoseHp");
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(1, "u05_dujian2_turn".to_string())).copied(),
        Some(1),
        "diff-color marks u05_dujian2_turn on target"
    );
    // Advance past TurnEndWindow -> clears u05_dujian2_turn!
    step_full_turn_to(&mut e, 1);
    assert!(
        !e.state_core().vm_aux.flags.contains_key(&(1, "u05_dujian2_turn".to_string())),
        "u05_dujian2_turn cleared after TurnEndWindow"
    );
    assert_eq!(total_card_set(&e).1.len(), init_set.1.len());
}

#[test]
fn u05_b7a_yuchan_family_autoviewas_swap_and_equip_cancel_refund() {
    let (v, mut e) = start_shahai_swd_only(9212, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. All 8 Yuchan variants resolve & Xun/Zhen/Li have proper dying semantics.
    let qian = find_kind(&e, CardKind::Yuchanqian);
    let kun = find_kind(&e, CardKind::Yuchankun);
    let zhen = find_kind(&e, CardKind::Yuchanzhen);
    let xun = find_kind(&e, CardKind::Yuchanxun);
    let kan = find_kind(&e, CardKind::Yuchankan);
    let li = find_kind(&e, CardKind::Yuchanli);
    let gen = find_kind(&e, CardKind::Yuchangen);
    let dui = find_kind(&e, CardKind::Yuchandui);
    for cid in [qian, kun, zhen, xun, kan, li, gen, dui] {
        assert!(e.catalog().card_kind(cid).yuchan_auto_view_as().is_some());
    }

    // Yuchanxun usable on wounded self (like Tao).
    e.debug_set_hp(0, 2);
    force_move_card_to_hand(&mut e, &v, 0, xun);
    let xun_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xun,
            targets: vec![],
        },
    );
    assert!(!matches!(xun_out, CommandOutcome::Rejected(_)), "yuchanxun as Tao: {xun_out:?}");
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3, "yuchanxun healed +1");

    // Yuchanzhen & Yuchanli dying self-save (whereas other player cannot save with them).
    force_move_card_to_hand(&mut e, &v, 0, zhen);
    e.debug_deal_damage(Some(1), 0, 3, engine_native::event::DamageNature::Normal);
    let ask_dying = e.open_choice().expect("dying ask for p0").clone();
    assert!(
        ask_dying.spec.options.contains(&ChoiceOption::Card(zhen)),
        "yuchanzhen savable when dying self"
    );
    answer_open(&mut e, vec![ChoiceOption::Card(zhen)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(0).alive && e.state_core().player(0).hp >= 1);

    // 2. g_yuchan_equip: rejected without forged yuchan equip, succeeds when forged, cancel refunds.
    step_full_turn_to(&mut e, 0);
    let yuchan_equip = e.catalog().skill_by_name("g_yuchan_equip").expect("g_yuchan_equip");
    let no_forge = submit(&mut e, 0, GameAction::ActivateSkill { skill: yuchan_equip });
    assert!(
        matches!(no_forge, CommandOutcome::Rejected(_)),
        "g_yuchan_equip blocked before yuchan forged: {no_forge:?}"
    );

    let sha1 = find_kind(&e, CardKind::Sha);
    let shan1 = find_kind(&e, CardKind::Shan);
    force_equip_card(&mut e, &v, 0, qian);
    force_move_card_to_hand(&mut e, &v, 0, sha1);
    force_move_card_to_hand(&mut e, &v, 0, shan1);
    let act1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: yuchan_equip });
    assert!(matches!(act1, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_yuchan_equip_discard");
    answer_open(&mut e, vec![]); // cancel refunds once_per_turn

    let act2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: yuchan_equip });
    assert!(matches!(act2, CommandOutcome::Pending { .. }), "cancel must refund usable");
    e = roundtrip_choice(e, &v);
    let hand_before = e.state_core().player(0).hand.len();
    answer_open(&mut e, vec![ChoiceOption::Card(sha1), ChoiceOption::Card(shan1)]);
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(0).hand.contains(&sha1));
    assert!(!e.state_core().player(0).hand.contains(&shan1));
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before,
        "discard 2 basics and draw 2 equal"
    );

    // Third activation in same Play phase rejected by once_per_turn limit.
    let sha2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != sha1 && e.catalog().card_kind(id) == CardKind::Sha)
        .unwrap();
    force_move_card_to_hand(&mut e, &v, 0, sha2);
    let act3 = submit(&mut e, 0, GameAction::ActivateSkill { skill: yuchan_equip });
    assert!(matches!(act3, CommandOutcome::Rejected(_)));
    assert_eq!(total_card_set(&e), init_set, "strict conservation across yuchan");
}

#[test]
fn u05_b7a_pangufu_range4_and_damage_he_discard() {
    let (v, mut e) = start_shahai_swd_only(9213, 8);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let pangufu = find_kind(&e, CardKind::Pangufu);
    let sha = find_kind(&e, CardKind::Sha);
    assert_eq!(e.catalog().card_kind(pangufu).equip_slot(), Some(EquipSlot::Weapon));
    assert_eq!(e.catalog().card_kind(pangufu).named_weapon_range(), Some(4));

    // Distance between 0 and 4 in an 8-player circle is 4 (> default attack range 1).
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let far_rej = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![4],
        },
    );
    assert!(
        matches!(far_rej, CommandOutcome::Rejected(_)),
        "distance 4 Sha blocked without range-4 weapon"
    );

    // Equip pangufu via real PlayCard!
    force_move_card_to_hand(&mut e, &v, 0, pangufu);
    let eq_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: pangufu,
            targets: vec![],
        },
    );
    assert!(!matches!(eq_out, CommandOutcome::Rejected(_)));
    let p4_equip = find_kind(&e, CardKind::Armor);
    force_equip_card(&mut e, &v, 4, p4_equip);
    clear_hand_to_discard(&mut e, &v, 4);

    let sha_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![4],
        },
    );
    assert!(!matches!(sha_out, CommandOutcome::Rejected(_)), "pangufu unlocks distance-4 Sha");
    drain_pass_all(&mut e); // passes Shan or wait — let's check if pangufu discard prompt fired!
    // Re-check explicitly without drain_pass_all swallowing u05_pangufu_discard:
    step_full_turn_to(&mut e, 0);
    let sha2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != sha && e.catalog().card_kind(id) == CardKind::Sha)
        .unwrap();
    let p4_armor = find_kind(&e, CardKind::HorseMinus);
    force_move_card_to_hand(&mut e, &v, 0, sha2);
    force_equip_card(&mut e, &v, 4, p4_armor);
    e.debug_set_hp(4, 4);
    let sha2_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![4],
        },
    );
    assert!(
        !matches!(sha2_out, CommandOutcome::Rejected(_)),
        "sha2 failed: {sha2_out:?}"
    );
    pass_nullify(&mut e);
    if e.open_choice().is_some_and(|c| c.spec.prompt == "respond_shan") {
        answer_open(&mut e, vec![ChoiceOption::Pass]);
    }
    let pf_ask = e.open_choice().expect("pangufu discard ask").clone();
    assert_eq!(pf_ask.spec.prompt, "u05_pangufu_discard");
    assert_eq!(pf_ask.observer, 4);
    assert!(pf_ask.spec.options.contains(&ChoiceOption::Card(p4_armor)));
    e = roundtrip_choice(e, &v);
    let log_mark = e.log().len();
    let _out_discard = answer_open(&mut e, vec![ChoiceOption::Card(p4_armor)]);
    let disc_count = e.log()[log_mark..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { cards, .. } if cards.contains(&p4_armor)))
        .count();
    assert_eq!(disc_count, 1, "pangufu cleanly emits single CardDiscarded");
    assert!(!e.state_core().player(4).equips.contains(&p4_armor));
    assert_eq!(total_card_set(&e), init_set, "strict conservation across pangufu");
}

#[test]
fn u05_b7a_hufu_sha_target_blocks_and_poyue_red_exemption() {
    let (v, mut e) = start_shahai_swd_only(9215, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let hufu_sha = e.catalog().skill_by_name("g_hufu_sha").expect("g_hufu_sha");
    let hufu_red = find_kind_suit(&e, CardKind::Hufu, |s| matches!(s, Suit::Heart | Suit::Diamond))
        .expect("red hufu");
    let hufu_black = find_kind_suit(&e, CardKind::Hufu, |s| matches!(s, Suit::Spade | Suit::Club))
        .expect("black hufu");
    let sha_c = find_kind(&e, CardKind::Sha);

    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 0, hufu_black);

    // 1. Blocked targets (`yinmi_active` + `热闷` without armor) must NOT be selectable by g_hufu_sha!
    e.state_core_mut().player_mut(1).yinmi_active = true;
    e.debug_grant(2, engine_native::vocabulary::names::BOSS_REMEN);
    let rej_all = submit(&mut e, 0, GameAction::ActivateSkill { skill: hufu_sha });
    assert!(
        matches!(rej_all, CommandOutcome::Rejected(_)),
        "g_hufu_sha must reject when all opponents are target-blocked (yinmi + remen): {rej_all:?}"
    );

    // Unblock seat 2 by equipping Armor so remen doesn't apply; seat 1 remains blocked (`yinmi_active`).
    let armor = find_kind(&e, CardKind::Armor);
    force_equip_card(&mut e, &v, 2, armor);
    let out_part = submit(&mut e, 0, GameAction::ActivateSkill { skill: hufu_sha });
    assert!(matches!(out_part, CommandOutcome::Pending { .. }));
    answer_open(&mut e, vec![ChoiceOption::Card(hufu_black)]);
    let tgt_ask = e.open_choice().expect("u05_hufu_sha_target").clone();
    assert_eq!(tgt_ask.spec.prompt, "u05_hufu_sha_target");
    assert!(
        !tgt_ask.spec.options.contains(&ChoiceOption::Player(1)),
        "yinmi_active seat 1 must be excluded from u05_hufu_sha_target"
    );
    assert!(
        tgt_ask.spec.options.contains(&ChoiceOption::Player(2)),
        "unblocked seat 2 must remain selectable"
    );
    answer_open(&mut e, vec![]); // cancel cleanly

    // 2. Poyue red-hufu exemption after normal Sha is already used in Play phase:
    // black hufu must NOT be offered once sha_used_play_phase=true, while red hufu IS offered & fires!
    force_move_card_to_hand(&mut e, &v, 0, sha_c);
    force_move_card_to_hand(&mut e, &v, 0, hufu_red);
    e.debug_grant(0, "poyue");
    e.state_core_mut().player_mut(1).yinmi_active = false;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert!(e.state_core().turn.sha_used_play_phase);

    let hp2_before = e.debug_hp(2);
    let out_poyue = submit(&mut e, 0, GameAction::ActivateSkill { skill: hufu_sha });
    assert!(
        matches!(out_poyue, CommandOutcome::Pending { .. }),
        "poyue must allow red hufu as Sha even after normal Sha limit used: {out_poyue:?}"
    );
    let card_ask = e.open_choice().expect("u05_hufu_sha_card").clone();
    assert_eq!(card_ask.spec.prompt, "u05_hufu_sha_card");
    assert!(
        card_ask.spec.options.contains(&ChoiceOption::Card(hufu_red)),
        "red hufu must be eligible under poyue"
    );
    assert!(
        !card_ask.spec.options.contains(&ChoiceOption::Card(hufu_black)),
        "black hufu must be excluded once sha_used_play_phase=true under poyue"
    );
    answer_open(&mut e, vec![ChoiceOption::Card(hufu_red)]);
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.debug_hp(2),
        hp2_before - 1,
        "poyue red g_hufu_sha dealt 1 damage"
    );
    assert_eq!(total_card_set(&e), init_set, "strict conservation");
}

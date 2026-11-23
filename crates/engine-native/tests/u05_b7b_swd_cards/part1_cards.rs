use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b7b_xiangyuye_out_of_range_basic_discard_or_lose_hp_and_save_restore() {
    let (v, mut e) = start_shahai_swd_only(9311, 4);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let xyy = find_kind(&e, CardKind::Xiangyuye);
    let nullify = find_kind(&e, CardKind::Nullify);
    let black_h = find_kind_suit(&e, CardKind::Sha, |s| matches!(s, Suit::Spade | Suit::Club))
        .expect("black card");
    let red_h = find_kind_suit(&e, CardKind::Shan, |s| matches!(s, Suit::Heart | Suit::Diamond))
        .expect("red card");

    // 1. In-range target (seat 1 at dist 1 <= range 1) rejected
    force_move_card_to_hand(&mut e, &v, 0, xyy);
    let rej_in_range = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xyy,
            targets: vec![1],
        },
    );
    assert!(matches!(rej_in_range, CommandOutcome::Rejected(RejectReason::IllegalTargets)));

    // 2. Out-of-range target (seat 2 at dist 2 > range 1) accepted WITHOUT nullify_window!
    // Target holds Nullify + black card -> NO nullify_window opens for basic card Xiangyuye!
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 2, nullify);
    force_move_card_to_hand(&mut e, &v, 2, black_h);
    let hp2_before = e.state_core().player(2).hp;
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xyy,
            targets: vec![2],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let ask = e.open_choice().expect("xiangyuye discard ask").clone();
    assert_eq!(ask.spec.prompt, "u05_xiangyuye_discard");
    assert_eq!(ask.observer, 2);
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(black_h)]);
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(2).hand.contains(&black_h), "black hand card discarded");
    assert_eq!(e.state_core().player(2).hp, hp2_before, "discarding black card avoids HP loss");

    // 3. Cancel prompt -> loses 1 HP; no black hand card -> direct lose 1 HP
    step_full_turn_to(&mut e, 0);
    let black_spare = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != black_h && e.catalog().card_kind(id) == CardKind::Sha && matches!(e.catalog().cards[id as usize].suit, Suit::Spade | Suit::Club))
        .expect("second black card");
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 0, xyy);
    force_move_card_to_hand(&mut e, &v, 2, black_spare);
    let hp_cancel_before = e.state_core().player(2).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xyy,
            targets: vec![2],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_xiangyuye_discard");
    answer_open(&mut e, Vec::new()); // cancel
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(2).hp, hp_cancel_before - 1);

    // 4. Target with only red hand card -> direct lose 1 HP without prompt
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 0, xyy);
    force_move_card_to_hand(&mut e, &v, 2, red_h);
    let hp_no_black = e.state_core().player(2).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xyy,
            targets: vec![2],
        },
    );
    assert!(e.open_choice().is_none(), "no black hand card skips prompt");
    assert_eq!(e.state_core().player(2).hp, hp_no_black - 1);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_huanpodan_basic_grants_guard_no_nullify_and_dying_triggers() {
    let (v, mut e) = start_shahai_swd_only(9312, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let hpd = find_kind(&e, CardKind::Huanpodan);
    let nullify = find_kind(&e, CardKind::Nullify);
    let spare = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 0, hpd);
    force_move_card_to_hand(&mut e, &v, 1, nullify);
    force_move_card_to_hand(&mut e, &v, 1, spare);

    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: hpd,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "basic card Huanpodan must not open nullify_window");
    let guard_def = e.catalog().skill_by_name("huanpodan_skill").expect("huanpodan_skill");
    assert!(e.state_core().has_skill(1, guard_def));

    // Re-targeting someone who already has huanpodan_skill is rejected
    force_move_card_to_hand(&mut e, &v, 0, hpd);
    let rej = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: hpd,
            targets: vec![1],
        },
    );
    assert!(matches!(rej, CommandOutcome::Rejected(RejectReason::IllegalTargets)));

    // Bring seat 1 down via lethal damage -> huanpodan_skill saves seat 1 to 1 HP, discards all, draws 1, removes skill!
    e.debug_set_hp(1, 1);
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(1).alive);
    assert_eq!(e.state_core().player(1).hp, 1);
    assert_eq!(e.state_core().player(1).hand.len(), 1, "huanpodan discards all then draws 1");
    assert!(!e.state_core().has_skill(1, guard_def), "huanpodan_skill consumed");
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_qinglianxindeng_blocks_trick_damage_respects_unequip_and_physical_sha() {
    let v = content_native_loader::prepare_locked_content_with_bridge(vec![
        freeze_formal_package("shahai-swd-upstream-noname"),
        freeze_formal_package("zuan-upstream-liuli"),
    ])
    .expect("prepare swd+zuan");
    let mut e = content_native_loader::start_locked_engine(
        9313,
        2,
        content_native_loader::LockedStartMode::Prototype,
        &v,
    )
    .expect("start swd+ow");
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let ql = find_kind(&e, CardKind::Qinglianxindeng);
    force_move_card_to_hand(&mut e, &v, 0, ql);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ql,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }));
    let qdef = e.catalog().skill_by_name("qinglianxindeng").expect("qinglianxindeng skill");
    assert!(e.state_core().has_skill(0, qdef));

    // Advance to seat 1 Play phase
    advance_to_play(&mut e, 1);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);

    // 1. Trick damage (`ArcheryAttack`) is prevented by Qinglianxindeng!
    let aa = find_kind(&e, CardKind::ArcheryAttack);
    force_move_card_to_hand(&mut e, &v, 1, aa);
    let hp0_before = e.state_core().player(0).hp;
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: aa,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_before, "Qinglianxindeng prevents Trick damage");

    // 2. Non-trick physical `Sha` STILL deals damage!
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 1, sha);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_before - 1, "Qinglianxindeng does not prevent Sha damage");

    // 3. `unequip` source (`ssr_pojia` ignore-armor) bypasses Qinglianxindeng trick shield!
    grant_skill(&mut e, &v, 1, "ssr_pojia");
    force_move_card_to_hand(&mut e, &v, 1, aa);
    let hp0_before_unequip = e.state_core().player(0).hp;
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: aa,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        hp0_before_unequip - 1,
        "unequip bypasses Qinglianxindeng"
    );
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_baihupifeng_guangshatianyi_guiyanfadao_tianxianjiu_qiankundai() {
    let (v, mut e) = start_shahai_swd_only(9314, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. `Baihupifeng`: equip -> minHp & damaged at PhaseEnd -> recover 1 (with save/restore)
    let bhp = find_kind(&e, CardKind::Baihupifeng);
    force_move_card_to_hand(&mut e, &v, 0, bhp);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: bhp, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 4);
    clear_hand_to_discard(&mut e, &v, 0);
    // Advance from Play -> Discard -> End; then finish End so `baihupifeng` prompt opens!
    advance_to_phase(&mut e, 0, engine_native::event::Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_baihupifeng_recover");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hp, 3, "Baihupifeng recovers 1 HP at PhaseEnd");

    // When NOT min HP (`p0=4 > p1=3`), does NOT trigger at PhaseEnd
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 2);
    let bhp_def = e.catalog().skill_by_name("baihupifeng").unwrap();
    // Max HP 4 -> damaged (3/4), NOT min HP (3 > 2)
    advance_to_phase(&mut e, 0, engine_native::event::Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(
        e.open_choice()
            .is_none_or(|c| c.spec.prompt != "u05_baihupifeng_recover"),
        "Baihupifeng skips when not min HP: {:?}",
        e.open_choice()
    );
    let _ = bhp_def;

    // 2. `Guiyanfadao`: equip range 2 + `Tianxianjiu`: Sha damage draw 2 + `Qiankundai`: onLose draw 1
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let gy = find_kind(&e, CardKind::Guiyanfadao);
    let txj = find_kind(&e, CardKind::Tianxianjiu);
    let qkd = find_kind(&e, CardKind::Qiankundai);
    let stm = find_kind(&e, CardKind::Shentoumianju);
    force_move_card_to_hand(&mut e, &v, 0, gy);
    force_move_card_to_hand(&mut e, &v, 0, qkd);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: gy, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: qkd, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));

    // Replace Qiankundai with Shentoumianju -> triggers Qiankundai `onLose` draw 1!
    let hand_before_swap = e.state_core().player(0).hand.len();
    force_move_card_to_hand(&mut e, &v, 0, stm);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: stm, targets: vec![] });
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_swap + 1,
        "losing Qiankundai via equip swap draws 1"
    );

    // Tianxianjiu + Guiyanfadao test:
    // Case A: Guiyanfadao declined -> Sha deals damage -> Tianxianjiu draws 2 and clears!
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_set_hp(1, 4);
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, txj);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: txj, targets: vec![] });
    let txj_def = e.catalog().skill_by_name("tianxianjiu").expect("tianxianjiu");
    assert!(e.state_core().has_skill(0, txj_def));
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_guiyanfadao_confirm");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, Vec::new()); // decline -> normal Sha damage (1; tianxianjiu draws 2)
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(e.state_core().player(0).hand.len(), 2, "Tianxianjiu draws 2 after Sha damage");
    assert!(!e.state_core().has_skill(0, txj_def), "tianxianjiu removed after draw");

    // Case B: Guiyanfadao confirmed -> bypasses hujia shield, loses 1 HP, leaves hujia intact!
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_set_hp(1, 3);
    {
        let mut snap = e.save();
        snap.core.players[1].hujia = 2;
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_guiyanfadao_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 2, "Guiyanfadao LoseHp leaves hujia untouched");
    assert_eq!(e.state_core().player(1).hp, 2, "Guiyanfadao causes direct 1 HP loss");

    // 3. `Guangshatianyi`: equip & verify both reduction (1/3 chance over hits) & conservation
    let gs = find_kind(&e, CardKind::Guangshatianyi);
    force_move_card_to_hand(&mut e, &v, 1, gs);
    advance_to_play(&mut e, 1);
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: gs, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));
    let mut reduced_seen = false;
    for _ in 0..16 {
        e.debug_set_hp(1, 4);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        drain_pass_all(&mut e);
        if e.state_core().player(1).hp == 4 {
            reduced_seen = true;
            break;
        }
    }
    assert!(reduced_seen, "Guangshatianyi 1/3 reduction triggered within 16 hits");
    assert_eq!(total_card_set(&e), init_set);
}

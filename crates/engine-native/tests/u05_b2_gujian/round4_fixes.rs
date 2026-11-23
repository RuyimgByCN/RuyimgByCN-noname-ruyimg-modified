use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::LogEvent;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn stash_deck_cards_where(
    e: &mut engine_native::Engine,
    off_seat: u8,
    mut pred: impl FnMut(u32, &engine_native::catalog::CardDef) -> bool,
) {
    let cards = e.catalog().cards.clone();
    let core = e.state_core_mut();
    let mut kept = Vec::new();
    let mut stashed = Vec::new();
    for c in core.deck.drain(..) {
        if pred(c, &cards[c as usize]) {
            stashed.push(c);
        } else {
            kept.push(c);
        }
    }
    for c in core.discard.drain(..) {
        stashed.push(c);
    }
    core.deck = kept;
    core.player_mut(off_seat).off_court.extend(stashed);
}

fn find_hand_card_resolved(
    e: &engine_native::Engine,
    seat: u8,
    want: CardKind,
) -> Option<u32> {
    let core = e.state_core();
    let cat = e.catalog();
    core.player(seat)
        .hand
        .iter()
        .copied()
        .find(|&c| cat.resolved_card_kind(core, c) == want)
}

#[test]
fn round4_defect1_yange_cloned_carriers_full_parity() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9401, 3);
    e.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e.assign_general_named(1, "貂蝉").unwrap();
    e.assign_general_named(2, "gjqt_bailitusu").unwrap();

    // Part A1: DyingSave accepts cloned `Dinvxuanshuang` & cloned `Tao` on mismatched black `Sha` carriers,
    // and `gj_dinvxuanshuang_rider` executes on `DyingSave`.
    advance_to_phase(&mut e, 2, Phase::End);
    clear_all_hands(&mut e);
    // Reserve ONLY black basic `Sha` in deck (NO red, NO Tao/Jiu/Hufu/Tianxianjiu/Dinvxuanshuang/SavageAttack).
    stash_deck_cards_where(&mut e, 2, |_, d| {
        !(d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club))
    });
    let mut used = Vec::new();
    let dx1 = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Dinvxuanshuang);
    used.push(dx1);
    let tao1 = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Tao);
    used.push(tao1);
    let savage1 = find_unused_card_by(&e, &used, |d| d.kind == CardKind::SavageAttack);
    used.push(savage1);
    move_card_to_hand(&mut e, 1, dx1);
    move_card_to_hand(&mut e, 1, tao1);
    move_card_to_hand(&mut e, 1, savage1);
    assert!(
        e.state_core().deck.len() >= 8,
        "need enough black Sha carriers in deck"
    );

    let _ = submit(&mut e, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e, "gj_yange_target");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
    advance_to_phase(&mut e, 0, Phase::Play);

    let cloned_dx = find_hand_card_resolved(&e, 0, CardKind::Dinvxuanshuang)
        .expect("yange cloned Dinvxuanshuang on Sha carrier");
    let cloned_tao =
        find_hand_card_resolved(&e, 0, CardKind::Tao).expect("yange cloned Tao on Sha carrier");
    let cloned_savage = find_hand_card_resolved(&e, 0, CardKind::SavageAttack)
        .expect("yange cloned SavageAttack on Sha carrier");
    assert_eq!(e.catalog().card_kind(cloned_dx), CardKind::Sha);
    assert_eq!(e.catalog().card_kind(cloned_tao), CardKind::Sha);
    assert_eq!(e.catalog().card_kind(cloned_savage), CardKind::Sha);
    assert_eq!(
        e.catalog()
            .effective_card_suit(e.state_core(), 0, cloned_tao),
        e.catalog().cards[tao1 as usize].suit,
        "effective_card_suit must return cloned template suit"
    );

    clear_all_hands(&mut e);
    move_card_to_hand(&mut e, 0, cloned_dx);
    move_card_to_hand(&mut e, 0, cloned_tao);
    move_card_to_hand(&mut e, 0, cloned_savage);
    let hp_2eq = find_unused_card_by(&e, &used, |d| d.kind == CardKind::HorsePlus);
    used.push(hp_2eq);
    move_card_to_hand(&mut e, 2, hp_2eq);
    e.state_core_mut().player_mut(2).hand.retain(|&x| x != hp_2eq);
    e.state_core_mut().player_mut(2).equips.push(hp_2eq);
    e.debug_set_hp(2, 1);

    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cloned_savage,
            targets: vec![],
        },
    );
    drain_until_prompt(&mut e, "dying_save");
    let dying_c = e.open_choice().cloned().expect("dying_save choice");
    assert_eq!(dying_c.spec.prompt, "dying_save");
    assert!(
        dying_c.spec.options.contains(&ChoiceOption::Card(cloned_tao)),
        "dying_save must accept cloned Tao on Sha carrier"
    );
    assert!(
        dying_c.spec.options.contains(&ChoiceOption::Card(cloned_dx)),
        "dying_save must accept cloned Dinvxuanshuang on Sha carrier"
    );
    let planner = BoundedPlanner {
        node_budget: 8,
        deadline_ms: Some(100),
        cancel: Arc::new(AtomicBool::new(false)),
        lookahead_depth: 1,
        policy_variant: 1,
        content_tuning: None,
        l01_options: None,
    };
    let ai_plan = e.plan_action(0, &planner);
    assert!(
        matches!(ai_plan.action, GameAction::Answer { .. }),
        "public-info AI must handle dying_save with cloned cards"
    );
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(cloned_dx)]);
    assert_eq!(
        e.debug_hp(2),
        1,
        "cloned Dinvxuanshuang must heal dying target back to 1 HP"
    );
    drain_until_prompt(&mut e, "gj_dinvxuanshuang_rider");
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("gj_dinvxuanshuang_rider"),
        "cloned Dinvxuanshuang in DyingSave must fire gj_dinvxuanshuang_rider"
    );
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(hp_2eq)]);
    drain(&mut e);
    assert!(e.state_core().table.is_empty(), "table clean after DyingSave");
    assert_zone_conservation_all(&e);

    // Part A2: Negative `Tao` carrier cloned as `Club Wuzhong` / `Spade Sha` MUST NOT save dying,
    // and playing `tao_carrier_as_wuzhong` at full HP resolves `Wuzhong` and exits `table` cleanly.
    let (_, mut e2) = gujian_verified_and_engine(2026_9402, 3);
    e2.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e2.assign_general_named(1, "貂蝉").unwrap();
    e2.assign_general_named(2, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e2, 2, Phase::End);
    clear_all_hands(&mut e2);
    // Keep ONLY Heart Tao cards in deck as physical carriers.
    stash_deck_cards_where(&mut e2, 2, |_, d| d.kind != CardKind::Tao);
    let wz1 = find_card_by(&e2, |_, d| d.kind == CardKind::Wuzhong);
    let sha_sp = find_card_by(&e2, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let tao_self = find_card_by(&e2, |_, d| d.kind == CardKind::Tao);
    e2.state_core_mut().deck.retain(|&c| c != tao_self);
    move_card_to_hand(&mut e2, 1, wz1);
    move_card_to_hand(&mut e2, 1, sha_sp);

    let _ = submit(&mut e2, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e2, "gj_yange_target");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Player(1)]);
    advance_to_phase(&mut e2, 0, Phase::Play);

    let cloned_wz = find_hand_card_resolved(&e2, 0, CardKind::Wuzhong)
        .expect("cloned Wuzhong on Tao carrier");
    let cloned_sha =
        find_hand_card_resolved(&e2, 0, CardKind::Sha).expect("cloned Sha on Tao carrier");
    assert_eq!(e2.catalog().card_kind(cloned_wz), CardKind::Tao);
    assert_eq!(e2.catalog().card_kind(cloned_sha), CardKind::Tao);

    clear_all_hands(&mut e2);
    move_card_to_hand(&mut e2, 0, cloned_wz);
    move_card_to_hand(&mut e2, 0, cloned_sha);
    move_card_to_hand(&mut e2, 2, tao_self);
    e2.debug_set_hp(2, 1);

    let out_sha = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: cloned_sha,
            targets: vec![2],
        },
    );
    assert!(
        !matches!(out_sha, CommandOutcome::Rejected(_)),
        "cloned Sha on Tao carrier must play as Sha: {out_sha:?}"
    );
    drain_until_prompt(&mut e2, "dying_save");
    let c_neg = e2.open_choice().cloned().expect("dying_save choice");
    assert_eq!(
        c_neg.observer, 2,
        "Tao carrier cloned as Wuzhong MUST NOT open dying_save for seat 0"
    );
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(tao_self)]);
    drain(&mut e2);
    assert!(e2.state_core().player(2).alive, "seat 2 saved self with real Tao");

    // Seat 0 is at max_hp; cloned_wz on Tao carrier must still resolve as Wuzhong (draw 2) and clear table!
    let pre_h0 = e2.debug_hand(0).len();
    let out_wz = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: cloned_wz,
            targets: vec![],
        },
    );
    assert!(!matches!(out_wz, CommandOutcome::Rejected(_)), "Wuzhong on Tao carrier: {out_wz:?}");
    drain(&mut e2);
    assert!(
        e2.state_core().table.is_empty(),
        "cloned Wuzhong must not stay stranded in table"
    );
    assert_eq!(
        e2.state_core()
            .vm_aux
            .flags
            .get(&(0, format!("gj_yange_clone:{cloned_wz}")))
            .copied()
            .unwrap_or(0),
        0,
        "gj_yange_clone must clear after card settles into discard via on_cards_to_discard"
    );
    assert_eq!(e2.debug_hand(0).len(), pre_h0 - 1 + 2, "Wuzhong drew 2");
    assert_zone_conservation_all(&e2);

    // Part B1: Cloned Nullify, Dizaizhen, Jiedao, Shan on black Sha carriers + table cleanup
    let (_, mut e3) = gujian_verified_and_engine(2026_9403, 3);
    e3.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e3.assign_general_named(1, "貂蝉").unwrap();
    e3.assign_general_named(2, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e3, 2, Phase::End);
    clear_all_hands(&mut e3);
    stash_deck_cards_where(&mut e3, 2, |_, d| {
        !(d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club))
    });
    let mut u3 = Vec::new();
    let nul1 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Nullify);
    u3.push(nul1);
    let wz_t1 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Wuzhong);
    u3.push(wz_t1);
    let jd1 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Jiedao);
    u3.push(jd1);
    let shan1 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Shan);
    u3.push(shan1);
    let wpn1 = find_unused_card_by(&e3, &u3, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    u3.push(wpn1);
    move_card_to_hand(&mut e3, 1, nul1);
    move_card_to_hand(&mut e3, 1, wz_t1);
    move_card_to_hand(&mut e3, 1, jd1);
    move_card_to_hand(&mut e3, 1, shan1);

    let _ = submit(&mut e3, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e3, "gj_yange_target");
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Player(1)]);
    advance_to_phase(&mut e3, 0, Phase::Play);

    let c_nul = find_hand_card_resolved(&e3, 0, CardKind::Nullify).expect("cloned Nullify");
    let c_wz3 = find_hand_card_resolved(&e3, 0, CardKind::Wuzhong).expect("cloned Wuzhong");
    let c_jd = find_hand_card_resolved(&e3, 0, CardKind::Jiedao).expect("cloned Jiedao");
    let c_shan = find_hand_card_resolved(&e3, 0, CardKind::Shan).expect("cloned Shan");
    for cc in [c_nul, c_wz3, c_jd, c_shan] {
        assert_eq!(e3.catalog().card_kind(cc), CardKind::Sha);
    }

    // 1) Play cloned Wuzhong and nullify with cloned Nullify -> table empty!
    clear_all_hands(&mut e3);
    move_card_to_hand(&mut e3, 0, c_wz3);
    move_card_to_hand(&mut e3, 0, c_nul);
    move_card_to_hand(&mut e3, 0, c_jd);
    move_card_to_hand(&mut e3, 0, c_shan);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: c_wz3, targets: vec![] });
    let c_nw = e3.open_choice().cloned().expect("nullify_window");
    assert_eq!(c_nw.spec.prompt, "nullify_window");
    assert!(
        c_nw.spec.options.contains(&ChoiceOption::Card(c_nul)),
        "nullify_window must offer cloned Nullify on Sha carrier"
    );
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(c_nul)]);
    drain(&mut e3);
    assert!(
        e3.log().iter().any(|l| matches!(l, LogEvent::CardNullified { card, .. } if *card == c_wz3)),
        "cloned Nullify must nullify cloned Wuzhong"
    );
    assert!(e3.state_core().table.is_empty(), "nullified cloned Wuzhong removed from table");

    // 2) Seat 1 equips Weapon -> Seat 0 plays cloned Jiedao targeting (1, 2), Seat 1 chooses Tag(1) give weapon -> cloned Jiedao removed from table!
    move_card_to_hand(&mut e3, 1, wpn1);
    e3.state_core_mut().player_mut(1).hand.retain(|&x| x != wpn1);
    e3.state_core_mut().player_mut(1).equips.push(wpn1);
    let out_jd = submit(
        &mut e3,
        0,
        GameAction::PlayCard {
            card: c_jd,
            targets: vec![1, 2],
        },
    );
    assert!(!matches!(out_jd, CommandOutcome::Rejected(_)), "Jiedao play: {out_jd:?}");
    pass_nullify_window(&mut e3);
    assert_eq!(e3.open_choice().unwrap().spec.prompt, "jiedao_sha_or_give");
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Tag(1)]);
    drain(&mut e3);
    assert!(e3.state_core().player(2).equips.contains(&wpn1), "Jiedao surrendered weapon to victim 2");
    assert!(
        e3.state_core().table.is_empty(),
        "cloned Jiedao must not remain stranded in table"
    );
    // Also check cloned `Shan` on `Sha` carrier resolved kind:
    assert_eq!(e3.catalog().resolved_card_kind(e3.state_core(), c_shan), CardKind::Shan);
    assert_zone_conservation_all(&e3);

    // Part B2: Duel + Qinglong replay using cloned `Sha` on `Shan` carrier + `save_restore`
    let (_, mut e4) = gujian_verified_and_engine(2026_9404, 3);
    e4.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e4.assign_general_named(1, "貂蝉").unwrap();
    e4.assign_general_named(2, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e4, 2, Phase::End);
    clear_all_hands(&mut e4);
    // Reserve ONLY `Shan` in deck so cloned cards borrow `Shan` carriers!
    stash_deck_cards_where(&mut e4, 2, |_, d| d.kind != CardKind::Shan);
    let mut u4 = Vec::new();
    let duel_4 = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Duel);
    u4.push(duel_4);
    let sha_4a = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Sha);
    u4.push(sha_4a);
    let sha_4b = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Sha);
    u4.push(sha_4b);
    let ql_4 = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Qinglong);
    u4.push(ql_4);
    // Give seat 1 Duel, 2x Sha, and Qinglong (or advance to 1 Play first to equip Qinglong cleanly!)
    move_card_to_hand(&mut e4, 1, ql_4);
    advance_to_phase(&mut e4, 1, Phase::Play);
    let _ = submit(&mut e4, 1, GameAction::PlayCard { card: ql_4, targets: vec![] });
    advance_to_phase(&mut e4, 2, Phase::End);
    clear_all_hands(&mut e4);
    stash_deck_cards_where(&mut e4, 2, |_, d| d.kind != CardKind::Shan);
    move_card_to_hand(&mut e4, 1, duel_4);
    move_card_to_hand(&mut e4, 1, sha_4a);
    move_card_to_hand(&mut e4, 1, sha_4b);

    let _ = submit(&mut e4, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e4, "gj_yange_target");
    let _ = answer_choice(&mut e4, vec![ChoiceOption::Player(1)]);
    advance_to_phase(&mut e4, 0, Phase::Play);

    let c_duel = find_hand_card_resolved(&e4, 0, CardKind::Duel).expect("cloned Duel");
    let mut c_shas: Vec<u32> = e4
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .filter(|&c| e4.catalog().resolved_card_kind(e4.state_core(), c) == CardKind::Sha)
        .collect();
    assert!(c_shas.len() >= 2, "must have 2 cloned Sha on Shan carriers");
    let c_sha_a = c_shas.remove(0);
    let c_sha_b = c_shas.remove(0);
    assert_eq!(e4.catalog().card_kind(c_sha_a), CardKind::Shan);
    assert_eq!(e4.catalog().card_kind(c_sha_b), CardKind::Shan);

    clear_all_hands(&mut e4);
    move_card_to_hand(&mut e4, 0, c_duel);
    move_card_to_hand(&mut e4, 0, c_sha_a);
    move_card_to_hand(&mut e4, 0, c_sha_b);
    let r_sha = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Sha);
    u4.push(r_sha);
    let r_shan = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Shan);
    u4.push(r_shan);
    move_card_to_hand(&mut e4, 1, r_sha);
    let hp1_pre = e4.debug_hp(1);
    let _ = submit(
        &mut e4,
        0,
        GameAction::PlayCard {
            card: c_duel,
            targets: vec![1],
        },
    );
    pass_nullify_window(&mut e4);
    let _ = answer_choice(&mut e4, vec![ChoiceOption::Card(r_sha)]);
    let duel_ask = e4.open_choice().cloned().expect("duel_sha for seat 0");
    assert_eq!(duel_ask.spec.prompt, "duel_sha");
    assert!(
        duel_ask.spec.options.contains(&ChoiceOption::Card(c_sha_a)),
        "duel_sha must offer cloned Sha on Shan carrier"
    );
    let _ = answer_choice(&mut e4, vec![ChoiceOption::Card(c_sha_a)]);
    drain(&mut e4);
    assert_eq!(e4.debug_hp(1), hp1_pre - 1, "Duel dealt 1 dmg to seat 1");

    // Now test Qinglong replay (`qinglong_replay_sha`) with cloned `c_sha_b` on Shan carrier!
    let r_sha_attack = find_unused_card_by(&e4, &u4, |d| d.kind == CardKind::Sha);
    u4.push(r_sha_attack);
    move_card_to_hand(&mut e4, 0, r_sha_attack);
    move_card_to_hand(&mut e4, 1, r_shan);
    let out_ql = submit(
        &mut e4,
        0,
        GameAction::PlayCard {
            card: r_sha_attack,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(out_ql, CommandOutcome::Rejected(_)),
        "Qinglong replay setup Sha must not reject: {out_ql:?}"
    );
    drain_until_prompt(&mut e4, "respond_shan");
    assert_eq!(
        e4.open_choice().map(|c| c.spec.prompt),
        Some("respond_shan"),
        "expected respond_shan before Qinglong replay, log={:?}",
        e4.log()
    );
    let _ = answer_choice(&mut e4, vec![ChoiceOption::Card(r_shan)]);
    drain_until_prompt(&mut e4, "qinglong_replay_sha");
    let ql_c = e4.open_choice().cloned().expect("qinglong_replay_sha");
    assert_eq!(ql_c.spec.prompt, "qinglong_replay_sha");
    assert!(
        ql_c.spec.options.contains(&ChoiceOption::Card(c_sha_b)),
        "qinglong_replay_sha must offer cloned Sha on Shan carrier"
    );
    let r4 = save_restore_verified(&e4, &verified);
    assert_zone_conservation_all(&r4);

    // Part B3: `effective_card_suit` & `play_rules` (`wusheng`): Heart card cloned onto black carrier CAN play as Sha,
    // while Spade Sha cloned onto Heart Tao carrier CANNOT play as Wusheng red Sha (actually Spade Sha plays as Sha, so test Spade Shan on Heart Tao!).
    let (_, mut e5) = gujian_verified_and_engine(2026_9405, 3);
    e5.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e5.assign_general_named(1, "关羽").unwrap();
    e5.assign_general_named(2, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e5, 2, Phase::End);
    clear_all_hands(&mut e5);
    stash_deck_cards_where(&mut e5, 2, |_, d| {
        !(matches!(d.suit, Suit::Spade | Suit::Club)
            && matches!(d.kind, CardKind::Sha | CardKind::Dismantle | CardKind::Snatch))
    });
    let mut u5 = Vec::new();
    let heart_tao5 = find_unused_card_by(&e5, &u5, |d| d.kind == CardKind::Tao && d.suit == Suit::Heart);
    u5.push(heart_tao5);
    move_card_to_hand(&mut e5, 1, heart_tao5);
    let _ = submit(&mut e5, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e5, "gj_yange_target");
    let _ = answer_choice(&mut e5, vec![ChoiceOption::Player(1)]);
    advance_to_phase(&mut e5, 0, Phase::Play);
    let c_heart_on_black = find_hand_card_resolved(&e5, 0, CardKind::Tao).expect("cloned Heart Tao on black carrier");
    assert_ne!(e5.catalog().cards[c_heart_on_black as usize].suit, Suit::Heart);
    assert_eq!(
        e5.catalog().effective_card_suit(e5.state_core(), 0, c_heart_on_black),
        Suit::Heart
    );
    let hp2_pre = e5.debug_hp(2);
    let out_ws = submit(
        &mut e5,
        0,
        GameAction::PlayCard {
            card: c_heart_on_black,
            targets: vec![2],
        },
    );
    assert!(
        !matches!(out_ws, CommandOutcome::Rejected(_)),
        "Wusheng must allow Heart cloned card on black carrier as Sha: {out_ws:?}"
    );
    drain(&mut e5);
    assert_eq!(e5.debug_hp(2), hp2_pre - 1);
    assert_zone_conservation_all(&e5);
}

#[test]
fn round4_defect2_yanjia_merged_equip_cleanup_on_discard_and_reshuffle() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9410, 3);
    e.assign_general_named(0, "gjqt_yuewuyi").unwrap();
    e.assign_general_named(1, "貂蝉").unwrap();
    e.assign_general_named(2, "gjqt_bailitusu").unwrap();
    let sk_yj = e.catalog().skill_by_name("yanjia").unwrap();

    // Path A: Merge Renwang (c1) + Zhuge (c2), equip c1, replace c1 with Bagua (`commit_equip`).
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let rw = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Renwang);
    used.push(rw);
    let zhuge = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Zhuge);
    used.push(zhuge);
    let bagua = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Armor && d.name == "armor");
    used.push(bagua);
    move_card_to_hand(&mut e, 0, rw);
    move_card_to_hand(&mut e, 0, zhuge);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yj });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(rw)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(zhuge)]);
    assert!(e.state_core().detached_cards.iter().any(|&(c, _)| c == zhuge));
    assert_eq!(flag(&e, 0, &format!("gj_yanjia_merged:{rw}")), zhuge + 1);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: rw, targets: vec![] });
    let sha1 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Heart, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    drain(&mut e);

    // Replace rw with Bagua -> rw enters discard, gj_yanjia_merged:{rw} MUST clear & zhuge MUST release to discard!
    move_card_to_hand(&mut e, 0, bagua);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bagua, targets: vec![] });
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, &format!("gj_yanjia_merged:{rw}")),
        0,
        "slot replacement into discard must clear gj_yanjia_merged:{rw}"
    );
    assert!(
        !e.state_core().detached_cards.iter().any(|&(c, _)| c == zhuge),
        "zhuge must be released from detached_cards on c1 discard"
    );
    assert!(
        e.state_core().discard.contains(&rw) && e.state_core().discard.contains(&zhuge),
        "both c1 and c2 must be in discard after slot replacement"
    );
    assert!(
        !e.catalog().equip_card_has_kind(e.state_core(), rw, CardKind::Zhuge),
        "c1 equip definition must no longer merge Zhuge"
    );
    let sha2 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Diamond, &mut used);
    let out_sha2 = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(
        matches!(out_sha2, CommandOutcome::Rejected(_)),
        "after c1 replaced, Zhuge unlimited Sha must be gone: {out_sha2:?}"
    );
    assert_zone_conservation_all(&e);

    // Path B: Dismantle c1 + Deck Reshuffle (reuse unmerged rw cleanly!)
    clear_all_hands(&mut e);
    let fangtian = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Fangtian);
    used.push(fangtian);
    let rw2 = rw;
    move_card_to_hand(&mut e, 0, fangtian);
    move_card_to_hand(&mut e, 0, rw2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yj });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(fangtian)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(rw2)]);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fangtian, targets: vec![] });

    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    // Also remove Bagua from seat 0 equips so fangtian is first equip dismantled:
    if e.state_core().player(0).equips.contains(&bagua) {
        let dism_b = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Dismantle);
        used.push(dism_b);
        move_card_to_hand(&mut e, 1, dism_b);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism_b, targets: vec![0] });
        pass_nullify_window(&mut e);
    }
    let dism_f = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Dismantle);
    used.push(dism_f);
    move_card_to_hand(&mut e, 1, dism_f);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism_f, targets: vec![0] });
    pass_nullify_window(&mut e);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, &format!("gj_yanjia_merged:{fangtian}")),
        0,
        "Dismantle must clear gj_yanjia_merged:{fangtian}"
    );
    assert!(
        !e.state_core().detached_cards.iter().any(|&(c, _)| c == rw2),
        "Dismantle must release rw2 from detached_cards"
    );
    assert!(e.state_core().discard.contains(&fangtian) && e.state_core().discard.contains(&rw2));

    // Path C: Hand discard in Phase::Discard + Deck Reshuffle (`LogEvent::DeckReshuffled`)
    let qg_c = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Qinggang);
    used.push(qg_c);
    let jb_c = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Zhangba);
    used.push(jb_c);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    move_card_to_hand(&mut e, 0, qg_c);
    move_card_to_hand(&mut e, 0, jb_c);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yj });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(qg_c)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(jb_c)]);
    e.debug_set_hp(0, 1);
    let spare_x = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Club, &mut used);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard (need to discard down to 1)
    let disc_c = e.open_choice().cloned().expect("discard_to_limit");
    assert_eq!(disc_c.spec.prompt, "discard_to_limit");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(qg_c)]);
    let _ = spare_x;
    drain(&mut e);
    assert_eq!(flag(&e, 0, &format!("gj_yanjia_merged:{qg_c}")), 0);
    assert!(!e.state_core().detached_cards.iter().any(|&(c, _)| c == jb_c));
    assert!(e.state_core().discard.contains(&qg_c) && e.state_core().discard.contains(&jb_c));

    // Also force a DeckReshuffled via draw (Wuzhong) when deck is drained:
    advance_to_phase(&mut e, 0, Phase::Play);
    let deck_rem = std::mem::take(&mut e.state_core_mut().deck);
    e.state_core_mut().player_mut(0).off_court.extend(deck_rem);
    let wz_r = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Wuzhong);
    used.push(wz_r);
    move_card_to_hand(&mut e, 0, wz_r);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wz_r, targets: vec![] });
    drain(&mut e);
    assert!(
        e.log().iter().any(|l| matches!(l, LogEvent::DeckReshuffled)),
        "expected DeckReshuffled"
    );
    assert_zone_conservation_all(&e);
    let _ = save_restore_verified(&e, &verified);
}

#[test]
fn round4_defect3_xuanzhen_excludes_dinvxuanshuang() {
    let (verified, _base) = gujian_verified_and_engine(2026_9420, 3);
    for seed_off in 0u64..80 {
        let mut e = content_native_loader::start_locked_engine(
            2026_9420 + seed_off,
            3,
            content_native_loader::LockedStartMode::Prototype,
            &verified,
        )
        .expect("start engine");
        e.assign_general_named(1, "gjqt_cenying").unwrap();
        advance_to_phase(&mut e, 0, Phase::Play);
        clear_all_hands(&mut e);
        let mut used = Vec::new();
        let sha = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
        let _spare = move_kind_suit_to_hand(&mut e, 1, CardKind::Sha, Suit::Club, &mut used);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_xuanzhen_confirm");
        let _ = answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        let vc = e.open_choice().cloned().expect("gj_xuanzhen_vcard");
        for opt in vc.spec.options {
            let mut branch = e.clone();
            let _ = answer_choice(&mut branch, vec![opt]);
            drain_until_prompt(&mut branch, "gj_dinvxuanshuang_rider");
            assert_ne!(
                branch.open_choice().map(|c| c.spec.prompt),
                Some("gj_dinvxuanshuang_rider"),
                "xuanzhen seed {} option {:?} must NOT select Dinvxuanshuang",
                2026_9420 + seed_off,
                opt
            );
        }
    }
}

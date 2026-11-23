use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::machine::LogEvent;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b7a_r3_qiankundai_on_lose_draws_card() {
    let (v, mut e) = start_shahai_swd_only(9240, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. Equipping another equip5 (yuruyi) replaces qiankundai -> onLose draws 1 card.
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let qkd = find_kind(&e, CardKind::Qiankundai);
    let yry = find_kind(&e, CardKind::Yuruyi);
    force_move_card_to_hand(&mut e, &v, 0, qkd);
    force_move_card_to_hand(&mut e, &v, 0, yry);
    let eq_qkd = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qkd,
            targets: vec![],
        },
    );
    assert!(!matches!(eq_qkd, CommandOutcome::Rejected(_)));
    drain_pass_all(&mut e);
    assert!(e.state_core().player(0).equips.contains(&qkd));
    assert_eq!(e.debug_hand(0).len(), 1);

    let eq_yry = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yry,
            targets: vec![],
        },
    );
    assert!(!matches!(eq_yry, CommandOutcome::Rejected(_)));
    drain_pass_all(&mut e);
    assert!(e.state_core().player(0).equips.contains(&yry));
    assert!(!e.state_core().player(0).equips.contains(&qkd));
    assert_eq!(
        e.debug_hand(0).len(),
        1,
        "qiankundai.onLose must draw 1 card when replaced in equips"
    );

    // 2. Dismantling qiankundai from player 0 also triggers onLose draw 1.
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, qkd);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qkd,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hand(0).len(), 0);

    advance_to_play(&mut e, 1);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let dismantle = find_kind(&e, CardKind::Dismantle);
    force_move_card_to_hand(&mut e, &v, 1, dismantle);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: dismantle,
            targets: vec![0],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(0).equips.contains(&qkd));
    assert_eq!(
        e.debug_hand(0).len(),
        1,
        "qiankundai.onLose must draw 1 card when dismantled from equips"
    );
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_r3_shenmiguo_yuchankan_useas_and_target_can_use() {
    let (v, mut e) = start_shahai_swd_only(9241, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. Dismantle empties p1's only card -> post-use canUse is false -> g_shenmiguo MUST NOT prompt.
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let p1_only = find_kind(&e, CardKind::Shan);
    let smg = find_kind(&e, CardKind::Shenmiguo);
    let dism = find_kind(&e, CardKind::Dismantle);
    force_move_card_to_hand(&mut e, &v, 1, p1_only);
    force_move_card_to_hand(&mut e, &v, 0, smg);
    force_move_card_to_hand(&mut e, &v, 0, dism);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dism,
            targets: vec![1],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    pass_nullify(&mut e);
    assert!(
        e.open_choice().is_none(),
        "g_shenmiguo must NOT trigger after Dismantle leaves target empty (canUse == false)"
    );
    assert!(
        e.debug_hand(0).contains(&smg),
        "shenmiguo remains unconsumed in hand when target cannot be re-targeted"
    );

    // 2. Next turn: use_as card (yuchanqian as Sha) + consuming yuchankan as Shenmiguo discards cleanly.
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    e.debug_set_hp(1, 4);
    let ycq = find_kind(&e, CardKind::Yuchanqian);
    let yck = find_kind(&e, CardKind::Yuchankan);
    force_move_card_to_hand(&mut e, &v, 0, ycq);
    force_move_card_to_hand(&mut e, &v, 0, yck);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ycq,
            targets: vec![1],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "yuchanqian as Sha: {out:?}");
    let ch = e.open_choice().expect("g_shenmiguo must prompt for use_as Sha with yuchankan");
    assert_eq!(ch.observer, 0);
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(1), 2, "yuchanqian as Sha hit + shenmiguo replay hit = 2 damage");
    assert!(
        !e.state_core().table.contains(&yck) && e.state_core().discard.contains(&yck),
        "yuchankan consumed by g_shenmiguo must land in discard, NOT leak to table"
    );

    // 3. Skill view_as card (Wusheng: red Tao as Sha) preserves use_as across ShenmiguoRestage!
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    e.debug_set_hp(1, 4);
    e.debug_grant(0, "武圣");
    let red_tao = find_kind_suit(&e, CardKind::Tao, |s| {
        matches!(s, Suit::Heart | Suit::Diamond)
    })
    .expect("red tao");
    force_move_card_to_hand(&mut e, &v, 0, red_tao);
    force_move_card_to_hand(&mut e, &v, 0, smg);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_tao,
            targets: vec![1],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "wusheng red tao as Sha: {out:?}");
    let ch2 = e
        .open_choice()
        .expect("g_shenmiguo must prompt for skill use_as (Wusheng tao as Sha)");
    assert_eq!(ch2.observer, 0);
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(1), 2, "wusheng tao-as-Sha + shenmiguo replay must deal 2 damage (NOT heal P1!)");
    assert!(e.state_core().discard.contains(&smg));
    assert!(e.state_core().table.is_empty());
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_r3_shencaojie_trick_only_and_nullifiable() {
    let (v, mut e) = start_shahai_swd_only(9242, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. Basic Sha & delayed trick (ShanDian) damage must NOT open shencaojie window.
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let scj1 = find_kind(&e, CardKind::Shencaojie);
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, scj1);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(e.open_choice().is_none(), "basic Sha damage must not trigger shencaojie");

    let shandian = find_kind(&e, CardKind::ShanDian);
    let spade5 = (0..e.catalog().cards.len() as u32)
        .find(|&id| {
            let c = &e.catalog().cards[id as usize];
            c.suit == Suit::Spade && c.rank == 5
        })
        .unwrap();
    {
        force_move_card_to_hand(&mut e, &v, 0, shandian);
        let mut snap = e.save();
        for p in &mut snap.core.players {
            p.hand.retain(|&x| x != shandian);
        }
        snap.core.players[0].judge.push(shandian);
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    // Step to seat 0 Judge phase on next turn with Spade 5 on deck top so ShanDian hits seat 0.
    for _ in 0..48 {
        if e.turn().current == 1 && e.turn().phase == Phase::End && e.open_choice().is_none() {
            place_top_of_deck(&mut e, &v, &[spade5]);
            let _ = submit(&mut e, 1, GameAction::NextPhase);
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                vec![]
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            answer_open(&mut e, sel);
        } else {
            let cur = e.turn().current;
            let _ = submit(&mut e, cur, GameAction::NextPhase);
        }
    }
    assert!(
        e.open_choice().is_none(),
        "delayed trick ShanDian damage must NOT trigger shencaojie!"
    );
    assert_eq!(e.debug_hp(0), 1, "ShanDian dealt 3 Thunder damage without shencaojie");

    // 1b. Non-trick Jiguan damage (jiguantong) must NOT open shencaojie window.
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let jgt = (0..e.catalog().cards.len() as u32)
        .find(|&id| e.catalog().cards[id as usize].name.contains("jiguantong"))
        .unwrap();
    force_move_card_to_hand(&mut e, &v, 0, scj1);
    force_move_card_to_hand(&mut e, &v, 0, jgt);
    let hp1_before = e.debug_hp(1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jgt,
            targets: vec![1],
        },
    );
    assert!(
        e.open_choice().is_none(),
        "Jiguan (jiguantong) damage must NOT trigger shencaojie!"
    );
    assert_eq!(e.debug_hp(1), hp1_before - 1);

    // 2. Normal trick damage (Jingleishan): confirming shencaojie uses it (opens nullify_window + logs CardPlayed),
    //    and Nullify cancels the +1 damage bonus!
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    let jls = find_kind(&e, CardKind::Jingleishan);
    let wuxie = find_kind(&e, CardKind::Nullify);
    force_move_card_to_hand(&mut e, &v, 0, scj1);
    force_move_card_to_hand(&mut e, &v, 0, jls);
    force_move_card_to_hand(&mut e, &v, 1, wuxie);

    let log_mark = e.log().len();
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jls,
            targets: vec![],
        },
    );
    // Pass Jingleishan's nullify window -> P1 has no Sha -> enters DamageBefore!
    pass_nullify(&mut e);
    let ch_scj = e.open_choice().expect("shencaojie confirm prompt");
    assert_eq!(ch_scj.observer, 0);
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    let nw = e
        .open_choice()
        .expect("using shencaojie must open nullify_window!");
    assert_eq!(nw.spec.prompt, "nullify_window");
    assert!(nw.spec.options.contains(&ChoiceOption::Card(wuxie)));
    assert!(
        e.log()[log_mark..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardPlayed { card, .. } if *card == scj1)),
        "shencaojie use emits real CardPlayed"
    );
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(wuxie)]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.debug_hp(1),
        3,
        "nullified shencaojie must NOT increase Jingleishan damage (4 -> 3)"
    );

    // 3. Un-nullified shencaojie deals 1 + 1 = 2 damage cleanly.
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let scj2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != scj1 && e.catalog().card_kind(id) == CardKind::Shencaojie)
        .unwrap();
    force_move_card_to_hand(&mut e, &v, 0, scj2);
    force_move_card_to_hand(&mut e, &v, 0, jls);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jls,
            targets: vec![],
        },
    );
    pass_nullify(&mut e);
    let ch2 = e.open_choice().expect("shencaojie confirm");
    assert_eq!(ch2.observer, 0);
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(1), 1, "un-nullified shencaojie increases Jingleishan damage 3 -> 1 (-2)");
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_r3_zhufangshenshi_nullify_and_usecardafter_draw() {
    let (v, mut e) = start_shahai_swd_only(9243, 4);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    for s in 0..4 {
        clear_hand_to_discard(&mut e, &v, s);
    }

    let zfs1 = find_kind(&e, CardKind::Zhufangshenshi);
    let zfs2 = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != zfs1 && e.catalog().card_kind(id) == CardKind::Zhufangshenshi)
        .unwrap();
    let wuxie = find_kind(&e, CardKind::Nullify);
    let snatch = find_kind(&e, CardKind::Snatch);
    let p2_card = find_kind(&e, CardKind::Sha);

    force_move_card_to_hand(&mut e, &v, 0, zfs1);
    force_move_card_to_hand(&mut e, &v, 0, zfs2);
    force_move_card_to_hand(&mut e, &v, 0, snatch);
    force_move_card_to_hand(&mut e, &v, 1, wuxie);
    force_move_card_to_hand(&mut e, &v, 2, p2_card);

    // 1. Play zfs1 on distance-2 seat 2 -> opens nullify_window; P1 nullifies it:
    //    zhufangshenshi_target stays None, BUT g_zhufangshenshi (useCardAfter) STILL draws 1!
    let h0_before = e.debug_hand(0).len();
    let out1 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zfs1,
            targets: vec![2],
        },
    );
    assert!(!matches!(out1, CommandOutcome::Rejected(_)));
    let nw = e
        .open_choice()
        .expect("zhufangshenshi must open nullify_window");
    assert_eq!(nw.spec.prompt, "nullify_window");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(wuxie)]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().turn.zhufangshenshi_target,
        None,
        "nullified zhufangshenshi must NOT set distance-waiver buff"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        h0_before,
        "g_zhufangshenshi must still draw 1 on useCardAfter even when nullified (-1 + 1)"
    );
    let sn_blocked = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    );
    assert!(matches!(sn_blocked, CommandOutcome::Rejected(_)));

    // 2. Play zfs2 without nullify -> buff set + draws 1 + Snatch distance 2 works.
    let h0_before2 = e.debug_hand(0).len();
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zfs2,
            targets: vec![2],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().turn.zhufangshenshi_target, Some(2));
    assert_eq!(e.debug_hand(0).len(), h0_before2);
    let sn_ok = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    );
    assert!(!matches!(sn_ok, CommandOutcome::Rejected(_)));
    drain_pass_all(&mut e);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_r3_fuxiqin_grants_kongxin_equip_skill() {
    let (v, mut e) = start_shahai_swd_only(9244, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);

    let fxq = find_card_name(&e, "fuxiqin");
    assert_eq!(
        e.catalog().cards[fxq as usize].equip_skills,
        vec!["kongxin"],
        "fuxiqin CardDef must declare equip_skills: [\"kongxin\"]"
    );
    let kongxin_def = e.catalog().skill_by_name("kongxin").expect("kongxin in catalog");
    let yry = find_kind(&e, CardKind::Yuruyi);
    force_move_card_to_hand(&mut e, &v, 0, fxq);
    force_move_card_to_hand(&mut e, &v, 0, yry);

    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fxq,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        e.state_core().skills.contains_key(&(0, kongxin_def)),
        "equipping fuxiqin must grant kongxin"
    );
    if let Some(wx_def) = e.catalog().skill_by_name("wuxingpan_skill") {
        assert!(!e.state_core().skills.contains_key(&(0, wx_def)));
    }

    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yry,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        !e.state_core().skills.contains_key(&(0, kongxin_def)),
        "replacing fuxiqin revokes kongxin"
    );
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_r3_baishouzhihu_triggers_on_discard_end_any_phase() {
    let (v, mut e) = start_shahai_swd_only(9245, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    for s in 0..3 {
        clear_hand_to_discard(&mut e, &v, s);
    }

    let bszh = find_kind(&e, CardKind::Baishouzhihu);
    let c1 = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 0, bszh);
    force_move_card_to_hand(&mut e, &v, 0, c1);
    let p1_sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 1, p1_sha);

    // 1. Discard phase with 0 discards (2 <= hp 4) MUST NOT trigger g_baishouzhihu.
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    assert_eq!(e.turn().phase, Phase::Discard);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    assert!(
        e.open_choice().is_none(),
        "empty Discard phase (0 discards) must NOT open baishouzhihu_target"
    );

    // 2. Discard during Play phase (seat 1 uses Dismantle on seat 0's c1) triggers g_baishouzhihu for seat 0!
    advance_to_play(&mut e, 1);
    let dism = find_kind(&e, CardKind::Dismantle);
    let p2_card = find_kind(&e, CardKind::Duel);
    clear_hand_to_discard(&mut e, &v, 1);
    clear_hand_to_discard(&mut e, &v, 2);
    force_move_card_to_hand(&mut e, &v, 1, dism);
    force_move_card_to_hand(&mut e, &v, 2, p2_card);
    // Keep only [c1, bszh] in seat 0 hand with c1 FIRST so Dismantle discards c1, keeping bszh in hand.
    {
        let mut snap = e.save();
        snap.core.players[0].hand.retain(|&x| x != c1 && x != bszh);
        snap.core.players[0].hand.push(c1);
        snap.core.players[0].hand.push(bszh);
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: dism,
            targets: vec![0],
        },
    );
    pass_nullify(&mut e);
    let bszh_def = e.catalog().skill_by_name("g_baishouzhihu");
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "baishouzhihu_target" {
            break;
        }
        if c.spec.prompt == "trigger_order" {
            let pick = bszh_def
                .filter(|d| c.spec.options.contains(&ChoiceOption::Skill(*d)))
                .map(ChoiceOption::Skill)
                .unwrap_or_else(|| c.spec.options[0]);
            answer_open(&mut e, vec![pick]);
        } else if c.spec.prompt == "trigger_confirm" {
            answer_open(&mut e, vec![ChoiceOption::Confirm]);
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            answer_open(&mut e, vec![ChoiceOption::Pass]);
        } else {
            break;
        }
    }
    let ch = e
        .open_choice()
        .expect("CardsDiscarded during Play phase must trigger g_baishouzhihu");
    assert_eq!(ch.observer, 0);
    assert_eq!(ch.spec.prompt, "baishouzhihu_target");
    e = roundtrip_choice(e, &v);
    let planner = BoundedPlanner::new(64);
    let plan = e.plan_action(0, &planner);
    let _ = e.apply_plan(&plan).expect("ai apply baishouzhihu");
    drain_pass_all(&mut e);
    assert!(e.debug_hand(0).contains(&bszh));
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7a_r3_yuchan_response_kind_cards_core() {
    let (v, mut e) = start_shahai_swd_only(9246, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);

    // 1. Duel: P1 responds using Yuchanqian as Sha -> P0 has no Sha -> P0 loses 1 HP.
    let duel = find_kind(&e, CardKind::Duel);
    let ycq = find_kind(&e, CardKind::Yuchanqian);
    force_move_card_to_hand(&mut e, &v, 0, duel);
    force_move_card_to_hand(&mut e, &v, 1, ycq);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![1],
        },
    );
    pass_nullify(&mut e);
    let duel_ch = e.open_choice().expect("duel_sha for P1 holding yuchanqian");
    assert_eq!(duel_ch.observer, 1);
    assert_eq!(duel_ch.spec.prompt, "duel_sha");
    assert!(
        duel_ch.spec.options.contains(&ChoiceOption::Card(ycq)),
        "response_kind_cards_core must include Yuchanqian as Sha"
    );
    answer_open(&mut e, vec![ChoiceOption::Card(ycq)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(0), 3, "P1's yuchanqian Sha won Duel vs P0");
    assert!(e.state_core().discard.contains(&ycq));

    // 2. AOE SavageAttack: P1 responds using Yuchanqian as Sha -> takes 0 damage.
    let nanman = find_kind(&e, CardKind::SavageAttack);
    force_move_card_to_hand(&mut e, &v, 0, nanman);
    force_move_card_to_hand(&mut e, &v, 1, ycq);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: nanman,
            targets: vec![],
        },
    );
    pass_nullify(&mut e);
    let aoe_ch = e.open_choice().expect("aoe_response for P1 holding yuchanqian");
    assert_eq!(aoe_ch.observer, 1);
    assert_eq!(aoe_ch.spec.prompt, "aoe_response");
    assert!(aoe_ch.spec.options.contains(&ChoiceOption::Card(ycq)));
    answer_open(&mut e, vec![ChoiceOption::Card(ycq)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(1), 4, "yuchanqian defended SavageAttack cleanly");
    assert_eq!(total_card_set(&e), init_set);
}

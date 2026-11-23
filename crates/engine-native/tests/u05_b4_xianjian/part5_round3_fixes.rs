use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::machine::LogEvent;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, RejectReason};

fn find_non_offdeck_card(e: &Engine, kind: CardKind, suit_pred: fn(Suit) -> bool) -> Option<u32> {
    let cat = e.catalog();
    (0..cat.cards.len() as u32).find(|&c| {
        !cat.cards[c as usize].off_deck
            && cat.card_kind(c) == kind
            && suit_pred(cat.cards[c as usize].suit)
    })
}

#[test]
fn u05_b4_r3_xjzhimeng2_target_vs_owner_and_token_isolation() {
    // 3 seats so Player 0's TurnBegin (`xjzhimeng3`) doesn't expire `xjzhimeng2` before Player 1 -> Player 2 tests run.
    let (v, mut e) = start_xianjian_only(8481, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "ctianfu");
    e.debug_grant(0, "xjzhimeng");

    // Put an unrelated Basic (`shan`) token onto Player 2's `off_court` via `ctianfu`,
    // then attach a Trick (`wuzhong`) as Player 2's tracked `xj_zhimeng_card` at Player 0's End phase.
    let ctianfu_shan = e.debug_place(0, "shan");
    let ct_def = e.catalog().skill_by_name("ctianfu").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: ct_def });
    answer_open(&mut e, vec![ChoiceOption::Card(ctianfu_shan)]);
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().player(2).off_court, vec![ctianfu_shan]);

    let dark_wz = find_non_offdeck_card(&e, CardKind::Wuzhong, |_| true).expect("wuzhong card");
    place_top_of_deck(&mut e, &v, &[dark_wz]);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> triggers xjzhimeng
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().player(2).off_court, vec![ctianfu_shan, dark_wz]);

    // Step into Player 1 Play: test non-matching Basic (`sha` matching unrelated `ctianfu_shan`)
    // vs matching Trick (`duel` targeting Player 2).
    step_full_turn_to(&mut e, 1);
    e.debug_clear_hand(2);
    let p1_sha = e.debug_place(1, "sha");
    let p1_duel = e.debug_place(1, "duel");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: p1_sha,
            targets: vec![2],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        e.debug_hand(2).is_empty(),
        "unrelated ctianfu Shan in off_court must NOT make Sha trigger xjzhimeng2 when tracked dark card is Trick"
    );

    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: p1_duel,
            targets: vec![2],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(
        e.debug_hand(2).len(),
        1,
        "other player's Trick (Duel) targeting xjzhimeng2 owner MUST trigger xjzhimeng2 draw 1"
    );

    // Step into Player 2 Play: owner using Trick (`guohe`) targeting OTHER player (deals 0 dmg so ctianfu2 doesn't trigger)
    // must NOT trigger own xjzhimeng2, while owner self-targeting Wuzhong (empty `targets`) MUST trigger (`liumengli.rs:207`).
    step_full_turn_to(&mut e, 2);
    e.debug_clear_hand(2);
    e.debug_clear_hand(0);
    let _p0_dummy = e.debug_place(0, "tao");
    let p2_guohe = find_non_offdeck_card(&e, CardKind::Dismantle, |_| true).expect("guohe card");
    force_move_card_to_hand(&mut e, &v, 2, p2_guohe);
    let p2_wz = e.debug_place(2, "wuzhong");
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: p2_guohe,
            targets: vec![0],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(
        e.debug_hand(2),
        vec![p2_wz],
        "xjzhimeng2 owner playing Dismantle targeting Player 0 must NOT trigger own xjzhimeng2"
    );

    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: p2_wz,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(
        e.debug_hand(2).len(),
        3,
        "owner self-targeting Wuzhong (u.targets.is_empty()) draws 2 + 1 from xjzhimeng2"
    );
}

#[test]
fn u05_b4_r3_eight_phase_use_usable1_skills_reject_second_activate() {
    // 1. sajin
    {
        let (_, mut e) = start_xianjian_only(84821, 2);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_set_hp(1, 2);
        let c1 = e.debug_place(0, "sha");
        let _c2 = e.debug_place(0, "shan");
        e.debug_grant(0, "sajin");
        let def = e.catalog().skill_by_name("sajin").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
        answer_open(&mut e, vec![ChoiceOption::Player(1)]);
        drain_pass_all(&mut e);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "sajin usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 2. xtanxi (also verifies cancel refund before commit)
    {
        let (_, mut e) = start_xianjian_only(84822, 2);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let c1 = e.debug_place(0, "sha");
        let _c2 = e.debug_place(0, "shan");
        let _p1 = e.debug_place(1, "sha");
        e.debug_grant(0, "xtanxi");
        let def = e.catalog().skill_by_name("xtanxi").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![]); // cancel before spending card
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "xtanxi usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 3. ctianfu (also verifies cancel refund before commit)
    {
        let (_, mut e) = start_xianjian_only(84823, 3);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let s1 = e.debug_place(0, "shan");
        let _s2 = e.debug_place(0, "shan");
        e.debug_grant(0, "ctianfu");
        let def = e.catalog().skill_by_name("ctianfu").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![]); // cancel before spending Shan
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(s1)]);
        answer_open(&mut e, vec![ChoiceOption::Player(1)]);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "ctianfu usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 4. minsha
    {
        let (_, mut e) = start_xianjian_only(84824, 3);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_set_max_hp(1, 5);
        e.debug_set_hp(1, 5);
        e.debug_set_max_hp(2, 5);
        e.debug_set_hp(2, 5);
        let c1 = e.debug_place(0, "sha");
        let c2 = e.debug_place(0, "shan");
        let _c3 = e.debug_place(0, "tao");
        let _c4 = e.debug_place(0, "wuzhong");
        e.debug_grant(0, "minsha");
        let def = e.catalog().skill_by_name("minsha").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
        answer_open(&mut e, vec![ChoiceOption::Player(1)]);
        drain_pass_all(&mut e);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "minsha usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 5. yuexing (also verifies cancel refund before commit)
    {
        let (_, mut e) = start_xianjian_only(84825, 3);
        advance_to_play(&mut e, 0);
        e.debug_grant(0, "yuexing");
        let def = e.catalog().skill_by_name("yuexing").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![]); // cancel before picking target
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Player(1)]);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "yuexing usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 6. tianjian
    {
        let (_, mut e) = start_xianjian_only(84826, 2);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let s1 = e.debug_place(0, "sha");
        let _s2 = e.debug_place(0, "sha");
        e.debug_grant(0, "tianjian");
        let def = e.catalog().skill_by_name("tianjian").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(s1)]);
        drain_pass_all(&mut e);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "tianjian usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 7. xuanmo
    {
        let (_, mut e) = start_xianjian_only(84827, 2);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let t1 = e.debug_place(0, "tao");
        let _t2 = e.debug_place(0, "tao");
        e.debug_grant(0, "xuanmo");
        let def = e.catalog().skill_by_name("xuanmo").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(t1)]);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "xuanmo usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }

    // 8. zhuyue
    {
        let (_, mut e) = start_xianjian_only(84828, 3);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let nb1 = e.debug_place(0, "wuzhong");
        let _nb2 = e.debug_place(0, "duel");
        let _ = e.debug_place(1, "tao");
        let _ = e.debug_place(2, "tao");
        e.debug_grant(0, "zhuyue");
        let def = e.catalog().skill_by_name("zhuyue").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Pending { .. }
        ));
        answer_open(&mut e, vec![ChoiceOption::Card(nb1)]);
        answer_open(&mut e, vec![ChoiceOption::Player(1)]);
        let r = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        assert!(
            matches!(r, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
            "zhuyue usable: 1 must reject 2nd ActivateSkill: {r:?}"
        );
    }
}

#[test]
fn u05_b4_r3_xiepan_supports_all_basic_kinds_like_jiu_and_elemental_sha() {
    let (v, mut e) = start_xianjian_full(8483, 2);
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "xiepan");

    let jiu_cid = find_non_offdeck_card(&e, CardKind::Jiu, |_| true).expect("Jiu in deck");
    let fire_cid = find_non_offdeck_card(&e, CardKind::FireSha, |_| true).expect("FireSha in deck");
    let lei_cid = find_non_offdeck_card(&e, CardKind::LeiSha, |_| true).expect("LeiSha in deck");

    // Negative: discarding Sha while Jiu (`is_basic_kind`) remains in hand MUST NOT trigger xiepan
    {
        let mut en = content_native_loader::restore_locked_engine(&e.save(), &v).unwrap();
        en.debug_clear_hand(0);
        let sha = en.debug_place(0, "sha");
        force_move_card_to_hand(&mut en, &v, 0, jiu_cid);
        en.debug_set_hp(0, 1);
        let _ = submit(&mut en, 0, GameAction::NextPhase); // Play -> Discard
        answer_open(&mut en, vec![ChoiceOption::Card(sha)]);
        assert!(
            en.open_choice()
                .is_none_or(|c| c.spec.prompt != "xj_xiepan_cards"),
            "Jiu remaining in hand is basic; xiepan must NOT trigger"
        );
    }

    // Positive: discarding Jiu / FireSha / LeiSha leaving only non-basics MUST trigger xiepan
    for (label, basic_cid) in [("Jiu", jiu_cid), ("FireSha", fire_cid), ("LeiSha", lei_cid)] {
        let mut ep = content_native_loader::restore_locked_engine(&e.save(), &v).unwrap();
        ep.debug_clear_hand(0);
        force_move_card_to_hand(&mut ep, &v, 0, basic_cid);
        let duel = ep.debug_place(0, "duel");
        ep.debug_set_hp(0, 1);
        let _ = submit(&mut ep, 0, GameAction::NextPhase); // Play -> Discard
        answer_open(&mut ep, vec![ChoiceOption::Card(basic_cid)]);
        let xp = ep
            .open_choice()
            .unwrap_or_else(|| panic!("discarding {label} leaving Duel should trigger xiepan"));
        assert_eq!(xp.spec.prompt, "xj_xiepan_cards");
        answer_open(&mut ep, vec![ChoiceOption::Card(duel)]);
        assert!(
            !ep.debug_hand(0).contains(&duel)
                && ep
                    .debug_hand(0)
                    .iter()
                    .any(|&c| ep.catalog().card_kind(c).equip_slot().is_some()),
            "xiepan triggered after {label} discard -> discarded Duel and discovered equip"
        );
    }
}

#[test]
fn u05_b4_r3_yujia_ignores_equip_used_as_non_equip_via_tuoqiao() {
    let (v, mut e) = start_xianjian_only(8484, 2);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_max_hp(0, 5);
    e.debug_set_hp(0, 5);
    e.debug_set_hp(1, 5);
    e.debug_grant(0, "yujia");
    e.debug_grant(0, "tuoqiao");

    // Put a real `!off_deck` black Equip into Player 0's hand and advance to Player 1's Play begin.
    let black_eq = find_non_offdeck_card(
        &e,
        CardKind::Weapon,
        |s| matches!(s, Suit::Spade | Suit::Club),
    )
    .or_else(|| {
        let cat = e.catalog();
        (0..cat.cards.len() as u32).find(|&c| {
            !cat.cards[c as usize].off_deck
                && cat.card_kind(c).equip_slot().is_some()
                && matches!(cat.cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
    })
    .expect("non-off_deck black equip");
    force_move_card_to_hand(&mut e, &v, 0, black_eq);

    while e
        .open_choice()
        .is_none_or(|c| c.spec.prompt != "xj_u05_tuoqiao_card")
    {
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                Vec::new()
            };
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        } else {
            let cur = e.turn().current;
            let _ = submit(&mut e, cur, GameAction::NextPhase);
        }
    }

    // Use `black_eq` AS `Shihuifen` via `tuoqiao`. Assert `xj_u05_yujia_confirm` NEVER triggers.
    answer_open(&mut e, vec![ChoiceOption::Card(black_eq)]);
    for _ in 0..20 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        assert_ne!(
            c.spec.prompt, "xj_u05_yujia_confirm",
            "yujia must NOT trigger when an equip card is used via use_as=Shihuifen (tuoqiao)"
        );
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
    assert!(
        e.debug_hand(0).is_empty(),
        "no Jiguan gained from tuoqiao converting black equip into Shihuifen"
    );

    // Control: actually playing an equip as an Equip on Player 0's Play MUST trigger yujia.
    step_full_turn_to(&mut e, 0);
    let real_eq = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: real_eq,
            targets: vec![],
        },
    );
    let yj = e.open_choice().expect("real equip triggers yujia");
    assert_eq!(yj.spec.prompt, "xj_u05_yujia_confirm");
}

#[test]
fn u05_b4_r3_fenglue_skips_play_phase_and_draws_on_target_damage_end() {
    // Subcase A: Cancel keeps Play phase intact
    let (_, mut ec) = start_xianjian_only(84851, 3);
    advance_to_play(&mut ec, 0);
    for p in 0..3 {
        ec.debug_clear_hand(p);
        ec.debug_set_max_hp(p, 6);
        ec.debug_set_hp(p, 6);
    }
    ec.debug_grant(0, "fenglue");
    step_full_turn_to(&mut ec, 1);
    ec.debug_clear_hand(1);
    step_full_turn_to(&mut ec, 2);
    ec.debug_clear_hand(2);
    // Advance to Player 0 Play where fenglue opens
    while ec.open_choice().is_none() {
        let cur = ec.turn().current;
        let _ = submit(&mut ec, cur, GameAction::NextPhase);
    }
    assert_eq!(
        ec.open_choice().unwrap().spec.prompt,
        engine_native::content::host::WUDIAN_XJ_FENGLUE_TARGET
    );
    answer_open(&mut ec, vec![]); // cancel fenglue
    assert_eq!(
        ec.turn().phase,
        Phase::Play,
        "declining fenglue must NOT skip Play phase"
    );

    // Subcase B: Commit deals 2 separate damages (Sha + Duel) -> draws 2 via fenglue_draw + skips Play cleanly ONCE to Discard (even with DiscardToLimit)
    let (_, mut e) = start_xianjian_only(84852, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "fenglue");
    // Advance to Player 0's Draw phase cleanly, then set hp=2 & exact pre-Play hands before Draw -> Play.
    while !(e.turn().current == 0 && e.turn().phase == Phase::Draw) {
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else {
                vec![ChoiceOption::Pass]
            };
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        } else {
            let cur = e.turn().current;
            let _ = submit(&mut e, cur, GameAction::NextPhase);
        }
    }
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 2); // after fenglue_draw (+2), wuzhong + 2 drawn = 3 > hp 2 -> DiscardToLimit must open ONCE
    let sha = e.debug_place(0, "sha");
    let duel = e.debug_place(0, "duel");
    let wz = e.debug_place(0, "wuzhong");
    let log_mark = e.log().len();
    let phase_num_before = e.state_core().turn.phase_number;
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play (BeforeTriggers runs fenglue -> Commit skips Play -> Discard)
    assert_eq!(
        e.open_choice().unwrap().spec.prompt,
        engine_native::content::host::WUDIAN_XJ_FENGLUE_TARGET
    );
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    // Drain Sha/Duel responses ONLY until DiscardToLimit (or settled) so we assert DiscardToLimit fires ONCE.
    for _ in 0..20 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "discard_to_limit" {
            break;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }

    assert_eq!(e.debug_hp(1), 4, "Sha + Duel dealt 2 separate damage instances");
    assert!(!e.debug_hand(0).contains(&sha) && !e.debug_hand(0).contains(&duel));
    assert!(
        e.debug_hand(0).contains(&wz) && e.debug_hand(0).len() == 3,
        "fenglue_draw drew 2 cards at end of fenglue (wuzhong + 2 drawn = 3), got {:?}",
        e.debug_hand(0)
    );
    assert_eq!(
        e.turn().phase,
        Phase::Discard,
        "fenglue skipped Play phase directly to Discard"
    );
    let play_begin_count = e.log()[log_mark..]
        .iter()
        .filter(|ev| {
            matches!(
                ev,
                LogEvent::PhaseChanged {
                    player: 0,
                    phase: Phase::Play,
                    begin: true
                }
            )
        })
        .count();
    let discard_begin_count = e.log()[log_mark..]
        .iter()
        .filter(|ev| {
            matches!(
                ev,
                LogEvent::PhaseChanged {
                    player: 0,
                    phase: Phase::Discard,
                    begin: true
                }
            )
        })
        .count();
    assert_eq!(play_begin_count, 0, "Phase::Play begin:true must NEVER be entered");
    assert_eq!(
        discard_begin_count, 1,
        "Phase::Discard begin:true must fire EXACTLY once"
    );
    assert_eq!(
        e.state_core().turn.phase_number - phase_num_before,
        1,
        "phase_number must increment only once (for Discard, not skipped Play)"
    );
    let disc_ask = e
        .open_choice()
        .expect("single discard_to_limit prompt because hand(3) > hp(2)");
    assert_eq!(disc_ask.observer, 0);
    assert_eq!(disc_ask.spec.prompt, "discard_to_limit");
    assert_eq!((disc_ask.spec.min, disc_ask.spec.max), (1, 1));
    answer_open(&mut e, vec![ChoiceOption::Card(wz)]);
    assert!(
        e.open_choice().is_none(),
        "DiscardToLimit must NOT open a second time in the same Discard phase"
    );
    let r = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(r, CommandOutcome::Rejected(RejectReason::WrongPhase)),
        "cannot play cards because Play phase was skipped into Discard: {r:?}"
    );

    // Subcase C: Target dodges Sha -> 0 damage -> 0 drawn, Play STILL skipped to Discard
    let (_, mut e0) = start_xianjian_only(84853, 2);
    advance_to_play(&mut e0, 0);
    e0.debug_clear_hand(0);
    e0.debug_clear_hand(1);
    e0.debug_grant(0, "fenglue");
    step_full_turn_to(&mut e0, 1);
    while !(e0.turn().current == 0 && e0.turn().phase == Phase::Draw) {
        if let Some(c) = e0.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else {
                vec![ChoiceOption::Pass]
            };
            let _ = submit(
                &mut e0,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        } else {
            let cur = e0.turn().current;
            let _ = submit(&mut e0, cur, GameAction::NextPhase);
        }
    }
    e0.debug_clear_hand(0);
    e0.debug_clear_hand(1);
    let _sha0 = e0.debug_place(0, "sha");
    let p1_shan = e0.debug_place(1, "shan");
    let _ = submit(&mut e0, 0, GameAction::NextPhase); // Draw -> Play triggers fenglue
    answer_open(&mut e0, vec![ChoiceOption::Player(1)]);
    answer_open(&mut e0, vec![ChoiceOption::Card(p1_shan)]);
    drain_pass_all(&mut e0);
    assert_eq!(e0.debug_hp(1), 4);
    assert!(e0.debug_hand(0).is_empty(), "0 target damage -> 0 draw");
    assert_eq!(e0.turn().phase, Phase::Discard);
}

#[test]
fn u05_b4_r3_ctianfu2_deals_thunder_damage_before_he_discard_and_clear() {
    let (v, mut e) = start_xianjian_only(8486, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 4);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "ctianfu");
    let attached_shan = e.debug_place(0, "shan");
    let ct_def = e.catalog().skill_by_name("ctianfu").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: ct_def });
    answer_open(&mut e, vec![ChoiceOption::Card(attached_shan)]);
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);

    step_full_turn_to(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(1, "遗计");
    let p1_sha = e.debug_place(1, "sha");
    let p1_tao = e.debug_place(1, "tao");
    let ct2_def = e.catalog().skill_by_name("ctianfu2").unwrap();

    let log_mark = e.log().len();
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: p1_sha,
            targets: vec![2],
        },
    );
    drain_pass_all_until_yiji(&mut e);

    // At the suspended `DamageEnd` ("遗计") window of ctianfu2's thunder damage (step 0):
    // damage MUST already be applied, while random he discard (step 1) & ctianfu_clear (step 2) MUST NOT have run yet.
    let c = e.open_choice().expect("yiji trigger_confirm during ctianfu2 DamageEnd");
    assert_eq!(c.observer, 1);
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(e.debug_hp(1), 3, "step 0 Thunder damage already landed");
    assert!(
        e.debug_hand(1).contains(&p1_tao),
        "he card MUST still be intact during step 0 Damage resolution"
    );
    assert_eq!(e.state_core().player(1).off_court, vec![attached_shan]);
    assert_eq!(e.state_core().player(1).ctianfu_source, Some(0));
    assert!(e.state_core().skills.contains_key(&(1, ct2_def)));

    // Unbroken direct trace verifies exact log ordering: step 0 DamageDealt < step 1 he discard < step 2 ctianfu_clear;
    // archive roundtrip on a separate branch verifies mid-step0 restore parity.
    let mut e_rt = roundtrip_choice(e.clone(), &v);
    answer_open(&mut e_rt, vec![ChoiceOption::Pass]);
    drain_pass_all(&mut e_rt);
    assert!(
        !e_rt.debug_hand(1).contains(&p1_tao) && e_rt.state_core().discard.contains(&p1_tao),
        "restored roundtrip step 1 discarded he card after Damage resolved"
    );
    assert!(e_rt.state_core().player(1).off_court.is_empty());
    assert!(e_rt.state_core().discard.contains(&attached_shan));
    assert_eq!(e_rt.state_core().player(1).ctianfu_source, None);
    assert!(!e_rt.state_core().skills.contains_key(&(1, ct2_def)));

    answer_open(&mut e, vec![ChoiceOption::Pass]);
    drain_pass_all(&mut e);

    assert!(
        !e.debug_hand(1).contains(&p1_tao) && e.state_core().discard.contains(&p1_tao),
        "step 1 discarded he card after Damage resolved"
    );
    assert!(e.state_core().player(1).off_court.is_empty());
    assert!(e.state_core().discard.contains(&attached_shan));
    assert_eq!(e.state_core().player(1).ctianfu_source, None);
    assert!(!e.state_core().skills.contains_key(&(1, ct2_def)));

    let slice = &e.log()[log_mark..];
    let dmg_idx = slice
        .iter()
        .position(|ev| {
            matches!(
                ev,
                LogEvent::DamageDealt {
                    source: Some(0),
                    target: 1,
                    amount: 1,
                    presentation: Some(p),
                    ..
                } if matches!(
                    p.as_ref(),
                    engine_api::PublicPresentationEvent::DamageDealt {
                        nature: engine_api::PresentationNature::Thunder,
                        ..
                    }
                )
            )
        })
        .expect("ctianfu2 Thunder DamageDealt logged");
    let tao_disc_idx = slice
        .iter()
        .position(|ev| matches!(ev, LogEvent::CardDiscarded { player: 1, cards } if cards == &vec![p1_tao]))
        .expect("step 1 he CardDiscarded logged");
    let shan_clear_idx = slice
        .iter()
        .position(|ev| matches!(ev, LogEvent::CardDiscarded { player: 1, cards } if cards == &vec![attached_shan]))
        .expect("step 2 ctianfu_clear CardDiscarded logged");
    assert!(
        dmg_idx < tao_disc_idx && tao_disc_idx < shan_clear_idx,
        "expected step 0 Damage ({dmg_idx}) < step 1 he discard ({tao_disc_idx}) < step 2 clear ({shan_clear_idx})"
    );

    // Subcase B (#273): ctianfu2 holder deals damage with their LAST card (hand.is_empty() && equips.is_empty()).
    // Step 1 MUST guard against `pool.is_empty()` without panicking on `rng.gen_range(0)` and clean up ctianfu.
    let (_, mut e2) = start_xianjian_only(8487, 3);
    advance_to_play(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
        e2.debug_set_max_hp(p, 4);
        e2.debug_set_hp(p, 4);
    }
    e2.debug_grant(0, "ctianfu");
    let shan2 = e2.debug_place(0, "shan");
    let ct_def2 = e2.catalog().skill_by_name("ctianfu").unwrap();
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: ct_def2 });
    answer_open(&mut e2, vec![ChoiceOption::Card(shan2)]);
    answer_open(&mut e2, vec![ChoiceOption::Player(1)]);
    step_full_turn_to(&mut e2, 1);
    e2.debug_clear_hand(1);
    e2.debug_clear_hand(2);
    let last_sha = e2.debug_place(1, "sha");
    let ct2_def2 = e2.catalog().skill_by_name("ctianfu2").unwrap();
    submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: last_sha,
            targets: vec![2],
        },
    );
    drain_pass_all(&mut e2);
    assert!(e2.debug_hand(1).is_empty() && e2.state_core().player(1).equips.is_empty());
    assert_eq!(e2.debug_hp(1), 3, "ctianfu2 Thunder damage still applies with empty he");
    assert!(e2.state_core().player(1).off_court.is_empty());
    assert!(e2.state_core().discard.contains(&shan2));
    assert_eq!(e2.state_core().player(1).ctianfu_source, None);
    assert!(!e2.state_core().skills.contains_key(&(1, ct2_def2)));
}

fn drain_pass_all_until_yiji(e: &mut Engine) {
    for _ in 0..20 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.observer == 1 && c.spec.prompt == "trigger_confirm" {
            break;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

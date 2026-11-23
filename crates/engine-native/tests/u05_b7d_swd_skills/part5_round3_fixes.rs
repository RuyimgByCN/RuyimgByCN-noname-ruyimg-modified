use super::common::*;

fn check_swdtianshu_exposed(
    e: &mut Engine,
    ts_def: u16,
    general_name: &str,
    expected_skills: &[&str],
) {
    e.debug_remove_general(1);
    e.assign_general_named(1, general_name).expect(general_name);
    advance_to_play(e, 0);
    e.debug_clear_hand(0);
    let c_pay = place_matching_card(e, 0, |_, d| d.kind == CardKind::Le);
    let out = submit(e, 0, GameAction::ActivateSkill { skill: ts_def });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "activate swdtianshu for {general_name} failed: {out:?}"
    );
    let ask_tsc = e.open_choice().expect("swdtianshu_card").clone();
    submit(
        e,
        0,
        GameAction::Answer {
            choice_id: ask_tsc.id,
            selection: vec![ChoiceOption::Card(c_pay)],
        },
    );
    let ask_tst = e.open_choice().expect("swdtianshu_target").clone();
    submit(
        e,
        0,
        GameAction::Answer {
            choice_id: ask_tst.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let ask_tss = e.open_choice().expect("swdtianshu_skill").clone();
    assert_eq!(ask_tss.spec.prompt, "swdtianshu_skill");
    let expected_tags: Vec<ChoiceOption> = (0..expected_skills.len())
        .map(|i| ChoiceOption::Tag(i as u8))
        .collect();
    assert_eq!(
        ask_tss.spec.options, expected_tags,
        "{general_name} should expose exact non-unique skill set {expected_skills:?}"
    );
    submit(
        e,
        0,
        GameAction::Answer {
            choice_id: ask_tss.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    settle_confirm(e);
    assert!(
        e.debug_has_skill(0, expected_skills[0]),
        "{general_name} tag 0 must grant {}",
        expected_skills[0]
    );
}

#[test]
fn u05_b7d_r3_rexue_optional_target_enabled_real_sha_and_oncard_draw() {
    let (v, mut e) = load_formal_swd_only(7981, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "rexue");
    e.debug_grant(2, engine_native::content::standard::KONG_CHENG);
    advance_to_play(&mut e, 0);

    // Case A: seat 0 has ONLY a Shan (no Sha -> hasSha false): seat 1's TurnBegin must NOT trigger rexue!
    e.debug_clear_hand(0);
    let _shan = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Shan);
    for _ in 0..8 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            panic!("rexue must NOT open any prompt without Sha in hand: {:?}", c.spec.prompt);
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, 1);

    // Case B: seat 0 now holds real Sha, seat 2 has kongcheng + empty hand -> targetEnabled is false!
    e.debug_clear_hand(0);
    let sha = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Sha);
    e.debug_clear_hand(2);
    for _ in 0..8 {
        if e.turn().current == 2 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            assert_ne!(
                c.observer, 0,
                "rexue must NOT trigger on empty-hand kongcheng seat 2: {:?}",
                c.spec.prompt
            );
            settle_confirm(&mut e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, 2);

    // Case C: seat 3 (non-kongcheng) starts turn -> rexue prompts optional trigger_confirm -> decline -> 0 draw & sha kept!
    for _ in 0..8 {
        if e.open_choice().is_some() || e.turn().current == 3 {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    let ask_opt = e.open_choice().expect("rexue optional trigger_confirm").clone();
    assert_eq!((ask_opt.observer, ask_opt.spec.prompt), (0, "trigger_confirm"));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ask_opt.id,
            selection: vec![ChoiceOption::Pass],
        },
    );
    settle_confirm(&mut e);
    assert_eq!(e.debug_hand(0), vec![sha], "declining rexue keeps Sha and draws 0");
    assert_eq!(e.debug_hp(3), 4);

    // Case D: seat 2 now has a card in hand (no longer kongcheng), AND is distance 2 from seat 0 ->
    // `targetEnabled` still passes; accept `trigger_confirm` then cancel `rexue_sha` -> 0 draw & sha kept!
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha1 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Sha);
    let _c2 = place_matching_card(&mut e, 2, |_, d| d.kind == CardKind::Tao);
    for _ in 0..16 {
        if e.open_choice().is_some_and(|c| c.observer == 0) {
            break;
        }
        if e.turn().current == 1 && e.turn().phase == Phase::Play {
            break;
        }
        if e.open_choice().is_some() {
            settle_confirm(&mut e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    let c_tc = e.open_choice().expect("rexue trigger_confirm on seat 1").clone();
    assert_eq!((c_tc.observer, c_tc.spec.prompt), (0, "trigger_confirm"));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tc.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c_sha_cancel = e.open_choice().expect("rexue_sha").clone();
    assert_eq!((c_sha_cancel.observer, c_sha_cancel.spec.prompt), (0, "rexue_sha"));
    assert!(c_sha_cancel.spec.allow_cancel);
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_sha_cancel.id,
            selection: Vec::new(),
        },
    );
    settle_confirm(&mut e);
    assert_eq!(e.debug_hand(0), vec![sha1], "cancelling rexue_sha draws 0 and keeps Sha");
    assert_eq!(e.debug_hp(1), 4);

    // Case E: seat 2 (distance 2, non-kongcheng because holding Tao) starts turn ->
    // accept `trigger_confirm`, archive-roundtrip mid-`rexue_sha`, pick real `sha1`,
    // check real `sha1` consumed, seat 0 drew 1 (`hand.len() == 1` != sha1), seat 2 takes 1 dmg!
    for _ in 0..12 {
        if e.open_choice().is_some_and(|c| c.observer == 0) {
            break;
        }
        if e.open_choice().is_some() {
            settle_confirm(&mut e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    let c_tc2 = e.open_choice().expect("rexue trigger_confirm on distance-2 seat 2").clone();
    assert_eq!((c_tc2.observer, c_tc2.spec.prompt), (0, "trigger_confirm"));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tc2.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c_sha = e.open_choice().expect("rexue_sha").clone();
    assert_eq!((c_sha.observer, c_sha.spec.prompt), (0, "rexue_sha"));
    assert!(c_sha.spec.options.contains(&ChoiceOption::Card(sha1)));
    let ai_plan = e.plan_action(0, &BoundedPlanner::new(64));
    assert!(matches!(ai_plan.action, GameAction::Answer { choice_id, .. } if choice_id == c_sha.id));
    let mut er = roundtrip_bytes(&v, &e);
    submit(
        &mut er,
        0,
        GameAction::Answer {
            choice_id: c_sha.id,
            selection: vec![ChoiceOption::Card(sha1)],
        },
    );
    settle_confirm(&mut er);
    assert_eq!(er.debug_hp(2), 3, "rexue Sha hit distance-2 seat 2 (4 -> 3)");
    assert!(!er.debug_hand(0).contains(&sha1), "real Sha must be consumed");
    assert_eq!(er.debug_hand(0).len(), 1, "consumed 1 Sha + drew 1 on use = 1 card");
}

#[test]
fn u05_b7d_r3_swdtianshu_excludes_all_swd_unique_skills_and_allows_huanyin() {
    let (_v, mut e) = load_formal_swd_only(7982, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "swdtianshu");
    let ts_def = e.catalog().skill_by_name("swdtianshu").unwrap();

    // Excludes pozhou + xuanyuan on swd_xuanyuanjianxian
    check_swdtianshu_exposed(&mut e, ts_def, "swd_xuanyuanjianxian", &["huajian"]);
    // Excludes hxunzhi on swd_huiyan
    check_swdtianshu_exposed(&mut e, ts_def, "swd_huiyan", &["hwendao", "lingfeng"]);
    // Excludes jinlin on swd_situqiang
    check_swdtianshu_exposed(&mut e, ts_def, "swd_situqiang", &["fengze", "lingyue"]);
    // Excludes shouyin on swd_muyue
    check_swdtianshu_exposed(&mut e, ts_def, "swd_muyue", &["xingzhui", "lingxian"]);
    // Allows non-unique huanyin on swd_qiner (huanying typo removed)
    check_swdtianshu_exposed(&mut e, ts_def, "swd_qiner", &["huanyin", "tianhuo", "xuanzhou"]);
    // Excludes cards.json companion helper subskills without _info (daixing2/daixing3/hyunshen2)
    check_swdtianshu_exposed(&mut e, ts_def, "swd_yuxiaoxue", &["huanhun", "daixing", "yinyue"]);
    check_swdtianshu_exposed(&mut e, ts_def, "swd_huyue", &["hyunshen", "fengming"]);
}

#[test]
fn u05_b7d_r3_zhanlu_activation_allows_spade_equip_without_hand() {
    let (_v, mut e) = load_formal_swd_only(7983, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "zhanlu");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_set_hp(1, 2);

    let zl_def = e.catalog().skill_by_name("zhanlu").unwrap();
    let spade_equip = (0..e.catalog().cards.len() as u32)
        .find(|&id| {
            let cd = &e.catalog().cards[id as usize];
            cd.kind.is_equip() && cd.suit == Suit::Spade
        })
        .expect("spade equip in catalog");
    {
        let core = e.state_core_mut();
        core.deck.retain(|&c| c != spade_equip);
        core.discard.retain(|&c| c != spade_equip);
        core.table.retain(|&c| c != spade_equip);
        for p in core.players.iter_mut() {
            p.hand.retain(|&c| c != spade_equip);
            p.equips.retain(|&c| c != spade_equip);
        }
        core.players[0].equips.push(spade_equip);
    }
    assert!(e.debug_hand(0).is_empty());
    assert!(
        e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: zl_def }),
        "zhanlu must be legal when holding 0 hand cards and 1 Spade equip"
    );
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: zl_def });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "zhanlu must activate with only Spade equip: {out:?}"
    );
    let c_disc = e.open_choice().expect("zhanlu_discard").clone();
    assert_eq!(c_disc.spec.prompt, "zhanlu_discard");
    assert_eq!(c_disc.spec.options, vec![ChoiceOption::Card(spade_equip)]);
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_disc.id,
            selection: vec![ChoiceOption::Card(spade_equip)],
        },
    );
    let c_tgt = e.open_choice().expect("zhanlu_targets").clone();
    assert_eq!(c_tgt.spec.prompt, "zhanlu_targets");
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_confirm(&mut e);
    assert!(!e.state_core().player(0).equips.contains(&spade_equip));
    assert!(e.state_core().discard.contains(&spade_equip));
    assert_eq!(e.debug_hp(1), 3, "zhanlu healed target seat 1 (2 -> 3)");
}

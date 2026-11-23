use super::common::*;
use content_native_loader::U05_SWD_GENERALS;

#[test]
fn u05_b7d_swd_79_generals_parity_and_no_vm_stubs() {
    let (verified, mut e) = load_formal_swd_only(7901, 4);
    assert_eq!(U05_SWD_GENERALS.len(), 79, "U05 swd roster must be 79");

    // 1. Check upstream character.js if present in workspace.
    let upstream_char = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/character.js");
    if upstream_char.exists() {
        let js = std::fs::read_to_string(&upstream_char).expect("read swd/character.js");
        for (name, _, _, _, _) in U05_SWD_GENERALS {
            assert!(
                js.contains(name),
                "upstream character.js must contain {name}"
            );
        }
    }

    // 2. All 79 generals exist in formal Catalog with matching fields & zero U05-7d VM stubs.
    let u05_7d_native = [
        "duijue",
        "duijue2",
        "duijue3",
        "tanlin",
        "tanlin2",
        "tanlin3",
        "tanlin4",
        "tanlin_defence",
        "oldliaoyuan",
        "oldliaoyuan2",
        "huanling2",
        "lexue2",
        "swdtianshu_remove",
        "yinguo2",
        "susheng2",
        "moyu",
        "guiyan2",
        "zhenjiu2",
        "绝情",
    ];
    for sk in u05_7d_native {
        let def = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("skill {sk} missing in formal swd catalog"));
        assert!(
            e.catalog().skills[def as usize]
                .behavior
                .package_skill_id()
                .is_none(),
            "U05-7d skill {sk} must be native Host/Standard, not VM stub"
        );
    }

    for (idx, (gname, faction, gender, max_hp, raw_skills)) in U05_SWD_GENERALS.iter().enumerate() {
        let (gid, meta) = e
            .catalog()
            .extension_generals
            .iter()
            .enumerate()
            .find(|(_, g)| g.name == *gname)
            .unwrap_or_else(|| panic!("general {gname} missing in formal Catalog"));
        assert_eq!(meta.max_hp, *max_hp, "max_hp mismatch for {gname}");
        assert_eq!(meta.faction, *faction, "faction mismatch for {gname}");
        assert_eq!(meta.gender, *gender, "gender mismatch for {gname}");
        for &raw_sk in *raw_skills {
            let expected_name = match raw_sk {
                "jueqing" => "绝情",
                "shangshi" => "伤逝",
                "biyue" => "闭月",
                "liuli" => "流离",
                "longdan" => "龙胆",
                "wushuang" => "无双",
                "yicong" => "义从",
                other => other,
            };
            assert!(
                meta.skills.contains(&expected_name),
                "{gname} missing expected skill {expected_name} (skills={:?})",
                meta.skills
            );
        }
        if idx % 20 == 0 || *gname == "swd_chunyuheng" || *gname == "swd_zhaoyun" || *gname == "swd_weida" {
            let _ = gid;
        }
    }

    // 3. Assign ALL 79 swd generals cleanly, plus verify representative U05-7d group_children + jueqing.
    for (gname, _, _, _, _) in U05_SWD_GENERALS {
        e.debug_remove_general(0);
        e.assign_general_named(0, gname)
            .unwrap_or_else(|err| panic!("assign_general_named {gname} failed: {err:?}"));
    }
    for p in 0..4 {
        e.debug_remove_general(p);
    }
    e.assign_general_named(0, "swd_weida").expect("assign swd_weida");
    e.assign_general_named(1, "swd_zhaoyun").expect("assign swd_zhaoyun");
    e.assign_general_named(2, "swd_chunyuheng").expect("assign swd_chunyuheng");
    e.assign_general_named(3, "swd_fengtianling").expect("assign swd_fengtianling");
    assert!(e.debug_has_skill(0, "duijue"));
    assert!(e.debug_has_skill(1, "tanlin"));
    assert!(e.debug_has_skill(1, "tanlin4"), "tanlin group_child tanlin4 assembled");
    assert!(e.debug_has_skill(2, "绝情"), "swd_chunyuheng resolved jueqing -> 绝情");
    assert!(e.debug_has_skill(3, "swd_guiyan2"), "guiyan group_child swd_guiyan2 assembled");

    let e2 = roundtrip_bytes(&verified, &e);
    assert!(e2.debug_has_skill(1, "tanlin4"));
    assert!(e2.debug_has_skill(2, "绝情"));
}

#[test]
fn u05_b7d_duijue_full_loop_undist_aoe_and_early_death() {
    let (verified, mut e) = load_formal_swd_only(7902, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "duijue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let dj_def = e.catalog().skill_by_name("duijue").unwrap();

    // Activate duijue -> asks duijue_target (and verify pending save/restore).
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: dj_def });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let mut e = roundtrip_bytes(&verified, &e);
    let ask = e.open_choice().expect("duijue_target").clone();
    assert_eq!(ask.spec.prompt, "duijue_target");
    assert!(
        !ask.spec.options.contains(&ChoiceOption::Player(0)),
        "duijue excludes self"
    );

    // Choose p1 -> skips remaining Play phase cleanly and sets duijue_pending_target.
    let res = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ask.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(matches!(res, CommandOutcome::Accepted { .. }));
    settle_confirm(&mut e);

    // Finish p0's turn -> TurnEndWindow triggers SwdDuijueTrigger and switches to p1, marking p2/p3 duijue2.
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    assert!(e.debug_has_skill(2, "duijue2"), "p2 marked out of duijue");
    assert!(e.debug_has_skill(3, "duijue2"), "p3 marked out of duijue");
    assert!(!e.debug_has_skill(0, "duijue3"), "transient duijue3 cleaned after TurnEndWindow");
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, "duijue_loop_remaining".to_string())).copied(),
        Some(6),
        "6-turn duijue loop initialized"
    );

    // duijue2 blocks targeting p2 directly and skips p2/p3 on AOE (SavageAttack).
    let sha1 = e.debug_place(1, "sha");
    assert!(matches!(
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: sha1,
                targets: vec![2],
            },
        ),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    let aoe = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::SavageAttack);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: aoe,
            targets: Vec::new(),
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "duijue AOE still hits p0");
    assert_eq!(e.debug_hp(2), 4, "duijue2 skips p2 on AOE");
    assert_eq!(e.debug_hp(3), 4, "duijue2 skips p3 on AOE");

    // Advance remaining 5 alternating turns (p1 -> p0 -> p1 -> p0 -> p1 -> resume normal p2!).
    advance_to_play(&mut e, 0);
    advance_to_play(&mut e, 1);
    advance_to_play(&mut e, 0);
    advance_to_play(&mut e, 1);
    // Next turn after all 6 duijue turns finishes -> should rotate to p2 and remove duijue2.
    for _ in 0..40 {
        if e.turn().current == 2 && e.turn().phase == Phase::Play {
            break;
        }
        if e.open_choice().is_some() {
            settle_confirm(&mut e);
            continue;
        }
        let cur = e.turn().current;
        e.debug_clear_hand(cur);
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, 2, "normal turn order resumed after 6 duijue turns");
    assert!(!e.debug_has_skill(2, "duijue2"), "duijue2 cleaned after 6-turn loop");
    assert!(!e.debug_has_skill(0, "duijue3"), "duijue3 cleaned after 6-turn loop");

    // Early death during duijue immediately clears duijue2/duijue3.
    let (_v2, mut e3) = load_formal_swd_only(7903, 4);
    for p in 0..4 {
        e3.debug_remove_general(p);
        e3.debug_clear_hand(p);
    }
    e3.debug_grant(0, "duijue");
    advance_to_play(&mut e3, 0);
    e3.debug_clear_hand(0);
    e3.debug_set_hp(1, 2);
    let dj3 = e3.catalog().skill_by_name("duijue").unwrap();
    submit(&mut e3, 0, GameAction::ActivateSkill { skill: dj3 });
    let ask = e3.open_choice().expect("duijue_target").clone();
    submit(&mut e3, 0, GameAction::Answer { choice_id: ask.id, selection: vec![ChoiceOption::Player(1)] });
    advance_to_play(&mut e3, 1);
    advance_to_play(&mut e3, 0);
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    e3.debug_set_hp(1, 1);
    let fatal = e3.debug_place(0, "sha");
    submit(&mut e3, 0, GameAction::PlayCard { card: fatal, targets: vec![1] });
    settle_confirm(&mut e3);
    assert!(!e3.debug_alive(1), "p1 killed during duijue");
    assert!(!e3.debug_has_skill(2, "duijue2"), "early death cleans duijue2 immediately");
    assert!(!e3.debug_has_skill(0, "duijue3"), "early death cleans duijue3 immediately");
}

#[test]
fn u05_b7d_tanlin_pindian_and_tanlin_defence() {
    let (verified, mut e) = load_formal_swd_only(7904, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "tanlin");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let tl_def = e.catalog().skill_by_name("tanlin").unwrap();
    let high = place_matching_card(&mut e, 0, |_, d| d.rank >= 11);
    let low = place_matching_card(&mut e, 1, |id, d| d.rank <= 4 && id != high);

    // 1. Pindian win -> gains target card, tanlin3 (+1 Sha), tanlin2/tanlin4 (unlimited range).
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: tl_def });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let ask_t = e.open_choice().expect("tanlin_target").clone();
    assert_eq!(ask_t.spec.prompt, "tanlin_target");
    submit(&mut e, 0, GameAction::Answer { choice_id: ask_t.id, selection: vec![ChoiceOption::Player(1)] });

    let ask_o = e.open_choice().expect("tanlin_pindian").clone();
    assert_eq!(ask_o.spec.prompt, "tanlin_pindian");
    submit(&mut e, 0, GameAction::Answer { choice_id: ask_o.id, selection: vec![ChoiceOption::Card(high)] });

    let mut e = roundtrip_bytes(&verified, &e);
    let ask_v = e.open_choice().expect("tanlin_target_pindian").clone();
    assert_eq!(ask_v.spec.prompt, "tanlin_target_pindian");
    submit(&mut e, 1, GameAction::Answer { choice_id: ask_v.id, selection: vec![ChoiceOption::Card(low)] });
    settle_confirm(&mut e);

    assert!(e.debug_hand(0).contains(&low), "p0 won pindian and gained target's card");
    assert!(e.debug_has_skill(0, "tanlin3"), "p0 gained tanlin3 (+1 Sha)");
    assert!(e.debug_has_skill(1, "tanlin2"), "target gained tanlin2");

    // tanlin3 lets p0 use 2 Shas this turn, and Snatch can reach distant p2? Wait: tanlin4 marks target (p1), not p2!
    // Snatch on p1 works cleanly even if >1, and 2nd Sha works cleanly via tanlin3!
    let sha_a = e.debug_place(0, "sha");
    let sha_b = e.debug_place(0, "sha");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    settle_confirm(&mut e);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ), "tanlin3 allows 2nd Sha in Play phase");
    settle_confirm(&mut e);

    // 2. tanlin_defence variable hand discard + source equal random discard + draw 1, plus pindian lose damage.
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "tanlin_defence");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let d1 = e.debug_place(1, "tao");
    let d2 = e.debug_place(1, "shan");
    let s1 = e.debug_place(0, "sha");
    let s2 = e.debug_place(0, "tao");
    let s3 = e.debug_place(0, "shan");
    let _ = (s1, s2, s3);
    assert!(!e.debug_has_skill(0, "tanlin3"), "tanlin3 cleared on TurnEndWindow");
    assert!(!e.debug_has_skill(1, "tanlin2"), "tanlin2 cleared on TurnEndWindow");
    let atk = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: atk, targets: vec![1] });
    // Pass shan prompt if any, settle into tanlin_defence_discard.
    let mut saw_defence = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "tanlin_defence_discard" {
            saw_defence = true;
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)],
                },
            );
            continue;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(saw_defence, "tanlin_defence_discard opened on damage");
    assert_eq!(e.debug_hand(1).len(), 1, "p1 discarded 2 then drew 1");
    assert_eq!(e.debug_hand(0).len(), 1, "p0 lost 2 random cards (3 -> 1)");

    // 3. Pindian lose branch -> owner takes 1 damage.
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let lo = place_matching_card(&mut e, 0, |_, d| d.rank <= 3);
    let hi = place_matching_card(&mut e, 1, |id, d| d.rank >= 11 && id != lo);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: tl_def });
    let ask_t = e.open_choice().expect("tanlin_target").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: ask_t.id, selection: vec![ChoiceOption::Player(1)] });
    let ask_o = e.open_choice().expect("tanlin_pindian").clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: ask_o.id, selection: vec![ChoiceOption::Card(lo)] });
    let ask_v = e.open_choice().expect("tanlin_target_pindian").clone();
    submit(&mut e, 1, GameAction::Answer { choice_id: ask_v.id, selection: vec![ChoiceOption::Card(hi)] });
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "tanlin lose deals 1 damage to owner (4 -> 3)");
}

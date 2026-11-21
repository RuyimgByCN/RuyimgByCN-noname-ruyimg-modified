use super::common::*;

#[test]
fn u01_fix_11_dddmoyan() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:472-509
    let (mut e, d3, p9) = new_identity_clean_engine(91011, 4);
    let liuchong = find_general(&e, "ddd_liuchong");
    let sunliang = find_general(&e, "ddd_sunliang");
    e.assign_general(0, sunliang).unwrap(); // has `dddguiying` (`phaseZhunbeiBegin`) on seat 0
    e.assign_general(1, liuchong).unwrap(); // has `dddmoyan` (`roundStart`) on seat 1!
    let wu_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            idx != sunliang
                && e.catalog()
                    .general_meta(idx)
                    .is_some_and(|m| m.1 == engine_native::catalog::Faction::Wu)
        })
        .unwrap();
    e.assign_general(2, wu_g).unwrap();
    let c_wu = find_card_by(&e, |_, _| true);
    put_in_hand(&mut e, 2, c_wu);

    // Handoff into seat 0 round 2: `dddmoyan` (`roundStart`) MUST fire at `TurnBegin` BEFORE `dddguiying` (`phaseZhunbeiBegin`)!
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let my_confirm = e.open_choice().expect("dddmoyan optional trigger_confirm").clone();
    assert_eq!(
        (my_confirm.observer, my_confirm.spec.prompt),
        (1, "trigger_confirm"),
        "roundStart dddmoyan must open trigger_confirm BEFORE revealing cards or seat 0 Prepare"
    );
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 1, vec![ChoiceOption::Confirm]);
    let my = e.open_choice().expect("dddmoyan d3_moyan_target after trigger_confirm").clone();
    assert_eq!(
        (my.observer, my.spec.prompt),
        (1, "d3_moyan_target"),
        "roundStart dddmoyan must trigger at TurnBegin BEFORE seat 0 Prepare phase"
    );
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 1, vec![ChoiceOption::Player(1)]);
    let my_gain = e.open_choice().expect("d3_moyan_gain").clone();
    let pick0 = my_gain.spec.options[0];
    answer(&mut e, 1, vec![pick0]);

    // After roundStart resolves, seat 0's `phaseZhunbeiBegin` (`dddguiying`) runs!
    let gy = e.open_choice().expect("seat 0 dddguiying runs after roundStart dddmoyan").clone();
    assert_eq!((gy.observer, gy.spec.prompt), (2, "dddguiying_confirm"));
    answer(&mut e, 2, Vec::new());
    drain(&mut e);

    // Damage amplification (+1 when seat 1 hand <= 1):
    let hp1 = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 2);
}

#[test]
fn u01_fix_12_dddshilie() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:511-654
    let (mut e, d3, p9) = new_clean_engine(91012, 4);
    let luoxian = find_general(&e, "ddd_luoxian");
    let zhuge = find_general(&e, "诸葛亮");
    e.assign_general(0, luoxian).unwrap();
    e.assign_general(1, zhuge).unwrap(); // seat 1 Kongcheng (0 hand cards)
    let shilie = e.catalog().skill_by_name("dddshilie").unwrap();

    // seat 0 hp=4, seat 0 turn current hp=4 => threshold = 8!
    let r5 = find_card_by(&e, |_, d| d.rank == 5);
    let r3 = find_card_by(&e, |_, d| d.rank == 3);
    let r1 = find_card_by(&e, |id, d| id != r5 && id != r3 && d.rank == 1);
    put_in_hand(&mut e, 0, r5);
    put_in_hand(&mut e, 0, r3);
    put_in_hand(&mut e, 0, r1);
    e.debug_set_turn(0, Phase::Play, 1);

    // Defensive Shan test fork first (attacker seat 3 hp=4 + defender seat 0 hp=4 => threshold 8).
    {
        let mut def_e = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
        let sha3 = find_card_by(&def_e, |id, d| id != r5 && id != r3 && id != r1 && d.kind == CardKind::Sha);
        put_in_hand(&mut def_e, 3, sha3);
        def_e.debug_set_turn(3, Phase::Play, 1);
        let hp0 = def_e.debug_hp(0);
        let _ = submit(
            &mut def_e,
            3,
            GameAction::PlayCard {
                card: sha3,
                targets: vec![0],
            },
        );
        let resp = def_e.open_choice().expect("respond_shan dddshilie").clone();
        assert!(resp.spec.options.contains(&ChoiceOption::Skill(shilie)));
        answer(&mut def_e, 0, vec![ChoiceOption::Skill(shilie)]);
        let sc = def_e.open_choice().expect("dddshilie_cards").clone();
        assert_eq!(sc.spec.prompt, "dddshilie_cards");
        // Exact sum 5 + 3 == 8 -> draws 2 cards AND dodges Sha!
        let h0_before = def_e.state_core().player(0).hand.len();
        assert_mid_choice_archive_and_ai(
            &def_e,
            &d3,
            &p9,
            vec![ChoiceOption::Card(r5), ChoiceOption::Card(r3)],
        );
        answer(&mut def_e, 0, vec![ChoiceOption::Card(r5), ChoiceOption::Card(r3)]);
        drain(&mut def_e);
        assert_eq!(def_e.debug_hp(0), hp0);
        assert_eq!(def_e.state_core().player(0).hand.len(), h0_before + 2);
    }

    // Play Sha via `ActivateSkill(dddshilie)`:
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: shilie });
    let sc = e.open_choice().expect("dddshilie_cards").clone();
    // Invalid rank sum (5 + 1 = 6 < 8) must be Rejected!
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: sc.id,
                selection: vec![ChoiceOption::Card(r5), ChoiceOption::Card(r1)],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    // Exact sum 5 + 3 = 8 -> draws 2 new UNSHOWN cards, enforces Sha target legality (no Kongcheng seat 1, no dist 2 seat 2).
    answer(&mut e, 0, vec![ChoiceOption::Card(r5), ChoiceOption::Card(r3)]);
    let st = e.open_choice().expect("dddshilie_target").clone();
    assert!(!st.spec.options.contains(&ChoiceOption::Player(1)), "Kongcheng excluded");
    assert!(!st.spec.options.contains(&ChoiceOption::Player(2)), "dist 2 excluded");
    assert!(st.spec.options.contains(&ChoiceOption::Player(3)));
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    drain(&mut e);

    // Next turn: `r5` & `r3` are still marked shown (cannot be reused!), while newly drawn cards ARE unshown!
    e.debug_set_turn(0, Phase::Play, 2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: shilie });
    if let Some(sc2) = e.open_choice() {
        assert!(!sc2.spec.options.contains(&ChoiceOption::Card(r5)));
        assert!(!sc2.spec.options.contains(&ChoiceOption::Card(r3)));
        assert!(sc2.spec.options.contains(&ChoiceOption::Card(r1)));
    }
}

#[test]
fn u01_fix_13_dddqiaoduan() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:747-801
    let (mut e, d3, p9) = new_clean_engine(91013, 3);
    let caoshuang = find_general(&e, "ddd_caoshuang");
    let _ = caoshuang;
    let ddd_liuye = find_general(&e, "ddd_liuye");
    let _ = ddd_liuye;
    let guanzhu_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.0 == "ddd_xujing" || m.0 == "ddd_zhenji")
        })
        .unwrap_or(0);
    let _ = guanzhu_g;
    e.debug_grant(0, "dddqiaoduan");
    e.debug_set_turn(0, Phase::Play, 1);

    let chain = find_card_by(&e, |_, d| d.kind == CardKind::Chain);
    let h1 = find_card_by(&e, |id, _| id != chain);
    let eq1 = find_card_by(&e, |id, d| id != chain && id != h1 && d.kind.is_equip());
    let h2 = find_card_by(&e, |id, _| id != chain && id != h1 && id != eq1);
    put_in_hand(&mut e, 0, chain);
    put_in_equip(&mut e, 0, eq1); // equip does NOT satisfy `dddqiaoduan` hand-only check!
    let _ = e.debug_deal_damage(None, 2, 1, DamageNature::Normal);

    // Negative guard: 1 linked (target 2) when owner has 0 hand cards (only 1 equip) -> linked branch must NOT trigger!
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: chain,
            targets: vec![2],
        },
    );
    drain(&mut e);
    assert!(
        e.open_choice().is_none(),
        "0 hand cards (equip only) must not trigger dddqiaoduan linked branch"
    );

    // Unlink seat 2 -> triggers unlinked draw branch (`num == 0` -> wait at unlink `num = 1`? No, linked count BEFORE unlink or AFTER? Unlinked count AFTER unlink = remaining linked! Wait if seat 1 stays linked so num == 1!).
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().player_mut(1).linked = true; // now 2 linked: seats 1 and 2
    put_in_hand(&mut e, 0, h1);
    put_in_hand(&mut e, 0, h2);
    let chain2 = find_card_by(&e, |id, d| {
        id != chain && id != h1 && id != h2 && id != eq1 && d.kind == CardKind::Chain
    });
    put_in_hand(&mut e, 0, chain2);
    // Unlinking seat 2 leaves 1 linked (seat 1) -> `num == 1` -> `dddqiaoduan_draw_targets` `[1..=1]`!
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: chain2,
            targets: vec![2],
        },
    );
    let dt = e.open_choice().expect("dddqiaoduan_draw_targets").clone();
    assert_eq!(dt.spec.prompt, "dddqiaoduan_draw_targets");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.state_core().player(1).hand.len(), 1);

    // Turn 3: Link seat 2 (now 2 linked: seats 1 & 2); owner has 2 hand cards `[h1, h2]` + `eq1`!
    e.debug_set_turn(0, Phase::Play, 3);
    let chain3 = find_card_by(&e, |id, d| {
        id != chain && id != chain2 && id != h1 && id != h2 && id != eq1 && d.kind == CardKind::Chain
    });
    put_in_hand(&mut e, 0, chain3);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: chain3,
            targets: vec![2],
        },
    );
    let qc = e.open_choice().expect("dddqiaoduan_cards").clone();
    assert_eq!(qc.spec.prompt, "dddqiaoduan_cards");
    assert!(
        !qc.spec.options.contains(&ChoiceOption::Card(eq1)),
        "dddqiaoduan_cards must only include hand cards (`h`), not equips"
    );
    assert_eq!((qc.spec.min, qc.spec.max), (2, 2));
    answer(&mut e, 0, vec![ChoiceOption::Card(h1), ChoiceOption::Card(h2)]);
    let rt = e.open_choice().expect("dddqiaoduan_recover_target").clone();
    assert_eq!(rt.spec.prompt, "dddqiaoduan_recover_target");
    let hp2 = e.debug_hp(2);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hp(2), hp2 + 1);
    // Both h1 and h2 placed at bottom of deck (`deck[0..2]`).
    assert!(e.state_core().deck[0..2].contains(&h1) && e.state_core().deck[0..2].contains(&h2));
}

#[test]
fn u01_fix_14_ddddiedang() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:802-860
    let (mut e, d3, p9) = new_clean_engine(91014, 3);
    let sunliang = find_general(&e, "ddd_sunliang");
    e.assign_general(0, sunliang).unwrap();
    let diedang = e.catalog().skill_by_name("ddddiedang").unwrap();

    // Turn 1 Play phase 1: state 0 -> draws 3, forced discard 1 `he` (equip allowed!).
    let eq0 = find_card_by(&e, |_, d| d.kind.is_equip());
    put_in_equip(&mut e, 0, eq0);
    let h1 = find_card_by(&e, |id, _| id != eq0);
    put_in_hand(&mut e, 1, h1);
    let h2_init = find_card_by(&e, |id, _| id != eq0 && id != h1);
    put_in_hand(&mut e, 2, h2_init);
    e.debug_set_turn(0, Phase::Play, 1);

    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang });
    let c0 = e.open_choice().expect("ddddiedang_discard st=0").clone();
    assert_eq!(c0.spec.prompt, "ddddiedang_discard");
    assert!(c0.spec.options.contains(&ChoiceOption::Card(eq0)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(eq0)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(eq0)]);
    assert_eq!(e.state_core().player(0).hand.len(), 3); // max hand size -> toggled to 1!
    assert_eq!(flag(&e, 0, "ddddiedang_state"), 1);

    // Same Play phase rejected:
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang }),
        CommandOutcome::Rejected(_)
    ));

    // Second Phase::Play in the SAME turn (via Phase::Draw -> NextPhase): state 1 -> requires 3 `he`, pays 3 `he` first, draws 1!
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!((e.turn().phase, e.turn().round), (Phase::Play, 1));
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang });
    let c1 = e.open_choice().expect("ddddiedang_discard st=1 in 2nd Phase::Play").clone();
    assert_eq!((c1.spec.min, c1.spec.max, c1.spec.allow_cancel), (3, 3, true));
    let pay3: Vec<ChoiceOption> = c1.spec.options.iter().take(3).copied().collect();
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, pay3.clone());
    let hand_before_pay = e.state_core().player(0).hand.len();
    answer(&mut e, 0, pay3);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_pay - 2);
}

#[test]
fn u01_fix_15_dddanliu() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:861-967
    let (mut e, d3, p9) = new_clean_engine(91015, 3);
    let sunliang = find_general(&e, "ddd_sunliang");
    e.assign_general(0, sunliang).unwrap();

    let red0 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart | Suit::Diamond));
    let blk1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade | Suit::Club));
    let red1 = find_card_by(&e, |id, d| {
        id != red0 && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    put_in_hand(&mut e, 0, red0);
    put_in_hand(&mut e, 1, blk1);
    put_in_hand(&mut e, 2, red1);

    // Branch A: DIFFERENT colors (owner red0 vs target blk1) -> MUST finish with NO swap and NO bonus!
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(blk1)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(red0)]);
    assert!(
        e.open_choice().is_none(),
        "dddanliu color mismatch must NOT ask swap confirm"
    );
    assert_eq!(e.state_core().player(0).hand, vec![red0]);
    assert_eq!(e.state_core().player(1).hand, vec![blk1]);

    // Branch B: SAME color (red0 vs red1) -> asks `dddanliu_swap_confirm`, swaps + heals 1 if confirmed!
    let _ = e.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    let hp0 = e.debug_hp(0);
    e.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red1)]);
    answer(&mut e, 2, vec![ChoiceOption::Card(red0)]);
    let sw = e.open_choice().expect("dddanliu_swap_confirm").clone();
    assert_eq!(sw.spec.prompt, "dddanliu_swap_confirm");
    assert!(sw.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand, vec![red1]);
    assert_eq!(e.state_core().player(2).hand, vec![red0]);
    assert_eq!(e.debug_hp(0), hp0 + 1);
}

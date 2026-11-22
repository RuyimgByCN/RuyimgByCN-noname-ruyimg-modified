use super::common::*;

#[test]
fn u01_fix2_16_17_18_dddfusi_global_tuoji() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3149-3387
    let (mut e, _d3, _p9) = new_clean_engine(92022, 3);
    let xianglang = find_general(&e, "ddd_xianglang");
    e.assign_general(0, xianglang).unwrap();
    let fusi_g_def = e.catalog().skill_by_name("dddfusi_global").unwrap();
    for p in 0..3 {
        assert!(
            e.state_core().has_skill(p, fusi_g_def),
            "dddfusi_global must be auto-granted to seat {p} when ddd_xianglang is in play"
        );
    }

    let c_shown_wz = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let c_shown_shan = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let c_unshown1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let c_unshown2 = find_card_by(&e, |id, d| id != c_unshown1 && d.kind == CardKind::Sha);
    let p1_shan = find_card_by(&e, |id, d| id != c_shown_shan && d.kind == CardKind::Shan);
    let p1_tao = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    let c_shown_tao = find_card_by(&e, |id, d| id != p1_tao && d.kind == CardKind::Tao);

    // 1) `dddfusi`: ONLY shown card ignored from hand limit AND undiscardable in Phase::Discard!
    e.state_core_mut().player_mut(0).hp = 1;
    put_in_hand(&mut e, 0, c_shown_wz);
    put_in_hand(&mut e, 0, c_unshown1);
    put_in_hand(&mut e, 0, c_unshown2);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{c_shown_wz}")), 1);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let cd = e.open_choice().expect("discard_to_limit with dddfusi").clone();
    assert_eq!(cd.spec.prompt, "discard_to_limit");
    assert_eq!(cd.spec.min, 1, "2 unshown - 1 hp = 1 discard (shown card ignored)");
    assert!(!cd.spec.options.contains(&ChoiceOption::Card(c_shown_wz)), "shown card undiscardable in Discard phase");
    answer(&mut e, 0, vec![ChoiceOption::Card(c_unshown1)]);
    assert!(e.state_core().player(0).hand.contains(&c_shown_wz));

    // 2) `dddfusi_global` cardEnabled2 + borrow in Sha / Play / Dying / Nullify + 4-way consent + `dddtuoji`!
    let (mut e2, d3, p9) = new_clean_engine(92023, 3);
    e2.assign_general(0, xianglang).unwrap();
    for p in 0..3 {
        assert!(
            e2.state_core().has_skill(p, fusi_g_def),
            "dddfusi_global auto-granted on e2 seat {p}"
        );
    }
    let sha_p0 = find_card_by(&e2, |id, d| id != c_unshown1 && id != c_unshown2 && d.kind == CardKind::Sha);
    put_in_hand(&mut e2, 0, c_shown_shan);
    put_in_hand(&mut e2, 0, c_shown_wz);
    put_in_hand(&mut e2, 0, c_shown_tao);
    put_in_hand(&mut e2, 0, sha_p0);
    // Notice `sha_p0` is NOT shown yet -> P1's own hand `Shan` IS still allowed!
    e2.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{c_shown_shan}")), 1);
    e2.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{c_shown_wz}")), 1);
    e2.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{c_shown_tao}")), 1);
    // Give P1 an alternate dodge skill (`dddbingjian` with 1 hand card) so after P0 refuses borrow, `sha_shan` remains open without `dddfusi_global`!
    e2.debug_grant(1, "dddbingjian");
    put_in_hand(&mut e2, 1, p1_shan);
    e2.debug_set_turn(0, Phase::Play, 1);

    // P0 plays `sha_p0` -> now all remaining cards in P0's hand ARE shown (`[c_shown_shan, c_shown_wz, c_shown_tao]`)!
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_p0, targets: vec![1] });
    let cs = e2.open_choice().expect("sha_shan when P0 all-shown").clone();
    assert_eq!(cs.observer, 1);
    assert!(
        !cs.spec.options.contains(&ChoiceOption::Card(p1_shan)),
        "dddfusi_global cardEnabled2 must block P1's own hand card while current turn P0 is all-shown"
    );
    assert!(cs.spec.options.contains(&ChoiceOption::Skill(fusi_g_def)));

    // P1 chooses to borrow via `dddfusi_global` -> picks `c_shown_shan` -> P0 asked 4-choice `dddfusi_agree`!
    answer(&mut e2, 1, vec![ChoiceOption::Skill(fusi_g_def)]);
    let cp = e2.open_choice().expect("dddfusi_pick").clone();
    assert_eq!((cp.observer, cp.spec.prompt), (1, "dddfusi_pick"));
    answer(&mut e2, 1, vec![ChoiceOption::Card(c_shown_shan)]);
    let ca = e2.open_choice().expect("dddfusi_agree").clone();
    assert_eq!((ca.observer, ca.spec.prompt), (0, "dddfusi_agree"));
    assert_eq!(
        ca.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2), ChoiceOption::Tag(3)]
    );
    assert_mid_choice_archive_and_ai(&e2, &d3, &p9, vec![ChoiceOption::Tag(0)]);

    // Fork A: P0 disagrees (`Tag(1)`) -> card stays in P0's hand, P1 returns to `respond_shan` (and cannot loop infinitely on same window)!
    let mut e_dis = Engine::restore_with_extensions(&e2.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_dis, 0, vec![ChoiceOption::Tag(1)]);
    assert!(e_dis.state_core().player(0).hand.contains(&c_shown_shan));
    let cs_after = e_dis.open_choice().expect("respond_shan back after disagree").clone();
    assert_eq!(cs_after.spec.prompt, "respond_shan");
    assert!(!cs_after.spec.options.contains(&ChoiceOption::Skill(fusi_g_def)), "disagree sets _dddfusi_refused for this window");

    // Live B: P0 chooses Tag(2) ("同意且本回合内不再提示") -> P1 uses `c_shown_shan` AND `dddtuoji_confirm` triggers on P0 (all hand shown!) -> draws 3!
    let p0_h_before = e2.state_core().player(0).hand.len(); // 3 cards -> -1 borrowed = 2
    answer(&mut e2, 0, vec![ChoiceOption::Tag(2)]);
    let ctj = e2.open_choice().expect("dddtuoji_confirm").clone();
    assert_eq!((ctj.observer, ctj.spec.prompt), (0, "dddtuoji_confirm"));
    assert_mid_choice_archive_and_ai(&e2, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e2.state_core().player(0).hand.len(), p0_h_before - 1 + 3, "dddtuoji draws 3!");

    // Check DyingSave borrow (`dddfusi_allowed` skips asking P0!)
    e2.state_core_mut().player_mut(2).hp = 1;
    let _ = e2.debug_deal_damage(None, 2, 1, DamageNature::Normal);
    // Drain P0 if needed so responder = 1 gets `dying_save`:
    if e2.open_choice().is_some_and(|c| c.observer == 0) {
        answer(&mut e2, 0, vec![ChoiceOption::Pass]);
    }
    let cds = e2.open_choice().expect("dying_save for P1").clone();
    assert_eq!((cds.observer, cds.spec.prompt), (1, "dying_save"));
    assert!(cds.spec.options.contains(&ChoiceOption::Skill(fusi_g_def)));
    answer(&mut e2, 1, vec![ChoiceOption::Skill(fusi_g_def)]);
    let _cdp = e2.open_choice().expect("dddfusi_pick tao").clone();
    answer(&mut e2, 1, vec![ChoiceOption::Card(c_shown_tao)]);
    // Notice `dddfusi_allowed` was set on P1 in this turn -> skips `dddfusi_agree` and directly heals P2!
    assert_eq!(e2.state_core().player(2).hp, 1, "borrowed Tao saved P2 without re-asking due to dddfusi_allowed");
    drain(&mut e2);
}

#[test]
fn u01_fix2_16b_dddfusi_tuoji_scope_and_equip_delayed() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3194-3387
    let (mut e, d3, p9) = new_clean_engine(92116, 3);
    let xianglang = find_general(&e, "ddd_xianglang");
    e.assign_general(0, xianglang).unwrap();
    let fusi_g_def = e.catalog().skill_by_name("dddfusi_global").unwrap();
    for p in 0..3 {
        assert!(
            e.state_core().has_skill(p, fusi_g_def),
            "dddfusi_global must be auto-granted to seat {p} when ddd_xianglang is in play"
        );
    }

    let p0_shown_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let p0_shown_shan = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let p0_shown_eq = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let p0_shown_le = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let p0_own_card = find_card_by(&e, |id, d| id != p0_shown_sha && d.kind == CardKind::Tao);
    let duel = find_card_by(&e, |_, d| d.kind == CardKind::Duel);
    let archery = find_card_by(&e, |_, d| d.kind == CardKind::ArcheryAttack);

    // (a) `dddtuoji` must NOT trigger when P0 uses his OWN card leaving remaining hand empty/all-shown!
    e.state_core_mut().player_mut(0).hp = 2;
    put_in_hand(&mut e, 0, p0_shown_sha);
    put_in_hand(&mut e, 0, p0_own_card);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{p0_shown_sha}")), 1);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: p0_own_card, targets: Vec::new() });
    assert!(e.open_choice().is_none(), "dddtuoji must NOT trigger when P0 uses his own card");
    assert_eq!(e.state_core().player(0).hand, vec![p0_shown_sha]);

    // (b) `dddfusi_global` NOT offered on chooseToRespond (`Duel` / `ArcheryAttack`) even when P0 has shown Sha/Shan!
    let p2_sha = find_card_by(&e, |id, d| id != p0_shown_sha && d.kind == CardKind::Sha);
    put_in_hand(&mut e, 2, p2_sha);
    put_in_hand(&mut e, 0, p0_shown_shan);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{p0_shown_shan}")), 1);
    put_in_hand(&mut e, 1, duel);
    put_in_hand(&mut e, 1, archery);
    e.debug_set_turn(1, Phase::Play, 2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: duel, targets: vec![2] });
    let cd = e.open_choice().expect("duel_sha for P2").clone();
    assert_eq!((cd.observer, cd.spec.prompt), (2, "duel_sha"));
    assert!(cd.spec.options.contains(&ChoiceOption::Card(p2_sha)));
    assert!(!cd.spec.options.contains(&ChoiceOption::Skill(fusi_g_def)), "chooseToRespond Duel must NOT offer dddfusi_global");
    drain(&mut e);

    let p2_shan = find_card_by(&e, |id, d| id != p0_shown_shan && d.kind == CardKind::Shan);
    put_in_hand(&mut e, 2, p2_shan);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: archery, targets: Vec::new() });
    let ca = e.open_choice().expect("aoe_response for P2").clone();
    assert_eq!((ca.observer, ca.spec.prompt), (2, "aoe_response"));
    assert!(ca.spec.options.contains(&ChoiceOption::Card(p2_shan)));
    assert!(!ca.spec.options.contains(&ChoiceOption::Skill(fusi_g_def)), "chooseToRespond ArcheryAttack must NOT offer dddfusi_global");
    drain(&mut e);

    // (c) Borrowing Equip & Delayed trick in Phase::Play IS allowed and triggers `dddtuoji`!
    put_in_hand(&mut e, 0, p0_shown_eq);
    put_in_hand(&mut e, 0, p0_shown_le);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{p0_shown_eq}")), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("u01_shown_card:{p0_shown_le}")), 1);
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: fusi_g_def });
    let cp = e.open_choice().expect("dddfusi_pick play").clone();
    assert!(cp.spec.options.contains(&ChoiceOption::Card(p0_shown_eq)), "Equip must be borrowable in Play");
    assert!(cp.spec.options.contains(&ChoiceOption::Card(p0_shown_le)), "Delayed trick must be borrowable in Play");
    assert!(!cp.spec.options.contains(&ChoiceOption::Card(p0_shown_shan)), "Shan must NOT be borrowable in Play");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(p0_shown_eq)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(p0_shown_eq)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(2)]); // agree for rest of turn
    assert!(e.state_core().player(1).equips.contains(&p0_shown_eq), "borrowed Equip equipped on P1");
    let ctj = e.open_choice().expect("dddtuoji after borrowed Equip").clone();
    assert_eq!((ctj.observer, ctj.spec.prompt), (0, "dddtuoji_confirm"));
    answer(&mut e, 0, Vec::new()); // decline draw so all P0 cards stay shown

    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: fusi_g_def });
    let _cp2 = e.open_choice().expect("dddfusi_pick le").clone();
    answer(&mut e, 1, vec![ChoiceOption::Card(p0_shown_le)]);
    let ctgt = e.open_choice().expect("dddfusi_target for Le").clone();
    assert_eq!(ctgt.spec.prompt, "dddfusi_target");
    assert!(ctgt.spec.options.contains(&ChoiceOption::Player(2)));
    assert!(!ctgt.spec.options.contains(&ChoiceOption::Player(1)), "Le cannot target self");
    answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    assert!(e.state_core().player(2).judge.contains(&p0_shown_le), "borrowed Le placed in P2.judge");
    assert_eq!(e.open_choice().expect("dddtuoji after Le").spec.prompt, "dddtuoji_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
}

#[test]
fn u01_fix2_19_dddchashi() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3388-3506
    let (mut e, d3, p9) = new_clean_engine(92024, 3);
    let liuye = find_general(&e, "ddd_liuye");
    e.assign_general(0, liuye).unwrap();
    retain_only_skill(&mut e, 0, Some("dddchashi"));

    let heart_basic = find_card_by(&e, |_, d| d.kind.is_basic_kind() && d.suit == Suit::Heart);
    let spade_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let heart_wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong && d.suit == Suit::Heart);
    let club_trick = find_card_by(&e, |_, d| {
        !d.kind.is_basic_kind() && !d.kind.is_equip() && d.kind == CardKind::Dismantle && d.suit == Suit::Club
    });
    let spade_armor = find_card_by(&e, |_, d| d.kind == CardKind::Armor && d.suit == Suit::Spade);
    let diam_equip = find_card_by(&e, |_, d| d.kind.is_equip() && d.suit == Suit::Diamond);

    put_in_hand(&mut e, 0, heart_basic);
    put_in_hand(&mut e, 1, spade_sha);
    put_in_hand(&mut e, 1, club_trick);

    // Enter P1's Play phase -> `dddchashi_discard` opens!
    e.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let cc = e.open_choice().expect("dddchashi_discard").clone();
    assert_eq!((cc.observer, cc.spec.prompt), (0, "dddchashi_discard"));
    assert!(cc.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(heart_basic)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(heart_basic)]);

    // Fork A: P1 plays `club_trick` (diff suit Club != Heart, diff type2 Trick != Basic) -> NO draw, still watches next card!
    let mut e_miss = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let h0_m = e_miss.state_core().player(0).hand.len();
    put_in_hand(&mut e_miss, 2, spade_armor);
    let _ = submit(&mut e_miss, 1, GameAction::PlayCard { card: club_trick, targets: vec![2] });
    assert_eq!(e_miss.state_core().player(0).hand.len(), h0_m);

    // Live B (`type2` Basic vs Basic, different suit!): P1 plays `spade_sha` (Spade Basic matches Heart Basic by type2!)
    let h0_before = e.state_core().player(0).hand.len();
    let h1_before = e.state_core().player(1).hand.len();
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: spade_sha, targets: vec![2] });
    assert_eq!(e.state_core().player(0).hand.len(), h0_before + 1, "dddchashi owner draws 1 on Basic-vs-Basic match");
    assert_eq!(e.state_core().player(1).hand.len(), h1_before - 1 + 1, "dddchashi target draws 1 on match");
    // Consumed once! Playing another matching card in same Play phase must NOT trigger again:
    put_in_hand(&mut e, 1, heart_wuzhong);
    let h0_after = e.state_core().player(0).hand.len();
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: heart_wuzhong, targets: Vec::new() });
    assert_eq!(e.state_core().player(0).hand.len(), h0_after);

    // Fork C (`type2` Equip vs Equip): discard `spade_armor`, P1 equips `diam_equip` -> both draw 1!
    let (mut e_eq, _d3, _p9) = new_clean_engine(92025, 3);
    e_eq.assign_general(0, liuye).unwrap();
    retain_only_skill(&mut e_eq, 0, Some("dddchashi"));
    put_in_hand(&mut e_eq, 0, spade_armor);
    put_in_hand(&mut e_eq, 1, diam_equip);
    e_eq.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e_eq, 1, GameAction::NextPhase);
    answer(&mut e_eq, 0, vec![ChoiceOption::Card(spade_armor)]);
    let h0_eq = e_eq.state_core().player(0).hand.len();
    let h1_eq = e_eq.state_core().player(1).hand.len();
    let _ = submit(&mut e_eq, 1, GameAction::PlayCard { card: diam_equip, targets: Vec::new() });
    assert_eq!(e_eq.state_core().player(0).hand.len(), h0_eq + 1);
    assert_eq!(e_eq.state_core().player(1).hand.len(), h1_eq - 1 + 1);
}

#[test]
fn u01_fix2_19b_dddchashi_expiry_and_non_use_equip() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3414,3477,3498-3500
    let (mut e, _, _) = new_clean_engine(92119, 3);
    let liuye = find_general(&e, "ddd_liuye");
    let yujin = find_general(&e, "ddd_yujin");
    e.assign_general(0, liuye).unwrap();
    retain_only_skill(&mut e, 0, Some("dddchashi"));
    e.assign_general(1, yujin).unwrap();
    retain_only_skill(&mut e, 1, Some("dddzhengjun"));

    let spade_armor = find_card_by(&e, |_, d| d.kind == CardKind::Armor && d.suit == Suit::Spade);
    let dism_spade = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle && d.suit == Suit::Spade);
    let p1_shield = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let p2_weapon = find_card_by(&e, |id, d| id != p1_shield && d.kind == CardKind::Weapon);
    let p2_horse = find_card_by(&e, |_, d| d.kind == CardKind::HorsePlus);
    let p0_keep = find_card_by(&e, |_, d| d.kind == CardKind::Tao);

    // Arm `dddchashi` on P1's Play phase using `spade_armor` (Spade Equip); `p0_keep` keeps P0.hand=1 != P1.hand=0:
    put_in_hand(&mut e, 0, spade_armor);
    put_in_hand(&mut e, 0, p0_keep);
    put_in_equip(&mut e, 1, p1_shield);
    put_in_equip(&mut e, 2, p2_weapon);
    put_in_equip(&mut e, 2, p2_horse);
    e.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Card(spade_armor)]);
    assert_eq!(flag(&e, 0, "dddchashi_armed"), 1);

    // 1) Non-use equip move (`dddzhengjun` moves `p2_horse` -> P1.equips) triggers `EventData::Equipped` WITHOUT PlayCard use -> must NOT trigger `dddchashi`!
    let dism_non_spade = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle && d.suit != Suit::Spade);
    put_in_hand(&mut e, 1, dism_non_spade);
    let h0_pre_move = e.state_core().player(0).hand.len();
    let out = submit(&mut e, 1, GameAction::PlayCard { card: dism_non_spade, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "play dism got {out:?}");
    // P1 loses hand card to 0 == P1 -> clear `dddzhengjun_hs_choice` first if opened!
    if e.open_choice().is_some_and(|c| c.spec.prompt == "dddzhengjun_hs_choice") {
        answer(&mut e, 1, vec![ChoiceOption::Tag(0)]);
    }
    // Dismantle discards one of P2's 2 equips (picking if prompted, or auto-discards first equip), leaving P2 with 1 == P1's 1 equip!
    if e.open_choice().is_some_and(|c| c.spec.prompt != "dddzhengjun_es_confirm") {
        let opt = e.open_choice().unwrap().spec.options[0];
        answer(&mut e, 1, vec![opt]);
    }
    let cz = e.open_choice().expect("dddzhengjun_es_confirm").clone();
    assert_eq!((cz.observer, cz.spec.prompt), (1, "dddzhengjun_es_confirm"));
    answer(&mut e, 1, vec![ChoiceOption::Confirm]);
    let mv_opt = e.open_choice().expect("dddzhengjun_es_card").spec.options[0];
    let ChoiceOption::Card(moved_eq) = mv_opt else {
        panic!("expected Card option");
    };
    answer(&mut e, 1, vec![mv_opt]);
    answer(&mut e, 1, vec![ChoiceOption::Player(1)]);
    assert!(e.state_core().player(1).equips.contains(&moved_eq), "equip actually moved onto P1 via Equipped");
    assert_eq!(e.state_core().player(0).hand.len(), h0_pre_move, "non-use Equipped move must NOT trigger dddchashi");
    assert_eq!(flag(&e, 0, "dddchashi_armed"), 1);

    // 2) `phaseUseAfter`: leaving P1's Play phase clears `dddchashi_*` flags immediately before Discard!
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::Discard);
    assert_eq!(flag(&e, 0, "dddchashi_armed"), 0);
    assert_eq!(flag(&e, 0, "dddchashi_target"), 0);
    assert_eq!(flag(&e, 0, "dddchashi_suit"), 0);
    assert_eq!(flag(&e, 0, "dddchashi_kind"), 0);

    // Next turn P1 Play phase WITHOUT re-arming must NOT trigger on matching Spade!
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // P2 Prepare
    e.debug_set_turn(1, Phase::Play, 2);
    put_in_hand(&mut e, 1, dism_spade);
    let h0_next = e.state_core().player(0).hand.len();
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism_spade, targets: vec![2] });
    assert_eq!(e.state_core().player(0).hand.len(), h0_next);
}

#[test]
fn u01_fix2_20_dddmiaoxing() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3569-3726
    let (mut e, d3, p9) = new_match_engine(92026, 3);
    let zhenji = find_general(&e, "ddd_zhenji");
    assert_eq!(e.state_core().player(0).hand.len(), 4);
    e.assign_general(0, zhenji).unwrap();
    assert_eq!(
        e.state_core().player(0).hand.len(),
        12,
        "dddmiaoxing gameDrawBegin triples opening deal (4 -> 12)"
    );

    let initial_12 = e.state_core().player(0).hand.clone();
    // Advancing from round 1 initial Judge triggers `dddmiaoxing_out_group`!
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let cg = e.open_choice().expect("dddmiaoxing_out_group").clone();
    assert_eq!((cg.observer, cg.spec.prompt), (0, "dddmiaoxing_out_group"));
    assert!(!cg.spec.allow_cancel);
    assert_eq!(cg.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)]);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);

    assert_eq!(&e.state_core().player(0).hand[..4], &initial_12[4..8]);
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_1".to_string())).cloned(),
        Some(initial_12[0..4].to_vec())
    );
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_2".to_string())).cloned(),
        Some(initial_12[8..12].to_vec())
    );

    // DrawPhase drawn +2 -> hand == 6, p1 == 4, p2 == 4 -> `dddmiaoxing_balance` (`phaseDrawAfter`) forced (both mismatch)!
    assert_eq!(e.state_core().player(0).hand.len(), 6);
    let cp = e.open_choice().expect("dddmiaoxing_pile after draw").clone();
    assert_eq!((cp.observer, cp.spec.prompt), (0, "dddmiaoxing_pile"));
    assert!(!cp.spec.allow_cancel);
    assert_eq!((cp.spec.min, cp.spec.max), (1, 2));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_1".to_string())).map(Vec::len), Some(6));
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_2".to_string())).map(Vec::len), Some(6));

    // Empty-pile refill check (#148 Item 7): drain `dddmiaoxing_1` to 0 (`p2 == 6`), reduce hand to 3, then Draw (+2 -> hand=5):
    // BOTH `p1 (0 != 5)` AND `p2 (6 != 5)` mismatch -> Tag(0) refills `0 -> 5` cards, while Fork Tag(1) trims `6 -> 5`!
    let p1_cards = e.state_core_mut().vm_aux.piles.insert((0, "dddmiaoxing_1".to_string()), Vec::new()).unwrap();
    e.state_core_mut().discard.extend(p1_cards);
    // Drop 3 hand cards to discard so hand == 3
    for _ in 0..3 {
        let c = e.state_core_mut().player_mut(0).hand.pop().unwrap();
        e.state_core_mut().discard.push(c);
    }
    e.debug_set_turn(0, Phase::Judge, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let cp2 = e.open_choice().expect("dddmiaoxing_pile with p1 empty (0) and p2=6 vs hand=5").clone();
    assert!(!cp2.spec.allow_cancel);
    assert_eq!(cp2.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(0)]);

    // Fork A: pick Tag(1) only -> reduces `p2` from 6 to 5 while `p1` stays 0 (player opted not to balance p1)
    let mut e_trim = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_trim, 0, vec![ChoiceOption::Tag(1)]);
    let cd = e_trim.open_choice().expect("dddmiaoxing_discard").clone();
    assert_eq!((cd.observer, cd.spec.prompt), (0, "dddmiaoxing_discard"));
    assert_eq!((cd.spec.min, cd.spec.max), (1, 1));
    let drop = e_trim.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_2".to_string())).unwrap()[0];
    answer(&mut e_trim, 0, vec![ChoiceOption::Card(drop)]);
    assert_eq!(e_trim.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_2".to_string())).map(Vec::len), Some(5));

    // Live B: pick empty Tag(0) -> refills `dddmiaoxing_1` from 0 up to 5!
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_1".to_string())).map(Vec::len), Some(5), "0-card pile must refill to hand.len()");
}

#[test]
fn u01_fix2_21_dddfenji() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3866-3943
    let (mut e, d3, p9) = new_clean_engine(92027, 3);
    let zhaoang = find_general(&e, "ddd_zhaoang");
    e.assign_general(0, zhaoang).unwrap();

    let sha_p1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let eq_p0 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let h1 = find_card_by(&e, |id, d| id != sha_p1 && d.kind == CardKind::Shan);
    let h2 = find_card_by(&e, |id, d| id != sha_p1 && id != h1 && d.kind == CardKind::Tao);
    let h3 = find_card_by(&e, |id, d| id != sha_p1 && id != h1 && id != h2 && d.kind == CardKind::Sha);
    let eq_p1 = find_card_by(&e, |id, d| id != eq_p0 && d.kind == CardKind::Weapon);
    let h_p1 = find_card_by(&e, |id, d| {
        id != sha_p1 && id != h1 && id != h2 && id != h3 && d.kind == CardKind::Shan
    });
    put_in_equip(&mut e, 0, eq_p0);
    put_in_hand(&mut e, 0, h1);
    put_in_hand(&mut e, 0, h2);
    put_in_hand(&mut e, 0, h3);
    put_in_equip(&mut e, 1, eq_p1);
    put_in_hand(&mut e, 1, h_p1);

    // 1) DrawPhaseBegin1: ALL usable normal tricks offered (incl. Wuzhong, SavageAttack, ArcheryAttack, Taoyuan, Wugu, Chain, Duel, Dismantle, Snatch)!
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let cf = e.open_choice().expect("dddfenji_trick on Draw").clone();
    assert_eq!((cf.observer, cf.spec.prompt), (0, "dddfenji_trick"));
    let duel_tag = ChoiceOption::Tag(0); // Duel
    let wuzhong_tag = ChoiceOption::Tag(3); // Wuzhong
    let savage_tag = ChoiceOption::Tag(6); // SavageAttack
    assert!(cf.spec.options.contains(&duel_tag));
    assert!(cf.spec.options.contains(&wuzhong_tag));
    assert!(cf.spec.options.contains(&savage_tag));
    assert!(cf.spec.options.len() >= 10, "expected all usable normal tricks from deck/catalog, got {:?}", cf.spec.options);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![duel_tag]);

    // Fork A: choose Wuzhong -> draws 2 from Wuzhong, DrawPhase changed to 0 (`changeToZero`), draws 0 from damage!
    let mut e_wz = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let h_pre_wz = e_wz.state_core().player(0).hand.len();
    answer(&mut e_wz, 0, vec![wuzhong_tag]);
    assert_eq!(e_wz.state_core().player(0).hand.len(), h_pre_wz + 2, "Wuzhong draws 2, and DrawPhase changeToZero() draws 0");

    // Live B: choose Duel on P1 -> deals 1 damage -> DrawPhase changed to 0 (`changeToZero`), P0 draws 1 from damage!
    let h_pre_duel = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![duel_tag]);
    let ct = e.open_choice().expect("dddfenji_target").clone();
    assert_eq!(ct.spec.prompt, "dddfenji_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        h_pre_duel + 1,
        "DrawPhase changeToZero() + draws 1 for 1 sourceDamage from Duel!"
    );

    // 2) Phase::Discard begin: `duel_tag` permanently removed! Pick `SavageAttack` (2 targets [P1, P2]):
    //    Discard phase is CANCELLED, and forces discard `min(2, hand.len())` from HAND ONLY (equip untouched)!
    e.state_core_mut().player_mut(0).hp = 1; // normally hand (4) > hp (1)!
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let cf2 = e.open_choice().expect("dddfenji_trick on Discard").clone();
    assert!(!cf2.spec.options.contains(&duel_tag), "used Duel must remain excluded");
    answer(&mut e, 0, vec![savage_tag]);
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "dddfenji_discard" {
            break;
        }
        let sel = if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
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
    let cd = e.open_choice().expect("dddfenji_discard").clone();
    assert_eq!((cd.observer, cd.spec.prompt), (0, "dddfenji_discard"));
    assert_eq!((cd.spec.min, cd.spec.max), (2, 2));
    assert!(!cd.spec.allow_cancel);
    assert!(!cd.spec.options.contains(&ChoiceOption::Card(eq_p0)), "chooseToDiscard(num, true) defaults to hand ONLY");
    let drop_cards = vec![cd.spec.options[0], cd.spec.options[1]];
    answer(&mut e, 0, drop_cards);
    assert!(e.state_core().player(0).equips.contains(&eq_p0));
    assert!(e.open_choice().is_none(), "Discard phase was cancelled so NO discard_to_limit after!");
}

#[test]
fn u01_fix2_21b_dddfenji_discard_targets_and_draw_dmg() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3916-3940
    let (mut e, d3, p9) = new_clean_engine(92121, 3);
    let zhaoang = find_general(&e, "ddd_zhaoang");
    e.assign_general(0, zhaoang).unwrap();

    let h1 = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let h2 = find_card_by(&e, |id, d| id != h1 && d.kind == CardKind::Tao);
    let h3 = find_card_by(&e, |id, d| id != h1 && id != h2 && d.kind == CardKind::Shan);
    let p1_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, h1);
    put_in_hand(&mut e, 0, h2);
    put_in_hand(&mut e, 0, h3);

    // 1) phaseDraw + Duel backfire: P1 wins Duel (`P1 -> P0` 1 damage!) -> P0 draws 1 (`getParent(4)` sourceDamage)!
    put_in_hand(&mut e, 1, p1_sha);
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let h0_pre = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]); // Duel
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(p1_sha)]); // P1 plays Sha back, P0 has no Sha -> takes 1 dmg from P1!
    assert_eq!(e.state_core().player(0).hp, 3);
    assert_eq!(e.state_core().player(0).hand.len(), h0_pre + 1, "Duel backfire damage by P1 still draws 1 for P0");

    // 2) phaseDiscard + Wuzhong (0 damage, 1 target) -> MUST discard 1 (from drawn 2 = 6 -> 5)!
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Tag(3)]); // Wuzhong (0 dmg, 1 target [P0])
    let cd_wz = e.open_choice().expect("dddfenji_discard after Wuzhong").clone();
    assert_eq!((cd_wz.observer, cd_wz.spec.prompt), (0, "dddfenji_discard"));
    assert_eq!((cd_wz.spec.min, cd_wz.spec.max), (1, 1), "Wuzhong has 1 target -> discard 1 despite 0 damage");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![cd_wz.spec.options[0]]);
    let h_pre_drop = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![cd_wz.spec.options[0]]);
    assert_eq!(e.state_core().player(0).hand.len(), h_pre_drop - 1);

    // 3) phaseDiscard + defended SavageAttack (2 targets [P1, P2], only 1 dmg dealt because P1 plays Sha) -> MUST discard 2 (`targets.len()`)!
    let (mut e2, _, _) = new_clean_engine(92122, 3);
    e2.assign_general(0, zhaoang).unwrap();
    put_in_hand(&mut e2, 0, h1);
    put_in_hand(&mut e2, 0, h2);
    put_in_hand(&mut e2, 0, h3);
    put_in_hand(&mut e2, 1, p1_sha);
    e2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    answer(&mut e2, 0, vec![ChoiceOption::Tag(6)]); // SavageAttack
    let a1 = e2.open_choice().expect("aoe_response for P1").clone();
    assert_eq!(a1.observer, 1);
    answer(&mut e2, 1, vec![ChoiceOption::Card(p1_sha)]); // P1 blocks damage -> total dmg = 1, targets = 2
    for _ in 0..8 {
        let Some(c) = e2.open_choice().cloned() else { break; };
        if c.spec.prompt == "dddfenji_discard" {
            break;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(&mut e2, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    let cd_sa = e2.open_choice().expect("dddfenji_discard after defended SavageAttack").clone();
    assert_eq!((cd_sa.observer, cd_sa.spec.prompt), (0, "dddfenji_discard"));
    assert_eq!((cd_sa.spec.min, cd_sa.spec.max), (2, 2), "SavageAttack had 2 targets even with only 1 damage -> must discard 2");
    answer(&mut e2, 0, vec![cd_sa.spec.options[0], cd_sa.spec.options[1]]);

    // 4) Jiedao in phaseDraw -> inner Sha damage has getParent(4) != dddfenji (draws 0); in phaseDiscard -> evt.targets.length == 1 (discards 1)!
    let (mut e3, _, _) = new_clean_engine(92123, 3);
    e3.assign_general(0, zhaoang).unwrap();
    let p1_weapon = find_card_by(&e3, |_, d| d.kind == CardKind::Weapon);
    put_in_equip(&mut e3, 1, p1_weapon);
    put_in_hand(&mut e3, 1, p1_sha);
    put_in_hand(&mut e3, 0, h1);
    put_in_hand(&mut e3, 0, h2);
    e3.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e3, 0, GameAction::NextPhase);
    let h0_pre_jd = e3.state_core().player(0).hand.len();
    answer(&mut e3, 0, vec![ChoiceOption::Tag(9)]); // Jiedao
    answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
    answer(&mut e3, 1, vec![ChoiceOption::Tag(0)]); // jiedao_sha_or_give -> play Sha
    if e3.open_choice().is_some_and(|c| c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(p1_sha))) {
        answer(&mut e3, 1, vec![ChoiceOption::Card(p1_sha)]);
    }
    assert_eq!(e3.state_core().player(2).hp, 3, "Jiedao Sha damaged P2");
    assert_eq!(e3.state_core().player(0).hand.len(), h0_pre_jd, "Jiedao Sha damage has getParent(4) != dddfenji so draws 0");

    // 5) Jiedao in phaseDiscard -> upstream evt.targets.length == 1 ([holder]), so discards 1 (NOT 2)!
    let (mut e4, _, _) = new_clean_engine(92124, 3);
    e4.assign_general(0, zhaoang).unwrap();
    put_in_equip(&mut e4, 1, p1_weapon);
    put_in_hand(&mut e4, 0, h1);
    put_in_hand(&mut e4, 0, h2);
    put_in_hand(&mut e4, 0, h3);
    e4.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e4, 0, GameAction::NextPhase);
    answer(&mut e4, 0, vec![ChoiceOption::Tag(9)]); // Jiedao
    answer(&mut e4, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e4, 0, vec![ChoiceOption::Player(2)]);
    answer(&mut e4, 1, vec![ChoiceOption::Tag(1)]); // P1 gives weapon instead of Sha
    let cd_jd = e4.open_choice().expect("dddfenji_discard after Jiedao").clone();
    assert_eq!((cd_jd.observer, cd_jd.spec.prompt), (0, "dddfenji_discard"));
    assert_eq!((cd_jd.spec.min, cd_jd.spec.max), (1, 1), "Jiedao targets.length is 1 ([holder]), NOT 2");
}

#[test]
fn u01_fix2_16c_dddfusi_global_auto_grant_sync() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3185-3210
    let (mut e, _, _) = new_clean_engine(92117, 3);
    let fusi_g_def = e.catalog().skill_by_name("dddfusi_global").unwrap();
    let xianglang = find_general(&e, "ddd_xianglang");
    let non_xianglang = find_general(&e, "ddd_wuzhi");

    // 1) Before any dddfusi exists: nobody has `dddfusi_global`.
    e.assign_general(0, non_xianglang).unwrap();
    for p in 0..3 {
        assert!(!e.state_core().has_skill(p, fusi_g_def), "no dddfusi_global without dddfusi");
    }

    // 2) Assigning `ddd_xianglang` auto-grants `dddfusi_global` to ALL seats immediately!
    e.assign_general(0, xianglang).unwrap();
    for p in 0..3 {
        assert!(e.state_core().has_skill(p, fusi_g_def), "dddfusi_global auto-granted on assign_general");
    }

    // 3) Assigning another seat keeps `dddfusi_global`, while swapping P0 off removes it!
    e.assign_general(1, non_xianglang).unwrap();
    for p in 0..3 {
        assert!(e.state_core().has_skill(p, fusi_g_def), "dddfusi_global preserved while seat 0 holds dddfusi");
    }
    e.assign_general(0, non_xianglang).unwrap();
    for p in 0..3 {
        assert!(!e.state_core().has_skill(p, fusi_g_def), "dddfusi_global cleared when last dddfusi leaves");
    }

    // 4) `debug_grant(1, "dddfusi")` also auto-grants `dddfusi_global` across all seats.
    e.debug_grant(1, "dddfusi");
    for p in 0..3 {
        assert!(e.state_core().has_skill(p, fusi_g_def), "debug_grant(dddfusi) syncs dddfusi_global");
    }

    // 5) `debug_remove_general` on `ddd_xianglang` clears `dddfusi_global` when no `dddfusi` remains.
    let (mut e_rm, _, _) = new_clean_engine(92118, 3);
    e_rm.assign_general(0, xianglang).unwrap();
    assert!(e_rm.state_core().has_skill(2, fusi_g_def));
    e_rm.debug_remove_general(0);
    for p in 0..3 {
        assert!(!e_rm.state_core().has_skill(p, fusi_g_def), "debug_remove_general clears dddfusi_global");
    }
}

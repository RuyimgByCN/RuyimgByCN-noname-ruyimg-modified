use super::*;
use engine_native::ai::BoundedPlanner;
use engine_native::machine::LogEvent;
use engine_native::state::KingdomIdentity;
use engine_native::RejectReason;

fn drain_nullify_local(e: &mut Engine) {
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt != "nullify_window" {
            break;
        }
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

fn find_general_id(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} missing"))
}

#[test]
fn u05_b1_r3_fix01_lingyong_subplayer_lifecycle() {
    let subplayer_id_absent_before_fix = |e: &Engine, name: &str| -> bool {
        e.catalog().skill_by_name(name).is_some()
    };

    // Part A: single captured subplayer auto-call + self-name exclusion + SubPlayer dieBefore permanent exit!
    let (verified, mut e) = start_u05_b1_engine(
        93001,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    assert!(subplayer_id_absent_before_fix(&e, "subplayer"), "subplayer skill must exist in shahai-mtg");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let gid_lili = find_general_id(&e, "mtg_lilianna");
    let gid_jiding = find_general_id(&e, "mtg_jiding");
    e.debug_assemble_general_def(0, gid_lili);
    e.debug_assemble_general_def(1, gid_lili);
    e.debug_assemble_general_def(2, gid_jiding);
    e.debug_set_hp(1, 1);
    e.debug_set_hp(2, 1);

    // Put seat 0 armor + hand Tao; kill seat 1 (same general name `mtg_lilianna`) via real Sha from seat 3 (with range>=2 weapon so seat 2 mtg_jiding gets 0 hujia!)
    advance_to_phase(&mut e, 3, Phase::Play);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let weap3 = find_card(&e, |c| c.kind == CardKind::Weapon && c.range >= 2);
    let sha3 = find_card(&e, |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 3, weap3);
    assert!(!matches!(
        submit(&mut e, 3, GameAction::PlayCard { card: weap3, targets: vec![] }),
        CommandOutcome::Rejected(_)
    ));
    drain(&mut e);
    move_card_to_hand(&mut e, 3, sha3);
    assert!(!matches!(
        submit(&mut e, 3, GameAction::PlayCard { card: sha3, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));
    drain(&mut e);
    assert!(!e.state_core().player(1).alive, "seat 1 dead");

    // Seat 0 Play: same-name `mtg_lilianna` must NOT be captured!
    advance_to_phase(&mut e, 0, Phase::Play);
    assert!(
        matches!(activate(&mut e, 0, "lingyong"), CommandOutcome::Rejected(_)),
        "lingyong must NOT capture fallen general sharing owner's name (`mtg_lilianna`)"
    );

    // Equip armor + keep Tao on seat 0, then seat 0 uses real Sha to kill adjacent seat 2 (`mtg_jiding`)!
    let armor0 = find_card(&e, |c| c.kind == CardKind::Armor);
    let tao0 = find_card(&e, |c| c.kind == CardKind::Tao);
    let sha0 = find_card_excl(&e, &[sha3], |c| c.kind == CardKind::Sha);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    move_card_to_hand(&mut e, 0, armor0);
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: armor0, targets: vec![] }),
        CommandOutcome::Rejected(_)
    ));
    drain(&mut e);
    move_card_to_hand(&mut e, 0, tao0);
    move_card_to_hand(&mut e, 0, sha0);
    let out_sha0 = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![2] });
    assert!(
        !matches!(out_sha0, CommandOutcome::Rejected(_)),
        "play sha0 vs seat 2 rejected: {out_sha0:?}, alive={:?}, seat_ring={:?}, hand0={:?}, dist={}",
        [e.state_core().player(0).alive, e.state_core().player(1).alive, e.state_core().player(2).alive, e.state_core().player(3).alive],
        e.state_core().seat_ring,
        e.debug_hand(0),
        e.state_core().computed_distance(e.catalog(), 0, 2)
    );
    drain(&mut e);
    assert!(!e.state_core().player(2).alive, "seat 2 mtg_jiding dead (hp={})", e.debug_hp(2));

    let tracked = total_tracked_cards(&e);
    let hp0_before = e.debug_hp(0);
    let max0_before = e.state_core().player(0).max_hp;
    assert_eq!(e.debug_hand(0), vec![tao0]);
    assert_eq!(e.debug_equips(0), vec![armor0]);

    // Summon sole captured subplayer `mtg_jiding`: auto-enters immediately (NO choice prompt)!
    let out = activate(&mut e, 0, "lingyong");
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "lingyong activate: {out:?}");
    assert!(e.open_choice().is_none(), "single subplayer callSubPlayer enters directly without button prompt");
    assert_eq!((e.debug_hp(0), e.state_core().player(0).max_hp, e.state_core().player(0).hujia), (1, 1, 0));
    let sub_def = e.catalog().skill_by_name("subplayer").unwrap();
    let ling_def = e.catalog().skill_by_name("lingyong").unwrap();
    let duo_def = e.catalog().skill_by_name("mduohun").unwrap();
    let bz_def = e.catalog().skill_by_name("mbaizhan").unwrap();
    let sl_def = e.catalog().skill_by_name("msilian").unwrap();
    assert!(e.state_core().has_skill(0, sub_def), "inSub has `subplayer`");
    assert!(!e.state_core().has_skill(0, ling_def), "inSub hides `lingyong`");
    assert!(!e.state_core().has_skill(0, duo_def), "inSub hides `mduohun`");
    assert!(e.state_core().has_skill(0, bz_def), "inSub grants `mbaizhan`");
    assert!(e.state_core().has_skill(0, sl_def), "inSub grants `msilian`");
    assert_eq!(e.debug_hand(0).len(), 2, "SubPlayer draws hs: 2 cards");
    assert!(!e.debug_hand(0).contains(&tao0) && e.debug_equips(0).is_empty(), "main hs/es stashed off-field");
    assert_eq!(total_tracked_cards(&e), tracked, "cards conserved upon entering SubPlayer");

    let mut e = snapshot_roundtrip(&e, &verified);

    // Play a real Duel from SubPlayer seat 0 against seat 3 (who has a Sha) to kill the 1 HP SubPlayer!
    e.debug_clear_hand(0); // ensure no Tao/Sha so Duel hurts seat 0 cleanly
    let duel = find_card(&e, |c| c.kind == CardKind::Duel);
    let sub_eq = find_card_excl(&e, &[armor0, weap3], |c| c.kind == CardKind::Weapon);
    let sha3_b = find_card_excl(&e, &[sha3, sha0], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, sub_eq);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sub_eq, targets: vec![] });
    drain(&mut e);
    move_card_to_hand(&mut e, 0, duel);
    move_card_to_hand(&mut e, 3, sha3_b);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![3] });
    drain_nullify_local(&mut e);
    // Seat 3 answers Sha first, then seat 0 has no Sha -> loses Duel -> 1 dmg -> dying -> dieBefore `subplayer`!
    while let Some(c) = e.open_choice().cloned() {
        let sel = if c.observer == 3 && c.spec.options.contains(&ChoiceOption::Card(sha3_b)) {
            vec![ChoiceOption::Card(sha3_b)]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            Vec::new()
        };
        let _ = submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    drain(&mut e);

    assert!(e.state_core().player(0).alive, "SubPlayer exit prevents permanent death");
    assert!(!e.state_core().player(0).dying, "dying cleared on exitSubPlayer");
    assert_eq!((e.debug_hp(0), e.state_core().player(0).max_hp), (hp0_before, max0_before));
    assert!(!e.state_core().has_skill(0, sub_def), "afterDeath removes `subplayer`");
    assert!(e.state_core().has_skill(0, ling_def), "afterDeath restores `lingyong`");
    assert!(e.state_core().has_skill(0, duo_def), "afterDeath restores `mduohun`");
    assert!(!e.state_core().has_skill(0, bz_def), "afterDeath clears sub skill `mbaizhan`");
    assert!(!e.state_core().has_skill(0, sl_def), "afterDeath clears sub skill `msilian`");
    assert_eq!(e.debug_hand(0), vec![tao0], "afterDeath restores main hand");
    assert_eq!(e.debug_equips(0), vec![armor0], "afterDeath restores main equips & discards sub_eq");
    assert!(e.state_core().discard.contains(&sub_eq), "subplayer equip moved to discard");
    assert_eq!(total_tracked_cards(&e), tracked, "cards conserved on exitSubPlayer");
    assert!(
        matches!(activate(&mut e, 0, "lingyong"), CommandOutcome::Rejected(_)),
        "destroyed SubPlayer is permanently removed (0 remaining)"
    );

    // Part B: 2 captured subplayers -> forced choice + `nosub` skill stripping + NO `once_per_turn`!
    let (verified2, mut e2) = start_u05_b1_engine(
        93002,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    // Owner seat 0 = `mtg_jiesi` granted `lingyong`, so both `mtg_jiding` & `mtg_lilianna` can be captured!
    let gid2_jiesi = find_general_id(&e2, "mtg_jiesi");
    e2.debug_assemble_general_def(0, gid2_jiesi);
    e2.debug_grant(0, "lingyong");
    let gid2_jd = find_general_id(&e2, "mtg_jiding");
    let gid2_ll = find_general_id(&e2, "mtg_lilianna");
    e2.debug_assemble_general_def(1, gid2_jd);
    e2.debug_assemble_general_def(3, gid2_ll);
    e2.debug_set_hp(1, 1);
    e2.debug_set_hp(3, 1);
    e2.debug_grant(0, "咆哮");
    advance_to_phase(&mut e2, 0, Phase::Play);
    e2.debug_clear_hand(0);
    let sha_a = find_card(&e2, |c| c.kind == CardKind::Sha);
    let sha_b = find_card_excl(&e2, &[sha_a], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e2, 0, sha_a);
    move_card_to_hand(&mut e2, 0, sha_b);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    drain(&mut e2);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_b, targets: vec![3] });
    drain(&mut e2);

    // Activate lingyong with 2 subplayers: MUST prompt forced `u05_lingyong_summon`!
    assert!(!matches!(activate(&mut e2, 0, "lingyong"), CommandOutcome::Rejected(_)));
    let c_sub = e2.open_choice().expect("2 subplayers must ask u05_lingyong_summon").clone();
    assert_eq!((c_sub.observer, c_sub.spec.prompt, c_sub.spec.min, c_sub.spec.max, c_sub.spec.allow_cancel), (0, "u05_lingyong_summon", 1, 1, false));
    assert_eq!(c_sub.spec.options.len(), 2);
    let planner = BoundedPlanner::new(16);
    assert!(matches!(e2.plan_action(0, &planner).action, GameAction::Answer { .. }), "public-info AI handles u05_lingyong_summon");
    let mut e2 = snapshot_roundtrip(&e2, &verified2);
    // Pick Tag(1) (`mtg_lilianna`): must strip `lingyong` (`nosub: true`) and grant `mduohun`!
    answer(&mut e2, "u05_lingyong_summon", vec![ChoiceOption::Tag(1)]);
    drain(&mut e2);
    let ling2 = e2.catalog().skill_by_name("lingyong").unwrap();
    let duo2 = e2.catalog().skill_by_name("mduohun").unwrap();
    assert!(!e2.state_core().has_skill(0, ling2), "nosub skill `lingyong` stripped from lilianna SubPlayer");
    assert!(e2.state_core().has_skill(0, duo2), "`mduohun` kept on lilianna SubPlayer");

    // Kill lilianna SubPlayer in SAME Play phase via Duel against seat 2!
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(2);
    let duel2 = find_card(&e2, |c| c.kind == CardKind::Duel);
    let sha_d = find_card_excl(&e2, &[sha_a, sha_b], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e2, 0, duel2);
    move_card_to_hand(&mut e2, 2, sha_d);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: duel2, targets: vec![2] });
    drain_nullify_local(&mut e2);
    while let Some(c) = e2.open_choice().cloned() {
        let sel = if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(sha_d)) {
            vec![ChoiceOption::Card(sha_d)]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(&mut e2, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    drain(&mut e2);
    assert!(e2.state_core().has_skill(0, ling2), "main lingyong restored");

    // In the VERY SAME TURN (`no once_per_turn`!), summon the remaining `mtg_jiding` subplayer!
    let out2 = activate(&mut e2, 0, "lingyong");
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "lingyong has NO once_per_turn: {out2:?}");
    let bz2 = e2.catalog().skill_by_name("mbaizhan").unwrap();
    assert!(e2.state_core().has_skill(0, bz2), "remaining subplayer `mtg_jiding` summoned in same turn");
}

#[test]
fn u05_b1_r3_fix02_shezhanqunru_tie_and_single_card() {
    let (verified, mut e) = start_u05_b1_engine(
        93003,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let sz = find_card(&e, |c| c.kind == CardKind::Shezhanqunru);
    let p0_c5 = find_card(&e, |c| c.rank == 5 && c.kind != CardKind::Shezhanqunru);
    let p0_keep = find_card_excl(&e, &[sz, p0_c5], |c| c.rank == 10);
    let p1_c3 = find_card(&e, |c| c.rank == 3);
    let p2_c5 = find_card_excl(&e, &[p0_c5], |c| c.rank == 5);
    move_card_to_hand(&mut e, 0, sz);
    move_card_to_hand(&mut e, 0, p0_c5);
    move_card_to_hand(&mut e, 0, p0_keep);
    move_card_to_hand(&mut e, 1, p1_c3);
    move_card_to_hand(&mut e, 2, p2_c5);
    let tracked = total_tracked_cards(&e);

    let out = submit(&mut e, 0, GameAction::PlayCard { card: sz, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_nullify_local(&mut e);

    answer(&mut e, "u05_shezhan_respond", vec![ChoiceOption::Confirm]); // seat 1
    answer(&mut e, "u05_shezhan_respond", vec![ChoiceOption::Confirm]); // seat 2

    // Only ONE `u05_shezhan_own_pick` prompt!
    let c_own = e.open_choice().expect("single u05_shezhan_own_pick").clone();
    assert_eq!((c_own.observer, c_own.spec.prompt), (0, "u05_shezhan_own_pick"));
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "u05_shezhan_own_pick", vec![ChoiceOption::Card(p0_c5)]);

    // Seat 1 responder pick (3 vs 5 -> +1 win)
    let c_t1 = e.open_choice().expect("seat 1 pick").clone();
    assert_eq!((c_t1.observer, c_t1.spec.prompt), (1, "u05_shezhan_target_pick"));
    answer(&mut e, "u05_shezhan_target_pick", vec![ChoiceOption::Card(p1_c3)]);

    // Seat 2 responder pick IMMEDIATELY (must NOT re-ask `u05_shezhan_own_pick`!)
    let c_t2 = e.open_choice().expect("seat 2 pick (no second own pick)").clone();
    assert_eq!((c_t2.observer, c_t2.spec.prompt), (2, "u05_shezhan_target_pick"));
    let planner = BoundedPlanner::new(16);
    assert!(matches!(e.plan_action(2, &planner).action, GameAction::Answer { .. }));
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "u05_shezhan_target_pick", vec![ChoiceOption::Card(p2_c5)]); // 5 vs 5 tie -> -1!
    drain(&mut e);

    assert_eq!(e.debug_hand(0), vec![p0_keep], "p0 keeps unpicked card and draws 0 (+1 - 1 = 0, not > 0)");
    assert!(e.debug_hand(1).is_empty() && e.debug_hand(2).is_empty());
    assert_eq!(total_tracked_cards(&e), tracked);
}

#[test]
fn u05_b1_r3_fix03_mtg_bingheyaosai_mandatory_discard() {
    let (verified, mut e) = start_u05_b1_engine(
        93004,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let bh = find_card(&e, |c| c.kind == CardKind::Bingheyaosai);
    let sha = find_card(&e, |c| c.kind == CardKind::Sha);
    let fodder = find_card(&e, |c| c.kind == CardKind::Shan);
    move_card_to_hand(&mut e, 0, bh);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bh, targets: vec![] });
    drain(&mut e);
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, sha);
    move_card_to_hand(&mut e, 0, fodder);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    let c_bh = e.open_choice().expect("bingheyaosai mandatory discard ask").clone();
    assert_eq!(
        (c_bh.observer, c_bh.spec.prompt, c_bh.spec.min, c_bh.spec.max, c_bh.spec.allow_cancel),
        (0, "u05_mtg_bingheyaosai_sha_discard", 1, 1, false)
    );
    let mut e = snapshot_roundtrip(&e, &verified);
    let rej = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_bh.id,
            selection: Vec::new(),
        },
    );
    assert!(matches!(rej, CommandOutcome::Rejected(_)), "mandatory discard must reject empty cancel");
    answer(&mut e, "u05_mtg_bingheyaosai_sha_discard", vec![ChoiceOption::Card(fodder)]);
    drain(&mut e);
    assert!(!e.debug_hand(0).contains(&fodder));
}

#[test]
fn u05_b1_r3_fix04_zhenfa_targets_and_qixingzhen_observer() {
    let (verified, mut e) = start_u05_b1_engine(
        93005,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-zhenfa-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let wei = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wei))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let shu = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Shu))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();

    // 1. `longfeizhen`: 0=Wei, 1=Shu, 2=Wei, 3=Shu -> neighbors of 1 are [0, 2].
    advance_to_phase(&mut e, 1, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &shu);
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &shu);
    let lf = find_card(&e, |c| c.name == "longfeizhen");
    let c0 = find_card(&e, |c| c.kind == CardKind::Shan);
    let c2 = find_card(&e, |c| c.kind == CardKind::Tao);
    move_card_to_hand(&mut e, 1, lf);
    move_card_to_hand(&mut e, 0, c0);
    // Only seat 0 has `he`; seat 2 is empty -> only `[0]` is legal!
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: lf, targets: vec![0, 2] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "longfeizhen must reject empty-he neighbor 2");
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: lf, targets: vec![2] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    // Now BOTH neighbors 0 and 2 have `he` -> partial `[0]` MUST reject; full `[0, 2]` MUST accept!
    move_card_to_hand(&mut e, 2, c2);
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: lf, targets: vec![0] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "longfeizhen selectTarget:-1 must reject partial target set [0]");
    assert!(!matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: lf, targets: vec![0, 2] }),
        CommandOutcome::Rejected(_)
    ));
    drain(&mut e);

    // 2. `huyizhen`: seat 0 (Wei) & seat 2 (Wei) siege BOTH seat 1 (Shu) AND seat 3 (Shu)!
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let hy = find_card(&e, |c| c.name == "huyizhen");
    let nb0a = find_card(&e, |c| c.kind == CardKind::Armor);
    let nb0b = find_card(&e, |c| c.kind == CardKind::Weapon);
    let nb2a = find_card(&e, |c| c.kind == CardKind::HorsePlus);
    let nb2b = find_card(&e, |c| c.kind == CardKind::HorseMinus);
    move_card_to_hand(&mut e, 0, hy);
    move_card_to_hand(&mut e, 0, nb0a);
    move_card_to_hand(&mut e, 0, nb0b);
    move_card_to_hand(&mut e, 2, nb2a);
    move_card_to_hand(&mut e, 2, nb2b);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: hy, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "huyizhen selectTarget:-1 must reject single target [1] when [1, 3] both sieged");
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: hy, targets: vec![1, 3] }),
        CommandOutcome::Rejected(_)
    ));
    drain_nullify_local(&mut e);
    let mut e = snapshot_roundtrip(&e, &verified);
    let mut picks = Vec::new();
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "u05_zhenfa_huyizhen_card" {
            picks.push(c.observer);
            answer(&mut e, "u05_zhenfa_huyizhen_card", vec![c.spec.options[0]]);
        } else {
            let _ = submit(&mut e, c.observer, GameAction::Answer {
                choice_id: c.id,
                selection: if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] } else { Vec::new() },
            });
        }
    }
    assert_eq!(picks, vec![0, 2, 0, 2], "huyizhen asks flanking actors [0, 2] for BOTH victims 1 & 3");
    assert_eq!((e.debug_hp(1), e.debug_hp(3)), (2, 2), "both sieged victims 1 & 3 took 2 Sha damage");

    // 3. `qixingzhen`: observer == caster (`2`), hand-only (equips excluded and survive!)
    advance_to_phase(&mut e, 2, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let qx = find_card(&e, |c| c.kind == CardKind::Qixingzhen);
    let p1_eq = find_card(&e, |c| c.kind == CardKind::Armor);
    let p1_h = find_card(&e, |c| c.kind == CardKind::Jiu);
    let p3_h = find_card(&e, |c| c.kind == CardKind::Tao);
    advance_to_phase(&mut e, 1, Phase::Play);
    move_card_to_hand(&mut e, 1, p1_eq);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: p1_eq, targets: vec![] });
    drain(&mut e);
    advance_to_phase(&mut e, 2, Phase::Play);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    move_card_to_hand(&mut e, 1, p1_h);
    move_card_to_hand(&mut e, 3, p3_h);
    move_card_to_hand(&mut e, 2, qx);

    let _ = submit(&mut e, 2, GameAction::PlayCard { card: qx, targets: vec![2] });
    drain_nullify_local(&mut e);
    let mut e = snapshot_roundtrip(&e, &verified);
    let c_qx = e.open_choice().expect("qixingzhen discard choice").clone();
    assert_eq!((c_qx.observer, c_qx.spec.prompt), (2, "swd_qixingzhen_discard"), "observer MUST be caster 2");
    assert!(c_qx.spec.options.contains(&ChoiceOption::Card(p1_h)));
    assert!(!c_qx.spec.options.contains(&ChoiceOption::Card(p1_eq)), "qixingzhen discards hand only (`h`), NOT equips");
    let planner = BoundedPlanner::new(16);
    assert!(matches!(e.plan_action(2, &planner).action, GameAction::Answer { .. }), "AI handles swd_qixingzhen_discard");
    drain(&mut e);
    assert_eq!(e.debug_equips(1), vec![p1_eq], "equip survived qixingzhen");
}

#[test]
fn u05_b1_r3_fix05_diyjianxiong_kingdoms() {
    let (verified, mut e) = start_u05_b1_engine(
        93006,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.state_core_mut().mode = GameMode::Kingdoms;
    let wei = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wei))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let shu = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Shu))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_assemble_general(3, &shu);
    for p in 0..4 {
        e.state_core_mut().player_mut(p).revealed_generals = [true, false];
    }
    e.state_core_mut().set_kingdom_identity(0, KingdomIdentity::Declared("wei".into())).unwrap();
    e.state_core_mut().set_kingdom_identity(1, KingdomIdentity::Declared("wei".into())).unwrap();
    e.state_core_mut().set_kingdom_identity(2, KingdomIdentity::Declared("shu".into())).unwrap();
    e.state_core_mut().set_kingdom_identity(3, KingdomIdentity::Declared("shu".into())).unwrap();
    e.debug_grant(0, "ycndiyjianxiong");
    e.debug_grant(0, "咆哮");
    e.debug_set_hp(1, 1);
    e.debug_set_hp(2, 1);
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    let mut e = snapshot_roundtrip(&e, &verified);

    assert!(e.state_core().is_friend_of(e.catalog(), 1, 0), "precondition: p1 is wei friend of p0");
    let sha1 = find_card(&e, |c| c.kind == CardKind::Sha);
    let sha2 = find_card_excl(&e, &[sha1], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, sha1);
    move_card_to_hand(&mut e, 0, sha2);

    // Kill friend seat 1 -> diyjianxiong mutates seat 1's kingdom_identity away from "wei"!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    drain(&mut e);
    assert!(!e.state_core().player(1).alive);
    assert!(
        !e.state_core().is_friend_of(e.catalog(), 1, 0),
        "diyjianxiong in Kingdoms must mutate dying friend's kingdom_identity away from owner's group: {:?}",
        e.state_core().player(1).kingdom_identity
    );
    assert_eq!(e.state_core().source_is_enemy_of(e.catalog(), 0, 1), Some(true));

    // Negative check: killing enemy seat 2 ("shu") leaves "shu" unchanged!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![2] });
    drain(&mut e);
    assert_eq!(e.state_core().player(2).kingdom_identity, KingdomIdentity::Declared("shu".into()));
}

#[test]
fn u05_b1_r3_fix06_step_asks_alignment_beige_xiongzi_yaliang() {
    let (verified, mut e) = start_u05_b1_engine(
        93007,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }

    // 1. `YunchouBeige`: mandatory discard (`min:1, allow_cancel:false`) + `snapshot_roundtrip` + Club `src` choice + `beige_gains`!
    e.debug_grant(2, "beige");
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    // Use Club judge card of rank R, and seat 2 discards Club card of rank R -> recovers BOTH judged & discarded!
    let judge_club = find_card(&e, |c| c.suit == Suit::Club);
    let judge_rank = e.catalog().cards[judge_club as usize].rank;
    let pay_same_sr = find_card_excl(&e, &[judge_club], |c| c.suit == Suit::Club && c.rank == judge_rank);
    let sha_a = find_card_excl(&e, &[judge_club, pay_same_sr], |c| c.kind == CardKind::Sha);
    let src_eq1 = find_card_excl(&e, &[judge_club, pay_same_sr, sha_a], |c| c.kind == CardKind::Armor);
    let src_eq2 = find_card_excl(&e, &[judge_club, pay_same_sr, sha_a, src_eq1], |c| c.kind == CardKind::Shan);
    let src_keep = find_card_excl(&e, &[judge_club, pay_same_sr, sha_a, src_eq1, src_eq2], |c| c.kind == CardKind::Tao);

    move_card_to_hand(&mut e, 2, pay_same_sr);
    move_card_to_hand(&mut e, 0, src_eq1);
    move_card_to_hand(&mut e, 0, src_eq2);
    move_card_to_hand(&mut e, 0, src_keep);
    move_card_to_hand(&mut e, 0, sha_a);
    put_deck_top(&mut e, judge_club);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
    let c_bd = e.open_choice().expect("beige_discard").clone();
    assert_eq!((c_bd.observer, c_bd.spec.prompt, c_bd.spec.min, c_bd.spec.max, c_bd.spec.allow_cancel), (2, "beige_discard", 1, 1, false));
    let mut e = snapshot_roundtrip(&e, &verified); // Would FAIL before Fix #6 due to ChoiceSpec mismatch!
    assert!(matches!(
        submit(&mut e, 2, GameAction::Answer { choice_id: c_bd.id, selection: Vec::new() }),
        CommandOutcome::Rejected(_)
    ));
    answer(&mut e, "beige_discard", vec![ChoiceOption::Card(pay_same_sr)]);

    // Club judge MUST ask `u05_beige_club_discard` on `observer == 0` (Sha source)!
    let c_club = e.open_choice().expect("u05_beige_club_discard on Sha source").clone();
    assert_eq!((c_club.observer, c_club.spec.prompt, c_club.spec.min, c_club.spec.max), (0, "u05_beige_club_discard", 2, 2));
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "u05_beige_club_discard", vec![ChoiceOption::Card(src_eq1), ChoiceOption::Card(src_eq2)]);
    drain(&mut e);

    assert_eq!(e.debug_hand(0), vec![src_keep], "src discarded player-chosen 2 cards, keeping src_keep");
    assert!(
        e.debug_hand(2).contains(&judge_club) && e.debug_hand(2).contains(&pay_same_sr),
        "beige_gains must return same-suit judged card and same-rank discarded card to seat 2"
    );

    // 1b. Diamond judge branch draws 2 BEFORE `beige_gains` runs!
    e.debug_clear_hand(2);
    let judge_dia = find_card_excl(&e, &[sha_a], |c| c.suit == Suit::Diamond);
    let pay_dia = find_card_excl(&e, &[judge_dia, sha_a], |c| c.suit == Suit::Diamond);
    let filler_d1 = find_card_excl(&e, &[judge_dia, pay_dia, sha_a], |c| c.kind == CardKind::Shan);
    let filler_d2 = find_card_excl(&e, &[judge_dia, pay_dia, sha_a, filler_d1], |c| c.kind == CardKind::Tao);
    let sha_b = find_card_excl(&e, &[judge_dia, pay_dia, sha_a, filler_d1, filler_d2], |c| c.kind == CardKind::Sha);
    e.debug_grant(0, "咆哮");
    e.debug_clear_hand(1);
    move_card_to_hand(&mut e, 2, pay_dia);
    move_card_to_hand(&mut e, 0, sha_b);
    // Stack deck top (popped from back): judge_dia last (popped first for judge), then filler_d2, filler_d1 for victim draw 2
    put_deck_top(&mut e, filler_d1);
    put_deck_top(&mut e, filler_d2);
    put_deck_top(&mut e, judge_dia);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
    answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
    answer(&mut e, "beige_discard", vec![ChoiceOption::Card(pay_dia)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "Diamond branch drew 2 for victim seat 1");
    assert!(e.debug_hand(2).contains(&judge_dia), "beige_gains works after Diamond DrawSettle");

    // 2. `YunchouXiongzi`: draws `1 + equips/2` (0->3, 2->4, 3->4)!
    e.debug_grant(1, "xiongzi");
    let eq_x1 = find_card(&e, |c| c.kind == CardKind::Weapon);
    let eq_x2 = find_card(&e, |c| c.kind == CardKind::HorseMinus);
    let eq_x3 = find_card(&e, |c| c.kind == CardKind::HorsePlus);
    advance_to_phase(&mut e, 1, Phase::Play);
    move_card_to_hand(&mut e, 1, eq_x1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: eq_x1, targets: vec![] });
    drain(&mut e);
    move_card_to_hand(&mut e, 1, eq_x2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: eq_x2, targets: vec![] });
    drain(&mut e);
    e.debug_clear_hand(1);
    // Advance past seat 1 to seat 2, then back to seat 1 Play -> Draw phase with 2 equips draws 2 + (1 + 2/2) = 4!
    advance_to_phase(&mut e, 2, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(e.debug_hand(1).len(), 4, "YunchouXiongzi with 2 equips draws 2 + (1 + 2/2) = 4");
    move_card_to_hand(&mut e, 1, eq_x3);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: eq_x3, targets: vec![] });
    drain(&mut e);
    e.debug_clear_hand(1);
    advance_to_phase(&mut e, 2, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(e.debug_hand(1).len(), 4, "YunchouXiongzi with 3 equips draws 2 + (1 + 3/2) = 4");

    // 3. `YunchouYaliang`: multi-point damage (`Jiu` + `Sha` = 2 dmg) offers 2nd draw (`getIndex`)!
    advance_to_phase(&mut e, 0, Phase::Play);
    let beige_def = e.catalog().skill_by_name("beige").unwrap();
    e.state_core_mut().skills.remove(&(2, beige_def));
    e.debug_grant(0, "yaliang");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 4);
    let weap0 = find_card_excl(&e, &[eq_x1], |c| {
        c.kind == CardKind::Weapon
            && c.range >= 2
            && !c.name.contains("hanbing")
            && !c.name.contains("cixiong")
    });
    let jiu = find_card(&e, |c| c.kind == CardKind::Jiu);
    let sha_j = find_card_excl(&e, &[sha_a, sha_b], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, weap0);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: weap0, targets: vec![] });
    drain(&mut e);
    move_card_to_hand(&mut e, 0, jiu);
    move_card_to_hand(&mut e, 0, sha_j);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
    drain(&mut e);
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_j, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));
    // First Yaliang draw (+1/+1) asks `trigger_confirm`, then 2nd `trigger_confirm` asks for dmg amount 2 (`getIndex`)!
    answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
    let mut saw_second_confirm = false;
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "trigger_confirm" {
            saw_second_confirm = true;
            let planner = BoundedPlanner::new(16);
            assert!(matches!(e.plan_action(0, &planner).action, GameAction::Answer { .. }));
            e = snapshot_roundtrip(&e, &verified);
            answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
            break;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            vec![]
        };
        let _ = submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    drain(&mut e);
    assert!(saw_second_confirm, "YunchouYaliang with 2 damage must ask 2nd trigger_confirm for getIndex");
    assert_eq!((e.debug_hand(0).len(), e.debug_hand(1).len()), (2, 2), "YunchouYaliang draws 2 pairs for 2 damage");
}

#[test]
fn u05_b1_r3_fix07_shuigong_all_equips_and_diyguhuo_judge_log() {
    let (verified, mut e) = start_u05_b1_engine(
        93008,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 1. `Shuigong` `selectTarget: -1`: must select ALL alive equip-bearing players!
    let sg = find_card(&e, |c| c.kind == CardKind::Shuigong);
    let eq1 = find_card(&e, |c| c.kind == CardKind::Weapon);
    let eq2 = find_card(&e, |c| c.kind == CardKind::Armor);
    advance_to_phase(&mut e, 1, Phase::Play);
    move_card_to_hand(&mut e, 1, eq1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: eq1, targets: vec![] });
    drain(&mut e);
    advance_to_phase(&mut e, 2, Phase::Play);
    move_card_to_hand(&mut e, 2, eq2);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: eq2, targets: vec![] });
    drain(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    move_card_to_hand(&mut e, 0, sg);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sg, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sg, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "shuigong selectTarget:-1 must reject partial equip target [1] when both 1 & 2 have equips");
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sg, targets: vec![1, 2] }),
        CommandOutcome::Rejected(_)
    ));
    drain_nullify_local(&mut e);
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "u05_shuigong_discard", vec![ChoiceOption::Card(eq1)]);
    answer(&mut e, "u05_shuigong_discard", vec![ChoiceOption::Card(eq2)]);
    drain(&mut e);
    assert!(e.debug_equips(1).is_empty() && e.debug_equips(2).is_empty());

    // 2. `diyguhuo` discarding from `judge` MUST emit `LogEvent::CardDiscarded`!
    e.debug_grant(0, "diyguhuo");
    advance_to_phase(&mut e, 2, Phase::Play);
    e.debug_clear_hand(0);
    let le = find_card(&e, |c| c.kind == CardKind::Le);
    move_card_to_hand(&mut e, 2, le);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: le, targets: vec![0] });
    drain(&mut e);
    assert_eq!(e.state_core().player(0).judge, vec![le], "Le placed into seat 0 judge zone");

    // Advance from seat 2's Play into seat 0's TurnBegin -> `ycndiyguhuo` fires BEFORE Le judge!
    for _ in 0..24 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "u05_diyguhuo_discard" {
                break;
            }
            let _ = submit(&mut e, c.observer, GameAction::Answer {
                choice_id: c.id,
                selection: if c.spec.options.contains(&ChoiceOption::Confirm) {
                    vec![ChoiceOption::Confirm]
                } else {
                    c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
                },
            });
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    let c_gh = e.open_choice().expect("u05_diyguhuo_discard prompt").clone();
    assert_eq!(c_gh.spec.prompt, "u05_diyguhuo_discard");
    assert!(c_gh.spec.options.contains(&ChoiceOption::Card(le)), "judge card in u05_diyguhuo_discard options");
    let mut e = snapshot_roundtrip(&e, &verified);
    let log_start = e.log().len();
    let other_card = c_gh
        .spec
        .options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) if c != le => Some(c),
            _ => None,
        })
        .next()
        .unwrap();
    answer(&mut e, "u05_diyguhuo_discard", vec![ChoiceOption::Card(le), ChoiceOption::Card(other_card)]);
    drain(&mut e);
    assert!(!e.state_core().player(0).judge.contains(&le), "judge card removed");
    assert!(e.state_core().discard.contains(&le), "judge card in discard pile");
    assert!(
        e.log()[log_start..].iter().any(|ev| matches!(
            ev,
            LogEvent::CardDiscarded { player: 0, cards } if cards.contains(&le)
        )),
        "diyguhuo discarding from judge MUST emit LogEvent::CardDiscarded containing {le}: {:?}",
        &e.log()[log_start..]
    );
}

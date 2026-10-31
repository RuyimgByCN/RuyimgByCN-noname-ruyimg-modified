use super::*;

/// 飞刃：黑桃杀 +1（4→2）；杀无视距离（距离 2 可达）；梅花杀双目标
/// 可达（非梅花杀双目标仍拒）。
#[test]
fn yuanshi_feiren_bonus_range_and_club_extra_target() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9931, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feiren");
    // 诸葛连弩：出杀不限次（飞刃无次数豁免，测试需连出多杀）
    let zhuge = e.debug_place(0, "zhuge");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备连弩: {out:?}"
    );
    // 多放几张杀保证三种花色齐备
    for _ in 0..8 {
        e.debug_place(0, "sha");
    }
    let spade = pick_suit(&e, engine_native::catalog::Suit::Spade).expect("应有黑桃杀");
    let club = pick_suit(&e, engine_native::catalog::Suit::Club).expect("应有梅花杀");
    let red = pick_suit(&e, engine_native::catalog::Suit::Heart)
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Diamond))
        .expect("应有红杀");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 黑桃杀 +1（距离 1）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: spade,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(1), 2, "黑桃杀 +1（4→2）");

    // ② 杀无视距离：距离 2 的 2 号可达（红杀无加成 4→3）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "飞刃杀无视距离：{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(2), 3, "红杀无加成（4→3）");

    // ③ 梅花杀双目标（feiren_extra=1）；红杀双目标仍拒
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: club,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "梅花杀双目标：{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 1/2 各再受 1（梅花无黑桃加成）
    assert_eq!(e.debug_hp(1), 1, "梅花杀无黑桃加成（2→1）");
    assert_eq!(e.debug_hp(2), 2, "梅花杀（3→2）");

    let red2 = pick_suit(&e, engine_native::catalog::Suit::Heart)
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Diamond));
    if let Some(r2) = red2 {
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: r2,
                targets: vec![1, 2],
            },
        );
        assert!(
            matches!(
                &out,
                CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
            ),
            "非梅花杀双目标应拒：{out:?}"
        );
    }
}

/// 飞刃 notLink：黑桃火杀经铁索传导——主伤害 +1、传导伤害不加成
///（DamageData.from_chain 新字段）。
#[test]
fn yuanshi_feiren_chain_damage_not_bonused() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9932, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feiren");
    // 铁索连环连接 1、2
    let chain = e.debug_place(0, "chain");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: chain,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 黑桃火杀打 1：主伤害 1+1=2，传导给 2 为 1（不加成）
    let mut fire_spade = None;
    for _ in 0..8 {
        e.debug_place(0, "fire_sha");
        if let Some(c) = pick_suit(&e, engine_native::catalog::Suit::Spade) {
            // 排除普通杀误选（手牌里只有 fire_sha）
            fire_spade = Some(c);
            break;
        }
    }
    let fsha = fire_spade.expect("应有黑桃火杀");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fsha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(1), 2, "主伤害 +1（4→2）");
    assert_eq!(e.debug_hp(2), 3, "传导伤害不加成（4→3）");
}

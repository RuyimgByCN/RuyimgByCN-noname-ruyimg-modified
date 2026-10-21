use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十七（规则 v134）：莱因哈特（ow_laiyinhate）猛击（mengji，
// 伤害前置窗口触发锁定）——你无护甲时你的杀伤害 +1（非铁索传导）。
// zhongdun（init2 开局甲+主动转甲）登记暂缓，storage.zhongdun 以持技
// 等价。
// ---------------------------------------------------------------------------

/// 猛击：无甲时杀伤害 +1（4→2）；有甲时不加成（4→3）。
#[test]
fn laiyinhate_mengji_no_armor_bonus() {
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

    let mut e = Engine::new_with_extensions(9981, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "mengji");
    // 连弩解除限次
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    for _ in 0..3 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 无甲：杀伤害 +1（4→2）
    let s1 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s1,
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
    assert_eq!(e.debug_hp(1), 2, "无甲杀 +1（4→2）");

    // ② 有甲（debug 直设甲 1）：不加成（3→2，仍打距离 1 的 1 号）
    e.debug_set_hp(1, 4);
    e.debug_set_hujia(0, 1);
    let s2 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应仍有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s2,
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
    assert_eq!(e.debug_hp(1), 3, "有甲不加成（4→3）");
}

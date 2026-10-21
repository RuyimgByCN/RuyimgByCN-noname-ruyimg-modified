use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第五切片（规则 v146）：温慧（pal_wenhui）龙翔（longx-
// iang，ShaResolve 内联钩）——出杀结算开始时目标手牌多于自己→其随机
// 弃差值张。huxi（虚拟用牌）登记暂缓。
// ---------------------------------------------------------------------------

/// 龙翔：目标手牌多于自己→随机弃差值张；不多于则不弃。
#[test]
fn wenhui_longxiang_random_discard_diff() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10003, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "longxiang");
    // 自己 1 张，目标 4 张→差 3 张随机弃
    e.debug_place(0, "sha");
    for _ in 0..4 {
        e.debug_place(1, "tao");
    }
    e.debug_set_hp(1, 4);
    let hand1_before = e.debug_hand(1).len();

    let sha0 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha0,
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
    // 1 号有 4 张（tao 可当闪？tao 非闪——出闪询问选项仅 Pass/闪）
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
    // 杀已在结算区（不计入手牌）——差值=4-0=4（上游 countCards 同口径）
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_before - 4,
        "龙翔：随机弃差值 4 张（结算区杀不计入来源手牌）"
    );
    assert_eq!(e.debug_hp(1), 3, "杀照常命中（4→3）");
}

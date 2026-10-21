use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三十四批切片（规则 v176）：仙卿（pal_xianqing，qun 4
// 男）探息（xtanxi，phaseUse usable:1 主动）——弃一张手牌，持有同名牌
// 手牌的其他角色中随机一名，随机获得其一张手牌。xiaoyue 复合缺口登记
// 暂缓。
// ---------------------------------------------------------------------------

/// 探息：弃牌→持有同名牌者随机夺取一张；无人持有则牌照弃。
#[test]
fn xianqing_xtanxi_same_name_steal() {
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

    let mut e = Engine::new_with_extensions(10048, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xtanxi");

    // ① 布阵：0 号弃 sha（黑桃杀）；1/2 号各持一张 sha（同牌种）
    let disc_c = e.debug_place(0, "sha");
    let t1 = e.debug_place(1, "sha");
    let t2 = e.debug_place(2, "sha");

    let def = e.catalog().skill_by_name("xtanxi").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(disc_c)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 自动执行：随机目标 1 或 2 被夺一张杀 → 0 号手牌 +1
    let got: Vec<u32> = e
        .debug_hand(0)
        .into_iter()
        .filter(|c| *c == t1 || *c == t2)
        .collect();
    assert_eq!(got.len(), 1, "随机夺取一张同名牌");
    assert_eq!(e.debug_hand(0).len(), 1, "弃 1 夺 1");
    assert!(e.state_core().discard.contains(&disc_c), "所弃牌入弃堆");
}

/// 探息取消路径：无人持有同名牌→牌照弃直接结束（询问取消）。
#[test]
fn xianqing_xtanxi_no_holder_cancel() {
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

    let mut e = Engine::new_with_extensions(10049, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xtanxi");
    let wz = e.debug_place(0, "wuzhong"); // 无人持无中
    let _other = e.debug_place(1, "tao"); // 1 号有手牌（activate 门槛）

    let def = e.catalog().skill_by_name("xtanxi").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("探息弃牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_XTANXI_CARDS
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(wz)],
        },
    );
    assert!(e.state_core().discard.contains(&wz), "牌照弃");
    assert!(e.debug_hand(0).is_empty(), "无人可夺：无获得");
}

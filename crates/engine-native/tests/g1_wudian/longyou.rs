use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十八切片（规则 v159）：龙幽（pal_longyou，wei 4 男）月
// 行（yuexing，出牌阶段限一次主动）+敏杀（minsha，出牌阶段限一次主动）
// ——距离基数交换至自己出牌阶段结束（computed_distance 座位代换）；弃
// 两张 he 牌令体力>1 的其他角色受 1 雷伤、距其 1 内其他角色各随机弃。
// ---------------------------------------------------------------------------

/// 月行：激活后双向距离按配对者座位为基数；出牌阶段结束解除。
#[test]
fn longyou_yuexing_distance_swap_and_clear() {
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

    let mut e = Engine::new_with_extensions(10021, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_grant(0, "yuexing");
    let dist = |e: &Engine, a: u8, b: u8| e.state_core().computed_distance(e.catalog(), a, b);
    // 基线：4 人 0→2 座位距 2
    assert_eq!(dist(&e, 0, 2), 2, "基线 0→2 距 2");

    // 激活月行指定 2 号：0 的出距以 2 的座位为基数（0→1 = 座位(2→1)=1）；
    // 2 的出距以 0 的座位为基数（2→3 = 座位(0→3)=1）
    let def = e.catalog().skill_by_name("yuexing").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("月行目标挂起");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    assert_eq!(dist(&e, 0, 1), 1, "0→1 以 2 座位为基数（座位(2→1)=1）");
    assert_eq!(dist(&e, 2, 3), 1, "2→3 以 0 座位为基数（座位(0→3)=1）");
    assert_eq!(e.state_core().player(0).yuexing_pair, Some(2), "双向互指");
    assert_eq!(e.state_core().player(2).yuexing_pair, Some(0));

    // 出牌阶段结束解除：Play→Discard 后距离复原
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(dist(&e, 0, 2), 2, "阶段结束解除复原");
    assert_eq!(e.state_core().player(0).yuexing_pair, None);
    assert_eq!(e.state_core().player(2).yuexing_pair, None);
}

/// 敏杀：弃两张 he 牌→目标雷伤 1→距其 1 内其他角色（除自己）各随机弃。
#[test]
fn longyou_minsha_thunder_and_neighbor_discard() {
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

    let mut e = Engine::new_with_extensions(10022, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "minsha");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4); // 目标（hp>1）
    e.debug_set_hp(2, 4); // 距 1 号 1 → 随机弃 1
    e.debug_set_hp(3, 4); // 距 1 号 2 → 不弃
    let c1 = e.debug_place(0, "wuzhong");
    let c2 = e.debug_place(0, "wuzhong");
    let d1 = e.debug_place(2, "tao"); // 邻座被随机弃候选

    // 激活：弃两张 → 指定 1 号
    let def = e.catalog().skill_by_name("minsha").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("敏杀目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_XJ_MINSHA_TARGET, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );

    // 断言：1 号 4→3 雷伤；2 号随机弃 1（4→3 手牌）；3 号不弃；所弃两张入弃堆
    assert_eq!(e.debug_hp(1), 3, "目标雷伤（4→3）");
    assert_eq!(e.debug_hand(0).len(), 0, "两张已弃");
    assert!(e.state_core().discard.contains(&c1) && e.state_core().discard.contains(&c2));
    assert_eq!(e.debug_hand(2).len(), 0, "距 1 内邻座随机弃 1（1 桃→0）");
    assert!(
        e.state_core().discard.contains(&d1),
        "唯一候选=桃被随机弃入弃堆"
    );
    assert_eq!(e.debug_hand(3).len(), 0, "距 2 者不弃");
}

use super::*;

/// 乱剑：两张杀当一张杀虚拟使用（多目标）+乱剑2 载体锚定 50% 增伤
///（每目标独立 roll）；普通杀不经乱剑2（载体不匹配）。
#[test]
fn changqing_luanjian_dual_sha_viewas_multi_target() {
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

    let mut e = Engine::new_with_extensions(10062, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "luanjian");
    e.debug_grant(0, "luanjian2");
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");

    // 发动乱剑：选两杀 → 多目标 [1,2]
    let def = e.catalog().skill_by_name("luanjian").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("乱剑选牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_LUANJIAN_CARDS
    );
    assert_eq!((c.spec.min, c.spec.max), (2, 2));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(sha1), ChoiceOption::Card(sha2)],
        },
    );
    let c = e.open_choice().expect("乱剑选目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_LUANJIAN_TARGETS
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        },
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
    let (h1, h2) = (e.debug_hp(1), e.debug_hp(2));
    assert_eq!(h1, 5, "目标 1 未增伤（seed 10062 首 roll≥50）: {h1}");
    assert_eq!(
        h2, 4,
        "目标 2 增伤 2 点（seed 10062 次 roll<50，逐目标独立）: {h2}"
    );
    assert!(
        !e.debug_hand(0).contains(&sha1) && !e.debug_hand(0).contains(&sha2),
        "两杀均已弃置"
    );
}

/// 天符：挂附（闪置离场区+授实例+记来源）→挂附者造成伤害后受来源
/// 1 雷伤+闪入弃堆+清挂附；挂附者死亡时清挂附。
#[test]
fn changqing_ctianfu_attach_thunder_and_death_clear() {
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

    // ---- 场景 A：反噬雷伤 ----
    let mut e = Engine::new_with_extensions(10063, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 6);
    e.debug_grant(0, "ctianfu");
    let shan = e.debug_place(0, "shan");
    let def = e.catalog().skill_by_name("ctianfu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("天符选闪挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_CTIANFU_CARD
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan)],
        },
    );
    let c = e.open_choice().expect("天符选目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_CTIANFU_TARGET
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
    // 挂附断言：闪离手入离场区+1 号获实例+来源=0
    assert!(!e.debug_hand(0).contains(&shan), "闪离手");
    assert!(
        e.state_core().player(1).off_court.contains(&shan),
        "闪置离场区挂附"
    );
    let has_ct2 = e
        .state_core()
        .skills
        .iter()
        .any(|((o, d), _)| *o == 1 && e.catalog().skill_name(*d) == "ctianfu2");
    assert!(has_ct2, "1 号获反噬实例");
    assert_eq!(e.state_core().player(1).ctianfu_source, Some(0));

    // 推到 1 号回合 Play：0 NP×2→Discard/End，1 号 Judge 起 NP×2 到 Play
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let sha = e.debug_place(1, "sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
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
    // 断言：0 被杀 -1（4→3）；1 反噬雷伤 -1（6→5，上游 player.damage
    // =挂附者承受）；挂附清（闪入弃堆+实例移除+标记 None）
    assert_eq!(e.debug_hp(0), 3, "0 被杀 -1（4→3）");
    assert_eq!(e.debug_hp(1), 5, "1 反噬雷伤 -1（6→5）");
    let cleared = !e
        .state_core()
        .skills
        .iter()
        .any(|((o, d), _)| *o == 1 && e.catalog().skill_name(*d) == "ctianfu2");
    assert!(cleared, "反噬实例已清");
    assert_eq!(e.state_core().player(1).ctianfu_source, None, "挂附标记清");
    assert!(e.state_core().discard.contains(&shan), "挂附闪入弃堆");

    // ---- 场景 B：死亡清理 ----
    let mut e2 = Engine::new_with_extensions(10064, 3, &[&pack]).expect("建局2");
    advance_loop(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "ctianfu");
    let shan2 = e2.debug_place(0, "shan");
    let def = e2.catalog().skill_by_name("ctianfu").expect("def");
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: def });
    let c = e2.open_choice().expect("选闪挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan2)],
        },
    );
    let c = e2.open_choice().expect("选目标挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    e2.debug_set_hp(1, 1);
    let sha = e2.debug_place(0, "sha");
    let out = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: sha,
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
    while let Some(c) = e2.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e2,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 断言：1 死亡+挂附闪入弃堆+标记清（上游凭空消失=登记差异）
    assert!(!e2.state_core().player(1).alive, "1 号死亡");
    assert!(e2.state_core().discard.contains(&shan2), "死亡清挂闪入弃堆");
    assert_eq!(e2.state_core().player(1).ctianfu_source, None, "死亡标记清");
}

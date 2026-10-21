use super::*;

/// T12 swd 武将面批 20：分舵（习嫣 jiefen 整将转正）——出牌主动技
/// （每回合限一次）三段机：选交牌目标（手>自己）→ 目标交 1 张手牌 →
/// 自己交 1 张手牌给手牌最少的其他角色。
#[test]
fn swd_generals_b20_jiefen_multistep() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62003, 4, &[&pack]).expect("建局");
    e.debug_grant(3, "jiefen");
    advance_to_play(&mut e, 3);
    // 布阵：全清手后 p3 手 1（mine）、p1 手 3（手>自己）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let mine = e.debug_place(3, "tao");
    let give1 = e.debug_place(1, "tao");
    let _give2 = e.debug_place(1, "tao");
    let _give3 = e.debug_place(1, "tao");
    let jiefen_def = e.catalog().skill_by_name("jiefen").expect("分舵 def");
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: jiefen_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "分舵受理: {out:?}"
    );

    // 问 1：选交牌目标 p1
    let (cid, obs) = e
        .open_choice()
        .map(|ch| (ch.id, ch.observer))
        .expect("q1 分舵目标");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Player(1)))
            .cloned()
            .collect()
    };
    assert_eq!(sel.len(), 1, "q1 目标面含 p1");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );

    // 问 2：p1 选 1 张手牌交出
    let (cid, obs) = e
        .open_choice()
        .map(|ch| (ch.id, ch.observer))
        .expect("q2 交牌窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .cloned()
            .take(1)
            .collect()
    };
    assert_eq!(sel.len(), 1, "q2 交牌面");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );

    // 登记差异：上游 chooseCardTarget 玩家自选自己交出的牌与目标，原生
    // 确定性直发（自己交手牌 first 给手牌最少的其他存活角色）——无 q3 窗。
    // 断言：p1 3-1=2；p3 +1-1=1；p0 +1=1
    assert_eq!(e.state_core().players[1].hand.len(), 2, "分舵：p1 交 1");
    assert_eq!(e.state_core().players[3].hand.len(), 1, "分舵：p3 持平");
    assert_eq!(e.state_core().players[0].hand.len(), 1, "分舵：p0 得 1");
    let _ = (mine, give1);
}

use super::*;

/// T12 swd deferred 消解批 104：灵弦（慕月 `swd_muyue`）——非你回合时，你
/// 使用或打出牌后，与一名攻击距离大于 1 的角色各摸 1（座位序首个近似）。
#[test]
fn swd_deferred_lingxian_draws_with_far_target() {
    let root = repo_extensions_root();
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

    let mut e = Engine::new_with_extensions(62063, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "lingxian");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    let out = play(&mut e, sha, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 1 号应答闪（打出闪 → CardResponded 触发灵弦）。
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let has_shan = c
            .spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == shan));
        let sel: Vec<ChoiceOption> = if has_shan {
            vec![ChoiceOption::Card(shan)]
        } else if c.spec.min == 0 || c.spec.allow_cancel {
            vec![ChoiceOption::Pass]
        } else {
            c.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    // 1 号打出闪后摸 1（净持平）；座位 3（距 1 号 2）也摸 1。
    assert_eq!(e.debug_hand(1).len(), 1, "灵弦：1 号打出闪后摸 1");
    assert_eq!(e.debug_hand(3).len(), 1, "灵弦：远位（座位 3）摸 1");
    assert_eq!(e.debug_hp(1), 4, "闪生效，1 号未受伤");
}

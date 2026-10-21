use super::*;

/// 守护（U07 ow 批四，规则 v121）：天使——出牌阶段弃一张【杀】令一名
/// 其他受伤角色回复 1 点体力；持有者不能使用【杀】（锁定禁杀）。
#[test]
fn tianshi_shouhu_discard_heal_and_sha_ban() {
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

    let mut e = Engine::new_with_extensions(9401, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "shouhu");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 3);
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 2); // 1 号受伤（可为目标）

    // ① 弃杀救 1 号（2→3）
    let sha = e.debug_place(0, "sha");
    let def_sh = e.catalog().skill_by_name("shouhu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sh });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("守护弃杀挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_SHOUHU_CARD, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(sha)],
        },
    );
    let c = e.open_choice().expect("守护目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_SHOUHU_TARGET, 0)
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
    assert_eq!(e.debug_hp(1), 3, "守护回复 1（2→3）");
    assert!(e.debug_hand(0).is_empty(), "杀已弃");

    // ② 禁杀锁定：0 号出杀被拒
    let sha2 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "守护禁杀: {out:?}"
    );

    // ③ 无杀可弃 → 不可发动
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sh });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
        ),
        "无杀应拒: {out:?}"
    );
}

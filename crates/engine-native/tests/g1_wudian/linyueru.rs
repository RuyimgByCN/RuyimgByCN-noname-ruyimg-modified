use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第六切片（规则 v147）：林月如（pal_linyueru）归元（gui-
// yuan，出牌阶段限一次）——已受伤时弃一张杀→回复 1 点体力+摸一张牌。
// qijian（虚拟用牌）登记暂缓。
// ---------------------------------------------------------------------------

/// 归元：弃杀→回复 1+摸 1；满血不可发动。
#[test]
fn linyueru_guiyuan_discard_sha_recover_draw() {
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

    let mut e = Engine::new_with_extensions(10004, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "guiyuan");
    e.debug_set_hp(0, 3); // 已受伤
    let sha = e.debug_place(0, "sha");

    // 弃杀→回复 1（3→4）+摸 1
    let def_gy = e.catalog().skill_by_name("guiyuan").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gy });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), 4, "回复 1（3→4）");
    assert_eq!(e.debug_hand(0).len(), 1, "摸 1");
    assert!(e.state_core().discard.contains(&sha), "弃杀入弃牌堆");

    // 满血+限一次：再发动拒绝（once_used 先于 activate 门控命中）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gy });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(
                engine_native::RejectReason::SkillAlreadyUsed
                    | engine_native::RejectReason::InvalidChoice
            )
        ),
        "满血/限一次应拒绝：{out:?}"
    );
}

use super::*;

/// 影锋：杀结算完毕后随机对一名非本杀目标的其他角色使用虚拟杀
///（3 人局候选唯一=确定性）；虚拟杀 use_as 挡递归（否则继续链杀）。
#[test]
fn yuejinzhao_xjyingfeng_random_virtual_sha() {
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

    let mut e = Engine::new_with_extensions(10058, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "xjyingfeng");
    let sha = e.debug_place(0, "sha");

    // 真实杀目标 1 → 结算后影锋自动虚拟杀 2（候选排除目标 1 唯一）
    let out = submit(
        &mut e,
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
    assert_eq!(e.debug_hp(1), 3, "真杀目标 1 -1（闪 Pass）");
    assert_eq!(e.debug_hp(2), 3, "影锋虚拟杀 2 -1（闪 Pass）");
    assert!(!e.debug_hand(0).contains(&sha), "真实杀已离手");
    // hp1=3 未再扣=虚拟杀 use_as 挡递归（否则链杀继续打 1 号）
}

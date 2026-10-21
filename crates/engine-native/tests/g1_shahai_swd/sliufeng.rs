use super::*;

/// T12 swd 武将面批㊟（司空宇疾风；linyun 两牌当杀/hutian 护天 deferred）：
/// marker 面行为 + 反例。
#[test]
fn swd_generals_b80_sliufeng() {
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
    let mut e = Engine::new_with_extensions(62048, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 疾风·司空宇面：体力值不大于自己的角色视为在攻击范围
    e.debug_grant(0, "sliufeng");
    e.debug_set_hp(0, 3); // 自身 hp3：p2 hp2（≤3 ✓ 入范围）
    e.debug_set_hp(2, 2);
    e.debug_set_hp(1, 4); // p1 hp4（>3 ✗ 距离 1 本就在）
    let sha1 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![2],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 1, "疾风：p2（hp2≤3 对座）入范围受 1");
    // 反例：p3 hp4（>3）对座出界
    let sha1b = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1b,
            targets: vec![3],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "疾风反例：p3（hp4>3）对座出界拒 {out:?}"
    );
    let _ = sha1b;
    let _ = e.debug_hand(0);
}

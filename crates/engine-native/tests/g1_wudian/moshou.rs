use super::*;

/// G1 墨守目标免疫：乐不思蜀不能指定墨守持有者为目标；其他目标不受影响。
#[test]
fn moshou_blocks_delayed_trick_target() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(3504, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 2 号授予墨守（费诗技能）
    e.debug_grant(2, "moshou");
    let le = e.debug_place(0, "le");
    // 目标 1 号（无免疫）→ 正常置入判定区
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 兵粮寸断 → 目标 2 号（墨守免疫乐/兵）→ 拒绝
    let bing = e.debug_place(0, "bing");
    let out2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: bing,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out2,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "墨守应使延时锦囊目标免疫: {out2:?}"
    );
    assert_zone_conservation_wudian(&e);
}

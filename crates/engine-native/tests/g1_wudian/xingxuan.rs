use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 首切片（规则 v142）：星璇（pal_xingxuan）灵雪（lingxue，
// 回复后触发锁定）——你回复体力后获得 1 点护甲。新转换包
// shaai-xianjian-upstream-noname（character 管线第二例）；feizhua/
// leiyu 登记暂缓。
// ---------------------------------------------------------------------------

/// 灵雪：回复体力后 +1 护甲（多次回复各 +1；他人回复不触发）。
#[test]
fn xingxuan_lingxue_recover_grants_hujia() {
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

    let mut e = Engine::new_with_extensions(9998, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lingxue");
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 2);
    e.debug_place(0, "tao");
    e.debug_place(0, "tao");

    // ① 自回复：吃桃 → +1 甲
    let tao = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Tao)
        .expect("应有桃");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), 3, "回复 1（2→3）");
    assert_eq!(e.save().core.players[0].hujia, 1, "灵雪 +1 护甲");

    // ② 再次回复：再 +1 甲
    let tao2 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Tao)
        .expect("应仍有桃");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao2,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.save().core.players[0].hujia, 2, "多次回复各 +1");
}

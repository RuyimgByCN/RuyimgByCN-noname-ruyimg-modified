use super::*;

/// 校验臂：舒筋散/飞镖目标合法性（无手牌目标拒、自-target 拒）。
#[test]
fn gujian_rejects_illegal_targets() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(5207, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sj, fb, black) = {
        let catalog = e.catalog();
        let find = |prefix: &str| {
            catalog
                .cards
                .iter()
                .position(|c| c.name.starts_with(prefix))
                .expect("牌在目录") as u32
        };
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        (find("gj_shujinsan"), find("gj_feibiao"), black)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sj, fb, black]);
    snap.core.players[0].hand.push(sj);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 舒筋散：无手牌目标被拒
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sj,
            targets: vec![2],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "{out:?}"
    );
    // 飞镖：自-target 被拒
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fb,
            targets: vec![0],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "{out:?}"
    );
    // 飞镖：双目标被拒
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fb,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "{out:?}"
    );
    assert_zone_conservation(&e);
}

/// 整包验收续（2026-10-08，第三例）：shahai-gujian 真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本
/// 续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn gujian_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-gujian")
        .expect("古剑安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "激活集"
    );

    // ② 安装包 → 原生建局（40 卡切片目录）
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-gujian-upstream-noname")
        .expect("古剑可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5501, 3, &[&pack]).expect("建局");
    assert!(e.catalog().cards.len() > base_cards, "古剑卡入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shahai-gujian", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "shahai-gujian"));
    assert!(
        !store.read_active().unwrap().packages.contains_key(&slot),
        "移出激活集"
    );

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert!(resumed.catalog().cards.len() > base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "在局续玩: {out:?}"
    );

    // ⑤ 重装守卫
    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(
        matches!(err, Err(content_runtime::RuntimeError::Revoked(_))),
        "撤销后重装应被拒: {err:?}"
    );

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

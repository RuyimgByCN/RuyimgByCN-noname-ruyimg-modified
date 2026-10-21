use super::*;

/// 内容锁绑定：宿主技能快照在扩展集合不含该包时拒绝恢复。
#[test]
fn host_skill_restore_rejects_without_pack() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3203, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "huanle_kuwu");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let sha = e.debug_place(0, "sha");
    // 杀 → 苦无机器短暂入快照；在杀结算的出闪询问挂起时苦无机器已
    // 结束——改为直接以「含宿主技能的目录版本」验证：存档带 v(host)，
    // 纯标准恢复必须拒绝。
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    let saved = e.save();
    match Engine::restore(&saved) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(e) => panic!("应报规则版本不符: {e:?}"),
        Ok(_) => panic!("纯标准恢复带宿主扩展存档必须拒绝"),
    }
}

use super::*;

/// xianjian（shaai-xianjian-upstream-noname，36 名武将元数据、0 张卡）
/// 全生命周期；赵灵儿三技（回盟/天蛇/天蛇2）全映射接线生效。
#[test]
fn xianjian_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shaai-xianjian")
        .expect("shaai-xianjian 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "激活集"
    );

    // ② 安装包 → 原生建局（0 卡注入；赵灵儿=回盟/天蛇/天蛇2 全映射）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.card_count(), 0, "xianjian 为纯武将包");
    assert_eq!(pack.generals.len(), 36, "36 名武将元数据");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6106, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards, "纯武将包不注入卡牌");
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let mut linger_skills = None;
    for idx in gen_base..(gen_base + 40) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "赵灵儿" {
                linger_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        linger_skills.map(|v| v.to_vec()),
        Some(vec!["huimeng", "tianshe", "tianshe2"]),
        "赵灵儿回盟/天蛇/天蛇2 全映射接线生效"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shaai-xianjian", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "shaai-xianjian"));
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
    assert_eq!(resumed.catalog().cards.len(), base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "在局续玩: {out:?}"
    );

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(
        matches!(err, Err(content_runtime::RuntimeError::Revoked(_))),
        "撤销后重装应被拒: {err:?}"
    );

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

use super::*;

/// T12 swd deferred 消解批 100：兰芷（环婉之）——使用梅花牌后，体力不大于
/// 自己的已受伤角色（含自己）按座位序各回复 1。恒发动近似（上游含 check）。
#[test]
fn swd_deferred_lanzhi_heals_le_own_hp_damaged() {
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
    // 兰芷已映射为宿主行为。
    let gen = pack
        .generals
        .iter()
        .find(|g| g.name == "swd_huanyuanzhi")
        .expect("swd_huanyuanzhi 在包内");
    let _ = gen;

    let mut e = Engine::new_with_extensions(62057, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lanzhi");
    // 0 号满血；1/2/3 号受伤且体力 <= 0 号。
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(2, 2);
    e.debug_set_hp(3, 3);
    // 找一张梅花牌（酒 ♣9，自身使用不造成伤害）。
    let club = (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let def = &e.catalog().cards[*c as usize];
            def.suit == engine_native::catalog::Suit::Club && def.kind == CardKind::Jiu
        })
        .expect("梅花酒在目录");
    inject_hand(&mut e, &pack, &[club]);
    let out = play(&mut e, club, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 4, "兰芷：1 号回复 1（3→4）");
    assert_eq!(e.debug_hp(2), 3, "兰芷：2 号回复 1（2→3）");
    assert_eq!(e.debug_hp(3), 4, "兰芷：3 号回复 1（3→4）");
    assert_eq!(e.debug_hp(0), 4, "兰芷：0 号满血不回复");

    // 反例：非梅花牌不触发（用黑桃装备，自身使用、不造成伤害、无次数限）。
    let spade_equip = (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let def = &e.catalog().cards[*c as usize];
            def.suit == engine_native::catalog::Suit::Spade && def.kind.equip_slot().is_some()
        })
        .expect("黑桃装备在目录");
    e.debug_set_hp(1, 3);
    inject_hand(&mut e, &pack, &[spade_equip]);
    let out = play(&mut e, spade_equip, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "兰芷：非梅花牌不回复");
}

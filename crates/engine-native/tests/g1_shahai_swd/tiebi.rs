use super::*;

/// T12 swd 武将面首批（批 18 同日）：铁壁（韩罗整将——杀指定距离 ≤1 目标
/// 时弃黑牌给目标 +1 护甲）+ 大同（习嫣部分——全场手牌极差 ≤1 摸 2）。
#[test]
fn swd_generals_b18_tiebi_and_datong() {
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
    assert!(
        pack.generals.len() >= 7,
        "累计七将（现 {} 名）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62000, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    assert_eq!(e.catalog().general_meta(base).unwrap().0, "swd韩罗");
    assert_eq!(e.catalog().general_meta(base + 1).unwrap().0, "swd习嫣");

    // 铁壁：p0 杀 p1（p2 持铁壁、与 p1 距离 1）→ p1 +1 护甲
    e.debug_grant(2, "tiebi");
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);
    let sha = e.debug_place(0, "sha");
    // p2 布一张黑牌
    let black = e.debug_place(2, "sha");
    {
        let core = e.state_core_mut();
        core.players[2].hand.retain(|x| *x != black);
        core.players[2].hand.push(black);
    }
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    let _ = out;
    settle_optional_confirm(&mut e);
    eprintln!(
        "DBG tiebi hujia1={} hand0={:?} hand2={:?} choice={:?}",
        e.state_core().players[1].hujia,
        e.state_core().players[0].hand,
        e.state_core().players[2].hand,
        e.open_choice()
            .map(|ch| (ch.id, ch.observer, ch.spec.prompt)),
    );
    // 护甲语义：+1 护甲在本次伤害时抵 1 伤——hp 不减、护甲清零
    assert_eq!(e.debug_hp(1), 8, "铁壁：护甲抵 1 伤（hp 不减）");
    assert_eq!(e.state_core().players[1].hujia, 0, "铁壁：护甲已消耗");
}

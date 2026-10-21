use super::*;

/// G1 纵向：发现 → 导入 → 桥接 → 建局 → 面里金针结算 → 内容锁。
#[test]
fn g1_gujian_vertical_existing_kinds_and_lock() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "shahai-gujian-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-gujian")
        .expect("古剑安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let (pack, deferred_names) = bridge(&json).expect("桥接");
    assert_eq!(
        (pack.def.name, pack.def.version),
        ("shahai-gujian-upstream-noname", 1),
        "包元数据来自 cards.json"
    );
    let companion: Vec<&str> = pack.host_skills.iter().map(|d| d.name).collect();
    assert_eq!(companion.len(), 18, "18 个伴随技能由引擎牌种表登记");
    assert_eq!(
        (companion[0], companion[17]),
        (
            engine_native::content::host::GJ_JINLIANZHU_KEY,
            engine_native::content::host::GJ_YUHENG_SKILL_KEY
        ),
        "按牌首次出现次序登记（决定目录技能编号）"
    );
    assert_eq!(
        pack.card_count(),
        40,
        "切片 40/40 张全卡转换（食物 12+机关 6+锦囊基本等 22）"
    );
    let deferred: usize = parsed_deferred_count(&json);
    assert_eq!(deferred, 0, "deferred 清零：古剑全卡转换收官");
    assert_eq!(deferred_names.len(), parsed_deferred_len(&json));

    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5201, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 40);
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 面里金针：目标摸 1 然后受 1 点伤害（无选择、无无懈窗口）
    let (ml, black, red) = {
        let catalog = e.catalog();
        let ml = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_mianlijinzhen"))
            .expect("面里金针在目录") as u32;
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        (ml, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[ml, black, red]);
    snap.core.players[0].hand.push(ml);
    snap.core.players[1].hand.push(black);
    snap.core.players[2].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 3);
    let h1 = e.debug_hand(1).len();
    let hp1 = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ml,
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
    assert!(
        e.open_choice().is_none(),
        "面里金针无无懈窗口，不应挂起选择"
    );
    assert_eq!(e.debug_hand(1).len(), h1 + 1, "目标摸 1");
    assert_eq!(
        e.debug_hp(1),
        hp1 - 1,
        "目标失去 1 点体力（规则 v77 对齐上游 loseHp）"
    );
    // 失去体力≠伤害：无伤害日志（不经护甲/不触发伤害来源技）。
    assert!(
        !e.log()
            .iter()
            .any(|l| matches!(l, LogEvent::DamageDealt { target: 1, .. })),
        "失去体力不产生 DamageDealt 日志"
    );
    assert!(
        e.log().iter().any(|l| matches!(
            l,
            LogEvent::HpLost {
                player: 1,
                amount: 1
            }
        )),
        "应有 HpLost 日志"
    );
    assert_zone_conservation(&e);

    // 内容锁：异扩展集合恢复拒绝
    let snap = e.save();
    let (mut pack2, _) = bridge(&json).expect("桥接2");
    pack2.def.name = "other-pack";
    let wrong = Engine::restore_with_extensions(&snap, &[&pack2]);
    assert!(wrong.is_err(), "异集合恢复应被拒绝");

    // 校验臂：面里金针目标体力须 ≥ 使用者（重新注入已入弃牌堆的金针）
    e.debug_set_hp(1, 1);
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[ml]);
    snap.core.players[0].hand.push(ml);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ml,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "低体力目标应被拒: {out:?}"
    );
}

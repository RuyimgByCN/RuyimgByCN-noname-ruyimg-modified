use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三十二批切片（规则 v174）：夏侯瑾轩（pal_xiahoujinxuan，
// wei 3 男）玄墨（xuanmo，phaseUse usable:1 主动）——弃三类手牌置牌堆
// 顶+实体发现同类型 2 张入手（实体发现基建首例=羊皮卷同款登记差异）。
// danqing 表现层登记不转。
// ---------------------------------------------------------------------------

/// 玄墨：弃锦囊置顶→实体池发现同类型 2 张入手；置顶牌在牌堆顶。
#[test]
fn xiahoujinxuan_xuanmo_discover_same_class() {
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

    let mut e = Engine::new_with_extensions(10045, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xuanmo");
    let wz = e.debug_place(0, "wuzhong"); // 锦囊（弃置+置顶）

    // 激活：弃 wz 置顶 → 实体发现同类型 2 张入手
    let def = e.catalog().skill_by_name("xuanmo").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(wz)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 断言：置顶牌=牌堆顶
    assert_eq!(*e.state_core().deck.last().expect("非空"), wz, "置牌堆顶");
    // 断言：入手 2 张锦囊类实体（从牌堆/弃堆发现，非 wz）
    let hand = e.debug_hand(0);
    assert_eq!(hand.len(), 2, "发现 2 张入手");
    for c in &hand {
        let k = e.catalog().cards[*c as usize].kind;
        assert!(
            !matches!(
                k,
                CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
            ) && !k.equip_slot().is_some(),
            "同类型=锦囊（{:?}）",
            k
        );
        assert_ne!(*c, wz, "不含置顶牌");
    }
    // 入手牌不再在牌堆/弃堆
    for c in &hand {
        assert!(!e.state_core().deck.contains(c), "离牌堆");
        assert!(!e.state_core().discard.contains(c), "离弃堆");
    }
}

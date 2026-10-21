use super::*;

/// 机关簇：冰魄通（逐目标首张同色比对）/乾坤镖（目标+邻座弃牌）/
/// 龙须钩（获得目标首张装备）/神火飞鸦（目标+邻座需闪否则火伤，
/// 响应循环 + 挂起派生字节恢复）。
#[test]
fn jiguan_batch_four_kinds() {
    use engine_native::archive::{decode_archive, encode_archive};
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
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };
    let card_of = |e: &Engine, kind: CardKind, suit: Suit| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == kind && c.suit == suit)
            .unwrap_or_else(|| panic!("牌种在目录")) as u32
    };

    // —— 冰魄通：使用者红桃首张 vs 目标红桃首张（同色）→ 弃+伤害 ——
    let mut e = Engine::new_with_extensions(5222, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (bpt, tao_red, sha_black, heart_sha) = {
        let catalog = e.catalog();
        let bpt = find(&e, "gj_bingpotong");
        let tao_red = card_of(&e, CardKind::Tao, Suit::Heart);
        let sha_black = card_of(&e, CardKind::Sha, Suit::Spade);
        let heart_sha = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Heart)
            .expect("红杀在目录") as u32;
        (bpt, tao_red, sha_black, heart_sha)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[bpt, tao_red, sha_black, heart_sha]);
    snap.core.players[0].hand.push(bpt);
    snap.core.players[0].hand.push(tao_red); // 使用者首张：红
    snap.core.players[1].hand.push(heart_sha); // 目标首张：红 → 同色
    snap.core.players[2].hand.push(sha_black); // 异色对照
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(1, 9);
    e.debug_set_hp(2, 9);
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: bpt,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    pass_nullify_window(&mut e);
    // 双段自选展示（逐技对照批五 v79）：使用者选红桃桃、目标 1 选红杀
    // → 同色：使用者弃展示牌、目标 1 失去 1 点体力（v77 原语）
    let c = e.open_choice().expect("冰魄通使用者展示挂起");
    assert_eq!(c.spec.prompt, "gj_bingpo_show_own");
    let cid = c.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao_red)],
        },
    );
    let c = e.open_choice().expect("冰魄通目标展示挂起");
    assert_eq!((c.spec.prompt, c.observer), ("gj_bingpo_show_target", 1));
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(heart_sha)],
        },
    );
    assert_eq!(e.debug_hp(1), hp1 - 1, "同色目标失去 1 点体力（规则 v77）");
    // 目标 2：使用者展示牌已弃、手牌空 → 上游 step0 直接跳过（无事）
    assert!(e.open_choice().is_none(), "使用者无手牌后后续目标跳过");
    assert_eq!(e.debug_hp(2), hp2, "异色目标无事");
    assert!(!e.debug_hand(0).contains(&tao_red), "同色后使用者弃展示牌");
    assert!(
        e.debug_hand(1).contains(&heart_sha),
        "目标展示牌保留（展示非弃置）"
    );
    assert!(e.debug_hand(2).contains(&sha_black), "异色目标牌保留");
    assert_zone_conservation(&e);

    // —— 乾坤镖：目标弃首张手牌；龙须钩：获得目标首张装备 ——
    let mut e = Engine::new_with_extensions(5223, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (qkb, lxg, armor) = {
        let catalog = e.catalog();
        let qkb = find(&e, "gj_qiankunbiao");
        let lxg = find(&e, "gj_longxugou");
        let armor = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Armor)
            .expect("防具在目录") as u32;
        (qkb, lxg, armor)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[qkb, lxg, armor]);
    snap.core.players[0].hand.push(qkb);
    snap.core.players[0].hand.push(lxg);
    snap.core.players[1].equips.push(armor);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 龙须钩：使用者自选 1 号装备区一张获得（逐技对照批五 v79）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: lxg,
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
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("龙须钩使用者选装备挂起");
    assert_eq!((c.spec.prompt, c.observer), ("gj_longxu_pick", 0));
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(armor)]);
    let cid = c.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(armor)],
        },
    );
    assert!(e.debug_hand(0).contains(&armor), "使用者获得所选装备");
    assert!(e.save().core.players[1].equips.is_empty(), "目标装备被夺");
    // 乾坤镖：1 号无手牌 → 弃首张装备（若有）；0 号邻座距离全 1
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qkb,
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
    pass_nullify_window(&mut e);
    assert_zone_conservation(&e);

    // —— 神火飞鸦：有闪出闪免伤；无闪 1 火伤；挂起点字节恢复 ——
    let mut e = Engine::new_with_extensions(5224, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (shfy, shan_c, sha_red2) = {
        let catalog = e.catalog();
        let shfy = find(&e, "gj_shenhuofeiya");
        let shan_c = card_of(&e, CardKind::Shan, Suit::Diamond);
        let sha_red2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Heart)
            .expect("红杀在目录") as u32;
        (shfy, shan_c, sha_red2)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[shfy, shan_c, sha_red2]);
    snap.core.players[0].hand.push(shfy);
    snap.core.players[1].hand.push(shan_c);
    snap.core.players[1].hand.push(sha_red2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(1, 9);
    let hp1 = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shfy,
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
    pass_nullify_window(&mut e);
    // 目标（含邻座）有闪 → 出闪询问；先在第一个询问点字节恢复
    let c = e.open_choice().expect("神火飞鸦出闪挂起");
    assert_eq!(c.spec.prompt, "aoe_response");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(shan_c))
            || c.spec.options.contains(&ChoiceOption::Pass)
    );
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("神火飞鸦挂起点字节恢复");
    // 恢复后逐个响应（出闪或 Pass）
    let mut answered = 0;
    while let Some(c) = restored.open_choice() {
        assert_eq!(c.spec.prompt, "aoe_response");
        let cid = c.id;
        let obs = c.observer;
        let sel = if c.spec.options.len() > 1
            && obs == 1
            && c.spec.options.contains(&ChoiceOption::Card(shan_c))
        {
            vec![ChoiceOption::Card(shan_c)]
        } else {
            vec![ChoiceOption::Pass]
        };
        submit(
            &mut restored,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
        answered += 1;
        assert!(answered < 6, "响应循环有界");
    }
    assert!(answered >= 1, "至少一次出闪询问");
    assert_eq!(restored.debug_hp(1), hp1, "1 号出闪免伤");
    assert!(!restored.debug_hand(1).contains(&shan_c), "闪已消耗");
    assert_zone_conservation(&restored);
}

use super::*;

/// 余簇：紫阳丹（+3 护甲+回合衰减）/玉女元神（濒死一次性回复）/
/// 石会粉（需闪否则禁用对其他角色用牌）/六风散（获得两张实体闪）。
#[test]
fn remaining_clusters_batch() {
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

    // —— 紫阳丹：+3 护甲，回合结束阶段 -1 ——
    let mut e = Engine::new_with_extensions(5225, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (zyd, _shan_c) = {
        let _catalog = e.catalog();
        let zyd = find(&e, "gj_ziyangdan");
        let shan_c = card_of(&e, CardKind::Shan, Suit::Diamond);
        (zyd, shan_c)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[zyd]);
    snap.core.players[0].hand.push(zyd);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zyd,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(e.save().core.players[0].hujia, 3, "获得 3 点护甲");
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    if e.open_choice().is_some() {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .take(c.spec.min.max(1))
            .copied()
            .collect();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    assert_eq!(e.save().core.players[0].hujia, 2, "结束阶段衰减 -1");
    assert_zone_conservation(&e);

    // —— 玉女元神：濒死一次性回复 ——
    let mut e = Engine::new_with_extensions(5226, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (yvs, sha_black) = {
        let _catalog = e.catalog();
        let yvs = find(&e, "gj_yunvyuanshen");
        let sha_black = card_of(&e, CardKind::Sha, Suit::Spade);
        (yvs, sha_black)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[yvs, sha_black]);
    snap.core.players[0].hand.push(yvs);
    snap.core.players[1].hand.push(sha_black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_yvs = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_YUNVYUANSHEN_LISTENER_KEY)
        .expect("玉女元神监听技在目录");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yvs,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(e.save().core.skills.contains_key(&(0, def_yvs)));
    e.debug_set_hp(0, 1);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            drop(c);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hp0 = e.debug_hp(0);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_black,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    // 1 点伤害被状态抵消：濒死时回复 1 → 体力回到 1（未死）
    assert_eq!(e.debug_hp(0), hp0, "濒死时状态回复 1 点体力");
    assert!(e.save().core.players[0].alive, "未死亡");
    assert!(
        e.save().core.skills.get(&(0, def_yvs)).is_none(),
        "一次性状态触发即移除"
    );
    assert_zone_conservation(&e);

    // —— 石会粉：无闪 → 禁用对其他角色用牌；有闪 → 出闪免禁 ——
    let mut e = Engine::new_with_extensions(5227, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (shf, shan_c2) = {
        let _catalog = e.catalog();
        let shf = find(&e, "gj_shihuifen");
        let shan_c2 = card_of(&e, CardKind::Shan, Suit::Diamond);
        (shf, shan_c2)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[shf]);
    snap.core.players[0].hand.push(shf);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 1 号无闪 → 直接禁用
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shf,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    assert!(e.open_choice().is_none(), "无闪不挂起，直接置禁用");
    // 1 号（被禁用者）在其出牌阶段无法对其他人用牌——由校验钩子拦截，
    // 交由下个场景验证有闪分支；此处验证守恒
    assert_zone_conservation(&e);

    // 有闪分支：目标出闪免禁用
    let mut e = Engine::new_with_extensions(5228, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[shf, shan_c2]);
    snap.core.players[0].hand.push(shf);
    snap.core.players[1].hand.push(shan_c2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shf,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("石会粉出闪挂起");
    assert_eq!(c.spec.prompt, "respond_shan");
    assert_eq!(c.observer, 1);
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan_c2)],
        },
    );
    // 出闪后无事发生
    assert!(e.open_choice().is_none());
    assert_zone_conservation(&e);

    // —— 六风散：从牌堆获得两张实体闪 ——
    let mut e = Engine::new_with_extensions(5229, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let lfs = find(&e, "gj_liufengsan");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[lfs]);
    snap.core.players[0].hand.push(lfs);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let shans_in_deck_before = e
        .save()
        .core
        .deck
        .iter()
        .filter(|c| e.catalog().cards[**c as usize].kind == CardKind::Shan)
        .count();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: lfs,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    let gained = e.debug_hand(1).len();
    let expected = shans_in_deck_before.min(2);
    assert_eq!(gained, expected, "获得至多两张实体闪");
    let shans_in_deck_after = e
        .save()
        .core
        .deck
        .iter()
        .filter(|c| e.catalog().cards[**c as usize].kind == CardKind::Shan)
        .count();
    assert_eq!(
        shans_in_deck_before - shans_in_deck_after,
        expected,
        "牌堆闪被移出"
    );
    assert_zone_conservation(&e);
}

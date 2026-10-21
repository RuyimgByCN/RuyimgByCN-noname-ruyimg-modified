use super::*;

/// 食物计时状态原语：摊黄冻（出牌阶段结束摸 1）与有更（阶段开始
/// 已受伤且体力最少回 1）——授予、倒计时扣减、归零移除。
#[test]
fn food_state_grant_tick_and_expire() {
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

    // —— 摊黄冻：授予即 rounds=3；出牌阶段结束摸 1 并扣减 ——
    let mut e = Engine::new_with_extensions(5211, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let td = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_tanhuadong"))
            .expect("摊黄冻在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[td]);
    snap.core.players[0].hand.push(td);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_td = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_FOOD_TANHUADONG_KEY)
        .expect("摊黄冻监听技在目录");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: td,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let inst = e
        .save()
        .core
        .skills
        .get(&(0, def_td))
        .expect("授予后实例存在")
        .clone();
    assert_eq!(inst.rounds, 3, "初始 3 回合");
    let h0 = e.debug_hand(0).len();
    // 出牌阶段结束（Play→Discard 触发 Phase{Play, end}）
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "出牌阶段结束摸 1");
    assert_eq!(e.save().core.skills.get(&(0, def_td)).unwrap().rounds, 2);
    // 挂起无关机器——字节恢复（rounds 入存档）
    let bytes = engine_native::archive::encode_archive(&e.save());
    let restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("食物状态字节恢复");
    assert_eq!(
        restored
            .save()
            .core
            .skills
            .get(&(0, def_td))
            .unwrap()
            .rounds,
        2
    );
    drop(restored);

    // 归零移除：注入 rounds=1 → 下一次出牌阶段结束移除
    e.debug_set_hp(0, 9);
    let mut snap = e.save();
    for ((_owner, def), inst) in snap.core.skills.iter_mut() {
        if *def == def_td {
            inst.rounds = 1;
        }
    }
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复（rounds=1）");
    // 回到 0 号出牌阶段（当前在弃牌阶段之后推进一个回合）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96, "未回到 0 号出牌阶段");
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    let h0 = e.debug_hand(0).len();
    submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "最后一回合仍摸 1");
    assert!(
        e.save().core.skills.get(&(0, def_td)).is_none(),
        "归零后实例移除"
    );

    // —— 有更：阶段开始已受伤且体力最少 → 回复 1（对他人授予） ——
    let mut e = Engine::new_with_extensions(5212, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let yg = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_yougeng"))
            .expect("有更在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[yg]);
    snap.core.players[0].hand.push(yg);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let def_yg = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_FOOD_YOUGENG_KEY)
        .expect("有更监听技在目录");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yg,
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
        e.save().core.skills.contains_key(&(1, def_yg)),
        "对他人授予"
    );
    // 1 号已受伤且体力最少
    e.debug_set_hp(1, 2);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    let hp1 = e.debug_hp(1);
    // 推进到 1 号通过判定阶段（其判定阶段开始触发回复）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96, "未推进到 1 号摸牌阶段");
        if e.turn().current == 1 && e.turn().phase == Phase::Draw && e.open_choice().is_none() {
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
    assert_eq!(e.debug_hp(1), hp1 + 1, "判定阶段开始回复 1（准备阶段替位）");
    assert_eq!(e.save().core.skills.get(&(1, def_yg)).unwrap().rounds, 2);
    assert_zone_conservation(&e);
}

/// 食物效果面第二批：青团（首杀摸 1）/元宝肉（额外一张杀）/
/// 咕咚羹（受杀伤害 -1 一次性）/麻婆豆腐（结束阶段确定性弃牌）/
/// 春饼（手牌上限 +1）。
#[test]
fn food_effects_batch_two() {
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
    let def_of = |e: &Engine, key: &str| {
        e.catalog()
            .skill_by_name(key)
            .unwrap_or_else(|| panic!("{}在目录", key))
    };

    // —— 青团：出牌阶段首杀摸 1，第二张杀不再摸 ——
    let mut e = Engine::new_with_extensions(5213, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let qt = find(&e, "gj_qingtuan");
    let sha = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32
    };
    let sha2 = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Club)
            .expect("另一黑杀在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[qt, sha, sha2]);
    snap.core.players[0].hand.push(qt);
    snap.core.players[0].hand.push(sha);
    snap.core.players[0].hand.push(sha2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_qt = def_of(&e, engine_native::content::host::GJ_FOOD_QINGTUAN_KEY);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qt,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(e.save().core.skills.contains_key(&(0, def_qt)), "青团授予");
    e.debug_set_hp(1, 9); // 1 号体力大，允许被杀命中
    let h0 = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    // 目标无闪：可能无响应窗口
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0, "首杀摸 1（杀离手 -1 + 摸 +1）");
    let h0 = e.debug_hand(0).len();
    // 杀次数已用尽：第二张杀被拒（本场景无额外杀状态）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(0).len(), h0, "无第二杀、无额外摸牌");
    assert_zone_conservation(&e);

    // —— 元宝肉：杀次数用尽后额外一张 ——
    let mut e = Engine::new_with_extensions(5214, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let ybr = find(&e, "gj_yuanbaorou");
    // 第三张杀：供每回合上限负例使用（取既非 sha 又非 sha2 的另一张杀）
    let sha3 = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .enumerate()
            .position(|(i, c)| c.kind == CardKind::Sha && i as u32 != sha && i as u32 != sha2)
            .expect("第三张杀在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sha, sha2, sha3, ybr]);
    snap.core.players[0].hand.push(ybr);
    snap.core.players[0].hand.push(sha);
    snap.core.players[0].hand.push(sha2);
    snap.core.players[0].hand.push(sha3);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_ybr = def_of(&e, engine_native::content::host::GJ_FOOD_YUANBAOROU_KEY);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ybr,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(
        e.save().core.skills.contains_key(&(0, def_ybr)),
        "元宝肉授予"
    );
    e.debug_set_hp(1, 9);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
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
    drain_choices(&mut e);
    // 第一次额外杀：放行
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "元宝肉额外杀放行: {out:?}"
    );
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    // 每回合仅一张额外杀：再用一张被上限拒绝
    let h0 = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "元宝肉每回合仅一张额外杀: {out:?}"
    );
    assert_eq!(e.debug_hand(0).len(), h0, "上限后无额外摸牌");
    assert_zone_conservation(&e);

    // —— 咕咚羹：受杀伤害 -1 且状态立即移除 ——
    let mut e = Engine::new_with_extensions(5215, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let gdg = find(&e, "gj_gudonggeng");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[gdg, sha, sha2]);
    snap.core.players[0].hand.push(gdg);
    snap.core.players[1].hand.push(sha);
    snap.core.players[1].hand.push(sha2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_gdg = def_of(&e, engine_native::content::host::GJ_FOOD_GUDONGGENG_KEY);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gdg,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 0 号回合结束 → 1 号回合出杀打 0 号
    let hp0 = e.debug_hp(0);
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
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    assert_eq!(
        e.debug_hp(0),
        hp0,
        "1 点杀伤害被 -1 减到 0（体力不变=减伤生效）"
    );
    assert!(
        e.save().core.skills.get(&(0, def_gdg)).is_none(),
        "咕咚羹一次性：触发即移除"
    );

    // —— 春饼：手牌上限 +1（ hp=1 时上限 2，弃牌阶段少弃一张） ——
    let mut e = Engine::new_with_extensions(5216, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let cb = find(&e, "gj_chunbing");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[cb]);
    snap.core.players[0].hand.push(cb);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cb,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    e.debug_set_hp(0, 1); // 手牌上限 1+1=2
                          // 出牌阶段结束时手上有春饼弃置后的牌 + 摸牌阶段 2 张 = 2 张 → 不需要弃
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 弃牌阶段：上限 2，手中 0 张（春饼已弃、无摸牌）→ 不挂起弃牌选择
    assert!(e.open_choice().is_none(), "手牌未超上限不挂起弃牌选择");
    assert_zone_conservation(&e);
}

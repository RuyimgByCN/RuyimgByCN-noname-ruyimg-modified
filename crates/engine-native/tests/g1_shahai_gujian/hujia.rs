use super::*;

/// 护甲原语 + 食物第三批：鲤鱼汤（护甲）/麻茶（黑色目标限制）/
/// 菱角（摸牌阶段 +1 弃 1，选择机含挂起派生）。
#[test]
fn hujia_primitive_and_food_batch_three() {
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
    let _find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 麻茶：黑色牌不能指定持有者为目标；红色牌不受限 ——
    let mut e = Engine::new_with_extensions(5217, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (mc, sha_black, sha_red) = {
        let catalog = e.catalog();
        let mc = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_molicha"))
            .expect("麻茶在目录") as u32;
        let sha_black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let sha_red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Heart)
            .expect("红杀在目录") as u32;
        (mc, sha_black, sha_red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[mc, sha_black, sha_red]);
    snap.core.players[0].hand.push(mc);
    snap.core.players[0].hand.push(sha_red);
    snap.core.players[0].hand.push(sha_black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 麻茶喂给 1 号
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: mc,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 黑杀指定持有者（其他玩家回合内使用路径）→ 出牌校验拒绝
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_black,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "黑色牌指定持有者应被拒: {out:?}"
    );
    // 红杀指定持有者 → 不受限（走完整杀结算）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_red,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "红色牌不受限: {out:?}"
    );
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    assert_zone_conservation(&e);

    // —— 鲤鱼汤：体力最少（之一）阶段开始得 1 护甲；护甲抵扣伤害 ——
    let mut e = Engine::new_with_extensions(5218, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (lyt, black, red) = {
        let catalog = e.catalog();
        let lyt = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_liyutang"))
            .expect("鲤鱼汤在目录") as u32;
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
        (lyt, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[lyt, black, red]);
    snap.core.players[0].hand.push(lyt);
    snap.core.players[1].hand.push(black);
    snap.core.players[2].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: lyt,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(e.save().core.players[0].hujia, 0, "授予时无护甲");
    // 0 号体力 3（全场并列最少）→ 结束阶段 +1 护甲；先受伤验证抵扣
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    // 推进到弃牌阶段前结束阶段？直接 NextPhase：Play→Discard→End
    submit(&mut e, 0, GameAction::NextPhase); // Play 结束
                                              // 弃牌阶段（手牌空，无弃牌选择）
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
    submit(&mut e, 0, GameAction::NextPhase); // Discard 结束 → End 开始
    // 第 991 轮：上游子技 `liyutang` 是 `trigger:{player:'phaseEnd'}`（**每个阶段结束**）+ 「剩余 2 次」
    // ⇒ 本回合 Play 结束与 Discard 结束各授予 1 点护甲，合计 2 点（此前钉在 End 开始只授 1 点）。
    assert_eq!(
        e.save().core.players[0].hujia,
        2,
        "体力最少：本回合两个阶段结束各得 1 护甲（共 2）"
    );
    // 护甲抵扣：1 号回合出杀打 0 号 → 护甲 -1、体力不变
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
            card: black,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    // 第 991 轮：本回合共得 2 点护甲，一次 1 点伤害只消耗 1 点 ⇒ 剩 1 点
    assert_eq!(e.save().core.players[0].hujia, 1, "护甲被消耗 1 点（2→1）");
    assert_eq!(e.debug_hp(0), hp0, "护甲抵扣后体力不变");
    assert_zone_conservation(&e);

    // —— 菱角：摸牌阶段额外摸 1 后弃 1（选择机 + 字节恢复） ——
    let mut e = Engine::new_with_extensions(5219, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (xj, black, red) = {
        let catalog = e.catalog();
        let xj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_xiajiao"))
            .expect("菱角在目录") as u32;
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
        (xj, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[xj, black, red]);
    snap.core.players[0].hand.push(xj);
    snap.core.players[1].hand.push(black);
    snap.core.players[1].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xj,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 推进到 1 号回合结束 → 0 号新回合：Judge→Draw（固定摸 2 + 菱角摸 1 → 弃 1 挂起）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
        if e.turn().current == 0 && e.turn().phase == Phase::Draw && e.open_choice().is_some() {
            break;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Draw && e.open_choice().is_none() {
            // 固定摸牌刚完成；菱角触发在 AfterTriggers——推进一次或等挂起
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
    let c = e.open_choice().expect("菱角弃牌挂起");
    assert_eq!(c.spec.prompt, "gj_xiajiao_discard");
    assert_eq!(c.observer, 0);
    assert_eq!(c.spec.min, 1);
    // 挂起点字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("菱角挂起点字节恢复");
    let cid = restored.open_choice().unwrap().id;
    let h0 = restored.debug_hand(0).len();
    let discard_card = restored.debug_hand(0)[0];
    submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(discard_card)],
        },
    );
    assert_eq!(
        restored.debug_hand(0).len(),
        h0 - 1,
        "额外摸 1 后弃 1（净 +1）"
    );
    assert_zone_conservation(&restored);
}

use super::*;

/// 指路小虎（swd 机关牌首切片）：自目标使用→无懈窗 Pass→随机对一
/// 名其他存活角色虚拟杀（3 人局候选唯一）→闪 Pass→-1；牌入弃堆。
#[test]
fn swd_zhiluxiaohu_random_enemy_virtual_sha() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10065, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    // 指路小虎：机关牌种唯一实体（loader 卡名带包前缀重写，按 kind 取）
    let zhhu = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Jiguan)
        .expect("指路小虎在目录") as u32;
    let mut snap = e.save();
    // 全域摘除（批①对齐后 zhiluxiaohu/mujiaren 同为 Jiguan，首实体可能
    // 已在发牌手牌中——仅 deck.retain 会造成双区）
    snap.core.deck.retain(|c| *c != zhhu);
    snap.core.discard.retain(|c| *c != zhhu);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != zhhu);
        pl.equips.retain(|c| *c != zhhu);
    }
    snap.core.players[0].hand.push(zhhu);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    assert_eq!(e.catalog().card_kind(zhhu), CardKind::Jiguan, "机关牌种");

    // 自目标使用（targets=[0]）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhhu,
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
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 随机敌唯一候选链：seed 10065 下恰一名敌 -1（另一敌不动）
    let (h1, h2) = (e.debug_hp(1), e.debug_hp(2));
    assert!(
        (h1 == 3 && h2 == 4) || (h1 == 4 && h2 == 3),
        "恰一名敌被虚拟杀 -1: ({h1},{h2})"
    );
    assert!(e.state_core().discard.contains(&zhhu), "指路小虎入弃堆");
}

/// 木甲人（机关牌第二切片）：弃全部非基本手牌→随机机关牌 1+N 入手
///（实体发现不足给余量）；usable:1 同回合再使用拒 CardLimit。
#[test]
fn swd_mujiaren_discard_nonbasic_discover_jiguan() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10066, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let mj = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("mujiaren"))
        .expect("木甲人在目录") as u32;
    let sha = e.debug_place(0, "sha");
    let wuzhong = e.debug_place(0, "wuzhong");
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != mj);
    snap.core.discard.retain(|c| *c != mj);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != mj);
    }
    snap.core.players[0].hand.insert(0, mj);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");

    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: mj,
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
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 断言：弃 wuzhong（非基本）留 sha；发现机关牌 1+N=2 张（池=小虎
    // +勾魂锣×2 共 3 张候选随机取 2）；木甲人本体入弃堆
    let hand = e.debug_hand(0);
    assert!(
        hand.contains(&sha) && !hand.contains(&wuzhong),
        "弃非基本留基本"
    );
    assert_eq!(hand.len(), 3, "弃 1 获 2（池 3 张候选=seed 10066 确定性）");
    let gained: Vec<_> = hand.iter().filter(|c| **c != sha).copied().collect();
    assert_eq!(gained.len(), 2, "获 2 张");
    for c in &gained {
        assert_eq!(
            e.catalog().card_kind(*c),
            CardKind::Jiguan,
            "获牌均为机关牌"
        );
    }
    assert!(e.state_core().discard.contains(&mj), "木甲人本体入弃堆");
    // usable:1：同回合再使用拒 CardLimit
    let hand0 = e.debug_hand(0);
    let mj2 = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| c.name.contains("mujiaren") && !hand0.contains(&(*i as u32)))
        .map(|(i, _)| i as u32)
        .expect("第二张木甲人");
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != mj2);
    snap.core.discard.retain(|c| *c != mj2);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != mj2);
    }
    snap.core.players[0].hand.push(mj2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入2");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: mj2,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::CardLimit)
        ),
        "{out:?}"
    );
}

/// 勾魂锣（机关牌第三切片）：挂 3 轮倒计时（来源回合开始逐阶段-1）
/// →归 0 失去 1 体力+弃全部手牌+清挂附；重复挂同目标拒。
#[test]
fn swd_gouhunluo_countdown_lose_hp_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10067, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let ghl = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("gouhunluo"))
        .expect("勾魂锣在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != ghl);
    snap.core.discard.retain(|c| *c != ghl);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != ghl);
    }
    snap.core.players[0].hand.push(ghl);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");

    // 挂 1 号
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ghl,
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
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.state_core().player(1).gouhunluo_rounds, 3, "倒计时 3");
    assert_eq!(e.state_core().player(1).gouhunluo_source, Some(0), "来源=0");
    let sub = e.catalog().skill_by_name("swd_gouhunluo_sub").expect("def");
    assert!(
        e.state_core().skills.contains_key(&(1, sub)),
        "1 号获倒计时实例"
    );

    // 重复挂同目标拒
    let ghl2 = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| c.name.contains("gouhunluo") && *i as u32 != ghl)
        .map(|(i, _)| i as u32)
        .expect("第二张勾魂锣");
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != ghl2);
    snap.core.discard.retain(|c| *c != ghl2);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != ghl2);
    }
    snap.core.players[0].hand.push(ghl2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入2");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ghl2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "{out:?}"
    );
    e.state_core();

    // 倒计时导航：循环推进（当前玩家 NP+排空询问）直到挂附者计数归 0
    for _ in 0..60 {
        if e.state_core().player(1).gouhunluo_rounds == 0 {
            break;
        }
        while let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 归 0 结算：1 号失去 1 体力（4→3）+手牌清+挂附清
    assert_eq!(e.debug_hp(1), 3, "倒计时归 0 失去 1 体力（4→3）");
    assert_eq!(e.debug_hand(1).len(), 0, "弃全部手牌");
    assert_eq!(e.state_core().player(1).gouhunluo_rounds, 0, "计数清");
    assert_eq!(e.state_core().player(1).gouhunluo_source, None, "来源清");
    assert!(
        !e.state_core().skills.contains_key(&(1, sub)),
        "倒计时实例清"
    );
}

/// 机关桶（机关牌第四切片）：AOE 逐目标「弃一张手牌或受 1 火伤」；
/// 无手牌目标直接火伤；取消=受火伤；本体入弃堆。
#[test]
fn swd_jiguantong_aoe_discard_or_fire() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10068, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let jgt = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguantong"))
        .expect("机关桶在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != jgt);
    snap.core.discard.retain(|c| *c != jgt);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != jgt);
    }
    snap.core.players[0].hand.push(jgt);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    let _shan = e.debug_place(0, "shan");
    // 1 号持牌（弃牌路径）、2 号空手（直接火伤路径）
    let h1 = e.debug_place(1, "tao");
    let _ = e.debug_place(0, "sha");

    // AOE 使用：targets=全部其他存活（1、2）→ 逆序结算（v193）：先 2 后 1
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jgt,
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
    // 询问流：2 号空手直接火伤（无询问）→ 1 号弃牌询问（选 tao 弃）
    while let Some(c) = e.open_choice() {
        let picks: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .copied()
            .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == h1))
            .collect();
        let (cid, obs) = (c.id, c.observer);
        let sel = if picks.len() == 1 {
            picks
        } else {
            vec![ChoiceOption::Pass]
        };
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    // 断言：1 号弃 tao 不掉血；2 号空手受 1 火伤（4→3）；本体入弃堆；
    // 有人受伤 → contentAfter 不摸（使用者手牌保持 shan+sha 两张）
    assert_eq!(e.debug_hp(1), 4, "1 号弃牌免伤");
    assert!(!e.debug_hand(1).contains(&h1), "1 号 tao 已弃");
    assert_eq!(e.debug_hp(2), 3, "2 号空手受 1 火伤（4→3）");
    assert_eq!(e.debug_hand(0).len(), 2, "有人受伤：contentAfter 不摸牌");
    assert!(e.state_core().discard.contains(&jgt), "机关桶本体入弃堆");
    assert!(e.state_core().discard.contains(&h1), "tao 入弃堆");
}

/// 机关桶 v193 补齐两支：逆序结算（上游 reverseOrder——targets 座位序
/// 反转，先问 2 号）+ contentAfter（全员弃牌无人受伤 → 使用者摸 1）。
#[test]
fn swd_jiguantong_reverse_order_and_content_after_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10069, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let jgt = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguantong"))
        .expect("机关桶在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != jgt);
    snap.core.discard.retain(|c| *c != jgt);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|c| *c != jgt);
    }
    snap.core.players[0].hand.push(jgt);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    let _shan = e.debug_place(0, "shan");
    let _ = e.debug_place(0, "sha");
    // 1、2 号各持一张（全员走弃牌路径）
    let h1 = e.debug_place(1, "tao");
    let h2 = e.debug_place(2, "tao");

    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jgt,
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
    // 逆序：首个弃牌询问 observer=2（座位序反转）；随后 observer=1
    let c = e.open_choice().expect("首个弃牌询问");
    assert_eq!(c.observer, 2, "逆序结算：先问 2 号（上游 reverseOrder）");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(h2)],
        },
    );
    let c = e.open_choice().expect("次个弃牌询问");
    assert_eq!(c.observer, 1, "逆序结算：后问 1 号");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(h1)],
        },
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 断言：无人受伤（全员弃牌免伤）；contentAfter → 使用者摸 1
    //（shan+sha 两张 + 摸 1 = 3）；两张弃牌与本体入弃堆
    assert_eq!(e.debug_hp(1), 4, "1 号弃牌免伤");
    assert_eq!(e.debug_hp(2), 4, "2 号弃牌免伤");
    assert_eq!(
        e.debug_hand(0).len(),
        3,
        "无人受伤：contentAfter 使用者摸 1"
    );
    assert!(e.state_core().discard.contains(&jgt), "机关桶本体入弃堆");
    assert!(
        e.state_core().discard.contains(&h1) && e.state_core().discard.contains(&h2),
        "两张弃牌入弃堆"
    );
}

use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十五批切片（规则 v166）：柳梦璃（pal_liumengli，shu
// 3 女）润心（runxin，CardUsed+CardResponded 双事件 direct）——使用/
// 打出红桃牌后令一名已受伤角色回复 1。tianxian（mod 系+逐目标判定取
// 消）/xjzhimeng（暗牌标记）缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 润心：使用红桃→选受伤者回复 1；打出红桃（闪响应）同触发；非红桃
/// 不触发；取消=放弃。
#[test]
fn liumengli_runxin_heart_use_and_respond() {
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

    let mut e = Engine::new_with_extensions(10035, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "runxin");

    // ① 使用红桃牌（无中生有不限体力；红桃由布阵循环保证）
    let heart_card = loop {
        let c = e.debug_place(0, "wuzhong");
        if e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Heart {
            break c;
        }
        e.debug_clear_hand(0);
    };
    e.debug_set_hp(1, 3); // 受伤者
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: heart_card,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 吃桃回自己的询问可能先挂（无——自己满血 4 桃自回满血无窗口）；润心询问
    let c = e.open_choice().expect("润心目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_RUNXIN_TARGET
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hp(1), 4, "润心回复（3→4）");

    // ② 打出红桃闪（响应杀）同触发：0 号过回合→1 号回合内杀 0 号，
    // 0 号红桃闪响应
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 3); // 再压伤
    let shan = loop {
        let c = e.debug_place(0, "shan");
        if e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Heart {
            break c;
        }
        e.debug_clear_hand(0);
    };
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(1, "sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![0],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 0 号闪响应（红桃）→ 润心询问
    let out = answer_open(&mut e, vec![ChoiceOption::Card(shan)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("润心目标挂起②");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_RUNXIN_TARGET
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hp(1), 4, "打出红桃闪：润心回复（3→4）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第二十六批切片（规则 v167）：柳梦璃织梦（xjzhimeng，
// phaseEnd direct 可放弃）+织梦·梦（xjzhimeng2，CardUsed frequent
// silent）——牌堆顶一张暗置于其他角色武将牌上（off_court）+runtime 授
// 梦实例；暗牌持有者被使用同类型（三分类）牌时摸 1（暗牌保留）。柳梦
// 璃升全闭环（tianxian 仍暂缓）。
// ---------------------------------------------------------------------------

/// 织梦：暗置+授技；织梦·梦：同类型牌被使用摸 1；异类型不摸。
#[test]
fn liumengli_xjzhimeng_darkmark_and_draw() {
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

    let mut e = Engine::new_with_extensions(10036, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xjzhimeng");

    // ① 0 号阶段末：织梦询问 → 选 1 号 → 牌堆顶暗置其离场区+授梦实例
    let deck_top = *e.state_core().deck.last().expect("牌堆非空");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：织梦询问
    let c = e.open_choice().expect("织梦目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_ZHIMENG_TARGET
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(
        e.state_core().player(1).off_court,
        vec![deck_top],
        "牌堆顶暗置 1 号离场区"
    );
    let zm2_def = e
        .catalog()
        .skill_by_name("xjzhimeng2")
        .expect("xjzhimeng2 def");
    assert!(
        e.state_core().skills.contains_key(&(1, zm2_def)),
        "runtime 授梦实例"
    );
    let dark_kind = e.catalog().card_kind(deck_top);
    let dark_class = if dark_kind.equip_slot().is_some() {
        2
    } else {
        match dark_kind {
            CardKind::Sha
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::Shan
            | CardKind::Tao
            | CardKind::Jiu => 0,
            _ => 1,
        }
    };

    // ② 1 号自己使用锦囊（无中）→ 目标=自己 → 织梦·梦按类型判摸
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let hand_before = e.debug_hand(1).len();
    let wz = loop {
        let c = e.debug_place(1, "wuzhong");
        let k = e.catalog().card_kind(c);
        if !k.equip_slot().is_some()
            && !matches!(
                k,
                CardKind::Sha
                    | CardKind::FireSha
                    | CardKind::LeiSha
                    | CardKind::Shan
                    | CardKind::Tao
                    | CardKind::Jiu
            )
        {
            break c;
        }
        e.debug_clear_hand(1);
    };
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wz,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 无中结算（1 号摸 2）+织梦·梦（同类型再摸 1）
    let wz_class = 1; // 锦囊
    if dark_class == wz_class {
        assert_eq!(e.debug_hand(1).len(), hand_before + 3, "无中 2+织梦·梦 1");
    } else {
        assert_eq!(e.debug_hand(1).len(), hand_before + 2, "异类型：仅无中 2");
    }
}

// ---------------------------------------------------------------------------
// U05 xianjian 第二十九批（规则 v170）：死亡前置窗口基建（Death 帧
// BeforeTriggers 起步=TurnOverBefore 同款）+ 柳梦璃长念（changnian，
// Death 窗口遗言）+长念·思（changnian2，runtime 授技结束阶段摸 1）
// ——柳梦璃 v170 时点全闭环（仅 tianxian 暂缓）。
// ---------------------------------------------------------------------------

/// 长念：死亡时遗言移交 hej 全部牌+授长念·思；目标下个结束阶段摸 1；
/// 死亡照常执行。
#[test]
fn liumengli_changnian_death_bequest() {
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

    let mut e = Engine::new_with_extensions(10040, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    // changnian 挂柳梦璃——映射后目录名即 changnian；grant 校验
    assert!(
        e.catalog().skill_by_name("changnian").is_some(),
        "changnian 在目录"
    );
    e.debug_grant(0, "changnian");
    // 0 号（将死）手牌 2 张；1 号（遗言对象）空手
    let h1 = e.debug_place(0, "tao");
    let h2 = e.debug_place(0, "wuzhong");
    // 0 号过回合（死亡后 current 停留在死亡者会卡推进——改在 1 号回合
    // 内击杀）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);

    // 1 号回合内伤 0 号 2 → 死亡 → 长念遗言询问 → 选 1 号
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    // 先走完濒死求桃链（仅 Pass 求桃询问）→ 死亡帧 → 长念遗言询问
    loop {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt != "dying_save" {
            break; // 长念询问到达
        }
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
    let c = e.open_choice().expect("长念遗言询问挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_CHANGNIAN_TARGET
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // 移交断言：1 号获 2 张；0 号已死亡手牌清空
    assert!(
        e.debug_hand(1).contains(&h1) && e.debug_hand(1).contains(&h2),
        "遗言移交"
    );
    assert!(!e.debug_alive(0), "遗言后死亡照常");
    // runtime 授长念·思实例
    let cn2 = e.catalog().skill_by_name("changnian2").expect("def");
    assert!(
        e.state_core().skills.contains_key(&(1, cn2)),
        "授长念·思实例"
    );

    // 1 号（当前回合）收束：清手 → End 末长念·思摸 1
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：长念·思摸 1
    assert_eq!(e.debug_hand(1).len(), 1, "长念·思摸 1（清手后 +1）");
}

/// 天弦：锁定三件——杀无视距离（距离 2 仍可指定）+目标数上限无限
///（双目标合法）+逐目标判定黑取消该目标结算（红=正常结算走闪询问）。
#[test]
fn liumengli_tianxian_range_targets_and_judge_cancel() {
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

    let mut e = Engine::new_with_extensions(10057, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
        if p == 2 || p == 3 {
            e.debug_set_hp(p, 4);
        }
    }

    // 无技：距离 2（0→2/3 均隔一位）拒绝；双目标拒绝
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "{out:?}"
    );
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2, 3],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "{out:?}"
    );

    // 天弦：距离 2 双目标合法；判定序=2 号红（正常结算）→3 号黑（取消）
    e.debug_grant(0, "tianxian");
    // 牌库顶叠两张：后推先抽——首判（2 号）=红，次判（3 号）=黑
    let _ = e.debug_set_deck_top_by(|cd| {
        matches!(
            cd.suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        )
    });
    let _ = e.debug_set_deck_top_by(|cd| {
        matches!(
            cd.suit,
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
        )
    });
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2, 3],
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
    assert_eq!(e.debug_hp(2), 3, "2 号判红正常结算（4→3，闪 Pass）");
    assert_eq!(e.debug_hp(3), 4, "3 号判黑取消结算（不受伤不问闪）");
    assert!(!e.debug_hand(0).contains(&sha), "杀已结算离手");
}

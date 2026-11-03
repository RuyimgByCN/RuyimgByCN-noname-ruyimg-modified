use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十（规则 v127）：美（ow_mei）——暴雪（baoxue，limited：
// 弃全部黑色 he 牌+逐目标随机弃 1+翻面+自身翻面）与急冻（jidong，
// hp=1 任意阶段结束：翻面+回复 2+冰封）双技；bingqiang 冰墙登记暂缓。
// 基建：翻面角色回合跳过（Judge begin 翻回+整回合无阶段帧）+冰封双门
// 控（commit_damage 无效化 / PlayCard 目标拒绝）。存档 v7→v8。
// ---------------------------------------------------------------------------

/// 暴雪：整体放弃（空选择）不消耗 limited；发动后弃全部黑色 he 牌、
/// 逐目标随机弃 1+翻至背面、自己翻至背面；每局限一次（再发动拒绝）。
#[test]
fn mei_baoxue_limited_discard_and_flips() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9911, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "baoxue");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "sha");
    let c3 = e.debug_place(0, "tao");
    let is_black = |e: &Engine, c: u32| {
        matches!(
            e.catalog().cards[c as usize].suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        )
    };
    let black = [c1, c2, c3].iter().filter(|c| is_black(&e, **c)).count();
    assert!(black >= 1, "种子 9911 应给出至少 1 张黑牌");
    let _c_t1 = e.debug_place(1, "shan");
    let _c_t2 = e.debug_place(2, "shan");

    let def_bx = e.catalog().skill_by_name("baoxue").expect("def");

    // 整体放弃（空选择）：limited 未消耗、无效果
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bx });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, Vec::new());
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(!e.state_core().player(0).baoxue_used, "放弃不消耗 limited");
    assert!(!e.debug_face_down(0), "放弃不翻面");

    // 发动：目标选项=其他存活者，上限=黑色牌数
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bx });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let ask = e.open_choice().cloned().expect("暴雪目标询问");
    assert_eq!(ask.spec.max, black, "目标上限=黑色 he 牌数");
    drop(ask);
    // 按实际上限选目标（黑牌数可能为 1）
    let mut sel = vec![ChoiceOption::Player(1)];
    if black >= 2 {
        sel.push(ChoiceOption::Player(2));
    }
    let out = answer_open(&mut e, sel);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );

    assert!(e.state_core().player(0).baoxue_used, "limited 已消耗");
    assert_eq!(e.debug_hand(0).len(), 3 - black, "弃全部黑色 he 牌");
    assert!(
        is_black_card_discarded(&e, &[c1, c2, c3], black),
        "黑牌入弃牌堆"
    );
    assert_eq!(e.debug_hand(1).len(), 0, "目标 1 随机弃 1（牌池仅 1 张）");
    assert!(e.debug_face_down(1), "目标 1 翻至背面");
    if black >= 2 {
        assert_eq!(e.debug_hand(2).len(), 0, "目标 2 随机弃 1");
        assert!(e.debug_face_down(2), "目标 2 翻至背面");
    }
    assert!(e.debug_face_down(0), "自己翻至背面");

    // 再发动拒绝（limited 每局一次）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bx });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
        ),
        "limited 后应拒绝：{out:?}"
    );

    // 存档往返（v8）：limited 标记与翻面状态随存档保留
    let snap = e.save();
    let e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(
        e2.state_core().player(0).baoxue_used,
        "limited 标记随存档保留"
    );
    assert!(e2.debug_face_down(0), "翻面状态随存档保留");
}

/// 翻面角色回合跳过：轮至翻面者——先经翻面前置窗口翻回，整回合无
/// 阶段帧（不摸牌），回合槽照常消耗（下家照常开局）。
#[test]
fn mei_flip_face_down_turn_skipped_and_flipped_back() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9912, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_set_face_down(1, true);

    // 0 收束回合 → 1 的回合被跳过（翻回+无摸牌）→ 2 开局
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End→回合收束
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(!e.debug_face_down(1), "跳过回合时翻回正面");
    assert_eq!(e.debug_hand(1).len(), 0, "跳过回合不摸牌");
    let t = e.turn();
    assert_eq!(t.current, 2, "回合槽消耗：轮至 2");
    assert_eq!(t.phase, Phase::Judge, "2 号正常开局");
}

/// 急冻：hp=1 时任意阶段结束——翻面+回复 2+冰封；冰封期间①其他角色
/// 不能以牌指定其为目标（IllegalTargets）②AOE 伤害无效（万箭照常问
/// 闪，伤害被无效化）；翻回（跳过回合）后解除。
#[test]
fn mei_jidong_freeze_damage_immune_and_untargetable() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9913, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "jidong");
    e.debug_set_hp(0, 1);

    // 0 出牌→弃牌→End。A2 H78：急冻已迁整回合结束窗 phaseEnd 段（上游
    // global phaseEnd＝任意玩家整回合结束）——0 号整回合结束时（hp=1）
    // 触发：翻面+回复 2+冰封，随后回合交接轮至 1（翻面在回合末＝下一
    // 轮到 0 时跳过）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End→整回合结束窗→急冻→交接
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.debug_face_down(0), "急冻翻面");
    assert_eq!(e.debug_hp(0), 3, "回复 2（1→3）");
    assert!(e.state_core().player(0).jidong_active, "冰封中");
    let t = e.turn();
    assert_eq!(t.current, 1, "整回合结束窗后交接轮至 1");

    // 1 回合：Judge→Draw→Play
    advance_loop(&mut e, 1);
    let sha = e.debug_place(1, "sha");
    // ①冰封者不可被指定为目标
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "冰封者不可被指定：{out:?}"
    );
    // ②AOE（万箭齐发）不询问 targetEnabled——冰封者被问闪（无闪），
    // 伤害被无效化；他人照常受伤
    let wanjian = e.debug_place(1, "archery");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wanjian,
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
    assert_eq!(e.debug_hp(0), 3, "冰封：万箭伤害无效");
    assert_eq!(e.debug_hp(2), 3, "他人照常受 1（4→3）");

    // 1 收束→2/3/4 回合→0 的回合被跳过并翻回→冰封解除（驱动至 0 跳
    // 过完成、1 开局）
    for _ in 0..200 {
        let t = e.turn();
        if t.current == 1 && t.phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    assert!(!e.debug_face_down(0), "跳过回合翻回正面");
    assert!(!e.state_core().player(0).jidong_active, "翻回即解除冰封");
}

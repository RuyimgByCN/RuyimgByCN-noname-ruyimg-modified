use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十二切片（规则 v163）：小蛮（pal_xiaoman，shu 3 女）
// 灵泉（lingquan，phaseEnd forced juexingji：轮次≥3+全局用牌数>体力→
// 摸 3+runtime 授水云+立即置牌一次）+神雾（shenwu，global phaseEnd
// forced juexingji：水云计数≥3→上限+1 回复 1+runtime 授回梦）。
// anwugu 与阿奴同技 mapping 复用。
// ---------------------------------------------------------------------------

/// 灵泉：round≥3 且用牌数>体力时觉醒——摸 3+授水云+立即置牌。
#[test]
fn xiaoman_lingquan_awaken_grants_shuiyun() {
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

    let mut e = Engine::new_with_extensions(10030, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_set_hp(0, 1); // 用牌门槛降为 2
    e.debug_grant(0, "lingquan");

    // r1：0 号用两张无中（total_used=2>1）
    let w1 = e.debug_place(0, "wuzhong");
    let w2 = e.debug_place(0, "wuzhong");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: w1,
            targets: vec![],
        },
    );
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: w2,
            targets: vec![],
        },
    );
    assert_eq!(e.state_core().player(0).total_used, 2, "全局用牌计数");
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
                                                      // r1 0 号 End：round 1 <3 → 不觉醒
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(
        !e.state_core().player(0).lingquan_awakened,
        "round 1 不觉醒"
    );

    // 轮转两圈到 r3 的 0 号 End（每回合 Judge 起 5 NP 收束）
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // wrap→r2
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge（r2 末：round 2 <3）
    assert!(
        !e.state_core().player(0).lingquan_awakened,
        "round 2 不觉醒"
    );
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // wrap→r3
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：灵泉觉醒（r3 末）
    assert!(e.state_core().player(0).lingquan_awakened, "r3 觉醒");
    assert_eq!(e.debug_hand(0).len(), 3, "觉醒摸 3");
    let c = e.open_choice().expect("灵泉内置水云置牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_LINGQUAN_CARDS
    );
    let placed = match c.spec.options[0] {
        ChoiceOption::Card(x) => x,
        _ => panic!("选项应为牌"),
    };
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(placed)],
        },
    );
    assert!(
        e.state_core().player(0).off_court.contains(&placed),
        "觉醒置牌入离场区"
    );
    assert_eq!(e.debug_hand(0).len(), 2, "置牌后 3-1=2");
    // runtime 授水云实例
    let shuiyun_def = e.catalog().skill_by_name("shuiyun").expect("shuiyun def");
    assert!(
        e.state_core().skills.contains_key(&(0, shuiyun_def)),
        "runtime 授水云实例"
    );
}

/// 神雾：水云计数≥3 时觉醒——上限+1、回复 1、授回梦实例。
#[test]
fn xiaoman_shenwu_awaken_grants_huimeng() {
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

    let mut e = Engine::new_with_extensions(10031, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 5); // 已受伤：回复可见
    }
    e.debug_grant(0, "shenwu");
    // 注入水云计数 3（save/restore 上帝视角；计数真实来源=水云·救）
    {
        let mut snap = e.save();
        snap.core.players[0].shuiyun_count = 3;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 任意阶段末觉醒（global phaseEnd forced）：0 号收束即触发
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：神雾觉醒
    assert!(e.state_core().player(0).shenwu_awakened, "觉醒");
    assert_eq!(e.state_core().player(0).max_hp, 7, "上限+1（6→7）");
    assert_eq!(e.debug_hp(0), 6, "回复 1（5→6）");
    let huimeng_def = e.catalog().skill_by_name("huimeng").expect("huimeng def");
    assert!(
        e.state_core().skills.contains_key(&(0, huimeng_def)),
        "runtime 授回梦实例"
    );

    // 一次性：后续阶段末不再触发（上限不再涨）
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.state_core().player(0).max_hp, 7, "觉醒一次不再触发");
}

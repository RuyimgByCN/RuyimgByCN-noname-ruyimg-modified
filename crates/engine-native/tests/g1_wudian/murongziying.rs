use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三十七批切片（规则 v179）：慕容紫英（pal_murongziying，
// wei 4 男）破云（poyun，source Damage forced direct）——玄砚计数>0 时
// 令受伤目标自选弃 he 1-2 张，计数 -1（计数获得路径上游缺失=调试/宿
// 主注入登记差异）。qianfang AOE 已通暂缓；xuanning 获得路径缺失登记。
// ---------------------------------------------------------------------------

/// 破云：注入计数 2→伤害后目标自选弃 2→计数归零；计数 0 不触发。
#[test]
fn murongziying_poyun_discard_and_decrement() {
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

    let mut e = Engine::new_with_extensions(10052, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "poyun");
    // 注入玄砚计数 2（save/restore；获得路径上游缺失=登记差异）
    {
        let mut snap = e.save();
        snap.core.players[0].xuanning_mark = 2;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let c1 = e.debug_place(1, "tao");
    let c2 = e.debug_place(1, "wuzhong");

    // 0 号伤 1 号 1 点 → 破云 → 1 号自选弃 2 张 → 计数归零
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("破云选牌询问挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_POYUN_CARDS
    );
    assert_eq!(c.observer, 1, "被弃者自选");
    assert_eq!((c.spec.min, c.spec.max), (1, 2), "1-2 张");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    );
    assert_eq!(e.state_core().player(0).xuanning_mark, 1, "计数 -1");
    assert!(
        e.state_core().discard.contains(&c1) && e.state_core().discard.contains(&c2),
        "两牌入弃堆"
    );

    // 计数 1 再伤一次：弃 1 张后计数归零，再伤不触发
    let d3 = e.debug_place(1, "shan");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("破云选牌询问挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(d3)],
        },
    );
    assert_eq!(e.state_core().player(0).xuanning_mark, 0, "计数归零");
    // 计数 0：再伤不触发
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(e.open_choice().is_none(), "计数 0：不触发");
}

/// 千方：玄砚 1→弃 2 张→清零+虚拟万箭（目标各-1、各随机弃 he）→紫
/// 英摸受伤人数；放弃=无事；玄砚 0 不触发。
#[test]
fn murongziying_qianfang_discard_and_aoe() {
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

    let mut e = Engine::new_with_extensions(10055, 3, &[&pack]).expect("建局");
    // 布阵+授技+注入在 advance_loop 前（千方在 Judge begin 触发——
    // advance_loop 停在 0 号 Play，首个 Judge begin 已带触发条件时不满足，
    // 故验证第二个 0 号回合的 Judge→Draw→Play 后由下个 Judge 触发）
    for p in 0..3 {
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "qianfang");
    {
        let mut snap = e.save();
        snap.core.players[0].xuanning_mark = 1;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    advance_loop(&mut e, 0);
    // 千方 filter 要求 he+玄砚>=3：place 后手牌 2+玄砚 1=3 ✓
    let _d1 = e.debug_place(0, "tao");
    let _d2 = e.debug_place(0, "wuzhong");

    // 走到 0 号下个 Judge begin：Play→Discard→End→1(5NP)→2(5NP)→0
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase);
    }
    // 现在 0 号 Judge begin 已过（Judge 帧 AfterTriggers）——千方询问
    // 挂起（Filter：he2+玄砚1=3≥3 ✓）
    let c = e.open_choice().expect("千方弃牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_QIANFANG_CARDS
    );
    assert_eq!((c.spec.min, c.spec.max), (2, 2), "num=3-1=2");
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: picks,
        },
    );
    // 万箭逐目标 aoe_response（无闪全 Pass）
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
    // 断言：玄砚清零+虚拟万箭（1/2 号各-1）+紫英摸受伤人数（2）
    assert_eq!(e.state_core().player(0).xuanning_mark, 0, "计数清零");
    assert_eq!(e.debug_hp(1), 5, "万箭 1 号 -1");
    assert_eq!(e.debug_hp(2), 5, "万箭 2 号 -1");
    assert_eq!(
        e.debug_hand(0).len(),
        4,
        "摸 2（被伤敌人 2 名；Draw2+place2-弃2+摸2=4）"
    );
}

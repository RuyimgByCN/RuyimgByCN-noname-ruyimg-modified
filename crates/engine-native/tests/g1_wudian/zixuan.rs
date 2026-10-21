use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十一切片（规则 v162）：紫萱（pal_zixuan，shu 3 女）
// 水云（shuiyun，阶段末置牌三类去重入离场区 ≤3）+水云·救（shuiyun5，
// global Dying 确认弃离场区牌救人——濒死救援通道首例，shuiyun_count++
// 小蛮链依赖）+忘忧（wangyou/wangyou2，受伤回合戳+他人阶段末弃牌令标
// 记者们各摸 1）。changnian 死亡拦截+跨将授技复合缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 水云：阶段末置牌（类型去重 ≤3）入离场区；取消=放弃。
#[test]
fn zixuan_shuiyun_off_court_and_type_dedup() {
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

    let mut e = Engine::new_with_extensions(10027, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "shuiyun");
    // 预置锦囊入离场区：类型去重——本阶段末手牌的锦囊不可置
    let pre = e.debug_off_court_place(0, "wuzhong");
    let tao = e.debug_place(0, "tao"); // 基本（可置）
    let wz = e.debug_place(0, "wuzhong"); // 锦囊（类型被 pre 占——不可置）
    let eq = e.debug_place(0, "zhuge"); // 装备（可置）

    // 阶段末：选项=可置类型（桃+弩），锦囊 wz 被排除
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：水云询问
    let c = e.open_choice().expect("水云置牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_SHUIYUN_CARDS
    );
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(tao)),
        "基本可置"
    );
    assert!(c.spec.options.contains(&ChoiceOption::Card(eq)), "装备可置");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Card(wz)),
        "锦囊类型已占：不可置"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao)],
        },
    );
    assert_eq!(
        e.state_core().player(0).off_court,
        vec![pre, tao],
        "桃入离场区（预置+新置）"
    );
    assert!(!e.debug_hand(0).contains(&tao), "手牌已离开");
    assert!(e.state_core().player(0).hand.contains(&eq), "未置牌留手");

    // 取消路径：第二次询问（装备/锦囊均占→只剩基本类；预填后测上限）
    // 走完 1/2 号回合回到 0 号（Judge 起 4 NP 收束）
    for _ in 0..4 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..4 {
        let _ = submit(&mut e, 2, GameAction::NextPhase);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
                                                      // 预填第三张（装备）：离场区满 3 → 水云关闭
    let _ = e.debug_off_court_place(0, "zhuge");
    assert_eq!(e.state_core().player(0).off_court.len(), 3, "离场区满");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(e.open_choice().is_none(), "离场区满 3：水云关闭");
}

/// 水云·救：他人濒死确认→弃离场区牌→回复 1+计数（小蛮链）。
#[test]
fn zixuan_shuiyun5_dying_rescue_channel() {
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

    let mut e = Engine::new_with_extensions(10028, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "shuiyun5");
    let saved = e.debug_off_court_place(0, "tao");

    // 1 号压到 1 再直伤 2 → 濒死 → 水云·救确认 → 弃离场区牌 → 回复 1
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 2, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("水云·救确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("离场区救牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_SHUIYUN5_CARDS
    );
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(saved)]);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(saved)],
        },
    );
    assert_eq!(e.debug_hp(1), 0, "回复 1（-1→0，脱离濒死待收束）");
    assert!(
        e.state_core().player(0).off_court.is_empty(),
        "离场区牌已弃"
    );
    assert!(e.state_core().discard.contains(&saved), "救牌入弃堆");
    assert_eq!(
        e.state_core().player(0).shuiyun_count,
        1,
        "水云计数（小蛮链）"
    );
}

/// 忘忧：受伤者获回合戳；他人阶段末弃牌令标记者们各摸 1。
#[test]
fn zixuan_wangyou_roundstamp_and_draw() {
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

    let mut e = Engine::new_with_extensions(10029, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "wangyou");
    e.debug_grant(0, "wangyou2");
    let w = e.debug_place(0, "wuzhong");

    // 标记语义（登记差异）：戳存活至标记者自己下回合开始——伤害须发生
    // 在标记者自己回合内（Judge begin 之后）才能活到其 End 末触发。
    // 0 号回合直接过（自身阶段末不触发=owner 排除）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(e.open_choice().is_none(), "自己阶段结束不触发");

    // 1 号回合内受伤（Judge begin 后）→ 获戳 → 其 End 末忘忧询问
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(
        e.state_core().player(1).wangyou3_round != 0,
        true,
        "受伤获戳"
    );
    e.debug_clear_hand(1); // 清摸牌阶段手牌（隔离摸 1 断言）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：忘忧询问
    let c = e.open_choice().expect("忘忧弃牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_WANGYOU_CARDS
    );
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(w)]);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(w)],
        },
    );
    assert_eq!(e.debug_hand(1).len(), 1, "标记者 1 号摸 1");

    // 2 号回合内受伤获戳 → 2 号 End 末询问②（1 号戳未清、2 号新戳——
    // 取消=放弃验证）
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Draw→Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(2);
    assert_eq!(
        e.state_core().player(2).wangyou3_round != 0,
        true,
        "2 号获戳"
    );
    let w2 = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End→0 Judge：忘忧询问②
    let c = e.open_choice().expect("忘忧弃牌挂起②");
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(w2)]);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(2).len(), 0, "取消=放弃（2 号不摸）");
}

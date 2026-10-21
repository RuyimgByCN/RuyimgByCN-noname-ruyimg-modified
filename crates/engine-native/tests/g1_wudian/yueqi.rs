use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十九批切片（规则 v171）：月琦（pal_yueqi，wei 3 女）
// 适盈（shiying，Death 前置窗口 limited，死亡取消型首例）——他人死亡
// 时取消之：双方上限/体力置 3、clearSkills 双方、弃 hej 全部、各摸 3。
// tianwu 表现层登记不转；liguang moveCard 基建暂缓。
// ---------------------------------------------------------------------------

/// 适盈：他人死亡→取消死亡（存活）+双方重置 3/3+清技能+弃全部+各摸 3；
/// 一次性。
#[test]
fn yueqi_shiying_cancel_death_and_reset() {
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

    let mut e = Engine::new_with_extensions(10041, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(1, "shiying"); // 持技者≠死亡者（filter 他人死亡）
    let keep = e.debug_place(1, "tao"); // 1 号将被清弃的牌

    // 0 号过回合；1 号回合内压 0 号 1 血后伤 2 → 死亡 → 适盈取消
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    // 求桃链全 Pass → 死亡帧 → 适盈取消（无询问 forced）
    loop {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt != "dying_save" {
            break;
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
    // 断言：0 号存活+双方 3/3+清技能+手牌=3（摸 3）+适盈一次性
    assert!(e.debug_alive(0), "死亡被取消");
    assert!(!e.state_core().player(0).death_cancelled, "拦截标记已消费");
    for p in 0..2 {
        assert_eq!(e.state_core().player(p).max_hp, 3, "{p} 号上限置 3");
        assert_eq!(e.debug_hp(p), 3, "{p} 号体力置 3");
    }
    let cn = e.catalog().skill_by_name("changnian");
    let _ = cn;
    assert!(
        e.state_core().player(1).shiying_used,
        "一次性置位（持技者 1 号）"
    );
    assert_eq!(e.debug_hand(0).len(), 3, "0 号摸 3");
    assert!(e.state_core().discard.contains(&keep), "1 号 hej 清弃");
    // 技能清空：0/1 号无剩余实例
    assert!(
        !e.state_core()
            .skills
            .keys()
            .any(|(o, _)| *o == 0 || *o == 1),
        "clearSkills 双方"
    );
}

// ---------------------------------------------------------------------------
// U05 xianjian 第三十一批切片（规则 v173）：月琦离光（liguang，phaseEnd
// direct 可放弃）——弃一张手牌并移动场上的一张牌（moveCard 基建首例：
// 确定性首张可移牌+首个合法目的地，登记差异=上游 AI 选取）。月琦升全
// 闭环（tianwu 登记不转）。
// ---------------------------------------------------------------------------

/// 离光：弃手牌→装备迁移（revoke+grant+Equipped 日志）；取消=放弃。
#[test]
fn yueqi_liguang_discard_and_move_equip() {
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

    let mut e = Engine::new_with_extensions(10044, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "liguang");
    let disc_card = e.debug_place(0, "wuzhong");

    // 布阵：1 号装备区一把武器（可移）；2 号装备区空（合法目的地）
    let weapon = e.debug_place(1, "zhuge");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != weapon);
        snap.core.players[1].equips.push(weapon);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 阶段末：弃牌询问（离光）→ 弃 wuzhong → 确定性移动：武器 1→2
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：离光询问
    let c = e.open_choice().expect("离光弃牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_LIGUANG_CARDS
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(disc_card)],
        },
    );
    // 移动断言：确定性首目的地=0 号（座位序首个空武器栏）；1 号失去
    assert!(
        e.state_core().player(0).equips.contains(&weapon),
        "武器迁至 0 号（首合法目的地）"
    );
    assert!(
        !e.state_core().player(1).equips.contains(&weapon),
        "1 号失去武器"
    );
    assert!(
        e.state_core().discard.contains(&disc_card),
        "所弃手牌入弃堆"
    );
    assert!(e.debug_hand(0).is_empty(), "弃 1 后无手牌");

    // 取消路径：下回合无移动（离场区空则 filter 拦——直接断言询问可取消）
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let d2 = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：离光询问②
    let c = e.open_choice().expect("离光弃牌挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    assert!(e.debug_hand(0).contains(&d2), "取消=放弃（手牌保留）");
}

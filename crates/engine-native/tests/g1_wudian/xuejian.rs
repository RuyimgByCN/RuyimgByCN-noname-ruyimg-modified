use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十四批切片（规则 v165）：雪见（pal_xuejian，shu 3
// 女）神暮（shenmu，global Dying 确认）+毒刺（duci，CardsDiscarded
// direct）——弃全部红手牌令濒死者回复 1 摸等量；弃装备牌入弃堆令距离 1
// 内一名其他角色受 1 伤害。xshuangren 武器技能转移缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 神暮：他人濒死确认→弃全部红手牌→濒死者回复 1 摸等量。
#[test]
fn xuejian_shenmu_red_discard_rescue() {
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

    let mut e = Engine::new_with_extensions(10033, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "shenmu");
    let _r1 = e.debug_place(0, "tao"); // 红
    let _r2 = e.debug_place(0, "wuzhong"); // 可能红可能黑——按实际花色清点
    let _blk = e.debug_place(0, "sha"); // 可能黑——置后按花色断言
    let reds: Vec<u32> = e
        .debug_hand(0)
        .into_iter()
        .filter(|c| {
            matches!(
                e.catalog().cards[*c as usize].suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            )
        })
        .collect();
    let n_red = reds.len();
    assert!(n_red >= 1, "至少一张红（桃保底）");

    // 1 号压到 1 再伤 1 → 0 濒死 → 确认 → 弃全部红 → 1 号回复 1 摸 n_red
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("神暮确认挂起");
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
    assert_eq!(e.debug_hp(1), 1, "濒死者回复 1（0→1）");
    assert_eq!(e.debug_hand(1).len(), n_red, "摸等量红牌数");
    assert_eq!(e.debug_hand(0).len(), 3 - n_red, "仅弃红牌（黑牌保留）");
    let hand0 = e.debug_hand(0);
    assert!(
        !hand0.iter().any(|c| {
            matches!(
                e.catalog().cards[*c as usize].suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            )
        }),
        "无红牌残留"
    );
    for c in &reds {
        assert!(e.state_core().discard.contains(c), "红牌入弃堆");
    }
}

/// 毒刺：弃装备牌入弃堆→距离 1 内一名其他角色受 1 伤害；取消=放弃。
#[test]
fn xuejian_duci_equip_discard_damage() {
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

    let mut e = Engine::new_with_extensions(10034, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "duci");
    e.debug_set_hp(0, 1); // 手牌上限 1：弃牌阶段弃 1
    let equip = e.debug_place(0, "zhuge"); // 装备（手牌弃置路径）
    let _tao = e.debug_place(0, "tao"); // 基本对照

    // ① 弃装备（手牌弃置）→ 毒刺目标询问 → 指定 1 号（距 1）受 1 伤
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("弃牌阶段挂起");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(equip)],
        },
    );
    let c = e.open_choice().expect("毒刺目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_DUCI_TARGET
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
    assert_eq!(e.debug_hp(1), 5, "毒刺伤害（6→5）");
    assert!(e.state_core().discard.contains(&equip), "装备入弃堆");

    // ② 弃基本牌 → 不触发（对照）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 号回合收束→0 Judge
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = e.debug_place(0, "tao");
    let tao2 = e.debug_place(0, "tao");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard：弃 1 张
    let c = e.open_choice().expect("弃牌阶段挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao2)],
        },
    );
    assert!(e.open_choice().is_none(), "弃基本牌：毒刺不触发");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第三十八批切片（规则 v180）：雪见（pal_xuejian）双刃
//（xshuangren，CardsDiscarded forced）+双刃·殁（xshuangren2，Death 前
// 置窗口 forced）——失去武器（弃置路径）置离场区+grant 武器技能；死亡
// 时弃置离场区全部牌。雪见升全闭环。
// ---------------------------------------------------------------------------

/// 双刃：弃武器→离场区+grant 武器技能；双刃·殁：死亡时弃离场区。
#[test]
fn xuejian_xshuangren_offcourt_and_death_discard() {
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

    let mut e = Engine::new_with_extensions(10053, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xshuangren");
    e.debug_grant(0, "xshuangren2");

    // 布阵：手牌武器 + 压 hp=1（手牌上限 1，弃牌阶段弃武器）
    let weapon = e.debug_place(0, "zhuge");
    let extra = e.debug_place(0, "tao");
    e.debug_set_hp(0, 1);

    // 弃牌阶段弃武器 → CardsDiscarded 含武器 → 双刃收离场区+grant
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("弃牌询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    // 断言：双刃收武器入离场区（非弃堆）+武器技能 grant
    assert!(
        e.state_core().player(0).off_court.contains(&weapon),
        "武器入离场区"
    );
    assert!(!e.state_core().discard.contains(&weapon), "未直接入弃堆");
    assert!(e.debug_hand(0).contains(&extra), "超额牌保留");

    // 压 1 血再伤 2 → 死亡 → 双刃·殁弃离场区全部 → 死亡照常
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
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
    assert!(!e.debug_alive(0), "死亡照常");
    assert!(
        e.state_core().player(0).off_court.is_empty(),
        "殁：离场区已清"
    );
    assert!(e.state_core().discard.contains(&weapon), "殁：武器入弃堆");
}

use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十八批切片（规则 v169）：王小虎（pal_wangxiaohu，qun
// 4 男）虎煞（husha_count+husha）——出牌回合内伤人 +伤害量标记（≤3）；
// 阶段末消耗标记虚拟用牌：1 枚虚拟杀（选目标）/2 枚虚拟南蛮 AOE
//（use_virtual_card 原语首例：牌堆底载体+use_as 声明）。元素毁灭牌种
// 不存在登记不转。
// ---------------------------------------------------------------------------

/// 虎煞：伤人集标记→阶段末虚拟杀（响应链走通）；虚拟南蛮 AOE 全体验
/// 伤害；标记上限 3。
#[test]
fn wangxiaohu_husha_marks_and_virtual_cards() {
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

    let mut e = Engine::new_with_extensions(10038, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "husha_count");
    e.debug_grant(0, "husha");

    // ① 0 号回合内伤 1 号 2 点 → 标记 +2；再伤 2 号 1 点 → +1（满 3）
    e.debug_deal_damage(Some(0), 1, 2, engine_native::event::DamageNature::Normal);
    e.debug_deal_damage(Some(0), 2, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(
        e.state_core().player(0).husha_mark,
        3,
        "标记 2+1=3（上限 3）"
    );

    // ② 阶段末：模式选择 [Tag0=虚拟杀, Tag1=虚拟南蛮] → Tag0 虚拟杀
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：虎煞询问
    let c = e.open_choice().expect("虎煞模式挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_HUSHA_MODE
    );
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert_eq!(e.state_core().player(0).husha_mark, 2, "扣 1 枚");
    let c = e.open_choice().expect("虚拟杀目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_HUSHA_TARGET
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
    assert_eq!(e.debug_hp(1), 3, "虚拟杀直伤（4→3）");
    assert_eq!(
        e.state_core().player(0).husha_mark,
        2,
        "回合已推进：不再集标记"
    );
}

/// 虎煞虚拟南蛮：注入标记 2 → Tag1 → AOE 全体其他角色各 -1、扣 2 枚。
#[test]
fn wangxiaohu_husha_virtual_nanman() {
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

    let mut e = Engine::new_with_extensions(10039, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "husha");
    {
        let mut snap = e.save();
        snap.core.players[0].husha_mark = 2;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：虎煞询问
    let c = e.open_choice().expect("虎煞模式挂起");
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
        "mark 2：杀+南蛮双选项"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(e.state_core().player(0).husha_mark, 0, "扣 2 枚");
    assert_eq!(e.debug_hp(1), 5, "南蛮 1 号直伤（6→5）");
    assert_eq!(e.debug_hp(2), 5, "南蛮 2 号直伤（6→5）");
}

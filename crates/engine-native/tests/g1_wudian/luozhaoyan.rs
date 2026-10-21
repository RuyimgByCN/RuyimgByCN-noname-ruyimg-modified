use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三十五批切片（规则 v177）：罗招嫣（pal_luozhaoyan，
// shu 4 女）凤略（fenglue，Play-begin direct 可放弃）——出牌阶段开始选
// 一名其他角色，逐张自动对其使用手中可指定其的杀与决斗（真实牌 emit
// CardUsed=虚拟管线复用真实牌变体）。tanhua Recover 前置窗口缺口登记
// 暂缓。
// ---------------------------------------------------------------------------

/// 凤略：选目标→逐张自动使用杀（距离内）与决斗；取消=放弃。
#[test]
fn luozhaoyan_fenglue_auto_use_sha_duel() {
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

    let mut e = Engine::new_with_extensions(10050, 2, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "fenglue");
    // 0 号过回合→1 号→回 0 号（技能已持有，Play-begin 触发）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 号回合收束
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    let sha = e.debug_place(0, "sha");
    let duel = e.debug_place(0, "duel");
    let wuzhong = e.debug_place(0, "wuzhong"); // 非杀/决斗：不自动

    // 凤略在 Play-begin 已过——需再绕一圈让带牌 Play-begin 触发
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    let c = e.open_choice().expect("凤略目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_FENGLUE_TARGET
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
    // 驱动自动使用链（杀/决斗可能引发响应与伤害询问）
    for _ in 0..40 {
        let Some(c) = e.open_choice() else { break };
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
    // 断言：两张牌均已离手；1 号受杀 1 伤+决斗 1 伤（无杀响应）=6-2=4
    assert!(!e.debug_hand(0).contains(&sha), "杀已自动使用");
    assert!(!e.debug_hand(0).contains(&duel), "决斗已自动使用");
    assert!(e.debug_hand(0).contains(&wuzhong), "非杀/决斗不自动");
    assert_eq!(e.debug_hp(1), 4, "杀+决斗各 1 伤（6→4）");
}

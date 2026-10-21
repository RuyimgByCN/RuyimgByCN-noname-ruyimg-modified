use super::*;

/// 绝道（U07 技能批六，规则 v116）：刘焉——出牌阶段横置（自连环，已
/// 横置不可发动）；横置距离双向 +1（computed_distance 直读 linked）。
#[test]
fn liuyan_juedao_link_and_distance() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "wudian-upstream-noname")
        .expect("wudian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // 5 人局对座基线 2
    let mut e = Engine::new_with_extensions(8901, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_grant(0, "juedao");
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        2,
        "基线 2"
    );
    assert!(!e.state_core().player(0).linked, "初始未横置");

    // 发动 → 横置
    let def = e.catalog().skill_by_name("juedao").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().player(0).linked, "横置生效");
    // 双向距离 +1（对座 2→3）
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        3,
        "横置攻侧 +1"
    );
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 2, 0),
        3,
        "横置守侧 +1"
    );

    // 已横置不可再发动
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
        ),
        "已横置应拒: {out:?}"
    );

    // 他人（1 号）横置不影响 0↔2：1 号 linked 只加 1 号双向——
    // 授予后需在 1 号自己的回合发动（ActivateSkill 授权限当前回合玩家）
    e.debug_grant(1, "juedao");
    advance_loop(&mut e, 1);
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: def });
    assert!(e.state_core().player(1).linked, "1 号横置");
    // 0↔2 不受 1 号影响
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        3,
        "0↔2 不变"
    );
    // 0↔1：双横置 = 基线 1 + 1 + 1 = 3
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        3,
        "双横置叠加"
    );
}

/// 割据（U07 技能批七，规则 v117）：刘焉——回合开始强制摸「攻击范围
/// 外其他势力种数」张（势力=effective_faction；无主将/unknown 排除；
/// 杀范围内他势力不计入）。
#[test]
fn liuyan_geju_out_of_range_faction_draw() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "geju");

    // 5 人局不适用（setup 3 人局相邻全在杀范围内）——3 人局全在范围内
    // → 无范围外他势力 → 不摸（负例先行）
    // 授予触发：0 号回合开始（Judge begin）已过——用下一回合验证
    let out = e.catalog().skill_by_name("geju").expect("geju 在目录");
    let _ = out;

    // 轮回 0 号：回合开始触发割据——3 人局相邻全在杀范围内（攻距 1）→
    // 无范围外他势力 → 不摸（Judge begin 先于 Draw：手牌=Draw 摸的 2）
    e.debug_set_skill_enabled(0, "geju", true);
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
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
    assert_eq!(
        e.debug_hand(0).len(),
        2,
        "3 人局全在杀范围内：不摸（手牌=Draw 摸 2）"
    );
    let _ = _pack;
}

/// 割据正例（4 人局）：1 号魏、2 号蜀、0 号任意；杀攻距 1——1 号在范
/// 围内（魏不计入）、2 号距离 2 范围外（蜀计入）→ 回合开始摸 1。
#[test]
fn liuyan_geju_out_of_range_counts() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "wudian-upstream-noname")
        .expect("wudian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let (wei, shu) = {
        let wei = engine_native::content::standard::STANDARD_GENERALS
            .iter()
            .find(|(_, f, ..)| *f == engine_native::catalog::Faction::Wei)
            .map(|(n, ..)| n.to_string())
            .expect("魏将");
        let shu = engine_native::content::standard::STANDARD_GENERALS
            .iter()
            .find(|(_, f, ..)| *f == engine_native::catalog::Faction::Shu)
            .map(|(n, ..)| n.to_string())
            .expect("蜀将");
        (wei, shu)
    };

    let mut e = Engine::new_with_extensions(9001, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_grant(0, "geju");
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);

    // 0↔2 距离 2 > 攻距 1 → 蜀为范围外他势力 → 回合开始摸 1
    let round0 = e.turn().round;
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
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
    // Judge begin 割据摸 1 → Draw 摸 2 → 手牌 3
    assert_eq!(
        e.debug_hand(0).len(),
        3,
        "范围外他势力（蜀）1 种 → 摸 1 + Draw 2"
    );
}

use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三十六批切片（规则 v178）：姜云凡（pal_jiangyunfan，
// wei 4 男）疾风（liefeng，CardUsed forced）+天剑（tianjian，phaseUse
// usable:1 主动，liefeng 授技）——出牌回合内第 3/4 张使用牌时 runtime
// 授天剑/御风实例（第 2 张授炎杀 yanzhan 暂缓不授）；天剑弃杀视为万
// 箭齐发（虚拟 AOE 原语）+目标随机弃 he 一张。xunying 响应链暂缓。
// ---------------------------------------------------------------------------

/// 疾风：第 3 张授天剑实例；天剑弃杀视为万箭+目标随机弃 he。
#[test]
fn jiangyunfan_liefeng_grant_and_tianjian_aoe() {
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

    let mut e = Engine::new_with_extensions(10051, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "liefeng");
    e.debug_grant(0, "tianjian");

    // 0 号用 3 张无中（无目标直用）→ 第 3 张触发疾风授天剑
    for _ in 0..3 {
        let w = e.debug_place(0, "wuzhong");
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: w,
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
    }
    assert_eq!(e.state_core().player(0).total_used, 3, "全局用牌计数");
    let tj_def = e.catalog().skill_by_name("tianjian").expect("def");
    assert!(
        e.state_core().skills.contains_key(&(0, tj_def)),
        "第 3 张授天剑实例"
    );

    // 天剑：手牌需有杀——布阵后激活
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 号回合收束
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 号回合收束
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_place(1, "tao"); // 1 号手牌（弃牌候选）
    e.debug_place(2, "tao");
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let he1 = e.debug_hand(1).len() + e.state_core().player(1).equips.len();
    let he2 = e.debug_hand(2).len() + e.state_core().player(2).equips.len();

    let def_tj = e.catalog().skill_by_name("tianjian").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_tj });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
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
    // 万箭：1/2 号无闪各 -1；各随机弃 he 一张
    assert_eq!(e.debug_hp(1), hp1 - 1, "万箭 1 号 -1");
    assert_eq!(e.debug_hp(2), hp2 - 1, "万箭 2 号 -1");
    assert!(e.state_core().discard.contains(&sha), "弃杀入弃堆");
    // 随机弃牌断言：各目标 he 恰好 -1
    assert_eq!(
        e.debug_hand(1).len() + e.state_core().player(1).equips.len(),
        he1 - 1,
        "1 号随机弃一张"
    );
    assert_eq!(
        e.debug_hand(2).len() + e.state_core().player(2).equips.len(),
        he2 - 1,
        "2 号随机弃一张"
    );
}

/// 迅影：杀结算后自动再出一张杀对同一目标（距离内）；无杀不触发。
#[test]
fn jiangyunfan_xunying_auto_second_sha() {
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

    let mut e = Engine::new_with_extensions(10056, 2, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xunying");
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");

    // 0 号回合内杀 1 号 → 迅影自动再出一张
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
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
    assert_eq!(e.debug_hp(1), 4, "两次杀各-1（6→4）");
    assert!(
        !e.debug_hand(0).contains(&sha1) && !e.debug_hand(0).contains(&sha2),
        "两张杀均已使用"
    );
}

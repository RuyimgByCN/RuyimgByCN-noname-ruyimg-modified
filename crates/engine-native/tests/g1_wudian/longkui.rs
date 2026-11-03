use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十二切片（规则 v153）：龙葵（pal_longkui）蝶舞（diewu，
// 出牌阶段主动）+灵愈（lingyu，回合结束确认）双技闭环。fenxing（变身）
// 结构性 deferred。存档 v18→v19（diewu_draw_used）。
// ---------------------------------------------------------------------------

/// 蝶舞：杀直接转移给其他角色（不弃置）+本回合首次摸 1（第二次不摸）；
/// 灵愈：令一名其他已受伤角色回复 1（可放弃）。
#[test]
fn longkui_diewu_give_and_lingyu_recover() {
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

    let mut e = Engine::new_with_extensions(10010, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "diewu");
    e.debug_grant(0, "lingyu");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 2); // 已受伤
    let sha = e.debug_place(0, "sha");
    let hand0_before = e.debug_hand(0).len();

    // ① 蝶舞：杀转移给 1 号+首次摸 1（手牌净变化 -1+1=0）
    let def_dw = e.catalog().skill_by_name("diewu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_dw });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0_before,
        "给出杀-1+首次摸 1=净 0"
    );
    assert!(e.debug_hand(1).contains(&sha), "杀转移到 1 号手牌");
    assert!(e.state_core().turn.diewu_draw_used, "首次摸牌已消耗");

    // ② 灵愈（A2 H81）：上游 player phaseEnd＝整回合结束——0 号整回合
    // 结束窗 phaseEnd 段确认门挂起，令 1 号（2/3）回复 1
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→整回合结束窗（灵愈挂起）
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 3, "灵愈回复 1（2→3）");
}

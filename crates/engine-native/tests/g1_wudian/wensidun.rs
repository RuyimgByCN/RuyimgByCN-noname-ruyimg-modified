use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十（规则 v137）：温斯顿（ow_wensidun）跳跃（feitiao，
// 出牌阶段主动无次数限制）——弃 1 张牌指定一名其他角色：其随机弃 1，
// 本回合自己算到其距离恒 1。dianji（惊雷杀牌种缺失）登记暂缓。
// ---------------------------------------------------------------------------

/// 跳跃：弃牌+目标随机弃 1；距离恒 1（无跳跃时距离 2 的杀被拒，跳跃
/// 后可达）；换回合解除。
#[test]
fn wensidun_feitiao_range_and_random_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9993, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feitiao");
    let c1 = e.debug_place(0, "tao");
    e.debug_place(1, "tao"); // 目标牌池
    e.debug_set_hp(2, 4);

    // 负例：距离 2 的 2 号杀不可达
    let sha0 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha0,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "距离 2 应被拒：{out:?}"
    );

    // 跳跃：弃 tao→目标 2→其随机弃 1+距离恒 1
    let def_ft = e.catalog().skill_by_name("feitiao").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ft });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().discard.contains(&c1), "跳跃弃牌入弃牌堆");
    assert_eq!(e.debug_hand(2).len(), 0, "目标随机弃 1（牌池仅 1 张）");
    assert_eq!(e.state_core().turn.feitiao_target, Some(2), "跳跃目标置位");

    // 正例：距离恒 1——杀 2 号可达
    let sha1 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "跳跃后距离 2 可达：{out:?}"
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
    assert_eq!(e.debug_hp(2), 3, "杀命中（4→3）");

    // 无次数限制：同回合可再发动
    let c2 = e.debug_place(0, "tao");
    e.debug_place(1, "tao");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ft });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "无次数限制：{out:?}"
    );
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c2)]);
    let _ = answer_open(&mut e, vec![ChoiceOption::Player(1)]);

    // 换回合解除
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let t = e.turn();
    assert_eq!(t.current, 1);
    assert_eq!(e.state_core().turn.feitiao_target, None, "换回合解除");
}

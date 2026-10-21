use super::*;

/// 扩音：弃 1 张 → 乐动目标数 +2；出牌阶段结束触发乐动 → 3 目标各摸
/// 1（基础摸量）；弃牌入弃牌堆。
#[test]
fn luxiao_kuoyin_num_extends_yuedong_draw() {
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

    let mut e = Engine::new_with_extensions(9901, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_grant(0, "kuoyin");
    let c1 = e.debug_place(0, "sha");
    let _c2 = e.debug_place(0, "tao");

    // 扩音：activate 挂起选牌（1–2 张）→ 弃 1 张
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().turn.yuedong_num, 2, "弃 1 张 → 目标数 +2");
    assert_eq!(e.state_core().turn.yuedong_eff, 0);
    assert!(e.state_core().discard.contains(&c1), "扩音弃牌入弃牌堆");

    // 出牌阶段结束 → 乐动触发挂起（目标数 1+2=3，摸量 1+0=1）
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let targets_sel = vec![
        ChoiceOption::Player(1),
        ChoiceOption::Player(2),
        ChoiceOption::Player(3),
    ];
    let out = answer_open(&mut e, targets_sel);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(1).len(), 1, "目标 1 摸 1");
    assert_eq!(e.debug_hand(2).len(), 1, "目标 2 摸 1");
    assert_eq!(e.debug_hand(3).len(), 1, "目标 3 摸 1");
    assert_eq!(e.debug_hand(0).len(), 1, "自己未选不摸（余 1 张 tao）");
}

/// 扩音双段：弃 2 张 → 摸/回量 +1（乐动 1 目标摸 2）；双满后 activate
/// 被拒（上游 filter 双存量同口径）。
#[test]
fn luxiao_kuoyin_eff_and_double_full_gate() {
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

    let mut e = Engine::new_with_extensions(9902, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_grant(0, "kuoyin");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    let c3 = e.debug_place(0, "shan");

    // 第一次扩音：弃 2 张 → 摸/回量 +1
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert_eq!(e.state_core().turn.yuedong_eff, 1, "弃 2 张 → 量 +1");
    assert_eq!(e.state_core().turn.yuedong_num, 0);

    // 第二次扩音（存量单满 → 恰 1 张）：弃 1 张 → 目标数 +2
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c3)]);
    assert_eq!(
        e.state_core().turn.yuedong_num,
        2,
        "第二次恰选 1 张 → 数 +2"
    );

    // 双满 → activate 拒绝
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
        ),
        "双存量后应拒绝：{out:?}"
    );

    // 乐动：1 目标摸 2（量 1+1）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(1).len(), 2, "1 目标摸 2");
}

/// 呼唤：弃 2 张 → 回复模式；乐动改回复（未满体力目标 +1 体力，不摸
/// 牌）；满血者不入目标选项（上游 filterTarget hp<maxHp 同口径）。
#[test]
fn luxiao_huhuan_switches_yuedong_to_recover() {
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

    let mut e = Engine::new_with_extensions(9903, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_grant(0, "huhuan");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    e.debug_set_hp(1, 2);
    e.debug_set_hp(3, 3);

    // 呼唤：弃 2 张 → 回复模式
    let def_hh = e.catalog().skill_by_name("huhuan").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert!(e.state_core().turn.yuedong_recover, "呼唤置回复模式");

    // 呼唤二次被拒（recover 已置位）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
        ),
        "呼唤限一次：{out:?}"
    );

    // 乐动触发：目标选项仅未满体力者（1/3；0/2/4 满血排除）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().cloned().expect("乐动挂起");
    let mut opts: Vec<u8> = c
        .spec
        .options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect();
    opts.sort_unstable();
    assert_eq!(opts, vec![1, 3], "仅未满体力者可选：{opts:?}");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 3, "回复模式：2→3");
    assert_eq!(e.debug_hand(1).len(), 0, "回复模式不摸牌");
}

/// 乐动可整体放弃（cancel 空选择）；挂起的乐动目标询问跨存档恢复
///（TurnState 三字段入存档 v7 + active_ask 派生贯通）。
#[test]
fn luxiao_yuedong_cancel_and_pending_ask_restores() {
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

    let mut e = Engine::new_with_extensions(9904, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_place(0, "sha");

    // 无扩音：1 目标摸 1；整体放弃（空选择=取消）→ 无效果
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let out = answer_open(&mut e, Vec::new());
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(1).len(), 0, "放弃乐动不摸牌");
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 弃牌→结束→下家

    // 下一次 0 的出牌阶段：扩音后挂起乐动 → 存档 → 恢复 → 应答生效
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_grant(0, "kuoyin");
    let c1 = e.debug_place(0, "sha");
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_some(), "乐动挂起");
    let snap = e.save();
    let mut e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    let before = e2.debug_hand(2).len();
    let out = answer_open(&mut e2, vec![ChoiceOption::Player(2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e2.debug_hand(2).len(), before + 1, "恢复后乐动应答摸 1");
    assert_eq!(e2.state_core().turn.yuedong_num, 2, "扩音存量随存档保留");
}

/// v126 遗留缺陷回归（v128 修复）：乐动三角换回合清零——TurnState 换
/// 回合为原地改字段而非重建，扩音/呼唤存量不得跨回合残留（上游 kuoyin2
/// phaseBegin 重置口径）。
#[test]
fn luxiao_yuedong_storage_resets_next_turn() {
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

    let mut e = Engine::new_with_extensions(9923, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "kuoyin");
    e.debug_grant(0, "huhuan");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    let c3 = e.debug_place(0, "shan");
    let c4 = e.debug_place(0, "sha");

    // 弃 1 张（num+2）、呼唤弃 2 张（recover）
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert_eq!(e.state_core().turn.yuedong_num, 2);
    let def_hh = e.catalog().skill_by_name("huhuan").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c2), ChoiceOption::Card(c3)]);
    assert!(e.state_core().turn.yuedong_recover);

    // 收束回合 → 驱动到 0 号下个出牌阶段：存量应全部清零
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    let t = e.state_core();
    assert_eq!(
        (
            t.turn.yuedong_num,
            t.turn.yuedong_eff,
            t.turn.yuedong_recover
        ),
        (0, 0, false),
        "乐动三角随换回合清零"
    );
    // 清零后扩音可再次发动（存量双满不再拒绝）
    e.debug_clear_hand(0);
    let c5 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "新回合扩音可发动：{out:?}"
    );
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c5)]);
    assert_eq!(e.state_core().turn.yuedong_num, 2, "新回合重新积攒");
    let _ = c4;
}

use super::*;

/// 链钩：判定非红桃→挂钩；路霸算到被钩者距离恒 1（无钩时距离 2 被拒）；
/// 被钩者首次受伤 +1（决斗 1+1）；换回合随 TurnState 清零解除。
#[test]
fn luba_liangou_hook_distance_and_damage() {
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

    let mut e = Engine::new_with_extensions(9921, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "liangou");
    let c1 = e.debug_place(0, "sha");
    let _c_t = e.debug_place(1, "tao"); // 1 号留牌（与本测试无涉）

    // 负例：无钩时 2 号（座位距离 2）不可达
    let c2 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: c2,
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

    // 构造黑桃判定顶
    let mut snap = e.save();
    rig_next_judge(&e, &mut snap, engine_native::catalog::Suit::Spade);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");

    // 链钩：弃 c1 → 目标 2 → 判定黑桃成功
    let def_lg = e.catalog().skill_by_name("liangou").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
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
    assert_eq!(e.state_core().turn.liangou_hook, Some(2), "判定非红桃→挂钩");
    assert!(!e.state_core().turn.liangou_bonus_used);
    assert!(e.state_core().discard.contains(&c1), "链钩弃牌入弃牌堆");

    // 正例：距离恒 1——杀可达 2 号；首次伤害 +1（4-1-1=2）
    let c3 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: c3,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "挂钩后距离 2 可达：{out:?}"
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
    assert_eq!(e.debug_hp(2), 2, "首伤 +1（4→2）");
    assert!(e.state_core().turn.liangou_bonus_used, "增幅已消耗");

    // 决斗（距离 1 可达）：无增幅（2→1）
    let duel = e.debug_place(0, "duel");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![2],
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
    assert_eq!(e.debug_hp(2), 1, "决斗无增幅（2→1）");

    // usable:1：同回合再发动拒绝（SkillAlreadyUsed——once_used 命令层）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)
        ),
        "链钩每回合限一次：{out:?}"
    );

    // 换回合随 TurnState 清零解除
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→收束→1 号
    let t = e.turn();
    assert_eq!(t.current, 1);
    assert_eq!(e.state_core().turn.liangou_hook, None, "换回合钩住解除");
}

/// 链钩判定红桃→不挂钩（距离仍受限）+发动已消耗；吸阳：受伤未翻面时
/// 回合结束确认——翻面+回复 2；翻面/满血不触发；翻面后下回合被跳过翻回。
#[test]
fn luba_liangou_heart_fail_and_xiyang_flip_recover() {
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

    let mut e = Engine::new_with_extensions(9922, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "liangou");
    e.debug_grant(0, "xiyang");
    let c1 = e.debug_place(0, "sha");

    // 构造红桃判定顶
    let mut snap = e.save();
    rig_next_judge(&e, &mut snap, engine_native::catalog::Suit::Heart);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");

    // 链钩判定失败：不挂钩
    let def_lg = e.catalog().skill_by_name("liangou").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    let _ = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().turn.liangou_hook, None, "红桃判定不挂钩");
    // 发动已消耗（usable:1 计发动不按结果）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)
        ),
        "{out:?}"
    );

    // 吸阳前置：0 号受伤（2/4）、正面
    e.debug_set_hp(0, 2);
    // 弃牌→End：End 阶段结束触发吸阳确认门
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End 结束→吸阳门
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.debug_face_down(0), "吸阳翻面");
    assert_eq!(e.debug_hp(0), 4, "吸阳回复 2（2→4）");

    // 翻面后下回合被跳过翻回（v127 基建）——驱动到 1 回合结束
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 收束→1 号 Judge
    let mut face_up_again = false;
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 0 && !e.debug_face_down(0) {
            face_up_again = true;
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
    assert!(face_up_again, "跳过回合翻回正面");
}

use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三切片（规则 v144）：赵灵儿（pal_zhaoliner）回梦
//（huimeng，回复后摸 2）+蛇蜕（tianshe+tianshe2，属性伤害免疫/造成属
// 性伤害后回复 1）三机全闭环。
// ---------------------------------------------------------------------------

/// 回梦：自回复后摸 2；蛇蜕：属性伤害免疫（体力不变）、普通伤害照常；
/// 蛇蜕·息：造成属性伤害后回复 1（已受伤时）。
#[test]
fn zhaoliner_huimeng_tianshe_family() {
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

    let mut e = Engine::new_with_extensions(10001, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "huimeng");
    e.debug_grant(0, "tianshe");
    e.debug_grant(0, "tianshe2");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    let tao = e.debug_place(0, "tao");
    e.debug_place(0, "fire_sha");

    // ① 回梦：吃桃自回复→摸 2（3→4 体力+手牌 +2）
    let hand_before = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
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
    assert_eq!(e.debug_hp(0), 4, "吃桃回复（3→4）");
    assert_eq!(
        e.debug_hand(0).len() as i64,
        (hand_before - 1 + 2) as i64,
        "回梦摸 2"
    );

    // ② 蛇蜕：1 号回合火杀 0 号→伤害无效（体力不变）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let fsha = e.debug_place(1, "fire_sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: fsha,
            targets: vec![0],
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
    assert_eq!(e.debug_hp(0), 4, "蛇蜕：属性伤害无效");

    // ③ 蛇蜕·息：1 号回合普通杀打 2 号不触发（普通伤害）——改由 0 号
    //    回合验证：火杀打 2 号（0 已受伤 3<4）→2 号掉 1+0 回复？0 未受
    //    伤已满血→不回复。先压 0 至 3：已满血 4——set 3
    e.debug_set_hp(0, 3);
    // 0 号自己的出牌阶段已过（收束后 1 回合）——由 1 号触发不了 0 的
    // source 技。驱动 2/3/4→0 回合出牌：火杀打 1 号
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
    let fsha2 = e.debug_place(0, "fire_sha");
    e.debug_set_hp(1, 4);
    let hp0_before = e.debug_hp(0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fsha2,
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
    assert_eq!(e.debug_hp(1), 3, "火杀命中（4→3）");
    assert_eq!(
        e.debug_hp(0),
        hp0_before + 1,
        "蛇蜕·息：造成属性伤害后回复 1"
    );
}

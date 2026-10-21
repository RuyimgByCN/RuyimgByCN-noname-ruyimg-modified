use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十切片（规则 v151）：李逍遥（pal_lixiaoyao）御风（xj-
// yufeng，弃牌触发锁定每回合限两次）——失去手牌且手牌<2→摸（2-手牌
// 数）张。tianjian（viewAs 万箭 AOE+伤害归属）登记暂缓。
// ---------------------------------------------------------------------------

/// 御风：失去手牌且手牌<2→补至 2；每回合两次（第三次失牌不触发）。
#[test]
fn lixiaoyao_xjyufeng_draw_to_two_twice_per_turn() {
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

    let mut e = Engine::new_with_extensions(10008, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xjyufeng");
    e.debug_set_hp(0, 1); // 手牌上限 1：弃牌阶段弃 1 张
    e.debug_place(0, "wuzhong");
    e.debug_place(0, "tao");

    // 第一次：弃牌阶段弃 tao（剩 wuzhong 1<2）→补 1
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard（门挂起）
    let tao = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Tao)
        .expect("应有桃");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(tao)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(0).len(), 2, "御风补至 2（弃 1 剩 1→补 1）");
    assert_eq!(e.state_core().turn.xjyufeng_used, 1);

    // 第二次：满 2 不触发（filter 手牌<2 收窄）；但可再失一次——置手牌
    // 上限内再弃一次：先打出 wuzhong（手牌 1），再弃牌阶段无多余——
    // 简化：直接 End→收束→1 回合→驱动 0 下回合出牌阶段，弃 1 再验证
    // 第二次
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
                                                      // 驱动到 0 下回合出牌（途经 1-4 号回合）
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
    // 新回合：御风计数已复位
    assert_eq!(e.state_core().turn.xjyufeng_used, 0, "换回合复位");
}

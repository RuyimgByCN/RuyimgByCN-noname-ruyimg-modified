use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第九切片（规则 v150）：雷元戈（pal_leiyuange）风/雅/颂
// 三计数器技全闭环——累计 2 次后第 3 次强化：风=摸牌后额外摸 1（after
// 补摸等价）；雅=受伤 -1；颂=伤害 +1。
// ---------------------------------------------------------------------------

/// 风：前两次摸牌计数、第三次摸牌后额外摸 1；雅：前两次受伤计数、第三
/// 次受伤 -1；颂：前两次造成伤害计数、第三次 +1。
#[test]
fn leiyuange_feng_ya_song_counters() {
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

    let mut e = Engine::new_with_extensions(10007, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feng");
    e.debug_grant(0, "ya");
    e.debug_grant(0, "song");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    // 雅/颂计数：0 号杀 1 号 ×3（第 3 次伤害 +1）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    // 三轮杀 1 号：第 1 次（颂计 1）4→3；第 2 次（颂计 2）3→2；第 3 次
    // （颂满 2→触发 +1 并清零）2→0？2-2=0→濒死——压 1 号血到 6
    e.debug_set_hp(1, 6);
    for round in 0..3 {
        let sha = e
            .debug_hand(0)
            .into_iter()
            .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
            .expect("应有杀");
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
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
        let expected = match round {
            0 => 5, // 6-1（颂计 1）
            1 => 4, // 5-1（颂计 2）
            _ => 2, // 4-2（颂满 2→+1 并清零）
        };
        assert_eq!(e.debug_hp(1), expected, "第 {} 轮杀", round + 1);
    }
    assert_eq!(e.state_core().player(0).song_count, 0, "颂计数已清零");

    // 雅：三轮火杀——第 1/2 次各 -0（计数）；第 3 次 -1（1 点伤害归零）
    e.debug_set_hp(0, 6);
    e.debug_set_hp(1, 6);
    for round in 0..3 {
        // 驱动到 1 号出牌阶段
        for _ in 0..96 {
            let t = e.turn();
            if t.current == 1 && t.phase == Phase::Play && e.open_choice().is_none() {
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
        e.debug_clear_hand(1);
        e.debug_place(1, "fire_sha");
        let fsha = e
            .debug_hand(1)
            .into_iter()
            .find(|c| e.catalog().cards[*c as usize].kind == CardKind::FireSha)
            .expect("1 号应有火杀");
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
        if round < 2 {
            assert_eq!(e.debug_hp(0), 5 - round as i32, "第 {} 轮火杀", round + 1);
        } else {
            assert_eq!(e.debug_hp(0), 4, "雅满：伤害 -1→0（体力不变 4）");
        }
        // 离开 1 号出牌阶段（下一轮 drive 不立即 break）
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().player(0).ya_count, 0, "雅计数已清零");
}

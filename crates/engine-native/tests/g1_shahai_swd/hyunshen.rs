use super::*;

/// T12 swd deferred 消解批 107：魂身（胡月 `swd_huyue`）——使用/打出闪后
/// 计数 +1；准备阶段摸等量并清零；他人算向持有者距离 +计数。
#[test]
fn swd_deferred_hyunshen_counter_draw_and_distance() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(62067, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hyunshen");
    e.debug_grant(1, "hyunshen2");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    let dist_before = e.state_core().computed_distance(e.catalog(), 0, 1);
    assert_eq!(dist_before, 1, "初始距离 1");
    let out = play(&mut e, sha, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let has_shan = c
            .spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == shan));
        let sel: Vec<ChoiceOption> = if has_shan {
            vec![ChoiceOption::Card(shan)]
        } else if c.spec.min == 0 || c.spec.allow_cancel {
            vec![ChoiceOption::Pass]
        } else {
            c.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert_eq!(
        e.state_core().player(1).hyunshen_count,
        1,
        "魂身：打出闪后计数 1"
    );
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        2,
        "魂身：他人算向持有者距离 +计数（1→2）"
    );
    // 推进到 1 号准备阶段 → 摸 1 并清零。
    for _ in 0..8 {
        if e.turn().current == 1 {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        for _ in 0..8 {
            let Some(c) = e.open_choice() else { break };
            let sel: Vec<ChoiceOption> = if c.spec.min == 0 || c.spec.allow_cancel {
                vec![ChoiceOption::Pass]
            } else {
                c.spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(c.spec.min.max(1))
                    .collect()
            };
            let (cid, obs) = (c.id, c.observer);
            let _ = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
        }
    }
    assert_eq!(
        e.state_core().player(1).hyunshen_count,
        0,
        "魂身：准备阶段清零"
    );
    assert_eq!(e.debug_hand(1).len(), 1, "魂身：准备阶段摸等量（1）");
}

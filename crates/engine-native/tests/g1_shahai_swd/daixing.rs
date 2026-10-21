use super::*;

/// T12 swd deferred 消解批 108：待行（吕晓雪 `swd_yuxiaoxue`）——结束阶段弃
/// 1..张 he 换取等量护甲；下回合准备阶段按剩余计数移除。
#[test]
fn swd_deferred_daixing_hujia_cycle() {
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

    let mut e = Engine::new_with_extensions(62068, 4, &[&pack]).expect("建局");
    for sk in ["daixing", "daixing2", "daixing3"] {
        e.debug_grant(0, sk);
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "shan");

    let mut answered = false;
    for _ in 0..20 {
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.prompt == "daixing_discard" {
                answered = true;
                vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]
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
            let _ = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            if answered {
                break;
            }
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert!(answered, "待行：结束阶段弃牌询问出现");
    assert_eq!(e.state_core().player(0).daixing_count, 2, "待行：计数 2");
    assert_eq!(e.state_core().player(0).hujia, 2, "待行：护甲 +2");

    // 推进到 0 号下回合准备阶段 → 清甲。
    for _ in 0..20 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
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
            let _ = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(
        e.state_core().player(0).daixing_count,
        0,
        "待行：准备阶段清零"
    );
    assert_eq!(e.state_core().player(0).hujia, 0, "待行：护甲移除");
}

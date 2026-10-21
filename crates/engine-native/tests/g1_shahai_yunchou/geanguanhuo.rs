use super::*;

/// 隔岸观火（双目标拼点链）：两目标各选拼点牌，大者赢——
/// 赢方获得输方一张手牌；平局目标一弃牌。字节恢复于第二段。
#[test]
fn geanguanhuo_two_target_compare_chain() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4301, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (gg, c_low, c_high) = {
        let catalog = e.catalog();
        let gg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_geanguanhuo"))
            .expect("隔岸观火在目录") as u32;
        let low = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan && c.rank <= 5)
            .unwrap_or(0) as u32;
        let high = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.rank >= 10)
            .unwrap_or(0) as u32;
        (gg, low, high)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != gg && *c != c_low && *c != c_high);
    }
    snap.core
        .deck
        .retain(|c| *c != gg && *c != c_low && *c != c_high);
    snap.core
        .discard
        .retain(|c| *c != gg && *c != c_low && *c != c_high);
    snap.core.players[0].hand.push(gg);
    snap.core.players[1].hand.push(c_low);
    snap.core.players[2].hand.push(c_high);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gg,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 无懈窗口（如有人持无懈）→ Pass；否则直接进入拼点
    while let Some(c) = e.open_choice() {
        if c.spec.prompt == "yc_gean_t1_pick" {
            break;
        }
        assert_eq!(c.spec.prompt, "nullify_window");
        let cid = c.id;
        let obs = c.observer;
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 目标一选牌（观察者 1）
    let c = e.open_choice().expect("目标一选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("yc_gean_t1_pick", 1));
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c_low)],
        },
    );
    // 目标二选牌挂起 → 字节恢复 → 应答高点牌
    let c2 = e.open_choice().expect("目标二选牌挂起");
    assert_eq!((c2.spec.prompt, c2.observer), ("yc_gean_t2_pick", 2));
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("第二段字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(
        &mut restored,
        2,
        GameAction::Answer {
            choice_id: cid2,
            selection: vec![ChoiceOption::Card(c_high)],
        },
    );
    // 目标二手牌大 → 赢方（2）获得输方（1）一张手牌
    assert!(
        restored.debug_hand(2).len() >= 2,
        "赢方 2 应获得输方 1 的一张手牌"
    );
}

/// 隔岸观火：目标数错误（1 个/3 个）→ IllegalTargets。
#[test]
fn geanguanhuo_rejects_wrong_target_count() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4302, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (gg,) = {
        let catalog = e.catalog();
        let gg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_geanguanhuo"))
            .expect("隔岸观火在目录") as u32;
        (gg,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != gg);
    snap.core.players[0].hand.push(gg);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 1 个目标 → 拒
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gg,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    // 3 个目标 → 拒
    let out2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gg,
            targets: vec![1, 2, 0],
        },
    );
    assert!(matches!(
        out2,
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
}

use super::*;

/// 草船借箭（分支选择原语）：目标有手牌杀 → PickTag 分支 0 =
/// 交出所有手牌杀并受 1 点反噬杀（简化：直接伤害）。
#[test]
fn caochuan_branch_zero_gives_shas_and_reprisal() {
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
    let mut e = Engine::new_with_extensions(4401, 3, &[&pack]).expect("建局");
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
    let (cc, sha1, sha2) = {
        let catalog = e.catalog();
        let cc = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_caochuanjiejian"))
            .expect("草船借箭在目录") as u32;
        let shas: Vec<u32> = catalog
            .cards
            .iter()
            .enumerate()
            .filter(|(i, c)| c.kind == CardKind::Sha && *i as u32 != cc)
            .take(2)
            .map(|(i, _)| i as u32)
            .collect();
        (cc, shas[0], shas[1])
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != cc && *c != sha1 && *c != sha2);
    }
    snap.core
        .deck
        .retain(|c| *c != cc && *c != sha1 && *c != sha2);
    snap.core
        .discard
        .retain(|c| *c != cc && *c != sha1 && *c != sha2);
    snap.core.players[0].hand.push(cc);
    snap.core.players[1].hand.push(sha1);
    snap.core.players[1].hand.push(sha2);
    e.debug_set_hp(0, 4);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cc,
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
    // 无懈窗口 → Pass（无人持无懈时窗口直过，无挂起）
    if let Some(c) = e.open_choice() {
        if c.spec.prompt == "nullify_window" {
            let cid0 = c.id;
            let obs0 = c.observer;
            submit(
                &mut e,
                obs0,
                GameAction::Answer {
                    choice_id: cid0,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }
    // 目标分支选择（PickTag，观察者 1）
    let c = e.open_choice().expect("分支选择挂起");
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    // 分支 0：交出所有手牌杀 + 使用者受 1 点反噬杀
    assert!(
        e.debug_hand(0).contains(&sha1) && e.debug_hand(0).contains(&sha2),
        "使用者获得目标的杀"
    );
    assert_eq!(e.debug_hp(0), 3, "反噬杀 1 点伤害");
}

/// 草船借箭：目标无手牌杀 → 直接走展示+被弃分支（仅剩选项 1）。
#[test]
fn caochuan_no_sha_branch_shows_and_discards() {
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
    let mut e = Engine::new_with_extensions(4402, 3, &[&pack]).expect("建局");
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
    let (cc, victim) = {
        let catalog = e.catalog();
        let cc = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_caochuanjiejian"))
            .expect("草船借箭在目录") as u32;
        let v = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (cc, v)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != cc && *c != victim);
    }
    snap.core.deck.retain(|c| *c != cc && *c != victim);
    snap.core.discard.retain(|c| *c != cc && *c != victim);
    snap.core.players[0].hand.push(cc);
    snap.core.players[1].hand.push(victim);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cc,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 目标无杀 → 仅选项 1（被弃一张）
    let c = e.open_choice().expect("分支选择挂起");
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Tag(1)],
        "无杀时仅剩被弃分支"
    );
    let cid = c.id;
    let obs = c.observer;
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert!(e.debug_hand(1).is_empty(), "展示+被弃一张");
    assert_eq!(e.debug_hp(1), 4, "未受伤");
}

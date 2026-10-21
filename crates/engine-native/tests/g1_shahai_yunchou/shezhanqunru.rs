use super::*;

/// 舌战群儒（确定性首轮拼点）：使用者以首张手牌点数为基准，
/// 每个有手牌的响应者各自拼点——大者摸 1，小/等者使用者摸 1。
#[test]
fn shezhanqunru_compares_all_responders() {
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
    let mut e = Engine::new_with_extensions(4501, 3, &[&pack]).expect("建局");
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
    let (sz,) = {
        let catalog = e.catalog();
        let sz = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_shezhanqunru"))
            .expect("舌战群儒在目录") as u32;
        (sz,)
    };
    // 使用者首张 = 高点数；响应者首张 = 低点数 → 使用者摸 2（每响应者各 1）
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != sz);
    }
    snap.core.deck.retain(|c| *c != sz);
    let mut picked: Vec<u32> = Vec::new();
    let hi = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.rank >= 13)
        .expect("高点数牌存在") as u32;
    picked.push(hi);
    for c in e.catalog().cards.iter() {
        if c.rank <= 2 {
            let id = e
                .catalog()
                .cards
                .iter()
                .position(|x| x.name == c.name)
                .unwrap() as u32;
            if picked.len() < 3 && !picked.contains(&id) {
                picked.push(id);
            }
        }
    }
    // 追加低点数牌直到 3 张（去重）
    for (i, c) in e.catalog().cards.iter().enumerate() {
        if picked.len() >= 3 {
            break;
        }
        if c.rank <= 5 && !picked.contains(&(i as u32)) {
            picked.push(i as u32);
        }
    }
    let hi = picked[0];
    let lo1 = picked[1];
    let lo2 = picked[2];
    for c in [hi, lo1, lo2, sz] {
        snap.core.deck.retain(|x| *x != c);
        snap.core.discard.retain(|x| *x != c);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|x| *x != c);
            p.equips.retain(|x| *x != c);
            p.judge.retain(|x| *x != c);
        }
        snap.core.table.retain(|x| *x != c);
    }
    snap.core.players[0].hand.push(hi);
    snap.core.players[0].hand.push(sz);
    snap.core.players[1].hand.push(lo1);
    snap.core.players[2].hand.push(lo2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let h0 = e.debug_hand(0).len();
    let _h1 = e.debug_hand(1).len();
    let _h2 = e.debug_hand(2).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sz,
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
    if let Some(c) = e.open_choice() {
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
    assert_eq!(
        e.debug_hand(0).len(),
        h0 - 1 + 2,
        "打出消耗 1 张 + 胜两局摸 2"
    );
    assert_zone_conservation_yunchou(&e);
}

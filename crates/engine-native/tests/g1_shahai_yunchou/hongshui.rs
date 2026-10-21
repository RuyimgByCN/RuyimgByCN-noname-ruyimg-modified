use super::*;

/// 洪水（自-target 延时锦囊）：置入自己判定区，梅花 2–9 判定 → 3 点雷伤。
#[test]
fn hongshui_self_delayed_trick_thunder_damage() {
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
    let mut e = Engine::new_with_extensions(3901, 3, &[&pack]).expect("建局");
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
    let (hs,) = {
        let catalog = e.catalog();
        let h = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_hongshui"))
            .expect("洪水在目录") as u32;
        (h,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != hs);
    snap.core.players[0].hand.push(hs);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 置入自己判定区
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: hs,
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
    // 无懈窗口（无人持无懈 → 直接入判定区）
    assert!(
        e.save().core.players[0].judge.contains(&hs),
        "洪水应置入自己判定区"
    );
    // 推进回合让判定发生（梅花 2–9 → 3 点雷伤；hp 4 → 1）
    // 每步前保持判定牌顶为梅花 2–9（确定性命中；否则随机摸牌会消耗顶牌）
    for _ in 0..48 {
        if e.save().core.players[0].hp <= 1 || e.save().core.players[0].judge.is_empty() {
            break;
        }
        if e.open_choice().is_none() {
            e.debug_set_deck_top_by(|def| def.suit == Suit::Club && (2..=9).contains(&def.rank));
        }
        if e.save().core.players[0].hp <= 1 || e.save().core.players[0].judge.is_empty() {
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
                other => panic!("judge answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    assert_eq!(e.debug_hp(0), 1, "洪水判定梅花命中 → 3 点雷伤");
}

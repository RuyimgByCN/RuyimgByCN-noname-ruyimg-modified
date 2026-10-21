use super::*;

/// 趁火打劫（新原语）：获得目标一张手牌（确定性首张）；
/// 无懈窗口包装。
#[test]
fn chenhuodajie_gains_target_hand_card() {
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
    let mut e = Engine::new_with_extensions(3801, 3, &[&pack]).expect("建局");
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
    let (chd, victim) = {
        let catalog = e.catalog();
        let c = catalog
            .cards
            .iter()
            .position(|x| x.name.starts_with("yc_chenhuodajie"))
            .expect("趁火打劫在目录") as u32;
        let v = catalog
            .cards
            .iter()
            .position(|x| x.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (c, v)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != chd && *c != victim);
    }
    snap.core.deck.retain(|c| *c != chd && *c != victim);
    snap.core.discard.retain(|c| *c != chd && *c != victim);
    snap.core.players[0].hand.push(chd);
    snap.core.players[1].hand.push(victim);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: chd,
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
    // 无懈窗口（无人持无懈 → 直结算）
    assert!(
        e.debug_hand(0).contains(&victim),
        "趁火打劫应获得目标首张手牌"
    );
    assert!(!e.debug_hand(1).contains(&victim));
}

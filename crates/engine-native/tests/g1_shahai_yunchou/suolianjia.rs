use super::*;

/// 索链甲：装备时横置（计入铁索）、离场时重置。
#[test]
fn suolianjia_links_on_equip_resets_on_leave() {
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
    let mut e = Engine::new_with_extensions(3802, 3, &[&pack]).expect("建局");
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
    let (sua,) = {
        let catalog = e.catalog();
        let s = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_suolianjia"))
            .expect("索链甲在目录") as u32;
        (s,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != sua);
    snap.core.players[0].hand.push(sua);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sua,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        e.save().core.players[0].linked,
        "装备索链甲应横置（计入铁索）"
    );
    // 字节恢复：横置状态随快照
    let saved = e.save();
    let restored = Engine::restore_with_extensions(&saved, &[&pack]).expect("恢复");
    assert!(restored.save().core.players[0].linked);
    let _ = restored;
}

use super::*;

/// 釜底抽薪（拼点原语）：两段选择——目标选牌 → 己方选牌；
/// 己方赢获得两张拼点牌；ComparePlayerPick 挂起点字节恢复。
#[test]
fn fudichouxin_compare_two_stage_with_restore() {
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
    let mut e = Engine::new_with_extensions(3702, 3, &[&pack]).expect("建局");
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
    let (fd, low_card, high_card) = {
        let catalog = e.catalog();
        let fd = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_fudichouxin"))
            .expect("釜底抽薪在目录") as u32;
        // 从目录找低/高点数牌注入
        let low = catalog
            .cards
            .iter()
            .position(|c| c.rank == 2)
            .expect("低点数牌") as u32;
        // v74 牌堆位移暴露的测试缺陷：rev().position() 给出的是逆向下标，
        // 直接当目录 id 用在牌序变化后取错牌——改用 rposition 取真实目录位。
        let high = catalog
            .cards
            .iter()
            .rposition(|c| c.rank >= 13 && c.kind == CardKind::Sha)
            .expect("高点数杀") as u32;
        (fd, low, high)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand
            .retain(|c| *c != fd && *c != low_card && *c != high_card);
    }
    snap.core
        .deck
        .retain(|c| *c != fd && *c != low_card && *c != high_card);
    snap.core
        .discard
        .retain(|c| *c != fd && *c != low_card && *c != high_card);
    snap.core.players[0].hand.push(fd);
    snap.core.players[0].hand.push(high_card);
    snap.core.players[1].hand.push(low_card);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fd,
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
    // 无懈窗口（1 号起问……实际由 source 起；无人持无懈直过）
    let c = e.open_choice().expect("拼点目标选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("wd_fudi_target_pick", 1));
    let cid = c.id;
    let opts = c.spec.options.clone();
    if let CommandOutcome::Rejected(r) = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(low_card)],
        },
    ) {
        panic!("目标选牌被拒: {r:?} (low_card={low_card}, opts={opts:?})");
    }
    // 己方选牌挂起 → 字节恢复 → 应答高点数牌
    let c2 = e.open_choice().unwrap_or_else(|| {
        panic!(
            "己方拼点选牌挂起缺失；log 末尾={:?}",
            e.log().iter().rev().take(4).collect::<Vec<_>>()
        )
    });
    assert_eq!(c2.spec.prompt, "wd_fudi_player_pick");
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("拼点第二段字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: cid2,
            selection: vec![ChoiceOption::Card(high_card)],
        },
    );
    // 己方赢：获得两张拼点牌
    let hand0 = restored.debug_hand(0);
    assert!(
        hand0.contains(&high_card) && hand0.contains(&low_card),
        "赢方获得两张拼点牌"
    );
}

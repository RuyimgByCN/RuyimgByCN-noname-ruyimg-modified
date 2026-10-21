use super::*;

/// 偷梁换柱（换牌双选）：己方选牌 → 目标选牌 → 交换；
/// 两段挂起点字节恢复。
#[test]
fn touliang_swap_two_stage_with_restore() {
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
    let mut e = Engine::new_with_extensions(3902, 3, &[&pack]).expect("建局");
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
    let (tl, own_card, target_card) = {
        let catalog = e.catalog();
        let tl = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_toulianghuanzhu"))
            .expect("偷梁换柱在目录") as u32;
        let oc = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀牌") as u32;
        let tc = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (tl, oc, tc)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand
            .retain(|c| *c != tl && *c != own_card && *c != target_card);
    }
    snap.core
        .deck
        .retain(|c| *c != tl && *c != own_card && *c != target_card);
    snap.core
        .discard
        .retain(|c| *c != tl && *c != own_card && *c != target_card);
    snap.core.players[0].hand.push(tl);
    snap.core.players[0].hand.push(own_card);
    snap.core.players[1].hand.push(target_card);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tl,
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
    // 第一段：己方选牌
    let c = e.open_choice().expect("己方选牌挂起");
    assert_eq!(c.spec.prompt, "wd_touliang_own");
    let cid = c.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(own_card)],
        },
    );
    // 第二段：目标选牌 → 字节恢复 → 应答
    let c2 = e.open_choice().expect("目标选牌挂起");
    assert_eq!(c2.spec.prompt, "wd_touliang_target");
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("偷梁换柱第二段字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: cid2,
            selection: vec![ChoiceOption::Card(target_card)],
        },
    );
    // 交换断言
    assert!(
        restored.debug_hand(0).contains(&target_card),
        "己方应获得目标的手牌"
    );
    assert!(
        restored.debug_hand(1).contains(&own_card),
        "目标应获得己方的手牌"
    );
    assert!(
        !restored.debug_hand(0).contains(&own_card)
            && !restored.debug_hand(1).contains(&target_card),
    );
}

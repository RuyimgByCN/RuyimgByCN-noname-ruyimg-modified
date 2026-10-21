use super::*;

/// 舒筋散：目标弃 2 张摸 2；取消则无事发生；挂起点字节恢复。
#[test]
fn shujinsan_discard_any_draws_equal_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(5202, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sj, c1, c2, c3) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_shujinsan"))
            .expect("舒筋散在目录") as u32;
        let c1 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀在目录") as u32;
        let c2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        let c3 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃在目录") as u32;
        (sj, c1, c2, c3)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sj, c1, c2, c3]);
    snap.core.players[0].hand.push(sj);
    snap.core.players[1].hand.push(c1);
    snap.core.players[1].hand.push(c2);
    snap.core.players[1].hand.push(c3);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sj,
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
    pass_nullify_window(&mut e);
    // 目标选牌挂起（观察者 = 目标）
    let c = e.open_choice().expect("舒筋散目标选牌挂起");
    assert_eq!(c.spec.prompt, "gj_shujinsan_discard");
    assert_eq!(c.observer, 1);
    assert_eq!(c.spec.min, 1, "至少弃 1 张");
    assert_eq!(c.spec.max, 3, "至多弃全部手牌");
    assert!(c.spec.allow_cancel, "可取消");
    // 挂起点字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("舒筋散挂起点字节恢复");
    let cid = restored.open_choice().unwrap().id;
    let out = submit(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let hand1 = restored.debug_hand(1);
    assert_eq!(hand1.len(), 3, "弃 2 摸 2：3-2+2=3");
    assert!(
        !hand1.contains(&c1) && !hand1.contains(&c2),
        "所选两张已弃置"
    );
    assert!(hand1.contains(&c3), "未选牌保留");
    assert_zone_conservation(&restored);

    // 取消分支：重开一局，目标取消 → 无弃无摸
    let mut e = Engine::new_with_extensions(5203, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sj, c1, c2, c3]);
    snap.core.players[0].hand.push(sj);
    snap.core.players[1].hand.push(c1);
    snap.core.players[1].hand.push(c2);
    snap.core.players[1].hand.push(c3);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sj,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("舒筋散目标选牌挂起");
    assert_eq!(c.spec.prompt, "gj_shujinsan_discard");
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(1).len(), 3, "取消：手牌不变");
    assert_eq!(e.debug_hand(0).len(), 0, "打出舒筋散后使用者手牌空");
    assert_zone_conservation(&e);
}

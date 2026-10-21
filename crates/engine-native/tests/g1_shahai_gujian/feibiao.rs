use super::*;

/// 飞镖：有黑色手牌 → 弃指定黑牌免伤；取消 → 受 1 点伤害；
/// 无黑色手牌 → 直接受 1 点伤害（无选择）。
#[test]
fn feibiao_discard_black_or_damage() {
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
    let mut e = Engine::new_with_extensions(5204, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (fb, black, red) = {
        let catalog = e.catalog();
        let fb = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_feibiao"))
            .expect("飞镖在目录") as u32;
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade | Suit::Club))
            .expect("黑杀在目录") as u32;
        let red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        (fb, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[fb, black, red]);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(black);
    snap.core.players[1].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let hp1 = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fb,
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
    // 目标选牌挂起：选项只含黑色手牌
    let c = e.open_choice().expect("飞镖目标选牌挂起");
    assert_eq!(c.spec.prompt, "gj_feibiao_discard");
    assert_eq!(c.observer, 1);
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Card(black)],
        "选项仅黑色手牌"
    );
    assert!(c.spec.allow_cancel, "可取消（取消则受伤）");
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(black)],
        },
    );
    assert_eq!(e.debug_hp(1), hp1, "弃黑牌免伤");
    assert!(!e.debug_hand(1).contains(&black), "黑牌已弃");
    assert_zone_conservation(&e);

    // 取消分支：目标取消 → 受 1 点伤害
    let mut e = Engine::new_with_extensions(5205, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[fb, black]);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let hp1 = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fb,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("飞镖目标选牌挂起");
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hp(1), hp1 - 1, "取消 → 失去 1 点体力（规则 v77）");
    assert!(
        !e.log()
            .iter()
            .any(|l| matches!(l, LogEvent::DamageDealt { target: 1, .. })),
        "飞镖取消分支为失去体力而非伤害"
    );

    // 无黑色手牌：直接受伤，无选择
    let mut e = Engine::new_with_extensions(5206, 3, &[&pack]).expect("建局3");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[fb, red]);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复3");
    let hp1 = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fb,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    assert!(e.open_choice().is_none(), "无黑色手牌不应挂起选择");
    assert_eq!(e.debug_hp(1), hp1 - 1, "无黑牌直接受伤");
    assert_zone_conservation(&e);
}

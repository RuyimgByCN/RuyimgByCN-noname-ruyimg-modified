use super::*;

/// 金莲烛（事件取消原语）：伤害前置持有者确认——确认则弃烛、防止
/// 伤害、来源摸 1；拒绝则伤害照常结算；确认分支挂起点字节恢复。
#[test]
fn jinlianzhu_cancels_damage_with_holder_confirm() {
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
    let mut e = Engine::new_with_extensions(5208, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (jz, sha, tao) = {
        let catalog = e.catalog();
        let jz = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_jinlianzhu"))
            .expect("金莲烛在目录") as u32;
        let sha = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀在目录") as u32;
        let tao = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃在目录") as u32;
        (jz, sha, tao)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[jz, sha, tao]);
    // 0 号持杀（来源），1 号持金莲烛（持有者），2 号持桃（受害者手牌）
    snap.core.players[0].hand.push(sha);
    snap.core.players[1].hand.push(jz);
    snap.core.players[2].hand.push(tao);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(2, 4);

    // 受害者 = 2 号，来源 = 0 号；持烛者 1 号确认 → 伤害防止 + 来源摸 1
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
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
    // 杀响应窗口：受害者无闪，直过（目标选牌可能不存在）——等待金莲烛确认
    let c = loop {
        let c = e.open_choice().expect("金莲烛确认挂起");
        if c.spec.prompt == "gj_jinlianzhu_confirm" {
            break c;
        }
        // 其他窗口（如杀的 respond）无人可应答时不会挂起——不应到达
        panic!("意外的选择: {} (obs={})", c.spec.prompt, c.observer);
    };
    assert_eq!(c.observer, 1, "持有者被询问");
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Confirm, ChoiceOption::Pass]
    );
    // 挂起点字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("金莲烛确认挂起点字节恢复");
    let cid = restored.open_choice().unwrap().id;
    let out = submit(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(!restored.debug_hand(1).contains(&jz), "金莲烛已弃置");
    assert_eq!(restored.debug_hp(2), 4, "伤害被防止");
    assert_eq!(restored.debug_hand(0).len(), 1, "来源摸 1（弃杀 1+摸 1）");
    assert_zone_conservation(&restored);

    // 拒绝分支：持烛者 Pass → 伤害照常
    let mut e = Engine::new_with_extensions(5209, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[jz, sha]);
    snap.core.players[0].hand.push(sha);
    snap.core.players[1].hand.push(jz);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    e.debug_set_hp(2, 4);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("金莲烛确认挂起");
    assert_eq!(c.spec.prompt, "gj_jinlianzhu_confirm");
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        },
    );
    // 受害者无闪 → 伤害结算
    assert_eq!(e.debug_hp(2), 3, "拒绝后伤害照常");
    assert!(e.debug_hand(1).contains(&jz), "金莲烛保留在手");
    assert_eq!(e.debug_hand(0).len(), 0, "来源未摸牌");
    assert_zone_conservation(&e);

    // 金莲烛不能主动使用
    let mut e = Engine::new_with_extensions(5210, 3, &[&pack]).expect("建局3");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[jz]);
    snap.core.players[0].hand.push(jz);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复3");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jz,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "主动使用应被拒: {out:?}"
    );
    assert_zone_conservation(&e);
}

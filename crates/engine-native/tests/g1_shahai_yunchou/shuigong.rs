use super::*;

/// 水攻（新原语）：多目标强制弃装备（登记差异：确定性首张）。
/// 无装备目标不参与；字节恢复于无懈窗口挂起点。
#[test]
fn shuigong_multi_target_forced_equip_discard() {
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
    let mut e = Engine::new_with_extensions(3701, 3, &[&pack]).expect("建局");
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
    let (sg, eq1, eq2) = {
        let catalog = e.catalog();
        let sg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_shuigong"))
            .expect("水攻在目录") as u32;
        let e1 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::HorsePlus)
            .expect("马牌") as u32;
        let e2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::HorseMinus)
            .expect("马牌") as u32;
        (sg, e1, e2)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != sg && *c != eq1 && *c != eq2);
        p.equips.retain(|c| *c != sg && *c != eq1 && *c != eq2);
        p.judge.retain(|c| *c != sg && *c != eq1 && *c != eq2);
    }
    snap.core
        .deck
        .retain(|c| *c != sg && *c != eq1 && *c != eq2);
    snap.core
        .discard
        .retain(|c| *c != sg && *c != eq1 && *c != eq2);
    snap.core
        .table
        .retain(|c| *c != sg && *c != eq1 && *c != eq2);
    snap.core.players[0].hand.push(sg);
    snap.core.players[1].equips.push(eq1);
    snap.core.players[2].equips.push(eq2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 水攻：全部有装备目标（1、2 号）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sg,
            targets: vec![1, 2],
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
    let eq1_after = e.save().core.players[1].equips.contains(&eq1);
    let eq2_after = e.save().core.players[2].equips.contains(&eq2);
    assert!(!eq1_after && !eq2_after, "两目标装备均被强制弃置");
    assert_eq!(e.debug_hp(1), 4, "水攻不造成伤害");

    // 字节恢复（结算完成态）
    let bytes = encode_archive(&e.save());
    Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
        .expect("字节恢复");
}

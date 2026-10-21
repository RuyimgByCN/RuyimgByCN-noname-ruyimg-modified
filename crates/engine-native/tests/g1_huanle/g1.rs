use super::*;

/// G1 纵向：建局 → 宿主技能授予 → 苦无（杀伤害强制弃牌）→
/// 手里剑（嵌套选择）→ 字节恢复 → 内容锁绑定。
#[test]
fn g1_huanle_vertical_host_behaviors_and_restore() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    // 建局（扩展卡入目录）
    let mut e = Engine::new_with_extensions(3201, 3, &[&pack]).expect("建局");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(
        e.catalog().cards.len(),
        base_cards + 7,
        "苦无 + 手里剑×2 + 猴子 + 写轮眼 + 面具 + 酒尾"
    );

    // 授予宿主技能（装备技能挂载生命周期未做，以授予验证行为）
    e.debug_grant(0, "huanle_kuwu");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 苦无：杀造成伤害 → 目标弃一张手牌
    let sha = e.debug_place(0, "sha");
    let victim_card = e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(
        !e.debug_hand(1).contains(&victim_card),
        "苦无：目标应弃一张手牌"
    );
    assert_eq!(e.debug_hp(1), 3);

    // 手里剑（嵌套选择）：注入扩展牌 → 出牌 → 目标选择弃装备或受伤
    let (shoulijian_id,) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_shoulijian"))
            .expect("扩展手里剑在目录") as u32;
        (sj,)
    };
    let mut snap = e.save();
    // 注入前置：目标牌从所有区清除（归属守恒校验要求唯一归属；
    // 牌堆构成变化会改变发牌位置，不能假设目标牌恰在牌堆）。
    let purge = |snap: &mut engine_native::StateSnapshot, id: u32| {
        snap.core.deck.retain(|c| *c != id);
        snap.core.discard.retain(|c| *c != id);
        snap.core.table.retain(|c| *c != id);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != id);
            p.equips.retain(|c| *c != id);
            p.judge.retain(|c| *c != id);
        }
    };
    purge(&mut snap, shoulijian_id);
    snap.core.players[0].hand.push(shoulijian_id);
    // 目标持装备：同一次快照注入（保持归属完整）
    let equip_id = {
        let cat = e.catalog();
        cat.card_by_name("horse_plus").expect("目录有马牌")
    };
    purge(&mut snap, equip_id);
    snap.core.players[1].equips.push(equip_id);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shoulijian_id,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "手里剑应挂起目标选择: {out:?}"
    );
    let c = e.open_choice().expect("嵌套选择挂起");
    assert_eq!(c.spec.prompt, "host_target_discard_equip");
    assert_eq!(c.observer, 1);
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(equip_id)),
        "选项应含目标的装备"
    );

    // 嵌套选择的字节恢复（扩展机器续行编码）
    let bytes = engine_native::archive::encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("扩展机器字节恢复");
    let cid = restored.open_choice().unwrap().id;
    // 目标选择弃装备 → 免伤
    submit(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(equip_id)],
        },
    );
    assert_eq!(restored.debug_hp(1), 3, "弃装备后免伤");
    assert!(
        !restored.state_core().players[1].equips.contains(&equip_id),
        "装备已弃"
    );

    // 内容锁：带扩展存档不能被纯标准目录恢复
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
}

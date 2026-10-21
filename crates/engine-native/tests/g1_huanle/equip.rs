use super::*;

/// G1 装备技能生命周期：装备苦无 → 技能授予（无需授予命令）→
/// 生效（杀伤害后目标弃牌）→ 被拆桥拆除 → 技能撤销。
#[test]
fn equip_skill_lifecycle_grant_and_revoke() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3304, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 注入苦无并装备（无 debug_grant——技能随装备授予）
    let (kuwu,) = {
        let catalog = e.catalog();
        let k = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_kuwu"))
            .expect("苦无在目录") as u32;
        (k,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != kuwu);
    snap.core.players[0].hand.push(kuwu);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def = e
        .catalog()
        .skill_by_name("huanle_kuwu")
        .expect("宿主技能在目录");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: kuwu,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        e.save().core.skills.contains_key(&(0, def)),
        "装备苦无应授予 huanle_kuwu 实例"
    );

    // 技能生效：杀伤害 → 目标弃牌（此前用 debug_grant 验证，此处走装备授予）
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
        "装备授予的苦无应生效"
    );

    // 苦无离场 → 技能撤销。规则路径：1 号出手里剑令 0 号弃装备
    //（推进到 1 号出牌阶段，途中挂起选择取首选项）
    for _ in 0..128 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    let (sj,) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_shoulijian"))
            .expect("手里剑在目录") as u32;
        (sj,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != sj);
    snap.core.players[1].hand.push(sj);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sj,
            targets: vec![0],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("手里剑嵌套选择");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(kuwu)),
        "选项应含苦无装备"
    );
    let cid = c.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(kuwu)],
        },
    );
    assert!(
        !e.save().core.skills.contains_key(&(0, def)),
        "苦无离场后 huanle_kuwu 实例应被撤销"
    );
    assert!(
        !e.save().core.players[0].equips.contains(&kuwu),
        "苦无已离场"
    );
    assert_eq!(e.debug_hp(0), 4, "弃装备免伤");
}

/// 装备技能一致性：装备在场但技能实例被移除的快照 → 恢复拒绝
///（EquipSkillMismatch，授予/撤销路径完整性的信任边界）。
#[test]
fn equip_skill_snapshot_mismatch_rejected() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3305, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (kuwu,) = {
        let catalog = e.catalog();
        let k = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_kuwu"))
            .expect("苦无在目录") as u32;
        (k,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != kuwu);
    snap.core.players[0].hand.push(kuwu);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: kuwu,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }));
    let mut snap = e.save();
    let def = e.catalog().skill_by_name("huanle_kuwu").unwrap();
    assert!(
        snap.core.skills.remove(&(0, def)).is_some(),
        "技能实例在快照中"
    );
    match Engine::restore_with_extensions(&snap, &[&pack]) {
        Err(engine_native::SnapshotError::EquipSkillMismatch { player }) => {
            assert_eq!(player, 0);
        }
        Err(e) => panic!("应报 EquipSkillMismatch: {e:?}"),
        Ok(_) => panic!("装备在而技能实例缺失必须拒绝"),
    }
    // 字节路径同样拒绝
    let bytes = engine_native::archive::encode_archive(&snap);
    match Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    ) {
        Err(engine_native::SnapshotError::EquipSkillMismatch { .. }) => {}
        Err(e) => panic!("字节路径应报 EquipSkillMismatch: {e:?}"),
        Ok(_) => panic!("字节路径也须拒绝"),
    }
}

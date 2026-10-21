use super::*;

/// 收官双卡：帝女玄霜（主动使用：回复 1 + 弃任意摸等量；濒死按桃
/// 结算）与玉衡（装备技：确定性取其他角色首张黑桃手牌）。
#[test]
fn dinvxuanshuang_and_gjyuheng_closeout() {
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
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 帝女玄霜：主动使用（受伤自用：回复 1 + 弃 1 摸 1） ——
    let mut e = Engine::new_with_extensions(5234, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (dvx, disc) = {
        let catalog = e.catalog();
        let dvx = find(&e, "gj_dinvxuanshuang");
        let disc = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        (dvx, disc)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[dvx, disc]);
    snap.core.players[0].hand.push(dvx);
    snap.core.players[0].hand.push(disc);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(0, 2); // 已受伤
    let hp0 = e.debug_hp(0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dvx,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), hp0 + 1, "回复 1 点体力");
    // rider：弃任意张（弃 1 摸 1）
    let c = e.open_choice().expect("帝女玄霜 rider 挂起");
    assert_eq!(c.spec.prompt, "gj_dinvxuanshuang_rider");
    assert_eq!(c.observer, 0);
    assert!(c.spec.allow_cancel, "rider 可取消");
    let cid = c.id;
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(disc)],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(e.debug_hand(0).len(), 1, "弃 1 摸 1（净不变）");
    assert_eq!(e.debug_hp(0), hp0 + 1, "回复保持");
    assert_zone_conservation(&e);

    eprintln!("ENTER SEG2");
    // —— 玉衡：装备技——确定性取其他角色首张黑桃手牌 ——
    let mut e = Engine::new_with_extensions(5235, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (yhx, spade) = {
        let catalog = e.catalog();
        let yhx = find(&e, "gj_gjyuheng");
        let spade = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Spade)
            .expect("黑桃杀在目录") as u32;
        (yhx, spade)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[yhx, spade]);
    snap.core.players[0].hand.push(yhx);
    snap.core.players[1].hand.push(spade);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_yh = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_YUHENG_SKILL_KEY)
        .expect("玉衡装备技在目录");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yhx,
            targets: vec![],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(
        e.save().core.skills.contains_key(&(0, def_yh)),
        "装备授予玉衡技"
    );
    // 发动：座位序首名有黑桃手牌的其他角色（1 号）交出首张黑桃
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yh });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.debug_hand(0).contains(&spade), "黑桃手牌被取走至使用者");
    assert!(!e.debug_hand(1).contains(&spade), "目标失去黑桃牌");
    // once_per_turn：再次发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yh });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "once_per_turn 拒绝: {out:?}"
    );
    assert_zone_conservation(&e);

    eprintln!("ENTER SEG3");
    // —— 帝女玄霜濒死路径：按桃结算（救回，无 rider） ——
    let mut e = Engine::new_with_extensions(5236, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (dvx2, sha_black) = {
        let catalog = e.catalog();
        let dvx2 = find(&e, "gj_dinvxuanshuang");
        let sha_black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        (dvx2, sha_black)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[dvx2, sha_black]);
    snap.core.players[0].hand.push(dvx2);
    snap.core.players[1].hand.push(sha_black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(0, 1);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
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
            drop(c);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hp0 = e.debug_hp(0);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_black,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    // 0 号濒死 → 帝女玄霜出现在救援选项并按桃结算救回。响应者队列
    // 按「濒死者起的座位序中持桃者」询问：其余玩家摸到的普通桃会被
    // 先问（发牌构成不同而异），逐个 Pass 后才轮到持玄霜的 0 号。
    let cid_dvx = loop {
        let c = e.open_choice().expect("濒死救援挂起");
        assert_eq!(c.spec.prompt, "dying_save");
        let has_dvx = c.spec.options.contains(&ChoiceOption::Card(dvx2));
        let cid = c.id;
        let obs = c.observer;
        if has_dvx {
            assert_eq!(obs, 0, "帝女玄霜应只在 0 号手中");
            break cid;
        }
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    };
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid_dvx,
            selection: vec![ChoiceOption::Card(dvx2)],
        },
    );
    assert_eq!(e.debug_hp(0), hp0, "帝女玄霜按桃救回（体力不变）");
    assert!(e.save().core.players[0].alive, "未死亡");
    assert_zone_conservation(&e);
}

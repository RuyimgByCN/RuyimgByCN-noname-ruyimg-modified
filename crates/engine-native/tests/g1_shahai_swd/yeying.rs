use super::*;

/// T12 swd 武将面批㉗（小黄龙夜影/虎魄归影/司徒强凤泽+玲月/暗卡暗莲映；
/// 聚悉/遁行/锦鳞/颂词 deferred）：五行为行为面 + 反例。
#[test]
fn swd_generals_b67_yeying_guiying_fengze_lingyue_anlianying() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62035, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    let black_card = |e: &mut Engine| -> u32 {
        // 黑色顺手牵羊（viewAs 链外层守卫跳过 Sha/Le；机关/异类排除）
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                ) && e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Snatch
            })
            .expect("黑顺手牵羊在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.discard.retain(|x| *x != b);
        core.table.retain(|x| *x != b);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != b);
        }
        core.players[0].hand.push(b);
        b
    };

    // ① 夜影（小黄龙）：主动发动选黑牌当乾坤镖（恰 1 目标——目标+距离 1 邻座各弃一张）
    e.debug_grant(0, "yeying");
    let bc = black_card(&mut e);
    let d1 = e.debug_place(1, "tao"); // p1 唯一手牌（必被弃）
    let dy = e.catalog().skill_by_name("yeying").expect("夜影在目录");
    submit(&mut e, 0, GameAction::ActivateSkill { skill: dy });
    let ch = e.open_choice().expect("夜影选黑牌窗");
    assert_eq!(ch.spec.prompt, "yeying_pick");
    let (cid, obs) = (ch.id, ch.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(bc)],
        },
    );
    let tgt = e.open_choice().expect("夜影选目标窗");
    assert_eq!(tgt.spec.prompt, "yeying_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.debug_hand(1).contains(&d1),
        "夜影：乾坤镖路由（p1 被弃牌）"
    );

    // ② 归影（虎魄）：主动发动选黑牌当偷梁换柱（单目标换牌：双方须有手牌）
    {
        e.state_core_mut().skills.remove(&(0, dy));
    }
    e.debug_grant(0, "guiying");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let bc2 = black_card(&mut e);
    let own2 = e.debug_place(0, "tao"); // 己方换出牌
    let v2 = e.debug_place(1, "tao");
    let v2b = e.debug_place(1, "shan");
    let dg = e.catalog().skill_by_name("guiying").expect("归影在目录");
    submit(&mut e, 0, GameAction::ActivateSkill { skill: dg });
    let ch = e.open_choice().expect("归影选黑牌窗");
    assert_eq!(ch.spec.prompt, "guiying_pick");
    let (cid, obs) = (ch.id, ch.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(bc2)],
        },
    );
    let tgt = e.open_choice().expect("归影选目标窗");
    assert_eq!(tgt.spec.prompt, "guiying_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // 偷梁换柱机器：own 选牌窗→目标选牌窗（settle 应答首选项）
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(0).contains(&v2) && !e.debug_hand(1).contains(&v2),
        "归影：偷梁换柱路由（v2 换入己方）"
    );
    assert!(
        e.debug_hand(1).contains(&own2) && e.debug_hand(1).contains(&v2b),
        "归影：换牌语义（own 入目标、v2b 保留）"
    );

    // ③ 凤泽（司徒强）：主动发动选黑牌当桃园结义（全体其他回 1）
    {
        e.state_core_mut().skills.remove(&(0, dg));
    }
    e.debug_grant(0, "fengze");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let bc3 = black_card(&mut e);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 3);
    let dfz = e.catalog().skill_by_name("fengze").expect("凤泽在目录");
    submit(&mut e, 0, GameAction::ActivateSkill { skill: dfz });
    let ch = e.open_choice().expect("凤泽选黑牌窗");
    assert_eq!(ch.spec.prompt, "fengze_pick");
    let (cid, obs) = (ch.id, ch.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(bc3)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "凤泽：桃园结义 p1 回 1");
    assert_eq!(e.debug_hp(2), 4, "凤泽：桃园结义 p2 回 1");

    // ④ 玲月（司徒强）：锁定——杀指定后目标弃 he 首张
    e.debug_grant(0, "lingyue");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    let t1 = e.debug_place(1, "tao");
    let t2 = e.debug_place(1, "shan");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(!e.debug_hand(1).contains(&t1), "玲月：目标弃 he 首张");
    assert_eq!(e.debug_hp(1), 3, "玲月：杀照常结算（p2 无闪受 1）");
    let _ = t2;

    // ⑤ 暗莲映（暗卡）：手牌被弃空 → 摸 2（过河拆桥路径——弃牌阶段
    // 无法清空手牌：弃牌数=手牌-hp 恒留 1）
    e.debug_grant(0, "anlianying");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let only = e.debug_place(0, "tao");
    let chai = {
        let d = (0..e.catalog().cards.len() as u32)
            .find(|c| e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Dismantle)
            .expect("拆桥在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != d);
        core.discard.retain(|x| *x != d);
        core.table.retain(|x| *x != d);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != d);
        }
        core.players[1].hand.push(d);
        d
    };
    // 推进到 p1 回合出拆桥
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: chai,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(!e.debug_hand(0).contains(&only), "暗莲映：唯一手牌被拆");
    assert_eq!(e.debug_hand(0).len(), 2, "暗莲映：手牌空后摸 2");
}

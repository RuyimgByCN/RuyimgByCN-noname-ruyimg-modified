use super::*;

/// T12 swd 武将面批 83（久悠乐学/临月枕流+麻醉）：两将行为面 + 反例。
#[test]
fn swd_generals_b83_lexue_zhenjiu_lmazui() {
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
    assert!(
        pack.generals.len() >= 73,
        "73 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62050, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 枕流：红牌挂附 p1 → p1 下个准备阶段回 1+入手
    e.debug_grant(0, "zhenjiu");
    let red = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                )
            })
            .expect("红桃牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 2);
    let def_zj = e.catalog().skill_by_name("zhenjiu").expect("枕流在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_zj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "枕流发动 {out:?}"
    );
    let c = e.open_choice().expect("枕流选牌窗");
    assert_eq!(c.spec.prompt, "zhenjiu_pick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(red)],
        },
    );
    let c = e.open_choice().expect("枕流目标窗");
    assert_eq!(c.spec.prompt, "zhenjiu_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(
        e.state_core().players[1].off_court.contains(&red),
        "枕流：红牌挂附 p1"
    );
    // 推进到 p1 准备阶段：回 1 且牌入手
    let mut guard = 0;
    let inv_after = loop {
        guard += 1;
        assert!(guard < 160, "p1 准备阶段未到");
        let inv = e.state_core().players[1].off_court.clone();
        if inv.is_empty() {
            break inv;
        }
        if let Some(ch) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if ch.spec.min > 0 {
                ch.spec.options.iter().take(ch.spec.min).copied().collect()
            } else {
                vec![]
            };
            let (cid, obs) = (ch.id, ch.observer);
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
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    };
    let _ = inv_after;
    assert_eq!(e.debug_hp(1), 3, "枕流：p1 准备阶段回 1（2→3）");
    assert!(e.debug_hand(1).contains(&red), "枕流：红牌入手");

    // ② 麻醉：黑牌挂附 p2 → p2 造成伤害 -1
    // 先绕回 p0 出牌阶段（枕流段结束时在 p1 的回合内）
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 200, "p0 出牌阶段未到");
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if ch.spec.min > 0 {
                ch.spec.options.iter().take(ch.spec.min).copied().collect()
            } else {
                vec![]
            };
            let (cid, obs) = (ch.id, ch.observer);
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
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    e.debug_clear_hand(0);
    e.debug_grant(0, "lmazui");
    e.debug_grant(0, "zhenjiu");
    let black = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                )
            })
            .expect("黑桃牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    // p2 出杀前先挂附（用当前回合推动到 p2 前先把 p0 的麻醉用完？——
    // 麻醉由 p0 主动发动、挂附 p2；随后轮到 p2 出杀验证 -1）
    let def_lm = e.catalog().skill_by_name("lmazui").expect("麻醉在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lm });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "麻醉发动 {out:?}"
    );
    let c = e.open_choice().expect("麻醉选牌窗");
    assert_eq!(c.spec.prompt, "lmazui_pick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(black)],
        },
    );
    let c = e.open_choice().expect("麻醉目标窗");
    assert_eq!(c.spec.prompt, "lmazui_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    assert!(
        e.state_core().players[2].off_court.contains(&black),
        "麻醉：黑牌挂附 p2"
    );
    // 推进到 p2 出牌阶段，p2 出杀命中 p1（伤害 -1 → 0：不扣血）
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 200, "p2 出牌阶段未到");
        if e.turn().current == 2 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if ch.spec.min > 0 {
                ch.spec.options.iter().take(ch.spec.min).copied().collect()
            } else {
                vec![]
            };
            let (cid, obs) = (ch.id, ch.observer);
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
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let hp1_before = e.debug_hp(1);
    let sha2 = e.debug_place(2, "sha");
    let out = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "p2 杀 p1 {out:?}"
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), hp1_before, "麻醉：p2 伤害 -1（1→0 不扣血）");
    assert!(
        !e.state_core().players[2].off_court.contains(&black),
        "麻醉：黑牌伤前返还"
    );

    // ③ 乐学：p3 回合开始授回合外池之一（或 p0 回合开始授回合内池）
    e.debug_grant(0, "lexue");
    // 当前回合是 p2——p0 之外的回合开始应授回合外池（遗计/急救/鬼道/反馈）
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 200, "回合循环未走完");
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if ch.spec.min > 0 {
                ch.spec.options.iter().take(ch.spec.min).copied().collect()
            } else {
                vec![]
            };
            let (cid, obs) = (ch.id, ch.observer);
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
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let in_pool = ["制衡", "集智"].iter().any(|n| e.debug_has_skill(0, n));
    let out_pool = ["遗计", "急救", "鬼道", "反馈"]
        .iter()
        .any(|n| e.debug_has_skill(0, n));
    assert!(
        in_pool || out_pool,
        "乐学：回合开始授予一技（在池 {in_pool} 外池 {out_pool}）"
    );
}

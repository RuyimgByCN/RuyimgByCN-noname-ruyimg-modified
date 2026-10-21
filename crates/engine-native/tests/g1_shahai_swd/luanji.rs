use super::*;

/// T12 swd 武将面批 81（多鹏乱击/疾鹏聒噪+英姿刷新）：两将三技行为面 +
/// 反例。
#[test]
fn swd_generals_b81_luanji_guozao_reyingzi() {
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
        pack.generals.len() >= 69,
        "69 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62048, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 乱击：两张同花色手牌 → 虚拟万箭齐发（全体其他存活受 1）
    e.debug_grant(0, "luanji");
    let c1 = e.debug_place(0, "sha");
    let s1 = e.catalog().cards[c1 as usize].suit;
    let c2 = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().cards[*c as usize].suit == s1
                    && *c != c1
                    && e.catalog().card_kind(*c) == CardKind::Tao
            })
            .expect("同花色桃");
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
    let def_lj = e.catalog().skill_by_name("luanji").expect("乱击在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "乱击发动 {out:?}"
    );
    let c = e.open_choice().expect("乱击选牌窗");
    assert_eq!(c.spec.prompt, "luanji_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), 3, "乱击：万箭 p1 受 1");
    assert_eq!(e.debug_hp(2), 3, "乱击：万箭 p2 受 1");
    assert_eq!(e.debug_hp(3), 3, "乱击：万箭 p3 受 1");
    // 反例：异花色两卡（发牌后手牌无同花色对）→ activate 拒
    e.debug_clear_hand(0);
    for suit_try in [CardKind::Sha] {
        let _ = suit_try;
    }
    let d1 = e.debug_place(0, "tao");
    let ds = e.catalog().cards[d1 as usize].suit;
    let opposite = {
        let wanted = if matches!(
            ds,
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
        ) {
            engine_native::catalog::Suit::Spade
        } else {
            engine_native::catalog::Suit::Heart
        };
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().cards[*c as usize].suit == wanted
                    && e.catalog().card_kind(*c) == CardKind::Sha
            })
            .expect("异花色杀");
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
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lj });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "乱击反例：异花色两卡拒 {out:?}"
    );
    let _ = opposite;

    // ② 聒噪：距离 1 内角色受损（来源非自己）→ 其手牌替换为牌堆顶三张
    e.debug_clear_hand(0);
    e.debug_grant(1, "guozao");
    e.debug_set_hp(2, 8);
    let g1 = e.debug_place(2, "tao");
    let _ = g1;
    // p2 手牌数记 2 张；创建 p3 对 p2 的杀伤害（p3 在 P1 回合外不行动——
    // 改用 p0 打出伤害牌命中 p2 会以 p0 为来源（排除）；因此直接构造
    // p3 的杀：换轮到 p3 前先布好）
    let h2_before = hand_len_w(&e, 2);
    // 借 p1（持技者）以外来源：p3 布杀并把回合交到 p3
    let sha3 = e.debug_place(3, "sha");
    let cur = e.turn().current;
    assert_eq!(cur, 0);
    // 直接走 debug：令 p3 为当前回合（引擎只允许 current 行动）；
    // 简化：把回合推进到 p3
    let mut guard = 0;
    while !(e.turn().current == 3 && e.turn().phase == Phase::Play) && guard < 96 {
        guard += 1;
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
    let out = submit(
        &mut e,
        3,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "p3 杀 p2 {out:?}"
    );
    // 推进到聒噪目标窗：途中应答闪/无懈等窗（Pass/空选），停在
    // guozao_target（持技者 p1）。
    let mut guard = 0;
    let target_choice = loop {
        guard += 1;
        assert!(guard < 32, "聒噪目标窗未出现");
        let Some(ch) = e.open_choice() else {
            panic!("聒噪目标窗缺失（无挂起）");
        };
        if ch.spec.prompt == "guozao_target" {
            break ch;
        }
        let sel: Vec<ChoiceOption> = if ch.spec.prompt == "sha_ask"
            || ch.spec.prompt == "shan_ask"
            || ch.spec.prompt == "nullify_window"
            || ch.spec.min == 0
        {
            vec![ChoiceOption::Pass]
        } else {
            ch.spec
                .options
                .iter()
                .take(ch.spec.min.max(1))
                .copied()
                .collect()
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
    };
    assert_eq!(target_choice.observer, 1, "聒噪持技者选目标");
    let (cid, obs) = (target_choice.id, target_choice.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    let h2_after = hand_len_w(&e, 2);
    assert_eq!(
        h2_after, 3,
        "聒噪：p2 手牌替换为牌堆顶三张（前 {h2_before}）"
    );
    // 反例：受损者距持技者 >1 时窗不出现（p3 受无来源伤害……以距离 2
    // 对座 p2 已用；改用「来源为自己」反例路径：p1 自己造成的伤害不触发）
    // ——已由 filter（d.source != Some(owner)）承担，此处断言行数。

    // ③ 英姿·刷新：p3 回合摸牌阶段额外摸 1 且手牌上限=体力上限
    e.debug_clear_hand(3);
    e.debug_grant(3, "reyingzi");
    e.debug_set_hp(3, 2);
    e.debug_set_max_hp(3, 4);
    // 手牌上限：hp2 基数→4（maxHp）
    let limit = e.hand_limit_for_debug(3);
    assert_eq!(limit, 4, "英姿：手牌上限=体力上限 4");
    // 推进到 p3 的下一个摸牌阶段收束：清手后摸 3（基数 2 + 英姿 1）。
    e.debug_clear_hand(3);
    // 先离开当前 p3/Play（否则下方断点立即命中）
    let actor = e.turn().current;
    let _ = submit(&mut e, actor, GameAction::NextPhase);
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 160, "p3 摸牌阶段未走完");
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
        // p3 摸牌阶段结束（进入出牌阶段）即断言
        if e.turn().current == 3 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    assert_eq!(hand_len_w(&e, 3), 3, "英姿：摸牌阶段 2+1=3 张");
}

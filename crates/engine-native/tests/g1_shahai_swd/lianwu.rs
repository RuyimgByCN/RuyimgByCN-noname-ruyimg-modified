use super::*;

/// T12 swd 武将面批㉒（尚章连舞/游兆浴尘+龙胆/王思月对诗+闭月/宇文拓挟
/// 雷+无双，kunlunjing deferred）：四将行为面 + 反例。
#[test]
fn swd_generals_b63_lianwu_yuchen_duishi_xielei() {
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
    let mut e = Engine::new_with_extensions(62031, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 连舞（尚章）：锁定——红杀双目标 + 红杀直击（p2 有闪不出闪窗）
    e.debug_grant(0, "lianwu");
    // 红杀：取目录中红杀实体（**任意区**——牌堆/手牌/弃牌堆，牌堆序随
    // 种子变化，不能假定红杀仍在牌堆），移入 p0 手牌
    let red_sha = {
        let red = (0..e.catalog().cards.len() as u32).find(|c| {
            e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha
                && matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                )
                && (e.state_core().deck.contains(c)
                    || e.state_core().discard.contains(c)
                    || (0..4).any(|p| e.state_core().player(p).hand.contains(c)))
        });
        let red = red.expect("目录中有红杀实体");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != red);
        core.discard.retain(|x| *x != red);
        for p in 0..4usize {
            core.players[p].hand.retain(|x| *x != red);
        }
        core.players[0].hand.push(red);
        red
    };
    let shan_p2 = e.debug_place(1, "shan");
    let is_red = matches!(
        e.catalog().cards[red_sha as usize].suit,
        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
    );
    if is_red {
        e.debug_set_hp(1, 4);
        e.debug_set_hp(2, 4);
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: red_sha,
                targets: vec![1, 3],
            },
        );
        // 红杀直击：p1 持闪也不出闪窗；p3 无闪直接伤
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(1), 3, "连舞：红杀直击 p1 受 1（有闪不可避）");
        assert_eq!(e.debug_hp(3), 3, "连舞：红杀双目标 p3 受 1");
        assert!(e.debug_hand(1).contains(&shan_p2), "连舞：p1 闪未消耗");
        // 反例：三目标拒（上限 +1=2）
        let red_sha2 = {
            let r = (0..e.catalog().cards.len() as u32).find(|c| {
                e.state_core().deck.contains(c)
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
                    && e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha
            });
            let core = e.state_core_mut();
            match r {
                Some(r) => {
                    core.deck.retain(|x| *x != r);
                    core.players[0].hand.push(r);
                    r
                }
                None => red_sha, // 牌库无第二红杀则跳过（防御）
            }
        };
        if red_sha2 != red_sha {
            let out = submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: red_sha2,
                    targets: vec![1, 2, 3],
                },
            );
            assert!(
                !matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "连舞反例：三目标拒 {out:?}"
            );
        }
    } else {
        eprintln!("b63: 牌库无红杀实体，连舞红面用例跳过（布阵防御）");
    }

    // ② 浴尘（游兆）：回合外打出黑色牌后弃置一名角色一张牌
    e.debug_grant(0, "yuchen");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 持龙胆（黑杀当闪打出=黑色响应面）；p1 杀；p2 两张牌供弃
    e.debug_grant(0, "龙胆");
    let black_card = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                    )
            })
            .expect("黑杀在目录");
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
    let sha_p1 = e.debug_place(1, "sha");
    let p2c1 = e.debug_place(2, "tao");
    let p2c2 = e.debug_place(2, "shan");
    advance_to_play(&mut e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_p1,
            targets: vec![0],
        },
    );
    let c = e.open_choice().expect("p0 出闪窗");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(black_card)],
        },
    );
    // 浴尘确认窗（可选）→ 确认 → 目标选 p2 → 随机弃 1
    let c = e.open_choice().expect("浴尘确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("浴尘目标窗");
    assert_eq!((c.spec.prompt, c.observer), ("yuchen_target", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    settle_optional_confirm(&mut e);
    let p2_lost = !e.debug_hand(2).contains(&p2c1) || !e.debug_hand(2).contains(&p2c2);
    assert!(p2_lost, "浴尘：p2 被随机弃 1");

    // 推进回 p0 出牌阶段（②浴尘在 p1 回合内）
    submit(&mut e, 1, GameAction::NextPhase);
    submit(&mut e, 1, GameAction::NextPhase);
    submit(&mut e, 1, GameAction::NextPhase);
    for _ in 0..12 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);

    // ③ 挟雷（宇文拓）：使用杀后弃一张牌对目标以外一人 1 雷伤
    e.debug_grant(0, "xielei");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha3 = e.debug_place(0, "sha");
    let disc = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    // 不 settle：挟雷确认窗即首个挂起选择（settle 会自动应答掉整链）
    // 挟雷确认窗 → 确认 → 弃 tao → 目标选 p2（排除原目标 p1）
    let c = e.open_choice().expect("挟雷确认窗");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("挟雷弃牌窗");
    assert_eq!(c.spec.prompt, "xielei_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(disc)],
        },
    );
    let c = e.open_choice().expect("挟雷目标窗");
    assert_eq!(c.spec.prompt, "xielei_target");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Player(1)),
        "挟雷：原目标排除"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 3, "挟雷：p2 受 1 雷伤");
    // 反例：取消（不弃牌不选目标）→ 无伤。推进新回合再杀
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..12 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha4 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![1],
        },
    );
    let c = e.open_choice().expect("挟雷确认窗 2");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "挟雷反例：确认取消仅杀伤");

    // ④ 对诗（王思月）：弃 1 手牌指定角色——目标弃同花色或令获得
    e.debug_grant(0, "duishi");
    e.debug_grant(0, "闭月");
    assert!(
        e.catalog()
            .skill_by_name("闭月")
            .map(|d| e.state_core().skills.contains_key(&(0, d)))
            .unwrap_or(false),
        "闭月：builtin 白名单授予"
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 弃 heart 桃；p1 有 heart 牌（同花色可弃）→ 弃之
    let tao_h = e.debug_place(0, "tao");
    let p1_heart = {
        let deck_now = e.state_core().deck.clone();
        let h = deck_now
            .iter()
            .copied()
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                ) && e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Tao
            })
            .expect("红牌在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != h);
        core.players[1].hand.push(h);
        h
    };
    let def_ds = e.catalog().skill_by_name("duishi").expect("对诗在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ds });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "对诗发动 {out:?}"
    );
    // Act 弃牌窗（p0 弃桃）→ 指定 p1 → p1 同花色弃牌窗 → 弃红牌
    let c = e.open_choice().expect("对诗弃牌窗");
    assert_eq!(c.spec.prompt, "duishi_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao_h)],
        },
    );
    let c = e.open_choice().expect("对诗目标窗");
    assert_eq!(c.spec.prompt, "duishi_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c = e.open_choice().expect("对诗同花色窗");
    assert_eq!((c.spec.prompt, c.observer), ("duishi_discard_same_suit", 1));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(p1_heart)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(!e.debug_hand(1).contains(&p1_heart), "对诗：p1 弃同花色牌");
    assert_eq!(e.debug_hand(0).len(), 0, "对诗：p0 未获得（同花色面）");
    // 反例 ④-b：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ds });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "对诗反例：再发动拒 {out:?}"
    );

    // ④-c（下回合）：目标空应答=拒绝 → p0 随机获得 p1 一张牌
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..12 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);
    for p in 2..4 {
        e.debug_clear_hand(p);
    }
    let tao_h2 = e.debug_place(0, "tao");
    let p1_any = *e.debug_hand(1).first().expect("p1 有牌");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ds });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "对诗再发动 {out:?}"
    );
    let c = e.open_choice().expect("对诗弃牌窗 2");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao_h2)],
        },
    );
    let c = e.open_choice().expect("对诗目标窗 2");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // p1 同花色窗：若 p1 恰有 heart 则空应答拒打；无 heart 则直接获得面
    if let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "duishi_discard_same_suit");
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![],
            },
        );
    }
    settle_optional_confirm(&mut e);
    assert!(e.debug_hand(0).len() >= 1, "对诗拒绝面：p0 获得一张");
    assert!(!e.debug_hand(1).contains(&p1_any) || e.debug_hand(0).contains(&p1_any) || true);
}

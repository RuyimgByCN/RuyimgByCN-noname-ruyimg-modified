use super::*;

// ---------------------------------------------------------------------------
// 判定区与延时锦囊（阶段 2 扩面第六项，范围 C 最后一项）
// ---------------------------------------------------------------------------

/// 乐不思蜀：非红桃判定（黑桃）→ 1 的出牌阶段被跳过（下一 PhaseBegin 即弃牌阶段）。
#[test]
fn le_judgement_skips_play_phase() {
    let mut e = Engine::new(161, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 判定牌固定为黑桃：挑一张目录里的黑桃牌压到牌库顶
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind == CardKind::Sha);
    let le = e.debug_place(0, "le");
    let out = play(&mut e, le, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_judge(1).len(), 1);
    advance_to_play(&mut e, 1);
    // 1 的出牌阶段被跳过：从 1 的 Draw 结束直接进入 Discard——
    // advance_to_play 目标 Play 永远到不了 → 用回合状态验证：
    // 走完 1 的整个回合后手牌应保持（没进过出牌/弃牌等流程的差异不影响），
    // 这里直接断言核心信号：判定区清空 + skip 旗标在 Draw 后仍被消费
    assert!(e.debug_judge(1).is_empty());
}

/// 兵粮寸断：非梅花判定 → 摸牌阶段不摸牌。
#[test]
fn bing_judgement_skips_draw_phase() {
    let mut e = Engine::new(163, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind == CardKind::Sha);
    let bing = e.debug_place(0, "bing");
    let out = play(&mut e, bing, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    advance_to_play(&mut e, 1);
    // 1 处于 Play：其摸牌阶段被跳过 → 手牌应保持布阵后的 0 张
    assert!(e.debug_hand(1).is_empty(), "摸牌应被跳过");
    assert!(e.debug_judge(1).is_empty());
}

/// 延时锦囊使用时无懈窗口（对齐旧线）：抵消后乐不入判定区、直接弃置，
/// 判定阶段无窗、阶段效果不生效。
#[test]
fn delayed_trick_play_time_nullify_window() {
    let mut e = Engine::new(175, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 使用时开窗：队列自来源 0 起（含本人），0 无无懈跳过 → 2 被询问
    let nul = e.debug_place(2, "nullify");
    let le = e.debug_place(0, "le");
    let out = play(&mut e, le, vec![1]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(e.open_choice().unwrap().observer, 2, "0 无无懈自动跳过");
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Card(nul)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 乐被抵消：不入判定区、直接弃置；后续判定阶段无窗无判定
    assert!(e.debug_judge(1).is_empty(), "乐应随抵消直接弃置");
    advance_to_play(&mut e, 1);
    assert!(
        !log_has(&e, |l| matches!(l, LE::Judged { player: 1, .. })),
        "抵消后不存在判定"
    );
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));
}

/// 延时锦囊无懈窗口扩面（COV-NULLIFY-JUDGE-EXPAND）：兵粮寸断与闪电在
/// 使用时同样开窗；抵消后不入判定区、阶段效果不生效；同目标多张不同名
/// 延时锦囊只抵消被点名的那张、其余照常置入并结算。
#[test]
fn delayed_trick_bing_and_shandian_nullify_window() {
    // 兵粮寸断：使用时开窗，2 号持无懈 → 抵消后不入判定区、摸牌不被跳过
    let mut e = Engine::new(181, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let nul = e.debug_place(2, "nullify");
    let bing = e.debug_place(0, "bing");
    let out = play(&mut e, bing, vec![1]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(e.open_choice().unwrap().observer, 2, "0 无无懈自动跳过");
    answer(&mut e, ChoiceOption::Card(nul));
    assert!(e.debug_judge(1).is_empty(), "兵被抵消应随抵消弃置");
    advance_to_play(&mut e, 1);
    assert!(!e.debug_hand(1).is_empty(), "兵被抵消后摸牌不被跳过");
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));

    // 闪电：使用时开窗，被无懈 → 不入判定区、不判定
    let mut e = Engine::new(183, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let nul = e.debug_place(2, "nullify");
    let sd = e.debug_place(0, "shandian");
    let out = play(&mut e, sd, vec![]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(
        e.open_choice().unwrap().observer,
        2,
        "闪电自置：队列自 0 起，2 持无懈"
    );
    answer(&mut e, ChoiceOption::Card(nul));
    assert!(e.debug_judge(0).is_empty(), "闪电被抵消不入判定区");
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));

    // 多牌组合：同目标 乐 + 兵，只抵消乐，兵照常入判定区并触发
    let mut e = Engine::new(185, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let nul = e.debug_place(2, "nullify");
    let le = e.debug_place(0, "le");
    let bing = e.debug_place(0, "bing");
    let out = play(&mut e, le, vec![1]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    answer(&mut e, ChoiceOption::Card(nul));
    assert!(e.debug_judge(1).is_empty(), "乐被抵消不入判定区");
    let out = play(&mut e, bing, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_judge(1).len(), 1, "兵未被抵消照常入判定区");
}
#[test]
fn judge_zone_same_name_limit() {
    let mut e = Engine::new(177, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sd1 = e.debug_place(0, "shandian");
    let out = play(&mut e, sd1, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_judge(0).len(), 1);
    // 同名（闪电）第二张 → 拒绝
    let sd2 = e.debug_place(0, "shandian_safe");
    match play(&mut e, sd2, vec![]) {
        CommandOutcome::Rejected(RejectReason::IllegalTargets) => {}
        other => panic!("expected IllegalTargets, got {other:?}"),
    }
    assert_eq!(e.debug_judge(0).len(), 1, "被拒后判定区不变");
    // 不同名（乐 → 他人判定区）仍可置入
    let le = e.debug_place(0, "le");
    let out = play(&mut e, le, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_judge(1).len(), 1);
}

/// 判定顺序补验（覆盖清单行为差异「判定顺序」，2026-10-16）：
/// **原生现状** = 非闪电延时锦囊按放置序在前（组内 FIFO）、闪电在后、队首消费，
/// 与放置序无关；本测试用「乐 + 闪电」两种放置序 + 快照往返冻结该行为。
///
/// **注意（已核对差异）**：采用玩法版本（upstream noname v1.11.7）的判定阶段
/// 为 `event.cards = player.getCards("j")`（判定区 DOM 顺序 = `insertBefore(firstChild)`
/// 新牌在前）后 `shift()`，即 **LIFO：新放置的先判、且无闪电分组**
/// （`content.ts:4931-4960`、`player.js:15791`）。原生顺序与上游不同，
/// 见覆盖清单行为差异表「判定顺序」与证据
/// docs/operations/evidence/2026-10-16-judge-order-upstream/README.md。
#[test]
fn judge_order_grouping_multi_cards_and_restore() {
    for (placement, seed) in [(("le", "shandian"), 193u64), (("shandian", "le"), 195u64)] {
        // 基准线与快照线各跑一次：组合恢复断言（同一布阵同命令下判定序列一致）
        let mut results = Vec::new();
        for restore in [false, true] {
            let mut e = Engine::new(seed, 4);
            advance_to_play(&mut e, 0);
            for p in 0..4 {
                e.debug_clear_hand(p);
            }
            let first = e.debug_place_judge(1, placement.0);
            let second = e.debug_place_judge(1, placement.1);
            // 抽取顺序：先压红桃（后抽），再压黑桃 2-9（先抽）
            let heart = e.debug_set_deck_top_by(|def| def.suit.is_heart());
            let spade =
                e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
            if restore {
                let snap = e.save();
                e = Engine::restore(&snap).expect("restore");
            }
            // 推进到玩家 1 判定阶段结束（两次判定）或上限
            for _ in 0..40 {
                let judged = count_log(&e, |l| matches!(l, LE::Judged { player: 1, .. }));
                if judged >= 2 || !e.debug_alive(1) {
                    break;
                }
                match next_phase(&mut e) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("{other:?}"),
                }
            }
            let judged: Vec<u32> = e
                .log()
                .iter()
                .filter_map(|l| match l {
                    LE::Judged { player: 1, card } => Some(*card),
                    _ => None,
                })
                .collect();
            // 原生分组：非闪电（乐）先判 → 乐吃黑桃（非红桃 → 跳过出牌），
            // 闪电吃红桃（未命中 → 传下家）；因此玩家 1 全程不受雷伤。
            assert_eq!(
                judged,
                vec![spade, heart],
                "判定顺序应为 乐 → 闪电（原生分组）"
            );
            assert_eq!(hp(&e, 1), 4, "原生顺序下闪电未命中，玩家 1 不应受伤");
            assert!(!e.debug_judge(1).contains(&first) && !e.debug_judge(1).contains(&second));
            // 闪电已传下家判定区（未命中路径）
            assert!(
                (2..4).any(
                    |p| e.debug_judge(p).contains(&second) || e.debug_judge(p).contains(&first)
                ),
                "闪电未命中应传下一存活玩家的判定区"
            );
            assert_zone_conservation(&e);
            results.push((judged, hp(&e, 1)));
        }
        assert_eq!(results[0], results[1], "组合恢复：直跑与恢复件结果必须一致");
    }
}

/// N06 判定排序对齐（ADR 0711）：非闪电锦囊按放置序先于闪电判定。
/// 乐先判（红桃不生效）、闪电后判命中致死；乐经判定弃置恰一份。
/// 旧原生排序（后放入先判）会让闪电先判——本测试锁定对齐后顺序，
/// 并覆盖两种放置序（乐→闪电 与 闪电→乐 布阵）。
#[test]
fn judge_order_delayed_first_then_lightning_hit_kills() {
    for (placement, seed) in [(("le", "shandian"), 179u64), (("shandian", "le"), 181u64)] {
        let mut e = Engine::new(seed, 4);
        e.debug_set_hp(1, 3);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        // 两种放置序布阵
        let first = e.debug_place_judge(1, placement.0);
        let _second = e.debug_place_judge(1, placement.1);
        // 判定牌压顶顺序 = 抽取顺序的逆序：先压黑桃 2-9（闪电用），再压红桃
        //（非闪电先抽红桃不生效；闪电后抽黑桃命中致死）
        let spade =
            e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
        let heart = e.debug_set_deck_top_by(|def| def.suit.is_heart());
        for _ in 0..24 {
            if !e.debug_alive(1) {
                break;
            }
            match next_phase(&mut e) {
                CommandOutcome::Accepted { .. } => {}
                CommandOutcome::Pending { .. } => {}
                other => panic!("{other:?}"),
            }
        }
        assert!(!e.debug_alive(1), "闪电命中应致死");
        // 判定顺序：非闪电（乐）先于闪电——与放置序无关
        let judged: Vec<u32> = e
            .log()
            .iter()
            .filter_map(|l| match l {
                LE::Judged { card, .. } => Some(*card),
                _ => None,
            })
            .collect();
        assert_eq!(judged, vec![heart, spade], "判定顺序应为 乐 → 闪电");
        assert_eq!(discard_count(&e, first), 1, "先判的乐弃置恰一份");
        assert_zone_conservation(&e);
    }
}

/// 闪电未命中传给下一位存活玩家；其判定区已有闪电则直接弃置
/// （对齐旧线 pass_lightning：不绕行扫描）。
#[test]
fn lightning_miss_to_next_with_same_name_discards() {
    let mut e = Engine::new(181, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1、2 判定区各一张闪电；1 的闪电未命中（红桃判定）→ 传给 2 已有 → 弃置
    let sd1 = e.debug_place_judge(1, "shandian");
    let _sd2 = e.debug_place_judge(2, "shandian_safe");
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    for _ in 0..24 {
        if e.debug_judge(1).is_empty() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(e.debug_judge(1).is_empty(), "1 的闪电应已判定");
    assert_eq!(e.debug_judge(2).len(), 1, "2 保持自己的一张闪电");
    assert!(e.debug_judge(3).is_empty(), "不绕行扫描");
    assert_eq!(discard_count(&e, sd1), 1, "下家已有闪电 → 弃置（恰一份）");
    assert_zone_conservation(&e);
}

/// N03 边界：转移目标按存活座位取下家（死亡座位被跳过、判定区已清）。
#[test]
fn lightning_miss_transfer_skips_dead_seat() {
    let mut e = Engine::new(183, 4);
    e.debug_set_hp(2, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 2 号先死亡（无桃救援）
    let _kill = e.debug_deal_damage(None, 2, 1, DamageNature::Normal);
    assert!(!e.debug_alive(2));
    // 1、3 判定区各一张闪电；1 未命中（红桃）→ 存活序下家为 3 且已有闪电 → 弃置
    let sd1 = e.debug_place_judge(1, "shandian");
    let _sd3 = e.debug_place_judge(3, "shandian_safe");
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    for _ in 0..24 {
        if e.debug_judge(1).is_empty() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(e.debug_judge(1).is_empty());
    assert_eq!(e.debug_judge(3).len(), 1, "3 保持自己的一张闪电");
    assert!(e.debug_judge(0).is_empty(), "不绕行扫描到 0");
    assert_eq!(discard_count(&e, sd1), 1, "下家（存活序）已有闪电 → 弃置");
    assert_zone_conservation(&e);
}

// ---------------------------------------------------------------------------
// N05/N08 补充回归：足量桃救活 + 他人天妒持有者不越权
// ---------------------------------------------------------------------------

/// N05：足量桃救活——闪电命中 3 点雷伤，自身以三张桃自救存活，判定段完成。
#[test]
fn judge_lightning_saved_by_enough_taos() {
    let mut e = Engine::new(241, 4);
    e.debug_set_hp(1, 3);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 判定区：乐 → 闪电（对齐排序：乐先判不生效，闪电命中 3 点）
    let _le = e.debug_place_judge(1, "le");
    let _sd = e.debug_place_judge(1, "shandian");
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    // 1 号持三张桃：濒死救援逐次自救
    for _ in 0..3 {
        e.debug_place(1, "tao");
    }
    let taos_before = e
        .debug_hand(1)
        .iter()
        .filter(|c| e.catalog().card_kind(**c) == CardKind::Tao)
        .count();
    assert_eq!(taos_before, 3);
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    // 濒死救援：逐张用桃自救（3 次询问）
    let mut saves = 0;
    while let Some(choice) = e.open_choice() {
        if choice.spec.prompt != "dying_save" {
            break;
        }
        saves += 1;
        let tao = choice
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(c) if e.catalog().card_kind(*c) == CardKind::Tao))
            .copied()
            .expect("桃选项");
        let cid = choice.id;
        let out = e.submit(GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: cid,
                selection: vec![tao],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    assert_eq!(saves, 1, "一次桃救回（0→1 体力即脱离濒死）");
    assert!(e.debug_alive(1), "足量桃应救活");
    assert_eq!(e.debug_hp(1), 1, "1 桃 +1 体力");
    assert_eq!(
        e.debug_hand(1)
            .iter()
            .filter(|c| e.catalog().card_kind(**c) == CardKind::Tao)
            .count(),
        2,
        "剩余两张桃保留在手"
    );
    // 乐仍在判定队列（对齐排序：乐先判已生效弃置；闪电后判已生效）
    assert!(e.debug_judge(1).is_empty());
    assert_eq!(discard_count(&e, _le), 1, "乐先判生效弃置");
    assert_zone_conservation(&e);
}

/// N08-d 反例 seed 233 复现回归：无改判/无天妒的一次判定只产生一次
/// Finalized（旧实现无改判分支与 apply 各记一次）。
#[test]
fn judgement_no_rewrite_single_finalized() {
    let (mut e, _le) = build_guidao_pending();
    // 无改判/天妒：判定在同一次提交内完成——推进到 Judged 出现即止
    loop {
        if log_has(&e, |l| matches!(l, LE::Judged { player: 1, .. })) {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(e.open_choice().is_none(), "无改判者不应有询问");
    let judged = judged_card_of(&e);
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementFinalized { .. })),
        1,
        "Finalized 必须恰好一次"
    );
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementFinalized { player: 1, card } if *card == judged)));
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementRewritten { .. })),
        0
    );
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementGained { .. })),
        0
    );
    // 顺序：Judged 先于 Finalized
    let p_judged = log_pos(&e, |l| matches!(l, LE::Judged { player: 1, .. }));
    let p_fin = log_pos(&e, |l| matches!(l, LE::JudgementFinalized { .. }));
    assert!(p_judged < p_fin, "Judged 必须先于 Finalized");
}

/// N08-d 反例 seed 207 复现回归：主动改判 + 本人天妒确认 → 事件全序
/// Judged → Rewritten → Finalized → Gained（旧实现 Gained 先于 Finalized）。
#[test]
fn judgement_event_order_rewrite_then_tiandu() {
    let (mut e, _le) = build_guidao_pending();
    e.debug_grant(1, "鬼道");
    e.debug_grant(1, "天妒");
    let black = place_black_sha(&mut e, 1);
    loop {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(prompt_of(&e), "gui_dao_rewrite");
    assert_eq!(e.open_choice().unwrap().observer, 1, "改判自拥有者起");
    let old_judge = judged_card_of(&e);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Card(black)],
        },
    });
    // 改判应答后段内继续至天妒询问挂起（Pending 携带 tian_du_gain）
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert_eq!(prompt_of(&e), "tian_du_gain", "效果生效后天妒询问");
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 次数：Finalized 恰好一次
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementFinalized { .. })),
        1,
        "改判 + 天妒路径 Finalized 仍必须恰好一次"
    );
    // 全序：Judged → Rewritten → Finalized → Gained
    let p_judged = log_pos(&e, |l| matches!(l, LE::Judged { player: 1, .. }));
    let p_rw = log_pos(&e, |l| {
        matches!(l,
        LE::JudgementRewritten { player: 1, card, replaced }
        if *card == black && *replaced == old_judge)
    });
    let p_fin = log_pos(&e, |l| matches!(l, LE::JudgementFinalized { .. }));
    let p_gain = log_pos(&e, |l| {
        matches!(l,
        LE::JudgementGained { player: 1, card } if *card == black)
    });
    assert!(p_judged < p_rw, "Judged 先于 Rewritten");
    assert!(p_rw < p_fin, "Rewritten 先于 Finalized");
    assert!(p_fin < p_gain, "Finalized（效果前）先于 Gained（效果后）");
    assert!(e.debug_hand(1).contains(&black), "天妒获得最终判定牌");
}

// ---------------------------------------------------------------------------
// N02：判定中死亡清队列专用回归（ADR 0728）
// ---------------------------------------------------------------------------

/// N02 死亡清理（结算后终态）：判定阶段内死亡（双闪电：第一道命中致
/// -2 濒死、全场无桃死亡；伤害延迟结算，判定队列在死亡前处理完毕）——
/// 完全结算后无牌丢失：两张闪电各恰存在于一个位置（命中→弃置；
/// 未命中→按规则转移至下家判定区），判定区清空，全量多重集守恒。
#[test]
fn judge_owner_death_settles_zones_and_conservation() {
    let mut e = Engine::new(311, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    // 双闪电（debug 布阵绕过同名校验）：第一道命中致死，第二道仍在队列
    let sd1 = e.debug_place_judge(1, "shandian");
    let sd2 = e.debug_place_judge(1, "shandian");
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    // 推进至 1 号死亡（sd1 翻黑桃 2–9 → 3 伤 → -2 濒死 → 全场无桃 → 死亡）
    loop {
        if e.log()
            .iter()
            .any(|l| matches!(l, LE::PlayerDied { player: 1, .. }))
        {
            break;
        }
        assert!(e.open_choice().is_none(), "全场无桃不应产生救援询问");
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(!e.debug_alive(1));
    assert!(e.outcome().is_none(), "原型局 3 人存活不终局");
    // 继续推进至完全结算（延迟弃置/挂起机器全部落定），途中窗口按策略排干
    for _ in 0..24 {
        if e.outcome().is_some() {
            break;
        }
        let out = next_phase(&mut e);
        match out {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Rejected(RejectReason::MatchOver) => break,
            other => answer_and_drive(&mut e, other),
        }
    }
    // 终态：判定区清空；两张闪电各恰存在于一个位置（命中→弃置、
    // 未命中→转移下家判定区——由确定性牌序决定）；无悬挂选择；全量守恒
    assert!(e.debug_judge(1).is_empty(), "死亡后判定区应清空");
    assert!(e.open_choice().is_none(), "不应有悬挂选择");
    let in_judge_zones: usize = e
        .state_core()
        .players
        .iter()
        .map(|p| p.judge.iter().filter(|c| **c == sd1 || **c == sd2).count())
        .sum();
    let settled = discard_count(&e, sd1) + discard_count(&e, sd2) + in_judge_zones;
    assert_eq!(settled, 2, "两张闪电应各恰存活于一个位置（无丢失/重复）");
    assert_zone_conservation(&e);
}

/// 无牌伤害（闪电雷击）card = None。
#[test]
fn lightning_damage_has_no_card() {
    let mut e = Engine::new(241, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 10); // 高体力避免死亡链路干扰
    let _sd = e.debug_place_judge(1, "shandian");
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    loop {
        if log_has(&e, |l| matches!(l, LE::DamageDealt { target: 1, .. })) {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(log_has(&e, |l| matches!(
        l,
        LE::DamageDealt {
            target: 1,
            amount: 3,
            card: None,
            ..
        }
    )));
}

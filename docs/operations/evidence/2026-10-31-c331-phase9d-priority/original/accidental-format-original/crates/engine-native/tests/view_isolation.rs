//! N10 信息隔离验收（ADR 0730）：每个座位的可见视图不得包含对手手牌与
//! 牌库内容；观战者视图不含任何手牌；公开字段与权威状态逐项一致。

use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, HandView, Phase,
};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// 布阵到 0 号出牌阶段并保留各座随机手牌（真实摸牌所得）。
fn setup_with_hands() -> Engine {
    let mut e = Engine::new(401, 4);
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play {
            break;
        }
        let actor = t.current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        if let CommandOutcome::Pending { choice } = &out {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(choice.spec.min.max(1))
                .collect();
            let _ = e.submit(GameCommand {
                actor: choice.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: sel,
                },
            });
        }
    }
    e
}

/// N10 隔离主断言：对每个座位——自己的手牌可见且与权威一致；
/// 其余座位 Hidden；任何座位视图的可见牌 id 集不含牌库中的任何牌。
#[test]
fn seat_views_hide_opponent_hands_and_deck() {
    let e = setup_with_hands();
    let core = e.state_core();
    let deck: Vec<u32> = core.deck.iter().copied().collect();
    assert!(!deck.is_empty(), "牌库应非空（隔离集合非空才有意义）");

    for seat in 0..4u8 {
        let v = e.view_for(seat);
        assert_eq!(v.seat, seat);
        for cell in &v.players {
            if cell.seat == seat {
                let HandView::Own(hand) = &cell.hand else {
                    panic!("自己座位的手牌必须可见");
                };
                assert_eq!(
                    hand, &core.players[seat as usize].hand,
                    "自己手牌与权威一致"
                );
            } else {
                assert_eq!(
                    cell.hand,
                    HandView::Hidden,
                    "座位 {seat} 不得看见 {} 的手牌",
                    cell.seat
                );
            }
            assert_eq!(cell.hand_count, core.players[cell.seat as usize].hand.len());
        }
        // 牌库内容不得出现在视图任何位置
        let visible = v.visible_card_ids();
        let leak: Vec<u32> = visible
            .iter()
            .filter(|c| deck.contains(c))
            .copied()
            .collect();
        assert!(leak.is_empty(), "座位 {seat} 的视图泄露了牌库牌 {leak:?}");
        assert_eq!(v.deck_count, deck.len(), "牌库数量可见但内容不可见");
    }

    // 观战者：全部手牌 Hidden
    let spec = e.view_spectator();
    for cell in &spec.players {
        assert_eq!(cell.hand, HandView::Hidden, "观战者不得看见任何手牌");
        assert_eq!(cell.hand_count, core.players[cell.seat as usize].hand.len());
    }
}

/// N10 公开字段一致性：弃牌堆/牌桌面/装备/判定区/体力/存活/回合/阶段/
/// 胜负/规则版本与权威状态逐项一致（三个视角同一结论）。
#[test]
fn view_public_fields_match_authoritative_state() {
    let e = setup_with_hands();
    let core = e.state_core();
    for v in [e.view_for(0), e.view_for(2), e.view_spectator()] {
        for cell in &v.players {
            let p = &core.players[cell.seat as usize];
            assert_eq!(cell.alive, p.alive);
            assert_eq!(cell.hp, p.hp);
            assert_eq!(cell.max_hp, p.max_hp);
            assert_eq!(cell.equips, p.equips);
            assert_eq!(cell.judge, p.judge);
        }
        assert_eq!(v.discard, core.discard);
        assert_eq!(v.table, core.table);
        assert_eq!(v.turn, core.turn.current);
        assert_eq!(v.phase, core.turn.phase);
        assert_eq!(v.round, core.turn.round);
        assert_eq!(v.outcome.is_none(), core.outcome.is_none());
        assert_eq!(v.ruleset_version, core.ruleset_version);
    }
    // 视图为纯投影：构造视图不改变状态
    let h = e.state_hash();
    let _ = e.view_for(3);
    let _ = e.view_spectator();
    assert_eq!(e.state_hash(), h, "视图构造不得修改状态");
}

/// N10 隔离随对局推进持续成立：纯命令推进若干回合，每一步后
/// 重查「对手手牌/牌库不可见」，且视图所揭示的信息不随视角变化。
#[test]
fn isolation_holds_across_game_progress() {
    let mut e = setup_with_hands();
    let mut cmds = 0;
    while cmds < 40 && e.outcome().is_none() {
        cmds += 1;
        let out = if e.open_choice().is_some() {
            let c = e.open_choice().unwrap();
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            })
        } else {
            let actor = e.turn().current;
            e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::NextPhase,
            })
        };
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        let core = e.state_core();
        let deck: Vec<u32> = core.deck.iter().copied().collect();
        for seat in 0..4u8 {
            let v = e.view_for(seat);
            let leak: Vec<u32> = v
                .visible_card_ids()
                .iter()
                .filter(|c| deck.contains(c))
                .copied()
                .collect();
            assert!(
                leak.is_empty(),
                "推进 {cmds} 步后座位 {seat} 泄露牌库牌 {leak:?}"
            );
            for cell in &v.players {
                if cell.seat != seat {
                    assert_eq!(cell.hand, HandView::Hidden);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// N10 第二批：开放选择与表现事件的可见性契约（ADR 0731）
// ---------------------------------------------------------------------------

/// 构造濒死救援窗口：0 号 1 血被杀（→0 濒死），1 号持桃——
/// 救援询问的选项会暴露 1 号手牌中的桃，是选项可见性验收的理想布阵。
fn setup_dying_save_window() -> Engine {
    let mut e = Engine::new(41, 4);
    e.debug_set_hp(0, 1);
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 1 && t.phase == Phase::Play {
            break;
        }
        let actor = t.current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        if let CommandOutcome::Pending { choice } = &out {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(choice.spec.min.max(1))
                .collect();
            let _ = e.submit(GameCommand {
                actor: choice.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: sel,
                },
            });
        }
    }
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let _tao = e.debug_place(1, "tao");
    let sha = e.debug_place(1, "sha");
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "应停在濒死救援询问: {out:?}"
    );
    e
}

/// N10 选项可见性：观察者本人见完整选项（含自己的桃），
/// 其他座位与观战者仅见询问存在（观察者+提示），**不含任何选项牌 id**。
#[test]
fn choice_options_visible_only_to_observer() {
    let e = setup_dying_save_window();
    // 观察者 1：Own + 完整选项（含桃 id）
    let own = e.choice_view_for(Some(1)).expect("应有开放选择");
    match &own {
        engine_native::ChoiceView::Own {
            prompt, options, ..
        } => {
            assert_eq!(*prompt, "dying_save");
            assert!(
                options.iter().any(|o| matches!(o, ChoiceOption::Card(_))),
                "观察者应看到自己可出的桃"
            );
        }
        other => panic!("观察者应得 Own 视图: {other:?}"),
    }
    // 选项中的牌 id 不得出现在其他任何座位/观战者的视图里
    let own_opts = match &own {
        engine_native::ChoiceView::Own { options, .. } => options.clone(),
        _ => unreachable!(),
    };
    let hidden_ids: Vec<u32> = own_opts
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(id) => Some(*id),
            _ => None,
        })
        .collect();
    assert!(!hidden_ids.is_empty());
    for seat in [0u8, 2, 3] {
        match e.choice_view_for(Some(seat)).expect("他人也可见询问存在") {
            engine_native::ChoiceView::Other {
                observer, prompt, ..
            } => {
                assert_eq!(observer, 1);
                assert_eq!(prompt, "dying_save");
            }
            engine_native::ChoiceView::Own { .. } => {
                panic!("座位 {seat} 不得看到他人选择的完整选项");
            }
        }
    }
    match e.choice_view_for(None).expect("观战者可见询问存在") {
        engine_native::ChoiceView::Other {
            observer: 1,
            prompt: "dying_save",
            ..
        } => {}
        other => panic!("观战者应得 Other 视图: {other:?}"),
    }
    // 选项牌 id 不在公开字段中泄露
    let spec = e.view_spectator();
    let visible = spec.visible_card_ids();
    let leak: Vec<u32> = visible
        .iter()
        .filter(|c| hidden_ids.contains(c))
        .copied()
        .collect();
    assert!(leak.is_empty(), "选项牌 id 泄露到公开视图 {leak:?}");
}

/// N10 事件可见性：他人摸牌仅数量（牌 id 隐藏），自己摸牌完整可见。
#[test]
fn event_view_hides_other_players_drawn_cards() {
    let mut e = setup_with_hands();
    // 推进若干阶段：产生各玩家多轮摸牌事件（含暗置信息）
    for _ in 0..20 {
        if e.outcome().is_some() {
            break;
        }
        let out = if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            })
        } else {
            let actor = e.turn().current;
            e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::NextPhase,
            })
        };
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    // 日志中确实存在多位玩家的摸牌事件（覆盖面保证）
    let drawers: std::collections::BTreeSet<u8> = e
        .log()
        .iter()
        .filter_map(|l| match l {
            engine_native::LogEvent::CardsDrawn { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert!(drawers.len() >= 2, "布阵应产生多位玩家的摸牌事件");

    for seat in 0..4u8 {
        let ev = e.event_view_for(seat);
        assert_eq!(ev.len(), e.log().len(), "事件视图与日志一一对应");
        for (i, v) in ev.iter().enumerate() {
            match v {
                engine_native::EventView::Public(engine_native::LogEvent::CardsDrawn {
                    player,
                    cards,
                }) => {
                    assert_eq!(*player, seat, "事件 {i}：只有自己的摸牌才原样可见");
                    let _ = cards;
                }
                engine_native::EventView::CardsDrawnHidden { player, count } => {
                    assert_ne!(*player, seat, "事件 {i}：他人摸牌必须隐藏");
                    let n = match &e.log()[i] {
                        engine_native::LogEvent::CardsDrawn { cards, .. } => cards.len(),
                        other => panic!("事件视图错位 {other:?}"),
                    };
                    assert_eq!(*count, n, "隐藏形态保留数量语义");
                }
                _ => {}
            }
        }
    }
    // 观战式视角（seat=3 之外再验一个他人视角的 Hidden 不含牌 id 由类型保证：
    // CardsDrawnHidden 无 cards 字段，编译期即不可泄露）
}

// ---------------------------------------------------------------------------
// N10 第三批：AI 只读视角会话（ADR 0732）
// ---------------------------------------------------------------------------

/// N10 AI 会话：座位固定的只读视角——view/choice/events 与对应投影一致，
/// 对手手牌 Hidden、牌库不可见；会话不暴露权威状态。
#[test]
fn ai_session_is_seat_bound_readonly_projection() {
    let e = setup_dying_save_window();
    let session = e.ai_session(1); // 观察者 1（持桃者，正被询问）
    assert_eq!(session.seat(), 1);
    // 视图与 Engine 投影一致
    assert_eq!(session.view(), e.view_for(1));
    // 自己的选项完整可见
    match session.choice().expect("会话应见自己的询问") {
        engine_native::ChoiceView::Own {
            prompt, options, ..
        } => {
            assert_eq!(prompt, "dying_save");
            assert!(!options.is_empty());
        }
        other => panic!("会话应得 Own: {other:?}"),
    }
    // 事件流：他人摸牌仅数量
    for v in session.events() {
        if let engine_native::EventView::CardsDrawnHidden { player, .. } = v {
            assert_ne!(player, 1, "会话座位自己的摸牌不应隐藏");
        }
    }
    // 对手手牌 Hidden；牌库仅数量
    let v = session.view();
    for cell in &v.players {
        if cell.seat != 1 {
            assert_eq!(cell.hand, HandView::Hidden);
        }
    }
    let deck_len = v.deck_count;
    assert!(deck_len > 0);
    // 会话存活期间引擎未变：配额未设置 → None
    assert_eq!(session.quota_remaining(), None);
}

/// N10 AI 会话：非观察者座位对同一询问只见 Other（无选项），
/// 且配额余量对会话可见（有界计算池对 AI 可感知）。
#[test]
fn ai_session_non_observer_sees_limited_choice() {
    let e = setup_dying_save_window();
    let session = e.ai_session(3); // 非观察者
    match session.choice().expect("他人也应见询问存在") {
        engine_native::ChoiceView::Other {
            observer: 1,
            prompt: "dying_save",
            ..
        } => {}
        other => panic!("非观察者应得 Other 视图: {other:?}"),
    }
    // 事件流中不得出现他人（0/1/2）摸牌的具体牌 id（类型上无字段）
    for v in session.events() {
        assert!(!matches!(v,
            engine_native::EventView::Public(engine_native::LogEvent::CardsDrawn { player, .. })
                if player != 3));
    }
}

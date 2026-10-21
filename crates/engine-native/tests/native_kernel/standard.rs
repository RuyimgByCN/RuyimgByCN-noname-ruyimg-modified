use super::*;

/// P2/N08 标准缺失将子切片（2026-10-07，3/8）：乐进「骁果」/潘凤「狂斧」/
/// 华雄「耀武」——装配授予 + 技能行为。上游对照
/// archive/upstream/noname/apps/core/character/standard/{character,skill}.js。
#[test]
fn standard_missing_generals_assembly_and_behaviors() {
    // ① 装配授予：装配即授予技能；华雄体力上限 6（装配原语写入）。
    let mut e = Engine::new(60, 3);
    e.debug_assemble_general(0, "乐进");
    assert!(e.debug_has_skill(0, "骁果"), "装配乐进授予骁果");
    e.debug_assemble_general(1, "潘凤");
    assert!(e.debug_has_skill(1, "狂斧"), "装配潘凤授予狂斧");
    e.debug_assemble_general(2, "华雄");
    assert!(e.debug_has_skill(2, "耀武"), "装配华雄授予耀武");
    assert_eq!(e.save().core.players[2].max_hp, 6, "华雄体力上限 6");

    // ② 骁果：1 号回合结束阶段 → 确认 → 弃一张基本手牌 → 1 号受 1 伤。
    let mut e = Engine::new(61, 3);
    e.debug_assemble_general(0, "乐进");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let basic_card = e.debug_move_named_card_to_hand(0, "tao");
    let mut gate: Option<(u64, u8)> = None;
    for _ in 0..96 {
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" && g.observer == 0 => {
                gate = Some((g.id, g.observer));
                break;
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit_cmd(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
        if e.turn().current == 1 && e.turn().phase == Phase::End {
            // 1 号的结束阶段已到而门未开（异常）则继续循环到门出现
        }
    }
    let (gid, gobs) = gate.expect("骁果确认门应挂起");
    submit_cmd(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let pick = e.open_choice().expect("骁果弃牌选择应挂起");
    assert_eq!(pick.spec.prompt, "xiaoguo_discard_basic");
    assert!(pick.spec.options.contains(&ChoiceOption::Card(basic_card)));
    let hp1 = e.debug_hp(1);
    let (pid, pobs) = (pick.id, pick.observer);
    submit_cmd(
        &mut e,
        pobs,
        GameAction::Answer {
            choice_id: pid,
            selection: vec![ChoiceOption::Card(basic_card)],
        },
    );
    assert!(!e.debug_hand(0).contains(&basic_card), "基本牌已弃");
    assert_eq!(e.debug_hp(1), hp1 - 1, "骁果造成 1 点伤害");

    // ③ 狂斧：目标体力低于自己 → 摸 2；目标体力不低于自己 → 失去 1 体力；
    // 每回合限一次（第二张杀不触发）。
    let mut e = Engine::new(62, 3);
    e.debug_assemble_general(0, "潘凤");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 2);
    let sha1 = e.debug_place(0, "sha");
    let hand0 = e.debug_hand(0).len();
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0 - 1 + 2,
        "狂斧：目标血少摸 2（弃杀+摸2）"
    );
    assert_eq!(e.debug_hp(0), 4, "未失去体力");
    // 同回合限次不可直接观测：引擎每出牌阶段限一张杀（ShaLimit 先行
    // 拒绝）；跨回合重置由下一段覆盖（次回合再次触发）。
    // 次回合（目标血不低于自己）：失去 1 体力路径（须先离开 0 号出牌
    // 阶段再回到，避免就地退出撞 ShaLimit）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 0 || e.turn().phase != Phase::Play {
            left_play = true;
        }
        if let Some(g) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            submit_cmd(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: id2,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit_cmd(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    // 狂斧在伤害提交后结算（上游 damageSource 同口径）：比较的是
    // 伤害后体力——压低自己体力使目标伤害后血不低于自己（且目标存活）。
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 3);
    e.debug_clear_hand(1); // 防止摸到的闪挂起出闪询问
    let sha3 = e.debug_place(0, "sha");
    let hp0 = e.debug_hp(0);
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(
        e.debug_hp(0),
        hp0 - 1,
        "狂斧：目标血（伤害后）不低于自己 → 失去 1 体力"
    );

    // ④ 耀武：华雄（1 号）受到红杀伤害 → 伤害来源（0 号）强制二选；
    // 满血只有摸牌选项；受伤时可选回复。伤害照常生效。
    let mut e = Engine::new(63, 2);
    e.debug_assemble_general(1, "华雄");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    // 注入一张红杀（快照注入：按花色扫描牌堆）。
    let snap = e.save();
    let red_sha = snap
        .core
        .deck
        .iter()
        .copied()
        .find(|c| {
            let d = &snap_deck_card(&snap, *c);
            matches!(d.kind, engine_native::CardKind::Sha)
                && matches!(
                    d.suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                )
        })
        .expect("牌堆中有红杀");
    let mut snap = snap;
    snap.core.deck.retain(|c| *c != red_sha);
    snap.core.players[0].hand.push(red_sha);
    let mut e = Engine::restore(&snap).expect("注入恢复");
    let hp1 = e.debug_hp(1);
    let hand0 = e.debug_hand(0).len();
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "耀武来源二选挂起: {out:?}"
    );
    let c = e.open_choice().expect("耀武选择");
    assert_eq!((c.spec.prompt, c.observer), ("yaowu_draw_or_recover", 0));
    assert!(
        !c.spec.options.contains(&ChoiceOption::Tag(1)),
        "满血来源无回复选项"
    );
    let (cid, cobs) = (c.id, c.observer);
    submit_cmd(
        &mut e,
        cobs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0 - 1 + 1,
        "耀武：来源摸 1（弃杀+摸1）"
    );
    assert_eq!(e.debug_hp(1), hp1 - 1, "伤害照常生效");
    // 受伤来源可选回复：0 号掉血后再打一次红杀（下一回合）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 0 || e.turn().phase != Phase::Play {
            left_play = true;
        }
        if let Some(g) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            submit_cmd(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: id2,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit_cmd(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    e.debug_set_hp(0, 2);
    let red_sha2 = {
        let snap = e.save();
        let found = snap
            .core
            .deck
            .iter()
            .copied()
            .find(|c| {
                let d = &snap_deck_card(&snap, *c);
                matches!(d.kind, engine_native::CardKind::Sha)
                    && matches!(
                        d.suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
            })
            .expect("牌堆中有红杀");
        let mut snap = snap;
        snap.core.deck.retain(|c| *c != found);
        snap.core.players[0].hand.push(found);
        e = Engine::restore(&snap).expect("注入恢复");
        found
    };
    let hp0 = e.debug_hp(0);
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_sha2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "耀武二选挂起: {out:?}"
    );
    // 1 号摸牌可能握闪：先放弃出闪（respond_shan 含 Pass 选项），
    // 直到耀武二选打开。
    let (cid, cobs) = loop {
        let c = e.open_choice().expect("耀武选择应挂起");
        if c.spec.prompt == "yaowu_draw_or_recover" {
            assert!(
                c.spec.options.contains(&ChoiceOption::Tag(1)),
                "受伤来源有回复选项"
            );
            assert_eq!(c.observer, 0);
            break (c.id, c.observer);
        }
        let sel = vec![c.spec.options[0]]; // Pass：放弃出闪
        let (id2, obs2) = (c.id, c.observer);
        submit_cmd(
            &mut e,
            obs2,
            GameAction::Answer {
                choice_id: id2,
                selection: sel,
            },
        );
    };
    submit_cmd(
        &mut e,
        cobs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(e.debug_hp(0), hp0 + 1, "耀武：来源回复 1");
}

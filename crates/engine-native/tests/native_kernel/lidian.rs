use super::*;

/// P2/N08 标准缺失将第二批（2026-10-07，5/8）：李典「恂恂/忘隙」——
/// 装配授予 + 技能行为。恂恂为观星型：固定摸牌还回 → 观看顶四张 →
/// 两张置顶（先选先摸）两张置底 → 摸 2（净效果与上游一致）。登记差异：
/// 忘隙双方摸牌顺序为座位号升序（上游按座位序自当前回合角色起算）。
#[test]
fn lidian_assembly_xunxun_wangxi() {
    let mut e = Engine::new(80, 3);
    // 先推进再装配（阶段门不与 advance_to_play 的通用应答纠缠）。
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // ① 装配授予（双技能）。
    e.debug_assemble_general(0, "李典");
    assert!(
        e.debug_has_skill(0, "恂恂") && e.debug_has_skill(0, "忘隙"),
        "装配李典授予双技能"
    );
    assert_eq!(e.save().core.players[0].max_hp, 3, "李典体力上限 3");

    // ② 恂恂：0 号摸牌阶段开始 → 确认 → 观看顶四张选两张 → 摸到所选两张
    //（先选先摸）；未选两张置于牌堆底。
    // 下一回合推进到 0 号摸牌阶段（门在固定摸牌后打开）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Draw
            && e.open_choice().is_some()
        {
            break;
        }
        if e.turn().current != 0 || e.turn().phase != Phase::Play {
            left_play = true;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> =
                    if g.spec.prompt == "trigger_confirm" && g.observer != 0 {
                        vec![]
                    } else if g.spec.min > 0 {
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
    }
    let gate = e.open_choice().expect("恂恂确认门应挂起");
    assert_eq!((gate.spec.prompt, gate.observer), ("trigger_confirm", 0));
    let (gid, gobs) = (gate.id, gate.observer);
    submit_cmd(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let arrange = e.open_choice().expect("恂恂观看选择应挂起");
    assert_eq!(arrange.spec.prompt, "xunxun_arrange");
    assert_eq!(arrange.spec.options.len(), 4, "观看牌堆顶四张");
    assert_eq!((arrange.spec.min, arrange.spec.max), (2, 2));
    let hand0 = e.debug_hand(0).len();
    // 选前两张（先选先摸）：记录所选，断言摸到的正是这两张且按所选序。
    let chosen: Vec<u32> = arrange
        .spec
        .options
        .iter()
        .take(2)
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    let rest: Vec<u32> = arrange
        .spec
        .options
        .iter()
        .skip(2)
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    let (aid, aobs) = (arrange.id, arrange.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(chosen[0]), ChoiceOption::Card(chosen[1])],
        },
    );
    let hand = e.debug_hand(0);
    assert_eq!(hand.len(), hand0 + 2, "恂恂摸 2");
    assert_eq!(
        hand[hand.len() - 2] as u32,
        chosen[0],
        "先选的先摸到手牌倒数第二"
    );
    assert_eq!(
        hand[hand.len() - 1] as u32,
        chosen[1],
        "后选的后摸到手牌末尾"
    );
    // 未选两张置牌堆底（下次洗牌前不再被摸到）：观星同款 insert(0)
    // 循环，断言牌堆底前两位恰为两张未选牌。
    let snap = e.save();
    let bottom: Vec<u32> = snap
        .core
        .deck
        .iter()
        .take(2)
        .copied()
        .map(|c| c as u32)
        .collect();
    assert!(
        bottom.contains(&rest[0]) && bottom.contains(&rest[1]),
        "未选两张置牌堆底: {bottom:?}"
    );

    // ③ 忘隙：0 号对 1 号造成伤害（杀）→ 确认 → 双方各摸 1。
    // 推进回 0 号出牌阶段。
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
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> =
                    if g.spec.prompt == "trigger_confirm" && g.observer != 0 {
                        vec![]
                    } else if g.spec.min > 0 {
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
    }
    e.debug_clear_hand(1); // 防闪
    let sha = e.debug_place(0, "sha");
    let hand0 = e.debug_hand(0).len();
    let hand1 = e.debug_hand(1).len();
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
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
    let gate2 = e.open_choice().expect("忘隙确认门应挂起");
    assert_eq!((gate2.spec.prompt, gate2.observer), ("trigger_confirm", 0));
    let (gid2, gobs2) = (gate2.id, gate2.observer);
    submit_cmd(
        &mut e,
        gobs2,
        GameAction::Answer {
            choice_id: gid2,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0 - 1 + 1,
        "忘隙：自己摸 1（弃杀+摸1）"
    );
    assert_eq!(e.debug_hand(1).len(), hand1 + 1, "忘隙：对方摸 1");
    // ④ 忘隙放弃路径：下一回合再杀一次 → 确认门放弃 → 双方手牌不变。
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
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> =
                    if g.spec.prompt == "trigger_confirm" && g.observer != 0 {
                        vec![]
                    } else if g.spec.min > 0 {
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
    }
    e.debug_set_hp(1, 4);
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(0, "sha");
    let hand0 = e.debug_hand(0).len();
    let out = submit_cmd(
        &mut e,
        0,
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
        "{out:?}"
    );
    let gate3 = e.open_choice().expect("忘隙确认门（放弃）");
    let (gid3, gobs3) = (gate3.id, gate3.observer);
    submit_cmd(
        &mut e,
        gobs3,
        GameAction::Answer {
            choice_id: gid3,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(0).len(), hand0 - 1, "放弃：不再摸牌");
    assert!(e.debug_hand(1).is_empty(), "放弃：对方不摸");
}

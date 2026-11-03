use super::*;

/// P2/N08 标准缺失将第二批（2026-10-07，4/8）：甘夫人「淑慎/神智」——
/// 装配授予 + 技能行为。上游对照 standard/skill.js（淑慎）与
/// sp/skill.js（神智）。登记差异：原生无准备阶段，神智以判定阶段
/// 开始为回合开始锚点。
#[test]
fn ganfuren_assembly_shushen_shenzhi() {
    // ① 装配授予（双技能）。
    let mut e = Engine::new(70, 3);
    e.debug_assemble_general(0, "甘夫人");
    assert!(
        e.debug_has_skill(0, "淑慎") && e.debug_has_skill(0, "神智"),
        "装配甘夫人授予双技能"
    );
    assert_eq!(e.save().core.players[0].max_hp, 3, "甘夫人体力上限 3");

    // ② 淑慎：0 号回复体力后 → 确认 → 选 1 号 → 1 号摸 1（有手牌）。
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2);
    e.debug_place(1, "shan"); // 1 号持牌 → 淑慎摸 1（空手则摸 2）
    let tao = e.debug_move_named_card_to_hand(0, "tao");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    let gate = e.open_choice().expect("淑慎确认门应挂起");
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
    let pick = e.open_choice().expect("淑慎目标选择应挂起");
    assert_eq!(pick.spec.prompt, "shushen_target");
    assert!(pick.spec.options.contains(&ChoiceOption::Player(1)));
    let hand1 = e.debug_hand(1).len();
    let (pid, pobs) = (pick.id, pick.observer);
    submit_cmd(
        &mut e,
        pobs,
        GameAction::Answer {
            choice_id: pid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hand(1).len(), hand1 + 1, "有手牌目标摸 1");
    assert_eq!(e.debug_hp(0), 3, "桃回复生效");
    // 无手牌目标摸 2：2 号空手 → 下一回合再回复时选 2 号。
    // （同回合内淑慎可再次触发：再受疗一次。）
    e.debug_set_hp(0, 2);
    let tao2 = e.debug_move_named_card_to_hand(0, "tao");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao2,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    let gate2 = e.open_choice().expect("淑慎再次触发");
    let (gid2, gobs2) = (gate2.id, gate2.observer);
    submit_cmd(
        &mut e,
        gobs2,
        GameAction::Answer {
            choice_id: gid2,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let pick2 = e.open_choice().expect("淑慎目标选择 2");
    let (pid2, pobs2) = (pick2.id, pick2.observer);
    submit_cmd(
        &mut e,
        pobs2,
        GameAction::Answer {
            choice_id: pid2,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    assert_eq!(e.debug_hand(2).len(), 2, "无手牌目标摸 2");

    // ③ 神智（A2 S09）：下回合**准备阶段开始**（上游 phaseZhunbeiBegin；
    // 原锚判定阶段开始）→ 确认 → 弃全部手牌；数量≥体力 → 回复 1。
    // 当前 0 号 hp 3 / 手牌 4（桃吃剩 + 摸牌）——先推进到 0 号下回合开始
    //（确认门在 Prepare 帧挂起，auto-advance 前停）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Prepare
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
                        vec![] // 其他玩家可选触发放弃
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
    let gate3 = e.open_choice().expect("神智确认门应挂起");
    assert_eq!((gate3.spec.prompt, gate3.observer), ("trigger_confirm", 0));
    let (gid3, gobs3) = (gate3.id, gate3.observer);
    let hand0 = e.debug_hand(0).len();
    let hp0 = e.debug_hp(0);
    submit_cmd(
        &mut e,
        gobs3,
        GameAction::Answer {
            choice_id: gid3,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(e.debug_hand(0).is_empty(), "神智弃置全部手牌");
    if hand0 as i32 >= hp0 {
        assert_eq!(e.debug_hp(0), hp0 + 1, "手牌数≥体力 → 回复 1");
    }
    // ④ 神智放弃路径：再下回合（准备阶段开始，A2 S09）确认门放弃 → 手牌保留。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Prepare
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
    let gate4 = e.open_choice().expect("神智确认门（放弃路径）");
    let (gid4, gobs4) = (gate4.id, gate4.observer);
    let hand0 = e.debug_hand(0).len();
    submit_cmd(
        &mut e,
        gobs4,
        GameAction::Answer {
            choice_id: gid4,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(0).len(), hand0, "放弃不弃牌");
}

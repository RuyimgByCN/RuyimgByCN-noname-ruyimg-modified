use super::*;

/// P2/N08 标准缺失将收尾批（2026-10-07，8/8）：袁术「妄尊/同疾」。
/// 妄尊：锁定——其他角色回合开始（判定阶段锚点）时其体力大于你则你
/// 摸 1；该角色为主公（原型模式沿 ADR 0776 惯例视为真）再摸 1 且其
/// 手牌上限 -1 到回合结束（妄尊印，弃牌阶段结算）。
/// 同疾：锁定——其他角色杀指定唯一目标（你不在目标内、你在目标攻击
/// 范围内）时，目标弃一张牌或取消：弃置则该杀转移指向你（改写
/// CardUsed 帧载荷，sha 结算机器 Intrinsic 阶段读取）。
#[test]
fn yuanshu_assembly_wangzun_tongji() {
    // ① 装配授予（双技能）。
    let mut e = Engine::new(110, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_assemble_general(0, "袁术");
    assert!(
        e.debug_has_skill(0, "妄尊") && e.debug_has_skill(0, "同疾"),
        "装配袁术授予双技能"
    );
    assert_eq!(e.save().core.players[0].max_hp, 4, "袁术体力上限 4");

    // ② 妄尊：1 号回合开始（体力 4 > 袁术 3）→ 锁定摸 2（原型 is_lord
    // 惯例）+ 1 号获得妄尊印；1 号弃牌阶段手牌上限 -1。
    e.debug_set_hp(0, 3);
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 1
            && e.turn().phase == Phase::Judge
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 1 {
            left_play = true;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else if !g.spec.options.is_empty() {
                    vec![g.spec.options[0]]
                } else {
                    vec![]
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
    assert_eq!(
        e.debug_hand(0).len(),
        2,
        "妄尊：主公分支摸 2（原型 is_lord 惯例）"
    );
    let def_mark = e.catalog().skill_by_name("妄尊印").unwrap();
    let mark = e.save().core.skills.get(&(1, def_mark)).cloned();
    assert!(
        mark.map(|i| i.enabled && i.expires_end_of_turn)
            .unwrap_or(false),
        "1 号获得妄尊印（回合级）"
    );

    // ③ 同疾：0 号不回合内也要能触发——推进到 1 号出牌阶段，1 号杀 2 号
    //（2 号在 1 号攻击范围内、0 号在 2 号攻击范围内）→ 2 号弃牌询问 →
    // 弃置 → 杀转移指向 0 号（0 号的出闪询问）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 1
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 1 || e.turn().phase != Phase::Play {
            left_play = true;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else if !g.spec.options.is_empty() {
                    vec![g.spec.options[0]]
                } else {
                    vec![]
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
    e.debug_clear_hand(2);
    let victim_card = e.debug_place(2, "tao");
    let sha = e.debug_place(1, "sha");
    let out = submit_cmd(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    let ask = e.open_choice().expect("同疾弃牌询问应挂起");
    assert_eq!(
        (ask.spec.prompt, ask.observer),
        ("tongji_transfer_discard", 2)
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(victim_card)],
        },
    );
    assert!(
        !e.debug_hand(2).contains(&victim_card),
        "同疾：目标弃置一张"
    );
    // 转移生效：伤害指向 0 号（若被转移方有闪会收到出闪询问——逐个
    // 应答排空后看伤害落点；2 号清空手牌后必无闪）。
    for _ in 0..16 {
        match e.open_choice() {
            Some(g) => {
                eprintln!(
                    "PROBE-DRAIN prompt={} obs={} opts={:?} min={}",
                    g.spec.prompt, g.observer, g.spec.options, g.spec.min
                );
                if g.spec.options.is_empty() {
                    let _ = g;
                    break;
                }
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
            None => break,
        }
    }
    assert_eq!(e.debug_hp(0), 2, "转移后的杀命中袁术（3→2）");
    assert_eq!(e.debug_hp(2), 4, "原目标 2 号不受伤害");

    // ④ 同疾取消路径：下回合 1 号再杀 2 号（2 号有牌）→ 取消 → 不转移
    //（出闪询问指向 2 号）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 1
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 1 {
            left_play = true;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else if !g.spec.options.is_empty() {
                    vec![g.spec.options[0]]
                } else {
                    vec![]
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
    e.debug_clear_hand(2);
    let victim_card2 = e.debug_place(2, "tao");
    let sha2 = e.debug_place(1, "sha");
    let out = submit_cmd(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    let ask2 = e.open_choice().expect("同疾弃牌询问（取消路径）");
    let (aid2, aobs2) = (ask2.id, ask2.observer);
    submit_cmd(
        &mut e,
        aobs2,
        GameAction::Answer {
            choice_id: aid2,
            selection: vec![],
        },
    );
    assert!(e.debug_hand(2).contains(&victim_card2), "取消：不弃牌");
    // 取消：不转移——2 号无闪，伤害直接结算到 2 号。
    for _ in 0..16 {
        match e.open_choice() {
            Some(g) => {
                if g.spec.options.is_empty() {
                    let _ = g;
                    break;
                }
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
            None => break,
        }
    }
    assert_eq!(e.debug_hp(2), 3, "取消：杀仍命中 2 号（4→3）");
    assert_eq!(e.debug_hp(0), 2, "取消：袁术不受转移伤害");
}

use super::*;

/// P2/N08 标准缺失将第四批（2026-10-07，7/8）：伊籍「机捷/急援」——
/// 装配授予 + 技能行为。机捷：出牌阶段限一次主动技，取牌堆底一张牌
/// 交给任意存活角色（上游 chooseTarget 无过滤器，原生含自选）。
/// 急援：登记差异——仅濒死臂（任一角色濒死时可令其摸 1；上游赠牌/
/// 失去牌臂需要 giver 跟踪面，未建模）。
#[test]
fn yiji_assembly_jijie_jiyuan() {
    // ① 装配授予（双技能）。
    let mut e = Engine::new(100, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_assemble_general(0, "伊籍");
    assert!(
        e.debug_has_skill(0, "机捷") && e.debug_has_skill(0, "急援"),
        "装配伊籍授予双技能"
    );
    assert_eq!(e.save().core.players[0].max_hp, 3, "伊籍体力上限 3");

    // ② 机捷：发动 → 选 1 号 → 牌堆底一张进入 1 号手牌；牌堆底移除。
    let def_jijie = e.catalog().skill_by_name("机捷").unwrap();
    let deck_bottom = e.state_core().deck[0];
    let hand1 = e.debug_hand(1).len();
    let out = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def_jijie });
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "机捷发动: {out:?}"
    );
    let pick = e.open_choice().expect("机捷目标选择应挂起");
    assert_eq!(pick.spec.prompt, "jijie_give_target");
    assert!(pick.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(
        pick.spec.options.contains(&ChoiceOption::Player(0)),
        "上游无目标过滤器：可自选"
    );
    let (pid, pobs) = (pick.id, pick.observer);
    submit_cmd(
        &mut e,
        pobs,
        GameAction::Answer {
            choice_id: pid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(e.debug_hand(1).contains(&deck_bottom), "牌堆底牌交给 1 号");
    assert_eq!(e.debug_hand(1).len(), hand1 + 1);
    assert_ne!(e.state_core().deck[0], deck_bottom, "牌堆底已移除");
    // 每回合限一次：同回合再发动被拒。
    let out = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def_jijie });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "同回合限一次: {out:?}"
    );

    // ③ 急援（可观察场景）：1 号濒死但手握桃（可自救）——确认门后急援
    // 先摸 1（手牌 = 桃 + 急援牌），随后濒死自救用桃回 1。
    e.debug_set_hp(1, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let self_peach = e.debug_place(1, "tao");
    let sha = e.debug_place(0, "sha");
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
    let gate = e.open_choice().expect("急援确认门应挂起");
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
    // 急援摸 1 + 濒死自救询问（1 号手中有桃）挂起。
    assert_eq!(e.debug_hand(1).len(), 2, "急援：濒死者摸 1（桃 + 急援牌）");
    let save = e.open_choice().expect("濒死自救询问应挂起");
    assert_eq!(save.observer, 1);
    let (sid, sobs) = (save.id, save.observer);
    submit_cmd(
        &mut e,
        sobs,
        GameAction::Answer {
            choice_id: sid,
            selection: vec![ChoiceOption::Card(self_peach)],
        },
    );
    assert!(e.save().core.players[1].alive, "用桃自救存活");
    assert_eq!(e.debug_hp(1), 1, "桃回复 1（0→1）");
    assert_eq!(e.debug_hand(1).len(), 1, "自救用掉桃，剩急援牌");

    // ④ 急援死亡路径：下一回合再杀至濒死（无桃）→ 确认 → 摸 1 仍死。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 0 {
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
    e.debug_set_hp(1, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha2 = e.debug_place(0, "sha");
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
    let gate2 = e.open_choice().expect("急援确认门（死亡路径）");
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
    // 无人救：急援摸的牌随死亡入弃牌堆，1 号死亡（存活 2 人对局继续）。
    for _ in 0..16 {
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
            None => break,
        }
    }
    assert!(!e.save().core.players[1].alive, "无人救 → 1 号死亡");
    assert!(e.outcome().is_none(), "三人局存活 2 人，对局继续");
}

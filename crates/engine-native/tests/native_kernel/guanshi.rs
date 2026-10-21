use super::*;

/// P2/N08 批③-b 第五把（规则 v69，2026-10-07）：贯石斧（♦5，武器射程 3）
/// ——杀被闪后来源可弃两张牌（装备区牌不可含贯石斧本体）强制造成伤害；
/// 登记差异：强伤不带裸衣/酒加成。
#[test]
fn guanshi_forces_damage_after_dodge() {
    // ① 弃二强伤路径：目标出闪后 → 贯石斧询问 → 弃两张手牌 → 强制伤害。
    let mut e = Engine::new(190, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let gs = e.debug_move_named_card_to_hand(0, "guanshi");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gs,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备贯石斧: {out:?}"
    );
    let d1 = e.debug_place(0, "tao");
    let d2 = e.debug_place(0, "shan");
    let shan_t = e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 目标出闪。
    let shan_ask = e.open_choice().expect("出闪询问应挂起");
    assert_eq!(shan_ask.observer, 1);
    let (sid, sobs) = (shan_ask.id, shan_ask.observer);
    submit_cmd(
        &mut e,
        sobs,
        GameAction::Answer {
            choice_id: sid,
            selection: vec![ChoiceOption::Card(shan_t)],
        },
    );
    // 贯石斧询问（来源 0）：弃 d1 + d2 → 强制伤害。
    let ask = e.open_choice().expect("贯石斧弃二询问应挂起");
    assert_eq!((ask.spec.prompt, ask.observer), ("guanshi_force_damage", 0));
    assert!(ask.spec.options.contains(&ChoiceOption::Card(d1)));
    assert!(ask.spec.options.contains(&ChoiceOption::Card(d2)));
    assert!(
        !ask.spec.options.contains(&ChoiceOption::Card(gs)),
        "弃牌池不含贯石斧本体"
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)],
        },
    );
    assert!(
        !e.debug_hand(0).contains(&d1) && !e.debug_hand(0).contains(&d2),
        "两张牌已弃"
    );
    assert_eq!(e.debug_hp(1), 3, "贯石斧强制伤害（被闪仍命中）");

    // ② 放弃路径：目标出闪后 → 放弃 → 不造成伤害。
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
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    // 来源持两张可弃牌（放弃路径需池 ≥2 才会开询问）。
    let _d3 = e.debug_place(0, "tao");
    let _d4 = e.debug_place(0, "shan");
    // 目标再持一张闪（放弃路径需要目标能出闪）。
    let _shan_extra = e.debug_place(1, "shan");
    let _shan_t2 = e.debug_place(1, "shan");
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![1]);
    let shan_ask2 = e.open_choice().expect("出闪询问（放弃路径）");
    let (sid2, sobs2) = (shan_ask2.id, shan_ask2.observer);
    let first_opt = shan_ask2
        .spec
        .options
        .get(1)
        .copied()
        .or_else(|| shan_ask2.spec.options.first().copied());
    // 用第一张可用闪应答（保证真正触发被闪路径）。
    submit_cmd(
        &mut e,
        sobs2,
        GameAction::Answer {
            choice_id: sid2,
            selection: vec![first_opt.expect("有闪可出")],
        },
    );
    let ask2 = e.open_choice().expect("贯石斧询问（放弃路径）");
    assert_eq!(
        (ask2.spec.prompt, ask2.observer),
        ("guanshi_force_damage", 0)
    );
    let (aid2, aobs2) = (ask2.id, ask2.observer);
    submit_cmd(
        &mut e,
        aobs2,
        GameAction::Answer {
            choice_id: aid2,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hp(1), 3, "放弃：不强制伤害（被闪后无伤）");
}

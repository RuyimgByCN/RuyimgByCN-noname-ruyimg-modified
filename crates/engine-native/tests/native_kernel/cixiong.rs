use super::*;

/// P2/N08 批③-b 第八把（规则 v72）+ 逐技对照批一（规则 v75）：雌雄双股剑
///（♠2，射程 2）——杀指定异性目标后（性别有槽取武将性别、无槽默认男），
/// 目标**自选**弃一张手牌（对齐上游 chooseToDiscard）或取消令使用者摸一张。
#[test]
fn cixiong_opposite_gender_discard_or_draw() {
    // ① 目标女性（甄姬）有手牌 → Confirm=弃一张手牌。
    let mut e = Engine::new(230, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_assemble_general(1, "甄姬");
    let cx = e.debug_move_named_card_to_hand(0, "cixiong");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cx,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备雌雄: {out:?}"
    );
    let hand_card = e.debug_place(1, "tao");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    let ask = e.open_choice().expect("雌雄询问应挂起");
    assert_eq!(
        (ask.spec.prompt, ask.spec.kind, ask.observer),
        (
            "cixiong_discard_choose",
            engine_native::ChoiceKind::SelectCards,
            1
        )
    );
    assert_eq!(
        ask.spec.options,
        vec![ChoiceOption::Card(hand_card)],
        "选项=目标手牌"
    );
    assert_eq!(
        (ask.spec.min, ask.spec.allow_cancel),
        (0, true),
        "可取消=令使用者摸"
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(hand_card)],
        },
    );
    assert!(!e.debug_hand(1).contains(&hand_card), "目标自选的手牌被弃");
    assert_eq!(e.debug_hand(0).len(), 0, "使用者未摸牌");

    // ② 取消路径：下回合再杀 1（甄姬），空应答 = 令使用者摸 1、目标手牌保留。
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
    let cancel_card = e.debug_place(1, "tao");
    let before0 = e.debug_hand(0).len();
    let hp_before = e.debug_hp(1);
    let sha_mid = e.debug_place(0, "sha");
    play(&mut e, sha_mid, vec![1]);
    let ask = e.open_choice().expect("雌雄取消询问应挂起");
    assert_eq!(
        (ask.spec.prompt, ask.observer),
        ("cixiong_discard_choose", 1)
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(0).len(), before0 + 1, "取消=使用者摸 1");
    assert!(e.debug_hand(1).contains(&cancel_card), "目标手牌保留");
    // 排空出闪/伤害（甄姬无闪：强制 Pass → 受 1 伤）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = g
                    .spec
                    .options
                    .iter()
                    .take(g.spec.min.max(1))
                    .copied()
                    .collect();
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
    assert_eq!(e.debug_hp(1), hp_before - 1, "取消路径：伤害照常（-1）");

    // ③ 同性目标（张飞男）不触发：直接进入出闪/伤害。
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
    e.debug_assemble_general(2, "张飞");
    let keep = e.debug_place(2, "tao");
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![2]);
    assert!(
        e.open_choice()
            .map(|c| c.spec.prompt != "cixiong_discard_choose")
            .unwrap_or(true),
        "同性目标不触发雌雄"
    );
    // 排空出闪/伤害。
    for _ in 0..8 {
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
    assert!(e.debug_hand(2).contains(&keep), "同性目标手牌保留");
}

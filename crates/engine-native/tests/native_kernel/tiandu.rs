use super::*;

/// 天妒（owner_is_target，对齐旧线内容定义/标准包「你的判定牌」）：
/// 自己判定牌生效后，持有者可将其收入手牌（不入弃牌堆）；
/// 他人的判定不触发天妒。
#[test]
fn tiandu_gain_own_judge_card_only() {
    let mut e = Engine::new(213, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 只给判定阶段拥有者 1 授予天妒：0 的乐打给 1，判定与获得都在 1
    e.debug_grant(1, "天妒");
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
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
    assert_eq!(prompt_of(&e), "tian_du_gain");
    assert_eq!(
        e.open_choice().unwrap().observer,
        1,
        "owner_is_target：由 1 自己决定"
    );
    let judged_card = judged_card_of(&e);
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
    // 事件顺序：Finalized（判定确定）→ Gained（收入手牌）
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementGained { player: 1, card } if *card == judged_card)));
    assert!(e.debug_hand(1).contains(&judged_card), "判定牌收入手牌");
    assert_eq!(discard_count(&e, judged_card), 0, "获得的判定牌不入弃牌堆");
    assert_zone_conservation(&e);
}

/// N08 他人天妒持有者不越权：天妒在 0 号，判定属于 1 号——
/// 不产生天妒询问，判定牌正常弃置。
#[test]
fn tiandu_other_holder_not_asked() {
    let mut e = Engine::new(243, 4);
    e.debug_grant(0, "天妒"); // 他人持有（判定属 1 号）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    // 推进经过 1 的判定阶段：不得出现天妒询问
    let mut saw_tian_du = false;
    for _ in 0..24 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "tian_du_gain" {
                saw_tian_du = true;
                break;
            }
            // 其他选择按首选项应答
            let sel = vec![c.spec.options[0]];
            let a = c.observer;
            let cid = c.id;
            let r = e.submit(GameCommand {
                actor: a,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            });
            match r {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => continue,
                other => panic!("{other:?}"),
            }
        }
        let t = e.turn();
        if t.current != 0 && t.phase == Phase::Play {
            // 已进入他人回合的出牌阶段：判定阶段已过
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(!saw_tian_du, "他人判定不得询问非持有者天妒");
    assert!(
        !log_has(&e, |l| matches!(l, LE::JudgementGained { .. })),
        "无人持有判定者的天妒，不得获得"
    );
}

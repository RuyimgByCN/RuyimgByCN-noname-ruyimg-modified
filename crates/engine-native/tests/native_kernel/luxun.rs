use super::*;

/// 谦逊（陆逊）：不能成为乐不思蜀目标（兵粮不受限——谦逊只防乐/顺牵）。
#[test]
fn luxun_qianxun_blocks_le_target() {
    let mut e = Engine::new(903, 3);
    advance_to_play(&mut e, 0);
    e.debug_grant(1, "谦逊");
    let le = e.debug_place(0, "le");
    assert!(
        matches!(
            play(&mut e, le, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "谦逊不得成为乐不思蜀目标"
    );
}

/// 连营（陆逊）：出牌失去最后一张手牌后摸一；给出路径同理。
#[test]
fn luxun_lianying_draws_after_losing_last_card() {
    let mut e = Engine::new(904, 3);
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "连营");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 恰置一张牌于手：打出后手牌清空 → 连营确认 → 摸一
    let sha = e.debug_place(0, "sha");
    assert_eq!(e.debug_hand(0).len(), 1);
    play(&mut e, sha, vec![1]);
    let c = e.open_choice().expect("连营触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    answer(&mut e, ChoiceOption::Confirm);
    assert_eq!(e.debug_hand(0).len(), 1, "连营应摸回一张");
    // 手牌多于一张时失去不触发（决斗无次数限制）
    let _tao = e.debug_place(0, "tao");
    let duel = e.debug_place(0, "duel");
    assert!(matches!(
        play(&mut e, duel, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    assert!(e.open_choice().is_none(), "非最后一张不得触发连营");
    assert_zone_conservation(&e);
}

/// 连营弃牌路径（ADR 0763）：被过河拆桥弃掉最后一张手牌 →
/// 连营触发（CardsDiscarded 事件）；同时验证顺手牵羊的转移也触发。
#[test]
fn luxun_lianying_triggers_on_forced_discard() {
    // (1) 拆桥弃掉最后一张手牌 → 连营确认 → 摸回一张
    let mut e = Engine::new(987, 3);
    advance_to_play(&mut e, 0);
    e.debug_grant(1, "连营");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let last = e.debug_place(1, "tao");
    let dismantle = e.debug_place(0, "dismantle");
    play(&mut e, dismantle, vec![1]);
    let c = e.open_choice().expect("连营触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    answer(&mut e, ChoiceOption::Confirm);
    assert_eq!(e.debug_hand(1).len(), 1, "连营应摸回一张");
    assert!(!e.debug_hand(1).contains(&last));
    assert_zone_conservation(&e);

    // (2) 顺手牵羊夺走最后一张手牌 → 连营同样触发
    let mut e2 = Engine::new(988, 3);
    advance_to_play(&mut e2, 0);
    e2.debug_grant(1, "连营");
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    let last2 = e2.debug_place(1, "tao");
    let snatch = e2.debug_place(0, "snatch");
    play(&mut e2, snatch, vec![1]);
    let c2 = e2.open_choice().expect("连营触发确认应挂起");
    assert_eq!(c2.spec.prompt, "trigger_confirm");
    answer(&mut e2, ChoiceOption::Confirm);
    assert_eq!(e2.debug_hand(1).len(), 1, "连营应摸回一张");
    assert!(!e2.debug_hand(1).contains(&last2));
    assert!(e2.debug_hand(0).contains(&last2), "拆桥方获牌不受影响");
    assert_zone_conservation(&e2);
}

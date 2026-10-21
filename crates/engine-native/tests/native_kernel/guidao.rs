use super::*;

/// 鬼道：改判者打出黑色手牌替换判定牌（红桃→黑桃），乐生效跳过出牌阶段。
#[test]
fn guidao_rewrite_replaces_judgment() {
    let (mut e, le) = build_guidao_pending();
    e.debug_grant(2, "鬼道");
    let black = place_black_sha(&mut e, 2);
    // 推进到 1 的判定阶段：改判询问（观察者 2）
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
    assert_eq!(prompt_of(&e), "gui_dao_rewrite");
    assert_eq!(e.open_choice().unwrap().observer, 2);
    let old_judge = judged_card_of(&e);
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Card(black)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 事件断言：改判 → 最终判定（黑桃）→ 无天妒不获得
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementRewritten { player: 2, card, replaced }
        if *card == black && *replaced == old_judge)));
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementFinalized { player: 1, card } if *card == black)));
    assert!(!log_has(&e, |l| matches!(l, LE::JudgementGained { .. })));
    // skip_play_phase 直接断言：Judge 结束 → Draw 后直达 Discard（不经 Play）
    next_phase(&mut e); // Judge 结束 → Draw 开始（摸 2）
    next_phase(&mut e); // Draw 结束 → skip_play → 直达 Discard
    assert_eq!(e.turn().phase, Phase::Discard, "skip_play 应直接进 Discard");
    // 被替换的旧判定牌由改判者获得（对齐上游），新判定牌生效后弃置恰一份
    assert!(
        e.debug_hand(2).contains(&old_judge),
        "鬼道应获得被替换的旧判定牌"
    );
    assert_eq!(discard_count(&e, black), 1, "打出的黑色牌生效后弃置恰一份");
    assert_eq!(discard_count(&e, le), 1, "乐经判定弃置恰一份");
    assert_zone_conservation(&e);
}

/// 鬼道：多个持有者按座位顺序（自判定阶段拥有者起）逐人询问一次；
/// 双方都改判——后改者替换先改者的牌并获得之，最终以最后改判牌生效。
#[test]
fn guidao_two_holders_asked_in_seat_order() {
    let (mut e, _le) = build_guidao_pending();
    e.debug_grant(2, "鬼道");
    e.debug_grant(3, "鬼道");
    let _black2 = place_black_sha(&mut e, 2);
    let black3 = place_black_sha(&mut e, 3);
    // 拥有者 1 → 座位序 [1,2,3,4,0]：先问 2 再问 3
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
    assert_eq!(e.open_choice().unwrap().observer, 2, "先问 2");
    let cid2 = e.open_choice().unwrap().id;
    // 双方都改判：2 先出黑牌，3 再出黑牌替换（并获得 2 的牌）
    let black2 = e
        .debug_hand(2)
        .iter()
        .copied()
        .find(|c| {
            matches!(
                e.catalog().cards[*c as usize].suit,
                Suit::Spade | Suit::Club
            )
        })
        .expect("2 应有黑色牌");
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid2,
            selection: vec![ChoiceOption::Card(black2)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "链式询问 3：{out:?}"
    );
    assert_eq!(e.open_choice().unwrap().observer, 3);
    let cid3 = e.open_choice().unwrap().id;
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid3,
            selection: vec![ChoiceOption::Card(black3)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 改判事件两次：2 出 black2、3 出 black3 替换 black2；最终判定 = black3
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementRewritten { player: 2, card, .. } if *card == black2)));
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementRewritten { player: 3, card, replaced } if *card == black3 && *replaced == black2)));
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementFinalized { player: 1, card } if *card == black3)));
    // 归属：black2 由 2 转给 3；black3 生效后弃置恰一份；black2 不在弃牌堆
    assert!(e.debug_hand(3).contains(&black2), "3 应获得 2 的改判牌");
    assert!(e.debug_hand(2).is_empty() || !e.debug_hand(2).contains(&black2));
    assert_eq!(discard_count(&e, black3), 1, "最终判定牌生效后弃置恰一份");
    assert_eq!(discard_count(&e, black2), 0, "被 3 获得的牌不入弃牌堆");
    // 最终判定黑桃 → 乐生效
    advance_to_play(&mut e, 1);
}

/// 鬼道 + 天妒 组合：改判后的最终判定牌由天妒持有者获得。
#[test]
fn guidao_tiandu_combo_final_card_gained() {
    let (mut e, _le) = build_guidao_pending();
    e.debug_grant(2, "鬼道");
    e.debug_grant(1, "天妒");
    let black = place_black_sha(&mut e, 2);
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
    // 2 改判成黑牌（2 获得被替换的旧判定牌）→ 生效（乐跳过出牌）→
    // 天妒询问（判定拥有者 1 持有）确认获得最终判定牌
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Card(black)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "链到天妒询问：{out:?}"
    );
    assert_eq!(prompt_of(&e), "tian_du_gain");
    let old_judge = judged_card_of(&e);
    assert!(
        e.debug_hand(2).contains(&old_judge),
        "鬼道应获得被替换的旧判定牌"
    );
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
    assert!(
        e.debug_hand(1).contains(&black),
        "改判后的最终判定牌应被获得"
    );
    assert_eq!(discard_count(&e, black), 0);
    assert_zone_conservation(&e);
}

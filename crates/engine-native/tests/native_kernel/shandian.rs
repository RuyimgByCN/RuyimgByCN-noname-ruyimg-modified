use super::*;

/// 闪电：黑桃 2-9 判定命中 → 3 点雷伤；未命中传下家判定区。
#[test]
fn shandian_hits_and_transfers() {
    // 命中：黑桃 2-9 判定 → 3 点雷伤（无来源）
    let mut e = Engine::new(165, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sd = e.debug_place(0, "shandian");
    assert!(matches!(
        play(&mut e, sd, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    // 闪电在 0 自己的下一回合判定阶段结算：推进一圈；
    // 判定牌在 3 号（0 的上家）End 阶段才压顶，避免被中途摸牌抽走
    for _ in 0..24 {
        if hp(&e, 0) != 4 {
            break;
        }
        if e.turn().current == 3 && e.turn().phase == Phase::End {
            e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => answer_cards(&mut e, &choice),
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(hp(&e, 0), 1, "闪电命中应受 3 点雷伤");
    assert!(e.debug_judge(0).is_empty());
    assert!(log_has(&e, |l| matches!(
        l,
        LE::DamageDealt {
            target: 0,
            amount: 3,
            ..
        }
    )));

    // 未命中：红桃判定 → 传下家判定区
    let mut e = Engine::new(167, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sd = e.debug_place(0, "shandian");
    assert!(matches!(
        play(&mut e, sd, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    // 推进到 0 的下一回合判定：未命中 → 传给下家（1）判定区
    for _ in 0..24 {
        if !e.debug_judge(1).is_empty() {
            break;
        }
        if e.turn().current == 3 && e.turn().phase == Phase::End {
            // 黑桃但点数在 2-9 之外 → 不命中，传下家
            e.debug_set_deck_top_by(|def| def.suit.is_spade() && !(2..=9).contains(&def.rank));
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => answer_cards(&mut e, &choice),
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(hp(&e, 0), 4, "未命中不受伤害");
    assert_eq!(e.debug_judge(1).len(), 1, "闪电应传到下家判定区");
}

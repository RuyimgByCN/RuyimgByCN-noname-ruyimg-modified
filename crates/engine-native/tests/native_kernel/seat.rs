use super::*;

// ---------------------------------------------------------------------------
// N10 有界计算池：每座位每轮配额——越界拒绝与公平性（ADR 0732）
// ---------------------------------------------------------------------------

/// N10 有界计算池：配额耗尽拒绝（QuotaExhausted）、拒绝不消耗配额、
/// 每座位池相互独立（公平性）、轮次翻转自动归零、宿主可随时调整。
#[test]
fn seat_quota_bounds_commands_and_resets_per_round() {
    let mut e = Engine::new(419, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0); // 清空手牌：阶段推进不触发弃牌窗口，计数确定
    e.set_seat_quota(Some(2));
    let actor = e.turn().current;
    assert_eq!(actor, 0);
    assert_eq!(e.seat_quota_remaining(0), Some(2));

    // 座位 0 的前两条命令被接受并计入配额
    for i in 0..2 {
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }),
            "第 {i} 条应接受: {out:?}"
        );
        assert_eq!(e.seat_quota_remaining(0), Some(1 - i));
    }
    // 第三条：配额耗尽 → QuotaExhausted（此时 0 仍是当前回合玩家，非回合错位）
    assert_eq!(e.turn().current, 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::QuotaExhausted)),
        "超配额应拒绝: {out:?}"
    );
    // 拒绝不消耗配额：余量仍为 0
    assert_eq!(e.seat_quota_remaining(0), Some(0));

    // 公平性：其他座位拥有独立配额池。先放宽让回合推进到 1 号，
    // 再恢复配额 2，验证 1 号拥有自己的满额池。
    e.set_seat_quota(None);
    loop {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        let out = e.submit(GameCommand {
            actor: e.turn().current,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        match out {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e.debug_clear_hand(1); // 清空 1 号手牌：避免弃牌窗口干扰计数
    e.set_seat_quota(Some(2));
    assert_eq!(e.seat_quota_remaining(1), Some(2), "1 号应有自己的满额池");
    for i in 0..2 {
        let out = e.submit(GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
        let _ = i;
    }
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::QuotaExhausted)
    ));

    // 轮次归零：放宽推进到下一轮 0 号回合，恢复配额后 0 号重新有池
    e.set_seat_quota(None);
    loop {
        let t = e.turn();
        if t.current == 0 && t.round >= 2 && t.phase == Phase::Play {
            break;
        }
        let out = e.submit(GameCommand {
            actor: e.turn().current,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        match out {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => answer_and_drive(&mut e, out),
            other => panic!("{other:?}"),
        }
    }
    e.debug_clear_hand(0);
    e.set_seat_quota(Some(2));
    assert_eq!(e.seat_quota_remaining(0), Some(2), "新轮次配额应归零重置");
    // 拒绝型命令（此处为非当前回合玩家）不消耗配额
    let wrong = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(wrong, CommandOutcome::Rejected(RejectReason::NotYourTurn)),
        "越权命令应被拒绝: {wrong:?}"
    );
    assert_eq!(e.seat_quota_remaining(0), Some(2), "被拒命令不应消耗配额");
}

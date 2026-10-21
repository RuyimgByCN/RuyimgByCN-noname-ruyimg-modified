use super::*;

/// N07 非空日志中止：在已有事件日志的段上 abort 后日志截断至段前。
#[test]
fn abort_segment_truncates_log() {
    let mut e = Engine::new(7, 4);
    // 第一段：用小预算制造快速耗尽
    e.debug_set_budget(1);
    let pre_hash = e.state_hash();
    let cur = e.turn().current;
    let out = e.submit(GameCommand {
        actor: cur,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::BudgetExhausted)),
        "{out:?}"
    );
    // 第二段：正常预算 + 时间片 → 暂停后中止
    e.debug_set_budget(1000);
    e.set_time_slice(Some(1));
    let out = e.submit(GameCommand {
        actor: cur,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    // 续行几次后中止
    let _ = e.resume();
    e.abort_segment().expect("abort open segment");
    assert!(!e.is_suspended());
    assert_eq!(e.state_hash(), pre_hash, "中止后应回到段前状态");
    // 取消时间片后继续对局
    e.set_time_slice(None);
    // 对局可继续
    let cur = e.turn().current;
    let out = e.submit(GameCommand {
        actor: cur,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
}

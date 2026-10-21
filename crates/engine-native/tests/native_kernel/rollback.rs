use super::*;

#[test]
fn budget_exhaustion_rolls_back() {
    let mut e = Engine::new(81, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    let (choice_id, _) = open_choice_snapshot(&e);

    // 极小预算：确认后的完整结算链必然耗尽
    e.debug_set_budget(2);
    let hash_before = e.state_hash();
    match e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Confirm],
        },
    }) {
        CommandOutcome::Rejected(RejectReason::BudgetExhausted) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(e.state_hash(), hash_before, "预算耗尽必须整段回滚");
    assert_eq!(open_choice_snapshot(&e), (choice_id, "trigger_confirm"));

    // 恢复预算后可正常推进
    e.debug_set_budget(4096);
    assert!(matches!(
        answer(&mut e, ChoiceOption::Confirm),
        CommandOutcome::Pending { .. }
    ));
}

/// N07-b：恢复件保留可用检查点——「耗尽后保存恢复→中止」回滚到段启动。
#[test]
fn abort_after_restore_restores_segment_start() {
    let (mut e, pre_hash, pre_rev) = build_stuck_suspended_segment(7, 80_000);
    // 挂起中保存：快照必须携带段检查点
    let mid = e.save();
    assert!(mid.segment_open);
    assert!(mid.segment_checkpoint.is_some(), "快照必须携带段检查点");
    // 耗尽（提交进度）后再保存：检查点仍在
    let _ = e.resume();
    assert!(matches!(
        e.resume(),
        CommandOutcome::Rejected(RejectReason::BudgetExhausted)
    ));
    let post = e.save();
    assert!(
        post.segment_checkpoint.is_some(),
        "耗尽后快照仍须携带检查点"
    );

    // 恢复（挂起中快照）→ 中止 → 回滚到段启动
    let mut r = Engine::restore(&mid).expect("恢复");
    assert!(r.is_suspended());
    r.abort_segment().expect("恢复件必须可中止");
    assert!(!r.is_suspended());
    assert_eq!(r.state_hash(), pre_hash, "恢复件中止回滚到段启动");
    assert_eq!(r.revision(), pre_rev, "恢复件中止恢复段前修订号");
    assert!(r.log().is_empty(), "恢复件日志从空开始，中止截断到 0");
    // 同键重试能完成：提高预算后同键命令完成
    r.debug_set_budget(4096);
    let cur = r.turn().current;
    let out = r.submit(GameCommand {
        actor: cur,
        expected_revision: r.revision(),
        dedup_id: 80_000,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "同键重试必须能完成：{out:?}"
    );
}

/// N07-b：直接中止与「耗尽后保存恢复→中止」契约对照（状态/栈/选择/修订/去重）。
#[test]
fn abort_contract_direct_vs_restored_identical() {
    // E1：直接中止
    let (mut e1, _pre_hash, _pre_rev) = build_stuck_suspended_segment(7, 81_000);
    e1.abort_segment().expect("direct abort");
    let h1 = e1.state_hash();
    let rev1 = e1.revision();
    let stack1 = e1.save().resolution.stack.len();
    let parks1 = e1.save().resolution.parks.len();

    // E2：耗尽前保存 → 恢复 → 中止
    let mut e2 = build_stuck_suspended_segment(7, 81_000).0;
    let mid = e2.save();
    let _ = e2.resume();
    let mut r = Engine::restore(&mid).expect("restore");
    r.abort_segment().expect("restored abort");
    let h2 = r.state_hash();
    let rev2 = r.revision();
    let stack2 = r.save().resolution.stack.len();
    let parks2 = r.save().resolution.parks.len();

    assert_eq!(h1, h2, "直接中止与恢复后中止终态一致");
    assert_eq!(rev1, rev2);
    assert_eq!(stack1, stack2);
    assert_eq!(parks1, parks2);
    // 日志契约：E1 截断回段前水位；E2（恢复件）截断到恢复点即空——
    // 宿主日志不随快照/中止恢复，恢复点即新起点（ADR 0709/0710）。

    // 同键重试均能完成且终态一致（中止后在「下一段」调整预算重试，
    // 并取消时间片使段可完成——不宣称提高预算后可直接 resume）
    for e in [&mut e1, &mut r] {
        e.set_time_slice(None);
        e.debug_set_budget(4096);
        let cur = e.turn().current;
        let out = e.submit(GameCommand {
            actor: cur,
            expected_revision: e.revision(),
            dedup_id: 81_000,
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
    assert_eq!(r.state_hash(), e1.state_hash(), "重试完成终态一致");
}

// ---------------------------------------------------------------------------
// N07 失败契约：暂停段预算耗尽 → abort_segment 回滚段前（复核复现项关闭）
// ---------------------------------------------------------------------------

/// N07：暂停段预算耗尽后段保持开放、命令/resume 均不可用；
/// `abort_segment` 恢复段前状态/修订号/去重水位/日志，对局可继续。
#[test]
fn aborted_segment_restores_pre_command_state() {
    let mut e = Engine::new(7, 4);
    e.debug_set_budget(2);
    let pre_hash = e.state_hash();
    let pre_rev = e.revision();
    let pre_log_len = e.log().len();
    e.set_time_slice(Some(1));

    // 段启动（固定去重键，用于验证中止后键退还）
    let starter_dedup = 70_000;
    let out = e.submit(GameCommand {
        actor: e.turn().current,
        expected_revision: e.revision(),
        dedup_id: starter_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    // 续行至硬预算耗尽：段保持开放
    let out = e.resume();
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::BudgetExhausted)
    ));
    assert!(e.is_suspended());
    // 卡住状态：新命令 SegmentSuspended、resume 确定性重复 BudgetExhausted
    let probe_actor = e.turn().current;
    let out = submit_cmd(&mut e, probe_actor, GameAction::NextPhase);
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::SegmentSuspended)
    ));
    assert!(matches!(
        e.resume(),
        CommandOutcome::Rejected(RejectReason::BudgetExhausted)
    ));

    // 中止：回滚到段前
    e.abort_segment().expect("开放段必须可中止");
    assert!(!e.is_suspended());
    assert_eq!(e.state_hash(), pre_hash, "中止恢复段前状态");
    assert_eq!(e.revision(), pre_rev, "修订号恢复段前值");
    assert_eq!(e.log().len(), pre_log_len, "宿主日志截断回段前");
    // 去重水位退还：段启动键可再次使用
    let cur = e.turn().current;
    let rev = e.revision();
    let out = e.submit(GameCommand {
        actor: cur,
        expected_revision: rev,
        dedup_id: starter_dedup,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. }
                | CommandOutcome::Pending { .. }
                | CommandOutcome::Suspended
        ),
        "中止后同键重试应被接受（预算覆盖仍为 2，可再次暂停）：{out:?}"
    );
    // 中止后再次放弃该段也应可中止（契约可重复使用）
    e.abort_segment().expect("再次中止");
    assert!(!e.is_suspended());
}

/// N07：未开放段调用 abort → NotSuspended；正常段（无分片）失败可整段回滚。
#[test]
fn abort_requires_open_segment_and_nonsliced_rollback_unchanged() {
    let mut e = Engine::new(7, 4);
    assert!(matches!(e.abort_segment(), Err(RejectReason::NotSuspended)));
    // 不限片的普通失败：整段回滚、状态逐字节不变（既有语义回归）
    e.debug_set_budget(2);
    let hash = e.state_hash();
    let cur = e.turn().current;
    let out = submit_cmd(&mut e, cur, GameAction::NextPhase);
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::BudgetExhausted)
    ));
    assert_eq!(e.state_hash(), hash, "非分片失败整段回滚");
    assert!(!e.is_suspended());
}

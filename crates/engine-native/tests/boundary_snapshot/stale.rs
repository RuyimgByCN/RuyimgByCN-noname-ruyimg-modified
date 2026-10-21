use super::*;

/// 过期修订号 → StaleRevision；拒绝不消耗去重键（同键以正确修订号重试成功）。
#[test]
fn stale_revision_rejected_then_retry_same_dedup() {
    let (mut e, _) = setup_play_with_sha();
    let rev = e.revision();
    let fixed_dedup = 90_000;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev + 1,
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::StaleRevision)),
        "{out:?}"
    );
    assert_eq!(e.revision(), rev, "拒绝不改状态");
    // 同键、正确修订号 → 接受（证明拒绝路径未消耗去重键）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.revision(), rev + 1);
}

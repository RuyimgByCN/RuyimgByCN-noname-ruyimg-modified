use super::*;

/// N04 检查点与当前段关系（正向）：开放段快照的检查点单调不超前、
/// 静止可中止；恢复件保留段开放状态且 abort 可用。
#[test]
fn open_segment_checkpoint_association_positive() {
    let (e, snap) = build_open_segment();
    let ckpt = snap
        .segment_checkpoint
        .as_ref()
        .expect("开放段快照必须携带检查点");
    // 单调关联：检查点 revision/dedup 不超前当前；自身静止（非段开放）
    assert!(
        ckpt.core.revision <= snap.core.revision,
        "检查点 revision 单调"
    );
    assert!(
        ckpt.last_dedup <= snap.last_dedup,
        "检查点 dedup 高水位单调"
    );
    assert!(!ckpt.segment_open, "检查点是段启动时的静止状态");
    assert!(!ckpt.segment_checkpoint.is_some(), "检查点不再嵌套");
    // 恢复件：段开放延续、abort 可回滚到段前并继续对局
    let mut r = Engine::restore(&snap).expect("restore open segment");
    assert!(r.is_suspended(), "恢复件应延续段开放状态");
    r.abort_segment().expect("恢复件 abort 应可用");
    assert!(!r.is_suspended());
    let cur = r.turn().current;
    let out = submit_cmd(&mut r, cur, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let _ = e;
}

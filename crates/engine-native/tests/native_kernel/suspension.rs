use super::*;

/// 分片与连续运行的终态摘要、日志事件序列完全一致；片越小暂停越多。
#[test]
fn time_slice_resumes_to_identical_result() {
    let (base, base_suspends) = drive_sancai_sliced(None);
    assert_eq!(base_suspends, 0, "不限片不应暂停");
    let base_log: Vec<_> = base.log().to_vec();

    for slice in [1u32, 3, 7, 64] {
        let (sliced, suspends) = drive_sancai_sliced(Some(slice));
        assert_eq!(
            sliced.state_hash(),
            base.state_hash(),
            "片={slice} 终态摘要必须与连续运行一致"
        );
        assert_eq!(sliced.log(), base_log, "片={slice} 事件序列必须一致");
        if slice <= 7 {
            assert!(suspends > 0, "片={slice} 小于段长，应产生暂停");
        }
        if slice == 1 {
            assert!(suspends >= 10, "片=1 应频繁暂停，实际 {suspends}");
        }
        // 片=64 可能大于各段步数：零暂停且结果一致即为通过
    }
}

/// 暂停中的段拒绝一切新命令且状态不变；未暂停的引擎拒绝 resume。
#[test]
fn suspended_segment_rejects_commands_and_resume_requires_open_segment() {
    let mut e = Engine::new(23, 4);
    assert!(matches!(
        e.resume(),
        CommandOutcome::Rejected(RejectReason::NotSuspended)
    ));
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.set_time_slice(Some(1));
    let out = play(&mut e, sha, vec![0]);
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    let hash = e.state_hash();
    // 暂停中的段：任何命令都被拒绝且不消耗去重键、不改状态
    let actor_now = e.turn().current;
    let out = submit_cmd(&mut e, actor_now, GameAction::NextPhase);
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::SegmentSuspended)
    ));
    assert_eq!(e.state_hash(), hash);
    // 续行至完成
    let mut out = e.resume();
    loop {
        match out {
            CommandOutcome::Suspended => {
                out = e.resume();
            }
            CommandOutcome::Pending { choice } => {
                let sel = sancai_answer(&choice);
                let actor = choice.observer;
                let cid = choice.id;
                out = e.submit(GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                });
            }
            CommandOutcome::Accepted { .. } => break,
            CommandOutcome::Rejected(r) => panic!("{r:?}"),
        }
    }
    assert_ne!(e.state_hash(), hash, "续行必须推进状态");
}

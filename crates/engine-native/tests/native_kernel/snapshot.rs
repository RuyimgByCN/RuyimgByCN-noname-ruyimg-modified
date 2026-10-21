use super::*;

/// 暂停点快照往返：暂停中的状态可保存/恢复，续行后与连续运行同终态。
#[test]
fn snapshot_roundtrip_while_suspended() {
    // 基准：不限片
    let (base, _) = drive_sancai_sliced(None);

    // 快照线：片=3 跑到某个暂停点，保存→恢复→继续
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.set_time_slice(Some(3));
    let mut out = play(&mut e, sha, vec![0]);
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    let snap = e.save();
    let mut r = Engine::restore(&snap).expect("暂停态可恢复");
    assert_eq!(r.state_hash(), e.state_hash());
    // 恢复件继续跑完（用同样的应答脚本）
    let mut answered = 0;
    loop {
        out = match out {
            CommandOutcome::Suspended => r.resume(),
            CommandOutcome::Pending { choice } => {
                answered += 1;
                let sel = sancai_answer(&choice);
                let actor = choice.observer;
                let cid = choice.id;
                r.submit(GameCommand {
                    actor,
                    expected_revision: r.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                })
            }
            CommandOutcome::Accepted { .. } => break,
            CommandOutcome::Rejected(rej) => panic!("{rej:?}"),
        };
    }
    assert_eq!(answered, 3);
    assert_eq!(
        r.state_hash(),
        base.state_hash(),
        "暂停点恢复后的终态必须一致"
    );
}

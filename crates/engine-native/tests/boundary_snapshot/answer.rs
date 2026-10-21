use super::*;

// ---------------------------------------------------------------------------
// 命令信任边界
// ---------------------------------------------------------------------------

/// 应答他人的选择 → WrongPlayer；状态不变。
#[test]
fn answer_by_wrong_player_rejected() {
    let mut e = setup_sancai_pending();
    let id = e.open_choice().unwrap().id;
    assert_eq!(e.open_choice().unwrap().observer, 0);
    let before = e.state_hash();
    let out = e.submit(GameCommand {
        actor: 1, // 非观察者
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::WrongPlayer)),
        "{out:?}"
    );
    assert_eq!(e.state_hash(), before);
    // 观察者本人应答正常
    assert!(matches!(
        e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Confirm],
            },
        }),
        CommandOutcome::Pending { .. } // 链到下一个选择（sancai_target）
    ));
}

/// 无选择挂起时 Answer → NoSuchChoice。
#[test]
fn answer_without_open_choice_rejected() {
    let (mut e, _) = setup_play_with_sha();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: 1,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchChoice)),
        "{out:?}"
    );
}

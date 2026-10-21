use super::*;

/// 已接受命令的去重键不可复用；键 0 恒拒绝。
#[test]
fn duplicate_and_zero_dedup_rejected() {
    let (mut e, _) = setup_play_with_sha();
    let rev = e.revision();
    let fixed_dedup = 91_000;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 完全相同的键（即使内容以新状态合法）→ DuplicateCommand
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(RejectReason::DuplicateCommand)
        ),
        "{out:?}"
    );
    // 更小的键同样拒绝
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fixed_dedup - 1,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(RejectReason::DuplicateCommand)
        ),
        "{out:?}"
    );
    // 键 0 恒拒绝
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 0,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(RejectReason::DuplicateCommand)
        ),
        "{out:?}"
    );
}

/// 应答校验的重复守卫：同一选项出现两次 → 拒绝（防下游二次移除同一实例）。
#[test]
fn duplicate_selection_rejected_by_spec() {
    let spec = ChoiceSpec {
        kind: engine_native::ChoiceKind::SelectCards,
        prompt: "unit",
        options: vec![ChoiceOption::Card(1), ChoiceOption::Card(2)],
        min: 2,
        max: 2,
        allow_cancel: false,
    };
    assert!(spec.accepts(&[ChoiceOption::Card(1), ChoiceOption::Card(2)]));
    assert!(!spec.accepts(&[ChoiceOption::Card(1), ChoiceOption::Card(1)]));
}

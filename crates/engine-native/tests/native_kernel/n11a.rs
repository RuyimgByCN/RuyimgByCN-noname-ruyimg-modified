use super::*;

/// P1/N11-a（ADR 0768）：会话令牌绑定实例熵与恢复代次——
/// 同种子跨实例令牌互不接受；恢复前旧令牌失效；正确重签仍可用。
#[test]
fn n11a_auth_token_instance_and_generation_binding() {
    // 跨实例：同种子、同签发顺序 → 令牌不同，A 的令牌被 B 拒绝
    let mut a = Engine::new(443, 4);
    let token_a = a.issue_token(0);
    let mut b = Engine::new(443, 4);
    let token_b = b.issue_token(0);
    assert_ne!(token_a, token_b, "同种子跨实例令牌必须不同（实例熵隔离）");
    let cmd = GameCommand {
        actor: 0,
        expected_revision: b.revision(),
        dedup_id: 1,
        action: GameAction::NextPhase,
    };
    assert!(
        matches!(
            b.submit_authed(token_a, cmd.clone()),
            CommandOutcome::Rejected(RejectReason::NoSuchSession)
        ),
        "A 的令牌不得被 B 接受"
    );
    assert!(
        matches!(
            b.submit_authed(token_b, cmd.clone()),
            CommandOutcome::Accepted { .. }
        ) || matches!(
            b.submit_authed(token_b, cmd),
            CommandOutcome::Pending { .. }
        ),
        "B 自己的令牌应可用"
    );

    // 恢复代次：恢复前旧令牌失效，恢复后重签有效
    let snap = a.save();
    let mut restored = Engine::restore(&snap).expect("restore");
    let old_cmd = GameCommand {
        actor: 0,
        expected_revision: restored.revision(),
        dedup_id: 1,
        action: GameAction::NextPhase,
    };
    assert!(
        matches!(
            restored.submit_authed(token_a, old_cmd),
            CommandOutcome::Rejected(RejectReason::NoSuchSession)
        ),
        "恢复前旧令牌必须失效（代次轮换）"
    );
    let reissued = restored.issue_token(0);
    assert_ne!(token_a, reissued, "恢复后重签必须产生新令牌");
    let new_cmd = GameCommand {
        actor: 0,
        expected_revision: restored.revision(),
        dedup_id: 2,
        action: GameAction::NextPhase,
    };
    assert!(
        matches!(
            restored.submit_authed(reissued, new_cmd),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "正确重签的令牌应可用"
    );
}

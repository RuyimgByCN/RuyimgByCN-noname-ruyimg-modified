use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：吕蒙（克己）（ADR 0750）
// ---------------------------------------------------------------------------

/// 克己：本回合未使用杀 → 结束阶段无需弃牌（手牌上限视为无限）。
#[test]
fn lvmeng_keji_skips_discard_without_sha() {
    let mut e = Engine::new(591, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "克己");
    // 手牌塞满（超过上限）且未出杀：End 阶段不应触发弃牌
    for _ in 0..6 {
        e.debug_place(0, "shan");
    }
    let before = e.debug_hand(0).len();
    loop {
        if e.turn().current == 0 && e.turn().phase == Phase::End {
            break;
        }
        assert!(e.open_choice().is_none(), "克己下不应有弃牌询问");
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.debug_hand(0).len(), before, "克己：未出杀回合手牌保留");
    assert!(e.open_choice().is_none(), "不应有弃牌询问");
    // 对照：本回合若使用过杀，克己不生效（正常弃牌逻辑接管）
    // ——该路径由 discard_to_limit golden 与既有用例覆盖。
}

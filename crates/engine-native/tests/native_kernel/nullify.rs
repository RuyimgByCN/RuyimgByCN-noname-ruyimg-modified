use super::*;

// ---------------------------------------------------------------------------
// 无懈可击响应链（阶段 2 扩面第五项）
// ---------------------------------------------------------------------------

/// 无懈抵消南蛮（逐目标语义，ADR 0705）：1 出无懈只抵消对目标 1 的结算，
/// 自其下家重新询问（无懈者跳过）；目标 2、3 照常结算受伤。
#[test]
fn nullify_cancels_savage() {
    let mut e = Engine::new(151, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    assert!(matches!(
        play(&mut e, savage, vec![]),
        CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    // 目标 1 窗口：1 出无懈 → 抵消；2/3/0 无无懈 → 窗口关闭
    let out = answer(&mut e, ChoiceOption::Card(nul1));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 逐目标抵消：1 免伤；2、3 无杀照常受伤；南蛮牌弃置
    assert_eq!(hp(&e, 1), 4);
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));
    assert!(!e.debug_hand(1).contains(&nul1));
}

/// 反无懈：第二张无懈反转回「未抵消」，南蛮正常生效。
#[test]
fn nullify_on_nullify_reverses() {
    let mut e = Engine::new(153, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    let nul2 = e.debug_place(2, "nullify");
    let sha1 = e.debug_place(1, "sha");
    play(&mut e, savage, vec![]);
    // 1 出无懈（抵消）→ 自 2 起重新询问
    let out = answer(&mut e, ChoiceOption::Card(nul1));
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.open_choice().unwrap().observer, 2);
    // 2 反无懈（未抵消）→ 3/0 无无懈 → 生效
    let out = answer(&mut e, ChoiceOption::Card(nul2));
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "aoe_response");
    assert_eq!(
        e.open_choice().unwrap().observer,
        1,
        "南蛮生效，先询问目标 1"
    );
    let out = answer(&mut e, ChoiceOption::Card(sha1));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 1 出杀免伤；2、3 无杀 → 跳过受伤
    assert_eq!(hp(&e, 1), 4);
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
    assert!(!log_has(&e, |l| matches!(l, LE::CardNullified { .. })));
}

/// 无懈可击只能在无懈窗口响应：出牌阶段主动使用（无目标/有目标）
/// 以 WrongPhase 拒绝且状态不变；恢复件同样拒绝。
#[test]
fn nullify_rejected_when_played_actively() {
    let mut e = Engine::new(7, 4);
    advance_to_play(&mut e, 0);
    let nul = e.debug_place(0, "nullify");
    let before = e.save();
    for targets in [vec![], vec![1]] {
        let out = play(&mut e, nul, targets.clone());
        assert!(
            matches!(out, CommandOutcome::Rejected(RejectReason::WrongPhase)),
            "主动使用无懈（targets={targets:?}）必须被拒: {out:?}"
        );
        assert_eq!(e.save(), before, "被拒命令不得改变任何状态");
    }
    let mut r = Engine::restore(&before).unwrap();
    let out = play(&mut r, nul, vec![]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::WrongPhase)),
        "{out:?}"
    );
}

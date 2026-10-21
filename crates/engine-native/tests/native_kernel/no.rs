use super::*;

/// 无懈不适用基础牌：杀的响应窗口前没有无懈询问。
#[test]
fn no_nullify_window_for_basic_card() {
    let mut e = Engine::new(155, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_place(1, "nullify"); // 1 有无懈，但杀不开窗口
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "respond_shan", "直接进入闪响应，无无懈窗口");
    let out = answer(&mut e, ChoiceOption::Card(shan));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
}

/// 对照：无咆哮时第二杀 → ShaLimit（既有语义回归）。
#[test]
fn no_paoxiao_second_sha_rejected() {
    let mut e = Engine::new(619, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let s1 = e.debug_place(0, "sha");
    let s2 = e.debug_place(0, "sha");
    let out = play(&mut e, s1, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    let out = play(&mut e, s2, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "第二杀应被 ShaLimit 拒绝: {out:?}"
    );
}

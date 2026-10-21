use super::*;

// ---------------------------------------------------------------------------
// 结算中途失效（肃清 vs 碎盾）与每回合限一次
// ---------------------------------------------------------------------------

#[test]
fn suqing_disables_queued_suidun_mid_resolution() {
    // 肃清（First）先于碎盾（Normal）入队；运行前重查发现已被禁用
    let mut e = Engine::new(51, 4);
    e.debug_grant(0, "碎盾");
    e.debug_grant(2, "肃清");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    assert_eq!(hp(&e, 0), 3, "肃清应使已入队的碎盾失效");
    assert!(!e.debug_has_skill(0, "碎盾"));
    assert!(!log_has(&e, |l| matches!(
        l,
        LogEvent::DamagePrevented { .. }
    )));
}

use super::*;

// ---------------------------------------------------------------------------
// 伤害转移（移伤）：前置窗口内改写帧载荷
// ---------------------------------------------------------------------------

#[test]
fn yishang_redirects_damage_in_before_window() {
    let mut e = Engine::new(31, 4);
    e.debug_grant(0, "移伤");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0); // 推进后清空，保证无闪
    play(&mut e, sha, vec![0]); // 0 无闪
    assert_eq!(prompt_of(&e), "trigger_confirm"); // 可选触发确认
    answer(&mut e, ChoiceOption::Confirm);
    assert_eq!(prompt_of(&e), "yishang_target");
    assert!(matches!(
        answer(&mut e, ChoiceOption::Player(2)),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 0), 4, "原目标不再受伤");
    assert_eq!(hp(&e, 2), 3);
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::DamageDealt {
            target: 2,
            redirected_from,
            ..
        } if *redirected_from == vec![0]
    )));
}

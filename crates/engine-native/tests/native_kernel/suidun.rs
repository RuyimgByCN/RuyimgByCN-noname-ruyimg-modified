use super::*;

#[test]
fn suidun_prevents_once_per_turn() {
    let mut e = Engine::new(53, 4);
    e.debug_grant(0, "碎盾");
    let sha_b = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha_b, vec![0]);
    assert_eq!(hp(&e, 0), 4, "碎盾防住第一段伤害");
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::DamagePrevented { target: 0 }
    )));
    // 同回合不能再防：换 3（与 0 相邻）攻击
    advance_to_play(&mut e, 3);
    let sha_d = e.debug_place(3, "sha");
    e.debug_clear_hand(0);
    play(&mut e, sha_d, vec![0]);
    assert_eq!(hp(&e, 0), 3, "每回合限一次，第二段伤害应生效");
}

use super::*;

/// 集智：使用普通锦囊（南蛮入侵）时摸一张牌。
#[test]
fn huangyueying_jizhi_draws_on_trick() {
    let mut e = Engine::new(521, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "集智");
    let savage = e.debug_place(0, "savage");
    let out = play(&mut e, savage, vec![]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 普通锦囊离手 + 集智摸一 → 手牌恰 1
    assert_eq!(e.debug_hand(0).len(), 1, "集智应在使用普通锦囊后摸一张");
    assert_zone_conservation(&e);
}

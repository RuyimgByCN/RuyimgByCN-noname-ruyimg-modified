use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：甘宁（奇袭）——过河拆桥新牌种（ADR 0752）
// ---------------------------------------------------------------------------

/// 奇袭：黑色手牌当过河拆桥——弃置目标一张手牌。
#[test]
fn ganning_qixi_black_card_as_dismantle() {
    let mut e = Engine::new(601, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "奇袭");
    // 转化素材须为黑色非拆牌；v74 对齐后闪全红（上游口径），改用黑雷杀
    let black_card = e.debug_move_named_card_to_hand(0, "leisha");
    let target_card = e.debug_place(1, "shan"); // 1 号持一张手牌（将被拆桥弃置）
    let before1 = e.debug_hand(1).len();
    assert_eq!(before1, 1);
    let out = play(&mut e, black_card, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_hand(1).len(), 0, "拆桥应弃置目标手牌");
    assert_eq!(discard_count(&e, black_card), 1, "拆桥牌照常弃置");
    assert_eq!(discard_count(&e, target_card), 1, "被拆的目标牌应弃置");
    assert!(e.state_core().table.is_empty(), "桌面不应残留");
    assert_zone_conservation(&e);
}

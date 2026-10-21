use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：关羽（武圣）/ 赵云（龙胆）——view-as 转化语义（ADR 0740）
// ---------------------------------------------------------------------------

/// 武圣：红色手牌当杀使用——路由/伤害/弃置全部按杀结算。
#[test]
fn guanyu_wusheng_red_card_as_sha() {
    let mut e = Engine::new(491, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "武圣");
    // 红色桃当杀（武圣不限牌种，红色即可）
    let red_card = loop {
        let c = e.debug_place(0, "tao");
        if e.catalog().cards[c as usize].suit == Suit::Heart {
            break c;
        }
    };
    let out = play(&mut e, red_card, vec![1]); // 1 无闪 → 受伤
    println!(
        "DEBUG out={out:?} hp1={} discard={:?} table={:?} hand0={:?}",
        hp(&e, 1),
        e.state_core().discard,
        e.state_core().table,
        e.debug_hand(0)
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 1), 3, "红色桃当杀应造成 1 点伤害");
    assert!(log_has(&e, |l| matches!(l,
        LE::DamageDealt { target: 1, amount: 1, card: Some(c), .. } if *c == red_card)));
    assert_eq!(discard_count(&e, red_card), 1, "转化使用的牌照常弃置");
    assert_zone_conservation(&e);
}

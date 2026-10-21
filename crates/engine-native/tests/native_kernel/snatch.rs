use super::*;

/// 顺手牵羊：距离 1 内获得目标一张牌（手牌优先、其次装备）；
/// 谦逊目标拒绝；距离 > 1 拒绝。
#[test]
fn snatch_steals_card_within_distance() {
    // (1) 获得手牌首张
    let mut e = Engine::new(983, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let victim = e.debug_place(1, "tao");
    let snatch = e.debug_place(0, "snatch");
    play(&mut e, snatch, vec![1]);
    assert!(e.debug_hand(0).contains(&victim), "顺手牵羊应获得目标手牌");
    assert!(!e.debug_hand(1).contains(&victim));
    // (2) 装备区：无手牌时取装备首张
    e.debug_clear_hand(1);
    let equip = e.debug_place(1, "horse_plus");
    let snatch2 = e.debug_place(0, "snatch");
    play(&mut e, snatch2, vec![1]);
    assert!(
        e.debug_equips(1).is_empty() && e.debug_hand(0).contains(&equip),
        "无手牌时应取走装备"
    );
    assert_zone_conservation(&e);

    // (3) 谦逊目标拒绝
    let mut e2 = Engine::new(984, 3);
    advance_to_play(&mut e2, 0);
    e2.debug_grant(1, "谦逊");
    let snatch3 = e2.debug_place(0, "snatch");
    assert!(
        matches!(
            play(&mut e2, snatch3, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "谦逊不得成为顺手牵羊目标"
    );
    // (4) 距离 > 1 拒绝（4 人局隔座）
    let mut e3 = Engine::new(985, 4);
    advance_to_play(&mut e3, 0);
    let snatch4 = e3.debug_place(0, "snatch");
    assert!(
        matches!(
            play(&mut e3, snatch4, vec![2]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "距离 2 不得发动顺手牵羊"
    );
}

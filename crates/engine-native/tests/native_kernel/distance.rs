use super::*;

// ---------------------------------------------------------------------------
// N06 差分用例：距离/传导/AOE 冻结/决斗来源——锁定当前原生行为
// ---------------------------------------------------------------------------

/// 距离下限 1：即使有 -1 马，距离也不得低于 1。
#[test]
fn distance_clamp_to_minimum_1() {
    let mut e = Engine::new(301, 2);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    // 双方各出 -1 马：0↔1 距离应为 max(1, 2-1-1) = 1（不小于 1）
    let _hm0 = e.debug_place(0, "horse_minus");
    play(&mut e, _hm0, vec![]);
    let _hm1 = e.debug_place(1, "horse_minus");
    play(&mut e, _hm1, vec![]);
    // 出杀 0→1：距离 1 = 射程 1 → 合法
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 1), 3);
    assert_zone_conservation(&e);
}

/// N06 距离/传导差分：4–8 人身份局 × 无装备/全 +1 马/全 -1 马/死座压缩四阶段，
/// 引擎 computed_distance/attack_range/next_alive/seat_order_from 与独立参照模型全对全一致。
#[test]
fn distance_range_reference_differential() {
    for seed in [17u64, 53] {
        for seats in [4u8, 5, 6, 7, 8] {
            let mut e = Engine::try_new_identity(seed * 100 + seats as u64, seats as usize)
                .expect("identity build");
            assert_distance_range_reference(&e, "无装备");
            equip_horse_for_all(&mut e, seats, "horse_plus");
            assert_distance_range_reference(&e, "+1 马");
            equip_horse_for_all(&mut e, seats, "horse_minus");
            assert_distance_range_reference(&e, "±1 马");
            // 死座压缩：忠臣动手除掉内奸（无奖惩、不终局），存活环出现空洞
            for p in 0..seats {
                e.debug_clear_hand(p);
            }
            let loyal = seats_with_role(&e, Role::Loyalist)[0];
            let traitor = seats_with_role(&e, Role::Traitor)[0];
            e.debug_set_hp(traitor, 1);
            let out = e.debug_deal_damage(Some(loyal), traitor, 1, DamageNature::Normal);
            assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
            assert!(!e.debug_alive(traitor));
            assert!(e.outcome().is_none(), "仅内奸死亡不终局");
            assert_distance_range_reference(&e, "死座压缩");
        }
    }
}

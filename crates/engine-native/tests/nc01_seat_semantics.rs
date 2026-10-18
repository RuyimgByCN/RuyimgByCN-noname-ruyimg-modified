//! NC01 座位语义回归：最短距离 / 顺座次序 / 存活座位环 三个概念分开验收。
//!
//! 审计反例（2026-10-18 架构审查，`seat-distance-probe.log`）：
//! - 5 人局 25 对中 14 对不符合环状最短距离，例如 `d(0,1)=0`（预期 1）；
//! - 6 人局 36 对中 12 对不符合，例如 `d(0,3)=1`（预期 3）；
//! - 8 人局全部吻合（说明只测 8 人局不足以发现算术错误）；
//! - 4 人局按最短距离排序得到 `[0,1,3,2]`，与顺座次序 `[0,1,2,3]` 不同。
//!
//! 本文件按人数逐一验收：全部座位对、旋转不变性、对称性、非同座距离非零、
//! 顺座次序与最短距离分离、存活环（死亡玩家不占位）。

use engine_native::Engine;

fn engine(seats: usize) -> Engine {
    Engine::new(20261018, seats)
}

/// 保守应答所有开放选择（用于让濒死结算走完）。
fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
        let _ = e.submit(engine_native::GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: 20261018 + c.id as u64,
            action: engine_native::GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 最短距离：逐对枚举，验证对称、非负、同座 0、非同座 ≥1、上界 floor(n/2)。
#[test]
fn seat_distance_all_pairs_for_supported_counts() {
    for seats in 2..=8usize {
        let e = engine(seats);
        let n = seats as u8;
        for a in 0..n {
            for b in 0..n {
                let d = e.state_core().seat_distance(a, b);
                let expect = {
                    let cw = (b as i16 - a as i16).rem_euclid(n as i16) as u8;
                    let ccw = (a as i16 - b as i16).rem_euclid(n as i16) as u8;
                    cw.min(ccw)
                };
                assert_eq!(d, expect, "{seats} 人局 d({a},{b})");
                assert_eq!(
                    d,
                    e.state_core().seat_distance(b, a),
                    "{seats} 人局 d({a},{b}) 对称"
                );
                if a == b {
                    assert_eq!(d, 0, "{seats} 人局 d({a},{a})=0");
                } else {
                    assert!(d >= 1, "{seats} 人局 d({a},{b}) 非同座须 ≥1");
                }
                assert!(
                    d <= n / 2,
                    "{seats} 人局 d({a},{b}) ≤ floor(n/2)={}",
                    n / 2
                );
            }
        }
    }
}

/// 审计反例逐条复现（修复后应通过）。
#[test]
fn audit_counterexamples_are_fixed() {
    let five = engine(5);
    assert_eq!(five.state_core().seat_distance(0, 1), 1, "5 人局 d(0,1) 曾为 0");
    let six = engine(6);
    assert_eq!(six.state_core().seat_distance(0, 3), 3, "6 人局 d(0,3) 曾为 1");
    let eight = engine(8);
    for a in 0..8u8 {
        for b in 0..8u8 {
            let cw = (b as i16 - a as i16).rem_euclid(8) as u8;
            let ccw = (a as i16 - b as i16).rem_euclid(8) as u8;
            assert_eq!(eight.state_core().seat_distance(a, b), cw.min(ccw));
        }
    }
}

/// 旋转不变性：整体旋转座位后，距离不变。
#[test]
fn seat_distance_is_rotation_invariant() {
    for seats in 3..=8usize {
        let e = engine(seats);
        let n = seats as u8;
        for shift in 0..n {
            for a in 0..n {
                for b in 0..n {
                    let ra = (a + shift) % n;
                    let rb = (b + shift) % n;
                    assert_eq!(
                        e.state_core().seat_distance(a, b),
                        e.state_core().seat_distance(ra, rb),
                        "{seats} 人局旋转 {shift}：d({a},{b}) == d({ra},{rb})"
                    );
                }
            }
        }
    }
}

/// 顺座次序与最短距离必须分离（审计 4 人局反例）。
#[test]
fn seat_steps_and_distance_are_distinct_concepts() {
    let e = engine(4);
    // 顺座次序：从 0 起 [0,1,2,3]
    let order: Vec<u8> = (0..4u8).map(|p| e.state_core().seat_steps_from(0, p)).collect();
    assert_eq!(order, vec![0, 1, 2, 3], "顺座次序");
    // 最短距离：从 0 起 [0,1,2,1]
    let dist: Vec<u8> = (0..4u8).map(|p| e.state_core().seat_distance(0, p)).collect();
    assert_eq!(dist, vec![0, 1, 2, 1], "最短距离");
    // 按距离排序得到 [0,1,3,2]，与顺座次序不同
    let mut by_dist: Vec<u8> = (0..4u8).collect();
    by_dist.sort_by_key(|p| e.state_core().seat_distance(0, *p));
    assert_eq!(by_dist, vec![0, 1, 3, 2], "按最短距离排序");
    assert_ne!(by_dist, order, "两个概念不能共用同一原语");

    // 顺座次序：逐座位递增且互不相同；起点自身为 0
    for seats in 2..=8usize {
        let e = engine(seats);
        let n = seats as u8;
        let mut seen = vec![false; seats];
        for p in 0..n {
            let s = e.state_core().seat_steps_from(0, p);
            assert_eq!(s, p, "{seats} 人局从 0 起顺座次序 = 座位号");
            assert!(!seen[s as usize], "{seats} 人局顺座次序互异");
            seen[s as usize] = true;
        }
        assert_eq!(e.state_core().seat_steps_from(2 % n, 2 % n), 0, "起点自身为 0");
    }
}

/// 存活座位环：死亡玩家不占位；存活距离与全座位距离在无人死亡时一致。
#[test]
fn alive_ring_ignores_dead_seats() {
    let mut e = engine(5);
    // 无人死亡：存活距离 == 全座位最短距离
    for a in 0..5u8 {
        for b in 0..5u8 {
            assert_eq!(
                e.state_core().seat_distance_alive(a, b),
                e.state_core().seat_distance(a, b),
                "无人死亡时两种距离一致 d({a},{b})"
            );
        }
    }
    // 杀死座位 1（致命伤害）：存活环为 [0,2,3,4] → d_alive(0,2)=1、d_alive(0,4)=1
    e.debug_set_hp(1, 1);
    let out = e.debug_deal_damage(
        Some(0),
        1,
        9,
        engine_native::event::DamageNature::Normal,
    );
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_alive(1), "1 号已阵亡");
    assert_eq!(e.state_core().seat_distance_alive(0, 2), 1, "跳过死亡座位");
    assert_eq!(e.state_core().seat_distance_alive(0, 4), 1, "另一侧相邻");
    assert_eq!(e.state_core().seat_distance_alive(0, 3), 2, "跨过一名存活者");
    // 全座位距离不受死亡影响（语义不同：不按存活计位）
    assert_eq!(e.state_core().seat_distance(0, 2), 2, "全座位距离仍按座号");
}

/// 换位（座位环交换）后，顺座次序与存活环随之改变。
#[test]
fn seat_ring_swap_changes_order() {
    let mut e = engine(4);
    assert_eq!(e.state_core().seat_steps_from(0, 2), 2);
    e.state_core_mut().swap_seat(1, 3);
    // 环变为 [0,3,2,1]：从 0 起顺座次序 → 0,3,2,1
    assert_eq!(e.state_core().seat_steps_from(0, 3), 1, "换位后 3 在 0 之后");
    assert_eq!(e.state_core().seat_steps_from(0, 1), 3, "换位后 1 在末位");
    let ring_next = e.state_core().ring_next(0);
    assert_eq!(ring_next, 3, "环下一位随换位改变");
}

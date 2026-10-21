use super::*;

/// 突袭（张辽）：摸牌阶段确认 → 选 1–2 名有手牌者各取一张（弃置本阶段
/// 固定摸牌）；弃权 → 保留摸牌。询问挂起时存档→恢复续行等价。
#[test]
fn zhangliao_tuxi_replaces_draw() {
    let build = |seed: u64| -> Engine {
        let mut e = Engine::new(seed, 3);
        advance_to_play(&mut e, 0);
        e.debug_grant(0, "突袭");
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        advance_to_judge(&mut e, 0);
        e.debug_clear_hand(0);
        e
    };

    // (1) 取两名：弃 2 摸牌、各偷 1 张 → 手牌 0+2-2+2 = 2，目标各 -1。
    let mut e = build(804);
    next_phase(&mut e); // Draw begin：固定摸 2 + 触发闸门
    let c = e.open_choice().expect("突袭触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    answer(&mut e, ChoiceOption::Confirm);
    // 目标选择（可存档续行）
    let sel = e.open_choice().expect("tuxi_target 应挂起");
    assert_eq!(sel.spec.prompt, "tuxi_target");
    assert_eq!((sel.spec.min, sel.spec.max), (1, 2));
    let snap = e.save();
    let mut e2 = Engine::restore(&snap).expect("restore mid-ask");
    let targets = vec![ChoiceOption::Player(1), ChoiceOption::Player(2)];
    for eng in [&mut e, &mut e2] {
        answer_sel(eng, targets.clone());
        assert_eq!(eng.debug_hand(0).len(), 2, "弃 2 摸牌换偷 2 张");
        assert_eq!(eng.debug_hand(1).len(), 1, "目标 1 被取走一张");
        assert_eq!(eng.debug_hand(2).len(), 1, "目标 2 被取走一张");
        assert_zone_conservation(eng);
    }
    assert_eq!(e.state_hash(), e2.state_hash(), "存档续行终态一致");

    // (2) 弃权：保留本阶段摸牌（手牌 2），目标手牌不变。
    let mut e3 = build(805);
    next_phase(&mut e3);
    answer(&mut e3, ChoiceOption::Confirm);
    answer_empty(&mut e3); // tuxi_target 弃权
    assert_eq!(e3.debug_hand(0).len(), 2, "弃权应保留摸牌");
    assert_eq!(e3.debug_hand(1).len(), 2);
    assert_eq!(e3.debug_hand(2).len(), 2);
    assert_zone_conservation(&e3);
}

use super::*;

/// Boss 关卡骨架（规则 v207）：关卡开局配置——boss/挑战者分列发牌数
/// （上游 bossinfo.gameDraw：穷奇 boss 8/玩家 4）与回合起点座位（上游
/// bossinfo.loopFirst 以显式座位承载，越界回退 0）；标准 new_boss 回归
/// 不变（各 4 张、起点 0）。
#[test]
fn boss_stage_opening_and_first_actor() {
    // 关卡面：boss（1 号）8 张、挑战者 4 张、回合起点=2 号（boss 下下家）
    let e = Engine::new_boss_stage(24001, 4, 1, 3, 8, 4, 2);
    assert_eq!(e.debug_hand(1).len(), 8, "boss 关卡发牌数 8");
    assert_eq!(e.debug_hand(0).len(), 4, "挑战者发牌数 4");
    assert_eq!(e.debug_hand(2).len(), 4, "挑战者发牌数 4");
    assert_eq!(e.debug_hand(3).len(), 4, "挑战者发牌数 4");
    assert_eq!(e.turn().current, 2, "loopFirst 起点=指定座位");
    // 存档往返：回合起点与手牌一致
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(r.turn().current, 2, "回合起点往返一致");
    assert_eq!(r.debug_hand(1).len(), 8, "boss 手牌往返一致");
    // 起点越界回退 0
    let e2 = Engine::new_boss_stage(24002, 3, 2, 2, 6, 4, 9);
    assert_eq!(e2.turn().current, 0, "非法起点回退 0");
    assert_eq!(e2.debug_hand(2).len(), 6, "boss 发牌数 6");
    // 标准构造回归：各 4 张、起点 0
    let e3 = Engine::new_boss(24003, 4, 0, 3);
    assert_eq!(e3.turn().current, 0, "标准起点 0");
    for p in 0..4 {
        assert_eq!(e3.debug_hand(p).len(), 4, "标准各 4 张");
    }
}

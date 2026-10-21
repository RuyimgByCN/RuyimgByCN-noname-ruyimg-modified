use super::*;

/// 炎灵陪将片（规则 v241）：焰灵回火+复燃——①杀上限 +1（额度每回合
/// 一次：第二张杀可出、第三张被拒）②复燃：焰灵濒死时敌方红色牌入救援
/// 选项（黑色牌不入）③回火死亡反噬：焰灵死亡后其敌方全体各受 3 点无来
/// 源火伤。焰灵条目（群目录近似/hp4/[回火,复燃]；上游无神裔）。
#[test]
fn boss_yanling_huihuo_furan() {
    // ① 回火：杀限 +1（两连杀可出，第三张拒）
    let mut e = Engine::new_boss_stage(61001, 4, 1, 3, 6, 4, 0);
    let yanling_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 17) as u16; // BOSS_GENERALS[17] 焰灵
    e.debug_set_general_def(1, yanling_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(yanling_def).map(|g| g.0),
        Some("焰灵"),
        "焰灵入目录"
    );
    let hdef = e.catalog().skill_by_name("回火").expect("回火入目录");
    let fdef = e.catalog().skill_by_name("复燃").expect("复燃入目录");
    assert!(e.state_core().skills.contains_key(&(1, hdef)));
    assert!(e.state_core().skills.contains_key(&(1, fdef)));

    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let sha1 = e.debug_place(1, "sha");
    play(&mut e, sha1, vec![0]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 3, "第一杀命中");
    let sha2 = e.debug_place(1, "sha");
    play(&mut e, sha2, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 3, "回火：第二杀（额度）命中");
    let sha3 = e.debug_place(1, "sha");
    let out = play(&mut e, sha3, vec![3]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "回火额度用尽：第三杀应被拒"
    );

    // ② 复燃：0 号回合杀焰灵至濒死 → 敌方红牌入救援选项 → 放弃 → 死亡
    //    （boss 死亡=挑战者胜，反噬在 e3 fellow 线观察）
    next_phase(&mut e); // 1 Play→Discard（手牌已清）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let red = e.debug_place(0, "tao");
    let _black = e.debug_place(0, "sha");
    let sha4 = e.debug_place(0, "sha");
    play(&mut e, sha4, vec![1]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 3, "sha 命中（4-1）");
    e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    // 濒死链：焰灵先自问（空手仅 Pass）→ 0 号救援询问（红牌入选项）
    let c = e.open_choice().expect("濒死询问挂起");
    if c.observer == 1 {
        answer(&mut e, ChoiceOption::Pass);
        drain_choices(&mut e);
    }
    let c = e.open_choice().expect("救援询问挂起");
    assert_eq!(c.observer, 0, "敌方救援询问");
    assert!(
        c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == red)),
        "复燃：敌方红色牌可当桃救"
    );
    answer(&mut e, ChoiceOption::Pass);
    drain_choices(&mut e);
    assert!(!e.debug_alive(1), "焰灵死亡（放弃救援）");
    assert!(e.outcome().is_some(), "boss 死亡 → 挑战者胜终局");

    // ③ 回火死亡反噬：fellow 炎灵（座位 4，非 boss）死亡 → 敌方全体
    //    各受 3 点无来源火伤（boss 死亡即终局截停队列，故用 fellow 线）
    let mut e3 = Engine::new_boss_stage(61003, 4, 1, 3, 6, 4, 0);
    e3.debug_set_hp(1, 4);
    let fellow = e3.debug_add_fellow(4).expect("fellow 入队");
    e3.debug_set_general_def(fellow, yanling_def);
    assert!(
        e3.state_core().skills.contains_key(&(fellow, hdef)),
        "fellow 装配炎灵目录"
    );
    advance_to_play(&mut e3, 0);
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    e3.debug_clear_hand(2);
    e3.debug_clear_hand(3);
    e3.debug_clear_hand(fellow);
    let sha5 = e3.debug_place(0, "sha");
    play(&mut e3, sha5, vec![fellow]);
    drain_choices(&mut e3);
    assert_eq!(e3.debug_hp(fellow), 3, "sha 命中 fellow");
    e3.debug_deal_damage(Some(0), fellow, 3, DamageNature::Normal);
    // 濒死链全 Pass（0 号无红牌、救援也放弃）
    for _ in 0..4 {
        if e3.open_choice().is_some() {
            answer(&mut e3, ChoiceOption::Pass);
            drain_choices(&mut e3);
        } else {
            break;
        }
    }
    assert!(!e3.debug_alive(fellow), "fellow 炎灵死亡");
    assert!(e3.outcome().is_none(), "非 boss 死亡不终局");
    assert_eq!(e3.debug_hp(0), 1, "回火反噬：0 号 4-3");
    assert_eq!(e3.debug_hp(2), 1, "回火反噬：2 号 4-3");
    assert_eq!(e3.debug_hp(3), 1, "回火反噬：3 号 4-3");
    assert_eq!(e3.debug_hp(1), 4, "boss 方同侧不受反噬");
    let snap = e3.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e3);
}

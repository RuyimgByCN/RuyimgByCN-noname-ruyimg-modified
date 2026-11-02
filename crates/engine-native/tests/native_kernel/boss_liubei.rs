use super::*;

/// 枭雄：他人用杀→刘备获得副本（入手+1）；自己用牌不触发；子技：
/// 未用牌者结束阶段受 1（用过牌的回合不受）。
#[test]
fn boss_liubei_xiaoxiong_gain_and_damage() {
    let mut e = engine_native::Engine::new_boss_stage(82001, 4, 1, 3, 6, 4, 0);
    let lb_def = liubei_def_of(&e);
    e.debug_set_general_def(1, lb_def);
    e.debug_set_hp(1, 8);
    assert_eq!(
        e.catalog().general_meta(lb_def).map(|g| g.0),
        Some("刘备"),
        "刘备入目录"
    );
    let xx = e.catalog().skill_by_name("枭雄").expect("枭雄入目录");
    let zw = e.catalog().skill_by_name("章武").expect("章武入目录");
    for sk in [xx, zw] {
        assert!(e.state_core().skills.contains_key(&(1, sk)), "实例齐备");
    }

    // 0 号先手：advance_to_play(0) 无跨回合，全员 4 血
    advance_to_play(&mut e, 0);

    // ① 枭雄入手：0 号出杀打 3（相邻）→ 刘备获得杀副本（手牌+1）
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    let sha = e.debug_place(0, "sha");
    assert!(matches!(
        play(&mut e, sha, vec![3]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "枭雄：获得杀副本入手");
    assert_eq!(hp(&e, 3), 3, "杀 1");

    // ② 子技负例：0 号已用牌（出杀）→ 0 号 End 不受伤
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin（cards_used_play_phase>0）
    assert_eq!(hp(&e, 0), 4, "已用牌回合不受枭雄子技伤害");

    // ③ 子技正例：0 收束→1 号（刘备）整回合[自己 End 不触发]→2 号未用
    //    牌直接 End→受 1
    next_phase_settled(&mut e); // 0 号收束→1 号 Judge
    advance_to_play(&mut e, 2);
    assert_eq!(hp(&e, 1), 8, "刘备自己回合 End 不自伤");
    e.debug_clear_hand(2);
    next_phase_settled(&mut e); // 2 Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → 子技伤 2 号
    drain_choices(&mut e);
    assert_eq!(hp(&e, 2), 3, "枭雄子技：2 号未用牌 End 受 1");

    // ④ 自己用牌不触发（独立局）：刘备回合出杀→手牌只减不增
    let mut e2 = engine_native::Engine::new_boss_stage(82004, 4, 1, 3, 6, 4, 0);
    e2.debug_set_general_def(1, lb_def);
    e2.debug_set_hp(1, 8);
    advance_to_play(&mut e2, 1); // 跨 0 号整回合（其 End 子技伤 0 号-1）
    e2.debug_clear_hand(1);
    e2.debug_clear_hand(0);
    let sha2 = e2.debug_place(1, "sha");
    assert!(matches!(
        play(&mut e2, sha2, vec![0]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e2);
    assert_eq!(e2.debug_hand(1).len(), 0, "自己用牌不触发枭雄");
    assert_eq!(hp(&e2, 0), 2, "0 号子技先-1+刘备杀 1");
    assert_zone_conservation(&e2);

    // ⑤ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, xx)));
    assert_zone_conservation(&r);
}

/// 章武：持有者选付一张，来源选付等量；来源不足时受持有者归因的伤害。
#[test]
fn boss_liubei_zhangwu_discard_or_damage() {
    // 场景 A：双方有手牌——各弃 1，无反伤
    let mut e = engine_native::Engine::new_boss_stage(82002, 4, 1, 3, 6, 4, 0);
    let lb_def = liubei_def_of(&e);
    e.debug_set_general_def(1, lb_def);
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    let extra = e.debug_place(0, "shan");
    let _ = extra; // 0 号第二张牌=弃牌源
                   // 刘备弃牌源用桃（闪会开杀响应询问）
    let lb_card = e.debug_place(1, "tao");
    let _ = lb_card;
    let n0 = e.debug_hand(0).len();
    let n1 = e.debug_hand(1).len();
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        engine_native::CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "c331_zhangwu_holder");
    answer_sel(&mut e, vec![ChoiceOption::Card(lb_card)]);
    assert_eq!(prompt_of(&e), "c331_zhangwu_source");
    answer_sel(&mut e, vec![ChoiceOption::Card(extra)]);
    drain_choices(&mut e);
    assert_eq!(hp(&e, 1), 7, "伤害 1 扣血");
    assert_eq!(e.debug_hand(0).len(), n0 - 2, "0 号：杀+章武弃 1");
    // 1 号：章武弃 1（DamageEnd invocation 子树）先于枭雄入手 1（CardUsed
    // AfterTriggers）→ 相抵不变
    assert_eq!(e.debug_hand(1).len(), n1, "章武弃 1+枭雄入手 1 相抵");

    // 场景 B：来源无手牌——受 1 反伤（新局）
    let mut e2 = engine_native::Engine::new_boss_stage(82003, 4, 1, 3, 6, 4, 0);
    e2.debug_set_general_def(1, lb_def);
    e2.debug_set_hp(1, 8);
    advance_to_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(1);
    let sha2 = e2.debug_place(0, "sha");
    let paid = e2.debug_place(1, "tao");
    assert!(matches!(
        play(&mut e2, sha2, vec![1]),
        engine_native::CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e2), "c331_zhangwu_holder");
    answer_sel(&mut e2, vec![ChoiceOption::Card(paid)]);
    drain_choices(&mut e2);
    assert_eq!(hp(&e2, 1), 7, "伤害 1");
    assert_eq!(hp(&e2, 0), 3, "章武反伤 1（0 先手无子技前置伤）");
    assert_zone_conservation(&e2);

    // 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

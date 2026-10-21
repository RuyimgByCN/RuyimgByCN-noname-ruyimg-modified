use super::*;

/// 星宿阶段2系续片（规则 v239/存档 v55）：星囚（蓐收）——出牌阶段开
/// 始交替横置全部敌方：①首回合三挑战者全横置（boss 本体不横置）②次回
/// 合开关取反不解除（只链不解）③fellow 属 boss 方不横置 ④toggled 入存
/// 档往返。授 boss_jiding 面登记差异（武将未入库）。
#[test]
fn boss_jinshenrushou_xingqiu_link_toggle() {
    let mut e = Engine::new_boss_stage(59001, 4, 1, 3, 6, 4, 0);
    let rus_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 15) as u16; // BOSS_GENERALS[15] 蓐收
    e.debug_set_general_def(1, rus_def);
    e.debug_set_hp(1, 5);
    assert_eq!(
        e.catalog().general_meta(rus_def).map(|g| g.0),
        Some("蓐收"),
        "蓐收入目录"
    );
    let xdef = e.catalog().skill_by_name("星囚").expect("星囚入目录");
    let ydef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, xdef)));
    assert!(e.state_core().skills.contains_key(&(1, ydef)), "神裔共用");

    // ① 首个出牌阶段开始：三挑战者全横置、boss 本体不横置
    advance_to_play(&mut e, 1);
    assert!(e.state_core().player(0).linked, "0 号被横置");
    assert!(e.state_core().player(2).linked, "2 号被横置");
    assert!(e.state_core().player(3).linked, "3 号被横置");
    assert!(!e.state_core().player(1).linked, "boss 本体不横置");
    assert!(e.state_core().player(1).xingqiu_toggled, "开关置 true");

    // ② 次回合开关取反：只链不解，横置保持（清空手牌→弃牌阶段无询问）
    e.debug_clear_hand(1);
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 1);
    assert!(!e.state_core().player(1).xingqiu_toggled, "开关取反");
    assert!(e.state_core().player(0).linked, "只链不解：横置保持");

    // ③ 第三回合再开：已横置保持（清空手牌→弃牌阶段无询问）
    e.debug_clear_hand(1);
    next_phase(&mut e);
    drain_choices(&mut e);
    next_phase(&mut e);
    drain_choices(&mut e);
    advance_to_play(&mut e, 1);
    assert!(e.state_core().player(1).xingqiu_toggled, "开关再置 true");
    assert!(e.state_core().player(0).linked);

    // ④ 存档 v55 往返：toggled 与横置态持久
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(r.state_core().player(1).xingqiu_toggled, true, "开关入存档");
    assert!(r.state_core().player(0).linked, "横置态往返");
    assert!(
        r.state_core().skills.contains_key(&(1, xdef)),
        "星囚实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);

    // ⑤ fellow 属 boss 方：不横置（新局追加 fellow 座位 4 后再触发）
    let mut e2 = Engine::new_boss_stage(59002, 4, 1, 3, 6, 4, 0);
    e2.debug_set_general_def(1, rus_def);
    e2.debug_set_hp(1, 5);
    let fellow = e2.debug_add_fellow(4).expect("fellow 入队");
    assert_eq!(fellow, 4, "fellow 追加为座位 4");
    advance_to_play(&mut e2, 1);
    assert!(
        !e2.state_core().player(4).linked,
        "fellow 属 boss 方：不横置"
    );
    assert!(e2.state_core().player(0).linked, "挑战者照常横置");
    assert!(e2.state_core().player(2).linked, "挑战者照常横置");
    assert!(e2.state_core().player(3).linked, "挑战者照常横置");
    assert_zone_conservation(&e2);
}

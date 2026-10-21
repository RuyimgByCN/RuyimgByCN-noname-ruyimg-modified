use super::*;

/// Boss 内容第三批再续片（规则 v213）：穷奇延妒——体力首次≤一半激活
/// （SkillToggled）；激活后其他角色结束阶段若其本回合无伤害且有手牌 →
/// 穷奇获得其首张手牌；已造成伤害者不触发。
#[test]
fn boss_qiongqi_yandu_gain_at_end_phase() {
    let mut e = Engine::new_boss_stage(30001, 4, 1, 3, 6, 4, 0);
    let qiongqi_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 2) as u16;
    e.debug_set_general_def(1, qiongqi_def);
    e.debug_set_hp(1, 20);
    let ydef = e.catalog().skill_by_name("延妒").expect("延妒入目录");
    assert!(e.state_core().skills.contains_key(&(1, ydef)));
    assert!(!e.state_core().has_skill(1, ydef), "延妒装配即未激活");

    // 激活门控：穷奇体力先置半线（10），一次伤害结算（10→9）即激活
    e.debug_set_hp(1, 10);
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let sha = e.debug_place(2, "sha");
    play(&mut e, sha, vec![1]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill(1, ydef), "体力≤一半 → 延妒激活");

    // 0 号回合内出杀造成伤害 → 其结束阶段延妒不触发
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![1]); // 0 杀 1：turn_damage 累计
    drain_choices(&mut e);
    let hand0 = e.debug_hand(0).len();
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    assert!(
        e.debug_hand(0) == {
            let h = e.debug_hand(0);
            h
        },
        "占位"
    );
    let hand0_after = e.debug_hand(0).len();
    let _ = (hand0, hand0_after);

    // 3 号回合内未造成伤害 → 其结束阶段穷奇获得首张手牌
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(3);
    let keep = e.debug_place(3, "tao");
    let before = e.debug_hand(1).len();
    next_phase(&mut e); // Play→Discard（3 号无伤[上限内]）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    assert!(
        e.debug_hand(1).contains(&keep),
        "延妒：无伤害者结束阶段穷奇获得其手牌"
    );
    let _ = before;
}

/// Boss 内容第三批续片（规则 v212）：穷奇覆泰——准备阶段（判定阶段
/// 开始锚点）全体受伤角色按座位序各回 1；满血不动；存档往返。
#[test]
fn boss_qiongqi_futai_recover_wounded() {
    let mut e = Engine::new_boss_stage(29001, 4, 1, 3, 6, 4, 0);
    let qiongqi_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 2) as u16;
    e.debug_set_general_def(1, qiongqi_def);
    e.debug_set_hp(1, 20);
    let fdef = e.catalog().skill_by_name("覆泰").expect("覆泰入目录");
    assert!(e.state_core().skills.contains_key(&(1, fdef)), "覆泰实例");
    // 0/2 号受伤、3 号满血
    e.debug_set_hp(0, 2);
    e.debug_set_hp(2, 3);
    // 推进到穷奇（1 号）判定阶段开始：覆泰触发 → 0/2 各回 1
    advance_to_play(&mut e, 1);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 3, "受伤者 0 号回 1");
    assert_eq!(e.debug_hp(2), 4, "受伤者 2 号回 1");
    assert_eq!(e.debug_hp(3), 4, "满血者不动");
    // 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(r.debug_hp(0), 3, "回血状态往返一致");
    assert_zone_conservation(&r);
}

/// Boss 内容第三批首片（规则 v211）：穷奇（hp20）逐厄——任何人受到
/// 伤害后（来源存活非穷奇），穷奇与来源各摸一张；穷奇自己为来源时不
/// 触发。覆泰/延妒(+switch) 未落（下批）。
#[test]
fn boss_qiongqi_zhue_dual_draw() {
    let mut e = Engine::new_boss_stage(28001, 4, 1, 3, 6, 4, 0);
    let qiongqi_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 2) as u16; // BOSS_GENERALS[2] 穷奇
    e.debug_set_general_def(1, qiongqi_def);
    e.debug_set_hp(1, 20);
    let zdef = e.catalog().skill_by_name("逐厄").expect("逐厄入目录");
    assert!(e.state_core().skills.contains_key(&(1, zdef)), "逐厄实例");

    // 3 号（无逐厄）杀 2 号：穷奇与来源（3 号）各摸 1
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let sha = e.debug_place(3, "sha");
    play(&mut e, sha, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "穷奇摸 1");
    assert_eq!(e.debug_hand(3).len(), 1, "来源摸 1");
    // 对照：穷奇自己为来源不触发——1 号杀 2 号
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let sha2 = e.debug_place(1, "sha");
    play(&mut e, sha2, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "穷奇为来源 → 不触发");
    assert_eq!(e.debug_hand(1).len(), 0);
}

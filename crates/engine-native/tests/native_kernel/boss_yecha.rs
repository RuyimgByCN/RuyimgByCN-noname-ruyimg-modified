use super::*;

/// 年兽系夜叉片（规则 v253）：魔道（准备摸 2）+魔剑（出牌阶段虚拟万箭
/// 打全部敌方）+胆术复用——①魔道摸 2 ②魔剑万箭（敌方无闪各受 1、本体
/// 不受）③胆术复用（夜叉本体失去手牌后非当前回合判定红回 1）。夜叉条
/// 目（[魔道,魔剑,胆术]，hp11；bazhen=标准装备八卦阵由 Armor 机承载）。
#[test]
fn boss_yecha_modao_mojian_danshu() {
    let mut e = Engine::new_boss_stage(73001, 4, 1, 3, 6, 4, 0);
    let yc_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 30) as u16; // BOSS_GENERALS[30] 夜叉
    e.debug_set_general_def(1, yc_def);
    e.debug_set_hp(1, 11);
    assert_eq!(
        e.catalog().general_meta(yc_def).map(|g| g.0),
        Some("夜叉"),
        "夜叉入目录"
    );
    let md = e.catalog().skill_by_name("魔道").expect("魔道入目录");
    let mj = e.catalog().skill_by_name("魔剑").expect("魔剑入目录");
    let ds = e.catalog().skill_by_name("胆术").expect("胆术入目录");
    assert!(e.state_core().skills.contains_key(&(1, md)));
    assert!(e.state_core().skills.contains_key(&(1, mj)));
    assert!(e.state_core().skills.contains_key(&(1, ds)));

    // 清空全场手牌后推进：0 号先手（摸 2）→ 1 号判定锚（魔道 +2）→
    // 摸牌（+2）→ 出牌锚（魔剑虚拟万箭）
    for t in [0u8, 1, 2, 3] {
        e.debug_clear_hand(t);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    advance_to_play(&mut e, 1);
    // ① 魔道：1 号手牌 = 魔道 2 + 摸牌 2
    assert_eq!(e.debug_hand(1).len(), 4, "魔道：准备阶段摸 2（+摸牌 2）");

    // ② 魔剑：Play begin 虚拟万箭 → 敌方空手（无牌不询问直接受伤——
    //    AOE 跳过语义挂账清单口径）
    assert_eq!(e.debug_hp(0), 3, "魔剑：0 号受 1（万箭）");
    assert_eq!(e.debug_hp(2), 3, "魔剑：2 号受 1");
    assert_eq!(e.debug_hp(3), 3, "魔剑：3 号受 1");
    assert_eq!(e.debug_hp(1), 11, "魔剑：本体不受");
    assert_zone_conservation(&e);

    // ③ 胆术复用（夜叉本体）：掉血后 0 号回合拆其手牌 → 判定红回 1
    e.debug_set_hp(1, 10);
    e.debug_clear_hand(1);
    next_phase(&mut e); // 1 Play→Discard（无牌不弃）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let _shan = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs = e.debug_place(0, "dismantle");
    play(&mut e, gs, vec![1]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 11, "胆术复用：判定红回 1（10→11）");

    // ④ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, md)), "魔道实例往返");
    assert!(r.state_core().skills.contains_key(&(1, mj)), "魔剑实例往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}

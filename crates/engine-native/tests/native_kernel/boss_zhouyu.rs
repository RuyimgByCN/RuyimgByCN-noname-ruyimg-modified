use super::*;

/// 火神：火属性伤害对周瑜无效+回复 1；对照：普通伤害照常。
#[test]
fn boss_zhouyu_huoshen_fire_immune_recover() {
    let mut e = engine_native::Engine::new_boss_stage(84001, 4, 1, 3, 6, 4, 0);
    let zy_def = zhouyu_def_of(&e);
    e.debug_set_general_def(1, zy_def);
    e.debug_set_hp(1, 6);
    assert_eq!(
        e.catalog().general_meta(zy_def).map(|g| g.0),
        Some("周瑜"),
        "周瑜入目录"
    );
    let hs = e.catalog().skill_by_name("火神").expect("火神入目录");
    let hl = e
        .catalog()
        .skill_by_name("红莲·周瑜")
        .expect("红莲·周瑜入目录");
    let xy = e.catalog().skill_by_name("仙音").expect("仙音入目录");
    for sk in [hs, hl, xy] {
        assert!(e.state_core().skills.contains_key(&(1, sk)), "实例齐备");
    }

    // ① 火伤免疫+回 1：周瑜 6→压伤后 4，火伤 2 → 取消（不扣血）+回 1=5
    e.debug_set_hp(1, 4);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Fire);
    drain_choices(&mut e);
    assert_eq!(hp(&e, 1), 5, "火神：火伤取消+回复 1");

    // ② 对照：普通伤害照常扣血
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(hp(&e, 1), 4, "普通伤害照常");

    // ③ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, hs)));
    assert_zone_conservation(&r);
}

/// 红莲·周瑜：End 摸 2+全部其他存活各 1 火伤；仙音：非己回合被拆牌
/// （CardsDiscarded）判定红→座位序首个其他存活失 1，判定黑不触发。
#[test]
fn boss_zhouyu_honglian_end_and_xianyin() {
    let mut e = engine_native::Engine::new_boss_stage(84002, 4, 1, 3, 6, 4, 0);
    let zy_def = zhouyu_def_of(&e);
    e.debug_set_general_def(1, zy_def);
    e.debug_set_hp(1, 6);
    advance_to_play(&mut e, 1);

    // ① 红莲：周瑜 End → 摸 2+其他存活各 1 火伤
    e.debug_clear_hand(1);
    let n0 = hp(&e, 0);
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → 红莲
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "红莲：摸 2");
    assert_eq!(hp(&e, 0), n0 - 1, "红莲：0 号火伤 1");
    assert_eq!(hp(&e, 2), 3, "红莲：2 号火伤 1");
    assert_eq!(hp(&e, 3), 3, "红莲：3 号火伤 1");

    // ② 仙音负例：非周瑜回合被拆但判定黑 → 不触发失血
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let zy_card = e.debug_place(1, "tao"); // 周瑜手牌=被拆对象
    let dc = e.debug_place(0, "dismantle");
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind != CardKind::Sha); // 判定预置黑
    assert!(matches!(
        play(&mut e, dc, vec![1]),
        engine_native::CommandOutcome::Accepted { .. }
            | engine_native::CommandOutcome::Pending { .. }
    ));
    pass_nullify_windows(&mut e);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "拆牌生效");
    let hp0_after_black = hp(&e, 0);
    assert_eq!(hp0_after_black, 2, "仙音负例：判定黑（预置黑桃）不触发失血");

    // ③ 仙音正例：再拆一次，判定红（牌堆尾预置红桃）→ 首个其他存活（0 号）失 1
    let zy_card2 = e.debug_place(1, "tao");
    let dc2 = e.debug_place(0, "dismantle");
    e.debug_set_deck_top_by(|def| def.suit.is_heart() && def.kind != CardKind::Sha);
    assert!(matches!(
        play(&mut e, dc2, vec![1]),
        engine_native::CommandOutcome::Accepted { .. }
            | engine_native::CommandOutcome::Pending { .. }
    ));
    pass_nullify_windows(&mut e);
    drain_choices(&mut e);
    let _ = (zy_card, zy_card2);
    assert_eq!(hp(&e, 0), hp0_after_black - 1, "仙音：判定红→0 号失 1");
    assert_zone_conservation(&e);

    // ④ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

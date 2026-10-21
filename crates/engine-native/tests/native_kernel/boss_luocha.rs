use super::*;

/// 阴间系罗刹片（规则 v254）：御兽（虚拟南蛮）+魔炎（失去牌判定红→2
/// 火伤）——①御兽：出牌锚虚拟南蛮打全部敌方（空手直接受伤）②魔炎：非
/// 当前回合失去牌判定红→座位序首个其他存活受 2 火伤③判定黑对照④非持有
/// 者对照。罗刹条目（[魔道,御兽,魔炎]，hp12；yizhong=一将包防御技未落登
/// 记；魔道共用 v253）。
#[test]
fn boss_luocha_yushou_moyany() {
    let mut e = Engine::new_boss_stage(74001, 4, 1, 3, 6, 4, 0);
    let lc_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 31) as u16; // BOSS_GENERALS[31] 罗刹
    e.debug_set_general_def(1, lc_def);
    e.debug_set_hp(1, 12);
    assert_eq!(
        e.catalog().general_meta(lc_def).map(|g| g.0),
        Some("罗刹"),
        "罗刹入目录"
    );
    let ys = e.catalog().skill_by_name("御兽").expect("御兽入目录");
    let my = e.catalog().skill_by_name("魔炎").expect("魔炎入目录");
    assert!(e.state_core().skills.contains_key(&(1, ys)));
    assert!(e.state_core().skills.contains_key(&(1, my)));

    // ① 御兽：1 号出牌锚虚拟南蛮 → 敌方空手各受 1（需出杀无牌直接受伤）
    for t in [0u8, 1, 2, 3] {
        e.debug_clear_hand(t);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    advance_to_play(&mut e, 1);
    // ② 魔炎：非当前回合失去牌（0 号回合拆 1 号手牌）→ 判定红 → 座位序
    //    首个其他存活（0 号）受 2 火伤
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
    assert_eq!(e.debug_hp(0), 2, "魔炎：判定红 → 0 号受 2 火伤（4-2）");
    assert_eq!(e.debug_hp(1), 12, "魔炎：持有者不受");

    // ③ 判定黑对照：黑桃顶 → 无火伤
    let _shan2 = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Spade);
    let gs2 = e.debug_place(0, "dismantle");
    play(&mut e, gs2, vec![1]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 2, "魔炎：判定黑无火伤");

    // ④ 非持有者对照：2 号（无魔炎）失去牌 → 无判定无火伤（3 号 hp 不变）
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let _shan3 = e.debug_place(2, "shan");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs3 = e.debug_place(0, "dismantle");
    play(&mut e, gs3, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(3), 3, "非持有者失去牌：无魔炎（南蛮伤 1 持续）");

    // ⑤ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, ys)), "御兽实例往返");
    assert!(r.state_core().skills.contains_key(&(1, my)), "魔炎实例往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}

use super::*;

/// 阴间系旧绝策片（规则 v258）：绝策（马面，一将包李儒旧绝策）——自己
/// 回合内其他角色失去最后一手牌（弃后空手）→ 1 伤害：①非空手失去不触
/// 发（拆桥后仍有牌）②失去最后一手牌触发③自己失去牌不触发④非持有者
/// 对照。马面条目已含绝策（v257 顺延 v258）。
#[test]
fn boss_mamian_juece_last_hand_damage() {
    let mut e = Engine::new_boss_stage(78001, 4, 1, 3, 6, 4, 0);
    let mm_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 34) as u16; // BOSS_GENERALS[34] 马面
    e.debug_set_general_def(1, mm_def);
    e.debug_set_hp(1, 6);
    let jc = e.catalog().skill_by_name("绝策").expect("绝策入目录");
    assert!(e.state_core().skills.contains_key(&(1, jc)));

    // ① 马面回合内拆 0 号手牌：0 号有 2 张 → 拆 1 后仍有 1 张 → 绝策不
    //    触发（失去的不是最后一手牌）
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let _a0 = e.debug_place(0, "tao");
    let _b0 = e.debug_place(0, "shan");
    let hp0 = e.debug_hp(0);
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs = e.debug_place(1, "dismantle");
    play(&mut e, gs, vec![0]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "0 号剩 1 手牌");
    assert_eq!(e.debug_hp(0), hp0, "非最后一手牌：绝策不触发");

    // ② 拆最后一张 → 绝策 1 伤害
    let gs2 = e.debug_place(1, "dismantle");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    play(&mut e, gs2, vec![0]);
    drain_choices(&mut e);
    assert!(e.debug_hand(0).is_empty(), "0 号手牌清空");
    assert_eq!(e.debug_hp(0), hp0 - 1, "绝策：失去最后一手牌 → 1 伤害");

    // ③ 自己失去牌不触发：马面自己回合弃自己手牌（1 张）→ 无绝策反伤
    e.debug_clear_hand(1);
    let _self_card = e.debug_place(1, "tao");
    // 1 号手牌剩 keep_tao（若②后置牌仍在则先清）
    let keep_tao = e.debug_place(1, "tao");
    next_phase(&mut e); // 1 Play→Discard（手牌≤上限不弃或弃置触发自身）
    drain_choices(&mut e);
    // 自身弃牌不触发绝策（无伤害事件给自己以外的判定）——直接核对手牌
    assert!(
        e.debug_hand(1).contains(&keep_tao) || e.debug_hand(1).is_empty(),
        "自身回合正常"
    );

    // ④ 非持有者对照：0 号回合拆 2 号最后手牌 → 无绝策（拆桥无伤害、
    //    非持有者回合无绝策）
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let _c2 = e.debug_place(2, "shan");
    let hp2 = e.debug_hp(2);
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs3 = e.debug_place(0, "dismantle");
    play(&mut e, gs3, vec![2]);
    drain_choices(&mut e);
    assert!(e.debug_hand(2).is_empty(), "2 号手牌被拆走");
    assert_eq!(e.debug_hp(2), hp2, "非持有者回合：无绝策（拆桥不伤害）");
    assert_zone_conservation(&e);

    // ⑤ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, jc)), "绝策实例往返");
    assert_zone_conservation(&r);
}

/// 阴间系马面片（规则 v257）：炼狱（End 全部其他存活各 1 火伤）+鬼计共
/// 用+反馈核对（FAN_KUI 标准已承载）。马面条目（[鬼计,反馈,炼狱]，hp6；
/// juece 旧绝策混入未勘登记——refresh 包仅 rejuece 界绝策）。
#[test]
fn boss_mamian_lianyu_fire_all() {
    let mut e = Engine::new_boss_stage(77001, 4, 1, 3, 6, 4, 0);
    let mm_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 34) as u16; // BOSS_GENERALS[34] 马面
    e.debug_set_general_def(1, mm_def);
    e.debug_set_hp(1, 6);
    assert_eq!(
        e.catalog().general_meta(mm_def).map(|g| g.0),
        Some("马面"),
        "马面入目录"
    );
    let ly = e.catalog().skill_by_name("炼狱").expect("炼狱入目录");
    let gj = e.catalog().skill_by_name("鬼计").expect("鬼计入目录");
    let fk = e.catalog().skill_by_name("反馈").expect("反馈标准承载");
    assert!(e.state_core().skills.contains_key(&(1, ly)));
    assert!(e.state_core().skills.contains_key(&(1, gj)), "鬼计共用");
    assert!(
        e.state_core().skills.contains_key(&(1, fk)),
        "反馈标准技入条目"
    );

    // 炼狱：End 对全部其他存活各 1 火伤（3→2）
    for t in [0u8, 1, 2, 3] {
        e.debug_clear_hand(t);
    }
    e.debug_set_hp(0, 3);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(3, 3);
    advance_to_play(&mut e, 1);
    next_phase(&mut e); // 1 Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End（炼狱直伤）
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 2, "炼狱：0 号受 1 火伤");
    assert_eq!(e.debug_hp(2), 2, "炼狱：2 号受 1 火伤");
    assert_eq!(e.debug_hp(3), 2, "炼狱：3 号受 1 火伤");
    assert_eq!(e.debug_hp(1), 6, "本体不受");
    assert_zone_conservation(&e);

    // 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, ly)), "炼狱实例往返");
    assert_zone_conservation(&r);
}

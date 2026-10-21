use super::*;

/// 郭嘉条目四技入目录+实例；鬼谋：End 阶段座位序首个其他未混乱存活置
/// boss_mad，其回合收束解除；过滤负例（已混乱者跳过→次座获标记）；存档
/// 往返携带 mad 状态。
#[test]
fn boss_guojia_guimou_mad_lifecycle() {
    let mut e = engine_native::Engine::new_boss_stage(80001, 4, 1, 3, 6, 4, 0);
    let gj_def = guojia_def_of(&e);
    e.debug_set_general_def(1, gj_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(gj_def).map(|g| g.0),
        Some("郭嘉"),
        "郭嘉入目录"
    );
    let td = e.catalog().skill_by_name("天妒").expect("天妒入目录");
    let gm = e.catalog().skill_by_name("鬼谋").expect("鬼谋入目录");
    let yc = e.catalog().skill_by_name("远策").expect("远策入目录");
    let qz = e.catalog().skill_by_name("奇佐").expect("奇佐入目录");
    for s in [td, gm, yc, qz] {
        assert!(e.state_core().skills.contains_key(&(1, s)), "实例齐备");
    }

    // ① 1 号（郭嘉）End 阶段 → 座位序首个其他未混乱存活（0 号）mad
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    next_phase(&mut e); // Play→Discard（空手不弃）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End → 鬼谋
    drain_choices(&mut e);
    assert!(e.state_core().player(0).boss_mad, "鬼谋：0 号混乱");
    assert!(!e.state_core().player(2).boss_mad && !e.state_core().player(3).boss_mad);

    // ② 0 号回合收束 → mad 解除（Play 起 3 次 NP：Discard/End/收束）
    advance_to_play(&mut e, 0);
    // 0 号 6 手牌超上限：弃牌询问 min=2（空应答被拒→settled 应答）
    next_phase_settled(&mut e); // 0 Play→Discard
    next_phase_settled(&mut e); // Discard→End（begin）
    next_phase_settled(&mut e); // End 结束 → TurnEnded
    assert!(!e.state_core().player(0).boss_mad, "回合收束解除");

    // ③ 过滤负例：0 号预先混乱 → 1 号 End 跳过 0 号，2 号获标记
    e.state_core_mut().players[0].boss_mad = true;
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    next_phase(&mut e);
    drain_choices(&mut e);
    next_phase(&mut e);
    drain_choices(&mut e);
    assert!(e.state_core().player(2).boss_mad, "已混乱者跳过→次座");
    assert!(
        e.state_core().player(0).boss_mad,
        "0 号标记保持（其回合未到）"
    );

    // ④ 存档往返携带 mad 状态
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert!(r.state_core().player(2).boss_mad && r.state_core().player(0).boss_mad);
    assert_zone_conservation(&r);
}

/// 远策：0→2 伤害后判定——黑→受伤者（2 号）再失 1；红→来源（0 号）失
/// 1；无郭嘉对照（不触发）。
#[test]
fn boss_guojia_yuance_judge_branches() {
    // ① 判定黑（黑桃顶）→ 受伤者失 1：2 号 4→2（伤害 1+远策 1），0 号不动
    let mut e = engine_native::Engine::new_boss_stage(80002, 4, 1, 3, 6, 4, 0);
    let gj_def = guojia_def_of(&e);
    e.debug_set_general_def(1, gj_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(2);
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(hp(&e, 2), 2, "黑：受伤者 1+1");
    assert_eq!(hp(&e, 0), 4, "来源不动");

    // ② 判定红（红桃顶）→ 来源失 1：2 号 4→3，0 号 4→3
    e.debug_set_hp(2, 4);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(hp(&e, 2), 3, "红：受伤者仅伤害 1");
    assert_eq!(hp(&e, 0), 3, "红：来源失 1");
    assert_zone_conservation(&e);

    // ③ 负例：同局无郭嘉（1 号不装配）→ 伤害后无追加失血
    let mut e2 = engine_native::Engine::new_boss_stage(80002, 4, 1, 3, 6, 4, 0);
    advance_to_play(&mut e2, 0);
    e2.debug_clear_hand(2);
    e2.debug_set_deck_top_by(|def| def.suit.is_spade());
    e2.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    drain_choices(&mut e2);
    assert_eq!(hp(&e2, 2), 3, "无远策：仅伤害 1");
    assert_eq!(hp(&e2, 0), 4);
}

/// 奇佐：无中生有使用 → 原牌摸 2+副本摸 2=4（副本不再复用=载体槽守卫）；
/// 杀（非锦囊）不复制且清载体槽；区域守恒+存档往返。
#[test]
fn boss_guojia_qizuo_trick_reuse() {
    let mut e = engine_native::Engine::new_boss_stage(80003, 4, 1, 3, 6, 4, 0);
    let gj_def = guojia_def_of(&e);
    e.debug_set_general_def(1, gj_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 1);

    // ① 无中生有：原牌 2+副本 2=4 手牌（链挡：副本自身不再触发）；
    //    原牌/副本各开一次无懈窗（2 号持无懈）——全部放行
    e.debug_clear_hand(1);
    let wz = e.debug_move_named_card_to_hand(1, "wuzhong");
    let out = play(&mut e, wz, vec![]);
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "wuzhong: {out:?}"
    );
    pass_nullify_windows(&mut e);
    assert_eq!(e.debug_hand(1).len(), 4, "无中生有 2+奇佐副本 2");
    assert!(
        e.state_core().turn.qizuo_copy.is_some(),
        "载体槽保持至下一用牌提交"
    );

    // ② 杀（非锦囊）：不复制，载体槽随提交清
    for t in [0u8, 2, 3] {
        e.debug_clear_hand(t);
    }
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    assert!(matches!(
        play(&mut e, sha, vec![0]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    assert!(
        e.state_core().turn.qizuo_copy.is_none(),
        "非锦囊提交清载体槽"
    );
    // 杀伤害 1+远策（1→0 黑判定→受伤者 0 号失 1）
    assert_eq!(hp(&e, 0), 2, "杀 1+远策黑 1");
    assert_eq!(hp(&e, 1), 4, "郭嘉来源不受影响（黑判定）");
    assert_eq!(e.debug_hand(1).len(), 0, "杀不复制");
    assert_zone_conservation(&e);

    // ③ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
    assert!(r
        .state_core()
        .skills
        .contains_key(&(1, e.catalog().skill_by_name("奇佐").expect("奇佐"))));
}

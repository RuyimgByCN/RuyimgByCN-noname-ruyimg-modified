use super::*;

/// 轮回：1 号（转轮王，hp 压到 2）准备阶段与 0 号（体力 4，并列最高取
/// 座位序首个）交换体力值；魔道共用技同阶段摸 2。
#[test]
fn boss_zhuanlunwang_lunhui_swap_at_judge() {
    let mut e = engine_native::Engine::new_boss_stage(87030, 4, 0, 3, 6, 4, 0);
    let zlw_def = zhuanlunwang_def_of(&e);
    e.debug_set_general_def(1, zlw_def);
    e.debug_set_hp(1, 2);
    assert_eq!(
        e.catalog().general_meta(zlw_def).map(|g| g.0),
        Some("转轮王"),
        "转轮王入目录"
    );
    for sk in ["魔道", "轮回", "往生", "反噬"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    advance_to_play(&mut e, 0);
    let n0 = hp(&e, 0);
    // 推进：0 Play→Discard→End→收束→1 号 Judge begin（魔道摸 2+轮回换血）
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    assert_eq!(
        hp(&e, 1),
        n0,
        "轮回：1 号获得 0 号体力（min(maxHp) 封顶内）"
    );
    assert_eq!(hp(&e, 0), 2, "轮回：0 号获得 1 号的 2 点体力");
    assert!(
        e.state_core().player(1).hand.len() >= 2,
        "魔道：准备阶段摸 2"
    );

    // 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

/// 往生：1 号出牌阶段开始恒定虚拟南蛮（全部其他存活为目标）；空手牌
/// 应答全 Pass → 0/2/3 各受 1 伤。
#[test]
fn boss_zhuanlunwang_wangsheng_virtual_savage() {
    let mut e = engine_native::Engine::new_boss_stage(87031, 4, 0, 3, 6, 4, 0);
    let zlw_def = zhuanlunwang_def_of(&e);
    e.debug_set_general_def(1, zlw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    for p in [0u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let (n0, n2, n3) = (hp(&e, 0), hp(&e, 2), hp(&e, 3));
    // 推进到 1 号出牌阶段开始（往生在此触发虚拟南蛮）
    next_phase_settled(&mut e); // 0 Play→Discard
    next_phase_settled(&mut e); // Discard→End begin
    next_phase_settled(&mut e); // End→收束→1 Judge begin（魔道摸 2）
    next_phase_settled(&mut e); // Judge→Draw
    next_phase_settled(&mut e); // Draw→Play begin → 往生+首问
                                // 全部挂起询问（无懈窗/南蛮响应）显式 Pass 放行
    for _ in 0..16 {
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            None => break,
        }
    }
    assert_eq!(hp(&e, 0), n0 - 1, "往生：0 号南蛮无杀受 1");
    assert_eq!(hp(&e, 2), n2 - 1, "往生：2 号南蛮无杀受 1");
    assert_eq!(hp(&e, 3), n3 - 1, "往生：3 号南蛮无杀受 1");
    assert_zone_conservation(&e);
}

/// 反噬：同回合第二次受伤 → 座位序首个其他存活（0 号）受 1；回合级
/// terra 旗交替（第三次受伤不再反噬）。
#[test]
fn boss_zhuanlunwang_fanshi_damage_toggle() {
    let mut e = engine_native::Engine::new_boss_stage(87032, 4, 0, 0, 6, 4, 0);
    let zlw_def = zhuanlunwang_def_of(&e);
    e.debug_set_general_def(1, zlw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let n0 = hp(&e, 0);

    // 第一次受伤：置 terra 旗，无反噬
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 3, "第一次受伤");
    assert_eq!(hp(&e, 0), n0, "首伤未触发反噬（terra 旗置位）");

    // 第二次受伤：反噬 → 0 号受 1
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 2, "第二次受伤");
    assert_eq!(hp(&e, 0), n0 - 1, "反噬：0 号（座位序首个其他）受 1");

    // 第三次受伤：旗已清，重新置位不反噬
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 1, "第三次受伤");
    assert_eq!(hp(&e, 0), n0 - 1, "反噬不重复触发（旗交替）");

    // 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

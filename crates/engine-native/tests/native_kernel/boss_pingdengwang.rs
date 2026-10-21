use super::*;

/// 锁足：平等王准备阶段——全部其他存活横置（不分阵营），本人不横置。
#[test]
fn boss_pingdengwang_suozu_link_all_others() {
    let mut e = engine_native::Engine::new_boss_stage(87046, 4, 0, 3, 6, 4, 0);
    let pdw_def = pingdengwang_def_of(&e);
    e.debug_set_general_def(1, pdw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(pdw_def).map(|g| g.0),
        Some("平等王"),
        "平等王入目录"
    );
    for sk in ["锁足", "阿鼻", "平等"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);
    // 推进 0 号回合收束 → 1 号准备阶段（锁足在此触发）
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    for t in [0u8, 2, 3] {
        assert!(e.state_core().player(t).linked, "锁足：{t} 号横置");
    }
    assert!(!e.state_core().player(1).linked, "持有者本人不横置");
    assert_eq!(
        count_log(&e, |l| matches!(l, LogEvent::Linked { linked: true, .. })),
        3,
        "横置日志×3"
    );
}

/// 阿鼻：受有来源伤害后来源受 1 火伤；无来源伤害不触发。
#[test]
fn boss_pingdengwang_abi_source_fire_no_source_skip() {
    let mut e = engine_native::Engine::new_boss_stage(87047, 4, 0, 0, 6, 4, 0);
    let pdw_def = pingdengwang_def_of(&e);
    e.debug_set_general_def(1, pdw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let n0 = hp(&e, 0);
    // 有来源：来源 0 号受 1 火伤（恒定火近似）
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 3, "1 号受 1 伤");
    assert_eq!(hp(&e, 0), n0 - 1, "阿鼻：来源受 1 火伤");
    // 无来源：不触发
    let out = e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 0), n0 - 1, "阿鼻：无来源伤害不触发");
}

/// 平等：平等王真死（无来源致死伤避开阿鼻）→ 波一最高体力（并列取
/// 0 号）受 2 火伤、波二重算（0 号掉队）取 2 号受 1 火伤；往返。
#[test]
fn boss_pingdengwang_pingdeng_death_two_wave_fire() {
    let mut e = engine_native::Engine::new_boss_stage(87048, 4, 0, 0, 6, 4, 0);
    let pdw_def = pingdengwang_def_of(&e);
    e.debug_set_general_def(1, pdw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let (n0, n2, n3) = (hp(&e, 0), hp(&e, 2), hp(&e, 3));

    let out = e.debug_deal_damage(None, 1, 10, DamageNature::Normal);
    let _ = out;
    for _ in 0..3 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "dying_save" => {
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
            _ => break,
        }
    }
    assert!(!e.state_core().player(1).alive, "平等王已死亡");
    assert_eq!(hp(&e, 0), n0 - 2, "平等：波一最高体力并列取 0 号受 2 火伤");
    assert_eq!(hp(&e, 2), n2 - 1, "平等：波二重算最高体力取 2 号受 1 火伤");
    assert_eq!(hp(&e, 3), n3, "3 号未受波及");

    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

use super::*;

/// 判官：乐不思蜀目标被拒；顺手（非延时）可指定。
#[test]
fn boss_qinguangwang_panguan_delay_reject() {
    let mut e = engine_native::Engine::new_boss_stage(87061, 4, 0, 3, 6, 4, 0);
    let qgw_def = qinguangwang_def_of(&e);
    e.debug_set_general_def(1, qgw_def);
    e.debug_set_hp(1, 3);
    assert_eq!(
        e.catalog().general_meta(qgw_def).map(|g| g.0),
        Some("秦广王"),
        "秦广王入目录"
    );
    for sk in ["判官", "拘魂", "望乡"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);
    let le = e.debug_place(0, "le");
    assert!(
        matches!(
            play(&mut e, le, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "判官：延时锦囊目标被拒"
    );
    let sn = e.debug_place(0, "snatch");
    let out = play(&mut e, sn, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "判官：非延时锦囊可指定（{out:?}）"
    );
    drain_choices(&mut e);
}

/// 拘魂：1 号（秦广王）结束阶段——座位序首个其他（0 号）被翻面。
#[test]
fn boss_qinguangwang_juhun_end_flip_first() {
    let mut e = engine_native::Engine::new_boss_stage(87062, 4, 0, 3, 6, 4, 0);
    let qgw_def = qinguangwang_def_of(&e);
    e.debug_set_general_def(1, qgw_def);
    e.debug_set_hp(1, 3);
    advance_to_play(&mut e, 0);
    assert!(!e.debug_face_down(0), "前置：0 号正面");
    // 0 号回合收束 → 1 号回合 → 1 号结束阶段（拘魂触发）
    advance_to_play(&mut e, 1);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    assert!(e.debug_face_down(0), "拘魂：0 号被翻面");
}

/// 望乡：秦广王真死 → 全部有装备的其他角色装备全弃（撤技能+入弃堆）；
/// 往返。
#[test]
fn boss_qinguangwang_wangxiang_death_strip_equips() {
    let mut e = engine_native::Engine::new_boss_stage(87063, 4, 0, 0, 6, 4, 0);
    let qgw_def = qinguangwang_def_of(&e);
    e.debug_set_general_def(1, qgw_def);
    e.debug_set_hp(1, 3);
    let armor0 = equip_card(&mut e, 0, "armor");
    let renwang2 = equip_card(&mut e, 2, "renwang");
    advance_to_play(&mut e, 0);
    let out = e.debug_deal_damage(Some(0), 1, 10, DamageNature::Normal);
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
    assert!(!e.state_core().player(1).alive, "秦广王已死亡");
    assert!(
        e.state_core().player(0).equips.is_empty(),
        "望乡：0 号装备全弃"
    );
    assert!(
        e.state_core().player(2).equips.is_empty(),
        "望乡：2 号装备全弃"
    );
    assert!(
        e.state_core().discard.contains(&armor0) && e.state_core().discard.contains(&renwang2),
        "望乡：装备入弃堆"
    );
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

use super::*;

/// 黑绳：宋帝王死亡 → 全部其他存活横置；胜负：End 首个有坐骑的其他存
/// 活弃其一。
#[test]
fn boss_songdiwang_heisheng_death_link_shengfu() {
    // boss=0 + chongzheng=0：宋帝王在 1 号位（Boss 模式全座位重整复活
    // 会拦真死，本测需真死触发黑绳遗言）
    let mut e = engine_native::Engine::new_boss_stage(87001, 4, 0, 0, 6, 4, 0);
    let sdw_def = songdiwang_def_of(&e);
    e.debug_set_general_def(1, sdw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(sdw_def).map(|g| g.0),
        Some("宋帝王"),
        "宋帝王入目录"
    );
    for sk in ["黑绳", "胜负"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);

    // ① 黑绳：宋帝王死亡 → 全部其他存活横置
    let out = e.debug_deal_damage(Some(0), 1, 10, DamageNature::Normal);
    let _ = out;
    // 濒死救援询问（dying_save min=1 空应答被拒）：显式 Pass 放行死亡
    for _ in 0..4 {
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
    assert!(!e.debug_alive(1), "宋帝王死亡");
    for p in [0u8, 2, 3] {
        assert!(e.state_core().player(p).linked, "黑绳：{p} 号横置");
    }

    // ② 胜负（新局）：1 号宋帝王 End → 首个有坐骑的其他存活（0 号）弃坐骑
    let mut e = engine_native::Engine::new_boss_stage(87002, 4, 0, 3, 6, 4, 0);
    let zj_def2 = songdiwang_def_of(&e);
    e.debug_set_general_def(1, zj_def2);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    // 0 号出 +1 马（装备入马栏）
    let horse = e.debug_place(0, "horse_plus");
    assert_eq!(
        e.catalog().cards[horse as usize].kind,
        engine_native::catalog::CardKind::HorsePlus
    );
    assert!(matches!(
        play(&mut e, horse, vec![]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    assert_eq!(e.state_core().player(0).equips.len(), 1, "马已装备");
    advance_to_play(&mut e, 1); // 穿过 0 回合余量 → 1 号 Play
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → 胜负
    drain_choices(&mut e);
    assert!(
        e.state_core().player(0).equips.is_empty(),
        "胜负：弃 0 号坐骑"
    );
    assert_zone_conservation(&e);

    // ③ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

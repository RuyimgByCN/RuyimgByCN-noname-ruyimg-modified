use super::*;

/// 雷舟：卞城王准备阶段（Judge begin 锚点）→ 座位序首个其他存活受 1 雷
/// 伤；雷斧：其结束阶段 → 同一口径目标横置态切换。
#[test]
fn boss_bianchengwang_leizhou_leifu_phase_arms() {
    let mut e = engine_native::Engine::new_boss_stage(87010, 4, 0, 3, 6, 4, 0);
    let bcw_def = bianchengwang_def_of(&e);
    e.debug_set_general_def(1, bcw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(bcw_def).map(|g| g.0),
        Some("卞城王"),
        "卞城王入目录"
    );
    for sk in ["雷舟", "雷斧", "雷诛"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    // ① 雷舟：推进到 1 号 Judge begin（固定摸牌前锚点同口径）即触发
    let n0 = hp(&e, 0);
    advance_to_play(&mut e, 1);
    assert_eq!(hp(&e, 0), n0 - 1, "雷舟：首个其他存活（0 号）受 1 雷伤");
    assert!(!e.state_core().player(0).linked, "雷斧未到：0 号未横置");

    // ② 雷斧：1 号 Play→Discard→End begin → 0 号横置态切换
    e.debug_clear_hand(1);
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → 雷斧
    drain_choices(&mut e);
    assert!(
        e.state_core().player(0).linked,
        "雷斧：首个其他存活（0 号）横置态切换为已横置"
    );

    // ③ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

/// 雷诛：卞城王真死（chongzheng=0）→ 座位序逐发雷伤；2 号压到 1 血使
/// 其进入濒死，挂起期存档往返验证 BossLeizhu 快照（emitted 进度）续跑。
#[test]
fn boss_bianchengwang_leizhu_death_thunder_sequential() {
    let mut e = engine_native::Engine::new_boss_stage(87011, 4, 0, 0, 6, 4, 0);
    let bcw_def = bianchengwang_def_of(&e);
    e.debug_set_general_def(1, bcw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);

    // 2 号压到 1 血：雷诛第二发将其打入濒死（挂起点=存档往返窗口）
    e.debug_set_hp(2, 1);
    let (n0, n3) = (hp(&e, 0), hp(&e, 3));

    let out = e.debug_deal_damage(Some(0), 1, 10, DamageNature::Normal);
    let _ = out;
    // 1 号自身濒死：其余三座位各一问（救援轮转 0→1→2），显式 Pass 放行
    // 死亡 → 雷诛遗言开档
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
    assert_eq!(hp(&e, 0), n0 - 1, "雷诛第 1 发：0 号（座位序首个）");

    // 第 2 发把 2 号打入濒死：机器挂起在 emitted=2，此处存档往返
    //（alive 旗标在死亡窗口收束才落，挂起期 1 号仍报活=窗口语义）
    let snap = e.save();
    let mut r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);

    // 濒死救援逐问 Pass → 2 号死，机器续跑第 3 发，窗口收束
    for _ in 0..6 {
        match r.open_choice() {
            Some(c) if c.spec.prompt == "dying_save" => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    &mut r,
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
    assert!(!r.debug_alive(1), "卞城王死亡（窗口收束后旗标落下）");
    assert!(!r.debug_alive(2), "2 号被雷诛打死的濒死未救援");
    assert_eq!(hp(&r, 0), n0 - 1, "续跑不重复第 1 发（emitted 进度保留）");
    assert_eq!(hp(&r, 3), n3 - 1, "雷诛第 3 发：3 号（座位序末位）");
    assert_zone_conservation(&r);
}

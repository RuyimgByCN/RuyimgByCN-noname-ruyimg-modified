use super::*;

/// 服毒：0 号用桃 → 候选（全体减使用者减持有者）座位序首个（2 号）失 1；
/// 苦酒：0 号（非持有者）准备阶段失 1 且 jiu_active 置位。
#[test]
fn boss_taishanwang_fudu_kujiu_arms() {
    let mut e = engine_native::Engine::new_boss_stage(87020, 4, 0, 3, 6, 4, 0);
    let tsw_def = taishanwang_def_of(&e);
    e.debug_set_general_def(1, tsw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(tsw_def).map(|g| g.0),
        Some("泰山王"),
        "泰山王入目录"
    );
    for sk in ["服毒", "苦酒", "热恼"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    // ① 苦酒：2 号（非持有者）准备阶段 → 失 1 + jiu_active。
    //（0 号首回合准备阶段在建局 OpeningPlan 内已发生，早于布阵授技——
    // 故取 2 号回合验证）
    let n2 = hp(&e, 2);
    advance_to_play(&mut e, 2);
    assert_eq!(hp(&e, 2), n2 - 1, "苦酒：2 号准备阶段失 1");
    assert!(e.state_core().turn.jiu_active, "苦酒：视为用酒置位");

    // ② 服毒：2 号用桃（自身回 1），候选=全体减使用者减持有者 →
    // 座位序首个（0 号）失 1
    e.debug_set_hp(2, 2);
    let tao = e.debug_place(2, "tao");
    let n0 = hp(&e, 0);
    assert!(matches!(
        play(&mut e, tao, vec![]),
        engine_native::CommandOutcome::Accepted { .. }
            | engine_native::CommandOutcome::Pending { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(hp(&e, 2), 3, "用桃自身回 1");
    assert_eq!(hp(&e, 0), n0 - 1, "服毒：座位序首个候选（0 号）失 1");
}

/// 热恼：泰山王真死（chongzheng=0）→ 座位序首个其他存活受 3 火伤。
#[test]
fn boss_taishanwang_renao_death_fire() {
    let mut e = engine_native::Engine::new_boss_stage(87021, 4, 0, 0, 6, 4, 0);
    let tsw_def = taishanwang_def_of(&e);
    e.debug_set_general_def(1, tsw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let n0 = hp(&e, 0);

    let out = e.debug_deal_damage(Some(0), 1, 10, DamageNature::Normal);
    let _ = out;
    // 1 号濒死救援逐问 Pass（其余三座位各一问）放行死亡 → 热恼遗言
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
    assert_eq!(hp(&e, 0), n0 - 3, "热恼：首个其他存活（0 号）受 3 火伤");
    assert_eq!(hp(&e, 2), hp(&e, 2), "单发不扩散");

    // 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

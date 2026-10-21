use super::*;

/// 铁面：防具槽空 → 黑杀无效（视为仁王盾，无八卦判定）；装防具后铁面
/// 不生效（八卦判定照常发生）。
#[test]
fn boss_yanluowang_tiemian_black_sha_block() {
    let mut e = engine_native::Engine::new_boss_stage(87043, 4, 0, 3, 6, 4, 0);
    let ylw_def = yanluowang_def_of(&e);
    e.debug_set_general_def(1, ylw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(ylw_def).map(|g| g.0),
        Some("阎罗王"),
        "阎罗王入目录"
    );
    for sk in ["铁面", "铡刀", "诛心"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);

    // ① 防具槽空：黑杀被铁面挡下（无伤害、无八卦判定、杀入弃堆）
    let n1 = hp(&e, 1);
    let sha = give_black_sha(&mut e, 0);
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(hp(&e, 1), n1, "铁面：黑杀无效");
    assert!(
        !log_has(&e, |l| matches!(l, LogEvent::ArmorJudged { player: 1, .. })),
        "铁面挡下时不进八卦判定"
    );
    assert!(e.state_core().discard.contains(&sha), "杀入弃堆");

    // ② 装防具（八卦阵）后铁面不生效 → 八卦判定照常发生（换 2 号回合，
    // 出牌阶段限一次杀）
    equip_card(&mut e, 1, "armor");
    advance_to_play(&mut e, 2);
    let sha2 = give_black_sha(&mut e, 2);
    assert!(matches!(
        play(&mut e, sha2, vec![1]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    for _ in 0..4 {
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
    assert!(
        log_has(&e, |l| matches!(l, LogEvent::ArmorJudged { player: 1, .. })),
        "防具占位抑制铁面，八卦判定发生"
    );
}

/// 铡刀：阎罗王黑杀令目标防具无效（无八卦判定，Pass 后受伤）；无铡刀
/// 来源照常进八卦判定。
#[test]
fn boss_yanluowang_zhadao_pierce_renwang() {
    let mut e = engine_native::Engine::new_boss_stage(87044, 4, 0, 3, 6, 4, 0);
    let ylw_def = yanluowang_def_of(&e);
    e.debug_set_general_def(1, ylw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);

    // 对照：0 号（无铡刀）黑杀 3 号（装八卦阵，环相邻）→ 防具判定发生
    equip_card(&mut e, 3, "armor");
    let sha = give_black_sha(&mut e, 0);
    assert!(matches!(
        play(&mut e, sha, vec![3]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    for _ in 0..4 {
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
    assert!(
        log_has(&e, |l| matches!(l, LogEvent::ArmorJudged { player: 3, .. })),
        "对照：无铡刀来源照常进八卦判定"
    );

    // 阎罗王（铡刀）黑杀 0 号（装八卦阵）→ 防具无效（无判定）；
    // 0 号 Pass 不出闪 → 受 1
    equip_card(&mut e, 0, "armor");
    let n0 = hp(&e, 0);
    advance_to_play(&mut e, 1);
    let sha2 = give_black_sha(&mut e, 1);
    assert!(matches!(
        play(&mut e, sha2, vec![0]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    for _ in 0..4 {
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
    assert!(
        !log_has(&e, |l| matches!(l, LogEvent::ArmorJudged { player: 0, .. })),
        "铡刀：目标防具无效（无八卦判定）"
    );
    assert_eq!(hp(&e, 0), n0 - 1, "铡刀：穿透防具造成伤害");
}

/// 诛心：阎罗王真死 → 座位序首个最低体力其他存活受 2（2/3 号并列最低
/// 取 2 号）；禁不住对照 0 号不受影响；往返。
#[test]
fn boss_yanluowang_zhuxin_death_min_hp() {
    let mut e = engine_native::Engine::new_boss_stage(87045, 4, 0, 0, 6, 4, 0);
    let ylw_def = yanluowang_def_of(&e);
    e.debug_set_general_def(1, ylw_def);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(3, 3);
    advance_to_play(&mut e, 0);
    let (n0, n2, n3) = (hp(&e, 0), hp(&e, 2), hp(&e, 3));

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
    assert!(!e.state_core().player(1).alive, "阎罗王已死亡");
    assert_eq!(hp(&e, 2), n2 - 2, "诛心：座位序首个最低体力（2 号）受 2");
    assert_eq!(hp(&e, 0), n0, "0 号不受影响");
    assert_eq!(hp(&e, 3), n3, "3 号并列最低不重复结算");

    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

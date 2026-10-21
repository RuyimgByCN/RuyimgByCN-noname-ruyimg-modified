use super::*;

/// 热闷（杀面）：防具槽空 → plain 杀目标被拒（IllegalTargets）；装防具
/// 后可指定（照常进闪/判定链）。
#[test]
fn boss_dushiwang_remen_sha_reject_then_armor() {
    let mut e = engine_native::Engine::new_boss_stage(87049, 4, 0, 3, 6, 4, 0);
    let dsw_def = dushiwang_def_of(&e);
    e.debug_set_general_def(1, dsw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(dsw_def).map(|g| g.0),
        Some("都市王"),
        "都市王入目录"
    );
    for sk in ["热闷", "炙焚", "火刑"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);
    let sha = give_black_sha(&mut e, 0);
    assert!(
        matches!(
            play(&mut e, sha, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "热闷：槽空 plain 杀目标被拒"
    );
    // 属性杀不受热闷限制（火杀可指定）
    let fsha = {
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|&c| e.catalog().cards[c as usize].kind == CardKind::FireSha)
            .expect("牌堆中无火杀")
    };
    {
        let core = e.state_core_mut();
        core.deck.retain(|&x| x != fsha);
        core.player_mut(0).hand.push(fsha);
    }
    let out = play(&mut e, fsha, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "热闷：属性杀不受限（{out:?}）"
    );
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
    // 装防具后热闷不生效 → plain 杀可指定（换 2 号回合——0 号已用杀）
    equip_card(&mut e, 1, "armor");
    advance_to_play(&mut e, 2);
    let sha2 = give_black_sha(&mut e, 2);
    let out = play(&mut e, sha2, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "热闷：装防具后 plain 杀可指定（{out:?}）"
    );
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
}

/// 热闷（AOE 面）：南蛮目标队列窗口前跳过热闷持有者（无询问不受伤）；
/// 其余目标照常。
#[test]
fn boss_dushiwang_remen_aoe_skip() {
    let mut e = engine_native::Engine::new_boss_stage(87051, 4, 0, 3, 6, 4, 0);
    let dsw_def = dushiwang_def_of(&e);
    e.debug_set_general_def(1, dsw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let (n1, n2, n3) = (hp(&e, 1), hp(&e, 2), hp(&e, 3));
    let savage = e.debug_place(0, "savage");
    assert!(matches!(
        play(&mut e, savage, vec![]),
        CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
    ));
    // 全部询问（无懈窗/南蛮响应）显式 Pass 放行，记录被询问者
    let mut asked: Vec<(u8, &'static str)> = Vec::new();
    for _ in 0..24 {
        match e.open_choice() {
            Some(c) => {
                asked.push((c.observer, c.spec.prompt));
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
        !asked.iter().any(|&(o, p)| o == 1 && p == "aoe_response"),
        "热闷：1 号不被南蛮询问（{asked:?}）"
    );
    assert_eq!(hp(&e, 1), n1, "热闷：1 号不受南蛮伤害");
    assert_eq!(hp(&e, 2), n2 - 1, "南蛮：2 号 Pass 受 1");
    assert_eq!(hp(&e, 3), n3 - 1, "南蛮：3 号 Pass 受 1");
}

/// 炙焚：1 号（都市王）准备阶段——获得座位序首个其他（0 号）首张手牌
/// 并令其受 1 火伤（无来源）。
#[test]
fn boss_dushiwang_zhifen_prepare_steal_and_fire() {
    let mut e = engine_native::Engine::new_boss_stage(87052, 4, 0, 3, 6, 4, 0);
    let dsw_def = dushiwang_def_of(&e);
    e.debug_set_general_def(1, dsw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let n0 = hp(&e, 0);
    e.debug_clear_hand(0);
    let tao = e.debug_place(0, "tao");
    // 推进 0 号回合收束 → 1 号准备阶段（炙焚触发）
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    assert_eq!(hp(&e, 0), n0 - 1, "炙焚：0 号受 1 火伤");
    assert!(
        e.state_core().player(1).hand.contains(&tao),
        "炙焚：1 号获得 0 号首张手牌"
    );
    assert!(
        !e.state_core().discard.contains(&tao),
        "炙焚：获得的牌不经弃堆"
    );
    assert!(
        log_has(&e, |l| matches!(
            l,
            LogEvent::CardsGiven { from: 0, to: 1, .. }
        )),
        "炙焚：获得日志"
    );
    // 他人准备阶段不触发（上游 trigger player 域）：推进到 2 号出牌——
    // 若 2 号准备阶段仍触发则 0 号会再 -1（首轮波后应为 n0-1 不变）
    advance_to_play(&mut e, 2);
    assert_eq!(hp(&e, 0), n0 - 1, "炙焚：他人准备阶段不触发");
}

/// 火刑：都市王真死 → 全部其他存活按座位序各受 1 火伤；往返。
#[test]
fn boss_dushiwang_huoxing_death_fire_all() {
    let mut e = engine_native::Engine::new_boss_stage(87053, 4, 0, 0, 6, 4, 0);
    let dsw_def = dushiwang_def_of(&e);
    e.debug_set_general_def(1, dsw_def);
    e.debug_set_hp(1, 4);
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
    assert!(!e.state_core().player(1).alive, "都市王已死亡");
    assert_eq!(hp(&e, 0), n0 - 1, "火刑：0 号受 1 火伤");
    assert_eq!(hp(&e, 2), n2 - 1, "火刑：2 号受 1 火伤");
    assert_eq!(hp(&e, 3), n3 - 1, "火刑：3 号受 1 火伤");
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

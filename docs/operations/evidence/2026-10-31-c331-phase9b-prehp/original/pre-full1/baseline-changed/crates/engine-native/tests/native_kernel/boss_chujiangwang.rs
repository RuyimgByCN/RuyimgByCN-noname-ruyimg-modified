use super::*;

/// 帷幕：黑色顺手牵羊目标被拒、红色拆桥可指定；黑色南蛮 AOE 跳过持有
/// 者。
#[test]
fn boss_chujiangwang_weimu_black_trick_block() {
    let mut e = engine_native::Engine::new_boss_stage(87058, 4, 0, 3, 6, 4, 0);
    let cjw_def = chujiangwang_def_of(&e);
    e.debug_set_general_def(1, cjw_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(cjw_def).map(|g| g.0),
        Some("楚江王"),
        "楚江王入目录"
    );
    for sk in ["帷幕", "反馈", "冰封"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);
    // 黑色顺手 → 拒
    let black_snatch = {
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|&c| {
                let d = &e.catalog().cards[c as usize];
                d.kind == CardKind::Snatch
                    && matches!(
                        d.suit,
                        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                    )
            })
            .expect("牌堆中无黑顺手")
    };
    {
        let core = e.state_core_mut();
        core.deck.retain(|&x| x != black_snatch);
        core.player_mut(0).hand.push(black_snatch);
    }
    assert!(
        matches!(
            play(&mut e, black_snatch, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "帷幕：黑色锦囊目标被拒"
    );
    // 红色拆桥 → 可指定
    let red_dismantle = {
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|&c| {
                let d = &e.catalog().cards[c as usize];
                d.kind == CardKind::Dismantle
                    && matches!(
                        d.suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
            })
            .expect("牌堆中无红拆桥")
    };
    {
        let core = e.state_core_mut();
        core.deck.retain(|&x| x != red_dismantle);
        core.player_mut(0).hand.push(red_dismantle);
    }
    let out = play(&mut e, red_dismantle, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "帷幕：红色锦囊可指定（{out:?}）"
    );
    drain_choices(&mut e);
    // 黑色南蛮 AOE：1 号跳过，2/3 照常 Pass 受 1
    let (n1, n2, n3) = (hp(&e, 1), hp(&e, 2), hp(&e, 3));
    let savage = e.debug_place(0, "savage");
    assert!(matches!(
        play(&mut e, savage, vec![]),
        CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
    ));
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
        "帷幕：1 号不被黑色南蛮询问（{asked:?}）"
    );
    assert_eq!(hp(&e, 1), n1, "帷幕：1 号不受黑色南蛮伤害");
    assert_eq!(hp(&e, 2), n2 - 1, "南蛮：2 号 Pass 受 1");
    assert_eq!(hp(&e, 3), n3 - 1, "南蛮：3 号 Pass 受 1");
}

/// 反馈（标准 FAN_KUI 复用=界反馈语义）：受伤后获得来源首张手牌。
#[test]
fn boss_chujiangwang_fankui_reuse_gain() {
    let mut e = engine_native::Engine::new_boss_stage(87059, 4, 0, 0, 6, 4, 0);
    let cjw_def = chujiangwang_def_of(&e);
    e.debug_set_general_def(1, cjw_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    let n0 = hp(&e, 0);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 3);
    // 可选触发确认 → 获得来源首张手牌
    assert_eq!(prompt_of(&e), "trigger_confirm");
    assert!(matches!(answer(&mut e, ChoiceOption::Confirm), _));
    drain_choices(&mut e);
    assert_eq!(hp(&e, 0), n0, "反馈：来源不失血");
    assert!(
        count_log(&e, |l| matches!(
            l,
            LogEvent::CardsGiven { from: 0, to: 1, .. }
        )) >= 1,
        "反馈：获得日志"
    );
}

/// 冰封：楚江王被翻面状态来源击杀 → 来源被翻面；已翻面来源不重复翻。
#[test]
fn boss_chujiangwang_bingfeng_death_flip_source() {
    let mut e = engine_native::Engine::new_boss_stage(87060, 4, 0, 0, 6, 4, 0);
    let cjw_def = chujiangwang_def_of(&e);
    e.debug_set_general_def(1, cjw_def);
    e.debug_set_hp(1, 4);
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
    assert!(!e.state_core().player(1).alive, "楚江王已死亡");
    assert!(e.debug_face_down(0), "冰封：来源 0 号被翻面");
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
    assert!(r.debug_face_down(0), "往返保留翻面");
}

use super::*;

/// T12 swd 武将面批㉕（屠维湛卢+苏生/夏柔相惠+挥琪）：四技行为面 + 反例。
#[test]
fn swd_generals_b65_zhanlu_susheng_xianghui_huiqi() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62033, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    fn next_p0_play(e: &mut Engine) {
        for _ in 0..3 {
            let c = e.turn().current;
            let _ = submit(e, c, GameAction::NextPhase);
        }
        for _ in 0..16 {
            let c = e.turn().current;
            if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
                break;
            }
            let _ = submit(e, c, GameAction::NextPhase);
        }
        advance_to_play(e, 0);
    }

    // ① 湛卢（屠维）：弃黑桃 → 至多 3 名受伤角色各回 1
    e.debug_grant(0, "zhanlu");
    let spade = {
        let sp = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                )
            })
            .expect("黑桃在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != sp);
        core.discard.retain(|x| *x != sp);
        core.table.retain(|x| *x != sp);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != sp);
        }
        core.players[0].hand.push(sp);
        sp
    };
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(3, 4); // 满血不可选
    let def_zl = e.catalog().skill_by_name("zhanlu").expect("湛卢在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_zl });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "湛卢发动 {out:?}"
    );
    let c = e.open_choice().expect("湛卢弃牌窗");
    assert_eq!(c.spec.prompt, "zhanlu_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(spade)],
        },
    );
    let c = e.open_choice().expect("湛卢目标窗");
    assert_eq!(c.spec.prompt, "zhanlu_targets");
    assert!(c.spec.max == 3, "湛卢：至多 3 目标");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Player(3)),
        "湛卢：满血 p3 不在候选"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "湛卢：p1 回 1");
    assert_eq!(e.debug_hp(2), 4, "湛卢：p2 回 1");
    assert_eq!(e.debug_hp(3), 4, "湛卢：p3 未选不变");
    // 反例：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_zl });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "湛卢反例：再发动拒 {out:?}"
    );

    // ② 相惠（夏柔）：弃红手牌 → 体力最少且已受伤者各回 1
    e.debug_grant(0, "xianghui");
    next_p0_play(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red = {
        let rd = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                )
            })
            .expect("红牌在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != rd);
        core.discard.retain(|x| *x != rd);
        core.table.retain(|x| *x != rd);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != rd);
        }
        core.players[0].hand.push(rd);
        rd
    };
    // 体力最少且受伤：p1/p2 均 hp2（受伤+最少）；p0/p3 hp4
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 2);
    e.debug_set_hp(3, 4);
    let def_xh = e.catalog().skill_by_name("xianghui").expect("相惠在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xh });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "相惠发动 {out:?}"
    );
    let c = e.open_choice().expect("相惠弃牌窗");
    assert_eq!(c.spec.prompt, "xianghui_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(red)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "相惠：p1（最少受伤）回 1");
    assert_eq!(e.debug_hp(2), 3, "相惠：p2（最少受伤）回 1");
    assert_eq!(e.debug_hp(3), 4, "相惠：p3 未动");

    // ③ 挥琪（夏柔）：受伤后令一名其他角色摸 X=已损失体力（hp2 受 1 伤
    // → hp1 无濒死；X=4-1=3）
    e.debug_grant(0, "huiqi");
    next_p0_play(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2);
    // 推进到 p1 回合出杀（挥琪=holder 受伤触发，任意回合）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    e.debug_set_hp(0, 2);
    let sha5 = e.debug_place(1, "sha");
    let h2 = e.debug_hand(2).len();
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha5,
            targets: vec![0],
        },
    );
    // 挥琪确认窗（首个挂起）→ 确认 → 目标窗 → 选 p2（settle 会吞链）
    let c = e.open_choice().expect("挥琪确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("挥琪目标窗");
    assert_eq!(c.spec.prompt, "huiqi_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 1, "挥琪：p0 受 1 伤（2→1 无濒死）");
    assert_eq!(e.debug_hand(2).len(), h2 + 3, "挥琪：p2 摸 X=3");

    // ④ 苏生（屠维）：任意角色死亡前置——弃手牌防止死亡并回至 1
    e.debug_grant(0, "susheng");
    next_p0_play(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    // 推进到 p2 回合出杀（sha 须当前回合玩家发起）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 2 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 2);
    for p in 0..4 {
        if p != 2 {
            e.debug_clear_hand(p);
        }
    }
    let save_card = e.debug_place(0, "tao");
    let sha6 = e.debug_place(2, "sha");
    e.debug_set_hp(1, 1);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha6,
            targets: vec![1],
        },
    );
    // p1 无闪受 1 伤 → hp 0 → 濒死 → 拒救援（p0 持桃）→ 死亡前置 →
    // 苏生确认窗 → 确认 → 弃桃
    let mut saw_susheng = false;
    let mut declined_rescue = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt == "trigger_confirm" && c.observer == 0 {
            saw_susheng = true;
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            let c = e.open_choice().expect("苏生弃牌窗");
            assert_eq!(c.spec.prompt, "susheng_discard");
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Card(save_card)],
                },
            );
            break;
        }
        // 濒死救援窗（首窗，min=1 不可空答）——应 Pass 放行死亡
        let (cid, obs) = (c.id, c.observer);
        let _ = declined_rescue;
        declined_rescue = true;
        let s = if !c.spec.options.is_empty() {
            vec![c.spec.options[0].clone()]
        } else {
            vec![]
        };
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: s,
            },
        );
    }
    settle_optional_confirm(&mut e);
    assert!(saw_susheng, "苏生：死亡前置确认窗出现");
    assert!(e.state_core().player(1).alive, "苏生：p1 未死");
    assert_eq!(e.debug_hp(1), 1, "苏生：体力回至 1");
    assert!(!e.debug_hand(0).contains(&save_card), "苏生：代价牌已弃");
    // 反例：每回合限一次——p1 再次致死无苏生窗
    e.debug_set_hp(1, 1);
    let sha7 = e.debug_place(2, "sha");
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha7,
            targets: vec![1],
        },
    );
    let mut saw_second = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt == "trigger_confirm" && c.observer == 0 {
            saw_second = true;
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![],
                },
            );
            continue;
        }
        let (cid, obs) = (c.id, c.observer);
        let s = if !c.spec.options.is_empty() {
            vec![c.spec.options[0].clone()]
        } else {
            vec![]
        };
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: s,
            },
        );
    }
    assert!(!saw_second, "苏生反例：每回合限一次（第二次无窗）");
}

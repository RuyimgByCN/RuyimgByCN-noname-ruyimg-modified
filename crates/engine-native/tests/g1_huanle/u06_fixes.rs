use super::*;

#[test]
fn u06_kuwu_no_dup_log_use_as_and_notrigger() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};

    let huanle_pack = huanle_pack_from_repo();
    let d3_json = std::fs::read_to_string(
        repo_extensions_root().join("d3-upstream-noname/assets/character.json"),
    )
    .expect("read d3");
    let d3_pack = bridge("d3-upstream-noname", &d3_json).expect("bridge d3");
    let packs: Vec<&ExtensionPack> = vec![&huanle_pack, &d3_pack];

    // 1. 弃手牌不得重复记录 CardDiscarded；Choice A(手牌) vs Choice B(装备) + save/restore + AI PolicyInput 公开信息决策
    {
        let mut e = Engine::new_with_extensions(3696, 3, &packs).expect("建局");
        advance_to_play(&mut e, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let eq_monkey = e.debug_place(1, "hnl_monkey_3");
        assert!(matches!(
            submit(&mut e, 1, GameAction::PlayCard { card: eq_monkey, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        let hand_tao = e.debug_place(1, "tao");
        let _secret_p2 = e.debug_place(2, "jiu");
        let sha = e.debug_place(0, "sha");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
            CommandOutcome::Pending { .. }
        ));
        let kw = e.open_choice().expect("kuwu_discard").clone();
        assert_eq!(kw.spec.prompt, "kuwu_discard");

        // AI 仅用公开信息（更改 2 号不可见手牌不改变 AI 选择）并偏好保留桃、弃置猴子
        let planner = BoundedPlanner::new(16);
        let ai_plan = e.plan_action(1, &planner);
        assert_eq!(
            ai_plan.action,
            GameAction::Answer {
                choice_id: kw.id,
                selection: vec![ChoiceOption::Card(eq_monkey)],
            }
        );
        {
            let mut e_blind = e.clone();
            e_blind.debug_clear_hand(2);
            let _ = e_blind.debug_place(2, "tao");
            assert_eq!(e_blind.plan_action(1, &planner).action, ai_plan.action);
        }

        let bytes = encode_archive(&e.save());
        let mut e_equip =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();
        let mut e_hand =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();
        let log_len = e.log().len();
        let out_orig = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: kw.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        let out_rest = submit(
            &mut e_hand,
            1,
            GameAction::Answer {
                choice_id: kw.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        match (&out_orig, &out_rest) {
            (
                CommandOutcome::Accepted { state_hash: h1 },
                CommandOutcome::Accepted { state_hash: h2 },
            ) => assert_eq!(h1, h2),
            _ => panic!("期望双方 Accepted: out_orig={out_orig:?}, out_rest={out_rest:?}"),
        }
        let dup_hand_logs = e.log()[log_len..]
            .iter()
            .filter(|ev| {
                matches!(ev, engine_native::LogEvent::CardDiscarded { player: 1, cards } if cards == &vec![hand_tao])
            })
            .count();
        assert_eq!(dup_hand_logs, 1, "苦无弃手牌不得写入两条重复 CardDiscarded 日志");

        // Choice B: 弃装备也只记一条 CardDiscarded
        let log_eq = e_equip.log().len();
        assert!(matches!(
            submit(
                &mut e_equip,
                1,
                GameAction::Answer {
                    choice_id: kw.id,
                    selection: vec![ChoiceOption::Card(eq_monkey)],
                }
            ),
            CommandOutcome::Accepted { .. }
        ));
        let eq_logs = e_equip.log()[log_eq..]
            .iter()
            .filter(|ev| {
                matches!(ev, engine_native::LogEvent::CardDiscarded { player: 1, cards } if cards == &vec![eq_monkey])
            })
            .count();
        assert_eq!(eq_logs, 1);
    }

    // 2. 转化杀（武圣+红桃）与 0 卡虚拟杀（dddxiaxing use_as=Sha）均应触发苦无（含 save/restore）
    {
        let mut e = Engine::new_with_extensions(3697, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
        e.debug_grant(0, "武圣");
        let red_tao = e
            .state_core()
            .deck
            .iter()
            .copied()
            .find(|&c| {
                e.catalog().card_kind(c) == CardKind::Tao
                    && matches!(
                        e.catalog().cards[c as usize].suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
            })
            .expect("牌库中应有红色桃");
        {
            let core = e.state_core_mut();
            core.deck.retain(|c| *c != red_tao);
            core.players[0].hand.push(red_tao);
        }
        let victim_card = e.debug_place(1, "jiu");
        let wusheng_out = submit(&mut e, 0, GameAction::PlayCard { card: red_tao, targets: vec![1] });
        assert!(
            matches!(wusheng_out, CommandOutcome::Pending { .. }),
            "武圣转化杀造成伤害后必须挂起苦无: {wusheng_out:?}"
        );
        let kw = e
            .open_choice()
            .expect("武圣转化杀造成伤害后必须触发苦无 kuwu_discard")
            .clone();
        assert_eq!(kw.spec.prompt, "kuwu_discard");
        let bytes = encode_archive(&e.save());
        let mut restored =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();
        let a = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: kw.id,
                selection: vec![ChoiceOption::Card(victim_card)],
            },
        );
        let b = submit(
            &mut restored,
            1,
            GameAction::Answer {
                choice_id: kw.id,
                selection: vec![ChoiceOption::Card(victim_card)],
            },
        );
        match (&a, &b) {
            (
                CommandOutcome::Accepted { state_hash: h1 },
                CommandOutcome::Accepted { state_hash: h2 },
            ) => assert_eq!(h1, h2),
            _ => panic!("期望双方 Accepted: a={a:?}, b={b:?}"),
        }
    }
    {
        let p9_pack =
            content_native_loader::decision34_pack9_for_coexistence(&[&huanle_pack, &d3_pack]);
        let xiaxing_packs: Vec<&ExtensionPack> = vec![&huanle_pack, &d3_pack, &p9_pack];
        let mut e = Engine::new_with_extensions(3698, 3, &xiaxing_packs).expect("建局");
        e.debug_grant(0, "dddxiaxing");
        let top_card = e.debug_place(0, "shan");
        // 从第 1 轮他人结束推进到 0 号 Prepare 阶段以触发侠行获取「侠」标记
        e.debug_set_turn(1, Phase::End, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.state_core_mut().players[0].hand.push(top_card);
        let prep_out = submit(&mut e, 1, GameAction::NextPhase);
        assert!(matches!(prep_out, CommandOutcome::Pending { .. }));
        let c_xx = e.open_choice().expect("dddxiaxing_card").clone();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c_xx.id,
                selection: vec![ChoiceOption::Card(top_card)],
            },
        );
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(1);
        let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
        e.debug_clear_hand(0);
        let _victim_card = e.debug_place(1, "tao");
        let xiaxing_def = e.catalog().skill_by_name("dddxiaxing").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: xiaxing_def });
        let pick_t = e.open_choice().expect("侠行选目标").clone();
        let xiaxing_hit = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: pick_t.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        assert!(
            matches!(xiaxing_hit, CommandOutcome::Pending { .. }),
            "0卡虚拟杀(use_as=Sha)造成伤害后必须挂起苦无: {xiaxing_hit:?}"
        );
        let kw = e
            .open_choice()
            .expect("0卡虚拟杀(use_as=Sha)造成伤害后必须触发苦无 kuwu_discard")
            .clone();
        assert_eq!(kw.spec.prompt, "kuwu_discard");
    }

    // 3. DamageSource 时序：1 血目标无桃致死不得在濒死前弹苦无；有桃自救存活后方触发苦无
    {
        let mut e_die = Engine::new_with_extensions(3699, 3, &packs).expect("建局");
        advance_to_play(&mut e_die, 0);
        for p in 0..3 {
            e_die.debug_clear_hand(p);
        }
        let kuwu = e_die.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        submit(&mut e_die, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
        e_die.debug_set_hp(1, 1);
        let _victim_other = e_die.debug_place(1, "jiu");
        let sha = e_die.debug_place(0, "sha");
        submit(&mut e_die, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        // 自动跑完无桃可救的濒死流程（若有救人询问全部放弃）
        for _ in 0..32 {
            let Some(c) = e_die.open_choice().cloned() else {
                break;
            };
            assert_ne!(
                c.spec.prompt, "kuwu_discard",
                "1 号濒死死亡不得触发苦无 kuwu_discard（应位于 Dying 之后的 DamageSource）"
            );
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let out = submit(&mut e_die, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "濒死流程回答被拒 ({}): {out:?}",
                c.spec.prompt
            );
        }
        assert!(!e_die.debug_alive(1), "1 号应阵亡");

        let mut e_save = Engine::new_with_extensions(3700, 3, &packs).expect("建局");
        advance_to_play(&mut e_save, 0);
        for p in 0..3 {
            e_save.debug_clear_hand(p);
        }
        let kuwu = e_save.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        submit(&mut e_save, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
        e_save.debug_set_hp(1, 1);
        let save_tao = e_save.debug_place(1, "tao");
        let _extra = e_save.debug_place(1, "jiu");
        let sha = e_save.debug_place(0, "sha");
        submit(&mut e_save, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let dying_ask = e_save.open_choice().expect("应先进入濒死求桃，而非先弹苦无").clone();
        assert_ne!(dying_ask.spec.prompt, "kuwu_discard", "苦无必须在 Dying 之后");
        submit(
            &mut e_save,
            dying_ask.observer,
            GameAction::Answer {
                choice_id: dying_ask.id,
                selection: vec![ChoiceOption::Card(save_tao)],
            },
        );
        let kw = e_save.open_choice().expect("被救存活后 DamageSource 应触发苦无").clone();
        assert_eq!(kw.spec.prompt, "kuwu_discard");
    }

    // 4. `_notrigger`（settlement.notrigger = true）抑制苦无触发（对照 false 触发）
    for notrigger in [true, false] {
        let mut e = Engine::new_with_extensions(3701, 3, &packs).expect("建局");
        e.debug_grant(0, "huanle_kuwu");
        let sha = e.debug_place(0, "sha");
        let _victim_card = e.debug_place(1, "jiu");
        let mut core = e.save().core;
        let mut res = engine_native::resolution::Resolution::new(20000);
        let mut logs = Vec::new();
        let mut d = engine_native::machine::damage_data(Some(0), 1, 1);
        d.card = Some(sha);
        d.settlement.notrigger = notrigger;
        let _ = engine_native::resolution::push_frame(&mut res, engine_native::EventData::Damage(d));
        let mut saw_pending = false;
        for _ in 0..200 {
            match engine_native::resolution::run(&mut core, &mut res, e.catalog(), &mut logs, Some(1)) {
                engine_native::resolution::StepOutcome::Done => break,
                engine_native::resolution::StepOutcome::Yielded => continue,
                engine_native::resolution::StepOutcome::Pending => {
                    saw_pending = true;
                    break;
                }
                other => panic!("unexpected outcome: {other:?}"),
            }
        }
        assert_eq!(
            saw_pending,
            !notrigger,
            "notrigger={notrigger} 时苦无触发应为 {}",
            !notrigger
        );
    }
}

#[test]
fn u06_mianju_protects_all_normal_turnover_paths() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};

    let huanle_pack = huanle_pack_from_repo();
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("read swd");
    let swd_pack = bridge("shahai-swd-upstream-noname", &swd_json).expect("bridge swd");
    let packs: Vec<&ExtensionPack> = vec![&huanle_pack, &swd_pack];

    // 1. 风雪刃（sha/resume.rs）+ 急冻冷冻（swd_lingjian_cards.rs）：有面具取消 TurnOverBefore，无面具正常翻面；含 Choice A/B、AI PolicyInput 与存档恢复一致性
    for with_mianju in [true, false] {
        let mut e = Engine::new_with_extensions(3710, 3, &packs).expect("建局");
        advance_to_play(&mut e, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        if with_mianju {
            let mj = e.debug_place(1, "hnl_mianju_5");
            assert!(matches!(
                submit(&mut e, 1, GameAction::PlayCard { card: mj, targets: vec![] }),
                CommandOutcome::Accepted { .. }
            ));
        }
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let fxr_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == CardKind::Fengxueren)
            .expect("fengxueren 在目录")
            .name;
        let fxr = e.debug_move_named_card_to_hand(0, fxr_name);
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: fxr, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        let _secret_p2 = e.debug_place(2, "tao");
        let sha = e.debug_place(0, "sha");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
            CommandOutcome::Pending { .. }
        ));
        let fxr_ask = e.open_choice().expect("应询问风雪刃").clone();
        // AI 仅用公开信息决策（改 2 号手牌不影响决策）
        let planner = BoundedPlanner::new(16);
        let plan_a = e.plan_action(0, &planner).action;
        {
            let mut e_blind = e.clone();
            e_blind.debug_clear_hand(2);
            let _ = e_blind.debug_place(2, "jiu");
            assert_eq!(e_blind.plan_action(0, &planner).action, plan_a);
        }
        // 中途存档恢复一致（Choice A: Confirm 风雪刃翻面）
        let bytes = encode_archive(&e.save());
        let mut e_restored =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();
        let mut e_decline = e.clone();
        let out_a = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: fxr_ask.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        let out_b = submit(
            &mut e_restored,
            0,
            GameAction::Answer {
                choice_id: fxr_ask.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        match (&out_a, &out_b) {
            (
                CommandOutcome::Accepted { state_hash: h1 },
                CommandOutcome::Accepted { state_hash: h2 },
            ) => assert_eq!(h1, h2),
            _ => panic!("风雪刃确认后应 Accepted: out_a={out_a:?}, out_b={out_b:?}"),
        }
        assert_eq!(
            e.debug_face_down(1),
            !with_mianju,
            "风雪刃翻面：with_mianju={with_mianju} 时 face_down 应为 {}",
            !with_mianju
        );
        // Choice B: 取消风雪刃照常造成 1 点伤害
        let hp1 = e_decline.debug_hp(1);
        assert!(matches!(
            submit(
                &mut e_decline,
                0,
                GameAction::Answer {
                    choice_id: fxr_ask.id,
                    selection: vec![],
                }
            ),
            CommandOutcome::Accepted { .. }
        ));
        assert_eq!(e_decline.debug_hp(1), hp1 - 1);

        // 紧急冷冻（swd_lingjian_cards.rs）：有面具取消 TurnOverBefore 且不挂 jileng_active
        e.debug_set_face_down(1, false);
        let jl_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name.contains("jinjilengdong"))
            .expect("jinjilengdong 在目录")
            .name;
        let jl = e.debug_move_named_card_to_hand(0, jl_name);
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: jl, targets: vec![1] }),
            CommandOutcome::Accepted { .. }
        ));
        assert_eq!(
            e.debug_face_down(1),
            !with_mianju,
            "紧急冷冻：with_mianju={with_mianju} 时 face_down 应为 {}",
            !with_mianju
        );
        assert_eq!(
            e.state_core().player(1).jileng_active,
            !with_mianju,
            "面具取消紧急冷冻翻面后不得冻结目标"
        );
    }

    // 2. 拘魂（kernel/phase/end.rs）+ 冰封（standard/skills/bingfeng.rs）+ VmViewAsCost::TurnOver（handlers.rs/events.rs）
    for with_mianju in [true, false] {
        // 拘魂：1 号结束阶段试图翻 0 号面
        let mut e = Engine::new_with_extensions(3711, 3, &[&huanle_pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        if with_mianju {
            let mj = e.debug_place(0, "hnl_mianju_5");
            submit(&mut e, 0, GameAction::PlayCard { card: mj, targets: vec![] });
        }
        e.debug_grant(1, "拘魂");
        advance_to_play(&mut e, 1);
        submit(&mut e, 1, GameAction::NextPhase); // Play -> Discard
        submit(&mut e, 1, GameAction::NextPhase); // Discard -> End (triggers 拘魂)
        assert_eq!(
            e.debug_face_down(0),
            !with_mianju,
            "拘魂：with_mianju={with_mianju} 时 0 号 face_down 应为 {}",
            !with_mianju
        );

        // 冰封：0 号击杀 1 号后触发 1 号冰封试图翻 0 号面
        let mut e_bf = Engine::new_with_extensions(3712, 3, &[&huanle_pack]).expect("建局");
        advance_to_play(&mut e_bf, 0);
        for p in 0..3 {
            e_bf.debug_clear_hand(p);
        }
        if with_mianju {
            let mj = e_bf.debug_place(0, "hnl_mianju_5");
            submit(&mut e_bf, 0, GameAction::PlayCard { card: mj, targets: vec![] });
        }
        e_bf.debug_grant(1, "冰封");
        e_bf.debug_set_hp(1, 1);
        let sha = e_bf.debug_place(0, "sha");
        submit(&mut e_bf, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        for _ in 0..16 {
            let Some(c) = e_bf.open_choice().cloned() else {
                break;
            };
            submit(&mut e_bf, c.observer, GameAction::Answer { choice_id: c.id, selection: vec![] });
        }
        assert!(!e_bf.debug_alive(1));
        assert_eq!(
            e_bf.debug_face_down(0),
            !with_mianju,
            "冰封：with_mianju={with_mianju} 时 0 号 face_down 应为 {}",
            !with_mianju
        );
    }

    // VmViewAsCost::TurnOver：装备面具时 TurnOverBefore 取消翻面（无面具则正常翻面）
    {
        let jiushi_src = include_str!("../../../../content/original/ported/yijiang/jiushi1.skill");
        let pkg = content_compiler::compile_source_named("jiushi1", jiushi_src).expect("compile jiushi1");
        let vm_out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("bridge jiushi1");
        let vm_pack = ExtensionPack {
            def: engine_native::content::extension::ExtensionDef {
                name: "test-turnover-va",
                version: 1,
                description: "test VmViewAsCost::TurnOver",
            },
            generals: Vec::new(),
            cards: Vec::new(),
            host_skills: Vec::new(),
            vm_skills: vm_out.decls,
            init_hook: engine_native::content::extension::InitHook::None,
            enabled: true,
        };
        let vm_packs: Vec<&ExtensionPack> = vec![&huanle_pack, &vm_pack];
        for with_mianju in [true, false] {
            let mut e = Engine::new_with_extensions(3713, 3, &vm_packs).expect("建局");
            advance_to_play(&mut e, 0);
            for p in 0..3 {
                e.debug_clear_hand(p);
            }
            if with_mianju {
                let mj = e.debug_place(0, "hnl_mianju_5");
                submit(&mut e, 0, GameAction::PlayCard { card: mj, targets: vec![] });
            }
            e.debug_grant(0, "jiushi1");
            let jiu_action = e
                .legal_actions(0)
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        GameAction::PlayCard { card, targets }
                            if targets.is_empty() && !e.state_core().player(0).hand.contains(card)
                    )
                })
                .expect("jiushi1 合法动作");
            let out = submit(&mut e, 0, jiu_action);
            assert!(
                matches!(out, CommandOutcome::Accepted { .. }),
                "VmViewAsCost::TurnOver 转化酒应结算: {out:?}"
            );
            assert_eq!(
                e.debug_face_down(0),
                !with_mianju,
                "VmViewAsCost::TurnOver：with_mianju={with_mianju} 时 face_down 应为 {}",
                !with_mianju
            );
        }
    }
}

/// `monkey`（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:5-31`）补遗验收：
/// 1. 濒死救援【桃】触发 `huanle_monkey_confirm`：Confirm 弃猴夺桃并停在濒死顺位下一救援者、Pass 放行救回；
///    含存档恢复一致性、AI `BoundedPlanner` 公开态度决策、及实体转化【桃】触发；
/// 2. 濒死负例：濒死者本人持猴自救（`event.player === player`）、濒死使用【酒】/【护符】及 0 卡虚拟【桃】均不触发；
/// 3. `PlayCard` 用桃时猴子选择 Pass：仍须发射 `CardUsed`（触发 `_shengke` 等 `useCardToBegin` 监听与 `CardPlayed` 日志）。
#[test]
fn u06_monkey_dying_rescue_and_decline_card_used() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::state::{GameMode, Role};

    let huanle_pack = huanle_pack_from_repo();
    let wuxing_json = std::fs::read_to_string(
        repo_extensions_root().join("wuxing-upstream-liuli/assets/cards.json"),
    )
    .expect("read wuxing");
    let wuxing_pack = bridge("wuxing-upstream-liuli", &wuxing_json).expect("bridge wuxing");
    let vm_src = r#"
skill test_va_none_dying_tao(99060011) {
    view_as tao any any from none cost lose_max_hp_1 uses_per_turn 1;
}
"#;
    let vm_pkg = content_compiler::compile_source_named("monkey_dying_vm", vm_src).expect("compile vm");
    let vm_out = content_native_loader::bridge_vm_skill_decls(&vm_pkg).expect("bridge vm");
    let vm_pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "u06-monkey-dying-vm",
            version: 1,
            description: "monkey dying vm helpers",
        },
        generals: Vec::new(),
        cards: vec![engine_native::catalog::CardDef::basic("test_hufu", CardKind::Hufu)],
        host_skills: Vec::new(),
        vm_skills: vm_out.decls,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let packs: Vec<&ExtensionPack> = vec![&huanle_pack, &wuxing_pack, &vm_pack];

    // 1. 濒死求桃：seat 1 濒死，seat 0 装备猴子，seat 2 用桃相救、seat 3 亦持有桃
    {
        let mut e = Engine::new_with_extensions(3720, 4, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        e.debug_set_hp(1, 1);
        let tao2 = e.debug_place(2, "tao");
        let tao3 = e.debug_place(3, "tao");
        let sha = e.debug_place(0, "sha");
        let out_sha = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(matches!(out_sha, CommandOutcome::Pending { .. }));

        // 先轮到 2 号回答濒死求桃
        let save_p2 = e.open_choice().expect("p2 dying save").clone();
        assert_eq!(save_p2.observer, 2);
        let out_save2 = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: save_p2.id,
                selection: vec![ChoiceOption::Card(tao2)],
            },
        );
        assert!(matches!(out_save2, CommandOutcome::Pending { .. }));

        // 0 号猴子确认门弹出
        let mk_ask = e.open_choice().expect("huanle_monkey_confirm on dying tao").clone();
        assert_eq!(mk_ask.spec.prompt, "huanle_monkey_confirm");
        assert_eq!(mk_ask.observer, 0);

        // AI BoundedPlanner 公开信息决策：1 号为主公时，反贼应确认偷桃、忠臣应放行；且修改 3 号暗身份不影响决策
        let planner = BoundedPlanner::new(16);
        {
            let mut e_rebel = e.clone();
            e_rebel.state_core_mut().mode = GameMode::Identity {
                roles: vec![Role::Rebel, Role::Lord, Role::Loyalist, Role::Traitor],
            };
            let act_a = e_rebel.plan_action(0, &planner).action;
            assert_eq!(
                act_a,
                GameAction::Answer {
                    choice_id: mk_ask.id,
                    selection: vec![ChoiceOption::Confirm],
                }
            );
            let mut e_rebel2 = e_rebel.clone();
            e_rebel2.state_core_mut().mode = GameMode::Identity {
                roles: vec![Role::Rebel, Role::Lord, Role::Loyalist, Role::Rebel],
            };
            assert_eq!(e_rebel2.plan_action(0, &planner).action, act_a);

            let mut e_loyal = e.clone();
            e_loyal.state_core_mut().mode = GameMode::Identity {
                roles: vec![Role::Loyalist, Role::Lord, Role::Rebel, Role::Traitor],
            };
            assert_eq!(
                e_loyal.plan_action(0, &planner).action,
                GameAction::Answer {
                    choice_id: mk_ask.id,
                    selection: vec![ChoiceOption::Pass],
                }
            );
        }

        // 挂起中点存档恢复 + Choice A(Confirm 偷桃) vs Choice B(Pass 放行救回)
        let bytes = encode_archive(&e.save());
        let mut e_restored =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();
        let mut e_pass = e.clone();

        // Choice B: Pass -> seat 1 获救回至 1 血、tao2 入弃牌堆、猴子留场
        let out_pass = submit(
            &mut e_pass,
            0,
            GameAction::Answer {
                choice_id: mk_ask.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
        assert!(matches!(out_pass, CommandOutcome::Accepted { .. }), "Pass={out_pass:?}");
        assert_eq!(e_pass.debug_hp(1), 1);
        assert!(!e_pass.state_core().player(1).dying);
        assert!(e_pass.debug_equips(0).contains(&monkey));
        assert!(e_pass.save().core.discard.contains(&tao2));

        // Choice A: Confirm -> tao2 被 0 号夺走，猴子入弃牌堆，seat 1 仍为 0 血濒死并询问下一顺位 3 号
        let log_len = e.log().len();
        let out_a = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk_ask.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        let out_r = submit(
            &mut e_restored,
            0,
            GameAction::Answer {
                choice_id: mk_ask.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        match (&out_a, &out_r) {
            (CommandOutcome::Pending { .. }, CommandOutcome::Pending { .. }) => {
                assert_eq!(e.state_hash(), e_restored.state_hash(), "濒死偷桃存档恢复状态哈希一致");
            }
            _ => panic!("偷桃后 1 号仍濒死应转问 3 号: a={out_a:?}, r={out_r:?}"),
        }
        assert_eq!(&e.log()[log_len..], e_restored.log());
        assert_eq!(e.debug_hp(1), 0);
        assert!(e.state_core().player(1).dying);
        assert!(e.debug_hand(0).contains(&tao2));
        assert!(!e.debug_equips(0).contains(&monkey));
        let save_p3 = e.open_choice().expect("next responder seat 3").clone();
        assert_eq!(save_p3.observer, 3);
        let out_save3 = submit(
            &mut e,
            3,
            GameAction::Answer {
                choice_id: save_p3.id,
                selection: vec![ChoiceOption::Card(tao3)],
            },
        );
        assert!(matches!(out_save3, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hp(1), 1);
    }

    // 1b. 实体转化【桃】（急救红牌当桃救人）亦触发猴子偷桃
    {
        let mut e = Engine::new_with_extensions(3721, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
        e.debug_grant(2, "急救");
        e.debug_set_hp(1, 1);
        let red_shan = e
            .state_core()
            .deck
            .iter()
            .copied()
            .find(|&c| {
                e.catalog().card_kind(c) == CardKind::Shan
                    && matches!(
                        e.catalog().cards[c as usize].suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
            })
            .expect("红闪");
        {
            let core = e.state_core_mut();
            core.deck.retain(|c| *c != red_shan);
            core.players[2].hand.push(red_shan);
        }
        let sha = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let save_p2 = e.open_choice().expect("p2 急救").clone();
        assert_eq!(save_p2.observer, 2);
        submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: save_p2.id,
                selection: vec![ChoiceOption::Card(red_shan)],
            },
        );
        let mk = e.open_choice().expect("急救红牌当桃应触发猴子偷桃").clone();
        assert_eq!(mk.spec.prompt, "huanle_monkey_confirm");
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        assert!(e.debug_hand(0).contains(&red_shan), "实体转化桃载体进入猴子持有者手牌");
    }

    // 2. 濒死负例：本人持猴自救不触发；濒死自救使用【酒】【护符】及 0 卡虚拟桃不触发
    {
        let mut e = Engine::new_with_extensions(3722, 3, &packs).expect("建局");
        advance_to_play(&mut e, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey_self = e.debug_place(1, "hnl_monkey_3");
        submit(&mut e, 1, GameAction::PlayCard { card: monkey_self, targets: vec![] });
        let tao_self = e.debug_place(1, "tao");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_set_hp(1, 1);
        let sha = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let save_p1 = e.open_choice().expect("p1 self save").clone();
        let out_self = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: save_p1.id,
                selection: vec![ChoiceOption::Card(tao_self)],
            },
        );
        assert!(
            matches!(out_self, CommandOutcome::Accepted { .. }),
            "本人持猴自救不得触发自己猴子: {out_self:?}"
        );
        assert_eq!(e.debug_hp(1), 1);
        assert!(e.debug_equips(1).contains(&monkey_self));

        // 他人持猴，濒死者用酒/护符/0卡虚拟桃均不得触发他人猴子
        let mut e = Engine::new_with_extensions(37220, 3, &packs).expect("建局");
        advance_to_play(&mut e, 2);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey_other = e.debug_place(2, "hnl_monkey_3");
        submit(&mut e, 2, GameAction::PlayCard { card: monkey_other, targets: vec![] });
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_grant(0, "咆哮");
        for kind_name in ["jiu", "test_hufu"] {
            e.debug_set_hp(1, 1);
            let card = e.debug_place(1, kind_name);
            let sha2 = e.debug_place(0, "sha");
            let hit = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
            assert!(matches!(hit, CommandOutcome::Pending { .. }), "{kind_name} sha2={hit:?}");
            let ask = e.open_choice().expect("dying").clone();
            let out = submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Card(card)],
                },
            );
            assert!(
                matches!(out, CommandOutcome::Accepted { .. }),
                "{kind_name} 不是桃不得触发猴子: {out:?}"
            );
            assert_eq!(e.debug_hp(1), 1);
            assert!(e.debug_equips(2).contains(&monkey_other));
        }

        e.debug_grant(1, "test_va_none_dying_tao");
        e.debug_set_hp(1, 1);
        let sha3 = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha3, targets: vec![1] });
        let ask = e.open_choice().expect("0卡虚拟桃").clone();
        let va_opt = ask
            .spec
            .options
            .iter()
            .copied()
            .find(|o| matches!(o, ChoiceOption::Card(c) if e.catalog().card_kind(*c) == CardKind::Tao || !e.debug_hand(1).contains(c)))
            .expect("0卡虚拟桃选项");
        let out_va = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![va_opt],
            },
        );
        assert!(
            matches!(out_va, CommandOutcome::Accepted { .. }),
            "0卡虚拟桃不得触发猴子: {out_va:?}"
        );
        assert!(e.debug_equips(2).contains(&monkey_other));
    }

    // 3. PlayCard 出【桃】被猴子 Pass 放行后，须发射 CardUsed（触发 _shengke、CardPlayed 日志与 cards_used_play_phase 计数）
    {
        let mut e = Engine::new_with_extensions(3723, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        e.debug_set_hp(1, 2);
        e.state_core_mut().players[1].wunature = Some(4); // 土
        let tao = e.debug_place(1, "tao");
        e.state_core_mut().card_natures.insert(tao, 3); // 火生土
        let used_before = e.turn().cards_used_play_phase;
        let log_before = e.log().len();

        let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let mk = e.open_choice().expect("huanle_monkey_confirm").clone();
        let out_pass = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
        assert!(matches!(out_pass, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hp(1), 3, "Pass 放行后正常回复 1 血");
        assert_eq!(e.debug_hand(1).len(), 1, "Pass 放行后发射 CardUsed 须触发火生土 _shengke 摸 1 牌");
        assert_eq!(e.turn().cards_used_play_phase, used_before + 1, "Pass 放行须计入出牌阶段已用牌数");
        assert!(
            e.log()[log_before..].iter().any(|ev| matches!(ev, engine_native::LogEvent::CardPlayed { card, player: 1, .. } if *card == tao)),
            "Pass 放行须记录 CardPlayed"
        );
    }

    // 4. PlayCard 偷走【桃】与濒死救援使用【桃】均须记录 useCard 历史计数（total_used / cards_used_*）；
    // 且濒死救援使用【桃】须依 useCardToBegin 触发 _shengke（在猴子夺桃前，于濒死者身上生摸/克弃）
    {
        // 4a: PlayCard Confirm steal -> 1 号 total_used / cards_used_play_phase / cards_used_turn / bte_used_play_phase 仍 +1，且记 CardPlayed，不回血
        let mut e = Engine::new_with_extensions(3724, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        e.debug_set_hp(1, 2);
        let tao = e.debug_place(1, "tao");
        let total_before = e.state_core().player(1).total_used;
        let play_before = e.turn().cards_used_play_phase;
        let turn_before = e.turn().cards_used_turn;
        let bte_before = e.turn().bte_used_play_phase;
        let log_before = e.log().len();

        let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let mk = e.open_choice().expect("huanle_monkey_confirm").clone();
        let out_steal = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        assert!(matches!(out_steal, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hp(1), 2, "被猴子偷走桃不得回血");
        assert_eq!(e.state_core().player(1).total_used, total_before + 1, "上游 useCard step 0 须记 total_used");
        assert_eq!(e.turn().cards_used_play_phase, play_before + 1, "被偷后 cards_used_play_phase 仍 +1");
        assert_eq!(e.turn().cards_used_turn, turn_before + 1, "被偷后 cards_used_turn 仍 +1");
        assert_eq!(e.turn().bte_used_play_phase, bte_before + 1, "被偷后 bte_used_play_phase 仍 +1");
        assert!(
            e.log()[log_before..].iter().any(|ev| matches!(ev, engine_native::LogEvent::CardPlayed { card, player: 1, .. } if *card == tao)),
            "被偷时原出牌动作仍记录 CardPlayed"
        );
    }
    {
        // 4b: 濒死救援【桃】带火属性、1 号（濒死者）土属性（火生土摸 1）+ 0 号装备猴子：
        // 当前回合为 0 号（座位序 [0, 1, 2]），0 号持猴先于 1 号 _shengke：
        // 问完 0 号猴子 Pass 后，1 号才触发火生土 _shengke 摸 1 并获救；2 号 total_used / cards_used_turn +1
        let mut e = Engine::new_with_extensions(3725, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
        e.debug_set_hp(1, 1);
        e.state_core_mut().players[1].wunature = Some(4); // 土
        let tao2 = e.debug_place(2, "tao");
        e.state_core_mut().card_natures.insert(tao2, 3); // 火生土
        let sha = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let save_p2 = e.open_choice().expect("p2 dying save").clone();
        let p2_total_before = e.state_core().player(2).total_used;
        let turn_used_before = e.turn().cards_used_turn;
        let out_save = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: save_p2.id,
                selection: vec![ChoiceOption::Card(tao2)],
            },
        );
        assert!(matches!(out_save, CommandOutcome::Pending { .. }));
        assert_eq!(e.debug_hand(1).len(), 0, "座位序 0 号持猴先于 1 号，猴子未放行前 1 号不得先触发 _shengke");
        let mk = e.open_choice().expect("huanle_monkey_confirm").clone();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
        assert_eq!(e.debug_hand(1).len(), 1, "猴子 Pass 后 1 号触发火生土 _shengke 摸 1");
        assert_eq!(e.debug_hp(1), 1);
        assert_eq!(e.state_core().player(2).total_used, p2_total_before + 1, "濒死用桃须计入 2 号 total_used");
        assert_eq!(e.turn().cards_used_turn, turn_used_before + 1, "濒死用桃须计入 cards_used_turn");
    }
}

/// `shoulijian` 独立复核补遗（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:36-64`）：
/// 1. 弃置手牌装备时仅写 1 条 `LogEvent::CardDiscarded`（禁止重复日志）；
/// 2. 目标 `he` 仅含非装备手牌时（`countCards("he") > 0`），仍开启可选弃牌窗（仅 `Pass`，可取消），取消受 1 伤。
#[test]
fn u06_shoulijian_no_dup_log_and_non_equip_he_prompt() {
    let huanle_pack = huanle_pack_from_repo();

    // 1. 弃手牌装备不重日志
    {
        let mut e = Engine::new_with_extensions(3726, 4, &[&huanle_pack]).expect("4人局");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let sj = e.debug_place(0, "hnl_shoulijian_1");
        let hand_eq = e.debug_move_named_card_to_hand(2, "horse_minus");
        submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![2] });
        let ask = e.open_choice().expect("host_target_discard_equip").clone();
        let log_before = e.log().len();
        let out = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(hand_eq)],
            },
        );
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        let disc_count = e.log()[log_before..]
            .iter()
            .filter(|ev| matches!(ev, engine_native::LogEvent::CardDiscarded { player: 2, cards } if cards == &vec![hand_eq]))
            .count();
        assert_eq!(disc_count, 1, "手里剑弃置手牌装备不得重复写入两条 CardDiscarded 日志");
    }

    // 2. 目标仅有非装备手牌：须弹窗（避免泄露手牌无装备），放弃后受 1 伤
    {
        let mut e = Engine::new_with_extensions(3727, 4, &[&huanle_pack]).expect("4人局");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let sj = e.debug_place(0, "hnl_shoulijian_1");
        let _non_eq = e.debug_place(2, "sha");
        let hp2 = e.debug_hp(2);
        let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![2] });
        assert!(
            matches!(out, CommandOutcome::Pending { .. }),
            "目标 he 非空但无装备时，依 card.js:48 仍应弹窗而非静默跳窗: {out:?}"
        );
        let ask = e.open_choice().expect("host_target_discard_equip").clone();
        assert_eq!(ask.observer, 2);
        assert_eq!(ask.spec.prompt, "host_target_discard_equip");
        assert!(ask.spec.allow_cancel);
        assert_eq!(ask.spec.options, vec![ChoiceOption::Pass]);
        let out2 = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![],
            },
        );
        assert!(matches!(out2, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hp(2), hp2 - 1);
    }
}

/// `mianju` 对扩展内曾直接写 `.face_down =` 的路径补齐 `TurnOverBefore` 取消守护：
/// 覆盖宿主卡（`txhj_diaohu`）、`decision34`（`mbcmfangzhu` 选项 3）及 `L25`（`new_guixin`）。
#[allow(deprecated)]
#[test]
fn u06_mianju_protects_extension_direct_turn_over_paths() {
    use content_native_loader::{
        prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
        LockedStartMode,
    };
    use rule_api::{EffectRequest, SkillId};
    use std::collections::BTreeMap;

    let ext_root = repo_extensions_root();
    let freeze_pkg = |dir_name: &str| {
        let pkg_dir = ext_root.join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap_or_else(|e| panic!("read {dir_name}/package.npack.json: {e}"));
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap_or_else(|e| panic!("read {dir_name}/{rel_path}: {e}"));
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        validate_and_freeze_raw_package(raw_json, assets)
            .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"))
    };
    let verified = prepare_locked_content_with_bridge(vec![
        freeze_pkg("huanle-upstream-noname"),
        freeze_pkg("taixu-upstream-liuli"),
        freeze_pkg("longxue-upstream-liuli"),
        freeze_pkg("zuan-upstream-liuli"),
    ])
    .expect("prepare formal packages");
    let mut e = start_locked_engine(3728, 3, LockedStartMode::Prototype, &verified)
        .expect("start locked engine");
    for p in 0..3 {
        e.debug_remove_general(p);
        e.debug_set_face_down(p, false);
    }
    let drain_choices = |e: &mut Engine| {
        for _ in 0..32 {
            let Some(c) = e.open_choice().cloned() else { break };
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        }
    };

    advance_to_play(&mut e, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mj = e.debug_place(1, "hnl_mianju_5");
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: mj, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));

    advance_to_play(&mut e, 0);
    // 1. 对照：无面具的 2 号被 txhj_diaohu 指定时正常翻至背面
    let dh_ctrl = e.debug_move_named_card_to_hand(0, "txhj_diaohu");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: dh_ctrl, targets: vec![2] }),
        CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert!(e.debug_face_down(2), "未装备漩涡面具的 2 号应被 txhj_diaohu 翻至背面");

    // 2. 宿主卡路径（intrinsic_named_ext.rs）：装备面具的 1 号须免疫 txhj_diaohu 翻面
    let dh_guarded = e.debug_move_named_card_to_hand(0, "txhj_diaohu");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: dh_guarded, targets: vec![1] }),
        CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert!(!e.debug_face_down(1), "装备漩涡面具时 txhj_diaohu 翻面必须被 TurnOverBefore 取消");

    // 3. decision34 路径（decision34.rs mbcmfangzhu 选项 3）：1 号须保持正面
    e.state_core_mut().player_mut(0).mbqianlong_marks = 80;
    e.state_core_mut().vm_aux.flags.insert((0, "mbcmfangzhu_target".to_string()), 1);
    e.state_core_mut().vm_aux.flags.insert((0, "mbcmfangzhu_option".to_string()), 3);
    let out_fz = e.debug_execute_effect(
        "mbcmfangzhu",
        0,
        EffectRequest::StorageSet {
            skill: SkillId::new(content_native_loader::MBCMFANGZHU_ID),
            key: content_native_loader::DECISION34_STORAGE_DISPATCH_KEY,
            value: content_native_loader::MBCMFANGZHU_ID,
        },
    );
    assert!(!matches!(out_fz, CommandOutcome::Rejected(_)), "{out_fz:?}");
    drain_choices(&mut e);
    assert!(!e.debug_face_down(1), "装备漩涡面具时 mbcmfangzhu 选项 3 翻面必须被 TurnOverBefore 取消");

    // 4. L25 路径（decision34_l25_rules.rs new_guixin）：1 号须保持正面
    let out_gx = e.debug_execute_effect(
        "new_guixin",
        1,
        EffectRequest::StorageSet {
            skill: SkillId::new(content_native_loader::L25_ZUAN_SKILL_ID),
            key: content_native_loader::DECISION34_STORAGE_DISPATCH_KEY,
            value: content_native_loader::L25_ZUAN_SKILL_ID,
        },
    );
    assert!(!matches!(out_gx, CommandOutcome::Rejected(_)), "{out_gx:?}");
    drain_choices(&mut e);
    assert!(!e.debug_face_down(1), "装备漩涡面具时 new_guixin 翻面必须被 TurnOverBefore 取消");
}

/// 写轮眼跨包临时技与已觉醒/禁用技能防重开验收（`欢乐卡牌/card/skill.js:70-128`、`player.js:5643-5684,11267,11288`）：
/// 1. 1 号经 `SwdLexue`（`add_skill_until_turn_end`）获得回合末临时技并在回合结束后过期；下家 0 号写轮眼开窗时不得再看到该过期临时技；
/// 2. 0 号自身持已觉醒/禁用技能 `bingfeng`（`enabled=false`）时窃取 1 号 `bingfeng`，不得把 0 号 `bingfeng` 重新激活或回合末删去。
#[test]
fn u06_xuelunyang_ignores_expired_temp_skills_and_preserves_awakened_skill() {
    let huanle_pack = huanle_pack_from_repo();
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("read swd cards.json");
    let swd_pack = content_native_loader::bridge_pack(&swd_json)
        .expect("bridge swd")
        .pack;
    let packs: Vec<&ExtensionPack> = vec![&huanle_pack, &swd_pack];

    // 1. 过期临时技不得以 enabled=false 残余供写轮眼窃取
    {
        let mut e = Engine::new_with_extensions(3730, 3, &packs).expect("建局");
        for p in 0..3 {
            e.debug_remove_general(p);
            e.debug_clear_hand(p);
        }
        advance_to_play(&mut e, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "奸雄");
        e.debug_grant(2, "制衡");
        let zhiheng = e.catalog().skill_by_name("制衡").unwrap();
        let base_skills_count = e
            .save()
            .core
            .skills
            .range((1, 0)..(1, u16::MAX))
            .filter(|((_, d), _)| !e.catalog().is_charlotte_skill(*d))
            .count();
        let xly1 = e.debug_place(1, "hnl_xuelunyang_4");
        assert!(matches!(
            submit(&mut e, 1, GameAction::PlayCard { card: xly1, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        // 推进至下一轮 1 号 TurnBegin，让 1 号用写轮眼窃取 2 号的「制衡」(expires_end_of_turn=true)
        advance_to_play(&mut e, 2);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        for _ in 0..8 {
            if let Some(c) = e.open_choice() {
                if c.spec.prompt == "huanle_xuelunyang_target" && c.observer == 1 {
                    break;
                }
            }
            let cur = e.turn().current;
            e.debug_clear_hand(cur);
            submit(&mut e, cur, GameAction::NextPhase);
        }
        let p1_tgt = e.open_choice().expect("p1 huanle_xuelunyang_target").clone();
        assert_eq!(p1_tgt.observer, 1);
        submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: p1_tgt.id,
                selection: vec![ChoiceOption::Player(2)],
            },
        );
        let p1_sk = e.open_choice().expect("p1 huanle_xuelunyang_skill").clone();
        submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: p1_sk.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        assert!(
            e.debug_has_skill(1, "制衡"),
            "1 号 TurnBegin 应经写轮眼临时获得「制衡」"
        );
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        // 推进至 0 号出牌（已走完 1 号回合末，制衡已过期）；0 号顺手牵羊取走写轮眼并装备
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let snatch = e.debug_place(0, "snatch");
        submit(&mut e, 0, GameAction::PlayCard { card: snatch, targets: vec![1] });
        if let Some(nw) = e.open_choice().filter(|c| c.spec.prompt == "nullify_window").cloned() {
            submit(&mut e, nw.observer, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
        }
        assert!(e.debug_hand(0).contains(&xly1), "0 号顺手牵羊应取回 1 号写轮眼");
        submit(&mut e, 0, GameAction::PlayCard { card: xly1, targets: vec![] });
        // 推进 0->1->2 直到下轮 0 号 TurnBegin 触发写轮眼询问（1 号若再进自己 TurnBegin 则取消偷取）
        for _ in 0..24 {
            if let Some(c) = e.open_choice().cloned() {
                if c.spec.prompt == "huanle_xuelunyang_target" && c.observer == 0 {
                    break;
                }
                submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: Vec::new() });
                continue;
            }
            let cur = e.turn().current;
            e.debug_clear_hand(cur);
            submit(&mut e, cur, GameAction::NextPhase);
        }
        let tgt_ask = e.open_choice().expect("huanle_xuelunyang_target").clone();
        assert_eq!(tgt_ask.spec.prompt, "huanle_xuelunyang_target");
        let out_t = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: tgt_ask.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        assert!(matches!(out_t, CommandOutcome::Pending { .. }));
        let sk_ask = e.open_choice().expect("huanle_xuelunyang_skill").clone();
        assert_eq!(sk_ask.spec.prompt, "huanle_xuelunyang_skill");
        assert!(
            !e.save().core.skills.contains_key(&(1, zhiheng)),
            "1 号已过期的回合末临时技（制衡）不得保留于 core.skills"
        );
        assert_eq!(
            sk_ask.spec.options.len(),
            base_skills_count,
            "1 号写轮眼可窃技能列表长度须回到基线（无过期临时技泄露）"
        );
    }

    // 2. 已觉醒/禁用的永久技能不得被写轮眼重新启用或回合末移除
    {
        let mut e = Engine::new_with_extensions(3731, 3, &packs).expect("建局");
        for p in 0..3 {
            e.debug_remove_general(p);
            e.debug_clear_hand(p);
        }
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let xly = e.debug_place(0, "hnl_xuelunyang_4");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: xly, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        e.debug_grant(0, "bingfeng");
        e.debug_set_skill_enabled(0, "bingfeng", false);
        e.debug_grant(1, "bingfeng");

        advance_to_play(&mut e, 2);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        for _ in 0..8 {
            if let Some(c) = e.open_choice() {
                if c.spec.prompt == "huanle_xuelunyang_target" {
                    break;
                }
            }
            let cur = e.turn().current;
            e.debug_clear_hand(cur);
            submit(&mut e, cur, GameAction::NextPhase);
        }
        let tgt_ask = e.open_choice().expect("huanle_xuelunyang_target").clone();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: tgt_ask.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        let sk_ask = e.open_choice().expect("huanle_xuelunyang_skill").clone();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: sk_ask.id,
                selection: vec![sk_ask.spec.options[0]],
            },
        );
        assert!(
            !e.debug_has_skill(0, "bingfeng"),
            "0 号已觉醒（禁用）的永久技能 bingfeng 不得被写轮眼临时授技重新激活"
        );
        let bf_def = e.catalog().skill_by_name("bingfeng").expect("bingfeng def");
        let snap = e.save();
        let inst = snap.core.skills.get(&(0, bf_def)).expect("永久实例应保留");
        assert!(!inst.enabled && !inst.expires_end_of_turn);
    }
}

/// 猴子偷桃与五行生克统一经 `CardUsed` 窗口按 `useCardToBegin` 座位序竞争，及濒死用桃统一经 `CardUsed`
/// （`欢乐卡牌/card/skill.js:5-31`、`wuxing/extension.js:62-172`、`content.ts:4020-4048`）。
#[test]
fn u06_monkey_and_shengke_seat_order_and_dying_card_used() {
    use engine_native::archive::{decode_archive, encode_archive};

    let huanle_pack = huanle_pack_from_repo();
    let wuxing_json = std::fs::read_to_string(
        repo_extensions_root().join("wuxing-upstream-liuli/assets/cards.json"),
    )
    .expect("read wuxing");
    let wuxing_pack = bridge("wuxing-upstream-liuli", &wuxing_json).expect("bridge wuxing");
    let packs: Vec<&ExtensionPack> = vec![&huanle_pack, &wuxing_pack];

    // 1. PlayCard 用火属性【桃】：当前回合 1 号（土）先于 0 号（持猴），故 1 号 _shengke 先摸 1，随后 0 号猴子确认；
    //    Confirm 偷走桃时 1 号保留 _shengke 摸到的 1 张牌但不回血。
    {
        let mut e = Engine::new_with_extensions(3732, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
        advance_to_play(&mut e, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(1, 2);
        e.state_core_mut().players[1].wunature = Some(4); // 土
        let tao = e.debug_place(1, "tao");
        e.state_core_mut().card_natures.insert(tao, 3); // 火生土
        let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        assert_eq!(e.debug_hand(1).len(), 1, "座位序 1 号先于 0 号，问猴子前 1 号 _shengke 已摸 1");
        let mk = e.open_choice().expect("huanle_monkey_confirm").clone();
        assert_eq!(mk.spec.prompt, "huanle_monkey_confirm");
        let out_steal = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        assert!(matches!(out_steal, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hp(1), 2, "0 号偷桃取消结算，1 号不回血");
        assert!(e.debug_hand(0).contains(&tao));
        assert_eq!(e.debug_hand(1).len(), 1, "1 号已提前触发的 _shengke 摸牌保留");
    }

    // 2. 濒死座位序 A：当前回合 0 号（持猴）先于 1 号（濒死者、土），2 号出火属性桃救 1 号：
    //    先问 0 号猴子（此时 1 号手牌 0）；Confirm 夺桃则 1 号永不触发 _shengke；Pass 放行则 1 号触发 _shengke 摸 1、
    //    记 CardPlayed 日志、回血至 1 并触发 Boss 相柳「邪酬」shedu +1。
    {
        let mut e = Engine::new_with_extensions(3733, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
        e.debug_grant(0, "邪酬");
        e.debug_set_hp(1, 1);
        e.state_core_mut().players[1].wunature = Some(4); // 土
        let tao2 = e.debug_place(2, "tao");
        e.state_core_mut().card_natures.insert(tao2, 3); // 火生土
        let sha = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let save_p2 = e.open_choice().expect("p2 dying save").clone();
        let shedu_before = e.state_core().player(0).shedu;
        let log_before = e.log().len();
        let out_save = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: save_p2.id,
                selection: vec![ChoiceOption::Card(tao2)],
            },
        );
        assert!(matches!(out_save, CommandOutcome::Pending { .. }));
        assert!(
            e.log()[log_before..].iter().any(|ev| matches!(ev, engine_native::LogEvent::CardPlayed { card, player: 2, .. } if *card == tao2)),
            "濒死用桃须发射真实 CardUsed 并记录 CardPlayed"
        );
        assert_eq!(e.debug_hand(1).len(), 0, "0 号持猴在 1 号前，未放行时不得先触发生克");
        let mk = e.open_choice().expect("huanle_monkey_confirm").clone();
        assert_eq!(mk.observer, 0);

        let bytes = encode_archive(&e.save());
        let mut e_pass =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();

        // Confirm 分支：夺桃（0 号得 tao2 并进入下一轮濒死救援询问），1 号不得触发 _shengke（邪酬于 useCard 时先于 useCardToBegin 猴子触发）
        let out_confirm = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        assert!(matches!(out_confirm, CommandOutcome::Pending { .. }));
        assert_eq!(e.debug_hand(1).len(), 0, "桃在 _shengke 前被偷，1 号不得触发 _shengke");
        assert_eq!(e.state_core().player(0).shedu, shedu_before + 1, "邪酬(global useCard)先于猴子(useCardToBegin)触发");
        let save_p0_stolen = e.open_choice().expect("p0 holds stolen tao2 -> dying ask").clone();
        assert_eq!(save_p0_stolen.observer, 0);
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: save_p0_stolen.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
        assert!(!e.debug_alive(1), "夺桃且 0 号放弃施救后 1 号阵亡");
        assert_eq!(e.debug_hand(1).len(), 0, "阵亡全过程 1 号从未触发 _shengke");

        // Pass 分支：1 号触发 _shengke 摸 1、救回，且邪酬 shedu +1
        let out_pass = submit(
            &mut e_pass,
            0,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
        assert!(matches!(out_pass, CommandOutcome::Accepted { .. }));
        assert_eq!(e_pass.debug_hand(1).len(), 1, "放行后 1 号触发火生土 _shengke 摸 1");
        assert_eq!(e_pass.debug_hp(1), 1);
        assert_eq!(e_pass.state_core().player(0).shedu, shedu_before + 1, "濒死用桃放行后须触发 CardUsed 邪酬");
    }

    // 3. 濒死座位序 B：当前回合 0 号，1 号（土）先于 2 号（持猴），0 号出火属性桃救 1 号：
    //    1 号先于 2 号，问 2 号猴子确认时 1 号已经由 _shengke 摸 1；且濒死用【酒】自救亦触发 _shengke。
    {
        let mut e = Engine::new_with_extensions(3734, 3, &packs).expect("建局");
        advance_to_play(&mut e, 2);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(2, "hnl_monkey_3");
        submit(&mut e, 2, GameAction::PlayCard { card: monkey, targets: vec![] });
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(1, 1);
        e.state_core_mut().players[1].wunature = Some(4); // 土
        let tao0 = e.debug_place(0, "tao");
        e.state_core_mut().card_natures.insert(tao0, 3); // 火生土
        let sha = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.remove(&sha);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let save_p0 = e.open_choice().expect("p0 dying save").clone();
        assert_eq!(save_p0.observer, 0);
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: save_p0.id,
                selection: vec![ChoiceOption::Card(tao0)],
            },
        );
        let mk = e.open_choice().expect("p2 huanle_monkey_confirm").clone();
        assert_eq!(mk.observer, 2);
        assert_eq!(e.debug_hand(1).len(), 1, "1 号在 2 号持猴者之前，问 2 号前 1 号 _shengke 已摸 1");
        submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: mk.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        assert_eq!(e.debug_hp(1), 0, "Confirm 夺桃后 1 号未回血仍死");

        // 另测濒死【酒】自救亦触发 _shengke
        let mut e2 = Engine::new_with_extensions(3735, 3, &packs).expect("建局");
        advance_to_play(&mut e2, 0);
        for p in 0..3 {
            e2.debug_clear_hand(p);
        }
        e2.debug_set_hp(1, 1);
        e2.state_core_mut().players[1].wunature = Some(4); // 土
        let jiu1 = e2.debug_place(1, "jiu");
        e2.state_core_mut().card_natures.insert(jiu1, 3); // 火生土
        let sha2 = e2.debug_place(0, "sha");
        e2.state_core_mut().card_natures.remove(&sha2);
        submit(&mut e2, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
        let save_jiu = e2.open_choice().expect("p1 dying jiu save").clone();
        let out_jiu = submit(
            &mut e2,
            1,
            GameAction::Answer {
                choice_id: save_jiu.id,
                selection: vec![ChoiceOption::Card(jiu1)],
            },
        );
        assert!(matches!(out_jiu, CommandOutcome::Accepted { .. }));
        assert_eq!(e2.debug_hp(1), 1);
        assert_eq!(e2.debug_hand(1).len(), 1, "濒死使用火属性【酒】自救亦须触发火生土 _shengke 摸 1");
    }
}

/// 写轮眼回合内被禁用的临时技过期清理 + 内部状态/全局牌监听 charlotte 排除验收
/// （`欢乐卡牌/card/skill.js:70-128`、`player.js:5643-5685,11267-11291`、`library/skill.js:1702-1715`）：
/// 1. 1 号回合内经写轮眼偷取临时技 `制衡`（`expires_end_of_turn: true`）后，击杀持 `忘魂` 的 2 号令 `制衡` 在回合内被禁用（`enabled = false`），
///    1 号回合结束交接时该已禁用临时技仍须从 `core.skills` 彻底删除，下家 0 号写轮眼不得窃取之；
/// 2. 内部状态印记与全局牌监听（`GwentFengyinState`/`SwdFengyinState`/`GujianFoodShihuifenBan`/十神器与地牌附技等）须归为 `is_charlotte_skill`，
///    既不计入 `is_charlotte_skill == false`，实际被贴上 `gw_zumoshoukao`（`GW_FENGYIN_STATE_KEY`）时也不得被写轮眼窃取。
#[test]
fn u06_xuelunyang_disabled_temp_skill_expires_and_state_stamps_excluded() {
    let huanle_pack = huanle_pack_from_repo();
    let ext_root = repo_extensions_root();
    let swd_json = std::fs::read_to_string(ext_root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("read swd");
    let swd_pack = content_native_loader::bridge_pack(&swd_json).expect("bridge swd").pack;
    let gwent_json = std::fs::read_to_string(ext_root.join("shahai-gwent-upstream-noname/assets/cards.json"))
        .expect("read gwent");
    let gwent_pack = content_native_loader::bridge_pack(&gwent_json).expect("bridge gwent").pack;
    let gujian_json = std::fs::read_to_string(ext_root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("read gujian");
    let gujian_pack = content_native_loader::bridge_pack(&gujian_json).expect("bridge gujian").pack;
    let mtg_json = std::fs::read_to_string(ext_root.join("shahai-mtg-upstream-noname/assets/cards.json"))
        .expect("read mtg");
    let mtg_pack = content_native_loader::bridge_pack(&mtg_json).expect("bridge mtg").pack;
    let packs: Vec<&ExtensionPack> = vec![&huanle_pack, &swd_pack, &gwent_pack, &gujian_pack, &mtg_pack];

    // 1. 回合内被 `忘魂` 禁用的临时技 `制衡` 在回合末仍须彻底移除
    {
        let p1_packs: Vec<&ExtensionPack> = vec![&huanle_pack, &swd_pack];
        let mut e = Engine::new_with_extensions(3736, 3, &p1_packs).expect("建局");
        for p in 0..3 {
            e.debug_remove_general(p);
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "制衡");
        e.debug_grant(2, "忘魂");
        e.debug_set_hp(2, 1);
        let zhiheng = e.catalog().skill_by_name("制衡").unwrap();

        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        let xly1 = e.debug_place(1, "hnl_xuelunyang_4");
        submit(&mut e, 1, GameAction::PlayCard { card: xly1, targets: vec![] });

        // 推进至 0 号再回到 1 号 TurnBegin，让 1 号用写轮眼窃取 0 号的 `制衡`（expires_end_of_turn = true）
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        for _ in 0..8 {
            if let Some(c) = e.open_choice() {
                if c.spec.prompt == "huanle_xuelunyang_target" && c.observer == 1 {
                    break;
                }
            }
            let cur = e.turn().current;
            e.debug_clear_hand(cur);
            submit(&mut e, cur, GameAction::NextPhase);
        }
        let t_ask = e.open_choice().expect("p1 xuelunyang target").clone();
        assert_eq!(t_ask.spec.prompt, "huanle_xuelunyang_target");
        submit(&mut e, 1, GameAction::Answer { choice_id: t_ask.id, selection: vec![ChoiceOption::Player(0)] });
        let s_ask = e.open_choice().expect("p1 xuelunyang skill").clone();
        assert_eq!(s_ask.spec.prompt, "huanle_xuelunyang_skill");
        submit(&mut e, 1, GameAction::Answer { choice_id: s_ask.id, selection: vec![ChoiceOption::Tag(0)] });
        assert!(e.save().core.skills.get(&(1, zhiheng)).is_some_and(|i| i.enabled && i.expires_end_of_turn));

        // 1 号 Play 击杀 2 号触发 `忘魂`：1 号的 `制衡` 在本回合内被置为 enabled=false
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let sha = e.debug_place(1, "sha");
        submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![2] });
        assert!(!e.debug_alive(2));
        assert!(
            e.save().core.skills.get(&(1, zhiheng)).is_some_and(|i| !i.enabled && i.expires_end_of_turn),
            "忘魂须已将 1 号回合内临时技制衡置为禁用态"
        );

        // 1 号结束回合交接给 0 号：禁用的过期临时技须被移除
        advance_to_play(&mut e, 0);
        assert!(
            !e.save().core.skills.contains_key(&(1, zhiheng)),
            "回合内被禁用的临时技制衡于 1 号回合末交接时必须彻底删除"
        );
        // 0 号顺手牵羊取走 1 号写轮眼并装备，下个 TurnBegin 验证 1 号无法被偷到制衡
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let snatch = e.debug_place(0, "snatch");
        submit(&mut e, 0, GameAction::PlayCard { card: snatch, targets: vec![1] });
        if let Some(nw) = e.open_choice().filter(|c| c.spec.prompt == "nullify_window").cloned() {
            submit(&mut e, nw.observer, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
        }
        assert!(e.debug_hand(0).contains(&xly1), "顺手牵羊应取回 1 号写轮眼");
        submit(&mut e, 0, GameAction::PlayCard { card: xly1, targets: vec![] });
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        for _ in 0..8 {
            if let Some(c) = e.open_choice() {
                if c.spec.prompt == "huanle_xuelunyang_target" && c.observer == 0 {
                    break;
                }
            }
            let cur = e.turn().current;
            e.debug_clear_hand(cur);
            submit(&mut e, cur, GameAction::NextPhase);
        }
        if let Some(t_ask) = e.open_choice().filter(|c| c.spec.prompt == "huanle_xuelunyang_target").cloned() {
            assert!(
                !t_ask.spec.options.contains(&ChoiceOption::Player(1)),
                "1 号过期被禁临时技已清除，不得出现在写轮眼目标列表：{:?}",
                t_ask.spec.options
            );
        }
    }

    // 2. 内部状态印记与全局牌监听归入 `is_charlotte_skill`，且 `gw_zumoshoukao` 授予封印印记不可被写轮眼窃取
    {
        let mut e = Engine::new_with_extensions(3737, 3, &packs).expect("建局");
        for key in [
            engine_native::content::host::GW_FENGYIN_STATE_KEY,
            engine_native::content::host::SWD_FENGYIN_STATE_KEY,
            engine_native::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY,
            "qinglongzhigui",
            "zhuquezhizhang",
            "baishouzhihu",
            "xuanwuzhihuang",
            "huanglinzhicong",
            "cangchizhibi",
            "shencaojie",
            "yuchan_swap",
            "shenmiguo_window",
            engine_native::content::host::MTG_YIXIALAN_RECAST_KEY,
            engine_native::content::host::MTG_LINZHONGJIANTA_SKILL_KEY,
            engine_native::content::host::MTG_YOULIN_DISCOVER_KEY,
        ] {
            let def = e.catalog().skill_by_name(key).unwrap_or_else(|| panic!("missing {key}"));
            assert!(e.catalog().is_charlotte_skill(def), "内部状态/全局监听技 {key} 须为 is_charlotte_skill");
        }
        for p in 0..3 {
            e.debug_remove_general(p);
            e.debug_clear_hand(p);
        }
        advance_to_play(&mut e, 0);
        let xly = e.debug_place(0, "hnl_xuelunyang_4");
        submit(&mut e, 0, GameAction::PlayCard { card: xly, targets: vec![] });
        let zm_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == CardKind::Zumoshoukao)
            .expect("Zumoshoukao")
            .name;
        let zm = e.debug_place(0, zm_name);
        submit(&mut e, 0, GameAction::PlayCard { card: zm, targets: vec![1] });
        if let Some(nw) = e.open_choice().filter(|c| c.spec.prompt == "nullify_window").cloned() {
            submit(&mut e, nw.observer, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
        }
        assert!(
            e.debug_has_skill(1, engine_native::content::host::GW_FENGYIN_STATE_KEY),
            "1 号已挂 gw_fengyin_state"
        );
        // 再到 0 号回合开始前清除 1 号 `gw_fengyin_state` 前无法轮转（GwentFengyinState 任意阶段结束即解封）；
        // 在无其他技能仅持 `gw_fengyin_state` 时用写轮眼候选检查不暴露 `gw_fengyin_state`（见上方 is_charlotte 断言）
    }
}

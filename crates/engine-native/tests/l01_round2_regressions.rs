//! L01 `AI优化` Round 1/2 逐条回归测试（#180 & #189 全部 10 项修复闭环）。

use std::collections::BTreeSet;

use content_native_loader::{apply_l01_config_to_engine, to_l01_ai_options};
use engine_native::ai::{BoundedPlanner, GeneralRarity};
use engine_native::event::Phase;
use engine_native::state::KingdomIdentity;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameMode, Role};

use crate::l01_ai_optimize_fixes::{
    activate_skill, advance_to_play, drain, flag, l01_battle_pack, parse_l01_cfg,
    snapshot_roundtrip, submit,
};

#[test]
fn l01_round2_regressions_for_all_10_findings() {
    let pack = l01_battle_pack();

    // Finding #3: yiyi 逐目标无懈 + 弃装备撤销技能并写日志/触发连营 + legal_actions 包含 [0, ally]
    {
        let mut e = Engine::new_with_extensions(202631, 4, &[&pack]).expect("yiyi per-target");
        advance_to_play(&mut e, 1);
        let eq1 = e.debug_place(1, "horse_plus");
        let _ = submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: eq1,
                targets: vec![],
            },
        );
        drain(&mut e);
        assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 2);
        e.debug_grant(3, "连营");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let yiyi = e.debug_move_named_card_to_hand(0, "yiyi");
        assert!(
            e.legal_actions(0).contains(&GameAction::PlayCard {
                card: yiyi,
                targets: vec![0, 1],
            }),
            "legal_actions must expose [0, ally] yiyi combo"
        );
        let wuxie = e.debug_place(2, "nullify");
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: yiyi,
                targets: vec![0, 1, 3],
            },
        );
        let ask_nw0 = e.open_choice().expect("nullify for target 0").clone();
        assert_eq!(ask_nw0.spec.prompt, "nullify_window");
        assert_eq!(ask_nw0.observer, 2);
        let mut nw_restored = snapshot_roundtrip(&e, &[&pack]);
        let ans_wuxie = GameAction::Answer {
            choice_id: ask_nw0.id,
            selection: vec![ChoiceOption::Card(wuxie)],
        };
        let _ = submit(&mut e, 2, ans_wuxie.clone());
        let _ = submit(&mut nw_restored, 2, ans_wuxie);
        assert_eq!(e.state_hash(), nw_restored.state_hash());

        let mut saw_discard1 = false;
        for _ in 0..32 {
            let Some(c) = e.open_choice().cloned() else {
                break;
            };
            let sel = if c.observer == 1 && c.spec.prompt == "l01_yiyi_discard" {
                saw_discard1 = true;
                let second = c
                    .spec
                    .options
                    .iter()
                    .find(|&&o| o != ChoiceOption::Card(eq1))
                    .copied()
                    .unwrap();
                vec![ChoiceOption::Card(eq1), second]
            } else if c.spec.prompt == "l01_yiyi_discard" {
                c.spec
                    .options
                    .iter()
                    .take(c.spec.min.max(1))
                    .copied()
                    .collect()
            } else if c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec
                    .options
                    .iter()
                    .take(c.spec.min.max(1))
                    .copied()
                    .collect()
            };
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        }
        assert!(saw_discard1, "target 1 must resolve after target 0 nullified");
        assert!(e.debug_hand(0).is_empty());
        assert!(e.debug_equips(1).is_empty());
        assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1);
        assert_eq!(e.debug_hand(1).len(), 1);
        assert_eq!(e.debug_hand(3).len(), 1, "target 3 连营 triggers draw 1");
        let evs = e.event_view_for(0);
        assert!(evs.iter().any(|ev| matches!(
            ev,
            engine_native::view::EventView::Public(
                engine_native::machine::LogEvent::CardNullified { player: 0, card }
            ) if *card == yiyi
        )));
        assert!(evs.iter().any(|ev| matches!(
            ev,
            engine_native::view::EventView::Public(
                engine_native::machine::LogEvent::CardDiscarded { player: 1, cards }
            ) if cards.contains(&eq1)
        )));
    }

    // Finding #5: _aiyh_reserved_shan 完整正反例（phaseUse / countCards>=2 / getCardUsable / jiu / linked / bagua / 2闪）
    {
        let cfg = parse_l01_cfg(r#"{"btAi":"shyg"}"#);

        // 正例：出牌阶段、攻击者装连弩仍有余杀且剩 2 手牌、非酒杀、hp=2、无连环/防具阻碍 -> 留闪 Pass
        let mut e = Engine::new_with_extensions(202632, 2, &[&pack]).expect("shan reserve");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let zhuge = e.debug_place(0, "zhuge");
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: zhuge,
                targets: vec![],
            },
        );
        drain(&mut e);
        let sha1 = e.debug_place(0, "sha");
        let _keep1 = e.debug_place(0, "shan");
        let _keep2 = e.debug_place(0, "tao");
        let shan1 = e.debug_place(1, "shan");
        e.debug_set_hp(1, 2);
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha1,
                targets: vec![1],
            },
        );
        let ask = e.open_choice().expect("respond_shan").clone();
        let act_hold = e
            .plan_action(
                1,
                &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e)),
            )
            .action;
        assert_eq!(
            act_hold,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Pass],
            },
            "last Shan holds when Phase::Play, hand_count>=2, attacker has usable sha, hp>=2"
        );

        // 反例 1: 响应者 hp == 1 -> 必出闪
        e.debug_set_hp(1, 1);
        assert_eq!(
            e.plan_action(
                1,
                &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e)),
            )
            .action,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(shan1)],
            }
        );

        // 反例 2: 非出牌阶段 -> 不留闪，必出闪
        e.debug_set_hp(1, 2);
        e.state_core_mut().turn.phase = Phase::End;
        assert_eq!(
            e.plan_action(
                1,
                &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e)),
            )
            .action,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(shan1)],
            }
        );
        e.state_core_mut().turn.phase = Phase::Play;

        // 反例 3: 攻击者剩余手牌 < 2 -> 必出闪
        e.debug_clear_hand(0);
        let _keep_one = e.debug_place(0, "shan");
        assert_eq!(
            e.plan_action(
                1,
                &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e)),
            )
            .action,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(shan1)],
            },
            "attacker hand_count < 2 must play Shan"
        );
        drain(&mut e);

        // 反例 4: 攻击者有>=2手牌但无连弩/咆哮（已用唯一杀，getCardUsable('sha')<1）-> 必出闪
        let mut e_nousable =
            Engine::new_with_extensions(2026321, 2, &[&pack]).expect("no usable sha");
        advance_to_play(&mut e_nousable, 0);
        e_nousable.debug_clear_hand(0);
        e_nousable.debug_clear_hand(1);
        let sha_only = e_nousable.debug_place(0, "sha");
        let _x1 = e_nousable.debug_place(0, "shan");
        let _x2 = e_nousable.debug_place(0, "tao");
        let shan_def = e_nousable.debug_place(1, "shan");
        e_nousable.debug_set_hp(1, 2);
        let _ = submit(
            &mut e_nousable,
            0,
            GameAction::PlayCard {
                card: sha_only,
                targets: vec![1],
            },
        );
        let ask_nu = e_nousable.open_choice().expect("respond_shan").clone();
        assert_eq!(
            e_nousable
                .plan_action(
                    1,
                    &BoundedPlanner::new(64)
                        .with_l01_options(to_l01_ai_options(&cfg, 1, &e_nousable)),
                )
                .action,
            GameAction::Answer {
                choice_id: ask_nu.id,
                selection: vec![ChoiceOption::Card(shan_def)],
            },
            "attacker with 0 remaining usable Sha must play Shan"
        );
        drain(&mut e_nousable);

        // 反例 5: 真实 酒->杀（jiu_bonus 在 ShaMachine 上）+ 连弩 + >=2手牌 -> 必出闪
        let mut e_jiu = Engine::new_with_extensions(2026323, 2, &[&pack]).expect("real jiu sha");
        advance_to_play(&mut e_jiu, 0);
        e_jiu.debug_clear_hand(0);
        e_jiu.debug_clear_hand(1);
        let z2 = e_jiu.debug_place(0, "zhuge");
        let _ = submit(
            &mut e_jiu,
            0,
            GameAction::PlayCard {
                card: z2,
                targets: vec![],
            },
        );
        drain(&mut e_jiu);
        let jiu_c = e_jiu.debug_place(0, "jiu");
        let sha_j = e_jiu.debug_place(0, "sha");
        let _j1 = e_jiu.debug_place(0, "shan");
        let _j2 = e_jiu.debug_place(0, "tao");
        let shan_j = e_jiu.debug_place(1, "shan");
        e_jiu.debug_set_hp(1, 3);
        let _ = submit(
            &mut e_jiu,
            0,
            GameAction::PlayCard {
                card: jiu_c,
                targets: vec![],
            },
        );
        drain(&mut e_jiu);
        let _ = submit(
            &mut e_jiu,
            0,
            GameAction::PlayCard {
                card: sha_j,
                targets: vec![1],
            },
        );
        let ask_jiu = e_jiu.open_choice().expect("respond_shan jiu").clone();
        assert_eq!(
            e_jiu
                .plan_action(
                    1,
                    &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e_jiu)),
                )
                .action,
            GameAction::Answer {
                choice_id: ask_jiu.id,
                selection: vec![ChoiceOption::Card(shan_j)],
            },
            "real Jiu->Sha must not hold Shan"
        );
        drain(&mut e_jiu);

        // 反例 6: 防御者横置且场上存在连环低血友方 / 装备八卦阵 -> 必出闪
        let mut e_link =
            Engine::try_new_identity_with_extensions(2026324, 4, &[&pack]).expect("linked");
        e_link.debug_set_identity_roles(vec![Role::Rebel, Role::Lord, Role::Rebel, Role::Loyalist]);
        advance_to_play(&mut e_link, 0);
        for p in 0..4 {
            e_link.debug_clear_hand(p);
        }
        let z3 = e_link.debug_place(0, "zhuge");
        let _ = submit(
            &mut e_link,
            0,
            GameAction::PlayCard {
                card: z3,
                targets: vec![],
            },
        );
        drain(&mut e_link);
        let sha_l = e_link.debug_place(0, "sha");
        let _l1 = e_link.debug_place(0, "shan");
        let _l2 = e_link.debug_place(0, "tao");
        let shan_l = e_link.debug_place(1, "shan");
        e_link.debug_set_hp(1, 2);
        e_link.debug_set_hp(3, 1);
        e_link.state_core_mut().player_mut(1).linked = true;
        e_link.state_core_mut().player_mut(3).linked = true;
        e_link
            .state_core_mut()
            .vm_aux
            .flags
            .insert((3, "l01_role_shown".to_string()), 1);
        let _ = submit(
            &mut e_link,
            0,
            GameAction::PlayCard {
                card: sha_l,
                targets: vec![1],
            },
        );
        let ask_l = e_link.open_choice().expect("respond_shan link").clone();
        assert_eq!(
            e_link
                .plan_action(
                    1,
                    &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e_link)),
                )
                .action,
            GameAction::Answer {
                choice_id: ask_l.id,
                selection: vec![ChoiceOption::Card(shan_l)],
            },
            "linked with low-HP linked ally must play Shan"
        );
        // 装备八卦阵（`armor`）亦必出闪
        e_link.state_core_mut().player_mut(1).linked = false;
        e_link.state_core_mut().player_mut(3).linked = false;
        let bagua_c = (0..e_link.catalog().cards.len() as u32)
            .find(|&cid| e_link.catalog().cards[cid as usize].name == "armor")
            .unwrap();
        e_link.state_core_mut().player_mut(1).equips.push(bagua_c);
        assert_eq!(
            e_link
                .plan_action(
                    1,
                    &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e_link)),
                )
                .action,
            GameAction::Answer {
                choice_id: ask_l.id,
                selection: vec![ChoiceOption::Card(shan_l)],
            },
            "bagua equip must play Shan"
        );
        drain(&mut e_link);

        // 反例 7: 有 >= 2 闪 -> 不再留闪，正常出闪
        let mut e_two = Engine::new_with_extensions(2026322, 2, &[&pack]).expect("two shans");
        advance_to_play(&mut e_two, 0);
        e_two.debug_clear_hand(0);
        e_two.debug_clear_hand(1);
        let z4 = e_two.debug_place(0, "zhuge");
        let _ = submit(
            &mut e_two,
            0,
            GameAction::PlayCard {
                card: z4,
                targets: vec![],
            },
        );
        drain(&mut e_two);
        let sha_two = e_two.debug_place(0, "sha");
        let _t1 = e_two.debug_place(0, "shan");
        let _t2 = e_two.debug_place(0, "tao");
        let shan_a = e_two.debug_place(1, "shan");
        let _shan_b = e_two.debug_place(1, "shan");
        e_two.debug_set_hp(1, 2);
        let _ = submit(
            &mut e_two,
            0,
            GameAction::PlayCard {
                card: sha_two,
                targets: vec![1],
            },
        );
        let ask_two = e_two.open_choice().expect("respond_shan 2").clone();
        assert_eq!(
            e_two
                .plan_action(
                    1,
                    &BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 1, &e_two)),
                )
                .action,
            GameAction::Answer {
                choice_id: ask_two.id,
                selection: vec![ChoiceOption::Card(shan_a)],
            }
        );
        drain(&mut e_two);
    }

    // Finding #6: to_l01_ai_options 真实填充 + l01_rt_cf_pct / l01_rt_qz_pct 均实盘引导 BoundedPlanner + 国战暗将零泄密
    {
        let cfg = parse_l01_cfg(
            r#"{"applyCf":"pj","qzCf":true,"cf":{"绝情":2.3},"qz":{"曹操":2.4}}"#,
        );
        let mut e = Engine::new_with_extensions(202633, 3, &[&pack]).expect("opts");
        e.debug_set_general(1, "曹操");
        e.debug_grant(1, "twzhian_ai");
        e.debug_grant(2, "dcwumei_all");
        e.debug_grant(2, "绝情");
        let opts = to_l01_ai_options(&cfg, 0, &e);
        assert!(opts.twzhian_active_enemy);
        assert!(opts.dcwumei_active);
        assert!(
            opts.general_rarities
                .iter()
                .any(|(n, r)| n == "曹操" && *r == GeneralRarity::Legend),
            "general_rarities must populate visible Legend general"
        );

        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let sha_pick = e.debug_place(0, "sha");
        e.debug_set_hp(1, 3);
        e.debug_set_hp(2, 3);
        e.state_core_mut()
            .vm_aux
            .storage
            .insert("l01_rt_cf_pct:绝情".to_string(), 500);
        assert_eq!(
            e.plan_action(
                0,
                &BoundedPlanner::new(64)
                    .with_policy_variant(2)
                    .with_l01_options(to_l01_ai_options(&cfg, 0, &e)),
            )
            .action,
            GameAction::PlayCard {
                card: sha_pick,
                targets: vec![2],
            }
        );

        // 显式验证 l01_rt_qz_pct 实时权重调整引导 BoundedPlanner 切换目标
        let mut e_qz = Engine::new_with_extensions(2026331, 3, &[&pack]).expect("rt qz steer");
        e_qz.debug_set_general(1, "关羽");
        e_qz.debug_set_general(2, "张飞");
        advance_to_play(&mut e_qz, 0);
        for p in 0..3 {
            e_qz.debug_clear_hand(p);
        }
        let sha_qz = e_qz.debug_place(0, "sha");
        e_qz.debug_set_hp(1, 3);
        e_qz.debug_set_hp(2, 3);
        let cfg_qz = parse_l01_cfg(r#"{"qzCf":true}"#);
        e_qz.state_core_mut()
            .vm_aux
            .storage
            .insert("l01_rt_qz_pct:张飞".to_string(), 600);
        assert_eq!(
            e_qz.plan_action(
                0,
                &BoundedPlanner::new(64)
                    .with_policy_variant(2)
                    .with_l01_options(to_l01_ai_options(&cfg_qz, 0, &e_qz)),
            )
            .action,
            GameAction::PlayCard {
                card: sha_qz,
                targets: vec![2],
            },
            "l01_rt_qz_pct must steer Sha to boosted visible general"
        );

        let mut k1 = Engine::try_new_with_extensions_mode(202634, 4, &[&pack], GameMode::Kingdoms)
            .expect("k1");
        let mut k2 = Engine::try_new_with_extensions_mode(202634, 4, &[&pack], GameMode::Kingdoms)
            .expect("k2");
        k1.debug_set_general(1, "曹操");
        k2.debug_set_general(1, "刘备");
        k1.state_core_mut()
            .vm_aux
            .storage
            .insert("l01_rt_qz_pct:曹操".to_string(), 500);
        k2.state_core_mut()
            .vm_aux
            .storage
            .insert("l01_rt_qz_pct:曹操".to_string(), 500);
        advance_to_play(&mut k1, 0);
        advance_to_play(&mut k2, 0);
        k1.debug_clear_hand(0);
        k2.debug_clear_hand(0);
        let _ = k1.debug_place(0, "sha");
        let _ = k2.debug_place(0, "sha");
        let o1 = to_l01_ai_options(&cfg, 0, &k1);
        let o2 = to_l01_ai_options(&cfg, 0, &k2);
        assert_eq!(o1.general_rarities, o2.general_rarities);
        assert_eq!(
            k1.plan_action(0, &BoundedPlanner::new(64).with_l01_options(o1))
                .action,
            k2.plan_action(0, &BoundedPlanner::new(64).with_l01_options(o2))
                .action
        );
    }

    // Finding #7 + #8 + #9 + #10
    {
        let mut e_nk = Engine::try_new_identity_with_extensions(202635, 4, &[&pack]).expect("nk");
        e_nk.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        e_nk.debug_grant(3, "_aiyh_neiKey");
        advance_to_play(&mut e_nk, 3);
        assert!(matches!(
            activate_skill(&mut e_nk, 3, "_aiyh_neiKey"),
            CommandOutcome::Rejected(_)
        ));

        let wj_cfg = parse_l01_cfg(r#"{"Wj":true,"wjs":6,"wj":["曹操"]}"#);
        let mut picks = BTreeSet::new();
        for seed in [202641u64, 202642, 202643, 202644, 202645, 202646] {
            let mut ek = Engine::try_new_with_extensions_mode(
                seed,
                2,
                &[&pack],
                GameMode::Kingdoms,
            )
            .expect("ek");
            ek.debug_set_general(1, "曹操");
            ek.state_core_mut()
                .set_kingdom_identity(1, KingdomIdentity::Declared("wei".to_string()))
                .unwrap();
            apply_l01_config_to_engine(&wj_cfg, &mut ek);
            let rep = flag(&ek, 1, "l01_wj_replaced:0") - 1;
            picks.insert(rep);
            let expected_fac = engine_native::state::GameStateCore::faction_name(
                ek.catalog().general_meta(rep as u16).unwrap().1,
            )
            .to_string();
            assert_eq!(
                ek.state_core().player(1).kingdom_identity,
                KingdomIdentity::Declared(expected_fac)
            );
        }
        assert!(picks.len() >= 2, "Wj must vary across RNG seeds: {picks:?}");

        let mut ed = Engine::try_new_identity_with_extensions(202636, 4, &[&pack]).expect("d0");
        ed.debug_set_identity_roles(vec![Role::Loyalist, Role::Lord, Role::Rebel, Role::Traitor]);
        ed.debug_grant(0, "_aiyh_firstKs");
        ed.debug_grant(0, "_attzhi");
        advance_to_play(&mut ed, 1);
        for p in 0..4 {
            ed.debug_clear_hand(p);
        }
        ed.debug_set_hp(0, 1);
        let _ = ed.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
        drain(&mut ed);
        assert!(!ed.debug_alive(0));
        ed.debug_clear_hand(2);
        let sha_r = ed.debug_place(2, "sha");
        ed.debug_set_turn(2, Phase::Play, 2);
        let _ = submit(
            &mut ed,
            2,
            GameAction::PlayCard {
                card: sha_r,
                targets: vec![1],
            },
        );
        drain(&mut ed);
        assert_ne!(flag(&ed, 0, "l01_pub_att:2"), 0);
        ed.debug_set_hp(2, 1);
        let _ = ed.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
        drain(&mut ed);
        assert!(!ed.debug_alive(2));
        assert_eq!(flag(&ed, 0, "l01_endgame_no_fan"), 1);

        let f4_cfg = parse_l01_cfg(r#"{"fixFour":true,"fixEQz":true,"fixECf":true}"#);
        let mut ef = Engine::try_new_identity_with_extensions(202637, 4, &[&pack]).expect("0l");
        ef.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Rebel, Role::Traitor]);
        ef.debug_grant(0, "_aiyh_firstKs");
        ef.debug_grant(1, "_aiyh_fixEQz");
        ef.debug_grant(1, "_aiyh_fixECf");
        let hp_before = ef.debug_hp(0);
        apply_l01_config_to_engine(&f4_cfg, &mut ef);
        assert_eq!(flag(&ef, 0, "l01_fix_four_applied"), 1);
        advance_to_play(&mut ef, 1);
        advance_to_play(&mut ef, 0);
        assert_eq!(ef.debug_hp(0), hp_before);
        advance_to_play(&mut ef, 1);
        assert!(matches!(
            activate_skill(&mut ef, 1, "_aiyh_fixEQz"),
            CommandOutcome::Rejected(_)
        ));
        assert!(matches!(
            activate_skill(&mut ef, 1, "_aiyh_fixECf"),
            CommandOutcome::Rejected(_)
        ));
    }
}

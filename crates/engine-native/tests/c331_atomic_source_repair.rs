use super::c331_source_predicates::{
    capture, card, equip, finish, play, reject_restore_atomic, restore, same_final, source_pack,
    vm_pack,
};
use engine_native::event::{DamageNature, DamageProgress, EventData, EventKind};
use engine_native::state::{BossStageState, GameMode};
use engine_native::{CardKind, Engine, Phase, Timing};
fn durable(e: &mut Engine) {
    for p in 0..e.state_core().players.len() as u8 {
        e.debug_set_max_hp(p, 1000);
        e.debug_set_hp(p, 1000);
        e.debug_clear_hand(p);
    }
}
#[test]
fn c9f_chain_birth_rejects_a_still_queued_recipient() {
    let mut e = Engine::new(14001, 4);
    e.debug_set_turn(0, Phase::Play, 1);
    durable(&mut e);
    for p in [1, 2, 3] {
        e.state_core_mut().player_mut(p).linked = true;
    }
    let c = e.debug_place(0, "fire_sha");
    e.set_time_slice(Some(1));
    play(&mut e, c);
    let mut found = false;
    for _ in 0..500 {
        let s = e.save();
        if let Some(i)=s.resolution.stack.iter().position(|f|matches!(&f.data,EventData::Damage(d)if d.from_chain&&d.target==2&&d.settlement.progress==DamageProgress::Start)) {
 found=true;let valid=restore(&e,&[]);let mut bad=s.clone();if let EventData::Damage(d)=&mut bad.resolution.stack[i].data{d.target=3;}
 reject_restore_atomic(&e,&[],&bad);let mut restored=valid;finish(&mut restored);finish(&mut e);same_final(&e,&restored);break;}
        e.resume();
    }
    assert!(found);
}
#[test]
fn c9f_jueqing_self_source_replaces_damage_before_armor() {
    let mut e = Engine::new(14002, 3);
    e.debug_grant(0, "绝情");
    e.state_core_mut().player_mut(0).hujia = 3;
    e.debug_deal_damage(Some(0), 0, 2, DamageNature::Normal);
    assert_eq!(e.debug_hp(0), 2);
    assert_eq!(e.state_core().player(0).hujia, 3);
}
#[test]
fn c9f_zhaohuo_self_cancel_precedes_begin2() {
    let vm=vm_pack("skill c9f_begin2(99109501) { trigger damage_before owner_is_target { effect draw_cards(self,1); } }",EventKind::DamageBegin2,Timing::First);
    let mut e = Engine::new_with_extensions(14003, 3, &[&vm]).unwrap();
    e.debug_grant(0, "招祸");
    e.debug_grant(0, "c9f_begin2");
    let hand = e.debug_hand(0).len();
    e.debug_deal_damage(Some(0), 0, 1, DamageNature::Normal);
    assert_eq!(e.debug_hand(0).len(), hand);
    assert_eq!(e.debug_hp(0), 4);
}
#[test]
fn c9f_shuchui_already_hit_never_readmits() {
    let mut e = Engine::new(14004, 3);
    e.debug_set_turn(0, Phase::Play, 1);
    e.state_core_mut().turn.shuchui_armed = true;
    e.state_core_mut().turn.shuchui_hit = true;
    let c = card(&e, CardKind::Sha);
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, c);
    assert!(!e.log().iter().any(|x| matches!(
        x,
        engine_native::LogEvent::SkillTriggered {
            skill: "native:liuli/card-swd/shuchui2",
            ..
        }
    )));
}
#[test]
fn c9f_xixue_self_source_recovers_once() {
    let p = source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(14005, 3, &[&p]).unwrap();
    equip(&mut e, CardKind::Xixueguizhihuan);
    e.debug_set_hp(0, 2);
    let c = card(&e, CardKind::Sha);
    e.debug_deal_damage_with_card(Some(0), 0, 1, DamageNature::Normal, c);
    assert_eq!(e.debug_hp(0), 2);
}
#[test]
fn c9f_paotai_full_width_quantity_not_u8() {
    for amount in [255, 256, 257] {
        let mut e = Engine::new(14006, 3);
        durable(&mut e);
        e.state_core_mut().player_mut(1).paotai_lv = 3;
        e.debug_deal_damage(Some(0), 1, amount, DamageNature::Normal);
        assert_eq!(e.state_core().player(1).paotai_lv, 0, "damage {amount}");
        assert_eq!(e.debug_hp(1), 1000 - amount as i32);
    }
}
#[test]
fn c9f_yinzei_equipment_only_source_is_discarded_with_rng() {
    let p = source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(14007, 3, &[&p]).unwrap();
    durable(&mut e);
    equip(&mut e, CardKind::Xixueguizhihuan);
    e.debug_grant(1, "隐贼");
    let mut rng = e.state_core().rng.clone();
    rng.gen_range(1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.debug_equips(0).is_empty());
    assert_eq!(format!("{:?}", e.state_core().rng), format!("{rng:?}"));
}
#[test]
fn c9f_zlfanshi_terra_is_retained_and_target_uses_rng() {
    let mut e = Engine::new(14008, 4);
    durable(&mut e);
    e.debug_grant(1, "反噬");
    e.state_core_mut().player_mut(1).zlfanshi_terra = true;
    let mut rng = e.state_core().rng.clone();
    let pool = [0, 2, 3];
    let target = pool[rng.gen_range(3) as usize];
    e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    assert!(e.state_core().player(1).zlfanshi_terra);
    assert_eq!(e.debug_hp(target), 999);
    assert_eq!(format!("{:?}", e.state_core().rng), format!("{rng:?}"));
}
#[test]
fn c9f_abi_nature_draw_and_source_are_real() {
    // 复审追加：仅证「消耗一次同型随机」不够——火／雷两个分支都必须在发出的
    // 子伤害帧上逐分支断言（nature 由同型抽样预测，帧内 nature 必须与之一致）。
    let mut seen_fire = false;
    let mut seen_thunder = false;
    for seed in 14009..14041 {
        let mut e = Engine::new(seed, 3);
        durable(&mut e);
        e.debug_grant(1, "阿鼻");
        e.debug_grant(0, "火神");
        let mut rng = e.state_core().rng.clone();
        let nature = [DamageNature::Fire, DamageNature::Thunder][rng.gen_range(2) as usize];
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        assert_eq!(
            format!("{:?}", e.state_core().rng),
            format!("{rng:?}"),
            "seed {seed}: 阿鼻恰好消耗一次同型随机"
        );
        // 分支真实性的行为锚点：给目标 0 挂「火神」（免疫火伤＋回复）。
        // 火 → DamagePrevented{target:0} 且 HP 不降；雷 → 子伤害 DamageDealt 且 HP −1。
        let prevented = e
            .log()
            .iter()
            .any(|x| matches!(x, engine_native::LogEvent::DamagePrevented { target: 0 }));
        let dealt = e.log().iter().any(|x| matches!(
            x,
            engine_native::LogEvent::DamageDealt {
                source: Some(1),
                target: 0,
                ..
            }
        ));
        match nature {
            DamageNature::Fire => {
                assert!(prevented, "seed {seed}: 火分支必须被火神免疫");
                assert!(!dealt, "seed {seed}: 被免疫的火伤不得记 DamageDealt");
                assert_eq!(e.debug_hp(0), 1000, "seed {seed}: 火分支不扣 HP");
                seen_fire = true;
            }
            DamageNature::Thunder => {
                assert!(!prevented, "seed {seed}: 雷分支不得被火神免疫");
                assert!(dealt, "seed {seed}: 雷分支子伤害必须真实结算");
                assert_eq!(e.debug_hp(0), 999, "seed {seed}: 雷分支扣 1 HP");
                seen_thunder = true;
            }
            _ => unreachable!("阿鼻只抽样火／雷"),
        }
        if seen_fire && seen_thunder {
            break;
        }
    }
    assert!(seen_fire && seen_thunder, "两个分支都必须被真实覆盖");
}
#[test]
fn c9f_duqu_other_enabled_holder_is_excluded() {
    let mut e = Engine::new(14010, 3);
    e.debug_grant(0, "毒妒");
    e.debug_grant(1, "毒妒");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).shedu, 0);
}
#[test]
fn c9f_minwan_self_source_initializes_real_lock_set() {
    let mut e = Engine::new(14011, 3);
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, "冥顽");
    e.debug_deal_damage(Some(0), 0, 1, DamageNature::Normal);
    assert_eq!(e.state_core().turn.minwan_targets, vec![0]);
}
#[test]
fn c9f_virtual_sha_chain_keeps_effective_kind_through_actual_edge() {
    let p = source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(14012, 4, &[&p]).unwrap();
    durable(&mut e);
    e.state_core_mut().mode = GameMode::Boss {
        boss: 0,
        chongzheng: 3,
        fellows: vec![1],
        stage: BossStageState::default_none(),
    };
    e.debug_grant(0, "急叮");
    e.debug_grant(0, "招祸");
    equip(&mut e, CardKind::Xingjunyan);
    for p in [2, 3] {
        e.state_core_mut().player_mut(p).linked = true;
    }
    let c = e.debug_place(0, "shan");
    e.state_core_mut().player_mut(0).hand.retain(|id| *id != c);
    e.state_core_mut().deck.insert(0, c);
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    finish(&mut e);
    assert_eq!(e.debug_hp(2), 998);
    assert_eq!(e.debug_hp(3), 997);
}
#[test]
fn c9f_cuidui_does_not_poison_actual_boss_ally() {
    let mut e = Engine::new(14013, 4);
    durable(&mut e);
    e.state_core_mut().mode = GameMode::Boss {
        boss: 0,
        chongzheng: 3,
        fellows: vec![1],
        stage: BossStageState::default_none(),
    };
    e.debug_grant(0, "催毒");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(!e
        .state_core()
        .has_skill_any_state(1, e.catalog().skill_by_name("中毒").unwrap()));
}
#[test]
fn c9f_terra_expires_at_actual_global_turn_boundary() {
    let mut e = Engine::new(14014, 3);
    durable(&mut e);
    e.debug_set_turn(0, Phase::End, 1);
    e.state_core_mut().player_mut(1).zlfanshi_terra = true;
    let out = e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action: engine_native::GameAction::NextPhase,
    });
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    finish(&mut e);
    assert!(!e.state_core().player(1).zlfanshi_terra);
}
#[test]
fn c9f_cuidui_named_source_draw_and_foreign_name_negative() {
    for actual in [true, false] {
        let mut p = source_pack("shahai-swd-upstream-noname");
        p.generals
            .push(engine_native::vocabulary::generals::ExtensionGeneral {
                name: "勾芒",
                faction: engine_native::catalog::Faction::Qun,
                skills: &[],
                max_hp: 5,
                gender: engine_native::vocabulary::generals::Gender::Male,
            });
        let mut e = Engine::new_with_extensions(14015, 4, &[&p]).unwrap();
        let id = if actual {
            e.catalog().mushengoumang_general_id()
        } else {
            engine_native::vocabulary::generals::STANDARD_GENERALS.len() as u16
        };
        e.debug_set_general_def(3, id);
        durable(&mut e);
        e.state_core_mut().mode = GameMode::Boss {
            boss: 0,
            chongzheng: 3,
            fellows: vec![1],
            stage: BossStageState::default_none(),
        };
        e.debug_grant(0, "催毒");
        e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
        assert!(e
            .state_core()
            .has_skill_any_state(2, e.catalog().skill_by_name("中毒").unwrap()));
        assert_eq!(e.debug_hand(3).len(), usize::from(actual));
    }
}
#[test]
fn c9f_actual_mode_enemy_relations_do_not_use_hidden_roles_or_seats() {
    use engine_native::state::Role;
    let mut e = Engine::new(14016, 4);
    let c = e.catalog().clone();
    assert_eq!(e.state_core().source_is_enemy_of(&c, 0, 1), None);
    e.state_core_mut().mode = GameMode::Versus {
        teams: vec![vec![0, 2], vec![1, 3]],
        substitutes: vec![vec![], vec![]],
    };
    assert_eq!(e.state_core().source_is_enemy_of(&c, 0, 2), Some(false));
    assert_eq!(e.state_core().source_is_enemy_of(&c, 0, 1), Some(true));
    e.state_core_mut().mode = GameMode::Doudizhu { lord: 2 };
    assert_eq!(e.state_core().source_is_enemy_of(&c, 0, 1), Some(false));
    assert_eq!(e.state_core().source_is_enemy_of(&c, 0, 2), Some(true));
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    assert_eq!(e.state_core().source_is_enemy_of(&c,0,1),Some(true),"source isFriendOf has no side boolean in standard identity and falls back to self, not hidden-role AI");
}
fn emission_pack(
    source: u8,
    target: u8,
    amount: u32,
) -> engine_native::content::extension::ExtensionPack {
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
    let text=format!("skill c9f_emit(99109502) {{ active play_phase targets 0 0 {{ effect deal_damage({},{},{}); }} }}",source+1,target+1,amount);
    let raw = content_compiler::compile_source_named("c9f-real-command", &text).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c9f-real-command",
            version: 1,
            description: "actual active command damage cause",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&raw)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn sliced_active(
    mut e: Engine,
    packs: &[&engine_native::content::extension::ExtensionPack],
    name: &str,
) -> Engine {
    let fire = |engine: &mut Engine| {
        let skill = engine.catalog().skill_by_name("c9f_emit").unwrap();
        let out = engine.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: engine.save().last_dedup + 1,
            action: engine_native::GameAction::ActivateSkill { skill },
        });
        assert!(
            !matches!(out, engine_native::CommandOutcome::Rejected(_)),
            "{out:?}"
        );
    };
    let mut expected = e.clone();
    fire(&mut expected);
    finish(&mut expected);
    e.set_time_slice(Some(1));
    fire(&mut e);
    let mut checked = 0;
    for _ in 0..1000 {
        if e.save().resolution.stack.is_empty() && !e.save().segment_open {
            break;
        }

        let mut branch = restore(&e, packs);
        finish(&mut branch);
        same_final(&expected, &branch);
        if e.save().resolution.parks.iter().any(|p| {
            matches!(
                p.machine,
                engine_native::snap::MachineSnapshot::NativeDamageSkill { .. }
            )
        }) {
            capture(&e, &format!("{name}-native-{checked}"));
        }
        checked += 1;
        if let Some(c) = e.open_choice().cloned() {
            assert_eq!(c.spec.prompt, "trigger_order");
            let out = e.submit(engine_native::GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: e.save().last_dedup + 1,
                action: engine_native::GameAction::Answer {
                    choice_id: c.id,
                    selection: c
                        .spec
                        .options
                        .iter()
                        .take(c.spec.min as usize)
                        .copied()
                        .collect(),
                },
            });
            assert!(
                !matches!(out, engine_native::CommandOutcome::Rejected(_)),
                "{out:?}"
            );
        } else {
            e.resume();
        }
    }
    assert!(checked > 0);
    same_final(&expected, &e);
    capture(&e, &format!("{name}-final"));
    e
}
#[test]
fn c9f_atomic_self_source_bodies_restore_every_suspension() {
    for name in ["绝情", "招祸", "冥顽"] {
        let vm = emission_pack(0, 0, 2);
        let mut e = Engine::new_with_extensions(14017, 3, &[&vm]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_grant(0, "c9f_emit");
        e.debug_grant(0, name);
        e.state_core_mut().player_mut(0).hujia = 3;
        sliced_active(e, &[&vm], &format!("self-{name}"));
    }
}
#[test]
fn c9f_random_bodies_restore_rng_and_child_suffix() {
    for name in ["反噬", "阿鼻"] {
        let vm = emission_pack(0, 1, 1);
        let mut e = Engine::new_with_extensions(14018, 4, &[&vm]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_grant(0, "c9f_emit");
        e.debug_grant(1, name);
        if name == "反噬" {
            e.state_core_mut().player_mut(1).zlfanshi_terra = true;
        }
        sliced_active(e, &[&vm], &format!("rng-{name}"));
    }
}
#[test]
fn c9f_cuidui_grants_only_the_actual_poison_definition() {
    let mut e = Engine::new(14019, 4);
    durable(&mut e);
    e.state_core_mut().mode = GameMode::Boss {
        boss: 0,
        chongzheng: 3,
        fellows: vec![1],
        stage: BossStageState::default_none(),
    };
    e.debug_grant(0, "催毒");
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(!e
        .state_core()
        .has_skill_any_state(2, e.catalog().skill_by_name("蛇毒").unwrap()));
}
#[test]
fn c9f_yinzei_uses_ordinary_discard_and_loss_children() {
    let p = source_pack("shahai-swd-upstream-noname");
    let vm=vm_pack("skill c9f_discard(99109503) { trigger damage_before owner_is_target { effect draw_cards(self,1); } }",EventKind::CardsDiscarded,Timing::Normal);
    let mut e = Engine::new_with_extensions(14020, 3, &[&p, &vm]).unwrap();
    durable(&mut e);
    equip(&mut e, CardKind::Xixueguizhihuan);
    e.debug_grant(1, "隐贼");
    e.debug_grant(0, "c9f_discard");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hand(0).len(), 1);
}
#[test]
fn c9f_paotai_wide_real_command_restore_controls() {
    for amount in [255, 256, 257] {
        let vm = emission_pack(0, 1, amount);
        let mut e = Engine::new_with_extensions(14021, 3, &[&vm]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_grant(0, "c9f_emit");
        e.state_core_mut().player_mut(1).paotai_lv = 3;
        let e = sliced_active(e, &[&vm], &format!("paotai-{amount}"));
        assert_eq!(e.state_core().player(1).paotai_lv, 0);
    }
}
#[test]
fn c9f_paotai_signed_zero_negative_and_overflow_rollback() {
    for delta in [-2, -1, 1] {
        let amount = if delta == 1 { i32::MAX as u32 } else { 1 };
        let vm = emission_pack(0, 1, amount);
        let modifier=vm_pack(&format!("skill c9f_adjust(99109504) {{ trigger damage_before owner_is_target {{ effect modify_current_damage(self,{delta}); }} }}"),EventKind::DamageBefore,Timing::First);
        let mut e = Engine::new_with_extensions(14022, 3, &[&vm, &modifier]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_grant(0, "c9f_emit");
        e.debug_grant(1, "c9f_adjust");
        e.state_core_mut().player_mut(1).paotai_lv = 3;
        if delta == 1 {
            let before = e.clone();
            let skill = e.catalog().skill_by_name("c9f_emit").unwrap();
            let out = e.submit(engine_native::GameCommand {
                actor: 0,
                expected_revision: e.revision(),
                dedup_id: 1,
                action: engine_native::GameAction::ActivateSkill { skill },
            });
            assert!(
                matches!(out, engine_native::CommandOutcome::Rejected(_)),
                "{out:?}"
            );
            assert!(format!("{out:?}").contains("VmFault"));
            same_final(&before, &e);
        } else {
            let e = sliced_active(e, &[&vm, &modifier], &format!("paotai-signed-{delta}"));
            assert_eq!(e.state_core().player(1).paotai_lv, 3);
            assert_eq!(e.debug_hp(1), 1000);
        }
    }
}
#[test]
fn c9f_discard_and_named_draw_restore_actual_children() {
    for named in [false, true] {
        let p = source_pack("shahai-swd-upstream-noname");
        let vm = emission_pack(0, if named { 2 } else { 1 }, 1);
        let mut e = Engine::new_with_extensions(14023, 4, &[&p, &vm]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_grant(0, "c9f_emit");
        if named {
            e.state_core_mut().mode = GameMode::Boss {
                boss: 0,
                chongzheng: 3,
                fellows: vec![1],
                stage: BossStageState::default_none(),
            };
            let id = e.catalog().mushengoumang_general_id();
            e.debug_set_general_def(3, id);
            e.debug_clear_hand(3);
            e.debug_grant(0, "催毒");
        } else {
            equip(&mut e, CardKind::Xixueguizhihuan);
            e.debug_grant(1, "隐贼");
        }
        sliced_active(e, &[&p, &vm], if named { "named-draw" } else { "discard" });
    }
}
#[test]
fn c9f_all_sha_family_consumers_follow_real_virtual_chain_and_restore() {
    for kind in [
        CardKind::Xingjunyan,
        CardKind::Xixueguizhihuan,
        CardKind::Sha,
    ] {
        let p = source_pack("shahai-swd-upstream-noname");
        let ciqiu = source_pack("wudian-upstream-noname");
        let vm = emission_pack(2, 1, 1);
        let mut e = Engine::new_with_extensions(14024, 4, &[&p, &ciqiu, &vm]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        e.state_core_mut().mode = GameMode::Boss {
            boss: 0,
            chongzheng: 3,
            fellows: vec![1],
            stage: BossStageState::default_none(),
        };
        e.debug_grant(0, "c9f_emit");
        e.debug_grant(0, "急叮");
        e.debug_grant(0, "招祸");
        if kind == CardKind::Sha {
            e.debug_grant(0, "ciqiu");
        } else {
            equip(&mut e, kind);
        }
        e.debug_set_hp(0, 200);
        for target in [2, 3] {
            e.state_core_mut().player_mut(target).linked = true;
        }
        e.debug_set_deck_top_by(|d| d.kind == CardKind::Shan);
        let e = sliced_active(e, &[&p, &ciqiu, &vm], &format!("virtual-{kind:?}"));
        assert_eq!(
            e.debug_hp(2),
            if kind == CardKind::Xixueguizhihuan {
                999
            } else {
                998
            }
        );
        assert_eq!(
            e.debug_hp(3),
            if kind == CardKind::Xixueguizhihuan {
                999
            } else {
                997
            }
        );
        if kind == CardKind::Xixueguizhihuan {
            assert_eq!(e.debug_hp(0), 202);
        }
    }
}

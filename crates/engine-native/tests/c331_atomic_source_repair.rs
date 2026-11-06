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
            .any(|x| matches!(x, engine_native::LogEvent::DamagePrevented { target: 0, .. }));
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
                if !seen_fire {
                    let vm = emission_pack(0, 1, 1);
                    let mut s = Engine::new_with_extensions(seed, 3, &[&vm]).unwrap();
                    durable(&mut s);
                    s.debug_set_turn(0, Phase::Play, 1);
                    s.debug_grant(0, "c9f_emit");
                    s.debug_grant(0, "火神");
                    s.debug_grant(1, "阿鼻");
                    let s = sliced_active(s, &[&vm], "abi-fire-branch");
                    assert_eq!(s.debug_hp(0), 1000);
                }
                seen_fire = true;
            }
            DamageNature::Thunder => {
                assert!(!prevented, "seed {seed}: 雷分支不得被火神免疫");
                assert!(dealt, "seed {seed}: 雷分支子伤害必须真实结算");
                assert_eq!(e.debug_hp(0), 999, "seed {seed}: 雷分支扣 1 HP");
                if !seen_thunder {
                    let vm = emission_pack(0, 1, 1);
                    let mut s = Engine::new_with_extensions(seed, 3, &[&vm]).unwrap();
                    durable(&mut s);
                    s.debug_set_turn(0, Phase::Play, 1);
                    s.debug_grant(0, "c9f_emit");
                    s.debug_grant(0, "火神");
                    s.debug_grant(1, "阿鼻");
                    let s = sliced_active(s, &[&vm], "abi-thunder-branch");
                    assert_eq!(s.debug_hp(0), 999);
                }
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
struct DuquAliasBehavior;
impl engine_native::SkillBehavior for DuquAliasBehavior {
    fn native_damage_effect(
        &self,
        kind: EventKind,
    ) -> Option<engine_native::event::NativeDamageEffect> {
        (kind == EventKind::DamageEnd).then_some(engine_native::event::NativeDamageEffect::Duqu)
    }
    fn filter(
        &self,
        _ctx: &engine_native::catalog::QueryCtx<'_>,
        _owner: u8,
        _event: &EventData,
    ) -> bool {
        false
    }
    fn start(&self, _owner: u8, _event: &EventData) -> Box<dyn engine_native::SkillMachine> {
        Box::new(engine_native::kernel::NoopMachine)
    }
}
#[test]
fn c9f_duqu_other_enabled_holder_is_excluded() {
    let mut e = Engine::new(14010, 3);
    e.debug_grant(0, "毒妒");
    e.debug_grant(1, "毒妒");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).shedu, 0);

    // F15: disabled boss_duqu on source is not excluded -> shedu increments.
    e.debug_set_skill_enabled(0, "毒妒", false);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).shedu, 1);

    // F15: enabled alias SkillDef mapping to NativeDamageEffect::Duqu is excluded.
    let mut e_alias = Engine::new(14010, 3);
    let alias_def = e_alias.catalog().skills.len() as u16;
    e_alias
        .try_with_catalog_mutation(|cat| {
            cat.skills.push(engine_native::SkillDef {
                name: "c9f_duqu_alias",
                trigger: Some(engine_native::event::NativeDamageEffect::Duqu.meta()),
                limits: Default::default(),
                behavior: Box::new(DuquAliasBehavior),
                original_source: None,
                presentation_binding: None,
            });
            Ok(())
        })
        .unwrap();
    e_alias.state_core_mut().skills.insert(
        (0, alias_def),
        engine_native::state::SkillInstance {
            owner: 0,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    e_alias.debug_grant(1, "毒妒");
    e_alias.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e_alias.state_core().player(0).shedu, 0);
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
                presentation_binding: None,
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
#[test]
fn c9f_f02_alias_host_definitions_and_modified_chain_child_restore() {
    for kind in [
        CardKind::Xingjunyan,
        CardKind::Xixueguizhihuan,
        CardKind::Sha,
    ] {
        let p = source_pack("shahai-swd-upstream-noname");
        let mut ciqiu = source_pack("wudian-upstream-noname");
        ciqiu
            .host_skills
            .push(engine_native::content::host::HostSkillDecl {
                name: "c9f_ciqiu_alias",
                behavior: engine_native::content::host::HostBehavior::WudianCiqiu,
                presentation_binding: None,
            });
        let vm = emission_pack(2, 1, 1);
        let chain_mod = vm_pack(
            "skill c9f_chain_plus(99109505) { trigger damage_before owner_is_target { effect modify_current_damage(self,1); } }",
            EventKind::DamageBefore,
            Timing::First,
        );
        let mut e =
            Engine::new_with_extensions(14025, 4, &[&p, &ciqiu, &vm, &chain_mod]).unwrap();
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
        e.debug_grant(3, "c9f_chain_plus");
        if kind == CardKind::Sha {
            e.debug_grant(0, "c9f_ciqiu_alias");
        } else {
            equip(&mut e, kind);
        }
        e.debug_set_hp(0, 200);
        for target in [2, 3] {
            e.state_core_mut().player_mut(target).linked = true;
        }
        e.debug_set_deck_top_by(|d| d.kind == CardKind::Shan);
        let e = sliced_active(
            e,
            &[&p, &ciqiu, &vm, &chain_mod],
            &format!("f02-alias-{kind:?}"),
        );
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
                998
            } else {
                996
            }
        );
        if kind == CardKind::Xixueguizhihuan {
            assert_eq!(e.debug_hp(0), 203);
        }
    }
}
#[test]
fn c9f_f04_shuchui_direct_virtual_sha_first_hit_then_second_hit_excluded_and_turn_renewal() {
    let vm = emission_pack(2, 1, 1);
    let mut e = Engine::new_with_extensions(14026, 4, &[&vm]).unwrap();
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
    e.state_core_mut().turn.shuchui_armed = true;
    e.state_core_mut().turn.shuchui_hit = false;
    e.debug_set_deck_top_by(|d| d.kind == CardKind::Shan);
    let mut e = sliced_active(e, &[&vm], "f04-shuchui-hit1");
    let count_shuchui = |engine: &Engine| {
        engine
            .log()
            .iter()
            .filter(|x| {
                matches!(
                    x,
                    engine_native::LogEvent::SkillTriggered {
                        skill: "native:liuli/card-swd/shuchui2",
                        ..
                    }
                )
            })
            .count()
    };
    assert!(e.state_core().turn.shuchui_hit);
    assert_eq!(count_shuchui(&e), 1);

    // 2nd direct virtual Sha hit in the same turn is excluded because shuchui_hit == true.
    e.debug_grant(0, "急叮");
    e.debug_set_deck_top_by(|d| d.kind == CardKind::Shan);
    let skill = e.catalog().skill_by_name("c9f_emit").unwrap();
    let out = e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: engine_native::GameAction::ActivateSkill { skill },
    });
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    finish(&mut e);
    assert_eq!(count_shuchui(&e), 1, "2nd hit in same turn excluded");

    // Turn boundary resets shuchui_hit = false and shuchui_armed = false; re-arming re-admits.
    e.debug_set_turn(0, Phase::End, 1);
    let out = e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: engine_native::GameAction::NextPhase,
    });
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    finish(&mut e);
    assert!(!e.state_core().turn.shuchui_hit);
    assert!(!e.state_core().turn.shuchui_armed);

    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.shuchui_armed = true;
    e.debug_grant(0, "急叮");
    e.debug_set_deck_top_by(|d| d.kind == CardKind::Shan);
    let out = e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: engine_native::GameAction::ActivateSkill { skill },
    });
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    finish(&mut e);
    assert!(e.state_core().turn.shuchui_hit);
    assert_eq!(count_shuchui(&e), 2, "turn renewal re-admits shuchui2");
}
#[test]
fn c9f_f08_yinzei_mixed_hand_and_equip_rng_revokes_equip_skill_and_orders_children() {
    let p = source_pack("shahai-swd-upstream-noname");
    let vm = emission_pack(0, 1, 1);
    let mut seen_hand = false;
    let mut seen_equip = false;
    for seed in 14030..14060 {
        let mut e = Engine::new_with_extensions(seed, 3, &[&p, &vm]).unwrap();
        durable(&mut e);
        e.debug_set_turn(0, Phase::Play, 1);
        let equip_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == CardKind::Shuchui)
            .unwrap()
            .name;
        let eq = e.debug_place(0, equip_name);
        let out = e.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action: engine_native::GameAction::PlayCard {
                card: eq,
                targets: vec![],
            },
        });
        assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
        assert!(e.debug_has_skill(0, "shuchui"));
        let hand_card = e.debug_place(0, "sha");
        e.debug_grant(0, "c9f_emit");
        e.debug_grant(1, "隐贼");
        let mut rng = e.state_core().rng.clone();
        let pick_idx = rng.gen_range(2);
        let expected_card = if pick_idx == 0 { hand_card } else { eq };

        // Step with fuel-1 and verify CardsLost -> CardsDiscarded child frame order and byte-restore.
        let mut expected = e.clone();
        let skill = expected.catalog().skill_by_name("c9f_emit").unwrap();
        expected.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: expected.revision(),
            dedup_id: expected.save().last_dedup + 1,
            action: engine_native::GameAction::ActivateSkill { skill },
        });
        finish(&mut expected);

        e.set_time_slice(Some(1));
        e.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action: engine_native::GameAction::ActivateSkill { skill },
        });
        let mut saw_children_order = false;
        for _ in 0..400 {
            let s = e.save();
            if s.resolution.stack.is_empty() && !s.segment_open {
                break;
            }
            let lost_pos = s.resolution.stack.iter().position(|f| {
                matches!(
                    &f.data,
                    EventData::CardsLost { player: 0, cards, .. } if cards == &[expected_card]
                )
            });
            let disc_pos = s.resolution.stack.iter().position(|f| {
                matches!(
                    &f.data,
                    EventData::CardsDiscarded { player: 0, cards } if cards == &[expected_card]
                )
            });
            if let (Some(lp), Some(dp)) = (lost_pos, disc_pos) {
                assert!(
                    s.resolution.stack[lp].id < s.resolution.stack[dp].id && lp < dp,
                    "CardsLost child must be emitted before CardsDiscarded child"
                );
                saw_children_order = true;
            }
            let mut branch = restore(&e, &[&p, &vm]);
            finish(&mut branch);
            same_final(&expected, &branch);
            e.resume();
        }
        assert!(saw_children_order);
        same_final(&expected, &e);
        assert_eq!(format!("{:?}", e.state_core().rng), format!("{rng:?}"));
        if pick_idx == 0 {
            assert!(e.debug_hand(0).is_empty());
            assert_eq!(e.debug_equips(0), vec![eq]);
            assert!(e.debug_has_skill(0, "shuchui"));
            seen_hand = true;
        } else {
            assert_eq!(e.debug_hand(0), vec![hand_card]);
            assert!(e.debug_equips(0).is_empty());
            assert!(!e.debug_has_skill(0, "shuchui"), "equip skill revoked");
            seen_equip = true;
        }
        if seen_hand && seen_equip {
            break;
        }
    }
    assert!(seen_hand && seen_equip);
}
#[test]
fn c9f_f10_zlfanshi_first_second_third_hit_expiry_and_renewal() {
    let mut e = Engine::new(14040, 4);
    durable(&mut e);
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(1, "反噬");
    assert!(!e.state_core().player(1).zlfanshi_terra);

    // Hit 1: arms zlfanshi_terra without retaliation or RNG consumption.
    let rng0 = e.state_core().rng.clone();
    e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    assert!(e.state_core().player(1).zlfanshi_terra);
    for p in [0, 2, 3] {
        assert_eq!(e.debug_hp(p), 1000, "Hit 1 only arms without retaliation");
    }
    assert_eq!(format!("{:?}", e.state_core().rng), format!("{rng0:?}"));

    // Hit 2 & Hit 3: retaliate and retain zlfanshi_terra.
    let pool = [0u8, 2, 3];
    for hit in [2, 3] {
        let mut rng = e.state_core().rng.clone();
        let target = pool[rng.gen_range(3) as usize];
        let before_hp = e.debug_hp(target);
        e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
        assert!(e.state_core().player(1).zlfanshi_terra, "hit {hit} retains terra");
        assert_eq!(e.debug_hp(target), before_hp - 1, "hit {hit} retaliates");
        assert_eq!(format!("{:?}", e.state_core().rng), format!("{rng:?}"));
    }

    // Turn boundary expires zlfanshi_terra.
    e.debug_set_turn(0, Phase::End, 1);
    let out = e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: engine_native::GameAction::NextPhase,
    });
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    finish(&mut e);
    assert!(!e.state_core().player(1).zlfanshi_terra);

    // Next turn Hit 1 re-arms without retaliation.
    let hps_before: Vec<_> = [0, 2, 3].into_iter().map(|p| e.debug_hp(p)).collect();
    let rng_next = e.state_core().rng.clone();
    e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    assert!(e.state_core().player(1).zlfanshi_terra);
    let hps_after: Vec<_> = [0, 2, 3].into_iter().map(|p| e.debug_hp(p)).collect();
    assert_eq!(hps_after, hps_before, "next turn Hit 1 re-arms without retaliation");
    assert_eq!(format!("{:?}", e.state_core().rng), format!("{rng_next:?}"));
}


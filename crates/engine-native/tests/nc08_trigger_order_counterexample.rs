//! ADR0788: the player chooses one next skill, independently of registration order.
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn pack(reverse: bool, optional: [bool; 3], count: usize) -> ExtensionPack {
    let names = ["c331_heal", "c331_hurt", "c331_draw"];
    let effects = [
        "recover_hp(self, 1)",
        "lose_hp(self, 1)",
        "draw_cards(self, 1)",
    ];
    let mut decls = Vec::new();
    for i in 0..count {
        let source = format!(
            "skill {}({}) {{ trigger phase_play_end owner_is_target {} {{ effect {}; }} }}",
            names[i],
            4200000010u32 + i as u32,
            if optional[i] { "optional" } else { "" },
            effects[i]
        );
        let pkg = content_compiler::compile_source_named(names[i], &source).unwrap();
        decls.extend(
            content_native_loader::bridge_vm_skill_decls(&pkg)
                .unwrap()
                .decls,
        );
    }
    if reverse {
        decls.reverse();
    }
    ExtensionPack {
        def: ExtensionDef {
            name: "c331-order",
            version: 1,
            description: "ADR0788",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    out
}
fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
}
fn choose(e: &mut Engine, name: &str) {
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "trigger_order");
    assert_eq!(c.spec.min, 1);
    assert_eq!(c.spec.max, 1);
    assert!(!c.spec.allow_cancel);
    let def = e.catalog().skill_by_name(name).unwrap();
    assert!(c.spec.options.contains(&ChoiceOption::Skill(def)));
    answer(e, vec![ChoiceOption::Skill(def)]);
}
fn ready(pack: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(20261060, 4, &[pack]).unwrap();
    for _ in 0..40 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let selection = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                vec![]
            } else {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            };
            answer(&mut e, selection);
        } else {
            let actor = e.turn().current;
            submit(&mut e, actor, GameAction::NextPhase);
        }
    }
    assert_eq!((e.turn().current, e.turn().phase), (1, Phase::Play));
    for decl in &pack.vm_skills {
        e.debug_grant(1, &decl.name);
    }
    e
}
fn open(pack: &ExtensionPack) -> Engine {
    let mut e = ready(pack);
    submit(&mut e, 1, GameAction::NextPhase);
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt != "discard_to_limit" {
            break;
        }
        answer(
            &mut e,
            c.spec.options.iter().take(c.spec.min).copied().collect(),
        );
    }
    e
}
fn hp(e: &Engine) -> i32 {
    e.state_core().player(1).hp
}

#[test]
fn nc08_same_priority_pair_asks_player_for_next_skill() {
    for reverse in [false, true] {
        for heal_first in [false, true] {
            let p = pack(reverse, [true, true, false], 2);
            let mut e = open(&p);
            assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_order");
            choose(&mut e, if heal_first { "c331_heal" } else { "c331_hurt" });
            assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
            answer(&mut e, vec![ChoiceOption::Confirm]);
            assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
            answer(&mut e, vec![ChoiceOption::Confirm]);
            assert_eq!(
                hp(&e),
                if heal_first { 3 } else { 4 },
                "capped heal and damage do not commute; registration={reverse}"
            );
        }
    }
}

#[test]
fn three_skills_choose_next_then_rebuild_and_optional_can_decline() {
    for optional in [[false; 3], [true; 3], [true, false, true]] {
        let p = pack(false, optional, 3);
        let mut e = open(&p);
        let hand = e.state_core().player(1).hand.len();
        choose(&mut e, "c331_heal");
        if optional[0] {
            answer(&mut e, vec![]);
        }
        let choice = e.open_choice().unwrap();
        assert_eq!(choice.spec.prompt, "trigger_order");
        assert_eq!(choice.spec.options.len(), 2);
        choose(&mut e, "c331_hurt");
        if optional[1] {
            answer(&mut e, vec![ChoiceOption::Confirm]);
        }
        if optional[2] {
            answer(&mut e, vec![ChoiceOption::Confirm]);
        }
        assert_eq!(hp(&e), 3);
        assert_eq!(e.state_core().player(1).hand.len(), hand + 1);
    }
}

#[test]
fn pending_and_optional_byte_restart_preserve_full_continuation() {
    let p = pack(false, [true; 3], 3);
    let mut direct = open(&p);
    for wanted in ["c331_hurt", "c331_heal"] {
        let bytes = engine_native::archive::encode_archive(&direct.save());
        let snap = engine_native::archive::decode_archive(&bytes).unwrap();
        let mut resumed = Engine::restore_with_extensions(&snap, &[&p]).unwrap();
        choose(&mut direct, wanted);
        choose(&mut resumed, wanted);
        let gate = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
            &resumed.save(),
        ))
        .unwrap();
        resumed = Engine::restore_with_extensions(&gate, &[&p]).unwrap();
        answer(&mut direct, vec![ChoiceOption::Confirm]);
        answer(&mut resumed, vec![ChoiceOption::Confirm]);
        assert_eq!(direct.state_hash(), resumed.state_hash());
        let mut actual = resumed.save();
        let expected = direct.save();
        actual.auth_generation = expected.auth_generation;
        assert_eq!(
            engine_native::archive::encode_archive(&actual),
            engine_native::archive::encode_archive(&expected)
        );
    }
}

#[test]
fn invalid_order_answers_do_not_change_state_or_rng() {
    let p = pack(false, [false; 3], 3);
    let mut e = open(&p);
    let c = e.open_choice().unwrap().clone();
    for selection in [
        vec![],
        vec![c.spec.options[0], c.spec.options[1]],
        vec![c.spec.options[0]; 2],
        vec![ChoiceOption::Skill(u16::MAX)],
    ] {
        let before = engine_native::archive::encode_archive(&e.save());
        let out = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action: GameAction::Answer {
                choice_id: c.id,
                selection,
            },
        });
        assert!(matches!(out, CommandOutcome::Rejected(_)));
        assert_eq!(before, engine_native::archive::encode_archive(&e.save()));
    }
    let rng = e.state_core().rng.clone();
    let plan = e.plan_action(1, &engine_native::ai::BoundedPlanner::new(64));
    let GameAction::Answer { selection, .. } = plan.action else {
        panic!("expected direct answer");
    };
    assert_eq!(selection, vec![c.spec.options[0]]);
    assert_eq!(rng, e.state_core().rng);
}

#[test]
fn machine_fields_and_queue_alias_mutations_are_rejected() {
    use engine_native::snap::MachineSnapshot;
    let p = pack(false, [false; 3], 3);
    let e = open(&p);
    let base = e.save();
    for mutation in 0..11 {
        let mut s = base.clone();
        let park = s
            .resolution
            .parks
            .iter_mut()
            .find(|p| matches!(p.machine, MachineSnapshot::TriggerOrder { .. }))
            .unwrap();
        if let MachineSnapshot::TriggerOrder {
            frame,
            owner,
            candidates,
            asked,
            cursor,
            done,
            ..
        } = &mut park.machine
        {
            match mutation {
                0 => *frame += 1,
                1 => *owner = 0,
                2 => candidates.reverse(),
                3 => candidates.push(candidates[0]),
                4 => *asked = false,
                5 => park.slot = engine_native::resolution::Slot::Intrinsic { frame: *frame },
                6 => park.resume_depth = 0,
                7 => s.resolution.stack.last_mut().unwrap().queue[0].order.4 ^= 1,
                8 => *cursor = 0,
                9 => done.push((*owner, candidates[0])),
                10 => done.extend([(0, 0), (0, 0)]),
                _ => unreachable!(),
            }
        }
        assert!(
            Engine::restore_with_extensions(&s, &[&p]).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn same_priority_temporarily_false_filter_revives_but_passed_priority_does_not() {
    use engine_native::rvm_bridge::VmFilterPolicy;
    for high_priority in [false, true] {
        let mut p = pack(false, [false; 3], 3);
        p.vm_skills[0].filter = VmFilterPolicy::OwnerWounded;
        p.vm_skills[0].trigger.priority = i32::from(high_priority);
        let mut e = open(&p);
        let def = e.catalog().skill_by_name("c331_heal").unwrap();
        assert!(!e
            .open_choice()
            .unwrap()
            .spec
            .options
            .contains(&ChoiceOption::Skill(def)));
        choose(&mut e, "c331_hurt");
        if high_priority {
            assert_eq!(
                hp(&e),
                3,
                "passed unavailable higher priority stays discarded"
            );
            assert!(e
                .open_choice()
                .is_none_or(|c| c.spec.prompt != "trigger_order"));
        } else {
            assert!(e
                .open_choice()
                .unwrap()
                .spec
                .options
                .contains(&ChoiceOption::Skill(def)));
            choose(&mut e, "c331_heal");
            assert_eq!(hp(&e), 4);
        }
    }
}

#[test]
fn optional_confirmation_rechecks_existence_enabled_alive_and_filter() {
    for mutation in 0..4 {
        let mut p = pack(false, [true; 3], 1);
        if mutation == 3 {
            p.vm_skills[0].filter = engine_native::rvm_bridge::VmFilterPolicy::OwnerWounded;
        }
        let mut e = ready(&p);
        e.debug_set_hp(1, 3);
        submit(&mut e, 1, GameAction::NextPhase);
        while let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt != "discard_to_limit" {
                break;
            }
            answer(
                &mut e,
                c.spec.options.iter().take(c.spec.min).copied().collect(),
            );
        }
        assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
        let def = e.catalog().skill_by_name("c331_heal").unwrap();
        match mutation {
            0 => {
                e.state_core_mut().skills.remove(&(1, def));
            }
            1 => {
                e.state_core_mut()
                    .skills
                    .get_mut(&(1, def))
                    .unwrap()
                    .enabled = false
            }
            2 => e.state_core_mut().player_mut(1).alive = false,
            3 => e.debug_set_hp(1, 4),
            _ => unreachable!(),
        }
        let before = hp(&e);
        let logs = e.log().len();
        answer(&mut e, vec![ChoiceOption::Confirm]);
        assert_eq!(hp(&e), before);
        assert!(!e.log()[logs..].iter().any(|log| matches!(
            log,
            engine_native::machine::LogEvent::SkillTriggered {
                skill: "c331_heal",
                ..
            }
        )));
    }
}

#[test]
fn silent_and_unnamed_metadata_runs_before_any_selectable_sibling() {
    let mut p = pack(false, [false; 3], 3);
    p.vm_skills[1].trigger.order_selectable = false;
    let mut e = open(&p);
    assert_eq!(hp(&e), 3, "silent hurt runs although its def follows heal");
    assert_eq!(e.open_choice().unwrap().spec.options.len(), 2);
    choose(&mut e, "c331_heal");
    assert_eq!(hp(&e), 4);
}

#[test]
fn priority_extremes_never_negate_or_overflow() {
    let mut p = pack(false, [false; 3], 3);
    p.vm_skills[0].trigger.priority = i32::MIN;
    p.vm_skills[1].trigger.priority = i32::MAX;
    p.vm_skills[2].trigger.priority = -1;
    let e = open(&p);
    let trace: Vec<_> = e
        .log()
        .iter()
        .filter_map(|l| match l {
            engine_native::machine::LogEvent::SkillTriggered { skill, .. }
                if skill.starts_with("c331_") =>
            {
                Some(*skill)
            }
            _ => None,
        })
        .collect();
    assert_eq!(trace, ["c331_hurt", "c331_draw", "c331_heal"]);
    assert_eq!(hp(&e), 4);
}

#[test]
fn owner_buckets_precede_priority_and_first_last_are_automatic() {
    use engine_native::event::Timing;
    let mut p = pack(false, [false; 3], 3);
    p.vm_skills[0].trigger.priority = i32::MIN;
    p.vm_skills[1].trigger.priority = i32::MAX;
    let mut e = ready(&p);
    let defs: Vec<_> = p
        .vm_skills
        .iter()
        .map(|d| e.catalog().skill_by_name(&d.name).unwrap())
        .collect();
    for def in &defs {
        e.state_core_mut().skills.remove(&(1, *def));
    }
    e.debug_grant(1, "c331_heal");
    e.debug_grant(2, "c331_hurt");
    p.vm_skills[1].filter = engine_native::rvm_bridge::VmFilterPolicy::Always;
    // Use the filter in a fresh catalog, then retain the same seat grants.
    let mut e = ready(&p);
    for def in &defs {
        e.state_core_mut().skills.remove(&(1, *def));
    }
    e.debug_grant(1, "c331_heal");
    e.debug_grant(2, "c331_hurt");
    submit(&mut e, 1, GameAction::NextPhase);
    while let Some(c) = e.open_choice().cloned() {
        answer(
            &mut e,
            c.spec.options.iter().take(c.spec.min).copied().collect(),
        );
    }
    let trace: Vec<_> = e
        .log()
        .iter()
        .filter_map(|l| match l {
            engine_native::machine::LogEvent::SkillTriggered { skill, owner }
                if skill.starts_with("c331_") =>
            {
                Some((*skill, *owner))
            }
            _ => None,
        })
        .collect();
    assert_eq!(trace, [("c331_heal", 1), ("c331_hurt", 2)]);
    p.vm_skills[0].trigger.timing = Timing::First;
    p.vm_skills[2].trigger.timing = Timing::Last;
    let e = open(&p);
    let trace: Vec<_> = e
        .log()
        .iter()
        .filter_map(|l| match l {
            engine_native::machine::LogEvent::SkillTriggered { skill, .. }
                if skill.starts_with("c331_") =>
            {
                Some(*skill)
            }
            _ => None,
        })
        .collect();
    assert_eq!(trace, ["c331_heal", "c331_hurt", "c331_draw"]);
}

#[test]
fn direct_extension_metadata_and_filter_identity_bind_restore() {
    let p = pack(false, [false; 3], 3);
    let e = open(&p);
    let snap = e.save();
    for mutation in 0..4 {
        let mut changed = pack(false, [false; 3], 3);
        match mutation {
            0 => changed.vm_skills[0].trigger.priority += 1,
            1 => changed.vm_skills[0].trigger.order_selectable = false,
            2 => {
                changed.vm_skills[0].filter =
                    engine_native::rvm_bridge::VmFilterPolicy::OwnerWounded
            }
            3 => changed.vm_skills.swap(0, 1),
            _ => unreachable!(),
        }
        assert!(Engine::restore_with_extensions(&snap, &[&changed]).is_err());
    }
    // Original330/105 bytes remain unchanged; Decision29 now rejects their envelope first.
    assert!(matches!(
        engine_native::archive::decode_archive(include_bytes!("fixtures/c331-real330-pending.ngsa")),
        Err(engine_native::archive::ArchiveError::UnsupportedFormatVersion(105))
    ));
}

fn replace_hurt(p: &mut ExtensionPack, body: &str) {
    let source = format!(
        "skill c331_hurt(4200000011) {{ trigger phase_play_end owner_is_target {{ {body} }} }}"
    );
    let pkg = content_compiler::compile_source_named("c331_hurt", &source).unwrap();
    p.vm_skills[1] = content_native_loader::bridge_vm_skill_decls(&pkg)
        .unwrap()
        .decls
        .remove(0);
}
#[test]
fn actual_vm_add_skill_enters_current_bucket_and_done_identity_does_not_repeat() {
    for priority in [0, 10] {
        let mut p = pack(false, [false; 3], 3);
        let probe = ready(&p);
        let heal = probe.catalog().skill_by_name("c331_heal").unwrap();
        replace_hurt(
            &mut p,
            &format!("effect lose_hp(self, 1); effect add_skill(self, {heal});"),
        );
        p.vm_skills[0].trigger.priority = priority;
        let mut e = ready(&p);
        e.state_core_mut().skills.remove(&(1, heal));
        submit(&mut e, 1, GameAction::NextPhase);
        while let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt != "discard_to_limit" {
                break;
            }
            answer(
                &mut e,
                c.spec.options.iter().take(c.spec.min).copied().collect(),
            );
        }
        choose(&mut e, "c331_hurt");
        if priority == 0 {
            choose(&mut e, "c331_heal");
        }
        assert_eq!(
            hp(&e),
            4,
            "new high priority also runs in the current unfinished owner bucket"
        );
        let count = e
            .log()
            .iter()
            .filter(|l| {
                matches!(
                    l,
                    engine_native::machine::LogEvent::SkillTriggered {
                        skill: "c331_heal",
                        ..
                    }
                )
            })
            .count();
        assert_eq!(count, 1);
    }
    let mut p = pack(false, [false; 3], 3);
    let probe = ready(&p);
    let heal = probe.catalog().skill_by_name("c331_heal").unwrap();
    replace_hurt(
        &mut p,
        &format!("effect add_skill(self, {heal}); effect lose_hp(self, 1);"),
    );
    let mut e = open(&p);
    choose(&mut e, "c331_heal");
    e.state_core_mut().skills.remove(&(1, heal));
    choose(&mut e, "c331_hurt");
    assert_eq!(hp(&e), 3);
    assert_eq!(
        e.log()
            .iter()
            .filter(|l| matches!(
                l,
                engine_native::machine::LogEvent::SkillTriggered {
                    skill: "c331_heal",
                    ..
                }
            ))
            .count(),
        1
    );
}

#[test]
fn c331_review_k1_requires_one_window_for_nonasking_optional_continuation() {
    let p = pack(false, [true; 3], 3);
    let mut e = open(&p);
    choose(&mut e, "c331_heal");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let base = e.save();
    for mode in 0..3 {
        let mut s = base.clone();
        let i = s
            .resolution
            .parks
            .iter()
            .position(|p| {
                matches!(
                    p.machine,
                    engine_native::MachineSnapshot::TriggerOrder { .. }
                )
            })
            .unwrap();
        match mode {
            0 => {
                s.resolution.parks.remove(i);
            }
            1 => {
                let p = s.resolution.parks[i].clone();
                s.resolution.parks.push(p);
            }
            2 => s.resolution.parks[i].resume_depth = 0,
            _ => unreachable!(),
        }
        assert!(
            Engine::restore_with_extensions(
                &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
                    &s
                ))
                .unwrap(),
                &[&p]
            )
            .is_err(),
            "mode {mode}"
        );
    }
}

#[test]
fn c331_review_k2_window_survives_real_set_turn_state_and_seat_change() {
    use engine_native::event::Timing;
    let mut p = pack(false, [false; 3], 3);
    replace_hurt(&mut p, "effect set_turn_state(self, 3);");
    p.vm_skills[1].filter = engine_native::rvm_bridge::VmFilterPolicy::Always;
    p.vm_skills[1].trigger.timing = Timing::First;
    let extra=content_compiler::compile_source("skill c331_late(4200000092) locked { trigger phase_play_end owner_is_target { effect storage_set(self_skill,1,1); } }").unwrap();
    p.vm_skills.extend(
        content_native_loader::bridge_vm_skill_decls(&extra)
            .unwrap()
            .decls,
    );
    let probe = Engine::new_with_extensions(1, 4, &[&p]).unwrap();
    let late = probe.catalog().skill_by_name("c331_late").unwrap();
    let grant=content_compiler::compile_source(&format!("skill c331_heal(4200000010) {{ trigger phase_play_end owner_is_target {{ effect recover_hp(self,1); effect add_skill(self,{late}); }} }}")).unwrap();
    p.vm_skills[0] = content_native_loader::bridge_vm_skill_decls(&grant)
        .unwrap()
        .decls
        .remove(0);
    let mut e = ready(&p);
    let def = e.catalog().skill_by_name("c331_hurt").unwrap();
    e.state_core_mut().skills.remove(&(1, def));
    e.state_core_mut().skills.remove(&(1, late));
    e.debug_grant(0, "c331_hurt");
    submit(&mut e, 1, GameAction::NextPhase);
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt != "discard_to_limit" {
            break;
        }
        answer(
            &mut e,
            c.spec.options.iter().take(c.spec.min).copied().collect(),
        );
    }
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_order");
    e.state_core_mut().swap_seat(1, 3);
    let snap =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    let mut restored = Engine::restore_with_extensions(&snap, &[&p])
        .expect("self-produced pending window must restore");
    let event_cursor = e.log().len();
    choose(&mut e, "c331_heal");
    choose(&mut restored, "c331_heal");
    assert!(
        e.open_choice()
            .unwrap()
            .spec
            .options
            .contains(&ChoiceOption::Skill(late)),
        "dynamic new grant stays in the frozen owner bucket"
    );
    choose(&mut e, "c331_late");
    choose(&mut restored, "c331_late");
    assert_eq!(&e.log()[event_cursor..], restored.log());
    let expected = e.save();
    let mut actual = restored.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
}

fn review_pack(source: &str) -> ExtensionPack {
    let pkg = content_compiler::compile_source_named("c331-review", source).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c331-review",
            version: 1,
            description: "review regression",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&pkg)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
#[test]
fn c331_review_k4_active_damage_before_remains_explicitly_unsupported() {
    let p=content_compiler::compile_source("skill c331_windows(4200000090) locked { active play_phase targets 0 0 { effect draw_cards(self,1); } trigger damage_before owner_is_target { effect storage_set(self_skill,1,1); } trigger damage owner_is_target { effect draw_cards(self,1); } }").unwrap();
    let err = content_native_loader::bridge_vm_skill_decls(&p)
        .expect_err("shared Damage payload cannot identify both windows in a handler set");
    assert!(err.to_string().contains("damage_before"), "{err}");
}
#[test]
fn c331_review_s2_grouped_handlers_keep_max_hp_hand_limit() {
    let p=review_pack("skill c331_limit(4200000091) locked { modifier hand_limit_base_max_hp override 1; trigger phase_play_end owner_is_target { effect storage_set(self_skill,1,1); } trigger phase_discard_end owner_is_target { effect storage_set(self_skill,2,1); } }");
    let mut e = Engine::new_with_extensions(78, 4, &[&p]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, "c331_limit");
    e.debug_set_hp(0, 2);
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 4);
    submit(&mut e, 0, GameAction::NextPhase);
    assert!(
        e.open_choice().is_none(),
        "four cards require no discard under the retained modifier"
    );
    let s =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    let mut r = Engine::restore_with_extensions(&s, &[&p]).unwrap();
    let actor = e.turn().current;
    submit(&mut e, actor, GameAction::NextPhase);
    submit(&mut r, actor, GameAction::NextPhase);
    for key in [1, 2] {
        assert_eq!(
            e.state_core()
                .vm_aux
                .storage
                .get(&format!("c331_limit#{key}")),
            Some(&1)
        );
    }
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
}

#[test]
fn c331_review_k3_real_pending_reveal_injects_same_and_future_owner_once() {
    use engine_native::content::extension::ExtensionGeneral;
    for owner in [0u8, 1, 2] {
        let mut p = pack(false, [false; 3], 3);
        for d in &mut p.vm_skills {
            d.trigger.event = engine_native::event::EventKind::CardUsed;
            d.filter = engine_native::rvm_bridge::VmFilterPolicy::Always;
        }
        p.generals.push(ExtensionGeneral {
            name: "c331 hidden",
            faction: engine_native::catalog::Faction::Qun,
            skills: &["c331_heal"],
            max_hp: 4,
            gender: engine_native::content::standard::Gender::Male,
        });
        let mut e = Engine::new_with_extensions(33103, 4, &[&p]).unwrap();
        e.debug_set_turn(0, Phase::Play, 1);
        let def = (0..e.catalog().general_count() as u16)
            .find(|id| e.catalog().general_meta(*id).unwrap().0 == "c331 hidden")
            .unwrap();
        e.state_core_mut().mode = engine_native::GameMode::Kingdoms;
        e.state_core_mut().player_mut(owner).general_slots[0] = Some(def);
        e.state_core_mut().player_mut(owner).revealed_generals = [false, false];
        e.debug_grant(1, "c331_hurt");
        e.debug_grant(1, "c331_draw");
        e.debug_grant(0, "马术");
        e.debug_set_hp(0, 3);
        let card = e.debug_place(0, if owner == 0 { "tao" } else { "sha" });
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card,
                targets: if owner == 0 { vec![] } else { vec![owner] },
            },
        );
        let old_revision = e.revision();
        let old = e.open_choice().unwrap().clone();
        assert_eq!(old.spec.prompt, "trigger_order");
        submit(&mut e, owner, GameAction::RevealGeneral { slot: 0 });
        let heal = e.catalog().skill_by_name("c331_heal").unwrap();
        assert_eq!(
            e.resolution()
                .stack
                .iter()
                .any(|f| f.queue.iter().any(|q| q.owner == owner && q.def == heal)),
            owner != 0,
            "only current/future buckets register"
        );
        if owner == 1 {
            assert_ne!(
                e.open_choice().unwrap().id,
                old.id,
                "changed group gets a new choice identity"
            );
        }
        let before = e.save();
        let rejected = e.submit(GameCommand {
            actor: old.observer,
            expected_revision: if owner == 1 {
                e.revision()
            } else {
                old_revision
            },
            dedup_id: e.save().last_dedup + 1,
            action: GameAction::Answer {
                choice_id: old.id,
                selection: vec![old.spec.options[0]],
            },
        });
        assert!(matches!(rejected, CommandOutcome::Rejected(_)));
        assert_eq!(e.save(), before);
        let s = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
            &e.save(),
        ))
        .unwrap();
        let mut r = Engine::restore_with_extensions(&s, &[&p]).unwrap();
        let event_cursor = e.log().len();
        for _ in 0..10 {
            let Some(c) = e.open_choice().cloned() else {
                break;
            };
            let selection = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            };
            answer(&mut e, selection.clone());
            answer(&mut r, selection);
        }
        assert_eq!(e.log().iter().filter(|l|matches!(l,engine_native::LogEvent::SkillTriggered{skill:"c331_heal",owner:p} if *p==owner)).count(),usize::from(owner!=0));
        assert_eq!(&e.log()[event_cursor..], r.log());
        let expected = e.save();
        let mut actual = r.save();
        actual.auth_generation = expected.auth_generation;
        assert_eq!(actual, expected);
    }
}

#[test]
fn c331_review_k1_window_authority_cannot_hide_in_vm_following() {
    use engine_native::MachineSnapshot as M;
    use rule_vm::bytecode::{caps, op};
    let mut p = pack(false, [true; 3], 3);
    let mut code = vec![op::CONST_U32];
    code.extend(0u32.to_le_bytes());
    code.push(op::CONST_U32);
    code.extend(1u32.to_le_bytes());
    code.extend([op::HOST_CHOICE, 2, op::HALT]);
    let m = &mut p.vm_skills[0].module;
    m.code = code;
    m.caps_mask = caps::REQUEST_CHOICE;
    m.max_stack = 8;
    m.logic_hash = rule_vm::BytecodeModule::compute_logic_hash(
        &m.name,
        &m.code,
        m.local_count,
        m.caps_mask,
        m.entry_pc,
    );
    let mut e = open(&p);
    choose(&mut e, "c331_heal");
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_choice");
    let mut s = e.save();
    let window = s
        .resolution
        .parks
        .iter()
        .find(|p| matches!(p.machine, M::TriggerOrder { .. }))
        .unwrap()
        .machine
        .clone();
    let gate = s
        .resolution
        .parks
        .iter_mut()
        .find(|p| matches!(p.machine, M::OptionalGate { .. }))
        .unwrap();
    let M::OptionalGate {
        inner: Some(inner), ..
    } = &mut gate.machine
    else {
        panic!("optional VM");
    };
    let M::Rvm { following, .. } = inner.as_mut() else {
        panic!("VM");
    };
    following.push(window);
    assert!(Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&s))
            .unwrap(),
        &[&p]
    )
    .is_err());
}
#[test]
fn c331_review_k1_every_window_slice_restores_without_a_lifecycle_gap() {
    let p = pack(false, [true; 3], 3);
    let mut e = ready(&p);
    e.set_time_slice(Some(1));
    submit(&mut e, 1, GameAction::NextPhase);
    for _ in 0..200 {
        let s = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
            &e.save(),
        ))
        .unwrap();
        let mut r = Engine::restore_with_extensions(&s, &[&p])
            .expect("every yielded boundary must remain canonical");
        r.set_time_slice(Some(1));
        if let Some(c) = e.open_choice().cloned() {
            let selected = c
                .spec
                .options
                .iter()
                .take(c.spec.min)
                .copied()
                .collect::<Vec<_>>();
            answer(&mut e, selected.clone());
            answer(&mut r, selected);
        } else if e.save().segment_open {
            e.resume();
            r.resume();
        } else {
            break;
        }
        let expected = e.save();
        let mut actual = r.save();
        actual.auth_generation = expected.auth_generation;
        if let (Some(actual), Some(expected)) =
            (&mut actual.segment_checkpoint, &expected.segment_checkpoint)
        {
            actual.auth_generation = expected.auth_generation;
        }
        assert!(
            actual == expected,
            "{:?}",
            format!("{actual:#?}")
                .lines()
                .zip(format!("{expected:#?}").lines())
                .find(|(a, b)| a != b)
        );
    }
    assert!(!e.save().segment_open);
    assert!(e.open_choice().is_none());
}

#[test]
fn c331_review_k2_swapped_seats_nested_grant_rejoins_the_frozen_ancestor() {
    use engine_native::event::Timing;
    let mut p = pack(false, [false; 3], 3);
    let nested=content_compiler::compile_source("skill c331_late(4200000092) locked { trigger phase_play_end owner_is_target { effect storage_set(self_skill,1,1); } }").unwrap();
    p.vm_skills.extend(
        content_native_loader::bridge_vm_skill_decls(&nested)
            .unwrap()
            .decls,
    );
    let probe = Engine::new_with_extensions(1, 4, &[&p]).unwrap();
    let late = probe.catalog().skill_by_name("c331_late").unwrap();
    let grant=content_compiler::compile_source(&format!("skill c331_nested_grant(4200000093) {{ trigger damage owner_is_target optional {{ effect add_skill(self,{late}); }} }}")).unwrap();
    p.vm_skills.extend(
        content_native_loader::bridge_vm_skill_decls(&grant)
            .unwrap()
            .decls,
    );
    let hurt=content_compiler::compile_source("skill c331_heal(4200000010) { trigger phase_play_end owner_is_target { effect deal_damage(self,self,1); } }").unwrap();
    p.vm_skills[0] = content_native_loader::bridge_vm_skill_decls(&hurt)
        .unwrap()
        .decls
        .remove(0);
    replace_hurt(&mut p, "effect set_turn_state(self,3);");
    p.vm_skills[1].trigger.timing = Timing::First;
    p.vm_skills[1].filter = engine_native::rvm_bridge::VmFilterPolicy::Always;
    let mut e = ready(&p);
    let first = e.catalog().skill_by_name("c331_hurt").unwrap();
    e.state_core_mut().skills.remove(&(1, first));
    e.state_core_mut().skills.remove(&(1, late));
    e.debug_grant(0, "c331_hurt");
    submit(&mut e, 1, GameAction::NextPhase);
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "discard_to_limit")
    {
        let c = e.open_choice().unwrap().clone();
        answer(
            &mut e,
            c.spec.options.iter().take(c.spec.min).copied().collect(),
        );
    }
    choose(&mut e, "c331_heal");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    e.state_core_mut().swap_seat(1, 3);
    let s =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    let mut r = Engine::restore_with_extensions(&s, &[&p]).unwrap();
    let cursor = e.log().len();
    answer(&mut e, vec![ChoiceOption::Confirm]);
    answer(&mut r, vec![ChoiceOption::Confirm]);
    assert!(e
        .open_choice()
        .unwrap()
        .spec
        .options
        .contains(&ChoiceOption::Skill(late)));
    choose(&mut e, "c331_late");
    choose(&mut r, "c331_late");
    assert_eq!(&e.log()[cursor..], r.log());
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
}

#[test]
fn c331_huanle_equipment_priority_waits_for_normal_owner_peers() {
    use engine_native::content::host::{HostBehavior, HostSkillDecl};
    let mut p = pack(false, [false; 3], 2);
    p.host_skills.push(HostSkillDecl {
        name: "c331_jiuwei",
        behavior: HostBehavior::PhaseEndRecoverOrDraw,
    });
    let mut e = ready(&p);
    e.debug_grant(1, "c331_jiuwei");
    submit(&mut e, 1, GameAction::NextPhase);
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "discard_to_limit")
    {
        let c = e.open_choice().unwrap().clone();
        answer(
            &mut e,
            c.spec.options.iter().take(c.spec.min).copied().collect(),
        );
    }
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "trigger_order");
    assert_eq!(
        c.spec.options.len(),
        2,
        "upstream equipSkill priority -25 is below the two priority-zero skills"
    );
    let s =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    let mut r = Engine::restore_with_extensions(&s, &[&p]).unwrap();
    let cursor = e.log().len();
    choose(&mut e, "c331_heal");
    choose(&mut r, "c331_heal");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    answer(&mut e, vec![ChoiceOption::Confirm]);
    answer(&mut r, vec![ChoiceOption::Confirm]);
    assert_eq!(hp(&e), 4);
    assert_eq!(&e.log()[cursor..], r.log());
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
}

#[test]
fn c331_review_owner_ring_cannot_omit_a_seat_without_current_candidates() {
    let p = pack(false, [false; 3], 3);
    let e = open(&p);
    let mut snap = e.save();
    Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
            .unwrap(),
        &[&p],
    )
    .unwrap();
    let window = snap
        .resolution
        .parks
        .iter_mut()
        .find_map(|p| match &mut p.machine {
            engine_native::MachineSnapshot::TriggerOrder { owners, .. } => Some(owners),
            _ => None,
        })
        .unwrap();
    assert_eq!(window.len(), e.state_core().players.len());
    let removed = window.pop().unwrap();
    assert!(snap
        .resolution
        .stack
        .iter()
        .all(|f| f.queue.iter().all(|q| q.owner != removed)));
    assert!(
        Engine::restore_with_extensions(
            &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
                .unwrap(),
            &[&p]
        )
        .is_err(),
        "every player is a future owner bucket even without a current candidate"
    );
}

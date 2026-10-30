//! J2 real author DSL through the production bridge, with byte checkpoints at every choice.
mod nc08_common;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand};

fn pack(source: &str, name: &'static str) -> ExtensionPack {
    let pkg = content_compiler::compile_source_named(name, source).unwrap();
    let vm = content_native_loader::bridge_vm_skill_decls(&pkg)
        .expect("author package ABI and declarations supported");
    let known = vm.decls.iter().map(|d| d.name.clone()).collect();
    let generals = content_native_loader::bridge_generals_with_known(&serde_json::json!({
        "pack":{"name":name,"version":1}, "generals":[{"name":"j2-author","hp":4,"gender":"male","faction":"qun",
        "skills":vm.decls.iter().map(|d| serde_json::json!({"name":d.name})).collect::<Vec<_>>() }]
    }).to_string(), &known).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name,
            version: 1,
            description: "J2",
        },
        generals: generals.generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: vm.decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: nc08_common::next_dedup(),
        action,
    })
}

fn answer(e: &mut Engine, pack: &ExtensionPack, selected: Vec<O>) {
    let choice = e.open_choice().cloned().expect("choice");
    let before = e.save();
    let bytes = engine_native::archive::encode_archive(&before);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("byte decode");
    assert_eq!(decoded, before);
    let mut restored = Engine::restore_with_extensions(&decoded, &[pack]).expect("byte restore");
    assert_eq!(restored.open_choice(), Some(&choice));
    assert_eq!(restored.state_hash(), e.state_hash());
    let dedup = nc08_common::next_dedup();
    for g in [&mut *e, &mut restored] {
        let unchanged = g.save();
        let bad = g.submit(GameCommand {
            actor: choice.observer,
            expected_revision: g.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![O::Tag(255)],
            },
        });
        assert!(matches!(bad, CommandOutcome::Rejected(_)), "{bad:?}");
        assert_eq!(
            g.save(),
            unchanged,
            "invalid answer preserves RNG/continuation and retry key"
        );
    }
    let original_log = e.log().len();
    let restored_log = restored.log().len();
    for g in [&mut *e, &mut restored] {
        let out = g.submit(GameCommand {
            actor: choice.observer,
            expected_revision: g.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: selected.clone(),
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    assert_eq!(
        e.save().core,
        restored.save().core,
        "complete core and RNG match"
    );
    assert_eq!(
        e.save().resolution,
        restored.save().resolution,
        "event counters and continuation match"
    );
    assert_eq!(e.save().last_dedup, restored.save().last_dedup);
    assert_eq!(
        restored.save().auth_generation,
        before.auth_generation + 1,
        "restore rotates the existing session authorization generation"
    );
    assert_eq!(
        &e.log()[original_log..],
        &restored.log()[restored_log..],
        "event trajectory matches"
    );
    let settled = e.save();
    let duplicate = e.submit(GameCommand {
        actor: choice.observer,
        expected_revision: e.revision(),
        dedup_id: dedup,
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: selected,
        },
    });
    assert!(matches!(duplicate, CommandOutcome::Rejected(_)));
    assert_eq!(e.save(), settled, "retry cannot repeat effects");
}

fn engine(pack: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(99107860, 4, &[pack]).unwrap();
    e.assign_general_named(0, "j2-author").unwrap();
    nc08_common::advance_to_play(&mut e, 0);
    e
}

fn activate(e: &mut Engine, name: &str) {
    let skill = e.catalog().skill_by_name(name).unwrap();
    let out = submit(e, 0, GameAction::ActivateSkill { skill });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
}

#[test]
fn danshou_all_four_options_settle_then_discard_ordinal_with_every_choice_checkpoint() {
    let pack = pack(
        include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill"),
        "j2-danshou",
    );
    for option in 1..=4 {
        let mut e = engine(&pack);
        e.debug_clear_hand(0);
        for name in ["shan", "tao", "nullify", "sha", "shan", "tao", "sha"] {
            e.debug_place(0, name);
        }
        e.debug_clear_hand(1);
        let target_card = e.debug_place(1, "shan");
        let hand = e.state_core().player(0).hand.len();
        let target_hand = e.state_core().player(1).hand.len();
        let hp = e.debug_hp(1);
        activate(&mut e, "ol_jie_danshou");
        assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_danshou_option");
        answer(&mut e, &pack, vec![O::Tag(option)]);
        answer(&mut e, &pack, vec![O::Player(1)]);
        if option <= 2 {
            answer(&mut e, &pack, vec![O::Card(target_card)]);
        }
        let choice = e
            .open_choice()
            .cloned()
            .expect("ordinal discard after effect settlement");
        assert_eq!(choice.spec.prompt, "rvm_danshou_discard");
        assert_eq!(
            (choice.spec.min, choice.spec.max),
            (option as usize, option as usize)
        );
        if option == 3 {
            assert_eq!(e.debug_hp(1), hp - 1, "full damage settles before discard");
        }
        let picked = choice
            .spec
            .options
            .iter()
            .rev()
            .take(option as usize)
            .copied()
            .collect();
        answer(&mut e, &pack, picked);
        assert!(e.open_choice().is_none());
        let gain = if option == 2 {
            1
        } else if option == 4 {
            2
        } else {
            0
        };
        assert_eq!(
            e.state_core().player(0).hand.len(),
            hand + gain - option as usize
        );
        assert_eq!(
            e.state_core().player(1).hand.len(),
            if option <= 2 {
                target_hand - 1
            } else if option == 4 {
                target_hand + 2
            } else {
                target_hand
            }
        );
        activate(&mut e, "ol_jie_danshou");
        assert!(
            !e.open_choice()
                .unwrap()
                .spec
                .options
                .contains(&O::Tag(option)),
            "each option once per Play"
        );
        answer(&mut e, &pack, vec![]);
    }
}

#[test]
fn fenghe_actual_give_and_filtered_recast_keep_damage_prevented_across_checkpoints() {
    let source = r#"skill j2_fenghe(99107861) locked {
      trigger damage_before owner_is_target { effect prevent_current_damage(); effect choose_give_or_recast_one_hand(self, 0, 0, 0); }
    }"#;
    let pack = pack(source, "j2-fenghe");
    for recast in [false, true] {
        let mut e = engine(&pack);
        // The chosen general owns the skill on seat 0; seat 1 attacks it next turn.
        nc08_common::advance_to_play(&mut e, 1);
        e.debug_clear_hand(0);
        let card = e.debug_place(0, if recast { "chain" } else { "shan" });
        e.debug_clear_hand(1);
        let sha = e.debug_place(1, "sha");
        let hp = e.debug_hp(0);
        let out = submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: sha,
                targets: vec![0],
            },
        );
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        let response = e.open_choice().unwrap().spec.prompt;
        if response == "respond_shan" {
            answer(&mut e, &pack, vec![O::Pass]);
        }
        assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_fenghe_card");
        answer(&mut e, &pack, vec![O::Card(card)]);
        assert_eq!(
            e.open_choice().unwrap().spec.options.contains(&O::Pass),
            recast,
            "only recastable cards expose recast"
        );
        answer(
            &mut e,
            &pack,
            vec![if recast { O::Pass } else { O::Player(2) }],
        );
        nc08_common::drain(&mut e);
        assert_eq!(e.debug_hp(0), hp, "prevented damage never commits");
        if recast {
            assert!(e.state_core().discard.contains(&card));
            assert_eq!(e.state_core().player(0).hand.len(), 1);
        } else {
            assert!(e.state_core().player(2).hand.contains(&card));
        }
    }
}

#[test]
fn pingkou_counts_real_skips_and_settles_each_damage_before_drawing() {
    let source = r#"skill j2_skips(99107862) locked {
      trigger phase_draw owner_is_target { effect skip_current_phase(self); }
      trigger phase_discard owner_is_target { effect skip_current_phase(self); }
    }
    skill ol_pingkou(2376678765) { trigger phase_end owner_is_target_skipped_phase_count_positive optional {
      effect choose_other_targets_by_skipped_phase_count_damage_draw(self, 1);
    }}"#;
    let pack = pack(source, "j2-pingkou");
    let mut e = engine(&pack);
    for seat in 1..4 {
        e.debug_clear_hand(seat);
    }
    for _ in 0..2 {
        let out = submit(&mut e, 0, GameAction::NextPhase);
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    answer(&mut e, &pack, vec![O::Confirm]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_pingkou_targets");
    assert_eq!(
        e.open_choice().unwrap().spec.max,
        2,
        "draw and discard were actually skipped"
    );
    let hand = e.state_core().player(0).hand.len();
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    answer(&mut e, &pack, vec![O::Player(2), O::Player(1)]);
    assert_eq!(e.debug_hp(1), hp1 - 1);
    assert_eq!(e.debug_hp(2), hp2 - 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand + 2);
}

#[test]
fn content_callbacks_and_effect_codec_work_before_host_abi_is_advertised() {
    use engine_native::machine::{Ctx, Flow};
    use engine_native::resolution::{Resolution, StepOutcome};
    use rule_api::{EffectRequest as E, PlayerId, SkillId};
    let effects = [
        E::ChooseGiveOrRecastOneHand {
            player: PlayerId::new(1),
            skill: SkillId::new(200),
            require_skill_def: 0,
            grant_skill_def: 0,
            grant_mark_def: 0,
        },
        E::ResolveOlJieDanshou {
            player: PlayerId::new(1),
            skill: SkillId::new(200),
        },
        E::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player: PlayerId::new(1),
            amount: 1,
            skill: SkillId::new(200),
        },
    ];
    let mut bytes = Vec::new();
    for effect in &effects {
        engine_native::rvm_ops::accept_effect(effect, 4).unwrap();
    }
    engine_native::rvm_ops::enc_effects(&mut bytes, &effects);
    assert_eq!(
        engine_native::rvm_ops::dec_effects(&bytes).unwrap(),
        effects
    );
    for index in 0..3 {
        let e = Engine::new(99107869, 4);
        let mut core = e.save().core;
        core.skills.clear();
        core.vm_aux
            .flags
            .insert((0, "vm_skipped_phase_count".to_string()), 2);
        let catalog = e.catalog();
        let mut res = Resolution::new(10000);
        if index == 0 {
            let mut damage = engine_native::machine::damage_data(Some(1), 0, 1);
            damage.prevented = true;
            engine_native::resolution::push_frame(
                &mut res,
                engine_native::event::EventData::Damage(damage),
            );
        }
        let mut log = vec![];
        let mut state = vec![];
        let mut choices = 0;
        for _ in 0..40 {
            let flow = (catalog.hooks.vm_commit)(
                &mut Ctx {
                    core: &mut core,
                    res: &mut res,
                    catalog,
                    out: &mut log,
                },
                "j2_callback",
                &effects[index],
                &mut state,
            )
            .unwrap();
            match flow {
                Flow::Done => break,
                Flow::WaitChoice { spec, .. } => {
                    choices += 1;
                    let answer = match spec.prompt {
                        "rvm_fenghe_card" => vec![spec.options[0]],
                        "rvm_fenghe_destination" => vec![O::Player(1)],
                        "rvm_danshou_option" => vec![O::Tag(3)],
                        "rvm_danshou_target" => vec![O::Player(1)],
                        "rvm_danshou_discard" => {
                            spec.options.iter().take(spec.min).copied().collect()
                        }
                        "rvm_pingkou_targets" => vec![O::Player(1), O::Player(2)],
                        other => panic!("unexpected {other}"),
                    };
                    assert!(spec.accepts(&answer));
                    res.answer = Some(answer);
                }
                Flow::Yield => {
                    res.answer = None;
                    assert_eq!(
                        engine_native::resolution::run(
                            &mut core, &mut res, catalog, &mut log, None
                        ),
                        StepOutcome::Done
                    );
                }
            }
        }
        assert_eq!(choices, [2, 3, 1][index]);
        if index == 0 {
            assert_eq!(core.player(0).hand.len(), 3);
            assert_eq!(core.player(1).hand.len(), 5);
        }
        if index == 1 {
            assert_eq!(core.player(1).hp, 3);
            assert_eq!(core.player(0).hand.len(), 1);
        }
        if index == 2 {
            assert_eq!(core.player(1).hp, 3);
            assert_eq!(core.player(2).hp, 3);
            assert_eq!(core.player(0).hand.len(), 6);
        }
    }
}

#[test]
fn pingkou_prevention_and_nested_dying_save_or_death_resume_without_duplicate_draw() {
    let source = r#"skill j2_skips(99107862) locked {
      trigger phase_draw owner_is_target { effect skip_current_phase(self); }
      trigger phase_discard owner_is_target { effect skip_current_phase(self); }
    }
    skill j2_prevent(99107863) locked { trigger damage_before owner_is_target { effect prevent_current_damage(); } }
    skill ol_pingkou(2376678765) { trigger phase_end owner_is_target_skipped_phase_count_positive optional {
      effect choose_other_targets_by_skipped_phase_count_damage_draw(self, 1);
    }}"#;
    let pack = pack(source, "j2-pingkou-dying");
    for save in [false, true] {
        let mut e = engine(&pack);
        e.assign_general_named(1, "j2-author").unwrap();
        for seat in 0..4 {
            e.debug_clear_hand(seat);
        }
        let tao = save.then(|| e.debug_place(0, "tao"));
        e.debug_set_hp(2, 1);
        for _ in 0..2 {
            assert!(!matches!(
                submit(&mut e, 0, GameAction::NextPhase),
                CommandOutcome::Rejected(_)
            ));
        }
        answer(&mut e, &pack, vec![O::Confirm]);
        answer(&mut e, &pack, vec![O::Player(2), O::Player(1)]);
        let mut saved = false;
        for _ in 0..12 {
            let Some(choice) = e.open_choice().cloned() else {
                break;
            };
            assert_eq!(choice.spec.prompt, "dying_save");
            let selection = if choice.observer == 0 && !saved && tao.is_some() {
                saved = true;
                vec![O::Card(tao.unwrap())]
            } else {
                vec![O::Pass]
            };
            answer(&mut e, &pack, selection);
        }
        assert!(e.open_choice().is_none());
        assert_eq!(e.debug_hp(1), 4, "prevention is not actual damage");
        assert_eq!(e.state_core().player(2).alive, save);
        assert_eq!(
            e.state_core().player(0).hand.len(),
            1,
            "only actual damaged target grants one draw"
        );
        assert!(!e
            .state_core()
            .vm_aux
            .flags
            .keys()
            .any(|(_, key)| key.starts_with("vm_damage_receipt:")));
    }
}

#[test]
fn danshou_insufficient_discard_cancel_and_next_play_phase_reset() {
    let pack = pack(
        include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill"),
        "j2-danshou-boundaries",
    );
    let mut e = engine(&pack);
    e.debug_clear_hand(0);
    let card = e.debug_place(0, "shan");
    activate(&mut e, "ol_jie_danshou");
    answer(&mut e, &pack, vec![]);
    assert!(e.state_core().player(0).hand.contains(&card));
    activate(&mut e, "ol_jie_danshou");
    answer(&mut e, &pack, vec![O::Tag(3)]);
    answer(&mut e, &pack, vec![O::Player(1)]);
    assert_eq!(
        e.open_choice().unwrap().spec.min,
        1,
        "ordinal 3 caps at one available card"
    );
    answer(&mut e, &pack, vec![O::Card(card)]);
    let skill = e.catalog().skill_by_name("ol_jie_danshou").unwrap();
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill }),
            CommandOutcome::Rejected(_)
        ),
        "empty owner cannot activate"
    );
    nc08_common::advance_to_phase(&mut e, 1, engine_native::Phase::Play);
    nc08_common::advance_to_play(&mut e, 0);
    activate(&mut e, "ol_jie_danshou");
    assert!(
        e.open_choice().unwrap().spec.options.contains(&O::Tag(3)),
        "new Play phase resets option identity"
    );
}

#[test]
fn fenghe_rewards_only_actual_give_with_owned_yingba_and_living_damage_source() {
    for mode in 0..4 {
        let mut source = String::from(
            r#"skill j2_fenghe(99107861) locked {
          trigger damage_before owner_is_target { effect prevent_current_damage(); effect choose_give_or_recast_one_hand(self, 99107864, 99107865, 99107865); }
        }
        skill j2_yingba(99107864) { trigger phase_play owner_is_target { effect storage_set(self_skill, 9, 1); } }
        skill j2_mark(99107865) locked { trigger phase_end owner_is_target { effect storage_set(self_skill, 9, 1); } }"#,
        );
        if mode == 3 {
            source.push_str("skill j2_after_give(99107866) locked { trigger cards_given owner_alive_gave_cards_to_other_alive { effect kill_player(event_player, self); } }");
        }
        let pack = pack(&source, "j2-fenghe-reward");
        let mut e = engine(&pack);
        let yingba = e.catalog().skill_by_name("j2_yingba_phase_play").unwrap();
        if mode == 1 {
            e.state_core_mut().skills.remove(&(0, yingba));
        } else {
            e.state_core_mut()
                .skills
                .get_mut(&(0, yingba))
                .unwrap()
                .enabled = false;
        }
        nc08_common::advance_to_play(&mut e, 1);
        e.debug_clear_hand(0);
        let card = e.debug_place(0, if mode == 2 { "chain" } else { "shan" });
        e.debug_clear_hand(1);
        let sha = e.debug_place(1, "sha");
        assert!(!matches!(
            submit(
                &mut e,
                1,
                GameAction::PlayCard {
                    card: sha,
                    targets: vec![0]
                }
            ),
            CommandOutcome::Rejected(_)
        ));
        if e.open_choice().unwrap().spec.prompt == "respond_shan" {
            answer(&mut e, &pack, vec![O::Pass]);
        }
        answer(&mut e, &pack, vec![O::Card(card)]);
        answer(
            &mut e,
            &pack,
            vec![if mode == 2 { O::Pass } else { O::Player(1) }],
        );
        assert_eq!(
            e.state_core()
                .vm_aux
                .marks
                .get(&(1, 99107865))
                .copied()
                .unwrap_or(0),
            if mode == 0 { 1 } else { 0 }
        );
        assert!(
            !e.state_core().vm_aux.marks.contains_key(&(2, 99107865)),
            "reward belongs to damage source"
        );
        if mode == 3 {
            assert!(
                !e.state_core().player(1).alive,
                "give-trigger settles before reward source liveness check"
            );
        }
    }
}

#[test]
fn actual_pinghe_source_keeps_max_hp_hand_limit_and_prevention_timing() {
    let pack = pack(
        include_str!("../../../content/original/ported/shiji/pinghe.skill"),
        "j2-actual-pinghe",
    );
    let mut e = engine(&pack);
    e.debug_set_hp(0, 1);
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        4,
        "actual declared modifier uses max HP"
    );
    let def = e.catalog().skill_by_name("pinghe_damage_before").unwrap();
    for source in [None, Some(0), Some(2)] {
        e.state_core_mut().player_mut(2).alive = false;
        let event = engine_native::event::EventData::Damage(engine_native::machine::damage_data(
            source, 0, 1,
        ));
        assert!(!e.catalog().skills[def as usize].behavior.filter(
            &engine_native::catalog::QueryCtx {
                state: e.state_core(),
                catalog: e.catalog()
            },
            0,
            &event
        ));
    }
    e.state_core_mut().player_mut(2).alive = true;
    nc08_common::advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let card = e.debug_place(0, "shan");
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    assert!(!matches!(
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: sha,
                targets: vec![0]
            }
        ),
        CommandOutcome::Rejected(_)
    ));
    if e.open_choice().unwrap().spec.prompt == "respond_shan" {
        answer(&mut e, &pack, vec![O::Pass]);
    }
    assert_eq!(
        e.state_core().player(0).max_hp,
        3,
        "max HP loss occurs before choosing"
    );
    assert_eq!(e.debug_hp(0), 1, "damage is prevented before choosing");
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 3);
    answer(&mut e, &pack, vec![O::Card(card)]);
    answer(&mut e, &pack, vec![O::Player(2)]);
    assert_eq!(e.debug_hp(0), 1);
}

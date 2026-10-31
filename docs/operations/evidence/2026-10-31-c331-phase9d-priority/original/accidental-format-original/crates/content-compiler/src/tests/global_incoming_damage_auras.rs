use super::*;

#[test]
fn compile_global_incoming_damage_auras() {
    let package = compile_source(
            "skill stdhuoshui(3517180722) locked { modifier global_damage_taken_amount add queried_player_judge_nonempty_other_than_owner; }\nskill jyyuhuo(2185491588) locked { modifier global_damage_taken_final_amount(normal) add -queried_player_linked_other_than_owner; modifier global_damage_taken_final_amount(fire) add queried_player_linked_other_than_owner; modifier global_damage_taken_final_amount(thunder) add queried_player_linked_other_than_owner; }",
        )
        .unwrap();
    assert_eq!(package.modifier_index.len(), 4);
    assert_eq!(
        package.modifier_index[0].target,
        "global_damage_taken_amount"
    );
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("queried_player_judge_nonempty_other_than_owner")
    );
    assert_eq!(
        package.modifier_index[1].target,
        "global_damage_taken_final_amount"
    );
    assert_eq!(
        package.modifier_index[1].selector.as_deref(),
        Some("normal")
    );
    assert_eq!(package.modifier_index[1].value, -1);
    assert!(compile_source(
            "skill bad(1) { modifier damage_taken_amount add queried_player_judge_nonempty_other_than_owner; }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { modifier global_damage_taken_amount add queried_player_linked_other_than_owner; }"
        )
        .is_err());
}

#[test]
fn compile_global_hand_limit_aura() {
    let package = compile_source(
            "skill xianghai(3356961773) { modifier global_hand_limit add -queried_player_other_than_owner; }",
        )
        .unwrap();
    let modifier = &package.modifier_index[0];
    assert_eq!(modifier.target, "global_hand_limit");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("queried_player_other_than_owner")
    );
    assert_eq!(modifier.value, -1);
    assert!(compile_source(
        "skill bad(1) { modifier hand_limit add queried_player_other_than_owner; }"
    )
    .is_err());
}

#[test]
fn compile_global_distance_to_aura() {
    let package = compile_source(
            "skill zhenwei_three(3450139382) locked { modifier global_distance_to add queried_source_enemy_owner_ally_target_other_than_owner; }",
        )
        .unwrap();
    let modifier = &package.modifier_index[0];
    assert_eq!(modifier.target, "global_distance_to");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("queried_source_enemy_owner_ally_target_other_than_owner")
    );
    assert_eq!(modifier.value, 1);
    assert!(compile_source(
            "skill bad(1) { modifier distance_to add queried_source_enemy_owner_ally_target_other_than_owner; }"
        )
        .is_err());
}

#[test]
fn compile_owner_skill_mark_modifier_value() {
    let pkg = compile_source(
            "skill marked(17) { modifier hand_limit add -owner_skill_mark; modifier sha_use_limit add owner_skill_mark; modifier damage_amount(sha) add owner_skill_mark; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 3);
    assert_eq!(pkg.modifier_index[0].value, -1);
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("owner_skill_mark")
    );
    assert_eq!(pkg.modifier_index[1].value, 1);
    assert_eq!(pkg.modifier_index[2].target, "damage_amount");
    assert_eq!(pkg.modifier_index[2].selector.as_deref(), Some("sha"));
    assert_eq!(
        pkg.modifier_index[2].value_source.as_deref(),
        Some("owner_skill_mark")
    );
    assert_eq!(pkg.modifier_index[2].value, 1);
}

#[test]
fn compile_mark_count_with_full_u32_identifier() {
    let package = compile_source(
            "skill marked(1) { trigger phase_end owner_mark_positive_3247391666 { let marks = mark_count(self, 3247391666); effect lose_hp(self, marks); } }",
        )
        .unwrap();
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_mark_positive_3247391666"
    );
}

#[test]
fn compile_mark_id_above_i32_max() {
    compile_source(
            "skill marked(3847328245) { trigger card_used owner_source_ordinary_trick { effect add_mark(self, 3847328245, 1); effect remove_mark(self, 3847328245, 0); } }",
        )
        .unwrap();
}

#[test]
fn compile_alive_group_count_modifier_value() {
    let pkg = compile_source(
            "skill groups(19) { modifier distance_to add alive_group_count; modifier hand_limit add -alive_group_count; }",
        )
        .unwrap();
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("alive_group_count")
    );
    assert_eq!(pkg.modifier_index[0].value, 1);
    assert_eq!(pkg.modifier_index[1].value, -1);
}

#[test]
fn compile_card_action_count_attack_range_trigger() {
    let package = compile_source(
            "skill gzjili(3565936884) { trigger card_used owner_card_action_count_eq_attack_range optional { effect draw_cards(self, card_action_count(self)); } }",
        )
        .unwrap();
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_card_action_count_eq_attack_range"
    );
    assert!(package.trigger_index[0].optional);
}

#[test]
fn compile_alive_female_count_modifier_value() {
    let pkg = compile_source(
        "skill xiefang(3450775931) { modifier distance_from add -alive_female_count; }",
    )
    .unwrap();
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("alive_female_count")
    );
    assert_eq!(pkg.modifier_index[0].value, -1);
}

#[test]
fn compile_alive_wounded_threshold_damage_modifier_value() {
    let pkg = compile_source(
            "skill xinfu_xingzhao3(2896246690) locked { modifier damage_amount add alive_wounded_count_zero_or_at_least_four; }",
        )
        .unwrap();
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("alive_wounded_count_zero_or_at_least_four")
    );
    assert_eq!(pkg.modifier_index[0].value, 1);
}

#[test]
fn compile_parameterized_alive_wounded_threshold_phase_modifiers() {
    let pkg = compile_source(
            "skill xingzhao(2244503587) locked { modifier draw_phase_scry_four_split_two max alive_wounded_count_at_least_1; modifier skip_judge_phase max alive_wounded_count_at_least_3; modifier skip_discard_phase max alive_wounded_count_at_least_3; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 3);
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("alive_wounded_count_at_least_1")
    );
    assert!(pkg.modifier_index[1..].iter().all(|modifier| {
        modifier.value_source.as_deref() == Some("alive_wounded_count_at_least_3")
    }));

    assert!(compile_source(
        "skill bad(1) { modifier skip_judge_phase max alive_wounded_count_at_least_0; }",
    )
    .is_err());
    assert!(compile_source(
        "skill bad(1) { modifier damage_amount add alive_wounded_count_at_least_2; }",
    )
    .is_err());
}

#[test]
fn compile_alive_other_count_capped_four_modifier_value() {
    let pkg = compile_source(
            "skill twyuanbu(3356399020) { modifier hand_limit add alive_other_count_capped_four; modifier distance_from add alive_other_count_capped_four; modifier distance_to add alive_other_count_capped_four; }",
        )
        .unwrap();
    assert!(pkg
        .modifier_index
        .iter()
        .all(|modifier| modifier.value_source.as_deref() == Some("alive_other_count_capped_four")));
}

#[test]
fn compile_owner_no_mount_modifier_value() {
    let pkg = compile_source(
            "skill twshenxing(4258102225) { modifier distance_from add -owner_no_mount; modifier hand_limit add owner_no_mount; }",
        )
        .unwrap();
    assert!(pkg
        .modifier_index
        .iter()
        .all(|modifier| modifier.value_source.as_deref() == Some("owner_no_mount")));
    assert_eq!(pkg.modifier_index[0].value, -1);
    assert_eq!(pkg.modifier_index[1].value, 1);
}

#[test]
fn compile_armor_empty_sha_target_protection() {
    let pkg = compile_source(
            "skill armor(21) { modifier card_target_cancel(sha_black) override owner_armor_empty; modifier card_target_cancel(sha_elemental) override owner_armor_empty; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 2);
    assert!(pkg
        .modifier_index
        .iter()
        .all(|modifier| modifier.value_source.as_deref() == Some("owner_armor_empty")));
}

#[test]
fn compile_owner_hp_modifier_values() {
    let pkg = compile_source(
            "skill hp(20) { modifier hand_limit add owner_max_hp; modifier hand_limit add -owner_hp; modifier distance_from add -owner_hp_minus_one; modifier distance_to add owner_lost_hp_minus_one; }",
        )
        .unwrap();
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("owner_max_hp")
    );
    assert_eq!(
        pkg.modifier_index[1].value_source.as_deref(),
        Some("owner_hp")
    );
    assert_eq!(pkg.modifier_index[1].value, -1);
    assert_eq!(
        pkg.modifier_index[2].value_source.as_deref(),
        Some("owner_hp_minus_one")
    );
    assert_eq!(
        pkg.modifier_index[3].value_source.as_deref(),
        Some("owner_lost_hp_minus_one")
    );
}

#[test]
fn compile_sha_target_limit_modifier() {
    let pkg = compile_source(
        "skill shenji(18) { modifier sha_target_limit add 2; modifier sha_use_limit add 1; }",
    )
    .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "sha_target_limit");
    assert_eq!(pkg.modifier_index[0].value, 2);
}

#[test]
fn compile_card_target_policy_modifiers() {
    let pkg = compile_source(
            "skill all(19) { modifier card_targets_all(sha) override 1; } skill self_only(20) { modifier card_targets_self_only override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "card_targets_all");
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("sha"));
    assert_eq!(pkg.modifier_index[1].target, "card_targets_self_only");
}

#[test]
fn card_selector_changes_package_logic_hash() {
    let trick =
        compile_source("skill range(16) { modifier card_distance_unlimited(trick) override 1; }")
            .unwrap();
    let basic =
        compile_source("skill range(16) { modifier card_distance_unlimited(basic) override 1; }")
            .unwrap();
    assert_ne!(trick.manifest.logic_hash, basic.manifest.logic_hash);
}

#[test]
fn run_compiled_jianxiong() {
    let pkg = compile_source(JIANXIONG).unwrap();
    let module = &pkg.modules[0];
    let mut vm = RuleVm::new();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 2,
            event_player: 2,
            event_source: 1,
            event_card: 0,
            event_amount: 0,
        },
        ..Default::default()
    };
    let r = vm.execute(&vmod(module), &mut host, 10_000).unwrap();
    assert!(matches!(r, ExecResult::Halted { .. }));
    assert_eq!(host.effects.len(), 1);
}

#[test]
fn compile_and_run_relation_query() {
    use rule_api::RelationTag;
    // Retaliate: on taking damage, draw a card only if the source is an enemy.
    // relation() enemy encodes to 2 (see rule_vm::bytecode::relation_code).
    let src = r#"
skill fanji(7) {
  trigger damage owner_is_target {
    if relation(self, event_source) == 2 {
      effect draw_cards(self, 1);
    }
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let module = &pkg.modules[0];

    // Enemy source → draws.
    let mut enemy_host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 2,
            event_player: 2,
            event_source: 1,
            event_card: 0,
            event_amount: 0,
        },
        query_stub_relation: Some(RelationTag::Enemy),
        ..Default::default()
    };
    let mut vm_enemy = RuleVm::new();
    let r = vm_enemy
        .execute(&vmod(module), &mut enemy_host, 10_000)
        .unwrap();
    assert!(matches!(r, ExecResult::Halted { .. }));
    assert_eq!(enemy_host.effects.len(), 1, "enemy source should draw");

    // Ally source → no draw.
    let mut ally_host = CollectingHost {
        query_stub_relation: Some(RelationTag::Ally),
        ..Default::default()
    };
    let mut vm_ally = RuleVm::new();
    let _ = vm_ally
        .execute(&vmod(module), &mut ally_host, 10_000)
        .unwrap();
    assert_eq!(ally_host.effects.len(), 0, "ally source should not draw");
}

#[test]
fn compile_and_run_add_skill_and_enter_dying() {
    use rule_api::EffectRequest;
    // Awaken-style: on phase begin, gain a skill and mark self dying (contrived
    // but exercises both new host effects end-to-end).
    let src = r#"
skill juexing(8) {
  trigger phase_begin owner_is_target {
    effect add_skill(self, 3671794947);
    effect enter_dying(self);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let module = &pkg.modules[0];
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 3,
            event_player: 3,
            event_source: 3,
            event_card: 0,
            event_amount: 0,
        },
        ..Default::default()
    };
    let mut vm = RuleVm::new();
    let r = vm.execute(&vmod(module), &mut host, 10_000).unwrap();
    assert!(matches!(r, ExecResult::Halted { .. }));
    assert_eq!(host.effects.len(), 2);
    assert!(matches!(
        host.effects[0],
        EffectRequest::AddSkill { skill_def, .. } if skill_def.raw() == 3_671_794_947
    ));
    assert!(matches!(host.effects[1], EffectRequest::EnterDying { .. }));
}

#[test]
fn compile_and_run_kill_player() {
    use rule_api::EffectRequest;
    let src = r#"
skill wansha(15) locked {
  trigger dying always {
    effect kill_player(event_player, event_source);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 1,
            event_player: 2,
            event_source: 3,
            event_card: 0,
            event_amount: 0,
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::KillPlayer { player, killer }]
            if player.raw() == 2 && killer.raw() == 3
    ));
}

#[test]
fn compile_and_run_turn_over() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill jushou(3729566871) {
  trigger phase_end owner_is_target optional {
    effect draw_cards(self, 3);
    effect turn_over(self);
  }
}
"#,
    )
    .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 7,
            event_player: 7,
            event_source: 7,
            event_card: 0,
            event_amount: 0,
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DrawCards { player, count: 3 },
            EffectRequest::TurnOver { player: turned }
        ] if player.raw() == 7 && turned.raw() == 7
    ));
}

#[test]
fn restore_character_state_requires_abi93_and_emits_typed_effect() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill oldniepan(3136864229) {
  trigger dying owner_is_target optional {
    effect discard_all_hand_equip_judge(self);
    effect restore_character_state(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_93
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_player: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DiscardAllHandEquipJudge { player: discarded },
            EffectRequest::RestoreCharacterState { player: restored }
        ] if discarded.raw() == 7 && restored.raw() == 7
    ));
}

#[test]
fn jianchu_current_sha_branch_requires_abi94_and_emits_typed_effect() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill jianchu(2342849364) {
  trigger card_targeted owner_source_sha_target_alive_has_discardable_hand_equip optional {
    effect choose_discard_target_card_resolve_current_sha(self, event_card, event_player);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_94
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_player: 8,
            event_card: 99,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        }] if source.raw() == 7 && card.raw() == 99 && target.raw() == 8
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_discard_target_card_resolve_current_sha(self, event_card, event_player); } }"
        )
        .is_err());
}

#[test]
fn selected_target_virtual_duel_requires_abi95_and_captures_all_targets() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill wn_dutan(4152033537) {
  active play_phase uses_per_turn 1 targets 1 255 other {
    effect use_virtual_card_on_selected_targets(self, juedou);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_95
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        selected_targets: vec![8, 9],
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind: 23,
            targets,
        }] if player.raw() == 7
            && targets.iter().map(|target| target.raw()).collect::<Vec<_>>() == [8, 9]
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 { effect use_virtual_card_on_selected_targets(self, juedou); } }"
        )
        .is_err());

    let tiesuo = compile_source(
            "skill jylianhuan(3727355012) { active play_phase uses_per_play_phase 1 targets 1 2 { effect lose_hp(self, 1); effect use_virtual_card_on_selected_targets(self, tiesuo); } }",
        )
        .unwrap();
    assert_eq!(
        tiesuo.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_343
    );
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 3 { effect use_virtual_card_on_selected_targets(self, tiesuo); } }"
        )
        .is_err());
}

#[test]
fn damage_linked_active_turn_ban_requires_abi96_and_basic_hand_selection() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill eu_xiaoguo(2461210777) {
  active play_phase targets 1 1 other select_hand 1 1 card_basic {
    effect discard_selected(self);
    effect deal_damage_ban_active_if_dying(self, event_source, 1, self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_96
    );
    let selection = pkg.active_index[0]
        .selection
        .as_ref()
        .expect("active selection");
    assert_eq!(
        (
            selection.zone.as_str(),
            selection.min,
            selection.max,
            selection.filter.as_deref(),
        ),
        ("hand", 1, 1, Some("card_basic"))
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 91,
            self_id: 7,
            event_source: 8,
            ..Default::default()
        },
        selected_cards: vec![41],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DiscardCards { player, cards },
            EffectRequest::DealDamageBanActiveIfDying {
                source,
                target,
                amount: 1,
                skill,
            }
        ] if player.raw() == 7
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [41]
            && source.raw() == 7
            && target.raw() == 8
            && skill.raw() == 91
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect deal_damage_ban_active_if_dying(self, event_source, 1, self_skill); } }"
        )
        .is_err());
}

#[test]
fn target_equipment_reclaim_range_comparison_requires_abi97_and_target_contract() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill anguo(3231604344) {
  active play_phase uses_per_turn 1 targets 1 1 other has_equip {
    effect choose_target_reclaim_equipment_draw_if_range_reduced(self, event_source, self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_97
    );
    let targets = pkg.active_index[0]
        .targets
        .as_ref()
        .expect("active targets");
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(!targets.allow_self);
    assert!(targets.has_equip);

    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 73,
            self_id: 7,
            event_source: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            skill,
        }] if player.raw() == 7 && target.raw() == 8 && skill.raw() == 73
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_target_reclaim_equipment_draw_if_range_reduced(self, event_source, self_skill); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 other { effect choose_target_reclaim_equipment_draw_if_range_reduced(self, event_source, self_skill); } }"
        )
        .is_err());
}

#[test]
fn phase_history_virtual_card_use_requires_abi98() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill jujun(3855454532) locked {
  trigger phase_any_end owner_alive_phase_targeted_hp_changed_can_use_wanjian {
    effect choose_use_virtual_card(self, wanjian, self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_98
    );
    assert_eq!(pkg.trigger_index[0].event, "phase_any_end");
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_alive_phase_targeted_hp_changed_can_use_wanjian"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 86,
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseUseVirtualCard {
            player,
            kind: 25,
            skill,
        }] if player.raw() == 7 && skill.raw() == 86
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_any_end owner_alive { effect choose_use_virtual_card(self, sha, self_skill); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_any_end owner_alive { effect choose_use_virtual_card(self, juedou, event_source); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_use_virtual_card(self, juedou, self_skill); } }"
        )
        .is_err());
}

#[test]
fn current_sha_base_damage_and_card_type_filters_require_abi99() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill wangong(3743761196) locked {
  trigger card_used owner_source_basic {
    effect add_skill(self, 3164390759);
  }
}
skill wangong2(3164390759) locked {
  trigger card_used owner_source_sha {
    effect increase_current_sha_base_damage(self, event_card, 1);
  }
  trigger card_used owner_source_non_basic {
    effect remove_skill(self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_99
    );
    assert_eq!(pkg.trigger_index[0].filter, "owner_source_basic");
    assert_eq!(pkg.trigger_index[2].filter, "owner_source_non_basic");
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 86,
            self_id: 7,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    let sha_module = &pkg
        .trigger_index
        .iter()
        .find(|trigger| trigger.filter == "owner_source_sha")
        .unwrap()
        .module;
    let sha_module = pkg
        .modules
        .iter()
        .find(|module| &module.name == sha_module)
        .unwrap();
    RuleVm::new()
        .execute(&vmod(sha_module), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount: 1,
            skill,
        }] if player.raw() == 7 && card.raw() == 77 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger card_used owner_source_sha { effect increase_current_sha_base_damage(self, event_card); } }"
        )
        .is_err());
}

#[test]
fn skip_next_play_phase_requires_abi100() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
            "skill oldzishou(1) { trigger phase_draw owner_is_target_wounded optional { effect skip_next_play_phase(self); } }",
        )
        .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_100
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::SkipNextPlayPhase { player }] if player.raw() == 7
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_draw owner_is_target { effect skip_next_play_phase(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn choose_other_draw_or_recover_requires_abi101() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
            "skill shushen(1) { trigger hp_recovered owner_is_target optional { effect choose_other_draw_or_recover(self, 2, 1, event_amount); } }",
        )
        .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_101
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_amount: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count: 2,
            recover_count: 1,
            times: 2,
            skill,
        }] if player.raw() == 7 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger hp_recovered owner_is_target { effect choose_other_draw_or_recover(self, 2, 1); } }"
        )
        .is_err());
}

#[test]
fn field_equipment_count_and_one_shot_discard_require_abi102() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
            "skill kuiwei2(1) { trigger phase_draw owner_is_target { effect choose_discard_hand_equip_then_remove_self_skill(self, field_equipment_kind_count(weapon)); } }",
        )
        .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_102
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        query_stub_int: 3,
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count: 3,
            skill,
        }] if player.raw() == 7 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_draw owner_is_target { effect choose_discard_hand_equip_then_remove_self_skill(self); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_draw owner_is_target { effect draw_cards(self, field_equipment_kind_count(not_a_card)); } }"
        )
        .is_err());
}

#[test]
fn selected_target_draw_and_min_hp_recovery_require_abi103() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill xiongyi(2476017492) {
  active play_phase once_per_game targets 1 255 {
    effect draw_selected_targets(3);
    effect recover_if_min_current_hp(self, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_103
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        selected_targets: vec![7, 9],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DrawSelectedTargets {
                targets,
                count: 3,
                skill,
            },
            EffectRequest::RecoverIfMinCurrentHp { player, amount: 1 }
        ] if targets.iter().map(|target| target.raw()).collect::<Vec<_>>() == [7, 9]
            && skill.raw() == 0
            && player.raw() == 7
    ));
    assert!(compile_source(
        "skill bad(1) { active play_phase once_per_game { effect draw_selected_targets(3); } }"
    )
    .is_err());
    assert!(compile_source(
        "skill bad(1) { trigger phase_play owner_is_target { effect draw_selected_targets(3); } }"
    )
    .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 { effect recover_if_min_current_hp(self); } }"
        )
        .is_err());
}

#[test]
fn current_damaging_card_base_damage_requires_abi104() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill hanyong(3661917608) {
  trigger card_used owner_source_wounded_nanman_wanjian_or_normal_spade_sha optional {
    effect increase_current_card_base_damage(self, event_card, 1);
    if round_number_capped(2147483647) < hp(self) + 1 {
      effect add_mark(self, 638947751, 1);
    }
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_104
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_card: 77,
            ..Default::default()
        },
        query_stub_int: 3,
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::IncreaseCurrentCardBaseDamage {
                player,
                card,
                amount: 1,
                skill,
            },
            EffectRequest::AddMark {
                player: mark_player,
                mark,
                delta: 1,
            }
        ] if player.raw() == 7
            && card.raw() == 77
            && skill.raw() == 0
            && mark_player.raw() == 7
            && mark.raw() == 638947751
    ));
    assert!(compile_source(
            "skill bad(1) { trigger card_used owner_source_sha { effect increase_current_card_base_damage(self, event_card); } }"
        )
        .is_err());
}

#[test]
fn repeated_play_category_and_turn_damage_filters_require_abi105() {
    let package = compile_source(
            r#"
skill langmie(3392096402) {
  trigger phase_play_end owner_alive_other_play_phase_duplicate_card_category_has_hand_equip optional {
    effect draw_cards(self, 1);
  }
  trigger phase_end owner_alive_other_phase_end_damage_gt_one_has_hand_equip optional {
    effect discard_one_hand_or_equip(self);
    effect deal_damage(self, event_player, 1);
  }
}
"#,
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_105
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().all(|trigger| trigger.optional));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip"
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_alive_other_phase_end_damage_gt_one_has_hand_equip"
    }));
}

#[test]
fn current_play_phase_hp_loss_filter_requires_abi106() {
    let package = compile_source(
        r#"
skill zhaxiang(3298876867) locked {
  trigger hp_lost owner_is_target {
    effect draw_cards(self, event_amount * 3);
  }
  trigger hp_lost owner_alive_is_target_current_play_phase {
    effect add_skill(self, 4056078041);
    effect add_mark(self, 4056078041, event_amount);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_106
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(
        package.trigger_index[1].filter,
        "owner_alive_is_target_current_play_phase"
    );
}

#[test]
fn compile_color_discard_temp_skill_effects() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill nsduijue(4088249173) {
  trigger phase_play owner_is_target {
    effect remove_skill_by_def(self, 3607483734);
    effect choose_discard_hand_add_skill_by_color(self, 3607483734, 3115572483);
  }
}
"#,
    )
    .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::RemoveSkillByDef { player, skill_def },
            EffectRequest::ChooseDiscardHandAddSkillByColor {
                player: chooser,
                red_skill_def,
                black_skill_def,
            }
        ] if player.raw() == 7
            && chooser.raw() == 7
            && skill_def.raw() == 3_607_483_734
            && red_skill_def.raw() == 3_607_483_734
            && black_skill_def.raw() == 3_115_572_483
    ));
}

#[test]
fn compile_reveal_top_gain_or_swap_effect() {
    use rule_api::EffectRequest;
    let pkg = compile_source(
        r#"
skill junkjizhi(2783324283) {
  trigger card_used owner_source_physical_ordinary_trick optional {
    effect reveal_top_gain_non_basic_or_swap_basic(self);
  }
}
"#,
    )
    .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert_eq!(
        host.effects,
        vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic {
            player: rule_api::PlayerId::new(7),
        }]
    );
}

#[test]
fn compile_damage_before_prevention() {
    use rule_api::EffectRequest;
    let source = r#"
skill nsdunxing(4203825879) {
  trigger damage_before owner_is_target_wounded optional {
    effect prevent_current_damage();
    effect change_max_hp(self, 0 - event_amount);
    effect draw_cards(self, event_amount);
  }
}
"#;
    let package = compile_source(source).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_amount: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::PreventDamage { damage_id: 0 },
            EffectRequest::ChangeMaxHp { player, delta: -2 },
            EffectRequest::DrawCards { player: drawer, count: 2 },
        ] if player.raw() == 7 && drawer.raw() == 7
    ));
    assert!(compile_source(&source.replace("damage_before", "damage")).is_err());
}

#[test]
fn compile_and_run_reveal_event_card() {
    use rule_api::EffectRequest;
    // On using a card, reveal that card publicly (uses the event_card operand).
    let src = r#"
skill zhanshi(11) {
  trigger card_used owner_is_source {
    effect reveal_card(event_card);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let module = &pkg.modules[0];
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 1,
            event_player: 1,
            event_source: 1,
            event_card: 77,
            event_amount: 0,
        },
        ..Default::default()
    };
    let mut vm = RuleVm::new();
    let r = vm.execute(&vmod(module), &mut host, 10_000).unwrap();
    assert!(matches!(r, ExecResult::Halted { .. }));
    assert_eq!(host.effects.len(), 1);
    match &host.effects[0] {
        EffectRequest::RevealCards { cards, .. } => {
            assert_eq!(cards.len(), 1);
            assert_eq!(cards[0].raw(), 77, "should reveal the event card");
        }
        other => panic!("expected RevealCards, got {other:?}"),
    }
}

#[test]
fn compile_card_param_queries() {
    // card_usable / target_legal / card_attribute with a card operand compile & run.
    let src = r#"
skill probe(13) {
  trigger card_used owner_is_source {
    if card_usable(self, event_card) == 1 {
      effect draw_cards(self, 1);
    }
    if target_legal(self, event_card, event_player) == 1 {
      effect draw_cards(self, 1);
    }
    let v = card_attribute(event_card, 0);
    if v == 1 {
      effect emit_note(9);
    }
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let module = &pkg.modules[0];
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 1,
            event_player: 2,
            event_source: 1,
            event_card: 55,
            event_amount: 0,
        },
        query_stub_int: 1, // all queries answer 1
        ..Default::default()
    };
    let mut vm = RuleVm::new();
    let r = vm.execute(&vmod(module), &mut host, 10_000).unwrap();
    assert!(matches!(r, ExecResult::Halted { .. }));
    // Both draw branches (stub=1) plus the emit_note branch fire.
    assert_eq!(host.effects.len(), 3);
}

#[test]
fn card_query_rejects_player_where_card_expected() {
    // card_usable's 2nd arg must be a card; passing a player must fail.
    let src = r#"
skill bad(14) {
  trigger card_used owner_is_source {
    if card_usable(self, self) == 1 {
      effect draw_cards(self, 1);
    }
  }
}
"#;
    assert!(compile_source(src).is_err());
}

#[test]
fn compile_and_run_use_card() {
    use rule_api::EffectRequest;
    let src = r#"
skill autoplay(15) {
  trigger phase_begin owner_is_target {
    effect use_card(self, event_card);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 0,
            self_id: 4,
            event_player: 4,
            event_source: 4,
            event_card: 88,
            event_amount: 0,
        },
        ..Default::default()
    };
    let mut vm = RuleVm::new();
    let r = vm
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(r, ExecResult::Halted { .. }));
    assert_eq!(host.effects.len(), 1);
    match &host.effects[0] {
        EffectRequest::UseCard { player, card, .. } => {
            assert_eq!(player.raw(), 4);
            assert_eq!(card.raw(), 88);
        }
        other => panic!("expected UseCard, got {other:?}"),
    }
}

#[test]
fn compile_and_run_remove_cards_by_kind() {
    use rule_api::EffectRequest;
    let src = r#"
skill remove_mounts(16) locked {
  trigger game_started owner_alive {
    effect remove_cards_by_kind(horse_off);
    effect remove_cards_by_kind(horse_def);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let mut host = CollectingHost::default();
    let result = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(result, ExecResult::Halted { .. }));
    assert_eq!(
        host.effects,
        vec![
            EffectRequest::RemoveCardsByKind { kind: 12 },
            EffectRequest::RemoveCardsByKind { kind: 13 },
        ]
    );
    assert!(compile_source(&src.replace("horse_def", "unknown_mount")).is_err());
}

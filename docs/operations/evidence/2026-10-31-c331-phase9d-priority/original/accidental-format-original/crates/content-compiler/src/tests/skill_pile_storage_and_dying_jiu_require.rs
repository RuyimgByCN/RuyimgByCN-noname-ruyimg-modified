use super::*;

#[test]
fn skill_pile_storage_and_dying_jiu_require_abi46() {
    let package = compile_source(
        r#"
skill chunlao(2555108773) {
  trigger phase_end owner_is_target_hand_sha_skill_pile_empty optional {
    effect choose_store_hand_sha_in_skill_pile(self);
  }
  trigger dying owner_alive_dying_has_skill_pile_cards optional {
    effect choose_spend_skill_pile_for_dying_jiu(self, event_player);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_46
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_hand_sha_skill_pile_empty"
    );
    assert_eq!(
        package.trigger_index[1].filter,
        "owner_alive_dying_has_skill_pile_cards"
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    RuleVm::new()
        .execute(&vmod(&package.modules[1]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::ChooseStoreHandShaInSkillPile { player, .. },
            EffectRequest::ChooseSpendSkillPileForDyingJiu {
                player: second_player,
                dying,
                ..
            }
        ] if player.raw() == 9 && second_player.raw() == 9 && dying.raw() == 7
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect choose_store_hand_sha_in_skill_pile(self, event_player); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger dying always { effect choose_spend_skill_pile_for_dying_jiu(self); } }"
        )
        .is_err());
}

#[test]
fn selected_hand_comparison_and_bound_target_rules_require_abi47() {
    let package = compile_source(
        r#"
skill compare(1) {
  active play_phase uses_per_turn 1 targets 1 1 other can_compare select_hand 1 1 {
    effect choose_compare_selected_hand_card(self, event_player, 2, 3);
  }
}
skill win(2) {
  modifier card_distance_unlimited_for_skill_bound_target add 1;
  modifier sha_use_unlimited_for_skill_bound_target add 1;
  modifier ignore_armor_for_skill_bound_target add 1;
  trigger card_used owner_source_sha {
    effect choose_add_skill_bound_card_target(self, event_card);
  }
}
skill lose(3) {}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_47
    );
    assert!(
        package.active_index[0]
            .targets
            .as_ref()
            .unwrap()
            .can_compare
    );
    assert_eq!(package.modifier_index.len(), 3);

    let active_module_name = &package.active_index[0].module;
    let active_module = package
        .modules
        .iter()
        .find(|module| &module.name == active_module_name)
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            ..Default::default()
        },
        selected_cards: vec![41],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(active_module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            ..
        }] if player.raw() == 9
            && target.raw() == 7
            && card.raw() == 41
            && win_skill_def.raw() == 2
            && lose_skill_def.raw() == 3
    ));

    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 can_compare { effect choose_compare_selected_hand_card(self, event_player, 2); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger card_used owner_source_sha { effect choose_add_skill_bound_card_target(self, 1); } }"
        )
        .is_err());
}

#[test]
fn matching_equip_color_lock_and_no_sha_retaliation_require_abi48() {
    let package = compile_source(
        r#"
skill sidi(1) {
  trigger phase_play owner_alive_not_target {
    effect choose_discard_matching_equip_color_add_skill(self, event_player, 3172356775);
  }
}
skill lock(3172356775) {
  modifier card_action_cancel_for_skill_color_mask override 1;
  trigger phase_play_end owner_is_target {
    effect use_virtual_sha_from_card_color_lock_source_if_no_sha(self);
    effect remove_skill(self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_48
    );
    assert_eq!(
        package.modifier_index[0].target,
        "card_action_cancel_for_skill_color_mask"
    );

    let begin_trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.event == "phase_play")
        .unwrap();
    let begin_module = package
        .modules
        .iter()
        .find(|module| module.name == begin_trigger.module)
        .unwrap();
    let mut begin_host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(begin_module), &mut begin_host, 100)
        .unwrap();
    assert!(matches!(
        begin_host.effects.as_slice(),
        [EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            ..
        }] if player.raw() == 9 && target.raw() == 7 && skill_def.raw() == 3172356775
    ));

    assert!(compile_source(
            "skill bad(1) { trigger phase_play always { effect choose_discard_matching_equip_color_add_skill(self, event_player); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_play_end owner_is_target { effect use_virtual_sha_from_card_color_lock_source_if_no_sha(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn exact_card_use_enhancements_require_abi49() {
    let package = compile_source(
        r#"
skill benxi(1) locked {
  modifier distance_from add -owner_current_turn_card_use_count;
  trigger card_used owner_source_single_target_sha_or_ordinary_trick_current_turn {
    effect choose_current_card_use_enhancements(self, event_card, 2);
  }
}
skill helper(2) {
  modifier ignore_armor_for_skill_card_policy override 1;
  modifier card_use_unrespondable_for_skill_card_policy override 1;
  trigger damage owner_source_event_card_policy_draw {
    effect draw_cards(self, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_49
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.value_source.as_deref() == Some("owner_current_turn_card_use_count")
    }));

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| {
            trigger.filter == "owner_source_single_target_sha_or_ordinary_trick_current_turn"
        })
        .unwrap();
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            ..
        }] if player.raw() == 9 && card.raw() == 77 && helper_skill_def.raw() == 2
    ));

    assert!(compile_source(
            "skill bad(1) { trigger card_used always { effect choose_current_card_use_enhancements(self, 1, 2); } }"
        )
        .is_err());
    assert!(
        compile_source("skill bad(1) { modifier ignore_armor_for_skill_card_policy add 1; }")
            .is_err()
    );
}

#[test]
fn draw_or_discard_add_skill_requires_abi50() {
    let package = compile_source(
            "skill jiangchi(1) { trigger phase_draw_end owner_is_target optional { effect choose_draw_or_discard_add_skill(self, 2, 3); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_50
    );
    assert_eq!(package.trigger_index[0].event, "phase_draw_end");
    assert!(package.trigger_index[0].optional);

    let module = package
        .modules
        .iter()
        .find(|module| module.name == package.trigger_index[0].module)
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            three_way,
            ..
        }] if player.raw() == 9
            && draw_skill_def.raw() == 2
            && discard_skill_def.raw() == 3
            && !three_way
    ));

    let event_only = compile_source(
        "skill boundary(1) { trigger phase_draw_end always { effect emit_note(1); } }",
    )
    .unwrap();
    assert_eq!(
        event_only.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_50
    );
    assert!(compile_source(
            "skill bad(1) { trigger phase_draw_end always { effect choose_draw_or_discard_add_skill(self, 2); } }"
        )
        .is_err());
}

#[test]
fn discard_batch_bingliang_and_judge_sha_require_abi51() {
    let package = compile_source(
            "skill shenduan(1) { trigger cards_moved owner_discarded_black_basic_to_discard { effect choose_discarded_black_basic_as_bingliang(self); } }\n\
             skill yonglve(2) { trigger phase_judge owner_alive_other_judge_phase_in_range_has_judge_card { effect choose_discard_judge_use_virtual_sha_draw_if_no_damage(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_51
    );
    let mut effects = Vec::new();
    for trigger in &package.trigger_index {
        let module = package
            .modules
            .iter()
            .find(|module| module.name == trigger.module)
            .unwrap();
        let mut host = CollectingHost {
            ctx: HostContext {
                self_id: 9,
                event_player: 10,
                ..Default::default()
            },
            ..Default::default()
        };
        RuleVm::new()
            .execute(&vmod(module), &mut host, 100)
            .unwrap();
        effects.extend(host.effects);
    }
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ChooseDiscardedBlackBasicAsBingliang { player, cards, .. }
            if player.raw() == 9 && cards.is_empty()
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
            player,
            target,
            ..
        } if player.raw() == 9 && target.raw() == 10
    )));

    let filter_only = compile_source(
            "skill boundary(1) { trigger cards_moved owner_discarded_black_basic_to_discard { effect emit_note(1); } }",
        )
        .unwrap();
    assert_eq!(
        filter_only.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_51
    );
    assert!(compile_source(
            "skill bad(1) { trigger cards_moved always { effect choose_discarded_black_basic_as_bingliang(self, event_card); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_judge always { effect choose_discard_judge_use_virtual_sha_draw_if_no_damage(self); } }"
        )
        .is_err());
}

#[test]
fn active_unique_category_target_and_count_branch_require_abi52() {
    let package = compile_source(
            "skill pindi(1) { active play_phase targets 1 1 other unused_by_skill_this_turn select_he 1 1 unused_category_by_skill_this_turn { effect discard_selected_record_unique_category_target(self, event_source); effect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(self, event_source); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_52
    );
    let active = package.active_index.first().unwrap();
    assert!(active.targets.as_ref().unwrap().unused_by_skill_this_turn);
    assert!(
        active
            .selection
            .as_ref()
            .unwrap()
            .unused_category_by_skill_this_turn
    );
    let module = package
        .modules
        .iter()
        .find(|module| module.name == active.module)
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 10,
            ..Default::default()
        },
        selected_cards: vec![77],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.first(),
        Some(EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            ..
        }) if player.raw() == 9 && target.raw() == 10 && card.raw() == 77
    ));
    assert!(matches!(
        host.effects.get(1),
        Some(EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            player,
            target,
            ..
        }) if player.raw() == 9 && target.raw() == 10
    ));

    let flags_only = compile_source(
            "skill boundary(1) { active play_phase targets 1 1 other unused_by_skill_this_turn select_he 1 1 unused_category_by_skill_this_turn { effect discard_selected(self); } }",
        )
        .unwrap();
    assert_eq!(
        flags_only.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_52
    );
    assert!(compile_source(
            "skill bad(1) { active play_phase { effect discard_selected_record_unique_category_target(self); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase { effect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(self); } }"
        )
        .is_err());
}

#[test]
fn remembered_category_and_phase_gift_effects_require_abi53() {
    let package = compile_source(
            "skill qiangzhi(1) { trigger phase_play owner_is_target_other_hand_nonempty optional { effect choose_reveal_other_hand_card_remember_category(self); } trigger card_used owner_source_play_card_matches_remembered_category optional { effect draw_cards(self, 1); } } skill xiantu(2) { trigger phase_play owner_alive_not_target optional { effect draw_then_give_exact_record_phase_target(self, event_player, 2); } trigger death owner_phase_gift_target_is_source { effect clear_phase_gift_target(self); } trigger phase_play_end owner_phase_gift_target_is_target { effect clear_phase_gift_target(self); effect lose_hp(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_53
    );
    let mut effects = Vec::new();
    for module in &package.modules {
        let mut host = CollectingHost {
            ctx: HostContext {
                self_id: 9,
                event_player: 10,
                event_source: 11,
                ..Default::default()
            },
            ..Default::default()
        };
        RuleVm::new()
            .execute(&vmod(module), &mut host, 100)
            .unwrap();
        effects.extend(host.effects);
    }
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, .. }
            if player.raw() == 9
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count: 2,
            ready_to_give: false,
            ..
        } if player.raw() == 9 && target.raw() == 10
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ClearPhaseGiftTarget { player, .. } if player.raw() == 9
    )));

    let filter_only = compile_source(
            "skill boundary(1) { trigger card_used owner_source_play_card_matches_remembered_category { effect emit_note(1); } }",
        )
        .unwrap();
    assert_eq!(
        filter_only.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_53
    );
    assert!(compile_source(
            "skill bad(1) { trigger phase_play always { effect choose_reveal_other_hand_card_remember_category(self, event_player); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_play always { effect draw_then_give_exact_record_phase_target(self, event_player); } }"
        )
        .is_err());
    assert!(compile_source(
        "skill bad(1) { trigger phase_play_end always { effect clear_phase_gift_target(); } }"
    )
    .is_err());
}

#[test]
fn resolved_sha_gift_effect_requires_abi54() {
    let package = compile_source(
            "skill xinzhongyong(1) { trigger sha_resolved owner_source_sha_resolved_has_gainable_cards_and_recipient optional { effect choose_gain_resolved_sha_or_shan_then_optional_use_sha(self, event_card); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_54
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_card: 17,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
            player,
            sha_card,
            ..
        }] if player.raw() == 9 && sha_card.raw() == 17
    ));
    assert!(compile_source(
            "skill bad(1) { trigger sha_resolved always { effect choose_gain_resolved_sha_or_shan_then_optional_use_sha(self); } }"
        )
        .is_err());
}

#[test]
fn skipped_phase_effects_require_abi55() {
    let package = compile_source(
            "skill fenli(1) { trigger phase_draw owner_is_target_tied_max_hand optional { effect skip_current_phase(self); } } skill pingkou(2) { trigger phase_end owner_is_target_skipped_phase_count_positive optional { effect choose_other_targets_by_skipped_phase_count_damage(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_55
    );
    let mut effects = Vec::new();
    for module in &package.modules {
        let mut host = CollectingHost {
            ctx: HostContext {
                self_id: 9,
                ..Default::default()
            },
            ..Default::default()
        };
        RuleVm::new()
            .execute(&vmod(module), &mut host, 100)
            .unwrap();
        effects.extend(host.effects);
    }
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::SkipCurrentPhase { player } if player.raw() == 9
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount: 1,
            ..
        } if player.raw() == 9
    )));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end always { effect choose_other_targets_by_skipped_phase_count_damage(self); } }"
        )
        .is_err());
}

#[test]
fn uniform_hand_draw_effect_requires_abi56() {
    let package = compile_source(
            "skill olbingyi(1) { trigger cards_moved owner_discarded_has_hand_once_per_phase optional { effect reveal_uniform_hand_draw_self_choose_others(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_56
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger cards_moved always { effect reveal_uniform_hand_draw_self_choose_others(self, 1); } }"
        )
        .is_err());
}

#[test]
fn card_user_transfer_and_equipment_damage_reduction_require_abi57() {
    let chanhui = compile_source(
            "skill chanhui(1) { trigger card_used owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused optional { effect choose_card_user_or_add_target(self, event_card); } }",
        )
        .unwrap();
    assert_eq!(
        chanhui.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_57
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_card: 17,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&chanhui.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseCardUserOrAddTarget { player, card, .. }]
            if player.raw() == 9 && card.raw() == 17
    ));

    let jiaojin = compile_source(
            "skill jiaojin(2) { trigger damage_before owner_is_target_male_source_has_discardable_equipment optional { effect choose_discard_equipment_reduce_current_damage(self); } }",
        )
        .unwrap();
    assert_eq!(
        jiaojin.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_57
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&jiaojin.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_discard_equipment_reduce_current_damage(self); } }"
        )
        .is_err());
}

#[test]
fn turn_history_equipment_choice_and_limited_equipment_gift_require_abi58() {
    let qieting = compile_source(
            "skill qieting(1) { trigger turn_ended owner_alive_other_turn_ended_no_card_targeted_other { effect choose_move_target_equipment_or_draw(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        qieting.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_58
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 12,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&qieting.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, target, .. }]
            if player.raw() == 9 && target.raw() == 12
    ));

    let xianzhou = compile_source(
            "skill xianzhou(2) { active play_phase equip_nonempty once_per_game targets 1 1 other { effect give_all_equipment_choose_recover_or_damage(self, event_source); } }",
        )
        .unwrap();
    assert_eq!(
        xianzhou.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_58
    );
    let active = &xianzhou.active_index[0];
    assert!(active.equip_nonempty);
    assert!(active.once_per_game);
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 12,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&xianzhou.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, target, .. }]
            if player.raw() == 9 && target.raw() == 12
    ));

    assert!(compile_source(
            "skill bad(1) { trigger turn_ended always { effect choose_move_target_equipment_or_draw(self); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase { effect give_all_equipment_choose_recover_or_damage(self); } }"
        )
        .is_err());
}

#[test]
fn weapon_range_damage_choice_requires_abi59() {
    let compiled = compile_source(
            "skill qingxi(1) { trigger damage_before owner_source_sha_has_weapon optional { effect choose_target_discard_weapon_range_hand_or_increase_damage(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_59
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 12,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            ..
        }] if player.raw() == 9 && target.raw() == 12
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage_before always { effect choose_target_discard_weapon_range_hand_or_increase_damage(self); } }"
        )
        .is_err());
}

#[test]
fn hand_count_target_draw_or_random_discard_requires_abi60() {
    let compiled = compile_source(
            "skill yaoming(1) { trigger damage owner_source_or_target_hand_count_differs_once_per_turn optional { effect choose_hand_count_different_target_draw_or_random_discard(self); } }",
        )
        .unwrap();
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_60
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage always { effect choose_hand_count_different_target_draw_or_random_discard(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn current_draw_count_increase_requires_abi61_and_draw_phase() {
    let compiled = compile_source(
            "skill zishou(1) { trigger phase_draw owner_is_target optional { effect increase_current_draw_count(self, alive_group_count()); } }",
        )
        .unwrap();
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_61
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        query_stub_int: 3,
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::IncreaseCurrentDrawCount { player, count }]
            if player.raw() == 9 && *count == 3
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_play owner_is_target { effect increase_current_draw_count(self, 1); } }"
        )
        .is_err());
}

#[test]
fn prepare_target_discard_branch_requires_abi62_and_prepare_phase() {
    let src = "skill rezhenjun(4131625144) { trigger phase_prepare owner_is_target_any_alive_hand_nonempty optional { effect choose_target_discard_excess_then_self_discard_or_target_draw(self); } }";
    let compiled = compile_source(src).expect("compile rezhenjun");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_62
    );

    let wrong_phase = "skill bad(1) { trigger phase_draw owner_is_target_any_alive_hand_nonempty optional { effect choose_target_discard_excess_then_self_discard_or_target_draw(self); } }";
    let err = compile_source(wrong_phase).expect_err("effect is prepare-only");
    assert!(err.to_string().contains("only available in phase_prepare"));
}

#[test]
fn damage_target_neighbor_gain_requires_abi63_and_damage_event() {
    let src = "skill eu_tuxi(3876759954) { trigger damage owner_source optional { effect draw_then_choose_gain_one_from_damage_target_and_neighbors(self, event_player); } }";
    let compiled = compile_source(src).expect("compile eu_tuxi");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_63
    );

    let wrong_event = "skill bad(1) { trigger phase_end owner_source optional { effect draw_then_choose_gain_one_from_damage_target_and_neighbors(self, event_player); } }";
    let err = compile_source(wrong_event).expect_err("effect is damage-only");
    assert!(err
        .to_string()
        .contains("only available in damage triggers"));
}

#[test]
fn bound_damage_branch_requires_abi64_and_damage_before() {
    let src = "skill masato_baoquan(1) locked { trigger damage_before owner_source { effect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2); } }";
    let compiled = compile_source(src).expect("compile masato_baoquan");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_64
    );
    assert!(compiled.manifest.capabilities.request_choice);
    assert!(compiled.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE),
                ]
        }));

    let wrong_event = "skill bad(1) { trigger damage owner_source { effect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2); } }";
    let err = compile_source(wrong_event).expect_err("effect is damage-before-only");
    assert!(err.to_string().contains("only available in damage_before"));
}

#[test]
fn hand_compare_hooks_require_abi65_and_preserve_event_cards() {
    let src = r#"
skill hanzhan(4071802825) {
  trigger hand_compare_started owner_hand_compare_participant optional {
    effect randomize_current_hand_compare_opponent(self);
  }
  trigger hand_compare_finished owner_hand_compare_participant_has_sha optional {
    effect gain_highest_rank_sha_from_event_cards(self);
  }
}
"#;
    let compiled = compile_source(src).expect("compile hanzhan");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_65
    );
    assert_eq!(compiled.trigger_index.len(), 2);

    let gain_trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.event == "hand_compare_finished")
        .unwrap();
    let gain_module = compiled
        .modules
        .iter()
        .find(|module| module.name == gain_trigger.module)
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 91,
            self_id: 7,
            ..Default::default()
        },
        event_cards: vec![11, 12],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(gain_module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            ..
        }] if player.raw() == 7 && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == vec![11, 12]
    ));

    let wrong_event = "skill bad(1) { trigger damage owner_source optional { effect gain_highest_rank_sha_from_event_cards(self); } }";
    let err = compile_source(wrong_event).expect_err("comparison gain is finish-only");
    assert!(err
        .to_string()
        .contains("only available in hand_compare_finished"));
}

#[test]
fn hand_compare_result_card_gain_requires_abi66_and_preserves_context() {
    let src = r#"
skill jyzongshi(2485328360) {
  trigger hand_compare_finished owner_hand_compare_participant_result_card_in_discard optional {
    effect gain_hand_compare_result_card(self);
  }
}
"#;
    let compiled = compile_source(src).expect("compile jyzongshi");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_66
    );
    let module = &compiled.modules[0];
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_player: 7,
            event_source: 8,
            // Hand-comparison winners are encoded as raw id + 1 so a tie survives the
            // shared event scalar/target-count host slot.
            event_amount: 8,
            ..Default::default()
        },
        event_cards: vec![11, 12],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        }] if player.raw() == 7
            && initiator.raw() == 7
            && target.raw() == 8
            && winner.raw() == 7
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == vec![11, 12]
            && skill.raw() == 0
    ));

    let wrong_event = "skill bad(1) { trigger damage owner_source optional { effect gain_hand_compare_result_card(self); } }";
    let err = compile_source(wrong_event).expect_err("comparison gain is finish-only");
    assert!(err
        .to_string()
        .contains("only available in hand_compare_finished"));
}

#[test]
fn current_card_target_adjustment_requires_abi67_and_preserves_card() {
    let src = r#"
skill qiaoshui3(2260792673) {
  trigger card_used owner_source_basic_or_trick {
    effect choose_add_or_remove_current_card_target(self, event_card);
  }
}
"#;
    let compiled = compile_source(src).expect("compile qiaoshui3");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_67
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_card: 11,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player,
            card,
            skill,
        }] if player.raw() == 7 && card.raw() == 11 && skill.raw() == 0
    ));

    let wrong_event = "skill bad(1) { trigger damage owner_source { effect choose_add_or_remove_current_card_target(self, event_card); } }";
    let err = compile_source(wrong_event).expect_err("target adjustment is card-used-only");
    assert!(err.to_string().contains("only available in card_used"));
}

#[test]
fn triggered_hand_compare_requires_abi68() {
    let src = r#"
skill qiaoshui(3955213386) {
  trigger phase_play owner_is_target_can_compare_other optional {
    effect choose_hand_compare(self, 2260792673, 2280240537);
  }
}
"#;
    let compiled = compile_source(src).expect("compile qiaoshui");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_68
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill,
        }] if player.raw() == 7
            && win_skill_def.raw() == 2260792673
            && lose_skill_def.raw() == 2280240537
            && skill.raw() == 0
    ));
}

#[test]
fn choose_other_damage_requires_abi69() {
    let src = r#"
skill stdxingfa(2490051933) {
  trigger phase_prepare owner_is_target_hand_at_least_hp_other_alive optional {
    effect choose_other_damage(self, 1);
  }
}
"#;
    let compiled = compile_source(src).expect("compile stdxingfa");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_69
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseOtherDamage {
            player,
            amount: 1,
            skill,
        }] if player.raw() == 7 && skill.raw() == 0
    ));
}

#[test]
fn choose_other_discard_target_card_requires_abi70() {
    let src = r#"
skill sijian(3832046646) {
  trigger cards_moved owner_from_hand_empty_other_discardable_hand_equip optional {
    effect choose_other_discard_target_card(self, hand_equip);
  }
}
"#;
    let compiled = compile_source(src).expect("compile sijian");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_70
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&compiled.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseOtherDiscardTargetCard {
            player,
            scope: rule_api::TargetCardScope::HandEquip,
            skill,
        }] if player.raw() == 7 && skill.raw() == 0
    ));
}

#[test]
fn play_phase_card_count_filter_requires_abi71_and_positive_count() {
    let src = r#"
skill scszimou(2246121175) locked {
  trigger card_used owner_source_play_card_count_eq_2 {
    effect gain_card_by_kind(self, jiu);
  }
}
"#;
    let compiled = compile_source(src).expect("compile scszimou");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_71
    );

    let invalid = src.replace("count_eq_2", "count_eq_0");
    assert!(compile_source(&invalid).is_err());
}

#[test]
fn judgment_best_from_draw_pile_requires_abi72() {
    let compiled = compile_source(
        r#"
skill hisako_zhuanyun(3818579103) locked {
  modifier judgment_best_from_draw_pile override 1;
}
"#,
    )
    .expect("compile hisako_zhuanyun");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_72
    );
}

#[test]
fn gain_discard_card_by_kind_requires_abi74() {
    let package = compile_source(
        r#"
skill redangxian(4137190076) locked {
  trigger phase_begin owner_is_target {
    effect gain_discard_card_by_kind(self, sha);
    effect queue_opening_extra_play_phase(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_74
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::GAIN_DISCARD_CARD_BY_KIND),
            ]
    }));
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
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
            EffectRequest::GainDiscardCardByKind { player, kind },
            EffectRequest::QueueOpeningExtraPlayPhase { player: queued, .. }
        ] if player.raw() == 9 && *kind == 1 && queued.raw() == 9
    ));
}

#[test]
fn round_number_capped_requires_abi75() {
    let package = compile_source(
        r#"
skill godan_yuanyi(3610550580) locked {
  trigger phase_begin owner_is_target {
    effect draw_cards(self, round_number_capped(3));
    effect queue_opening_extra_play_phase(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_75
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::ROUND_NUMBER_CAPPED,
            ]
    }));
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        query_stub_int: 3,
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DrawCards { player, count },
            EffectRequest::QueueOpeningExtraPlayPhase { player: queued, .. }
        ] if player.raw() == 9 && *count == 3 && queued.raw() == 9
    ));
}

#[test]
fn reveal_uniform_hand_choose_damage_requires_abi76() {
    let package = compile_source(
        r#"
skill stdzhaoxin(2955343028) locked {
  trigger phase_prepare owner_is_target_hand_nonempty {
    effect reveal_uniform_hand_choose_damage(self, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_76
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RevealUniformHandChooseDamage {
            player,
            amount: 1,
            skill,
        }] if player.raw() == 9 && skill.raw() == 0
    ));
}

#[test]
fn choose_gain_discard_card_by_kind_requires_abi77() {
    let package = compile_source(
        r#"
skill tydangxian(3099116192) locked {
  trigger phase_begin owner_is_target {
    effect choose_gain_discard_card_by_kind(self, sha);
    effect queue_opening_extra_play_phase(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_77
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
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
            EffectRequest::ChooseGainDiscardCardByKind {
                player,
                kind: 1,
                skill,
            },
            EffectRequest::QueueOpeningExtraPlayPhase { player: queued, .. }
        ] if player.raw() == 9 && skill.raw() == 0 && queued.raw() == 9
    ));
}

#[test]
fn draw_for_lifetime_card_action_divisors_requires_abi78() {
    let package = compile_source(
        r#"
skill scls_qinzheng(1) locked {
  trigger card_used owner_lifetime_card_action {
    effect draw_for_lifetime_card_action_divisors(self, 3, 5, 8);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_78
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::DrawForLifetimeCardActionDivisors {
            player,
            divisors: [3, 5, 8],
        }] if player.raw() == 9
    ));
}

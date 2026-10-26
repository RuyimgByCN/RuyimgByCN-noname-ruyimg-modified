use super::*;

#[test]
fn choose_add_missing_skill_requires_abi79() {
    let package = compile_source(
        r#"
skill scls_zhanshen(1) locked {
  trigger phase_prepare owner_is_target {
    effect choose_add_missing_skill(self, 2, 3, 4);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_79
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
        [EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        }] if player.raw() == 9
            && [skill_defs[0].raw(), skill_defs[1].raw(), skill_defs[2].raw()] == [2, 3, 4]
            && skill.raw() == 0
    ));
}

#[test]
fn add_first_missing_skill_requires_abi575() {
    let package = compile_source(
        r#"
skill rebaobian(1) locked {
  trigger damage owner_is_target {
    effect add_first_missing_skill(self, 2, 3, 4);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_575
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
        [EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill,
        }] if player.raw() == 9
            && [skill_defs[0].raw(), skill_defs[1].raw(), skill_defs[2].raw()] == [2, 3, 4]
            && skill.raw() == 0
    ));
}

#[test]
fn resolve_dccunsi_requires_abi578() {
    let package = compile_source(
        r#"
skill dccunsi(1) {
  active play_phase once_per_game targets 1 1 {
    effect resolve_dccunsi(self, event_source, 2, 3, 0);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_578
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage: 0,
            skill,
        }] if player.raw() == 9
            && target.raw() == 7
            && guixiu_def.raw() == 2
            && yongjue_def.raw() == 3
            && skill.raw() == 0
    ));
}

#[test]
fn record_current_sha_all_targets_requires_abi580() {
    let package = compile_source(
        r#"
skill dcwushen(1) locked {
  trigger card_used owner_source_heart_sha {
    effect record_current_sha_all_targets(self, event_card);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_580
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_card: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        }] if player.raw() == 9 && card.raw() == 7 && skill.raw() == 0
    ));
}

#[test]
fn choose_give_or_discard_hand_requires_abi581_and_hand_empty_scope() {
    let package = compile_source(
        r#"
skill dczhaohan_choose(1) locked {
  trigger phase_draw_end owner_is_target {
    effect choose_give_or_discard_hand(self, 2, hand_empty);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_581
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
        [EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count: 2,
            scope: rule_api::DrawTargetScope::HandEmpty,
            skill,
        }] if player.raw() == 9 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_draw_end owner_is_target { effect choose_give_or_discard_hand(self, 2, other); } }"
        )
        .is_err());
}

#[test]
fn optional_discard_then_recover_target_requires_abi582() {
    let package = compile_source(
        r#"
skill dcjinjie(1) {
  trigger dying owner_alive_once_per_round {
    effect choose_discard_hand_equip_then_recover_target(self, event_player, 2, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_582
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
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count: 2,
            amount: 1,
            skill,
        }] if chooser.raw() == 9 && target.raw() == 7 && skill.raw() == 0
    ));
}

#[test]
fn dcjue_resolver_requires_abi583() {
    let package = compile_source(
        r#"
skill dcjue(1) {
  trigger phase_end owner_alive_global_phase_end_discarded_cards_available {
    effect resolve_dcjue(self, event_player);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_583
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
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveDcjue {
            player,
            target,
            stage: 0,
            remaining: 0,
            skill,
        }] if player.raw() == 9 && target.raw() == 7 && skill.raw() == 0
    ));
}

#[test]
fn ruanji_resolver_and_zhaowen_cost_require_abi584() {
    let package = compile_source(
        r#"
skill dczhaowen(1) {
  trigger phase_play owner_is_target_hand_nonempty optional {
    effect resolve_ruanji(self, event_source, event_card, 0);
  }
  view_as ordinary_trick any any from none cost dczhaowen;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_584
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            event_card: 6,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action: 0,
            cards,
            skill,
        }] if player.raw() == 9
            && source.raw() == 7
            && card.raw() == 6
            && cards.is_empty()
            && skill.raw() == 0
    ));
}

#[test]
fn choose_targets_draw_requires_abi80() {
    let package = compile_source(
        r#"
skill stdduanbi(3811377323) {
  trigger phase_end owner_is_target_hand_nonempty optional {
    effect discard_hand(self, 2147483647);
    effect choose_targets_draw(self, 2, 2, any);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_80
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
            EffectRequest::DiscardFromHand {
                player,
                count: 2_147_483_647
            },
            EffectRequest::ChooseTargetsDraw {
                player: draw_player,
                count: 2,
                target_count: 2,
                scope: rule_api::DrawTargetScope::Any,
                ..
            }
        ] if player.raw() == 9 && draw_player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect choose_targets_draw(self, 2, 0, any); } }"
        )
        .is_err());
}

#[test]
fn gain_event_cards_requires_abi82() {
    let package = compile_source(
        r#"
skill xingshang(1) {
  trigger death owner_alive_event_cards_nonempty optional {
    effect gain_event_cards(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_82
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        event_cards: vec![100, 101],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::GainEventCards { player, cards }]
            if player.raw() == 9
                && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [100, 101]
    ));
    assert!(compile_source(
            "skill bad(1) { trigger death owner_alive_event_cards_nonempty { effect gain_event_cards(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn min_hand_draw_target_scope_requires_abi83() {
    let package = compile_source(
        r#"
skill nzry_jianxiang(3232965394) {
  trigger card_targeted owner_alive_targeted_by_other optional {
    effect choose_target_draw(self, 1, min_hand);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_83
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
        [EffectRequest::ChooseTargetDraw {
            player,
            count: 1,
            scope: rule_api::DrawTargetScope::MinHand,
            ..
        }] if player.raw() == 9
    ));
}

#[test]
fn target_draw_to_max_hp_cap_requires_abi84() {
    let package = compile_source(
        r#"
skill jieming(3579069878) {
  trigger damage owner_is_target_per_damage_point optional {
    effect choose_target_draw_to_max_hp_cap(self, 5);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_84
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
        [EffectRequest::ChooseTargetDrawToMaxHpCap {
            player,
            cap: 5,
            ..
        }] if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target_per_damage_point { effect choose_target_draw_to_max_hp_cap(self, 0); } }"
        )
        .is_err());
}

#[test]
fn lost_hp_draw_then_turn_over_requires_abi85() {
    let package = compile_source(
        r#"
skill fangzhu(2335755764) {
  trigger damage owner_is_target_other_alive optional {
    effect choose_other_draw_owner_lost_hp_turn_over(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_85
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
        [EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }]
            if player.raw() == 9
    ));
}

#[test]
fn judgment_resolved_group_filter_requires_abi86() {
    let package = compile_source(
            r#"
skill songwei(2169124968) lord_only {
  trigger judgment_resolved owner_alive_other_judgment_black_event_player_group_wei optional by event_player {
    effect draw_cards(self, 1);
  }
}
"#,
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_86
    );
    assert_eq!(
        package.trigger_index[0].choice_actor,
        content_schema::TriggerChoiceActor::EventPlayer
    );
}

#[test]
fn virtual_bagua_modifier_requires_abi87() {
    let package = compile_source(
        r#"
skill bazhen(3741926095) locked {
  modifier virtual_bagua_if_armor_slot_empty override 1;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_87
    );
}

#[test]
fn other_damage_source_group_judgment_recovery_requires_abi88() {
    let package = compile_source(
        r#"
skill baonue(3659854073) lord_only {
  trigger damage owner_alive_other_damage_source_group_qun optional by event_source {
    effect judge_recover_other_on_suit(event_source, self, spade, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 88);
    assert_eq!(
        package.trigger_index[0].choice_actor,
        content_schema::TriggerChoiceActor::EventSource
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::JudgeRecoverOtherOnSuit {
            source,
            judge_player,
            recover_player,
            suit: rule_api::Suit::Spade,
            amount: 1,
        }] if source.raw() == 9 && judge_player.raw() == 7 && recover_player.raw() == 9
    ));
}

#[test]
fn conditional_sha_response_and_not_min_hp_filter_require_abi89() {
    let package = compile_source(
        r#"
skill roulin(3168437611) locked {
  modifier sha_response_count_if_other_female add 1;
}
skill benghuai(3365741831) locked {
  trigger phase_end owner_is_target_not_min_hp {
    let option = choose(1, 2);
    if option == 1 { effect lose_hp(self, 1); }
    if option == 2 { effect change_max_hp(self, -1); }
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 89);
    assert_eq!(
        package.modifier_index[0].target,
        "sha_response_count_if_other_female"
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_not_min_hp"
    );
}

#[test]
fn ordered_nearest_sha_or_hp_loss_requires_abi90() {
    let package = compile_source(
        r#"
skill luanwu(2772376501) {
  active play_phase once_per_game {
    effect force_others_use_sha_on_nearest_or_lose_hp(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 90);
    assert!(package.active_index[0].once_per_game);
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
        [EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started: false,
            skill,
        }] if player.raw() == 9 && remaining.is_empty() && skill.is_none()
    ));
}

#[test]
fn owner_turn_tao_restriction_requires_abi91() {
    let package = compile_source(
        r#"
skill wansha(2730287699) locked {
  modifier tao_use_save_cancel_for_non_dying_others_during_owner_turn override 1;
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 91);
    let modifier = &package.modifier_index[0];
    assert_eq!(
        modifier.target,
        "tao_use_save_cancel_for_non_dying_others_during_owner_turn"
    );
    assert_eq!(modifier.mode, "override");
    assert_eq!(modifier.value, 1);
}

#[test]
fn other_normal_damage_target_unlinked_filter_requires_abi73() {
    let compiled = compile_source(
        r#"
skill stdzhuying(2174724909) {
  trigger damage_before owner_alive_other_normal_damage_target_unlinked optional {
    effect add_mark(event_player, 2, 1);
  }
}
"#,
    )
    .expect("compile stdzhuying");
    assert_eq!(
        compiled.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_73
    );
}

#[test]
fn targeted_trick_and_character_state_guards_require_abi111() {
    let package = compile_source(
        r#"
skill olwuma(3608420255) locked {
  trigger card_targeted owner_alive_targeted_by_other_trick {
    effect draw_cards(self, 1);
  }
  modifier turn_face_down_cancel override 1;
  modifier restore_skipped_phase override 1;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_111
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_targeted_by_other_trick"
    );
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.target == "turn_face_down_cancel"));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.target == "restore_skipped_phase"));
    assert!(compile_source("skill bad(1) { modifier turn_face_down_cancel add 1; }").is_err());
    assert!(compile_source("skill bad(1) { modifier restore_skipped_phase override 2; }").is_err());
}

#[test]
fn other_player_discarded_black_filter_requires_abi112() {
    let package = compile_source(
        r#"
skill olhaizhu(2513379804) locked {
  trigger cards_moved owner_alive_other_discarded_black_to_discard {
    effect gain_event_cards(self);
  }
  trigger phase_prepare owner_is_target_tied_max_hand {
    effect lose_hp(self, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_112
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_discarded_black_to_discard"
    );
}

#[test]
fn larger_hand_active_and_repeated_take_require_abi113() {
    let package = compile_source(
        r#"
skill olzishu(2483133612) {
  active play_phase uses_per_turn 1 targets 1 1 other hand_gt_owner {
    effect take_target_hand_then_repeat_until_max(self, event_source, self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_113
    );
    let active = &package.active_index[0];
    assert!(active.targets.as_ref().unwrap().hand_gt_owner);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::TAKE_TARGET_HAND_THEN_REPEAT_UNTIL_MAX,
                ),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 other { effect take_target_hand_then_repeat_until_max(self, event_source, self_skill); } }"
        )
        .is_err());
}

#[test]
fn current_card_three_way_branch_requires_abi114_and_card_used() {
    let package = compile_source(
        r#"
skill mbhanyong(2639258372) {
  trigger card_used owner_alive_wounded_card_targets_gt_one optional {
    effect choose_draw_or_increase_current_card_damage_or_both_mark(
      self, event_card, 2, 1, 1206920502, 1
    );
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_114
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_wounded_card_targets_gt_one"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK),
                ]
        }));
    assert_ne!(
        package.modules[0].caps_mask & rule_vm::bytecode::caps::REQUEST_CHOICE,
        0
    );
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_source { effect choose_draw_or_increase_current_card_damage_or_both_mark(self, event_card, 2, 1, 1, 1); } }"
        )
        .is_err());
}

#[test]
fn current_play_phase_sha_or_trick_and_first_card_distance_require_abi115() {
    let package = compile_source(
        r#"
skill drlt_wanglie(3599509027) {
  modifier card_distance_unlimited override owner_play_phase_card_use_count_zero;
  trigger card_used owner_source_sha_or_trick_current_play_phase optional {
    effect add_skill(self, 2154023320);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_115
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_or_trick_current_play_phase"
    );
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("owner_play_phase_card_use_count_zero")
    );
    assert!(compile_source(
        "skill bad(1) { modifier hand_limit add owner_play_phase_card_use_count_zero; }"
    )
    .is_err());
}

#[test]
fn staged_draw_give_or_discard_hand_requires_abi116_and_active_skill() {
    let package = compile_source(
        r#"
skill scsxiaolu(3320236272) {
  active play_phase uses_per_turn 1 {
    effect draw_then_choose_give_or_discard_hand(self, 2, 2);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_116
    );
    assert_eq!(package.active_index[0].uses_per_turn, Some(1));
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND,
                ),
            ]
    }));
    assert_ne!(
        package.modules[0].caps_mask & rule_vm::bytecode::caps::REQUEST_CHOICE,
        0
    );
    assert!(compile_source(
            "skill bad(1) { trigger phase_play owner_is_target { effect draw_then_choose_give_or_discard_hand(self, 2, 2); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase { effect draw_then_choose_give_or_discard_hand(self, 2); } }"
        )
        .is_err());
}

#[test]
fn gain_target_equipment_and_weapon_source_filter_require_abi117() {
    let package = compile_source(
        r#"
skill xinduodao(3614910583) {
  trigger damage owner_is_target_source_has_weapon optional {
    effect gain_target_equipment(self, event_source, weapon);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_117
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_source_has_weapon"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::GAIN_TARGET_EQUIPMENT),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target_source_has_weapon { effect gain_target_equipment(self, event_source, trick); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target_source_has_weapon { effect gain_target_equipment(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn discard_all_target_equipment_and_discardable_filter_require_abi118() {
    let package = compile_source(
        r#"
skill ly_piliche(2383388842) {
  trigger damage owner_source_other_target_alive_has_discardable_hand_equip optional {
    effect discard_all_target_equipment(self, event_player);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_118
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_other_target_alive_has_discardable_hand_equip"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::DISCARD_ALL_TARGET_EQUIPMENT),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_source_other_target_alive_has_discardable_hand_equip { effect discard_all_target_equipment(self); } }"
        )
        .is_err());
}

#[test]
fn current_sha_target_damage_and_outside_range_filter_require_abi119() {
    let package = compile_source(
        r#"
skill xinanjian(2652033709) locked {
  trigger card_targeted owner_source_sha_target_alive_owner_outside_target_attack_range {
    let pick = choose(1, 2);
    if pick == 1 {
      effect make_current_sha_target_unrespondable(self, event_card, event_player);
    }
    if pick == 2 {
      effect increase_current_sha_target_damage(self, event_card, event_player, 1);
    }
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_119
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_target_alive_owner_outside_target_attack_range"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::INCREASE_CURRENT_SHA_TARGET_DAMAGE,
                ),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger card_targeted owner_source_sha_target_alive_owner_outside_target_attack_range { effect increase_current_sha_target_damage(self, event_card, event_player); } }"
        )
        .is_err());
}

#[test]
fn target_draw_then_owner_check_requires_abi120() {
    let package = compile_source(
        r#"
skill rejieming(4088661605) {
  trigger damage owner_is_target_per_damage_point optional {
    effect choose_target_draw_then_owner_if_below_max_hp(self, 2);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_120
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_per_damage_point"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_TARGET_DRAW_THEN_OWNER_IF_BELOW_MAX_HP,
                ),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target_per_damage_point { effect choose_target_draw_then_owner_if_below_max_hp(self); } }"
        )
        .is_err());
}

#[test]
fn target_draw_cap_then_discard_excess_requires_abi122() {
    let package = compile_source(
        r#"
skill oljieming(2506669514) {
  trigger damage owner_is_target_per_damage_point optional {
    effect choose_target_draw_max_hp_cap_then_discard_excess(self, 5);
  }
  trigger death owner_is_target optional {
    effect choose_target_draw_max_hp_cap_then_discard_excess(self, 5);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_122
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_DISCARD_EXCESS),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { trigger death owner_is_target { effect choose_target_draw_max_hp_cap_then_discard_excess(self, 0); } }"
        )
        .is_err());
}

#[test]
fn selected_gift_same_suit_discard_or_hp_requires_abi123() {
    let package = compile_source(
        r#"
skill refanjian(2975386328) {
  active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1 {
    effect give_selected_then_target_choose_discard_same_suit_or_lose_hp(self, event_source);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_123
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { active play_phase { effect give_selected_then_target_choose_discard_same_suit_or_lose_hp(self); } }"
        )
        .is_err());
}

#[test]
fn gain_equipment_from_draw_pile_requires_abi124() {
    let package = compile_source(
            "skill xinpingkou(3441378011) { trigger phase_end owner_is_target_skipped_phase_count_positive optional { effect gain_equipment_from_draw_pile(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_124
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::GAIN_EQUIPMENT_FROM_DRAW_PILE),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect gain_equipment_from_draw_pile(); } }"
        )
        .is_err());
}

#[test]
fn give_all_hand_recover_to_one_requires_abi125() {
    let package = compile_source(
            "skill stdwanlan(3664427038) { trigger dying owner_alive_other_dying_hand_nonempty_once_per_game optional { effect give_all_hand_recover_to_one(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_125
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::GIVE_ALL_HAND_RECOVER_TO_ONE),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger dying owner_alive_other_dying_hand_nonempty_once_per_game { effect give_all_hand_recover_to_one(self); } }"
        )
        .is_err());
}

#[test]
fn sha_or_duel_damage_participants_filter_requires_abi126() {
    let package = compile_source(
            "skill stdzongji(3394363597) { trigger damage owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty optional { effect choose_discard_target_card(self, event_player, hand_equip); effect choose_discard_target_card(self, event_source, hand_equip); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_126
    );
}

#[test]
fn self_discard_then_hand_hp_check_requires_abi127() {
    let package = compile_source(
            "skill stdbingzheng(3798362737) { trigger phase_end owner_is_target_any_alive_self_discardable_hand_equip optional { effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_127
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target_any_alive_self_discardable_hand_equip { effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn literal_hand_limit_active_target_requires_abi128() {
    let package = compile_source(
            "skill stdzhuguo(3147233152) { active play_phase uses_per_turn 1 targets 1 1 hand_lt 2 { effect draw_cards(event_source, 2 - hand_count(event_source)); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_128
    );
    assert_eq!(
        package.active_index[0].targets.as_ref().unwrap().hand_lt,
        Some(2)
    );
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 hand_lt 2 hand_lt 3 { effect draw_cards(event_source, 1); } }"
        )
        .is_err());
}

#[test]
fn choose_give_one_hand_equip_or_recover_requires_abi129() {
    let package = compile_source(
            "skill stdanda(3646320211) { trigger dying owner_alive_source_alive_event_player_dying_once_per_round optional { effect choose_give_one_hand_equip_or_recover(self, event_source, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_129
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER,
                ),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger dying owner_alive_source_alive_event_player_dying_once_per_round { effect choose_give_one_hand_equip_or_recover(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn active_armor_bound_skill_requires_abi130() {
    let package = compile_source(
            "skill stdfenxun(2579637455) { active play_phase uses_per_turn 1 targets 1 1 other select_he 1 1 card_armor { effect discard_selected(self); effect add_bound_skill(self, 3372226216, event_source); } } skill stdfenxun_effect(3372226216) { modifier card_distance_unlimited_for_skill_bound_target add 1; trigger turn_ended owner_is_target { effect remove_skill(self_skill); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_130
    );
    assert_eq!(
        package.active_index[0]
            .selection
            .as_ref()
            .and_then(|selection| selection.filter.as_deref()),
        Some("card_armor")
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::ADD_BOUND_SKILL),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { active play_phase select_he 1 1 card_armor { effect add_bound_skill(self, 2); } }"
        )
        .is_err());
}

#[test]
fn repeated_card_color_discard_filter_requires_abi131() {
    let package = compile_source(
            "skill stdfenyin(2225023788) { trigger phase_draw owner_is_target optional { effect increase_current_draw_count(self, 2); effect add_skill(self, 3965200877); } } skill stdfenyin_discard(3965200877) locked { trigger card_used owner_source_repeats_color_has_discardable_hand_equip { effect discard_one_hand_or_equip(self); } trigger turn_ended owner_is_target { effect remove_skill(self_skill); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_131
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_965_200_877
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_repeats_color_has_discardable_hand_equip"
    }));
}

#[test]
fn post_resolution_bound_skill_and_literal_hp_target_require_abi132() {
    let package = compile_source(
            "skill stdmingfa(2715469127) { active play_phase targets 1 1 hp_gt 1 { effect deal_damage(self, event_source, 1); effect replace_skill_with_bound_after_resolution_if_target_alive(self, self_skill, 2582921343, event_source); } } skill stdmingfa_used(2582921343) locked { trigger hp_recovered owner_alive_skill_bound_target_is_event_player { effect add_skill(self, 2715469127); effect remove_skill(self_skill); } trigger death owner_alive_skill_bound_target_is_event_player { effect add_skill(self, 2715469127); effect remove_skill(self_skill); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_132
    );
    assert_eq!(
        package.active_index[0].targets.as_ref().unwrap().hp_gt,
        Some(1)
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_582_921_343
            && trigger.event == "hp_recovered"
            && trigger.filter == "owner_alive_skill_bound_target_is_event_player"
    }));
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 hp_gt 1 hp_gt 2 { effect draw_cards(self, 1); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 hp_gt 1 { effect replace_skill_with_bound_after_resolution_if_target_alive(self, self_skill, 2); } }"
        )
        .is_err());
}

#[test]
fn full_zone_discard_draw_lost_hp_then_lose_hp_requires_abi133() {
    let package = compile_source(
            "skill stdzaoli(2735843050) locked { trigger phase_prepare owner_is_target { effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_133
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { trigger phase_prepare owner_is_target { effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(self, 1); } }"
        )
        .is_err());
}

#[test]
fn exact_discard_then_recover_and_qun_source_filter_require_abi134() {
    let package = compile_source(
            "skill stdjujin(2282519497) lord_only { trigger damage owner_is_target_source_group_qun_discardable_hand_equip_gte_two optional { effect choose_discard_hand_equip_then_recover(self, 2, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_134
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_282_519_497
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_group_qun_discardable_hand_equip_gte_two"
    }));
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER,
                ),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_discard_hand_equip_then_recover(self, 2); } }"
        )
        .is_err());
}

#[test]
fn ranged_target_damage_choice_requires_abi135() {
    let package = compile_source(
            "skill stdxiongsuan(3084450928) locked { trigger phase_prepare owner_is_target_tied_max_hp { effect choose_targets_damage(self, 1, 255, same_hp, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_135
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_TARGETS_DAMAGE),
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger phase_prepare owner_is_target { effect choose_targets_damage(self, 0, 2, same_hp, 1); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_prepare owner_is_target { effect choose_targets_damage(self, 2, 1, same_hp, 1); } }"
        )
        .is_err());
}

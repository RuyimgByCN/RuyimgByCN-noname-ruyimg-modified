use super::*;

#[test]
fn distinct_suit_draw_and_heart_inspection_require_abi209() {
    let package = compile_source(
        r#"
skill shelie(4211343786) {
  trigger phase_draw owner_is_target {
    effect replace_draw_with_distinct_suit_selection(self, 5);
  }
}
skill gongxin(3100453257) {
  active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty {
    effect inspect_target_hand_choose_heart_discard_or_top(self, event_source);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_209
    );
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(
                        rule_vm::bytecode::effect_tag::REPLACE_DRAW_WITH_DISTINCT_SUIT_SELECTION,
                    ),
                ]
        })
    }));
    assert!(package.modules.iter().any(|module| {
            module.code.windows(2).any(|bytes| {
                bytes
                    == [
                        rule_vm::bytecode::op::HOST_EFFECT,
                        legacy_effect_tag(rule_vm::bytecode::effect_tag::INSPECT_TARGET_HAND_CHOOSE_HEART_DISCARD_OR_TOP),
                    ]
            })
        }));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 { effect inspect_target_hand_choose_heart_discard_or_top(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn yeyan_and_qinyin_require_abi210() {
    let package = compile_source(
        r#"
skill yeyan(3293607680) {
  active play_phase once_per_game {
    effect resolve_yeyan(self);
  }
}
skill qinyin(3841403199) {
  trigger phase_discard_end owner_is_target_discarded_at_least_two_in_discard_phase {
    effect choose_all_lose_hp_or_recover(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_210
    );
    for tag in [
        rule_vm::bytecode::effect_tag::RESOLVE_YEYAN,
        rule_vm::bytecode::effect_tag::CHOOSE_ALL_LOSE_HP_OR_RECOVER,
    ] {
        assert!(package.modules.iter().any(|module| {
            module
                .code
                .windows(2)
                .any(|bytes| bytes == [rule_vm::bytecode::op::HOST_EFFECT, legacy_effect_tag(tag)])
        }));
    }
    assert!(
        compile_source("skill bad(1) { active play_phase { effect resolve_yeyan(self); } }")
            .is_err()
    );
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect choose_all_lose_hp_or_recover(self); } }"
        )
        .is_err());
}

#[test]
fn gain_from_each_other_draw_replacement_requires_abi300() {
    let package = compile_source(
        "skill hengzheng(3568349953) {
                trigger phase_draw owner_is_target_draw_not_fixed_hp_one_or_hand_empty optional {
                    effect choose_gain_card_from_each_other_replace_draw(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_300
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW
                    & 0xff) as u8,
                (rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW >> 8)
                    as u8,
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger phase_draw owner_is_target { effect choose_gain_card_from_each_other_replace_draw(self, 1); } }"
        )
        .is_err());
}

#[test]
fn dingpan_dynamic_limit_and_effect_require_abi302() {
    let package = compile_source(
        "skill dingpan(3739676314) {
                active play_phase uses_per_turn living_rebel_count targets 1 1 has_equip {
                    effect resolve_dingpan(self, event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_302
    );
    assert_eq!(
        package.active_index[0].uses_per_turn_source.as_deref(),
        Some("living_rebel_count")
    );
    assert!(package.active_index[0]
        .targets
        .as_ref()
        .is_some_and(|targets| targets.allow_self && targets.has_equip));
    assert_eq!(
        package.modules[0].code,
        [
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EVENT_SOURCE,
            rule_vm::bytecode::op::HOST_EFFECT_EXT,
            (rule_vm::bytecode::effect_tag::RESOLVE_DINGPAN & 0xff) as u8,
            (rule_vm::bytecode::effect_tag::RESOLVE_DINGPAN >> 8) as u8,
            rule_vm::bytecode::op::HALT,
        ]
    );
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn living_rebel_count targets 1 1 { effect resolve_dingpan(self, event_source); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active any uses_per_turn living_rebel_count targets 1 1 has_equip { effect resolve_dingpan(self, event_source); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn living_rebel_count targets 1 1 other has_equip { effect resolve_dingpan(self, event_source); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger cards_drawn owner_is_target { effect resolve_dingpan(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn tanlu_phase_begin_contract_and_effect_require_abi303() {
    let package = compile_source(
        "skill xktanlu(4005608784) {
                trigger phase_begin owner_alive_not_target optional {
                    effect resolve_tanlu(self, event_player);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_303
    );
    assert_eq!(
        package.modules[0].code,
        [
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EVENT_PLAYER,
            rule_vm::bytecode::op::HOST_EFFECT_EXT,
            (rule_vm::bytecode::effect_tag::RESOLVE_TANLU & 0xff) as u8,
            (rule_vm::bytecode::effect_tag::RESOLVE_TANLU >> 8) as u8,
            rule_vm::bytecode::op::HALT,
        ]
    );
    for source in [
            "skill bad(1) { trigger phase_prepare owner_alive_not_target optional { effect resolve_tanlu(self, event_player); } }",
            "skill bad(1) { trigger phase_begin owner_alive optional { effect resolve_tanlu(self, event_player); } }",
            "skill bad(1) { trigger phase_begin owner_alive_not_target { effect resolve_tanlu(self, event_player); } }",
            "skill bad(1) { active any { effect resolve_tanlu(self, event_source); } }",
        ] {
            assert!(compile_source(source).is_err());
        }
}

#[test]
fn active_same_suit_selection_requires_abi304() {
    let package = compile_source(
        "skill spfuluan(2671318114) {
                active play_phase uses_per_turn 1 targets 1 1 in_attack_range
                    select_he 3 3 allow_cancel same_suit {
                    effect discard_selected(self);
                    effect turn_over(event_source);
                    effect add_skill_until_turn_end(self, 4252815092);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_304
    );
    let selection = package.active_index[0].selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (3, 3));
    assert!(selection.allow_cancel && selection.same_suit);
    assert!(!selection.same_color);
}

#[test]
fn jdlianhuan_target_filter_requires_abi305() {
    let package = compile_source(
            "skill jdlianhuan(3773194665) {
                view_as tiesuo play_phase club from hand;
                trigger card_targeted owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty {
                    effect choose_optional_discard_target_card(self, event_player, hand_equip);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_305
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty"
    );
    assert!(!package.trigger_index[0].optional);
}

#[test]
fn pre_damage_linked_target_filter_requires_abi306() {
    let package = compile_source(
        "skill jyliezhou(3307262280) locked {
                modifier damage_nature override 1;
                trigger damage owner_alive_source_target_was_linked {
                    effect draw_cards(self, event_amount);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_306
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_source_target_was_linked"
    );
    assert!(!package.trigger_index[0].optional);
}

#[test]
fn same_color_judgment_repeat_requires_abi307() {
    let package = compile_source(
        "skill yjluoshen(3100602929) {
                trigger phase_prepare owner_is_target optional {
                    effect judge_same_color_gain_repeat(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_307
    );
    assert_eq!(
        package.modules[0].code,
        [
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EFFECT_EXT,
            (rule_vm::bytecode::effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT & 0xff) as u8,
            (rule_vm::bytecode::effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT >> 8) as u8,
            rule_vm::bytecode::op::HALT,
        ]
    );
}

#[test]
fn current_trick_target_nullification_requires_abi308() {
    let package = compile_source(
        "skill fenwei(3203648912) {
                trigger card_used owner_alive_non_multitarget_trick_targets_gt_one {
                    effect choose_current_trick_targets_to_nullify(self, event_source, event_card);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_308
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_non_multitarget_trick_targets_gt_one"
    );
    assert!(!package.trigger_index[0].optional);
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY & 0xff)
                    as u8,
                (rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY >> 8) as u8,
            ]
    }));

    let wrong_event = "skill bad(1) {
            trigger damage owner_is_target {
                effect choose_current_trick_targets_to_nullify(self, event_source, event_card);
            }
        }";
    assert!(compile_source(wrong_event)
        .unwrap_err()
        .to_string()
        .contains("requires `trigger card_used"));
}

#[test]
fn protected_black_judgment_repeat_requires_abi309() {
    let package = compile_source(
        "skill reluoshen(3265545896) {
                trigger phase_prepare owner_is_target optional {
                    effect judge_black_gain_repeat_protect_hand_limit(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_309
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT & 0xff)
                    as u8,
                (rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT >> 8)
                    as u8,
            ]
    }));
}

#[test]
fn all_resolved_sha_gift_requires_abi310() {
    let package = compile_source(
            "skill rezhongyong(3348933734) {
                trigger sha_resolved owner_source_sha_resolved_has_gainable_cards_and_recipient optional {
                    effect choose_gain_all_resolved_sha_shan_then_color_rewards(self, event_card);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_310
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS
                    & 0xff) as u8,
                (rule_vm::bytecode::effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS
                    >> 8) as u8,
            ]
    }));
}

#[test]
fn reduced_tuxi_requires_abi311() {
    let package = compile_source(
        "skill new_retuxi(3295460539) {
                trigger phase_draw owner_is_target_positive_draw_other_hand_nonempty optional {
                    effect choose_other_hand_targets_gain_one_reduce_draw(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_311
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW
                    & 0xff) as u8,
                (rule_vm::bytecode::effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW >> 8)
                    as u8,
            ]
    }));
}

#[test]
fn rejianchu_nonbasic_branch_requires_abi312() {
    let package = compile_source(
            "skill rejianchu(3270398373) {
                trigger card_targeted owner_source_sha_target_alive_has_discardable_hand_equip optional {
                    effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus(
                        self, event_card, event_player, 3848225310
                    );
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_312
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT_EXT,
                    (rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS
                        & 0xff) as u8,
                    (rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS
                        >> 8) as u8,
                ]
        }));
}

#[test]
fn dcqiaomeng_requires_abi313() {
    let package = compile_source(
            "skill dcqiaomeng(3871427916) {
                trigger card_targeted owner_source_first_target_black_card_any_target_other_discardable_hand_equip optional {
                    effect resolve_dcqiaomeng(self, event_card, 2662675115);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_313
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::RESOLVE_DCQIAOMENG & 0xff) as u8,
                (rule_vm::bytecode::effect_tag::RESOLVE_DCQIAOMENG >> 8) as u8,
            ]
    }));
}

#[test]
fn repindi_requires_abi314() {
    let package = compile_source(
            "skill repindi(3416188911) {
                active play_phase
                    targets 1 1 unused_by_skill_this_play_phase
                    select_he 1 1 unused_category_by_skill_this_play_phase {
                    effect discard_selected_record_unique_category_target(self, event_source);
                    effect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded(self, event_source);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_314
    );
    let active = package.active_index.first().unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert!(targets.allow_self);
    assert!(targets.unused_by_skill_this_play_phase);
    assert!(
        active
            .selection
            .as_ref()
            .unwrap()
            .unused_category_by_skill_this_play_phase
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT_EXT,
                    (rule_vm::bytecode::effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED
                        & 0xff) as u8,
                    (rule_vm::bytecode::effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED
                        >> 8) as u8,
                ]
        }));
}

#[test]
fn xinjiangchi_three_way_choice_requires_abi315() {
    let package = compile_source(
        "skill xinjiangchi(3041318040) {
                trigger phase_play owner_is_target {
                    effect choose_three_way_draw_or_discard_add_skill(
                        self, 2327944393, 3884080518
                    );
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_315
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL & 0xff)
                    as u8,
                (rule_vm::bytecode::effect_tag::CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL >> 8)
                    as u8,
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
        [EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            three_way: true,
            ..
        }] if player.raw() == 9
            && draw_skill_def.raw() == 2327944393
            && discard_skill_def.raw() == 3884080518
    ));
}

#[test]
fn retieji_full_resolution_requires_abi316() {
    let package = compile_source(
        "skill retieji(3731623167) {
                trigger card_targeted owner_source_sha optional {
                    effect resolve_retieji(self, event_card, event_player);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_316
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::RESOLVE_RETIEJI & 0xff) as u8,
                (rule_vm::bytecode::effect_tag::RESOLVE_RETIEJI >> 8) as u8,
            ]
    }));
}

#[test]
fn new_reluoyi_full_resolution_requires_abi317() {
    use rule_api::EffectRequest;
    let package = compile_source(
        "skill new_reluoyi(3935960913) {
                trigger phase_draw owner_is_target_draw_not_replaced {
                    effect resolve_new_reluoyi(self, 3768135792);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_317
    );
    assert!(package.modules[0].code.windows(3).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT_EXT,
                (rule_vm::bytecode::effect_tag::RESOLVE_NEW_RELUOYI & 0xff) as u8,
                (rule_vm::bytecode::effect_tag::RESOLVE_NEW_RELUOYI >> 8) as u8,
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
        [EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage: 0,
            cards,
            accepted: false,
            skill,
        }] if player.raw() == 9
            && buff_skill_def.raw() == 3768135792
            && cards.is_empty()
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect resolve_new_reluoyi(self, 2); } }"
        )
        .is_err());
}

#[test]
fn remoshi_place_and_retrieve_require_abi318() {
    use rule_api::EffectRequest;
    let package = compile_source(
            "skill remoshi(3096399491) locked {
                trigger damage owner_source_direct_sha_damage_target_armor_or_mount_card_gainable {
                    effect resolve_remoshi(self, event_player, event_card, 4090725192, 0);
                }
                trigger cards_moved owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile {
                    effect resolve_remoshi(self, event_player, event_card, 4090725192, 1);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_318
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 10,
            event_card: 11,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action: 0,
            skill,
        }] if player.raw() == 9
            && target.raw() == 10
            && card.raw() == 11
            && helper_skill_def.raw() == 4090725192
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect resolve_remoshi(self, event_player, event_card, 2, 0); } }"
        )
        .is_err());
}

#[test]
fn xinyaoming_target_and_action_choice_require_abi319() {
    use rule_api::EffectRequest;
    let package = compile_source(
        "skill xinyaoming(3897075796) {
                trigger damage owner_is_source_or_target_xinyaoming_available optional {
                    effect resolve_xinyaoming(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_319
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
        [EffectRequest::ResolveXinyaoming {
            player,
            target,
            action: 0,
            skill,
        }] if player.raw() == 9 && target.is_none() && skill.is_none()
    ));
    assert!(compile_source(
        "skill bad(1) { trigger phase_end owner_is_target { effect resolve_xinyaoming(self); } }"
    )
    .is_err());
}

#[test]
fn rehuaiyi_staged_resolution_requires_abi320() {
    use rule_api::EffectRequest;
    let package = compile_source(
        "skill rehuaiyi(4054411167) {
                active play_phase uses_per_play_phase 2 hand_nonempty {
                    effect resolve_rehuaiyi(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_320
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
        [EffectRequest::ResolveRehuaiyi {
            player,
            stage: 0,
            color: 0,
            discarded_count: 0,
            targets,
            index: 0,
            gained_count: 0,
            skill,
        }] if player.raw() == 9 && targets.is_empty() && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_play_phase 1 hand_nonempty { effect resolve_rehuaiyi(self); } }"
        )
        .is_err());
    assert!(compile_source(
        "skill bad(1) { trigger phase_end owner_is_target { effect resolve_rehuaiyi(self); } }"
    )
    .is_err());
}

#[test]
fn reanguo_shared_minimum_flow_and_recast_require_abi321() {
    use rule_api::EffectRequest;
    let package = compile_source(
        "skill reanguo(3231751411) {
                active play_phase uses_per_turn 1 targets 1 1 other {
                    effect resolve_reanguo(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_321
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        selected_targets: vec![10],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveReanguo {
            player,
            target,
            stage: 0,
            index: 0,
            remaining: 0b111,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && skill.is_none()
    ));
    assert!(compile_source(
        "skill bad(1) { active play_phase uses_per_turn 1 { effect resolve_reanguo(self); } }"
    )
    .is_err());
    assert!(compile_source(
        "skill bad(1) { trigger phase_end owner_is_target { effect resolve_reanguo(self); } }"
    )
    .is_err());
}

#[test]
fn distance_hand_discard_or_damage_requires_abi322() {
    use rule_api::EffectRequest;
    let package = compile_source(
            "skill xinqingxi(2962941234) {
                trigger damage_before owner_source_other_damage_before_once_per_turn optional {
                    effect choose_target_discard_distance_hand_or_increase_damage(self, event_player);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_322
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_other_damage_before_once_per_turn"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 10,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id: 0,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage_before owner_source_other_damage_before_once_per_turn { effect choose_target_discard_distance_hand_or_increase_damage(self); } }"
        )
        .is_err());
}

#[test]
fn draw_view_two_gain_or_move_equipment_requires_abi323() {
    use rule_api::EffectRequest;
    let package = compile_source(
        "skill xinqieting(3722360517) {
                trigger turn_ended owner_alive_other_turn_ended_no_damage_other {
                    effect choose_draw_view_two_gain_or_move_equipment(self, event_player);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_323
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_turn_ended_no_damage_other"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 10,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
            player,
            target,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger turn_ended owner_alive_other_turn_ended_no_damage_other { effect choose_draw_view_two_gain_or_move_equipment(self); } }"
        )
        .is_err());
}

#[test]
fn resolve_beizhu_requires_abi324_and_exact_active_contract() {
    use rule_api::EffectRequest;
    let package = compile_source(
        "skill beizhu(1) {
                active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty {
                    effect resolve_beizhu(self, event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_324
    );
    let active = &package.active_index[0];
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 10,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveBeizhu {
            player,
            target,
            stage: 0,
            cards,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && cards.is_empty() && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 targets 1 1 other { effect resolve_beizhu(self, event_source); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect resolve_beizhu(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn chengzhao_filter_requires_abi325_and_reuses_compare_virtual_card() {
    use rule_api::EffectRequest;
    let package = compile_source(
        r#"
skill chengzhao(3452131931) {
  trigger phase_end owner_alive_gained_two_this_turn_can_compare_other optional {
    effect choose_hand_compare(self, 2503452290, 4247003544);
  }
}
skill chengzhao_win(2503452290) {
  modifier card_distance_unlimited(sha) override 1;
  modifier ignore_armor override 1;
  trigger hand_compare_finished owner_is_target {
    effect use_virtual_card_on(self, sha, event_source);
    effect remove_skill(self_skill);
  }
}
skill chengzhao_lose(4247003544) {
  trigger hand_compare_finished owner_is_target {
    effect remove_skill(self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_325
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_gained_two_this_turn_can_compare_other"
    );
    let win = package
        .modules
        .iter()
        .find(|module| module.name == "chengzhao_win_hand_compare_finished")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
            self_id: 9,
            event_source: 10,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new().execute(&vmod(win), &mut host, 100).unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::UseVirtualCardOn {
                player,
                kind: 1,
                target,
            },
            EffectRequest::RemoveSkill { skill },
        ] if player.raw() == 9 && target.raw() == 10 && skill.raw() == 11
    ));
}

#[test]
fn mbshanjia_requires_abi326_and_emits_staged_resolver() {
    let package = compile_source(
        r#"
skill mbshanjia(3457547943) {
  trigger cards_moved owner_from_equip_per_card {
    effect add_mark(self, 3457547943, 1);
  }
  active play_phase uses_per_turn 1 {
    effect resolve_mbshanjia(self, mark_count(self, 3457547943), 4049419240);
  }
}
skill mbshanjia_nodis(4049419240) locked non_gainable {
  modifier card_distance_unlimited override 1;
  trigger phase_play_end owner_is_target {
    effect remove_skill(self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_326
    );
    assert_eq!(package.trigger_index[0].filter, "owner_from_equip_per_card");
    let active = package
        .modules
        .iter()
        .find(|module| module.name == "mbshanjia_active_play_phase_0")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
            self_id: 9,
            ..Default::default()
        },
        query_stub_int: 2,
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(active), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count: 2,
            distance_skill_def,
            stage: 0,
            reward_mask: 0,
            skill,
        }] if player.raw() == 9
            && distance_skill_def.raw() == 4_049_419_240
            && skill.is_none()
    ));
    assert!(compile_source(
        "skill bad(1) { trigger phase_end always { effect resolve_mbshanjia(self, 0, 2); } }"
    )
    .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 targets 1 1 other { effect resolve_mbshanjia(self, 0, 2); } }"
        )
        .is_err());
}

#[test]
fn daoji_requires_abi327_and_emits_staged_resolver() {
    let package = compile_source(
            r#"
skill daoji(4263252675) {
  active play_phase uses_per_turn 1 targets 1 1 other has_gainable_equip select_he 1 1 card_nonbasic {
    effect discard_selected(self);
    effect resolve_daoji(self, event_source);
  }
}
"#,
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_327
    );
    let active = &package.active_index[0];
    assert!(active.targets.as_ref().unwrap().has_gainable_equip);
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_nonbasic")
    );
    let module = package
        .modules
        .iter()
        .find(|module| module.name == "daoji_active_play_phase_0")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
            self_id: 9,
            event_source: 10,
            ..Default::default()
        },
        selected_cards: vec![12],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DiscardCards {
                player,
                cards,
            },
            EffectRequest::ResolveDaoji {
                player: resolver,
                target,
                card,
                stage: 0,
                tw: false,
                skill,
            }
        ] if player.raw() == 9
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [12]
            && resolver.raw() == 9
            && target.raw() == 10
            && card.is_none()
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 targets 1 1 other has_equip select_he 1 1 card_nonbasic { effect resolve_daoji(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn tw_daoji_requires_abi355_and_emits_tw_resolver() {
    let package = compile_source(
            r#"
skill twdaoji(3856876988) {
  active play_phase uses_per_turn 1 targets 1 1 other has_gainable_hand_equip in_attack_range select_he 1 1 card_nonbasic {
    effect discard_selected(self);
    effect resolve_tw_daoji(self, event_source);
  }
}
"#,
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_355
    );
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert!(targets.has_gainable_hand_equip);
    assert!(targets.in_attack_range);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == "twdaoji_active_play_phase_0")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
            self_id: 9,
            event_source: 10,
            ..Default::default()
        },
        selected_cards: vec![12],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DiscardCards { .. },
            EffectRequest::ResolveDaoji {
                tw: true,
                stage: 0,
                ..
            }
        ]
    ));
}

#[test]
fn jiaohua_requires_abi328_and_emits_staged_resolver() {
    let package = compile_source(
        r#"
skill jiaohua(4264685853) {
  active play_phase uses_per_play_phase 2 {
    effect resolve_jiaohua(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_328
    );
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_play_phase, Some(2));
    assert!(active.targets.is_none());
    assert!(active.selection.is_none());
    let module = package
        .modules
        .iter()
        .find(|module| module.name == "jiaohua_active_play_phase_0")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
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
        [EffectRequest::ResolveJiaohua {
            player,
            category: 0,
            target,
            skill,
        }] if player.raw() == 9 && target.is_none() && skill.is_none()
    ));
    assert!(compile_source(
        "skill bad(1) { active play_phase uses_per_turn 2 { effect resolve_jiaohua(self); } }"
    )
    .is_err());
}

#[test]
fn luanqun_requires_abi329_and_emits_private_reveal_resolver() {
    let package = compile_source(
        r#"
skill luanqun(3119764480) {
  active play_phase uses_per_turn 1 hand_nonempty {
    effect resolve_luanqun(self, 3973186966, 3537894821);
  }
}
skill luanqun_effect(3973186966) locked {
  trigger card_used owner_source_sha_current_play_phase {
    effect remove_skill(self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_329
    );
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_nonempty);
    assert!(active.targets.is_none());
    assert!(active.selection.is_none());
    let module = package
        .modules
        .iter()
        .find(|module| module.name == "luanqun_active_play_phase_0")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
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
        [EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage: 0,
            participants,
            cards,
            skill,
        }] if player.raw() == 9
            && effect_skill_def.raw() == 3_973_186_966
            && direct_hit_skill_def.raw() == 3_537_894_821
            && participants.is_empty()
            && cards.is_empty()
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 { effect resolve_luanqun(self, 2, 3); } }"
        )
        .is_err());
}

#[test]
fn different_color_gift_or_recover_requires_abi330() {
    let package = compile_source(
            "skill mbanda(2849838127) { trigger dying owner_alive_source_alive_event_player_dying_once_per_round optional { effect choose_give_two_different_colors_or_recover(self, event_source, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_330
    );
    let trigger = &package.trigger_index[0];
    assert_eq!(trigger.event, "dying");
    assert_eq!(
        trigger.filter,
        "owner_alive_source_alive_event_player_dying_once_per_round"
    );
    assert!(trigger.optional);
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
            self_id: 9,
            event_source: 8,
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
        [EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            skill,
        }] if owner.raw() == 9
            && giver.raw() == 8
            && target.raw() == 7
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger dying owner_alive_source_alive_event_player_dying_once_per_round { effect choose_give_two_different_colors_or_recover(self, event_source); } }"
        )
        .is_err());
}

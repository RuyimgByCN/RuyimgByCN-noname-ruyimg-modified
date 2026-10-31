use super::*;

#[test]
fn hand_empty_target_scope_requires_abi162() {
    let package = compile_source(
            "skill xinjuece(3145104894) { trigger phase_end owner_is_target_any_alive_hand_empty optional { effect choose_targets_damage(self, 1, 1, hand_empty, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_162
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_145_104_894
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_any_alive_hand_empty"
            && trigger.optional
    }));
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_TARGETS_DAMAGE),
            ]
    }));
}

#[test]
fn draw_reshuffle_filters_require_abi163() {
    let package = compile_source(
        "skill jsrgzhaohan(3529273369) locked {
                trigger phase_prepare owner_is_target_draw_reshuffle_zero_wounded {
                    effect recover_hp(self, 1);
                }
                trigger phase_prepare owner_is_target_draw_reshuffle_positive {
                    effect lose_hp(self, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_163
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_529_273_369
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_draw_reshuffle_zero_wounded"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_529_273_369
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_draw_reshuffle_positive"
            && !trigger.optional
    }));
}

#[test]
fn turn_over_cancel_requires_abi164() {
    let package = compile_source(
        "skill kamome_suitcase(3677089645) locked {
                modifier skip_judge_phase override 1;
                modifier skip_discard_phase override 1;
                modifier turn_over_cancel override 1;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_164
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == 3_677_089_645
            && modifier.target == "turn_over_cancel"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    assert!(compile_source("skill bad(1) { modifier turn_over_cancel add 1; }").is_err());
}

#[test]
fn source_alive_once_filter_requires_abi165() {
    let package = compile_source(
        "skill dclonggong(3209573631) {
                trigger damage_before owner_is_target_source_alive_once optional {
                    effect prevent_current_damage();
                    effect gain_equipment_from_draw_pile(event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_165
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_209_573_631
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target_source_alive_once"
            && trigger.optional
    }));
}

#[test]
fn distance_conditional_target_cancel_requires_abi166() {
    let package = compile_source(
            "skill nsyuanchou(2147508736) locked {
                modifier card_target_cancel_if_other_user_distance_gt_one(ordinary_trick) override 1;
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_166
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == 2_147_508_736
            && modifier.target == "card_target_cancel_if_other_user_distance_gt_one"
            && modifier.selector.as_deref() == Some("ordinary_trick")
            && modifier.mode == "override"
            && modifier.value == 1
    }));
}

#[test]
fn duel_opponent_response_count_requires_abi167() {
    let package = compile_source(
        "skill wushuang(18) locked {
                modifier response_count add 1;
                modifier duel_opponent_response_count add 1;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_167
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == 18
            && modifier.target == "duel_opponent_response_count"
            && modifier.mode == "add"
            && modifier.value == 1
    }));
}

#[test]
fn target_suit_guess_hand_gain_requires_abi168_and_exact_active_contract() {
    let package = compile_source(
            "skill fanjian(36) {
                active play_phase uses_per_turn 1 hand_nonempty targets 1 1 other {
                    effect target_choose_suit_gain_owner_hand_damage_if_mismatch(self, event_source);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_168
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) {
                active play_phase targets 1 1 other {
                    effect target_choose_suit_gain_owner_hand_damage_if_mismatch(self, event_source);
                }
            }"
        )
        .is_err());
}

#[test]
fn gainable_source_card_once_per_damage_event_requires_abi169() {
    let package = compile_source(
        "skill fankui(5) {
                trigger damage owner_is_target_source_alive_gainable_card optional {
                    if self == event_source {
                        effect take_one_card(event_source, self, equip);
                    } else {
                        effect take_one_card(event_source, self, hand_equip);
                    }
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_169
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_source_alive_gainable_card"
    );
}

#[test]
fn damage_card_in_processing_filter_requires_abi170() {
    let package = compile_source(
        "skill jianxiong(1) {
                trigger damage owner_is_target_card_in_processing optional {
                    effect move_card(event_card, zone(self, 3), zone(0, 6));
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_170
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_card_in_processing"
    );
}

#[test]
fn ganglie_judgment_discard_or_damage_requires_abi171() {
    let package = compile_source(
        "skill ganglie(7) {
                trigger damage owner_is_target optional {
                    effect judge_source_discard_two_hand_or_damage(self, event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_171
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE,
                ),
            ]
    }));
}

#[test]
fn rende_selected_gift_phase_threshold_requires_abi172() {
    let package = compile_source(
        "skill rende(11) {
                active play_phase targets 1 1 other select_hand 1 255 {
                    effect give_selected_recover_after_phase_total_two(self, event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_172
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO,
                ),
            ]
    }));
    assert!(compile_source(
        "skill bad(1) {
                active play_phase targets 1 1 other select_hand 1 1 {
                    effect give_selected_recover_after_phase_total_two(self, event_source);
                }
            }"
    )
    .is_err());
}

#[test]
fn tuxi_draw_replacement_requires_abi173() {
    let package = compile_source(
        "skill tuxi(15) {
                trigger phase_draw owner_is_target_other_hand_nonempty optional {
                    effect choose_other_hand_targets_gain_one_replace_draw(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_173
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW,
                ),
            ]
    }));
}

#[test]
fn yiji_top_card_distribution_requires_abi174() {
    let package = compile_source(
        "skill yiji(26) {
                trigger damage owner_is_target_per_damage_point optional {
                    effect distribute_top_two_cards(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_174
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::DISTRIBUTE_TOP_TWO_CARDS),
            ]
    }));
}

#[test]
fn xiaoji_per_equipment_card_filter_requires_abi175() {
    let package = compile_source(
        "skill xiaoji(22) {
                trigger cards_moved owner_from_equip_per_card optional {
                    effect draw_cards(self, 2);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_175
    );
}

#[test]
fn jijiu_outside_turn_red_view_as_requires_abi176() {
    let package = compile_source(
        "skill jijiu(2961041949) {
                view_as tao response owner_outside_turn_red from hand_equip;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_176
    );
    assert_eq!(package.view_as_index[0].filter, "owner_outside_turn_red");
    assert_eq!(
        package.view_as_index[0].source_zone.as_deref(),
        Some("hand_equip")
    );
}

#[test]
fn tieji_red_judgment_unrespondable_requires_abi177() {
    let package = compile_source(
        "skill tieji(3561072419) {
                trigger card_targeted owner_source_sha optional {
                    effect judge_make_current_sha_target_unrespondable_on_red(
                        self,
                        event_card,
                        event_player
                    );
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_177
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::JUDGE_MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE_ON_RED),
                ]
        }));
}

#[test]
fn stdxiaoguo_two_stage_cost_requires_abi178() {
    let package = compile_source(
        "skill stdxiaoguo(3757481350) {
                trigger phase_end owner_alive_other_phase_end_has_discardable_basic_hand {
                    effect choose_discard_basic_then_target_discard_equip_or_damage(
                        self,
                        event_player
                    );
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_178
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_BASIC_THEN_TARGET_DISCARD_EQUIP_OR_DAMAGE),
                ]
        }));
}

#[test]
fn rewangzun_prepare_hp_and_lord_filters_require_abi179() {
    let package = compile_source(
        "skill rewangzun(3817957374) {
                trigger phase_prepare owner_alive_prepare_player_hp_gt_owner {
                    effect draw_cards(self, 1);
                }
                trigger phase_prepare owner_alive_prepare_lord_hp_gt_owner {
                    effect draw_cards(self, 1);
                    effect add_skill(event_player, 2737456606);
                    effect add_mark(event_player, 2737456606, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_179
    );
    assert_eq!(package.trigger_index.len(), 2);
}

#[test]
fn retongji_target_discard_redirect_requires_abi180() {
    let package = compile_source(
        "skill retongji(2773091493) {
                trigger card_targeted owner_alive_other_sha_target_can_redirect_to_owner {
                    effect choose_current_sha_target_discard_redirect_to_owner(
                        self,
                        event_source,
                        event_card,
                        event_player
                    );
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_180
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_SHA_TARGET_DISCARD_REDIRECT_TO_OWNER),
                ]
        }));
}

#[test]
fn jijie_and_jiyuan_require_abi181() {
    let package = compile_source(
        "skill xinfu_jijie(4028496128) {
                active play_phase uses_per_turn 1 {
                    effect choose_target_gain_bottom_card(self);
                }
            }
            skill xinfu_jiyuan(2449404379) {
                trigger cards_given owner_alive_gave_cards_to_other_alive optional {
                    effect draw_cards(event_player, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_181
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_TARGET_GAIN_BOTTOM_CARD),
            ]
    }));
    assert_eq!(package.trigger_index[0].event, "cards_given");
}

#[test]
fn luoshen_black_judgment_repeat_requires_abi182() {
    let package = compile_source(
        "skill luoshen(3972660146) {
                trigger phase_prepare owner_is_target optional {
                    effect judge_black_gain_repeat(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_182
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT),
            ]
    }));
}

#[test]
fn lijian_virtual_duel_requires_abi183() {
    let package = compile_source(
        "skill lijian(2292478631) {
                active play_phase uses_per_turn 1
                    targets 2 2 other male second_can_use_juedou_on_first
                    select_he 1 1 {
                        effect discard_selected(self);
                        effect use_virtual_juedou_second_on_first();
                    }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_183
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::USE_VIRTUAL_JUEDOU_SECOND_ON_FIRST,
                ),
            ]
    }));
    assert!(
        package.active_index[0]
            .targets
            .as_ref()
            .unwrap()
            .second_can_use_juedou_on_first
    );
}

#[test]
fn group_provided_response_and_sha_use_require_their_additive_abis() {
    let package = compile_source(
        "skill hujia(3353250727) lord_only {
                view_as shan response any from group_wei;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_184
    );
    let view = &package.view_as_index[0];
    assert_eq!(view.as_kind, "shan");
    assert_eq!(view.source_zone.as_deref(), Some("group_wei"));
    let jijiang = compile_source(
        "skill jijiang(2377283906) lord_only {
                view_as sha any any from group_shu;
            }",
    );
    assert_eq!(
        jijiang.unwrap().manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_187
    );
    assert!(compile_source("skill bad(1) { view_as sha response any from group_wei; }").is_err());
}

#[test]
fn attributed_recovery_and_tao_group_filter_require_abi185() {
    let package = compile_source(
        "skill jiuyuan(3165045964) locked lord_only {
                trigger card_responded owner_dying_other_tao_responder_group_wu {
                    effect recover_hp_as(event_source, self, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_185
    );
    let trigger = &package.trigger_index[0];
    assert_eq!(trigger.filter, "owner_dying_other_tao_responder_group_wu");
    assert!(package.skill_index[0].locked);
    assert!(package.skill_index[0].lord_only);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RECOVER_HP_AS),
            ]
    }));
}

#[test]
fn current_sha_discard_redirect_requires_abi186() {
    let package = compile_source(
            "skill liuli(3733125857) {
                trigger card_targeted owner_sha_target_can_discard_redirect_to_other_in_attack_range {
                    effect choose_discard_redirect_current_sha(self, event_source, event_card);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_186
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_sha_target_can_discard_redirect_to_other_in_attack_range"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_REDIRECT_CURRENT_SHA,
                ),
            ]
    }));
}

#[test]
fn judgment_rank_sha_policies_require_abi188() {
    let package = compile_source(
        "skill qiangwu(2563176834) {
                active play_phase uses_per_turn 1 {
                    effect judge_set_skill_mark_to_rank(self, self_skill);
                }
                modifier sha_distance_unlimited_if_rank_lt_skill_mark override 1;
                modifier sha_no_use_count_if_rank_gt_skill_mark override 1;
                trigger phase_play_end owner_is_target {
                    effect remove_mark(self, 2563176834, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_188
    );
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::JUDGE_SET_SKILL_MARK_TO_RANK),
                ]
        })
    }));
    assert_eq!(package.modifier_index.len(), 2);
    assert!(compile_source(
        "skill bad(1) {
                modifier sha_no_use_count_if_rank_gt_skill_mark add 1;
            }"
    )
    .is_err());
}

#[test]
fn linked_seat_order_discard_requires_abi189() {
    let package = compile_source(
        "skill fenming(4273591874) {
                trigger phase_end owner_is_target_linked optional {
                    effect choose_discard_one_from_each_linked(self, self_skill);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_189
    );
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(
                        rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_ONE_FROM_EACH_LINKED,
                    ),
                ]
        })
    }));
    assert_eq!(package.trigger_index[0].filter, "owner_is_target_linked");
}

#[test]
fn recanshi_query_and_filters_require_abi190() {
    let package = compile_source(
        "skill recanshi(2466532461) {
                trigger phase_draw owner_is_target_alive_wounded_or_guiming_wu_exists optional {
                    effect increase_current_draw_count(
                        self,
                        alive_wounded_or_guiming_wu_count(self)
                    );
                    effect add_skill(self, 2932254333);
                }
            }
            skill recanshi2(2932254333) locked {
                trigger card_used owner_source_sha_or_ordinary_trick_has_discardable_hand_equip {
                    effect discard_one_hand_or_equip(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_190
    );
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_QUERY,
                    rule_vm::bytecode::query_tag::ALIVE_WOUNDED_OR_GUIMING_WU_COUNT,
                ]
        })
    }));
}

#[test]
fn temporary_skill_disable_requires_abi191() {
    let package = compile_source(
        "skill rejiuchi(3056787053) {
                view_as jiu play_phase spade;
                view_as jiu response spade;
                trigger damage owner_source_jiu_boosted_sha_damage_skill_3365741831_enabled {
                    effect disable_skill_by_def_until_turn_end(self, 3365741831);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_191
    );
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(
                        rule_vm::bytecode::effect_tag::DISABLE_SKILL_BY_DEF_UNTIL_TURN_END,
                    ),
                ]
        })
    }));
}

#[test]
fn target_self_discard_conditional_duel_requires_abi192() {
    let package = compile_source(
        "skill juesi(2412286994) {
                active play_phase
                    targets 1 1 other in_attack_range hand_equip_nonempty
                    select_hand 1 1 card_sha {
                        effect discard_selected(self);
                        effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(
                            self,
                            event_source
                        );
                    }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_192
    );
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert!(targets.in_attack_range && targets.hand_equip_nonempty);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::TARGET_DISCARD_THEN_DUEL_IF_NON_SHA_AND_HP_GTE_OWNER),
                ]
        }));
    assert!(compile_source(
        "skill bad(1) {
                active play_phase targets 1 1 other {
                    effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(
                        self,
                        event_source
                    );
                }
            }"
    )
    .is_err());
}

#[test]
fn choose_target_lose_hp_discard_hand_sha_if_basic_requires_abi194() {
    let package = compile_source(
        "skill stdbaobian(3406574661) {
                trigger phase_play owner_is_target_any_alive_self_discardable_hand optional {
                    effect choose_target_lose_hp_discard_hand_sha_if_basic(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_194
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_any_alive_self_discardable_hand"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_TARGET_LOSE_HP_DISCARD_HAND_SHA_IF_BASIC,
                ),
            ]
    }));
}

#[test]
fn choose_optional_wounded_targets_draw_requires_abi195() {
    let package = compile_source(
        "skill lizhan(2790547004) {
                trigger phase_end owner_is_target_any_alive_wounded {
                    effect choose_optional_targets_draw(self, 1, 1, 255, wounded);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_195
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
        [EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count: 1,
            min_count: 1,
            max_count: 255,
            scope: rule_api::DrawTargetScope::Wounded,
            ..
        }] if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect choose_optional_targets_draw(self, 1, 2, 1, wounded); } }"
        )
        .is_err());
}

#[test]
fn weikui_branch_and_bound_distance_require_abi196() {
    let package = compile_source(
        "skill weikui(3890756007) {
                active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty {
                    effect reveal_target_hand_sha_or_discard_bind(self, event_source, 2980539775);
                }
            }
            skill weikui2(2980539775) {
                modifier distance_from_skill_bound_target override 1;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_196
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 8,
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
        [EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target: branch_target,
            helper_skill_def,
            ..
        }] if player.raw() == 9
            && branch_target.raw() == 10
            && helper_skill_def.raw() == 2_980_539_775
    ));
    assert!(
        compile_source("skill bad(1) { modifier distance_from_skill_bound_target add 1; }")
            .is_err()
    );
}

#[test]
fn remembered_hand_card_effects_require_abi197() {
    let package = compile_source(
        "skill kangli(2295716313) {
                trigger damage_before owner_is_source {
                    effect discard_remembered_hand_cards(self);
                }
                trigger damage owner_is_source_or_target {
                    effect draw_remembered_hand_cards(self, 2);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_197
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 8,
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    for module in &package.modules {
        RuleVm::new()
            .execute(&vmod(module), &mut host, 100)
            .unwrap();
    }
    assert!(host.effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::DrawRememberedHandCards {
            player,
            count: 2,
            ..
        } if player.raw() == 9
    )));
    assert!(host.effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::DiscardRememberedHandCards { player, .. }
            if player.raw() == 9
    )));
}

#[test]
fn lifetime_card_reward_groups_require_abi198() {
    let package = compile_source(
        "skill qinzheng(3099323686) locked {
                trigger card_used owner_lifetime_card_action {
                    effect gain_draw_cards_for_lifetime_card_action_divisors(
                        self, 3, sha_or_shan, 5, tao_or_jiu, 8, juedou_or_wuzhong
                    );
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_198
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
        [EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player,
            divisors: [3, 5, 8],
            groups: [
                rule_api::lifetime_card_reward_group::SHA_OR_SHAN,
                rule_api::lifetime_card_reward_group::TAO_OR_JIU,
                rule_api::lifetime_card_reward_group::JUE_DOU_OR_WU_ZHONG,
            ],
        }] if player.raw() == 9
    ));
}

#[test]
fn outside_turn_group_shan_filter_requires_abi199() {
    let package = compile_source(
            "skill rehujia(3691667990) lord_only {
                view_as shan response any from group_wei;
                trigger card_used owner_alive_other_shan_outside_turn_group_wei_once_per_turn optional by event_player {
                    effect draw_cards(self, 1);
                }
                trigger card_responded owner_alive_other_shan_outside_turn_group_wei_once_per_turn optional by event_player {
                    effect draw_cards(self, 1);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_199
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().all(|trigger| {
        trigger.filter == "owner_alive_other_shan_outside_turn_group_wei_once_per_turn"
            && trigger.optional
            && trigger.choice_actor == content_schema::TriggerChoiceActor::EventPlayer
    }));
    assert!(compile_source(
        "skill bad(1) {
                trigger card_used owner_alive_other_shan_outside_turn_group_Wei_once_per_turn {}
            }"
    )
    .is_err());
}

#[test]
fn opening_extra_play_hp_loss_gain_requires_abi200() {
    let package = compile_source(
        r#"
skill xindangxian(3456939183) locked {
  trigger phase_begin owner_is_target {
    effect queue_opening_extra_play_phase(self);
  }
  trigger phase_play owner_is_target {
    effect lose_hp_gain_discard_card_for_opening_extra_play(self, sha, 3062292376);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_200
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[1]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind: 1,
            mandatory_skill_def,
            skill,
        }] if player.raw() == 9
            && mandatory_skill_def.raw() == 3062292376
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_play owner_is_target { effect lose_hp_gain_discard_card_for_opening_extra_play(self, tao, 0); } }"
        )
        .is_err());
}

#[test]
fn own_turn_lost_card_reclaim_requires_abi201() {
    let package = compile_source(
        r#"
skill stdshangjian(4047696367) {
  trigger phase_end owner_is_target_lost_cards_at_most_hp_gainable optional {
    effect choose_gain_own_lost_discard_card_this_turn(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_201
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_lost_cards_at_most_hp_gainable"
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
        [EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, skill }]
            if player.raw() == 9 && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect choose_gain_own_lost_discard_card_this_turn(self, 1); } }"
        )
        .is_err());
}

#[test]
fn fixed_destination_field_move_requires_abi202() {
    let package = compile_source(
        r#"
skill stdzuici(2244504256) {
  trigger damage owner_is_target_source_movable_field_card optional {
    effect choose_move_field_card_to(self, event_source);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_202
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_source_movable_field_card"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 12,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseMoveFieldCardTo {
            player,
            target,
            skill,
        }] if player.raw() == 9 && target.raw() == 12 && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_move_field_card_to(self); } }"
        )
        .is_err());
}

#[test]
fn optional_hand_distribution_requires_abi203() {
    let package = compile_source(
        r#"
skill new_reyiji(3740567047) {
  trigger damage owner_is_target_per_damage_point optional {
    effect draw_then_optionally_distribute_hand_cards(self, 2, 2);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_203
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
        [EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count: 2,
            give_cap: 2,
            cards,
            targets,
            skill,
            ready_to_choose: false,
            finish: false,
        }] if player.raw() == 9 && cards.is_empty() && targets.is_empty() && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect draw_then_optionally_distribute_hand_cards(self, 2); } }"
        )
        .is_err());
}

#[test]
fn distinct_phase_gift_requires_abi204() {
    let package = compile_source(
        r#"
skill junkrende(4276125264) {
  active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 255 {
    effect give_selected_then_continue_distinct_recover_at_two(self, event_source);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_204
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 other select_hand 1 255 { effect give_selected_then_continue_distinct_recover_at_two(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn max_mark_death_judgment_requires_abi205() {
    let package = compile_source(
        r#"
skill new_wuhun(3278077669) locked {
  trigger death owner_is_target {
    effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(self, 3278077669);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_205
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
        [EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            skill,
        }] if player.raw() == 9 && mark.raw() == 3_278_077_669 && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger death owner_is_target { effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(self); } }"
        )
        .is_err());
}

#[test]
fn zone_priority_random_gain_requires_abi206() {
    let package = compile_source(
        r#"
skill new_guixin(3519522126) {
  trigger damage owner_is_target_per_damage_point_other_hand_equip_judge_nonempty optional {
    effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_206
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
        [EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            skill,
        }] if player.raw() == 9
            && *priority == u8::MAX
            && remaining_targets.is_empty()
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(); } }"
        )
        .is_err());
}

#[test]
fn linked_mark_capped_targets_and_extra_resolvers_require_abi208() {
    let package = compile_source(
        r#"
skill cuike(2516554182) {
  trigger phase_play owner_is_target optional {
    effect resolve_mark_parity_control(self, 1681567279, 7, 1);
  }
}
skill dinghuo(3907113145) {
  active play_phase once_per_game mark_at_least 1681567279 1
    targets 1 255 linked max_by_owner_mark 1681567279 {
    effect resolve_selected_equipment_discard_then_choose_damage(self, 1, fire);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_208
    );
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert!(targets.linked);
    assert_eq!(targets.max_by_owner_mark, Some(1_681_567_279));
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_MARK_PARITY_CONTROL),
                ]
        })
    }));
    assert!(package.modules.iter().any(|module| {
            module.code.windows(2).any(|bytes| {
                bytes
                    == [
                        rule_vm::bytecode::op::HOST_EFFECT,
                        legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE),
                    ]
            })
        }));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 linked unlinked { effect draw_cards(self, 1); } }"
        )
        .is_err());
}

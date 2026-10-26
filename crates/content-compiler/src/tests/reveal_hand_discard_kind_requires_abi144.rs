use super::*;

#[test]
fn reveal_hand_discard_kind_requires_abi144() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdtaoluan(1) { trigger phase_end owner_alive_not_target_hand_equip_nonempty optional { effect draw_then_give_one(self, event_player, 0, any_hand_equip); effect reveal_hand_discard_kind(event_player, shan); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_144
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_not_target_hand_equip_nonempty"
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_player: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DrawThenGiveOne {
                player,
                target,
                count: 0,
                drawn_only: false,
                ..
            },
            EffectRequest::RevealHandDiscardKind {
                player: reveal_player,
                kind,
            }
        ] if player.raw() == 7
            && target.raw() == 9
            && reveal_player.raw() == 9
            && *kind == 3
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_alive_not_target_hand_equip_nonempty { effect reveal_hand_discard_kind(self); } }"
        )
        .is_err());
}

#[test]
fn stdcanshi_declarative_surfaces_require_abi145() {
    let package = compile_source(
            "skill stdcanshi(1) locked { trigger phase_draw owner_is_target { effect add_skill(self, 2); } } \
             skill stdcanshi_effect(2) locked { \
               trigger card_targeted owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip { effect discard_one_hand_or_equip(self); } \
               trigger turn_ended owner_is_target { effect remove_skill(self_skill); } \
               modifier draw_count override alive_wounded_or_guiming_wu_count_min_one; \
             }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_145
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter
            == "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.value_source.as_deref() == Some("alive_wounded_or_guiming_wu_count_min_one")
            && modifier.mode == "override"
            && modifier.target == "draw_count"
    }));
}

#[test]
fn olqieting_second_turn_history_branch_requires_abi146() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill olqieting(1) { trigger turn_ended owner_alive_other_turn_ended_no_card_targeted_other { effect choose_move_target_equipment_and_or_draw(self, event_player); } trigger turn_ended owner_alive_other_turn_ended_card_targeted_other_no_damage_other { effect choose_move_target_equipment_or_draw(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_146
    );
    assert_eq!(package.trigger_index.len(), 2);
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_player: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
            player,
            target,
            skill,
        }] if player.raw() == 7
            && target.raw() == 9
            && skill.raw() == 0
    ));
}

#[test]
fn olqingxi_direct_sha_choice_requires_abi147() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill olqingxi(1) { trigger damage_before owner_source_sha_direct_damage_before optional { effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_147
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_direct_damage_before"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_player: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id: 0,
            skill,
        }] if player.raw() == 7
            && target.raw() == 9
            && skill.raw() == 0
    ));
}

#[test]
fn draw_or_recast_helper_choice_requires_abi148() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill oljiangchi(1) { trigger phase_draw_end owner_is_target { effect choose_draw_or_recast_add_skill(self, 2, 3); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_148
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            skill,
        }] if player.raw() == 7
            && draw_skill_def.raw() == 2
            && recast_skill_def.raw() == 3
            && skill.raw() == 0
    ));
}

#[test]
fn opening_extra_play_sha_gain_requires_abi149() {
    let package = compile_source(
        r#"
skill oldangxian(4090178594) locked {
  trigger phase_begin owner_is_target {
    effect queue_opening_extra_play_phase(self);
  }
  trigger phase_play owner_is_target {
    effect choose_gain_sha_for_opening_extra_play(self);
  }
  trigger phase_play_end owner_is_target {
    effect finish_opening_extra_play_sha_gain(self);
  }
  modifier card_distance_unlimited_for_skill_card_policy override 1;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_149
    );
    assert_eq!(package.modules.len(), 3);
    assert_eq!(
        package.modifier_index[0].target,
        "card_distance_unlimited_for_skill_card_policy"
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
        [EffectRequest::ChooseGainShaForOpeningExtraPlay { player, .. }]
            if player.raw() == 9
    ));
    host.effects.clear();
    RuleVm::new()
        .execute(&vmod(&package.modules[2]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::FinishOpeningExtraPlayShaGain { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
        "skill bad(1) { modifier card_distance_unlimited_for_skill_card_policy add 1; }"
    )
    .is_err());
}

#[test]
fn hand_nonempty_reveal_gain_requires_abi150() {
    let package = compile_source(
        r#"
skill zj_cuijian(3620039376) {
  active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty {
    effect reveal_target_hand_gain_shan_armor_or_draw_two(self, event_source);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_150
    );
    assert!(
        package.active_index[0]
            .targets
            .as_ref()
            .unwrap()
            .hand_nonempty
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
        [EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
            player,
            target,
            ..
        }] if player.raw() == 9 && target.raw() == 7
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 other { effect reveal_target_hand_gain_shan_armor_or_draw_two(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn cunsi_yongjue_surfaces_require_abi151() {
    let package = compile_source(
            r#"
skill cunsi(3184006504) {
  active play_phase once_per_game hand_nonempty face_up targets 1 1 other male {
    effect give_all_hand(self, event_source);
    effect add_skill(event_source, 3901337090);
    effect turn_over(self);
  }
}
skill yongjue(3901337090) {
  trigger sha_resolved owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn optional {
    effect gain_card_if_discard(self, event_card);
  }
}
"#,
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_151
    );
    let active = &package.active_index[0];
    assert!(active.once_per_game && active.hand_nonempty && active.face_up);
    assert!(active.targets.as_ref().is_some_and(|targets| {
        !targets.allow_self && targets.male && targets.min == 1 && targets.max == 1
    }));

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.first(),
        Some(EffectRequest::GiveAllHand { player, target })
            if player.raw() == 9 && target.raw() == 7
    ));
    host.effects.clear();
    RuleVm::new()
        .execute(&vmod(&package.modules[1]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::GainCardIfDiscard { player, card }]
            if player.raw() == 9 && card.raw() == 77
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase once_per_game targets 1 1 other male { effect give_all_hand(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn moukui_surfaces_require_abi152() {
    let package = compile_source(
        r#"
skill moukui(2368472328) {
  trigger card_targeted owner_source_sha {
    effect choose_draw_or_discard_target_card_record_sha(self, event_card, event_player);
  }
  trigger sha_dodged owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip {
    effect choose_discard_target_card(event_player, self, hand_equip);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_152
    );
    assert_eq!(
        package.trigger_index[1].filter,
        "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip"
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            ..
        }] if player.raw() == 9 && card.raw() == 77 && target.raw() == 7
    ));
}

#[test]
fn directed_hand_reveal_and_optional_target_discard_require_abi153() {
    let package = compile_source(
            "skill zyshangyi(2778115845) { active play_phase uses_per_turn 1 targets 1 1 other { effect reveal_hand_to_player(self, event_source); effect choose_optional_discard_target_card(self, event_source, hand); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_153
    );
    for tag in [
        rule_vm::bytecode::effect_tag::REVEAL_HAND_TO_PLAYER,
        rule_vm::bytecode::effect_tag::CHOOSE_OPTIONAL_DISCARD_TARGET_CARD,
    ] {
        assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes == [rule_vm::bytecode::op::HOST_EFFECT, legacy_effect_tag(tag)]
        }));
    }
    assert_eq!(
            compile_source(
                "skill forced(2) { active play_phase uses_per_turn 1 targets 1 1 other { effect choose_discard_target_card(self, event_source, hand); } }"
            )
            .unwrap()
            .manifest
            .rule_vm_abi_version,
            ADDITIVE_RULE_VM_ABI_VERSION_153
        );
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 targets 1 1 other { effect choose_optional_discard_target_card(self, event_source, judge); } }"
        )
        .is_err());
}

#[test]
fn reveal_hand_choose_discard_color_requires_abi154() {
    let package = compile_source(
            "skill nsjianglie(3311281717) { trigger card_targeted owner_source_sha_target_alive_hand_nonempty optional { effect reveal_hand_choose_discard_color(event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_154
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_target_alive_hand_nonempty"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::REVEAL_HAND_CHOOSE_DISCARD_COLOR),
            ]
    }));

    let mut host = CollectingHost {
        ctx: HostContext {
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
        [EffectRequest::RevealHandChooseDiscardColor { player }]
            if player.raw() == 7
    ));
    assert!(compile_source(
            "skill bad(1) { trigger card_targeted owner_source_sha_target_alive_hand_nonempty { effect reveal_hand_choose_discard_color(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn target_discard_recover_same_category_requires_abi155() {
    let source = "skill yjxuepin(2926558107) { active play_phase uses_per_turn 1 targets 1 1 other in_attack_range has_discardable_hand_equip { effect lose_hp(self, 1); effect choose_discard_target_cards_recover_same_category(self, event_source, 2, 1); } }";
    let package = compile_source(source).unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_155
    );
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert!(!targets.allow_self);
    assert!(targets.in_attack_range);
    assert!(targets.has_discardable_hand_equip);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_TARGET_CARDS_RECOVER_SAME_CATEGORY),
                ]
        }));

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 3,
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
        [
            EffectRequest::DealDamage {
                target,
                amount: 1,
                is_lose_hp: true,
                ..
            },
            EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                player,
                target: discard_target,
                count: 2,
                amount: 1,
                ..
            }
        ] if target.raw() == 3 && player.raw() == 3 && discard_target.raw() == 7
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 other { effect choose_discard_target_cards_recover_same_category(self, event_source, 2, 1); } }"
        )
        .is_err());
}

#[test]
fn comparison_second_failure_seat_damage_requires_abi156() {
    let source = "skill ylygqiangxi(2647374576) { active play_phase targets 1 1 other can_compare unused_by_skill_this_turn select_hand 1 1 { effect choose_compare_selected_hand_card_second_failure_seat_damage(self, event_source, 1, 2, 1, 1); } }";
    let package = compile_source(source).unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_156
    );
    let active = &package.active_index[0];
    assert!(active.targets.as_ref().unwrap().can_compare);
    assert!(active.targets.as_ref().unwrap().unused_by_skill_this_turn);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD_SECOND_FAILURE_SEAT_DAMAGE),
                ]
        }));

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 3,
            event_source: 7,
            ..Default::default()
        },
        selected_cards: vec![11],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count: 1,
            failure_count: 2,
            lose_hp: 1,
            damage: 1,
            ..
        }] if player.raw() == 3 && target.raw() == 7 && card.raw() == 11
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 other can_compare select_hand 1 1 { effect choose_compare_selected_hand_card_second_failure_seat_damage(self, event_source, 1, 2, 1, 1); } }"
        )
        .is_err());
}

#[test]
fn selected_cards_matching_target_equipment_requires_abi157() {
    let source = "skill nslulve(2729205545) { active play_phase uses_per_turn 1 targets 1 1 other has_equip selected_cards_match_target_equip_count select_he 1 255 { effect discard_selected(self); effect deal_damage(self, event_source, 1); } }";
    let package = compile_source(source).unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_157
    );
    assert!(
        package.active_index[0]
            .targets
            .as_ref()
            .unwrap()
            .selected_cards_match_target_equip_count
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 3,
            event_source: 7,
            ..Default::default()
        },
        selected_cards: vec![11, 12],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DiscardCards { player, cards },
            EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                ..
            }
        ] if player.raw() == 3
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [11, 12]
            && source.raw() == 3
            && target.raw() == 7
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 2 has_equip selected_cards_match_target_equip_count select_he 1 255 { effect discard_selected(self); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 selected_cards_match_target_equip_count select_he 1 255 { effect discard_selected(self); } }"
        )
        .is_err());
}

#[test]
fn equipment_used_trigger_requires_abi158() {
    let package = compile_source(
            "skill tyzhuan(2515374814) { trigger card_used owner_alive_equip_used optional { effect draw_cards(self, 1); } active play_phase targets 1 1 other has_equip select_he 1 1 card_sha { effect discard_selected(self); effect take_one_card(event_source, self, equip); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_158
    );
    assert_eq!(package.trigger_index[0].filter, "owner_alive_equip_used");
    assert!(package.trigger_index[0].optional);
    let active = &package.active_index[0];
    assert!(active.targets.as_ref().unwrap().has_equip);
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_sha")
    );
}

#[test]
fn sha_discard_scoped_target_card_gain_mount_requires_abi34() {
    let package = compile_source(
            "skill reqiaomeng(1) { trigger damage owner_source_sha_target_alive_has_discardable_region_card optional { effect discard_target_card_gain_mount(self, event_player, hand_equip_judge); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_34
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_target_alive_has_discardable_region_card"
    );
    assert_eq!(
        package.modules[0].code,
        vec![
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EVENT_PLAYER,
            rule_vm::bytecode::op::CONST_I32,
            2,
            0,
            0,
            0,
            rule_vm::bytecode::op::HOST_EFFECT,
            legacy_effect_tag(rule_vm::bytecode::effect_tag::DISCARD_TARGET_CARD_GAIN_MOUNT),
            rule_vm::bytecode::op::HALT,
        ]
    );
}

#[test]
fn current_sha_target_unrespondable_requires_abi35() {
    let package = compile_source(
            "skill liegong(2510589015) { trigger card_targeted owner_source_sha_target_hand_gte_hp_or_lte_attack_range optional { effect make_current_sha_target_unrespondable(self, event_card, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_35
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_target_hand_gte_hp_or_lte_attack_range"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 10,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        }] if source.raw() == 9 && card.raw() == 77 && target.raw() == 10
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect make_current_sha_target_unrespondable(self, event_card, event_player); } }"
        )
        .is_err());
}

#[test]
fn sha_dodged_target_discard_requires_abi36() {
    let package = compile_source(
            "skill mengjin(2910124062) { trigger sha_dodged owner_source_sha_dodged_target_alive_has_discardable_hand_equip optional { effect choose_discard_target_card(self, event_player, hand_equip); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_36
    );
    assert_eq!(package.trigger_index[0].event, "sha_dodged");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_dodged_target_alive_has_discardable_hand_equip"
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
        [EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope: rule_api::TargetCardScope::HandEquip,
        }] if player.raw() == 9 && target.raw() == 10
    ));
}

#[test]
fn judge_prevent_damage_lose_max_hp_requires_abi37() {
    let package = compile_source(
            "skill oldqianxi(3282621718) { trigger damage_before owner_source_sha_target_alive_distance_le_one optional { effect judge_prevent_damage_lose_max_hp_on_nonheart(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_37
    );
    assert!(package.trigger_index[0].optional);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_target_alive_distance_le_one"
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
        [EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id: 0,
        }] if player.raw() == 9 && target.raw() == 10
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_source { effect judge_prevent_damage_lose_max_hp_on_nonheart(self, event_player); } }"
        )
        .is_err());
}

#[test]
fn top_card_suit_choice_and_max_hp_hand_contract_require_abi38() {
    let package = compile_source(
            "skill xinzhan(3275354148) { active play_phase uses_per_turn 1 hand_gt_max_hp { effect choose_gain_top_cards_by_suit(self, 3, heart); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_38
    );
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_gt_max_hp);
    assert!(!active.hand_gt_hp);
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
        [EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count: 3,
            suit: Suit::Heart,
        }] if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase hand_gt_max_hp { effect choose_gain_top_cards_by_suit(self, 0, none); } }"
        )
        .is_err());
}

#[test]
fn repeated_other_player_empty_hand_draw_requires_abi39() {
    let package = compile_source(
            "skill stdshushen(3504783479) { trigger hp_recovered owner_is_target { effect choose_other_draw_by_hand_count(self, 1, 2, event_amount); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_39
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 9,
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
        [EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count: 1,
            empty_count: 2,
            times: 2,
            ..
        }] if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger hp_recovered owner_is_target { effect choose_other_draw_by_hand_count(self, 1, 2); } }"
        )
        .is_err());
}

#[test]
fn discard_then_gain_equipment_slot_requires_abi40() {
    let package = compile_source(
            "skill duodao(4208671631) { trigger damage owner_is_target_sha_source_has_discardable_hand_equip { effect choose_discard_one_then_gain_target_equipment(self, event_source, weapon); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_40
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_sha_source_has_discardable_hand_equip"
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
        [EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
            player,
            target,
            slot: 1,
        }] if player.raw() == 9 && target.raw() == 10
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect choose_discard_one_then_gain_target_equipment(self, event_source, trick); } }"
        )
        .is_err());
}

#[test]
fn play_phase_active_count_and_typed_target_draw_require_abi41() {
    let package = compile_source(
        r#"
skill yanyu(2263835600) {
  active play_phase select_hand 1 1 card_sha {
    effect recast_selected(self);
  }
  trigger phase_play_end owner_is_target_active_used_at_least_2_this_play_phase optional {
    effect choose_target_draw(self, 2, male_other);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_41
    );
    assert_eq!(
        package.active_index[0]
            .selection
            .as_ref()
            .and_then(|selection| selection.filter.as_deref()),
        Some("card_sha")
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_active_used_at_least_2_this_play_phase"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    let trigger_module = package
        .modules
        .iter()
        .find(|module| module.name == package.trigger_index[0].module)
        .unwrap();
    RuleVm::new()
        .execute(&vmod(trigger_module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseTargetDraw {
            player,
            count: 2,
            scope: rule_api::DrawTargetScope::MaleOther,
            ..
        }] if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_play_end owner_is_target_active_used_at_least_0_this_play_phase { effect choose_target_draw(self, 2, male_other); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_play_end owner_is_target { effect choose_target_draw(self, 2, stranger); } }"
        )
        .is_err());
}

#[test]
fn bounded_card_histories_and_dynamic_draw_require_abi42() {
    let package = compile_source(
        r#"
skill rejingce(2158663152) {
  trigger phase_play_end owner_is_target_play_phase_card_category_count_positive optional {
    effect draw_by_play_phase_card_category_count(self);
  }
  modifier hand_limit add owner_distinct_hand_card_suits_used_this_turn;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_42
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_play_phase_card_category_count_positive"
    );
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("owner_distinct_hand_card_suits_used_this_turn")
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    let module = package
        .modules
        .iter()
        .find(|module| module.name == package.trigger_index[0].module)
        .unwrap();
    RuleVm::new().execute(&vmod(module), &mut host, 100).unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
        "skill bad(1) { modifier attack_range add owner_distinct_hand_card_suits_used_this_turn; }"
    )
    .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_play_end owner_is_target { effect draw_by_play_phase_card_category_count(self, 1); } }"
        )
        .is_err());
}

#[test]
fn global_play_phase_category_filter_requires_abi293() {
    let package = compile_source(
        r#"
skill oljingce(3351317413) {
  trigger phase_play_end owner_alive_play_phase_card_category_count_positive optional {
    effect draw_by_play_phase_card_category_count(self);
  }
  modifier hand_limit add owner_distinct_hand_card_suits_used_this_turn;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_293
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_play_phase_card_category_count_positive"
    );
}

#[test]
fn longyin_abi43_golden_and_relongyin_requires_abi295() {
    let longyin = compile_source(
        r#"
skill longyin(3578226944) {
  trigger card_used owner_alive_play_phase_sha_user_has_discardable_hand_equip optional {
    effect choose_discard_one_for_sha_refund_and_draw_if_red(self, event_source, event_card);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        longyin.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_43
    );
    assert_eq!(
        longyin.trigger_index[0].filter,
        "owner_alive_play_phase_sha_user_has_discardable_hand_equip"
    );
    assert_eq!(longyin.modules[0].code, [83, 85, 86, 80, 61, 1]);
    assert_eq!(
        longyin.modules[0].logic_hash,
        [
            23, 183, 110, 115, 100, 48, 169, 6, 243, 112, 120, 102, 154, 3, 105, 63, 222, 225, 72,
            185, 146, 62, 190, 239, 208, 78, 135, 229, 116, 236, 163, 70,
        ]
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&longyin.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
                player,
                sha_user,
                card,
                ..
            }
        ] if player.raw() == 9
            && sha_user.raw() == 7
            && card.raw() == 77
    ));

    let relongyin = compile_source(
        r#"
skill relongyin(2474771514) {
  trigger card_used owner_alive_play_phase_sha_user_has_discardable_hand_equip optional {
    effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(
      self, event_source, event_card, 2811249808
    );
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        relongyin.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_295
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            event_card: 77,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&relongyin.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            ..
        }] if player.raw() == 9
            && sha_user.raw() == 7
            && card.raw() == 77
            && restore_skill_def.raw() == 2_811_249_808
    ));
    assert!(compile_source(
            "skill bad(1) { trigger card_used owner_alive_card_is_sha { effect choose_discard_one_for_sha_refund_and_draw_if_red(self, event_source, 1); } }"
        )
        .is_err());
}

#[test]
fn opening_extra_play_phase_requires_abi44() {
    let package = compile_source(
        r#"
skill dangxian(2393638375) locked {
  trigger phase_begin owner_is_target {
    effect queue_opening_extra_play_phase(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_44
    );
    assert!(package.skill_index[0].locked);
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
        [EffectRequest::QueueOpeningExtraPlayPhase { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_begin owner_is_target { effect queue_opening_extra_play_phase(self, 1); } }"
        )
        .is_err());
}

#[test]
fn end_extra_draw_or_play_requires_abi232() {
    let package = compile_source(
        r#"
skill decadejingce(2705593131) {
  trigger phase_end owner_is_target_card_used_at_least_hp optional {
    effect choose_end_extra_draw_or_play_by_used_suits(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_232
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
        [EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, .. }]
            if player.raw() == 9
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect choose_end_extra_draw_or_play_by_used_suits(self, 1); } }"
        )
        .is_err());
}

#[test]
fn damage_rewrite_requires_abi233() {
    let package = compile_source(
        r#"
skill rewrite(1) {
  trigger damage_before owner_source_other_damage_before optional {
    effect lose_hp_double_current_damage_then_replace(self, 3481582878);
  }
}
skill replacement(3481582878) locked {
  modifier damage_as_hp_loss override 1;
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_233
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            skill_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id: 0,
            amount: 0,
            skill,
            replacement_skill_def,
            ready_to_replace: false,
        }] if player.raw() == 9
            && skill.raw() == 0
            && replacement_skill_def.raw() == 3_481_582_878
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage_before owner_source_other_damage_before { effect lose_hp_double_current_damage_then_replace(self); } }"
        )
        .is_err());
}

#[test]
fn current_sha_fire_conversion_and_extra_target_require_abi45() {
    let package = compile_source(
        r#"
skill lihuo(3289572540) {
  trigger card_used owner_source_normal_sha optional {
    effect convert_current_sha_to_fire_lose_hp_if_damage(self, event_card);
    effect choose_add_current_sha_target(self, event_card);
  }
  trigger card_used owner_source_fire_sha {
    effect choose_add_current_sha_target(self, event_card);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_45
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.trigger_index[0].filter, "owner_source_normal_sha");
    assert_eq!(package.trigger_index[1].filter, "owner_source_fire_sha");

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_card: 77,
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
            EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. },
            EffectRequest::ChooseAddCurrentShaTarget {
                player: second_player,
                card: second_card,
                ..
            }
        ] if player.raw() == 9
            && card.raw() == 77
            && second_player.raw() == 9
            && second_card.raw() == 77
    ));
    assert!(compile_source(
            "skill bad(1) { trigger card_used owner_source_normal_sha { effect convert_current_sha_to_fire_lose_hp_if_damage(self); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger card_used owner_source_fire_sha { effect choose_add_current_sha_target(self, 1); } }"
        )
        .is_err());
}

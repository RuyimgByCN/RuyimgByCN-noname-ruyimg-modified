use super::*;

#[test]
fn twqingkou_requires_abi396() {
    let package = compile_source(
        "skill twqingkou(2492727906) {
                trigger duel_resolved owner_alive_tracked_twqingkou_duel_resolved {
                    effect resolve_twqingkou(self, event_player, event_card, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_396
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            event_card: 5,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action: 1,
            skill,
        }] if player.raw() == 9 && winner.raw() == 7 && card.raw() == 5
            && skill.is_none()
    ));
}

#[test]
fn twchenxun_requires_abi397() {
    let package = compile_source(
        "skill twchenxun(4060298774) {
                trigger duel_resolved owner_alive_tracked_twchenxun_duel_target_damaged {
                    effect resolve_twchenxun(self, event_source, event_card, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_397
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            event_card: 5,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action: 1,
            skill,
        }] if player.raw() == 9 && target.raw() == 7 && card.raw() == 5
            && skill.is_none()
    ));
}

#[test]
fn equipment_discard_prevent_damage_requires_abi398() {
    let package = compile_source(
            "skill xinjiaojin(2479658413) {
                trigger damage_before owner_is_target_male_source_has_discardable_equipment optional {
                    effect choose_discard_equipment_prevent_current_damage(self);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_398
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id: 0,
            skill,
        }] if player.raw() == 9 && skill.is_none()
    ));
}

#[test]
fn twfenxian_requires_abi399() {
    let package = compile_source(
        "skill twfenxian(3149854807) {
                active play_phase uses_per_turn 1 targets 1 1 {
                    effect resolve_twfenxian(self, event_source, event_card, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_399
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            event_card: 5,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action: 0,
            skill,
        }] if player.raw() == 9 && target.raw() == 7 && card.raw() == 5
            && skill.is_none()
    ));
}

#[test]
fn twxingluan_requires_abi491() {
    let package = compile_source(
        "skill twxingluan(3047451649) {
                trigger phase_end owner_is_target optional {
                    effect reveal_choose_category_distribute_self_threshold_hp_loss(self, 6, 3);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_491
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count: 6,
            recipient_cap: 3,
            skill,
        }] if player.raw() == 9 && skill.raw() == 0
    ));
}

#[test]
fn nsdingzhou_requires_abi492() {
    let package = compile_source(
        "skill nsdingzhou(2793593832) {
                active play_phase uses_per_turn 1 {
                    effect choose_other_random_gain_region_then_draw_compare_color_lose_hp(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_492
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
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
            player,
            skill,
        }] if player.raw() == 9 && skill.raw() == 0
    ));
}

#[test]
fn gzjinfa_requires_abi495() {
    let package = compile_source(
            "skill gzjinfa(2401974486) {
                active play_phase uses_per_turn 1 targets 1 1 other hand_equip_nonempty select_he 1 1 {
                    effect resolve_gzjinfa(self, event_source);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_495
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 10,
            skill_id: 7,
            ..Default::default()
        },
        selected_cards: vec![11],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            stage: 0,
            skill,
            ..
        }] if player.raw() == 9 && target.raw() == 10 && cost_card.raw() == 11 && skill.raw() == 0
    ));
}

#[test]
fn wn_qifeng_requires_abi496() {
    let package = compile_source(
            "skill wn_qifeng(3930492591) {
                trigger phase_end owner_alive_global_phase_end_lost_cards_target_other_alive optional {
                    effect resolve_wn_qifeng(self, event_player);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_496
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
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveWnQifeng {
            player,
            target,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && skill.raw() == 0
    ));
}

#[test]
fn yuanlve_requires_abi497() {
    let package = compile_source(
        "skill yuanlve(3073596171) {
                active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1 card_non_equip {
                    effect give_selected_then_target_may_use_draw_owner(self, event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_497
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 10,
            skill_id: 7,
            ..Default::default()
        },
        selected_cards: vec![11],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage: 0,
            skill,
        }] if player.raw() == 9
            && target.raw() == 10
            && card.raw() == 11
            && targets.is_empty()
            && skill.raw() == 0
    ));
}

#[test]
fn jytongbei_requires_abi498() {
    let package = compile_source(
            "skill jytongbei(2908493496) {
                trigger damage_before owner_source_target_not_group_wei optional {
                    effect choose_category_then_target_give_matching_or_damage_plus_one(self, event_player);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_498
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 10,
            skill_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id: 0,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && skill.raw() == 0
    ));
}

#[test]
fn jiebing_requires_abi499() {
    let package = compile_source(
            "skill jiebing(3832378886) locked {
                trigger damage owner_is_target {
                    effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(self, event_source);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_499
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 10,
            skill_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            skill,
            ..
        }] if player.raw() == 9 && source.raw() == 10 && skill.raw() == 0
    ));
}

#[test]
fn mobiledanshou_requires_abi500() {
    let package = compile_source(
        "skill mobiledanshou(2790027559) {
                trigger phase_end owner_alive_other_phase_end_mobiledanshou {
                    effect resolve_mobiledanshou(self, event_player);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_500
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 10,
            skill_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveMobiledanshou {
            player,
            target,
            skill,
        }] if player.raw() == 9 && target.raw() == 10 && skill.raw() == 0
    ));
}

#[test]
fn jsrgchiying_requires_abi501() {
    let package = compile_source(
        "skill jsrgchiying(2375612422) {
                active play_phase uses_per_turn 1 targets 1 1 {
                    effect resolve_jsrgchiying(self, event_source);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_501
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 10,
            skill_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&package.modules[0], &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveJsrgchiying {
            player,
            target,
            skill,
            ..
        }] if player.raw() == 9 && target.raw() == 10 && skill.raw() == 0
    ));
}

#[test]
fn first_hand_card_suit_use_unlimited_requires_abi507() {
    let package = compile_source(
        "skill mbxianshuai(3336756615) locked {
                modifier first_hand_card_suit_use_unlimited override 1;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_507
    );
}

#[test]
fn bingshi_tianfeng_effects_require_abi508() {
    let package = compile_source(
        "skill mbganggeng(2184967024) {
                active play_phase uses_per_turn 1 targets 1 1 other select_hand 2 255 {
                    effect resolve_mbganggeng(self, event_source, 0);
                }
            }
            skill mbsijian(2168498663) {
                trigger dying owner_from_hand_empty_or_dying_mbsijian_twice_per_turn {
                    effect resolve_mbsijian(self, 2654683932);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_508
    );
    assert_eq!(package.modules.len(), 2);
}

#[test]
fn bingshi_yanghong_effects_require_abi509() {
    let package = compile_source(
            "skill mbjianji(3226006705) {
                active play_phase uses_per_turn 1 targets 2 2 select_hand 1 1 {
                    effect resolve_mbjianji(self);
                }
            }
            skill mbyuanmo(4112615395) {
                trigger phase_prepare owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn optional {
                    effect resolve_mbyuanmo(self);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_509
    );
    assert_eq!(package.modules.len(), 2);
}

#[test]
fn bingshi_zhangyan_effects_require_abi510() {
    let package = compile_source(
            "skill mbfeijing(2916762074) {
                view_as sha play_phase damage_trick from hand_equip_expansion;
                trigger card_targeted owner_source_first_single_target_sha_mbfeijing_twice_per_turn optional {
                    effect resolve_mbfeijing(self, event_player, event_card);
                }
            }
            skill mbxiaoge(3431894905) locked {
                trigger damage_before owner_source_damage_before_mbfeijing_extra {
                    effect resolve_mbxiaoge(self, event_player, event_card, 0);
                }
                trigger card_resolved owner_source_sha_resolved_original_single_mbxiaoge {
                    effect resolve_mbxiaoge(self, event_player, event_card, 1);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_510
    );
    assert_eq!(package.modules.len(), 3);
    assert_eq!(package.view_as_index[0].filter, "damage_trick");
}

#[test]
fn bingshi_pangxi_contract_and_effects_require_abi514() {
    let package = compile_source(
            "skill mbxuye(4127729199) { trigger damage owner_alive_damage_target_min_hand_mbxuye_once_per_turn optional { effect resolve_mbxuye(self, event_player); } }\n\
             skill mbkuangxiang(3758195005) { active play_phase uses_per_turn 1 targets 1 1 other hand_lte_owner { effect resolve_mbkuangxiang(self, event_source, 0); } trigger cards_moved owner_mbkuangxiang_received_cards_exhausted optional { effect resolve_mbkuangxiang(self, event_player, 1); } trigger phase_play owner_phase_play_mbkuangxiang_active { effect resolve_mbkuangxiang(self, self, 2); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 514);
    assert!(
        package.active_index[0]
            .targets
            .as_ref()
            .unwrap()
            .hand_lte_owner
    );
}

#[test]
fn bingshi_pot_chengpu_effects_require_abi515() {
    let package = compile_source(
            "skill potduzuo(3906103687) { trigger cards_moved owner_gained_cards_potduzuo optional { effect resolve_potchengpu(self, event_player, 0); } }\n\
             skill potbihan(4060934534) { trigger damage_before owner_sha_damage_before_potbihan_available optional { effect resolve_potchengpu(self, event_player, 2); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 515);
}

#[test]
fn bingshi_pot_zanghong_contracts_require_abi516() {
    let package = compile_source(
            "skill pot_liezhi(3956687454) { view_as tao any any from none cost lose_max_hp_1 uses_per_turn 1; view_as jiu any any from none cost lose_max_hp_1 uses_per_turn 1; }\n\
             skill pot_jugu(4277593099) { trigger card_targeted owner_alive_card_targeted_healthy optional { effect resolve_potjugu(self); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 516);
    assert_eq!(package.view_as_index.len(), 2);
}

#[test]
fn bingshi_pot_caozhen_effects_require_abi517() {
    let package = compile_source(
            "skill potsifeng(3809208030) { trigger phase_end owner_is_target optional { effect resolve_potsifeng(self, self, event_card, 0); } trigger card_resolved owner_alive_card_resolved_source_current_potsifeng_pile { effect resolve_potsifeng(self, event_source, event_card, 3); } trigger phase_end owner_alive_phase_end_player_potsifeng_pile { effect resolve_potsifeng(self, event_player, event_card, 4); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 517);
    assert_eq!(package.trigger_index.len(), 3);
}

#[test]
fn bingshi_pot_chenqun_effects_require_abi518() {
    let package = compile_source(
            "skill potfaen(4194455770) { trigger card_used owner_alive { effect resolve_potchenqun(self, event_source, 0); } }\n\
             skill potdingpin(2518587448) { trigger phase_end owner_alive_potdingpin_available_once_per_round optional { effect resolve_potchenqun(self, event_player, 1); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 518);
    assert_eq!(package.trigger_index.len(), 2);
}

#[test]
fn bingshi_pot_chendao_effects_require_abi519() {
    let package = compile_source(
            "skill potwanglie(3694841232) { modifier card_use_unrespondable_for_skill_card_policy override 1; modifier card_distance_unlimited_for_skill_card_policy override 1; trigger phase_play owner_is_target_hand_nonempty optional { effect resolve_potwanglie(self, event_card, 3694841233, 0); } trigger card_resolved owner_source_card_resolved_phase_policy { effect resolve_potwanglie(self, event_card, 3694841233, 1); } }\n\
             skill pothongyi(3733552053) locked { trigger game_started owner_alive { effect resolve_pothongyi(self, 0); } trigger damage owner_is_source_or_target { effect resolve_pothongyi(self, 1); } trigger phase_prepare owner_is_target_mark_positive_3733552053 { effect resolve_pothongyi(self, 2); } trigger phase_end owner_is_target { effect resolve_pothongyi(self, 3); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 519);
    assert_eq!(package.trigger_index.len(), 6);
}

#[test]
fn bingshi_pot_lougui_effects_require_abi520() {
    let package = compile_source(
            "skill potguansha(2542568623) { trigger phase_play_end owner_phase_play_end_potguansha_available_once_per_game optional { effect resolve_potguansha(self, 2542568624); } }\n\
             skill potjiyu(3103425312) locked { active play_phase uses_per_turn 1 select_hand 1 1 { effect resolve_potjiyu(self, 0); } trigger cards_moved owner_alive { effect resolve_potjiyu(self, 1); } trigger phase_play_end owner_is_target { effect resolve_potjiyu(self, 2); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 520);
    assert_eq!(package.trigger_index.len(), 3);
    assert_eq!(package.active_index.len(), 1);
}

#[test]
fn bingshi_sunshao_contracts_require_abi521() {
    let package = compile_source(
            "skill mbganjue(3789971232) { view_as sha play_phase any from equip uses_per_turn 1 ignore_distance no_use_count target_lacks_source_suit_unrespondable; }\n\
             skill mbzhuji(2468204018) { trigger phase_play_end owner_is_target_hand_nonempty optional { effect resolve_mbzhuji(self, 0); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 521);
    let view = &package.view_as_index[0];
    assert!(view.no_use_count);
    assert!(view.target_lacks_source_suit_unrespondable);
    assert_eq!(package.trigger_index.len(), 1);
}

#[test]
fn bingshi_pot_xiahouba_contracts_require_abi522() {
    let package = compile_source(
            "skill potlibing(2358132416) locked { modifier attack_range add owner_skill_state_u32; trigger card_used owner_is_source { effect resolve_potlibing(self, event_card, 1); } trigger card_resolved owner_is_source { effect resolve_potlibing(self, event_card, 0); } }\n\
             skill potpoxi(4156018278) locked { trigger cards_moved owner_discarded_sha_to_discard { effect resolve_potpoxi(self); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 522);
    assert_eq!(package.trigger_index.len(), 3);
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("owner_skill_state_u32")
    );
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.filter == "owner_discarded_sha_to_discard"));
}

#[test]
fn bingshi_pot_zhangren_contracts_require_abi525() {
    let package = compile_source(
            "skill potfuan(2955537231) { trigger phase_end owner_is_target optional { effect resolve_potfuan(self, event_card, 0, 2955537232, 2595028433); } trigger phase_end owner_alive_other_turn_discarded_recorded_potfuan_suit optional { effect resolve_potfuan(self, event_card, 2, 2955537232, 2595028433); } trigger sha_resolved owner_source_sha_resolved_potfuan_dealt_damage { effect resolve_potfuan(self, event_card, 3, 2955537232, 2595028433); } }\n\
             skill potfuan_range(2955537232) locked { modifier attack_range override 0; }\n\
             skill potyinxian(2595028433) locked { modifier attack_range add owner_skill_state_u32; trigger card_used owner_is_source { effect resolve_potyinxian(self, event_card, 0); } trigger card_resolved owner_is_source { effect resolve_potyinxian(self, event_card, 1); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 525);
    assert_eq!(package.trigger_index.len(), 5);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_alive_other_turn_discarded_recorded_potfuan_suit"
    }));
}

#[test]
fn bingshi_pot_huanjie_and_qice_require_abi529() {
    let package = compile_source(
            "skill qice(3567101194) { view_as ordinary_trick play_phase any from none uses_per_turn 1 cost all_hand; }\n\
             skill potgongmou(3894451488) { trigger phase_prepare owner_is_target optional { effect resolve_pothuanjie(self, event_source, 0, 3567101194, 2834927312); } }\n\
             skill potzhengshuo(2346184405) { active play_phase once_per_game { effect resolve_pothuanjie(self, self, 2, 3567101194, 2834927312); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 529);
    assert!(package
        .view_as_index
        .iter()
        .any(|view| view.cost.as_deref() == Some("all_hand")));
}

#[test]
fn bingshi_pot_sunchen_contracts_require_abi532() {
    let package = compile_source(
            "skill potnigu(3220807381) { active play_phase uses_per_turn 1 select_he 1 4 distinct_suits { effect discard_selected(self); effect resolve_potnigu(self, 0); } trigger damage_before owner_is_source { effect resolve_potnigu(self, 3); } trigger turn_ended owner_is_target { effect resolve_potnigu(self, 4); } }\n\
             skill potlulian(2734921313) locked { trigger card_used owner_is_source { effect resolve_potlulian(self, event_card, 0); } trigger card_resolved owner_is_source { effect resolve_potlulian(self, event_card, 1); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 532);
    assert!(package.active_index[0]
        .selection
        .as_ref()
        .is_some_and(|selection| selection.distinct_suits));
}

#[test]
fn bingshi_potdimeng_contract_requires_abi535() {
    let package = compile_source(
            "skill potdimeng(2916721654) { active play_phase uses_per_turn 1 targets 2 2 pair_hand_difference_lte_three { effect resolve_potdimeng(self); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 535);
    assert!(package.active_index[0]
        .targets
        .as_ref()
        .is_some_and(|targets| targets.pair_hand_difference_lte_three));
}

#[test]
fn bingshi_pothaoshi_contract_requires_abi536() {
    let package = compile_source(
            "skill pothaoshi(3050076876) { trigger phase_end owner_is_target { effect resolve_pothaoshi(self, 0); } trigger phase_begin owner_is_target { effect resolve_pothaoshi(self, 1); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 536);
    assert_eq!(package.trigger_index.len(), 2);
}

#[test]
fn bingshi_dongzhao_contracts_require_abi539() {
    let package = compile_source(
            "skill spmiaolve(2347949240) { trigger game_started owner_alive { effect resolve_spmiaolve(self, 0); } trigger damage owner_is_target optional { effect resolve_spmiaolve(self, 1); } }\n\
             skill spyingjia(3012325144) { trigger phase_begin owner_is_target { effect resolve_spyingjia(self, 0, 3612952432); } trigger card_used owner_is_source { effect resolve_spyingjia(self, 1, 3612952432); } trigger turn_ended owner_alive optional { effect resolve_spyingjia(self, 2, 3612952432); } }\n\
             skill spyingjia_draw(3612952432) locked { trigger phase_begin owner_is_target { effect resolve_spyingjia(self, 3, 3612952432); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 539);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 6);
}

#[test]
fn bingshi_dengai_contracts_require_abi543() {
    let package = compile_source(
            "skill pottuntian(2435155097) { trigger cards_moved owner_lost_non_damage_card_pottuntian_charge_not_full { effect resolve_pottuntian(self, 0, 2551307508); } trigger phase_begin owner_alive { effect resolve_pottuntian(self, 1, 2551307508); } active play_phase uses_per_turn 1 targets 1 8 max_by_owner_mark 2435155097 { effect resolve_pottuntian(self, 2, 2551307508); } }\n\
             skill potzaoxian(2551307508) locked {}\n\
             skill potjixi(4126984108) { trigger phase_begin owner_alive { effect resolve_potjixi(self, self, 0); } trigger card_targeted owner_is_source { effect resolve_potjixi(self, event_player, 1); } trigger turn_ended owner_alive optional { effect resolve_potjixi(self, event_player, 2); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 543);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 5);
}

#[test]
fn bingshi_chenjiao_contracts_require_abi544() {
    let package = compile_source(
            "skill potqingyan(4232147498) { view_as shan response any from hand up_to 5 cost potqingyan; view_as wuxie response any from hand up_to 5 cost potqingyan; }\n\
             skill potceduan(4257587443) { active play_phase uses_per_turn 1 targets 1 1 other owner_in_target_attack_range { effect resolve_potceduan(self, event_source, 0); } trigger sha_resolved owner_source_sha_resolved_tracked_potceduan_dealt_damage { effect resolve_potceduan(self, event_source, 1); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 544);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package.active_index[0]
        .targets
        .as_ref()
        .is_some_and(|targets| targets.owner_in_target_attack_range));
}

#[test]
fn bingshi_taishici_contracts_require_abi548() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_taishici.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 548);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.active_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 4);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_alive_sha_entered_discard_potzhanlie_available"
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_source_sha_resolved_tracked_potzhanlie_draw" }));
}

#[test]
fn bingshi_xinxianying_contracts_require_abi554() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_xinxianying.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 554);
    assert_eq!(package.skill_index.len(), 7);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.modifier_index.len(), 5);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused"
    );
}

#[test]
fn bingshi_weiyan_contracts_require_abi558() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_weiyan.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 558);
    assert_eq!(package.skill_index.len(), 7);
    assert_eq!(package.trigger_index.len(), 12);
    assert_eq!(package.modifier_index.len(), 5);
}

#[test]
fn bingshi_zhouyu_contracts_require_abi561() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_zhouyu.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 561);
    assert_eq!(package.skill_index.len(), 4);
    assert_eq!(package.trigger_index.len(), 10);
}

#[test]
fn bingshi_lvyi_contracts_require_abi564() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_lvyi.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 564);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 11);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_alive_other_responded_to_owner" }));
}

#[test]
fn bingshi_caoshuang_contracts_require_abi568() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_caoshuang.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 568);
    assert_eq!(package.skill_index.len(), 4);
    assert_eq!(package.trigger_index.len(), 13);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_used_potnizun_pile_card" }));
}

#[test]
fn bingshi_yuji_contracts_require_abi570() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/bingshi/pot_yuji.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 570);
    assert_eq!(package.skill_index.len(), 5);
    assert_eq!(package.trigger_index.len(), 9);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    let view_as = &package.view_as_index[0];
    assert_eq!(view_as.as_kind, "basic");
    assert_eq!(view_as.window, "any");
    assert_eq!(view_as.cost.as_deref(), Some("potdaozhuan"));
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| targets.match_selected_card_count));
}

#[test]
fn sp_rehengjiang_contracts_require_abi572() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/sp/rehengjiang.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 572);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.modifier_index.len(), 1);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_alive_turn_ended_rehengjiang_pending" }));
}

#[test]
fn xianding_dcsuishi_contract_requires_abi596() {
    let package = compile_source(include_str!(
        "../../../../content/original/ported/xianding/dcsuishi.skill"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 596);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_alive_event_player_same_group_hand_nonempty" }));
}

#[test]
fn xianding_dcctjiuxian_contract_requires_abi597() {
    let package = compile_source_named(
        "dcctjiuxian-test",
        include_str!("../../../../content/original/ported/xianding/dcctjiuxian.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 597);
    assert!(package
        .active_index
        .iter()
        .any(|active| active.skill_def_id == 3_598_325_442));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_598_325_442
            && trigger.event == "damage"
            && trigger.filter == "owner_tracked_juedou_original_target_damage_recoverable_in_range"
            && trigger.optional
    }));
}

#[test]
fn xianding_bianyue_contract_requires_abi598() {
    let package = compile_source_named(
        "bianyue-test",
        include_str!("../../../../content/original/ported/xianding/bianyue.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 598);
    assert_eq!(package.skill_index.len(), 2);
    assert!(package
        .active_index
        .iter()
        .any(|active| { active.skill_def_id == 2_914_157_062 && active.uses_per_turn.is_none() }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_514_743_897
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty"
            && trigger.optional
    }));
}

#[test]
fn xianding_zhupeilan_contract_requires_abi600() {
    let package = compile_source_named(
        "zhupeilan-test",
        include_str!("../../../../content/original/ported/xianding/zhupeilan.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 600);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 4);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_468_569_693
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_is_target_ordinary_trick_cilv_available"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_625_586_472
            && trigger.event == "dying"
            && trigger.filter == "owner_dying_zhupeilan_once_per_game"
            && trigger.optional
    }));
}

#[test]
fn xianding_dc_sb_zhangxiu_contract_requires_abi601() {
    let package = compile_source_named(
        "dc-sb-zhangxiu-test",
        include_str!("../../../../content/original/ported/xianding/dc_sb_zhangxiu.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 601);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_319_521_189
            && trigger.event == "phase_play"
            && trigger.filter == "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_398_354_936
            && trigger.event == "phase_end"
            && trigger.filter == "owner_phase_end_dcsbhaoyi_available"
            && trigger.optional
    }));
}

#[test]
fn xianding_dc_sb_zhouyu_contract_requires_abi602() {
    let package = compile_source_named(
        "dc-sb-zhouyu-test",
        include_str!("../../../../content/original/ported/xianding/dc_sb_zhouyu.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 602);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 5);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_240_860_827
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_fire_sha_or_huogong_dcsbronghuo"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 4_166_950_825
            && trigger.event == "card_resolved"
            && trigger.filter
                == "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn"
            && trigger.optional
    }));
}

#[test]
fn xianding_dc_sb_lusu_contract_requires_abi603() {
    let package = compile_source_named(
        "dc-sb-lusu-test",
        include_str!("../../../../content/original/ported/xianding/dc_sb_lusu.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 603);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 5);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 4_288_768_048
            && trigger.event == "phase_draw_end"
            && trigger.filter == "owner_is_target"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_188_556_454
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_hand_transfer_other_dcsbmengmou_branch_available"
            && trigger.optional
    }));
}

#[test]
fn xianding_zhangjian_contract_requires_abi604() {
    let package = compile_source_named(
        "zhangjian-test",
        include_str!("../../../../content/original/ported/xianding/zhangjian.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 604);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_792_442_500
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target_card_damage_before"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 3_520_470_433
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_phase_end_hand_equip_nonempty_other_alive"
            && trigger.optional
    }));
}

#[test]
fn xianding_zhugeruoxue_contract_requires_abi605() {
    let package = compile_source_named(
        "zhugeruoxue-test",
        include_str!("../../../../content/original/ported/xianding/zhugeruoxue.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 605);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert!(package.active_index.iter().any(|active| {
        active.skill_def_id == 2_641_115_727
            && active.window == "play_phase"
            && active.uses_per_turn == Some(1)
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 2_308_165_295
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target"
            && trigger.optional
    }));
}

#[test]
fn xianding_caoyi_contract_requires_abi606() {
    let package = compile_source_named(
        "caoyi-test",
        include_str!("../../../../content/original/ported/xianding/caoyi.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 606);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 5);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 4_165_081_339
            && trigger.event == "card_resolved"
            && trigger.filter
                == "owner_source_single_target_sha_or_trick_all_materials_from_hand_can_sha"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 4_165_081_339
            && trigger.event == "card_resolved"
            && trigger.filter == "dcyinjun_tracked_sha_resolved"
            && !trigger.optional
    }));
}

#[test]
fn xianding_malingli_contract_requires_abi607() {
    let xiaoyin = compile_source_named(
        "malingli-xiaoyin-test",
        include_str!("../../../../content/original/ported/xianding/dcxiaoyin.skill"),
    )
    .unwrap();
    assert_eq!(xiaoyin.manifest.rule_vm_abi_version, 607);
    assert_eq!(xiaoyin.skill_index.len(), 2);
    assert_eq!(xiaoyin.trigger_index.len(), 2);

    let huahuo = compile_source_named(
        "malingli-huahuo-test",
        include_str!("../../../../content/original/ported/xianding/dchuahuo.skill"),
    )
    .unwrap();
    assert_eq!(huahuo.manifest.rule_vm_abi_version, 607);
    assert_eq!(huahuo.view_as_index[0].source_zone, None);
    assert!(huahuo.view_as_index[0].no_use_count);
    assert_eq!(
        huahuo.trigger_index[0].filter,
        "owner_source_own_view_as_card_targeted_target_skill_pile_3398996454"
    );
}

#[test]
fn wandianlunsha_feishi_contract_requires_abi608() {
    let package = compile_source_named(
        "wandianlunsha-feishi-test",
        include_str!("../../../../content/original/ported/wandianlunsha/feishi.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 608);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 1);
    assert!(package.trigger_index[0].optional);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_recovered_outside_owner_turn"
    );
    let selectors: std::collections::BTreeSet<_> = package
        .modifier_index
        .iter()
        .filter_map(|modifier| modifier.selector.as_deref())
        .collect();
    assert_eq!(selectors, ["bingliang", "lebu"].into_iter().collect());
}

#[test]
fn wandianlunsha_hanlong_contract_requires_abi609() {
    let package = compile_source_named(
        "wandianlunsha-hanlong-test",
        include_str!("../../../../content/original/ported/wandianlunsha/hanlong.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 609);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 3);
    let filters: std::collections::BTreeSet<_> = package
        .trigger_index
        .iter()
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert!(filters.contains("owner_is_target_discarded_sha_in_discard_phase"));
    assert!(filters.contains("owner_source_sha_damage_before_target_healthy"));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.optional)
            .count(),
        1
    );
}

#[test]
fn wandianlunsha_liufu_contract_requires_abi610() {
    let package = compile_source_named(
        "wandianlunsha-liufu-test",
        include_str!("../../../../content/original/ported/wandianlunsha/liufu.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 610);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 3);
    let filters: std::collections::BTreeSet<_> = package
        .trigger_index
        .iter()
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert!(filters.contains("owner_phase_end_originating_skill_pile_empty"));
    assert!(filters.contains("owner_is_target_sha_skill_pile_nonempty_2400116759"));
    assert!(filters.contains("owner_alive_other_play_discard_skill_pile_nonempty_2400116759"));
    assert!(package.trigger_index.iter().all(|trigger| trigger.optional));
}

#[test]
fn wandianlunsha_liuyan_contract_requires_abi611() {
    let package = compile_source_named(
        "wandianlunsha-liuyan-test",
        include_str!("../../../../content/original/ported/wandianlunsha/liuyan.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 611);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.modifier_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_prepare_group_outside_attack_range_exists"
    );
    assert!(package.trigger_index[0].optional);
    assert!(package
        .modifier_index
        .iter()
        .all(|modifier| modifier.value_source.as_deref() == Some("owner_linked")));
}

#[test]
fn wandianlunsha_liuzan_contract_requires_abi612() {
    let package = compile_source_named(
        "wandianlunsha-liuzan-test",
        include_str!("../../../../content/original/ported/wandianlunsha/liuzan.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 612);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.modifier_index.len(), 2);
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().unwrap().has_discardable_hand_equip);
    let sources: std::collections::BTreeSet<_> = package
        .modifier_index
        .iter()
        .filter_map(|modifier| modifier.value_source.as_deref())
        .collect();
    assert_eq!(
        sources,
        std::collections::BTreeSet::from([
            "owner_skill_state_u32",
            "owner_skill_state_u32_negative"
        ])
    );
}

#[test]
fn wandianlunsha_tianyu_contract_requires_abi613() {
    let package = compile_source_named(
        "wandianlunsha-tianyu-test",
        include_str!("../../../../content/original/ported/wandianlunsha/tianyu.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 613);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.modifier_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_sha_target_distance_le_one_has_equip_card"
    );
    assert!(package.trigger_index[0].optional);
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(
                |modifier| modifier.value_source.as_deref() == Some("owner_equip_count_at_least_1")
            )
            .count(),
        2
    );
}

#[test]
fn wandianlunsha_xizhenxihong_contract_requires_abi614() {
    let package = compile_source_named(
        "wandianlunsha-xizhenxihong-test",
        include_str!("../../../../content/original/ported/wandianlunsha/xizhenxihong.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 614);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.modifier_index.len(), 2);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.target == "card_response_cancel"
            && modifier.selector.as_deref() == Some("black_trick")
            && modifier.value_source.as_deref() == Some("owner_hp_le_two")
    }));
}

#[test]
fn wandianlunsha_yangyi_contract_requires_abi615() {
    let package = compile_source_named(
        "wandianlunsha-yangyi-test",
        include_str!("../../../../content/original/ported/wandianlunsha/yangyi.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 615);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable"
    );
    assert!(package.trigger_index[0].optional);
}

#[test]
fn wandianlunsha_zaozhirenjun_contract_requires_abi616() {
    let package = compile_source_named(
        "wandianlunsha-zaozhirenjun-test",
        include_str!("../../../../content/original/ported/wandianlunsha/zaozhirenjun.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 616);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    assert_eq!(package.view_as_index[0].as_kind, "wugu");
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_alive_wugu_remained_in_discard" && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_is_target_non_multitarget_trick_multiple_targets"
            && trigger.optional
    }));
}

#[test]
fn ddd_handang_contract_requires_abi617() {
    let package = compile_source_named(
        "ddd-handang-test",
        include_str!("../../../../content/original/ported/ddd/handang.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 617);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_source_play_phase_sha_has_additional_target_ignoring_distance"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_source_card_resolved_tracked_dddxianxi" && !trigger.optional
    }));
}

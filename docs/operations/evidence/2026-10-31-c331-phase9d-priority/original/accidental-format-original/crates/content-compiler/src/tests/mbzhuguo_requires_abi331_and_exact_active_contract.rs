use super::*;

#[test]
fn mbzhuguo_requires_abi331_and_exact_active_contract() {
    let package = compile_source(
            "skill mbzhuguo(2303782932) { active play_phase uses_per_turn 1 targets 1 1 { effect resolve_mbzhuguo(self, event_source); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_331
    );
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max)),
        Some((1, 1))
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 11,
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
        [EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage: 0,
            skill,
        }] if player.raw() == 9 && target.raw() == 7 && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 { effect resolve_mbzhuguo(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn mbanxian_requires_abi332_and_first_hand_discard_contract() {
    let package = compile_source(
            "skill mbanxian(2708680147) { trigger cards_moved owner_first_hand_discard_has_sha_in_discard optional { effect resolve_mbanxian(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_332
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_first_hand_discard_has_sha_in_discard"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        event_cards: vec![101, 102],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage: 0,
            skill,
        }] if player.raw() == 9
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [101, 102]
            && card.is_none()
            && skill.is_none()
    ));
    assert!(compile_source(
        "skill bad(1) { trigger damage owner_is_target { effect resolve_mbanxian(self); } }"
    )
    .is_err());
}

#[test]
fn mbjinzu_requires_abi333_and_emits_active_and_next_sha_stages() {
    let package = compile_source(
            "skill mbjinzu(3257523591) { active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1 { effect resolve_mbjinzu(self, event_source, 4156343864); } } skill mbjinzu_effect(4156343864) locked { trigger card_used owner_source_sha { effect apply_mbjinzu_next_sha(self, event_card); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_333
    );
    let active = package
        .modules
        .iter()
        .find(|module| module.name == "mbjinzu_active_play_phase_0")
        .unwrap();
    let mut active_host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            ..Default::default()
        },
        selected_cards: vec![101],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(active), &mut active_host, 100)
        .unwrap();
    assert!(
        matches!(
            active_host.effects.as_slice(),
            [EffectRequest::ResolveMbjinzu {
                player,
                target,
                cards,
                card,
                helper_skill_def,
                stage: 0,
                skill,
            }] if player.raw() == 9
                && target.raw() == 7
                && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [101]
                && card.is_none()
                && helper_skill_def.raw() == 4_156_343_864
                && skill.is_none()
        ),
        "{:?}",
        active_host.effects
    );

    let trigger = package
        .modules
        .iter()
        .find(|module| module.name == "mbjinzu_effect_card_used")
        .unwrap();
    let mut trigger_host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_card: 202,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(trigger), &mut trigger_host, 100)
        .unwrap();
    assert!(matches!(
        trigger_host.effects.as_slice(),
        [EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage: 2,
            skill,
        }] if player.raw() == 9
            && target.is_none()
            && cards.is_empty()
            && card.raw() == 202
            && helper_skill_def.is_none()
            && skill.is_none()
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase uses_per_turn 1 targets 1 1 other { effect resolve_mbjinzu(self, event_source, 4156343864); } }"
        )
        .is_err());
}

#[test]
fn scoped_take_for_rezhiman_requires_abi334_and_precedes_damage_prevention() {
    let package = compile_source(
            "skill rezhiman(3521391741) { trigger damage_before owner_source_other_damage_before optional { effect take_one_card(event_player, self, hand_equip_judge); effect prevent_current_damage(); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_334
    );
    let module = package
        .modules
        .iter()
        .find(|module| module.name == "rezhiman_damage_before")
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::TakeOneScopedCard { from, to, scope },
            EffectRequest::PreventDamage { damage_id: 0 }
        ] if from.raw() == 7
            && to.raw() == 9
            && *scope == rule_api::TargetCardScope::HandEquipJudge
    ));
}

#[test]
fn sha_diamond_distance_selector_requires_abi335() {
    let package = compile_source(
        "skill new_rewusheng(3038912803) {
                view_as sha play_phase red from hand_equip_expansion;
                view_as sha response red from hand_equip_expansion;
                modifier card_distance_unlimited(sha_diamond) override 1;
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_335
    );
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == 3_038_912_803
            && modifier.target == "card_distance_unlimited"
            && modifier.selector.as_deref() == Some("sha_diamond")
    }));
}

#[test]
fn first_hp_change_unique_trick_majority_and_reveal_hand_require_abi336() {
    let package = compile_source(
        "skill clanbalong(2722267819) locked {
                trigger damage owner_is_target_first_hp_change_turn_unique_trick_majority {
                    effect reveal_hand(self);
                    effect draw_cards(self, alive_count() - hand_count(self));
                }
                trigger hp_lost owner_is_target_first_hp_change_turn_unique_trick_majority {
                    effect reveal_hand(self);
                }
                trigger hp_recovered owner_is_target_first_hp_change_turn_unique_trick_majority {
                    effect reveal_hand(self);
                }
                trigger max_hp_changed owner_is_target_first_hp_change_turn_unique_trick_majority {
                    effect reveal_hand(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_336
    );
    assert_eq!(package.trigger_index.len(), 4);
    assert!(package.trigger_index.iter().all(|trigger| {
        trigger.filter == "owner_is_target_first_hp_change_turn_unique_trick_majority"
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
            EffectRequest::RevealHand { player },
            EffectRequest::DrawCards {
                player: drawer,
                count: 0
            }
        ] if player.raw() == 9 && drawer.raw() == 9
    ));
}

#[test]
fn damage_prevented_trigger_requires_abi337() {
    let package = compile_source(
        "skill stdrenzheng(3020306305) locked {
                trigger damage_prevented owner_alive_event_player_alive {
                    effect draw_cards(event_player, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_337
    );
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(package.trigger_index[0].event, "damage_prevented");
}

#[test]
fn hand_lte_chooser_damage_scope_requires_abi338() {
    let package = compile_source(
        "skill oljuece(2978424418) {
                trigger phase_end owner_is_target_other_hand_lte_owner optional {
                    effect choose_targets_damage(self, 1, 1, hand_lte_chooser, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_338
    );
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_other_hand_lte_owner"
    );
}

#[test]
fn linked_player_draw_and_shared_turn_filter_require_abi344() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill jysuozhou(2628091225) {
                trigger card_used owner_source_or_targeted_club_linked_exists_once_per_turn optional {
                    effect draw_linked_players(1);
                }
                trigger card_targeted owner_source_or_targeted_club_linked_exists_once_per_turn optional {
                    effect draw_linked_players(1);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_344
    );
    assert_eq!(package.trigger_index.len(), 2);
    for module in &package.modules {
        let mut host = CollectingHost::default();
        RuleVm::new()
            .execute(&vmod(module), &mut host, 100)
            .unwrap();
        assert!(matches!(
            host.effects.as_slice(),
            [EffectRequest::DrawLinkedPlayers { count: 1 }]
        ));
    }
    assert!(compile_source(
        "skill bad(1) { trigger card_used always { effect draw_linked_players(); } }"
    )
    .is_err());
}

#[test]
fn nuzhan_and_danji_filters_require_abi345() {
    let package = compile_source(
        "skill nuzhan(2974830679) locked {
                trigger card_used owner_source_sha_single_trick_source_card {
                    effect refund_current_sha_use(self, event_card);
                }
                trigger card_used owner_source_sha_single_equip_source_card {
                    effect increase_current_card_base_damage(self, event_card, 1);
                }
            }
            skill danji(2150258599) locked {
                trigger phase_prepare owner_is_target_hand_above_hp_no_liubei_lord {
                    effect remove_skill(self_skill);
                    effect change_max_hp(self, -1);
                    effect add_skill(self, 2);
                    effect add_skill(self, 2974830679);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_345
    );
    assert_eq!(package.trigger_index.len(), 3);
}

#[test]
fn regongao_filter_requires_abi346() {
    let package = compile_source(
        "skill regongao(2805903172) locked {
                trigger dying owner_alive_other_dying_first_per_target {
                    effect change_max_hp(self, 1);
                    effect recover_hp(self, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_346
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_dying_first_per_target"
    );
}

#[test]
fn reluanwu_and_reweimu_keep_ordered_inherited_semantics_at_abi351() {
    use rule_api::EffectRequest;

    let package = compile_source(
        "skill reluanwu(2626196598) {
                active play_phase once_per_game {
                    effect force_others_use_sha_on_nearest_or_lose_hp(self);
                    effect choose_use_virtual_sha(self, ignore_distance);
                }
            }
            skill reweimu(2589015751) locked {
                modifier card_target_cancel(black_trick) override 1;
                trigger damage_before owner_is_target_current_turn {
                    effect prevent_current_damage();
                    effect draw_cards(self, event_amount * 2);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_351
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_current_turn"
    );
    let module = package
        .modules
        .iter()
        .find(|module| module.name.starts_with("reluanwu_active"))
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
        [
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { player, .. },
            EffectRequest::ChooseUseVirtualSha {
                player: second,
                ignore_distance: true,
            }
        ] if player.raw() == 9 && second.raw() == 9
    ));
}

#[test]
fn pszhonghun_reaches_typed_color_effect_at_abi352() {
    use rule_api::EffectRequest;

    let package = compile_source(
        "skill pszhonghun(4055252724) {
                trigger card_used owner_source_red optional {
                    effect reveal_top_gain_if_color(self, red);
                }
                trigger card_responded owner_source_red optional {
                    effect reveal_top_gain_if_color(self, red);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_352
    );
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package
        .trigger_index
        .iter()
        .all(|trigger| trigger.filter == "owner_source_red" && trigger.optional));
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
    assert_eq!(
        host.effects,
        [EffectRequest::RevealTopGainIfColor {
            player: rule_api::PlayerId::new(9),
            color_mask: 1,
        }]
    );
}

#[test]
fn zj_duanbing_requires_abi353_for_turn_pair_history_and_nearby_shan_count() {
    let package = compile_source(
            "skill zj_duanbing(2843523405) {
                trigger card_used owner_source_sha {
                    effect choose_add_current_sha_target_within_distance(self, event_card, 1);
                }
                trigger damage owner_source_target_distance_le_one_first_damage_to_target_this_turn {
                    effect draw_cards(self, 1);
                }
                modifier sha_response_count_if_source_owner_target_distance_le_one add 1;
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_353
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.filter == "owner_source_target_distance_le_one_first_damage_to_target_this_turn"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.target == "sha_response_count_if_source_owner_target_distance_le_one"
    }));
}

#[test]
fn spniluan_no_damage_refund_requires_abi354() {
    let package = compile_source(
            "skill spniluan(4063477590) {
                view_as sha play_phase black from hand_equip_expansion refund_use_count_if_no_damage;
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_354
    );
    let view = &package.view_as_index[0];
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip_expansion"));
    assert!(view.refund_use_count_if_no_damage);
}

#[test]
fn twjiaohua_requires_abi363_and_preserves_draw_event_cards() {
    let package = compile_source(
            "skill twjiaohua(3924813285) {
                trigger cards_drawn owner_alive_draw_player_owner_or_min_hp_twjiaohua_available optional {
                    effect resolve_twjiaohua(self, event_player);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_363
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            ..Default::default()
        },
        event_cards: vec![101, 102],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            skill,
        }] if player.raw() == 9
            && target.raw() == 7
            && event_cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [101, 102]
            && skill.is_none()
    ));
}

#[test]
fn twkujian_requires_abi364_and_preserves_selected_cards() {
    let package = compile_source(
        "skill twkujian(2903587563) {
                active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 3 {
                    effect resolve_twkujian(self, event_source, 0);
                }
                trigger cards_moved owner_alive_other_twkujian_lost {
                    effect resolve_twkujian(self, event_source, 2);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_364
    );
    let module = package
        .modules
        .iter()
        .find(|module| module.name.contains("active_play_phase"))
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            ..Default::default()
        },
        selected_cards: vec![101, 102, 103],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action: 0,
            skill,
        }] if player.raw() == 9
            && target.raw() == 7
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [101, 102, 103]
            && skill.is_none()
    ));
}

#[test]
fn twruilian_requires_abi365_and_preserves_discard_cards() {
    let package = compile_source(
        "skill twruilian(3419838857) {
                trigger cards_moved owner_alive_twruilian_selected_target_discarded {
                    effect resolve_twruilian(self, event_player, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_365
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 7,
            ..Default::default()
        },
        event_cards: vec![101, 102],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action: 1,
            category: 0,
            skill,
        }] if player.raw() == 9
            && target.raw() == 7
            && event_cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [101, 102]
            && skill.is_none()
    ));
}

#[test]
fn twfujian_requires_abi366() {
    let package = compile_source(
        "skill twfujian(2940701334) locked {
                trigger game_started owner_alive {
                    effect equip_weapon_from_draw_pile(self);
                }
                trigger cards_moved owner_alive_lost_weapon_outside_turn {
                    effect lose_hp(self, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_366
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
        [EffectRequest::EquipWeaponFromDrawPile { player }] if player.raw() == 9
    ));
}

#[test]
fn twjianwei_requires_abi367() {
    let package = compile_source(
        "skill twjianwei(3685532230) {
                trigger phase_prepare owner_alive_other_phase_prepare_can_compare_owner {
                    effect resolve_twjianwei(self, event_player, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_367
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
        [EffectRequest::ResolveTwjianwei {
            player,
            actor,
            stage: 1,
            skill,
            ..
        }] if player.raw() == 9 && actor.raw() == 7 && skill.is_none()
    ));
}

#[test]
fn twtanfeng_requires_abi368() {
    let package = compile_source(
        "skill twtanfeng(4187893298) {
                trigger phase_prepare owner_is_target {
                    effect resolve_twtanfeng(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_368
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
        [EffectRequest::ResolveTwtanfeng {
            player,
            stage: 0,
            skill,
            ..
        }] if player.raw() == 9 && skill.is_none()
    ));
}

#[test]
fn twzhibian_requires_abi369() {
    let package = compile_source(
        "skill twzhibian(4091292152) {
                trigger phase_play owner_is_target_can_compare_other {
                    effect resolve_twzhibian(self);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_369
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
        [EffectRequest::ResolveTwzhibian { player, stage: 0, skill, .. }]
            if player.raw() == 9 && skill.is_none()
    ));
}

#[test]
fn twyuyan_requires_abi370() {
    let package = compile_source(
        "skill twyuyan(3341156482) locked {
                trigger card_targeted owner_target_physical_sha_source_hp_gt_owner {
                    effect resolve_twyuyan(self, event_source, event_card);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_370
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
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(
        matches!(host.effects.as_slice(), [EffectRequest::ResolveTwyuyan { player, source, card, skill }] if player.raw() == 9 && source.raw() == 7 && card.raw() == 5 && skill.is_none())
    );
}

#[test]
fn twguoyi_requires_abi371() {
    let package = compile_source(
            "skill twguoyi(3002953620) {
                trigger card_targeted owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition optional {
                    effect resolve_twguoyi(self, event_player, event_card, 3469371128);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_371
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
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def: 3469371128,
            skill
        }] if player.raw() == 9 && target.raw() == 7 && card.raw() == 5 && skill.is_none()
    ));
}

#[test]
fn twchuhai_requires_abi372() {
    let package = compile_source(
        "skill twchuhai(3782702310) {
                trigger phase_end owner_alive_phase_end_caused_two_distinct_dying {
                    effect resolve_twchuhai(self, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_372
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
        [EffectRequest::ResolveTwchuhai {
            player,
            action: 0,
            stage: 0,
            index: 0,
            event_cards,
            skill
        }] if player.raw() == 9 && event_cards.is_empty() && skill.is_none()
    ));
}

#[test]
fn twyizhu_requires_abi373() {
    let package = compile_source(
        "skill twyizhu(2650180014) {
                trigger card_targeted owner_alive_other_single_target_used_tracked_twyizhu {
                    effect resolve_twyizhu(self, event_source, event_card, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_373
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
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action: 1,
            stage: 0,
            event_cards,
            skill
        }] if player.raw() == 9 && source.raw() == 7 && card.raw() == 5 && event_cards.is_empty() && skill.is_none()
    ));
}

#[test]
fn twluanchou_requires_abi374() {
    let package = compile_source(
        "skill twluanchou(2306392424) {
                active play_phase uses_per_turn 1 targets 2 2 {
                    effect resolve_twluanchou(self, 3503235693, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_374
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            ..Default::default()
        },
        selected_targets: vec![7, 8],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    let [EffectRequest::ResolveTwluanchou {
        player,
        helper_skill_def,
        action,
        targets,
        skill,
        ..
    }] = host.effects.as_slice()
    else {
        panic!("unexpected effects: {:?}", host.effects);
    };
    assert_eq!(player.raw(), 9);
    assert_eq!(*helper_skill_def, 3_503_235_693);
    assert_eq!(*action, 0);
    assert_eq!(
        targets
            .iter()
            .map(|target| target.raw())
            .collect::<Vec<_>>(),
        [7, 8]
    );
    assert!(skill.is_none());
}

#[test]
fn twxiawei_requires_abi375() {
    let package = compile_source(
        "skill twxiawei(3007668730) {
                trigger phase_prepare owner_is_target {
                    effect resolve_twxiawei(self, 3476660685, 2);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_375
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
        [EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def: 3476660685,
            action: 2,
            stage: 0,
            skill
        }] if player.raw() == 9 && skill.is_none()
    ));
}

#[test]
fn twbianfuren_requires_abi376() {
    let package = compile_source(
        "skill twwanwei(3733856593) {
                trigger damage_before owner_alive_damage_before_target_min_hp_unused {
                    effect resolve_twbianfuren(self, event_player, 3469371143, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_376
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
        [EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def: 3469371143,
            action: 0,
            stage: 0,
            cards,
            skill
        }] if player.raw() == 9 && target.raw() == 7 && cards.is_empty() && skill.is_none()
    ));
}

#[test]
fn twfeiyi_requires_abi377() {
    let package = compile_source(
        "skill twshengxi(3391342890) {
                trigger phase_prepare owner_is_target {
                    effect resolve_twfeiyi(self, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_377
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
        [EffectRequest::ResolveTwfeiyi {
            player,
            action: 0,
            stage: 0,
            card,
            event_cards,
            skill
        }] if player.raw() == 9 && card.is_none() && event_cards.is_empty() && skill.is_none()
    ));
}

#[test]
fn twchenzhen_requires_abi378() {
    let package = compile_source(
        "skill twchayi(3796108342) {
                trigger phase_end owner_is_target {
                    effect resolve_twchenzhen(self, event_player, 3469371153, 3469371154, 1);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_378
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
        [EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def: 3469371153,
            record_skill_def: 3469371154,
            action: 1,
            stage: 0,
            card_kind: 0,
            card,
            branch: 0,
            skill
        }] if player.raw() == 9 && target.raw() == 7 && card.is_none() && skill.is_none()
    ));
}

#[test]
fn twcaoxiu_requires_abi379() {
    let package = compile_source(
        "skill twqingxi(2467573737) {
                trigger card_targeted owner_source_first_sha_target_this_turn {
                    effect resolve_twqingxi(self, event_player, event_card);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_379
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
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage: 0,
            count: 0,
            cards,
            skill
        }] if player.raw() == 9 && target.raw() == 7 && card.raw() == 5
            && cards.is_empty() && skill.is_none()
    ));
}

#[test]
fn twniujin_requires_abi391() {
    let package = compile_source(
        "skill twcuorui(3369253433) {
                trigger phase_prepare owner_is_target_other_higher_hand_once_per_game optional {
                    effect resolve_twniujin(self, 3369253433, 0);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_391
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
        [EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action: 0,
            stage: 0,
            skill,
        }] if player.raw() == 9 && limited_skill_def.raw() == 3_369_253_433
            && skill.is_none()
    ));
}

#[test]
fn twzumao_requires_abi392() {
    let package = compile_source(
            "skill twtijin(3420559729) {
                trigger card_targeted owner_alive_other_source_single_sha_target_other_owner_in_attack_range optional {
                    effect resolve_twcaoang(self, event_player, event_source, event_card, 0, 0);
                }
                trigger sha_resolved owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip {
                    effect choose_discard_target_card(self, event_source, hand_equip);
                    effect resolve_twcaoang(self, self, event_source, event_card, 0, 2);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_392
    );
}

#[test]
fn twfuwan_requires_abi393() {
    let package = compile_source(
        "skill twmoukui(3462273565) {
                trigger card_targeted owner_source_sha {
                    effect resolve_twmoukui(self, event_player, event_card);
                }
            }",
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_393
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
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage: 0,
            skill,
        }] if player.raw() == 9 && target.raw() == 7 && card.raw() == 5
            && skill.is_none()
    ));
}

#[test]
fn twfuman_requires_abi394_and_preserves_selected_card() {
    let package = compile_source(
            "skill twfuman(2838266052) {
                modifier card_name(skill_tracked) override sha;
                active play_phase targets 1 1 other unused_by_skill_this_play_phase select_hand 1 1 {
                    effect resolve_twfuman(self, event_source, 0);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_394
    );
    let module = package
        .modules
        .iter()
        .find(|module| module.name.contains("active_play_phase"))
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_source: 7,
            ..Default::default()
        },
        selected_cards: vec![101],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(module), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action: 0,
            stage: 0,
            skill,
        }] if player.raw() == 9
            && target.raw() == 7
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [101]
            && skill.is_none()
    ));
}

#[test]
fn cuijin_requires_abi395() {
    let package = compile_source(
            "skill cuijin(2582559793) {
                trigger card_used owner_alive_sha_used_by_self_or_in_range_discardable_hand_equip optional {
                    effect resolve_cuijin(self, event_source, event_card, 0);
                }
            }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_395
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
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action: 0,
            skill,
        }] if player.raw() == 9 && source.raw() == 7 && card.raw() == 5
            && skill.is_none()
    ));
}

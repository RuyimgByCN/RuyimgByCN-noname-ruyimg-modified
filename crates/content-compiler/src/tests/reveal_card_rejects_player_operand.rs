use super::*;

#[test]
fn reveal_card_rejects_player_operand() {
    // reveal_card must take a card operand, not a player.
    let src = r#"
skill bad(12) {
  trigger card_used owner_is_source {
    effect reveal_card(self);
  }
}
"#;
    assert!(compile_source(src).is_err());
}

#[test]
fn loop_fuel_or_bound() {
    let src = r#"
skill looper(9) {
  trigger damage always {
    loop 100000 {
      effect emit_note(1);
    }
  }
}
"#;
    // Bounded loop of 100000 still needs fuel; compile ok, run may OOF
    let pkg = compile_source(src).unwrap();
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let err = vm.execute(&vmod(&pkg.modules[0]), &mut host, 100);
    assert!(err.is_err());
}

#[test]
fn rejects_unknown_effect() {
    let src = r#"
skill bad(1) {
  trigger damage always {
    effect not_a_real_effect(self, 1);
  }
}
"#;
    assert!(compile_source(src).is_err());
}

#[test]
fn compile_deal_damage_and_active() {
    let src = r#"
skill fankui(5) {
  trigger damage owner_is_target {
    effect take_hand(event_source, self);
  }
}
skill zhiheng(10) {
  active play_phase {
    effect discard_hand(self, 1);
    effect draw_cards(self, 1);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    assert_eq!(pkg.trigger_index.len(), 1);
    assert_eq!(pkg.active_index.len(), 1);
    assert_eq!(pkg.active_index[0].window, "play_phase");
    assert!(pkg.modules.len() >= 2);
}

#[test]
fn compile_other_source_sha_twice_filter() {
    let pkg = compile_source(
        r#"
skill vtbshuli(3163490927) {
  trigger damage owner_alive_other_source_sha_twice optional {
    effect draw_each_by_seat(self, event_source, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_alive_other_source_sha_twice"
    );
    assert!(pkg.trigger_index[0].optional);
}

#[test]
fn elemental_damage_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill scsniqu(2407745929) {
  active play_phase uses_per_turn 1 targets 1 1 {
    effect deal_damage(self, event_source, 1, fire);
  }
}
"#,
    )
    .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 4,
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
        [EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        }] if source.raw() == 4 && target.raw() == 7
    ));
    assert!(compile_source(
        "skill bad(1) { active play_phase { effect deal_damage(self, event_source, 1, poison); } }"
    )
    .is_err());
    assert_eq!(
            compile_source("skill old(1) { trigger phase_prepare owner_is_target { effect draw_cards(self, 1); } }")
                .unwrap()
                .manifest
                .rule_vm_abi_version,
            3
        );
}

#[test]
fn discard_one_hand_or_equip_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill dcyiju(3976872382) locked {
  trigger card_used owner_alive_targeted_by_other_single_card_discardable {
    effect discard_one_hand_or_equip(self);
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
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardOneHandOrEquip { player }] if player.raw() == 7
    ));
}

#[test]
fn discard_all_hand_and_equip_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill huilei(2889506960) locked {
  trigger death owner_is_target_source_alive {
    effect discard_all_hand_and_equip(event_source);
  }
}
"#,
    )
    .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            event_source: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::DiscardAllHandAndEquip { player }] if player.raw() == 9
    ));
}

#[test]
fn remove_all_skills_requires_abi159() {
    let package = compile_source(
        r#"
skill duanchang(2554836650) locked {
  trigger death owner_is_target_source_alive {
    effect remove_all_skills(event_source);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_159
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            event_source: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RemoveAllSkills { player }] if player.raw() == 9
    ));
}

#[test]
fn choose_other_recover_requires_abi160() {
    let package = compile_source(
        r#"
skill tyxianshou(4281024996) locked {
  trigger death owner_is_source {
    effect choose_other_recover(self, 2);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_160
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
        [EffectRequest::ChooseOtherRecover { player, amount }]
            if player.raw() == 9 && *amount == 2
    ));
}

#[test]
fn enable_equip_slot_requires_abi161() {
    let package = compile_source(
        r#"
skill mbkuili(2909034781) locked {
  trigger damage owner_is_target_source_alive_disabled_weapon {
    effect enable_equip_slot(event_source, 1, self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_161
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            event_source: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::EnableEquipSlot {
            player,
            slot: 1,
            source
        }] if player.raw() == 9 && source.raw() == 7
    ));
}

#[test]
fn judge_gain_equipment_by_color_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill biancai(2285861837) {
  trigger phase_begin owner_alive optional {
    effect judge_gain_equipment_by_color(self);
  }
}
"#,
    )
    .unwrap();
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
        [EffectRequest::JudgeGainEquipmentByColor { player }] if player.raw() == 9
    ));
}

#[test]
fn judge_recover_on_suit_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill fuzhao(3351225946) {
  trigger dying owner_alive optional {
    effect judge_recover_on_suit(event_player, heart, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 12);
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
        [EffectRequest::JudgeRecoverOnSuit {
            source,
            player,
            suit: rule_api::Suit::Heart,
            amount: 1,
        }] if source.raw() == 9 && player.raw() == 7
    ));
}

#[test]
fn judge_damage_other_on_suit_reaches_typed_effect_and_requires_abi15() {
    let package = compile_source(
        r#"
skill yinbao(3078155020) {
  trigger damage owner_is_target optional {
    effect judge_damage_other_on_suit(self, spade, 1, thunder);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 15);
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
        [EffectRequest::JudgeDamageOtherOnSuit {
            player,
            suit: rule_api::Suit::Spade,
            amount: 1,
            nature: rule_api::damage_nature::THUNDER,
        }] if player.raw() == 9
    ));
}

#[test]
fn judge_cancel_current_sha_target_on_red_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill olshenhou(2393323694) {
  trigger card_targeted owner_targeted_by_other_sha optional {
    effect judge_cancel_current_sha_target_on_red(self, event_card, event_player);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_targeted_by_other_sha"
    );
    assert!(package.trigger_index[0].optional);
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 9,
            event_player: 9,
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
        [EffectRequest::JudgeCancelCurrentShaTargetOnRed {
            player,
            card,
            target,
        }] if player.raw() == 9 && card.raw() == 77 && target.raw() == 9
    ));
    assert!(compile_source(
        r#"
skill bad(1) {
  trigger damage owner_is_target {
    effect judge_cancel_current_sha_target_on_red(self, event_card, event_player);
  }
}
"#,
    )
    .is_err());
}

#[test]
fn choose_draw_or_recover_and_sha_damage_filters_compile() {
    let package = compile_source(
        r#"
skill yaowu(2159889222) locked {
  trigger damage_before owner_is_target_red_sha_source_alive {
    effect choose_draw_or_recover(event_source, 1, 1, 0);
  }
  trigger damage_before owner_is_target_nonred_sha {
    effect draw_cards(self, 1);
  }
  trigger damage owner_is_target_red_or_jiu_sha {
    effect change_max_hp(self, -1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.trigger_index.len(), 3);
    let mut host = CollectingHost {
        ctx: HostContext {
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
        [EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count: 1,
            recover_count: 1,
            allow_cancel: false,
        }] if player.raw() == 7
    ));
}

#[test]
fn choose_use_virtual_sha_reaches_typed_host() {
    let package = compile_source(
        r#"
skill yingjian(3382160794) {
  trigger phase_prepare owner_is_target {
    effect choose_use_virtual_sha(self, ignore_distance);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 4);
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance: true,
        }] if player.raw() == 7
    ));
    assert!(compile_source(
        r#"
skill bad(1) {
  trigger phase_prepare owner_is_target {
    effect choose_use_virtual_sha(self, anywhere);
  }
}
"#,
    )
    .is_err());
}

#[test]
fn generic_card_damage_and_per_damage_filters_compile() {
    let package = compile_source(
        r#"
skill filters(1) locked {
  trigger damage_before owner_is_target_red_card_source_alive {
    effect draw_cards(event_source, 1);
  }
  trigger damage_before owner_is_target_nonred_card {
    effect draw_cards(self, 1);
  }
  trigger damage owner_is_target_first_damage {
    effect recover_hp(self, 1);
  }
  trigger damage owner_is_target_second_damage {
    effect lose_hp(self, 1);
  }
  trigger damage owner_is_target_other_source_alive_per_damage_point {
    effect discard_one_hand_or_equip(event_source);
  }
  trigger damage owner_is_target_after_first_damage {
    effect lose_hp(self, 1);
  }
  trigger damage owner_source_distance_le_one_per_damage_point {
    effect choose_draw_or_recover(self, 1, 1, 1);
  }
  trigger damage owner_source_target_alive_has_hand_or_equip {
    effect take_one_card(event_player, self, hand_equip);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        package
            .trigger_index
            .iter()
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_red_card_source_alive",
            "owner_is_target_nonred_card",
            "owner_is_target_first_damage",
            "owner_is_target_second_damage",
            "owner_is_target_other_source_alive_per_damage_point",
            "owner_is_target_after_first_damage",
            "owner_source_distance_le_one_per_damage_point",
            "owner_source_target_alive_has_hand_or_equip",
        ]
    );
}

#[test]
fn reveal_discard_suit_gain_or_use_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill chenjian(3452524123) {
  trigger phase_prepare owner_is_target optional {
    effect reveal_discard_suit_gain_or_use(self, 3452524123, 3, 2);
  }
}
"#,
    )
    .unwrap();
    assert!(package.trigger_index[0].optional);
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 6,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RevealDiscardSuitGainOrUse {
            player,
            mark,
            base_count: 3,
            max_mark: 2,
            revealed,
            completed: 0,
            step: 0,
            data: 0,
        }] if player.raw() == 6 && mark.raw() == 3_452_524_123 && revealed.is_empty()
    ));
}

#[test]
fn discard_or_fire_damage_skip_phases_reaches_typed_effect() {
    let package = compile_source(
        r#"
skill mbtanfeng(4191284417) {
  trigger phase_prepare owner_is_target {
    effect choose_discard_or_fire_damage_skip_phases(self, 2, 1);
  }
}
"#,
    )
    .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 4,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player,
            max_discard: 2,
            damage: 1,
            selected: 0,
            step: 0,
            target,
        }] if player.raw() == 4 && target.is_none()
    ));
}

#[test]
fn damage_event_amount_reaches_typed_effects() {
    let source = r#"
skill reflect(11) {
  trigger damage owner_is_target optional {
    effect deal_damage(self, event_source, event_amount);
    effect recover_hp(self, event_amount);
  }
}
"#;
    let package = compile_source(source).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_player: 2,
            event_source: 1,
            event_amount: 3,
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
                source,
                target,
                amount: 3,
                ..
            },
            EffectRequest::RecoverHp {
                source: recover_source,
                player,
                amount: 3,
            },
        ] if source.raw() == 2
            && target.raw() == 1
            && recover_source.raw() == 2
            && player.raw() == 2
    ));

    let outside_damage = source.replace("trigger damage", "trigger phase_end");
    assert!(compile_source(&outside_damage).is_err());
    let with_pause = source.replace(
        "effect deal_damage",
        "let choice = choose(0, 1); effect deal_damage",
    );
    assert!(compile_source(&with_pause).is_err());
}

#[test]
fn card_target_count_reaches_typed_effects() {
    let source = r#"
skill draw_targets(12) {
  trigger card_used owner_source_sha optional {
    effect draw_cards(self, event_target_count);
  }
}
"#;
    let package = compile_source(source).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_source: 2,
            event_amount: 3,
            ..Default::default()
        },
        event_targets: vec![4, 5, 6],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::DrawCards { player, count: 3 }] if player.raw() == 2
    ));

    assert!(compile_source(&source.replace("card_used", "damage")).is_err());
}

#[test]
fn turn_damaged_player_count_reaches_phase_end_effects() {
    let source = r#"
skill count_damage(13) locked {
  trigger phase_end owner_is_target {
    if turn_damaged_player_count < 4 {
      effect draw_cards(self, turn_damaged_player_count + 1);
    } else {
      effect draw_cards(self, 4);
    }
  }
}
"#;
    let package = compile_source(source).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_player: 2,
            event_amount: 3,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::DrawCards { player, count: 4 }] if player.raw() == 2
    ));

    assert!(compile_source(&source.replace("phase_end", "damage")).is_err());
}

#[test]
fn active_selection_reaches_vm_as_typed_effects() {
    let src = r#"
skill exchange(10) {
  active play_phase select_hand 1 4 {
    effect discard_selected(self);
    effect draw_selected(self);
  }
}
"#;
    let package = compile_source(src).expect("compile selected active");
    let selection = package.active_index[0]
        .selection
        .as_ref()
        .expect("selection metadata");
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand", 1, 4)
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..HostContext::default()
        },
        selected_cards: vec![41, 42],
        ..CollectingHost::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .expect("execute selected active");
    assert!(matches!(
        &host.effects[0],
        EffectRequest::DiscardCards { player, cards }
            if player.raw() == 7 && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == vec![41, 42]
    ));
    assert!(matches!(
        host.effects[1],
        EffectRequest::DrawCards { player, count } if player.raw() == 7 && count == 2
    ));
}

#[test]
fn active_equipment_filter_and_recast_require_abi16() {
    let package = compile_source(
        r#"
skill huairou(4137401010) {
  active play_phase select_he 1 1 card_equip {
    effect recast_selected(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 16);
    assert_eq!(
        package.active_index[0]
            .selection
            .as_ref()
            .and_then(|selection| selection.filter.as_deref()),
        Some("card_equip")
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..HostContext::default()
        },
        selected_cards: vec![41],
        ..CollectingHost::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::RecastCards { player, cards }]
            if player.raw() == 7 && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == vec![41]
    ));
}

#[test]
fn disable_all_equip_slots_requires_abi18() {
    let package = compile_source(
        r#"
skill qiongshou(2792246343) locked {
  trigger game_started owner_has_enabled_equip_slot {
    effect disable_all_equip_slots(self);
    effect draw_cards(self, 4);
  }
  modifier hand_limit add 4;
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 18);
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..HostContext::default()
        },
        ..CollectingHost::default()
    };
    RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DisableEquipSlots { player, slots },
            EffectRequest::DrawCards { player: draw_player, count: 4 }
        ] if player.raw() == 7 && slots == &[1, 2, 3, 4, 5] && draw_player.raw() == 7
    ));
}

#[test]
fn active_target_and_use_contract_reaches_package() {
    let package = compile_source(
        r#"
skill xiongsuan(3088500344) {
  active play_phase uses_per_turn 1 targets 1 1 select_hand 1 1 {
    effect discard_selected(self);
    effect deal_damage(self, event_source, 1);
    effect draw_cards(self, 3);
  }
}
"#,
    )
    .expect("compile constrained active");
    let active = &package.active_index[0];
    let targets = active.targets.as_ref().expect("target contract");
    assert_eq!((targets.min, targets.max), (1, 1));
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active.selection.as_ref().map(|spec| (spec.min, spec.max)),
        Some((1, 1))
    );

    assert!(compile_source(
        "skill bad(1) { active play_phase uses_per_turn 0 { effect draw_cards(self, 1); } }"
    )
    .is_err());
    assert!(compile_source(
        "skill bad(1) { active play_phase targets 2 1 { effect draw_cards(self, 1); } }"
    )
    .is_err());
}

#[test]
fn active_owner_missing_skill_contract_requires_abi234() {
    let package = compile_source(
        r#"
skill wuqian(2841820068) {
  active play_phase mark_at_least 1252226867 2 owner_missing_skill 18 {
    effect remove_mark(self, 1252226867, 2);
    effect add_skill_until_turn_end(self, 18);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 234);
    assert_eq!(package.active_index[0].owner_missing_skill, Some(18));
    assert!(compile_source(
        "skill bad(1) { active play_phase owner_missing_skill 0 { effect draw_cards(self, 1); } }"
    )
    .is_err());
}

#[test]
fn mass_damage_hand_equip_resolver_requires_abi235() {
    let package = compile_source(
        r#"
skill shenfen(2409921791) {
  active play_phase once_per_game mark_at_least 1252226867 6 {
    effect resolve_mass_damage_discard_hand_equip(self, 1252226867, 6, 1, 4);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 235);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP,
                ),
            ]
    }));
}

#[test]
fn choose_gain_card_from_each_other_resolver_requires_abi237() {
    let package = compile_source(
        r#"
skill guixin(3519219618) {
  trigger damage owner_is_target_per_damage_point_other_hand_equip_judge_nonempty optional {
    effect choose_gain_card_from_each_other_then_turn_over(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 237);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
                ),
            ]
    }));
}

#[test]
fn hidden_skill_pile_resolver_requires_abi238() {
    let package = compile_source(
        r#"
skill jilin(3993974385) {
  trigger game_started owner_alive {
    effect resolve_hidden_skill_pile(self, 3993974385, 0, event_card);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 238);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_HIDDEN_SKILL_PILE),
            ]
    }));
}

#[test]
fn non_locked_skill_suppression_requires_abi239() {
    let package = compile_source(
        r#"
skill yingtian_wansha(2204719707) locked {
  trigger dying owner_alive_current_turn {
    effect set_non_locked_skill_suppression(self, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 239);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::SET_NON_LOCKED_SKILL_SUPPRESSION),
            ]
    }));
}

#[test]
fn resolve_dili_yuheng_requires_abi240() {
    let package = compile_source(
        r#"
skill yuheng(3006389674) {
  active play_phase uses_per_turn 1 {
    effect resolve_dili_yuheng(self, 1, event_card);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 240);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_DILI_YUHENG),
            ]
    }));
    assert!(compile_source(
        "skill bad(1) { active play_phase { effect resolve_dili_yuheng(self, 1); } }"
    )
    .is_err());
}

#[test]
fn discard_selected_draw_all_hand_bonus_requires_abi241() {
    let package = compile_source(
        r#"
skill rezhiheng(2455708599) {
  active play_phase uses_per_turn 1 select_he 1 255 {
    effect discard_selected_draw_all_hand_bonus(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 241);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(
                    rule_vm::bytecode::effect_tag::DISCARD_SELECTED_DRAW_ALL_HAND_BONUS,
                ),
            ]
    }));
    assert!(compile_source(
        "skill bad(1) { active play_phase { effect discard_selected_draw_all_hand_bonus(self); } }"
    )
    .is_err());
}

#[test]
fn choose_other_yinghun_requires_abi242() {
    let package = compile_source(
        r#"
skill gzyinghun(2790748306) {
  trigger phase_prepare owner_is_target_wounded optional {
    effect choose_other_yinghun(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 242);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_OTHER_YINGHUN),
            ]
    }));
    assert!(compile_source(
        "skill bad(1) { trigger phase_prepare owner_is_target { effect choose_other_yinghun(); } }"
    )
    .is_err());
}

#[test]
fn choose_other_bizheng_requires_abi243() {
    let package = compile_source(
        r#"
skill bizheng(3859813014) {
  trigger phase_draw_end owner_is_target optional {
    effect choose_other_bizheng(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 243);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_OTHER_BIZHENG),
            ]
    }));
}

#[test]
fn choose_other_xiashu_requires_abi244() {
    let package = compile_source(
        r#"
skill xiashu(2873874515) {
  trigger phase_play owner_is_target_hand_nonempty optional {
    effect choose_other_xiashu(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 244);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_OTHER_XIASHU),
            ]
    }));
}

#[test]
fn resolve_guanchao_rank_requires_abi245() {
    let package = compile_source(
        r#"
skill xinfu_guanchao(3040931439) {
  trigger phase_play owner_is_target optional {
    let mode = choose(1, 2);
    effect add_mark(self, 3040931439, mode);
  }
  trigger card_used owner_source_play_skill_mark_mode_3040931439 {
    effect resolve_guanchao_rank(self, event_card);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 245);
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_GUANCHAO_RANK),
                ]
        })
    }));
}

#[test]
fn resolve_guanwei_requires_abi246() {
    let package = compile_source(
            r#"
skill xinfu_guanwei(3285976176) {
  trigger phase_play_end owner_alive_event_player_play_phase_same_suit_at_least_two_once_per_turn optional {
    effect resolve_guanwei(self, event_player);
  }
}
"#,
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 246);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_GUANWEI),
            ]
    }));
}

#[test]
fn resolve_lanjiang_requires_abi247() {
    let package = compile_source(
        r#"
skill lanjiang(3473961755) {
  trigger phase_end owner_is_target {
    effect resolve_lanjiang(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 247);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_LANJIANG),
            ]
    }));
}

#[test]
fn resolve_rebingyi_requires_abi248() {
    let package = compile_source(
        r#"
skill rebingyi(2238473770) {
  trigger phase_end owner_is_target_hand_nonempty optional {
    effect resolve_rebingyi(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 248);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_REBINGYI),
            ]
    }));
}

#[test]
fn resolve_rebotu_requires_abi249() {
    let package = compile_source(
        r#"
skill rebotu(2231312450) {
  trigger phase_end owner_is_event_player_discard_pile_four_suits_below_alive_cap optional {
    effect resolve_rebotu(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 249);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_REBOTU),
            ]
    }));
}

#[test]
fn resolve_oldimeng_requires_abi250() {
    let package = compile_source(
        r#"
skill oldimeng(2509094453) {
  active play_phase uses_per_turn 1 {
    effect resolve_oldimeng(self, 0);
  }
  trigger phase_play_end owner_is_target {
    effect resolve_oldimeng(self, 1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 250);
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_OLDIMENG),
                ]
        })
    }));
}

#[test]
fn resolve_rejieyin_requires_abi251() {
    let package = compile_source(
        r#"
skill rejieyin(3295441821) {
  active play_phase uses_per_turn 1 select_he 1 1 {
    effect resolve_rejieyin(self);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 251);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::RESOLVE_REJIEYIN),
            ]
    }));
}

#[test]
fn migrated_mobile_and_jueyan_effects_require_latest_additive_abis() {
    for (source, abi, tag) in [
            (
                "skill reanxu(2398589569) { active play_phase uses_per_turn 1 { effect resolve_reanxu(self); } }",
                252,
                rule_vm::bytecode::effect_tag::RESOLVE_REANXU,
            ),
            (
                "skill drlt_jueyan(3834102895) { active play_phase uses_per_turn 1 { effect resolve_drlt_jueyan(self, 2863335974, 3194432315, 3629461497, 4232768410, 3401109480); } }",
                253,
                rule_vm::bytecode::effect_tag::RESOLVE_DRLT_JUEYAN,
            ),
            (
                "skill rejizhi(3401109480) { trigger card_used owner_source_physical_ordinary_trick optional { effect resolve_rejizhi(self); } }",
                254,
                rule_vm::bytecode::effect_tag::RESOLVE_REJIZHI,
            ),
        ] {
            let package = compile_source(source).unwrap();
            assert_eq!(package.manifest.rule_vm_abi_version, abi);
            assert!(package.modules[0].code.windows(2).any(|bytes| {
                bytes == [rule_vm::bytecode::op::HOST_EFFECT, legacy_effect_tag(tag)]
            }));
        }
}

#[test]
fn migrated_yijiang_effects_require_abis_255_through_258() {
    for (source, abi, tag) in [
            (
                "skill jiexun(2662349862) { trigger phase_end owner_is_target optional { effect resolve_jiexun(self, 4079819056); } }",
                255,
                rule_vm::bytecode::effect_tag::RESOLVE_JIEXUN,
            ),
            (
                "skill wengua(4068014439) { trigger game_started owner_alive { effect resolve_wengua_setup(self, 2716743244); } }",
                256,
                rule_vm::bytecode::effect_tag::RESOLVE_WENGUA_SETUP,
            ),
            (
                "skill wengua2(2716743244) { active play_phase select_he 1 1 { effect resolve_wengua(self, 4068014439, 4005759674); } }",
                257,
                rule_vm::bytecode::effect_tag::RESOLVE_WENGUA,
            ),
            (
                "skill xinanguo(3075552814) { active play_phase uses_per_turn 1 targets 1 1 other { effect resolve_xinanguo(self); } }",
                258,
                rule_vm::bytecode::effect_tag::RESOLVE_XINANGUO,
            ),
        ] {
            let package = compile_source(source).unwrap();
            assert_eq!(package.manifest.rule_vm_abi_version, abi);
            assert!(
                package.modules[0]
                    .code
                    .windows(2)
                    .any(|bytes| bytes
                        == [rule_vm::bytecode::op::HOST_EFFECT, legacy_effect_tag(tag)])
            );
        }
}

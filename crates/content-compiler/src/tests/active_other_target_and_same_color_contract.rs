use super::*;

#[test]
fn active_other_target_and_same_color_contract_reaches_package() {
    let package = compile_source(
        r#"
skill shameng(3467425391) {
  active play_phase uses_per_turn 1 targets 1 1 other select_hand 2 2 same_color {
    effect discard_selected(self);
    effect draw_cards(event_source, 2);
    effect draw_cards(self, 3);
  }
}
"#,
    )
    .expect("compile constrained active");
    let active = &package.active_index[0];
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.min,
            selection.max,
            selection.same_color
        )),
        Some((2, 2, true))
    );
}

#[test]
fn active_hand_equip_lowest_hp_and_hand_gt_hp_contracts_reach_package() {
    let package = compile_source(
        r#"
skill nhhuoqi(2275957194) {
  active play_phase uses_per_turn 1 hand_gt_hp targets 1 1 lowest_hp unlinked select_he 1 1 {
    effect discard_selected(self);
    effect recover_hp(event_source, 1);
    effect draw_cards(event_source, 1);
  }
}
"#,
    )
    .unwrap();
    let active = &package.active_index[0];
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand_equip");
    assert!(active.targets.as_ref().unwrap().lowest_hp);
    assert!(active.targets.as_ref().unwrap().unlinked);
    assert!(active.hand_gt_hp);
}

#[test]
fn active_highest_hp_contract_reaches_package() {
    let package = compile_source(
        r#"
skill sanyao(3737839931) {
  active play_phase uses_per_turn 1 targets 1 1 highest_hp select_he 1 1 {
    effect discard_selected(self);
    effect deal_damage(self, event_source, 1);
  }
}
"#,
    )
    .unwrap();
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert!(targets.highest_hp);
    assert!(!targets.lowest_hp);
}

#[test]
fn active_hp_extrema_contracts_are_mutually_exclusive() {
    let error = compile_source(
        r#"
skill impossible(1) {
  active play_phase targets 1 1 lowest_hp highest_hp {
  }
}
"#,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("cannot require both lowest_hp and highest_hp"));
}

#[test]
fn compile_phase_end_biyue() {
    let src = r#"
skill biyue(4) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    assert_eq!(pkg.trigger_index[0].event, "phase_end");
}

#[test]
fn self_skill_query_and_remove_effect_reach_host() {
    let src = r#"
skill temporary(99) {
  active any {
    let remaining = skill_uses_remaining(self_skill);
    if remaining < 2 {
      effect remove_skill(self_skill);
    }
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let mut vm = RuleVm::new();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 9,
            ..Default::default()
        },
        query_stub_int: 1,
        ..Default::default()
    };
    vm.execute(&vmod(&pkg.modules[0]), &mut host, 1_000).unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [rule_api::EffectRequest::RemoveSkill { skill }] if skill.raw() == 9
    ));
}

#[test]
fn move_turn_and_targeted_use_reach_typed_host() {
    let src = r#"
skill routing(100) {
  trigger card_used owner_is_source {
    let old_zone = card_zone(event_card);
    let hand = zone(self, 3);
    effect move_card(event_card, hand, old_zone);
    effect set_turn_state(self, 4);
    effect use_card_on(self, event_card, event_player);
  }
}
"#;
    let pkg = compile_source(src).unwrap();
    let module = &pkg.modules[0];
    assert_eq!(
        module.caps_mask,
        rule_vm::bytecode::caps::EMIT_EFFECTS | rule_vm::bytecode::caps::RUN_QUERIES
    );
    assert!(pkg.manifest.capabilities.emit_effects);
    assert!(pkg.manifest.capabilities.run_queries);
    assert!(!pkg.manifest.capabilities.request_choice);

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_player: 3,
            event_card: 77,
            ..Default::default()
        },
        query_stub_int: 9,
        ..Default::default()
    };
    RuleVm::new().execute(&vmod(module), &mut host, 1_000).unwrap();
    assert!(matches!(
        &host.effects[0],
        rule_api::EffectRequest::MoveCards { cards, to, from }
            if cards[0].raw() == 77 && to.raw() == 9 && from.unwrap().raw() == 9
    ));
    assert!(matches!(
        host.effects[1],
        rule_api::EffectRequest::SetTurnState { current, phase }
            if current.raw() == 2 && phase == 4
    ));
    assert!(matches!(
        &host.effects[2],
        rule_api::EffectRequest::UseCard { player, card, targets }
            if player.raw() == 2 && card.raw() == 77 && targets[0].raw() == 3
    ));
}

#[test]
fn choose_declares_choice_capability() {
    let pkg = compile_source(
        r#"
skill chooser(101) {
  active any {
    let pick = choose(1, 2);
    effect emit_note(1);
  }
}
"#,
    )
    .unwrap();
    let mask = pkg.modules[0].caps_mask;
    assert_ne!(mask & rule_vm::bytecode::caps::REQUEST_CHOICE, 0);
    assert!(pkg.manifest.capabilities.request_choice);
}

#[test]
fn choose_range_uses_dynamic_inclusive_bounds() {
    let pkg = compile_source(
        r#"
skill chooser(102) {
  active play_phase {
    let pick = choose_range(1, hp(self));
    effect draw_cards(self, pick);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(pkg.manifest.rule_vm_abi_version, 92);
    assert!(pkg.manifest.capabilities.request_choice);

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        query_stub_int: 4,
        ..Default::default()
    };
    let paused = RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 1_000)
        .unwrap();
    assert!(matches!(
        paused,
        ExecResult::Paused { options, .. } if options == [1, 2, 3, 4]
    ));
}

#[test]
fn compiles_per_damage_point_cooperative_draw_and_give_effects() {
    let pkg = compile_source(
        r#"
skill cooperative(1010) {
  trigger damage owner_is_source_or_target_other_alive_per_damage_point optional {
    effect draw_each_by_seat(event_player, event_source, 1);
    effect draw_then_give_one(self, event_player + event_source - self, 2, drawn_only);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_is_source_or_target_other_alive_per_damage_point"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_player: 2,
            event_source: 3,
            event_amount: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 1_000)
        .unwrap();
    assert!(matches!(
        host.effects[0],
        rule_api::EffectRequest::DrawEachBySeat { first, second, count: 1, .. }
            if first.raw() == 2 && second.raw() == 3
    ));
    assert!(matches!(
        host.effects[1],
        rule_api::EffectRequest::DrawThenGiveOne {
            player,
            target,
            count: 2,
            drawn_only: true,
            ready_to_give: false,
            ..
        } if player.raw() == 2 && target.raw() == 3
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target { effect draw_then_give_one(self, event_source, 2, hand); } }"
        )
        .is_err());
}

#[test]
fn compiles_once_per_turn_draw_then_take_effect() {
    let pkg = compile_source(
        r#"
skill draw_take(1011) locked {
  trigger damage owner_is_target_once {
    effect draw_then_take_one(self, event_source, 3);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(pkg.trigger_index[0].filter, "owner_is_target_once");
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_player: 2,
            event_source: 3,
            event_amount: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 1_000)
        .unwrap();
    assert!(matches!(
        host.effects[0],
        rule_api::EffectRequest::DrawThenTakeOne {
            player,
            taker,
            count: 3,
            ready_to_take: false,
            ..
        } if player.raw() == 2 && taker.raw() == 3
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target_once { effect draw_then_take_one(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn compiles_scoped_take_one_card_effect() {
    let pkg = compile_source(
        r#"
skill take_equip(1012) {
  trigger damage owner_source_target_alive_has_equip optional {
    effect take_one_card(event_player, self, equip);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_source_target_alive_has_equip"
    );
    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 2,
            event_player: 3,
            event_source: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 1_000)
        .unwrap();
    assert!(matches!(
        host.effects[0],
        rule_api::EffectRequest::TakeOneCard {
            from,
            to,
            equip_only: true,
        } if from.raw() == 3 && to.raw() == 2
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_source { effect take_one_card(event_player, self, hand); } }"
        )
        .is_err());
}

#[test]
fn compiles_once_per_turn_targeted_sha_draw_and_give() {
    let pkg = compile_source(
        r#"
skill targeted_sha(1013) {
  trigger card_targeted owner_targeted_by_other_sha_once optional {
    effect draw_then_give_one(self, event_source, 2, any_hand_equip);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_targeted_by_other_sha_once"
    );
    assert!(pkg.trigger_index[0].optional);
}

#[test]
fn optional_trigger_choice_actor_is_typed_hashed_and_backwards_compatible() {
    let implicit_owner = compile_source(
            "skill choice(1014) { trigger phase_end owner_alive_event_player_equip_empty optional { effect draw_cards(event_player, 1); } }",
        )
        .unwrap();
    let explicit_owner = compile_source(
            "skill choice(1014) { trigger phase_end owner_alive_event_player_equip_empty optional by owner { effect draw_cards(event_player, 1); } }",
        )
        .unwrap();
    let event_player = compile_source(
            "skill choice(1014) { trigger phase_end owner_alive_event_player_equip_empty optional by event_player { effect draw_cards(event_player, 1); } }",
        )
        .unwrap();

    assert_eq!(
        implicit_owner.trigger_index[0].choice_actor,
        content_schema::TriggerChoiceActor::Owner
    );
    assert_eq!(
        event_player.trigger_index[0].choice_actor,
        content_schema::TriggerChoiceActor::EventPlayer
    );
    assert_eq!(
        implicit_owner.manifest.logic_hash,
        explicit_owner.manifest.logic_hash
    );
    assert_ne!(
        implicit_owner.manifest.logic_hash,
        event_player.manifest.logic_hash
    );
    assert!(!String::from_utf8(implicit_owner.to_json_bytes().unwrap())
        .unwrap()
        .contains("choice_actor"));
    assert!(String::from_utf8(event_player.to_json_bytes().unwrap())
        .unwrap()
        .contains("\"choice_actor\":\"event_player\""));

    assert!(compile_source(
        "skill bad(1015) { trigger phase_end owner_alive optional by nobody { } }"
    )
    .is_err());
}

#[test]
fn locked_and_optional_metadata_are_typed() {
    let pkg = compile_source(
        r#"
skill locked_draw(102) locked {
  trigger damage owner_is_target {
    effect draw_cards(self, 1);
  }
}
skill optional_draw(103) {
  trigger damage owner_is_target optional {
    effect draw_cards(self, 1);
  }
}
skill lord_draw(105) locked lord_only {
  modifier hand_limit add alive_other_qun_count;
}
skill internal_helper(106) locked non_gainable {}
skill conversion_branch(107) locked conversion {}
"#,
    )
    .unwrap();
    assert!(pkg.skill_index.iter().any(|skill| skill.locked));
    assert!(pkg
        .skill_index
        .iter()
        .any(|skill| skill.name == "lord_draw" && skill.lord_only));
    assert!(pkg
        .skill_index
        .iter()
        .any(|skill| skill.name == "internal_helper" && skill.non_gainable));
    assert!(pkg
        .skill_index
        .iter()
        .any(|skill| skill.name == "conversion_branch" && skill.conversion));
    assert!(pkg.trigger_index.iter().any(|trigger| trigger.optional));

    let invalid = r#"
skill impossible(104) locked {
  trigger damage owner_is_target optional {
    effect draw_cards(self, 1);
  }
}
"#;
    assert!(compile_source(invalid).is_err());

    let parameterized_group = r#"
skill wei_support(106) {
  trigger damage owner_alive_event_player_group_wei optional {
    effect draw_cards(event_player, 1);
  }
}
"#;
    assert!(compile_source(parameterized_group).is_ok());
    assert!(compile_source(&parameterized_group.replace("group_wei", "group_Wei")).is_err());

    let card_use_history = r#"
skill history_draw(107) {
  trigger phase_play_end owner_is_target_card_used_at_least_hp optional {
    effect draw_cards(self, 2);
  }
}
"#;
    assert!(compile_source(card_use_history).is_ok());

    let physical_discard_history = r#"
skill physical_history_draw(109) {
  trigger phase_end owner_is_target_use_response_discarded_at_least_hp optional {
    effect draw_cards(self, 2);
  }
}
"#;
    let package = compile_source(physical_discard_history).unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_21
    );

    let gainable_per_point = r#"
skill gainable_per_point(110) {
  trigger damage owner_is_target_source_alive_gainable_card_per_damage_point optional {
    if self == event_source {
      effect take_one_card(event_source, self, equip);
    } else {
      effect take_one_card(event_source, self, hand_equip);
    }
  }
}
"#;
    let package = compile_source(gainable_per_point).unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_22
    );
}

#[test]
fn compile_capped_alive_hand_refill_filters() {
    let pkg = compile_source(
        r#"
skill capped_refill(108) {
  trigger game_started owner_alive_hand_below_alive_count_capped_seven optional {
    effect draw_cards(self, alive_count() - hand_count(self));
  }
  trigger death owner_is_source_hand_below_alive_count_capped_seven optional {
    effect draw_cards(self, alive_count() - hand_count(self));
  }
}
"#,
    )
    .unwrap();
    let filters: Vec<_> = pkg
        .trigger_index
        .iter()
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        vec![
            "owner_alive_hand_below_alive_count_capped_seven",
            "owner_is_source_hand_below_alive_count_capped_seven"
        ]
    );
}

#[test]
fn compile_discarded_basic_outside_turn_filters() {
    let package = compile_source(
        r#"
skill shenxian(1) {
  trigger cards_moved owner_alive_other_discarded_basic_outside_turn_once optional {
    effect draw_cards(self, 1);
  }
}
skill oldshenxian(2) {
  trigger cards_moved owner_alive_other_discarded_basic_outside_turn optional {
    effect draw_cards(self, 1);
  }
}
"#,
    )
    .unwrap();
    let filters: Vec<_> = package
        .trigger_index
        .iter()
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        vec![
            "owner_alive_other_discarded_basic_outside_turn_once",
            "owner_alive_other_discarded_basic_outside_turn",
        ]
    );
    assert!(package.trigger_index.iter().all(|trigger| trigger.optional));
}

#[test]
fn compile_received_gift_filter_without_new_vm_abi() {
    let package = compile_source(
        r#"
skill noda_fengcheng(2452779390) locked {
  trigger cards_moved owner_received_gift_from_other {
    effect draw_cards(event_source, 1);
  }
}
"#,
    )
    .unwrap();
    let trigger = &package.trigger_index[0];
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_received_gift_from_other");
    assert!(!trigger.optional);
    assert_eq!(package.manifest.rule_vm_abi_version, 3);
}

#[test]
fn compile_turn_damage_participant_filter() {
    let package = compile_source(
        r#"
skill stdzhongzuo(3465971059) locked {
  trigger turn_ended owner_alive_dealt_or_received_damage_this_turn {
    effect draw_cards(event_player, 1);
    effect draw_cards(self, 1);
  }
}
"#,
    )
    .unwrap();
    let trigger = &package.trigger_index[0];
    assert_eq!(trigger.event, "turn_ended");
    assert_eq!(
        trigger.filter,
        "owner_alive_dealt_or_received_damage_this_turn"
    );
    assert!(package.skill_index[0].locked);
}

#[test]
fn alive_group_count_query_requires_abi6_only_when_used() {
    let package = compile_source(
        r#"
skill fuli(3748306630) {
  trigger dying owner_is_target optional {
    effect recover_hp(self, alive_group_count() - hp(self));
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 6);
    assert!(package.manifest.capabilities.run_queries);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::ALIVE_GROUP_COUNT,
            ]
    }));

    let existing = compile_source(
            "skill alive(1) { trigger game_started owner_alive { effect draw_cards(self, alive_count()); } }",
        )
        .unwrap();
    assert_eq!(existing.manifest.rule_vm_abi_version, 3);
}

#[test]
fn unique_max_current_hp_query_requires_abi7_only_when_used() {
    let package = compile_source(
        r#"
skill refuli(2837551482) {
  trigger dying owner_is_target optional {
    if unique_max_current_hp(self) == 1 {
      effect emit_note(1);
    }
    if unique_max_current_hp_at(self, max_hp(self)) == 1 {
      effect turn_over(self);
    }
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 7);
    assert!(package.manifest.capabilities.run_queries);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::UNIQUE_MAX_CURRENT_HP,
            ]
    }));
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::UNIQUE_MAX_CURRENT_HP_AT,
            ]
    }));

    let abi6 = compile_source(
            "skill fuli(1) { trigger dying owner_is_target optional { effect recover_hp(self, alive_group_count()); } }",
        )
        .unwrap();
    assert_eq!(abi6.manifest.rule_vm_abi_version, 6);
}

#[test]
fn lifetime_damage_dealt_query_requires_abi8_only_when_used() {
    let package = compile_source(
        r#"
skill olfuli(3137508676) {
  trigger dying owner_is_target optional {
    if lifetime_damage_dealt(self) < alive_group_count() {
      effect turn_over(self);
    }
  }
}
"#,
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 8);
    assert!(package.manifest.capabilities.run_queries);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::LIFETIME_DAMAGE_DEALT,
            ]
    }));

    let abi7 = compile_source(
            "skill refuli(1) { trigger dying owner_is_target optional { if unique_max_current_hp(self) == 1 { effect turn_over(self); } } }",
        )
        .unwrap();
    assert_eq!(abi7.manifest.rule_vm_abi_version, 7);
}

#[test]
fn adjust_hand_to_requires_abi9_only_when_used() {
    let package = compile_source(
            "skill dianyi(1) { trigger phase_end owner_is_target_no_turn_damage { effect adjust_hand_to(self, 4); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 9);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::ADJUST_HAND_TO),
            ]
    }));

    let abi8 = compile_source(
            "skill fuli(1) { trigger dying owner_is_target optional { if lifetime_damage_dealt(self) < 1 { effect turn_over(self); } } }",
        )
        .unwrap();
    assert_eq!(abi8.manifest.rule_vm_abi_version, 8);
}

#[test]
fn gain_card_by_kind_requires_abi10_only_when_used() {
    let package = compile_source(
            "skill qiusuo(1) { trigger damage owner_is_source_or_target optional { effect gain_card_by_kind(self, tiesuo); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 10);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::GAIN_CARD_BY_KIND),
            ]
    }));

    let abi9 = compile_source(
            "skill dianyi(1) { trigger phase_end owner_is_target_no_turn_damage { effect adjust_hand_to(self, 4); } }",
        )
        .unwrap();
    assert_eq!(abi9.manifest.rule_vm_abi_version, 9);
}

#[test]
fn public_card_kind_count_requires_abi11_only_when_used() {
    let package = compile_source(
            "skill huaishuang(1) { trigger phase_end owner_is_target { if 0 < public_card_kind_count(tao) { effect gain_card_by_kind(self, tao); effect lose_hp(self, 1); } } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 11);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::PUBLIC_CARD_KIND_COUNT,
            ]
    }));

    let abi10 = compile_source(
            "skill qiusuo(1) { trigger damage owner_is_source_or_target optional { effect gain_card_by_kind(self, tiesuo); } }",
        )
        .unwrap();
    assert_eq!(abi10.manifest.rule_vm_abi_version, 10);
}

#[test]
fn judgment_recovery_requires_abi12_only_when_used() {
    let package = compile_source(
            "skill fuzhao(1) { trigger dying owner_alive optional { effect judge_recover_on_suit(event_player, heart, 1); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 12);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::JUDGE_RECOVER_ON_SUIT),
            ]
    }));

    let abi11 = compile_source(
            "skill huaishuang(1) { trigger phase_end owner_is_target { if 0 < public_card_kind_count(tao) { effect gain_card_by_kind(self, tao); } } }",
        )
        .unwrap();
    assert_eq!(abi11.manifest.rule_vm_abi_version, 11);
}

#[test]
fn dynamic_active_limit_and_virtual_card_require_abi13() {
    let package = compile_source(
            "skill xunxin(1) { active play_phase uses_per_turn owner_hp targets 1 1 other hp_at_least_owner { effect use_virtual_card_on(self, juedou, event_source); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 13);
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, None);
    assert_eq!(active.uses_per_turn_source.as_deref(), Some("owner_hp"));
    assert!(active.targets.as_ref().unwrap().hp_at_least_owner);
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::USE_VIRTUAL_CARD_ON),
            ]
    }));
}

#[test]
fn lost_hp_min_one_active_limit_requires_abi193() {
    let package = compile_source(
            "skill dragtaiguan(1) { active play_phase uses_per_turn owner_lost_hp_min_one targets 1 1 other in_attack_range hand_equip_nonempty select_he 1 1 { effect discard_selected(self); effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(self, event_source); } }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 193);
    assert_eq!(
        package.active_index[0].uses_per_turn_source.as_deref(),
        Some("owner_lost_hp_min_one")
    );
}

#[test]
fn active_source_free_card_legality_requires_abi23() {
    let package = compile_source(
            "skill shizhan(1) { active play_phase uses_per_turn 2 targets 1 1 other can_use_on_owner juedou { effect use_virtual_card_on(event_source, juedou, self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_23
    );
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_turn, Some(2));
    assert_eq!(
        active.targets.as_ref().unwrap().can_use_on_owner.as_deref(),
        Some("juedou")
    );

    let unsupported = compile_source(
            "skill bad(2) { active play_phase targets 1 1 can_use_on_owner sha { effect draw_cards(self, 1); } }",
        )
        .unwrap_err();
    assert!(unsupported
        .to_string()
        .contains("can_use_on_owner currently supports juedou"));
}

#[test]
fn judgment_rewrite_from_hand_or_equip_requires_abi24() {
    let package =
        compile_source("skill reguicai(1) { modifier judgment_rewrite_hand_equip add 1; }")
            .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_24
    );
    assert_eq!(
        package.modifier_index[0].target,
        "judgment_rewrite_hand_equip"
    );
}

#[test]
fn character_state_changed_trigger_requires_abi25() {
    let package = compile_source(
            "skill nsqiyue(1) locked { trigger character_state_changed owner_alive { effect draw_cards(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_25
    );
    assert_eq!(package.trigger_index[0].event, "character_state_changed");
}

#[test]
fn outside_turn_last_hand_once_filter_requires_abi26() {
    let package = compile_source(
            "skill dcshoucheng(1) { trigger cards_moved owner_alive_event_player_outside_turn_from_hand_empty_once optional { effect draw_cards(event_player, 2); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_26
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_event_player_outside_turn_from_hand_empty_once"
    );
}

#[test]
fn typed_character_state_filters_require_abi27() {
    let package = compile_source(
            "skill faen(1) { trigger character_state_changed owner_alive_event_player_became_face_up_or_linked optional { effect draw_cards(event_player, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_27
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_event_player_became_face_up_or_linked"
    );
}

#[test]
fn last_equipment_loss_filter_requires_abi28() {
    let package = compile_source(
            "skill stddianfeng(1) { trigger cards_moved owner_alive_event_player_from_equip_empty optional { effect draw_cards(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_28
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_event_player_from_equip_empty"
    );
}

#[test]
fn prior_red_card_target_filter_requires_abi29() {
    let package = compile_source(
            "skill refuyuan(1) { trigger card_targeted owner_alive_sha_target_no_other_red_card_this_turn optional { effect draw_cards(event_player, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_29
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_sha_target_no_other_red_card_this_turn"
    );
}

#[test]
fn move_equipment_then_draw_requires_abi30() {
    let package = compile_source(
            "skill mbdiaodu(1) { trigger phase_prepare owner_is_target_has_movable_equipment optional { effect move_equipment_then_draw(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_30
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_has_movable_equipment"
    );
    assert_eq!(
        package.modules[0].code,
        vec![
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EFFECT,
            legacy_effect_tag(rule_vm::bytecode::effect_tag::MOVE_EQUIPMENT_THEN_DRAW),
            rule_vm::bytecode::op::HALT,
        ]
    );
}

#[test]
fn virtual_sha_or_draw_and_per_player_sha_history_require_abi31() {
    let package = compile_source(
            "skill stdzhiyi(1) { trigger phase_end owner_alive_used_sha_this_turn { effect choose_use_virtual_sha_or_draw(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_31
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_used_sha_this_turn"
    );
    assert_eq!(
        package.modules[0].code,
        vec![
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EFFECT,
            legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_USE_VIRTUAL_SHA_OR_DRAW),
            rule_vm::bytecode::op::HALT,
        ]
    );
}

#[test]
fn equipment_pair_with_armor_requires_abi32() {
    let package = compile_source(
            "skill stdtiaohe(1) { active play_phase uses_per_turn 1 targets 2 2 equipment_pair_with_armor { effect discard_equipment_pair_with_armor(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_32
    );
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (2, 2));
    assert!(targets.equipment_pair_with_armor);
    assert_eq!(
        package.modules[0].code,
        vec![
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EFFECT,
            legacy_effect_tag(rule_vm::bytecode::effect_tag::DISCARD_EQUIPMENT_PAIR_WITH_ARMOR),
            rule_vm::bytecode::op::HALT,
        ]
    );
}

#[test]
fn black_sha_discard_target_equipment_gain_mount_requires_abi33() {
    let package = compile_source(
            "skill qiaomeng(1) { trigger damage owner_source_black_sha_target_alive_has_discardable_equip optional { effect discard_target_equipment_gain_mount(self, event_player); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_33
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_black_sha_target_alive_has_discardable_equip"
    );
    assert_eq!(
        package.modules[0].code,
        vec![
            rule_vm::bytecode::op::HOST_SELF,
            rule_vm::bytecode::op::HOST_EVENT_PLAYER,
            rule_vm::bytecode::op::HOST_EFFECT,
            legacy_effect_tag(rule_vm::bytecode::effect_tag::DISCARD_TARGET_EQUIPMENT_GAIN_MOUNT),
            rule_vm::bytecode::op::HALT,
        ]
    );
}

#[test]
fn discard_target_equipment_draw_weapon_range_requires_abi136_and_target_contract() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdwanglu(1) { active play_phase uses_per_turn 1 targets 1 1 has_discardable_equip { effect discard_target_equipment_draw_weapon_range(self, event_source); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_136
    );
    let targets = package.active_index[0].targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(targets.allow_self);
    assert!(targets.has_discardable_equip);

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
        .execute(&vmod(&package.modules[0]), &mut host, 10_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player,
            target,
            skill,
        }] if player.raw() == 7 && target.raw() == 8 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 { effect discard_target_equipment_draw_weapon_range(self); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 { effect discard_target_equipment_draw_weapon_range(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn choose_target_reveal_hand_damage_if_sha_requires_abi137() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdmutao(1) { trigger phase_prepare owner_is_target_any_alive_hand_nonempty optional { effect choose_target_reveal_hand_damage_if_sha(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_137
    );
    assert_eq!(package.trigger_index[0].event, "phase_prepare");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_any_alive_hand_nonempty"
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 73,
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
        [EffectRequest::ChooseTargetRevealHandDamageIfSha {
            player,
            amount: 1,
            skill,
        }] if player.raw() == 7 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_prepare owner_is_target_any_alive_hand_nonempty optional { effect choose_target_reveal_hand_damage_if_sha(self); } }"
        )
        .is_err());
}

#[test]
fn choose_other_give_one_hand_equip_requires_abi138() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdyimou(1) { trigger damage owner_is_target_hand_equip_nonempty_other_alive optional { effect choose_other_give_one_hand_equip(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_138
    );
    assert_eq!(package.trigger_index[0].event, "damage");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_hand_equip_nonempty_other_alive"
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 74,
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
        [EffectRequest::ChooseOtherGiveOneHandEquip { player, skill }]
            if player.raw() == 7 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger damage owner_is_target_hand_equip_nonempty_other_alive optional { effect choose_other_give_one_hand_equip(self, event_source); } }"
        )
        .is_err());
}

#[test]
fn turn_gain_filter_requires_abi139() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdshushenx(1) { trigger phase_end owner_alive_gained_three_this_turn_other_alive optional { effect choose_other_draw_or_recover(self, 0, 1, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_139
    );
    assert_eq!(package.trigger_index[0].event, "phase_end");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_gained_three_this_turn_other_alive"
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
        [EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count: 0,
            recover_count: 1,
            times: 1,
            skill,
        }] if player.raw() == 7 && skill.raw() == 0
    ));
}

#[test]
fn bound_reveal_category_requires_abi140() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdzhijie(1) { trigger phase_play owner_alive_event_player_hand_nonempty_once_per_round optional { effect choose_reveal_hand_card_bind_category_target(self, event_player); } trigger card_used owner_source_bound_category_first_three_play_phase { effect draw_cards(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_140
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_event_player_hand_nonempty_once_per_round"
    );
    assert_eq!(
        package.trigger_index[1].filter,
        "owner_source_bound_category_first_three_play_phase"
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
        [EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player,
            target,
            skill,
        }] if player.raw() == 7 && target.raw() == 9 && skill.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger phase_play always { effect choose_reveal_hand_card_bind_category_target(self); } }"
        )
        .is_err());
}

#[test]
fn damage_card_once_per_turn_filter_requires_abi141() {
    let package = compile_source(
            "skill stdkengqiang(1) { trigger card_used owner_source_damage_card_once_per_turn optional { let option = choose(1, 2); if option == 1 { effect draw_cards(self, 2); } if option == 2 { effect lose_hp(self, 1); effect increase_current_card_base_damage(self, event_card, 1); } } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_141
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_damage_card_once_per_turn"
    );
}

#[test]
fn recover_to_max_optional_skill_exchange_requires_abi142() {
    use rule_api::EffectRequest;

    let package = compile_source(
            "skill stdshangjue(1) { trigger dying owner_is_target optional { effect recover_to_max_then_optional_exchange_skill(self, 3956037825); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_142
    );

    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 75,
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
        [EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose: false,
        }] if player.raw() == 7
            && skill.raw() == 0
            && removable_skill_def.raw() == 3_956_037_825
            && restore_skill_def.raw() == 0
    ));
    assert!(compile_source(
            "skill bad(1) { trigger dying owner_is_target optional { effect recover_to_max_then_optional_exchange_skill(self); } }"
        )
        .is_err());
}

#[test]
fn active_positive_hand_limit_requires_abi143() {
    let package = compile_source(
            "skill stdjishe(1) { active play_phase hand_limit_positive { effect draw_cards(self, 1); } modifier hand_limit add -owner_skill_use_count; }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_143
    );
    assert!(package.active_index[0].hand_limit_positive);
}

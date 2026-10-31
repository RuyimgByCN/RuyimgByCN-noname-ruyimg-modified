use super::*;

#[test]
fn compile_jianxiong_reproducible() {
    let a = compile_source(JIANXIONG).unwrap();
    let b = compile_source(JIANXIONG).unwrap();
    assert_eq!(a.manifest.logic_hash, b.manifest.logic_hash);
    assert_eq!(a.modules[0].logic_hash, b.modules[0].logic_hash);
    assert!(!a.trigger_index.is_empty());
}

#[test]
fn compile_accepts_full_u32_skill_id() {
    let pkg = compile_source("skill high(4294967295) locked {}").unwrap();
    assert_eq!(pkg.skill_index[0].skill_def_id, u32::MAX);
}

#[test]
fn compile_response_only_wuxie_view_as() {
    let pkg = compile_source("skill kanpo(9) { view_as wuxie response black; }").unwrap();
    let view = &pkg.view_as_index[0];
    assert_eq!(view.as_kind, "wuxie");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "black");
    assert_eq!(view.source_zone, None);
    assert_eq!(view.uses_per_turn, None);
    assert!(!view.ignore_distance);
    assert_eq!(pkg.manifest.rule_vm_abi_version, 3);
}

#[test]
fn compile_response_view_as_from_hand_and_equip() {
    let pkg = compile_source("skill qingguo(10) { view_as shan response black from hand_equip; }")
        .unwrap();
    assert_eq!(
        pkg.view_as_index[0].source_zone.as_deref(),
        Some("hand_equip")
    );
}

#[test]
fn compile_play_view_as_from_hand_and_equip_with_use_limit() {
    let pkg = compile_source(
            "skill zymoucheng(3584095001) { view_as jiedao play_phase black from hand_equip uses_per_turn 1; }",
        )
        .unwrap();
    let view = &pkg.view_as_index[0];
    assert_eq!(view.as_kind, "jiedao");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, Some(1));

    assert!(
        compile_source("skill bad(1) { view_as sha play_phase red uses_per_turn 0; }").is_err()
    );
}

#[test]
fn compile_tiesuo_view_as_for_use_or_recast() {
    let pkg =
        compile_source("skill lianhuan(2921824505) { view_as tiesuo play_phase club from hand; }")
            .unwrap();
    let view = &pkg.view_as_index[0];
    assert_eq!(view.as_kind, "tiesuo");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "club");
    assert_eq!(view.source_zone, None);
}

#[test]
fn compile_view_as_ignore_distance_requires_abi4() {
    let pkg = compile_source(
            "skill spweiwu(4254049099) { view_as shunshou play_phase red from hand_equip uses_per_turn 1 ignore_distance; }",
        )
        .unwrap();
    let view = &pkg.view_as_index[0];
    assert!(view.ignore_distance);
    assert_eq!(pkg.manifest.rule_vm_abi_version, 4);

    assert!(compile_source(
        "skill bad(1) { view_as sha play_phase red ignore_distance ignore_distance; }"
    )
    .is_err());
}

#[test]
fn compile_view_as_source_color_outcomes_require_abi14() {
    let pkg = compile_source(
            "skill qiulve(3761656212) { view_as sha play_phase non_basic from hand_equip ignore_distance red_unrespondable black_no_use_count; view_as sha response non_basic from hand_equip; }",
        )
        .unwrap();
    let play = &pkg.view_as_index[0];
    assert_eq!(play.filter, "non_basic");
    assert!(play.red_unrespondable);
    assert!(play.black_no_use_count);
    assert_eq!(pkg.manifest.rule_vm_abi_version, 14);

    assert!(compile_source(
        "skill bad(1) { view_as juedou play_phase non_basic red_unrespondable; }"
    )
    .is_err());
}

#[test]
fn compile_multi_card_view_as_requires_abi211() {
    let package = compile_source(
            "skill relonghun(2722411221) { view_as sha play_phase diamond from hand_equip up_to 2 same_suit nature fire multi_card_bonus damage_1; view_as shan response club from hand_equip up_to 2 same_suit multi_card_bonus discard_current_hand_equip; }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_211
    );
    let sha = &package.view_as_index[0];
    assert_eq!(sha.max_cards, 2);
    assert!(sha.same_suit);
    assert_eq!(sha.nature.as_deref(), Some("fire"));
    assert_eq!(sha.multi_card_bonus.as_deref(), Some("damage_1"));
    assert!(compile_source(
        "skill bad(1) { view_as tao response heart up_to 2 multi_card_bonus damage_1; }"
    )
    .is_err());
}

#[test]
fn compile_owner_hp_card_count_view_as_requires_abi236() {
    let package = compile_source(
            "skill longhun(3422266115) { view_as tao response heart from hand_equip_expansion owner_hp_min_one_cards; }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_236
    );
    assert!(package.view_as_index[0].owner_hp_min_one_cards);
    assert!(compile_source(
        "skill bad(1) { view_as tao response heart exactly 2 owner_hp_min_one_cards; }"
    )
    .is_err());
}

#[test]
fn compile_exact_two_wanjian_view_as_requires_abi221() {
    let package = compile_source(
            "skill luanji(2717757870) { view_as wanjian play_phase any from hand exactly 2 same_suit; }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_221
    );
    let rule = &package.view_as_index[0];
    assert_eq!((rule.min_cards, rule.max_cards), (2, 2));
    assert!(rule.same_suit);
}

#[test]
fn compile_discard_weapon_or_lose_hp_cost_requires_abi222() {
    let package = compile_source(
            "skill qiangxix(3151452629) { active play_phase uses_per_turn 2 targets 1 1 other unused_by_skill_this_turn { effect choose_discard_weapon_or_lose_hp(self); effect record_active_target(self, event_source); effect deal_damage(self, event_source, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_222
    );
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.modules.len(), 1);
}

#[test]
fn compile_xinliegong_contract_requires_abi223() {
    let package = compile_source(
            "skill xinliegong(3272111610) { trigger card_targeted owner_source_sha_target_hand_lte_owner_or_hp_gte_owner optional { if hand_count(event_player) < hand_count(self) + 1 { effect make_current_sha_target_unrespondable(self, event_card, event_player); } if hp(self) < hp(event_player) + 1 { effect increase_current_sha_target_damage(self, event_card, event_player, 1); } } modifier sha_distance_unlimited_if_target_distance_lte_rank override 1; }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_223
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner"
    );
    assert_eq!(
        package.modifier_index[0].target,
        "sha_distance_unlimited_if_target_distance_lte_rank"
    );
}

#[test]
fn compile_damage_then_discard_all_judge_requires_abi224() {
    let package = compile_source(
            "skill tyyizhuang(3442202518) { trigger phase_prepare owner_is_target_judge_nonempty optional { effect damage_then_discard_all_judge(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_224
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_judge_nonempty"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::DAMAGE_THEN_DISCARD_ALL_JUDGE),
            ]
    }));
}

#[test]
fn compile_top_two_draw_replacement_requires_abi225() {
    let package = compile_source(
            "skill old_fuhun(4126064689) { trigger phase_draw owner_is_target_draw_not_replaced optional { effect replace_draw_gain_top_two_add_skills_if_different_colors(self, 16, 23); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_225
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_draw_not_replaced"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::REPLACE_DRAW_GAIN_TOP_TWO_ADD_SKILLS_IF_DIFFERENT_COLORS),
                ]
        }));
}

#[test]
fn compile_alternate_zone_target_gain_requires_abi226() {
    let package = compile_source(
            "skill old_zuilun(2251707589) { active play_phase uses_per_turn 2 targets 1 1 other hand_equip_nonempty { effect choose_gain_target_card_alternate_zone_then_target_draw(self, event_source); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_226
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_GAIN_TARGET_CARD_ALTERNATE_ZONE_THEN_TARGET_DRAW),
                ]
        }));
}

#[test]
fn compile_optional_use_given_equipment_requires_abi227() {
    let package = compile_source(
            "skill kaikang(3829685432) { trigger card_targeted owner_alive_sha_target_distance_le_one optional { effect draw_then_give_one(self, event_player, 1, any_hand_equip_use_equipment); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_227
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_sha_target_distance_le_one"
    );
    let mut host = rule_vm::CollectingHost {
        ctx: rule_vm::HostContext {
            self_id: 2,
            event_player: 3,
            ..Default::default()
        },
        ..Default::default()
    };
    rule_vm::RuleVm::new()
        .execute(&vmod(&package.modules[0]), &mut host, 1_000)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [rule_api::EffectRequest::DrawThenGiveOne {
            player,
            target,
            count: 1,
            drawn_only: false,
            use_given_equipment: true,
            ..
        }] if player.raw() == 2 && target.raw() == 3
    ));
}

#[test]
fn compile_persistent_next_judge_skip_requires_abi228() {
    let package = compile_source(
            "skill olcuorui(2241852934) locked { trigger game_started owner_alive_hand_below_alive_count_capped_eight { effect draw_cards(self, alive_count() - hand_count(self)); } trigger card_targeted owner_alive_delayed_trick_target_without_pending_judge_skip { effect skip_next_judge_phase(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_228
    );
    assert_eq!(
        package.trigger_index[1].filter,
        "owner_alive_delayed_trick_target_without_pending_judge_skip"
    );
    assert!(package
        .modules
        .iter()
        .any(|module| module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::SKIP_NEXT_JUDGE_PHASE),
                ]
        })));
}

#[test]
fn compile_fengpo_query_and_filter_requires_abi229() {
    let package = compile_source(
            "skill fengpo(4276310203) { trigger card_targeted owner_source_sha_or_juedou_single_target_first_same_name_this_turn optional { let diamonds = hand_equip_suit_count(event_player, diamond); effect draw_cards(self, diamonds); effect increase_current_card_base_damage(self, event_card, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_229
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_or_juedou_single_target_first_same_name_this_turn"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_QUERY,
                rule_vm::bytecode::query_tag::HAND_EQUIP_SUIT_COUNT,
            ]
    }));
    assert!(compile_source(
            "skill bad(1) { trigger card_targeted owner_source { effect draw_cards(self, hand_equip_suit_count(event_player, purple)); } }"
        )
        .is_err());
}

#[test]
fn compile_twfengpo_hand_queries_and_filter_requires_abi490() {
    let package = compile_source(
            "skill twfengpo(4055936832) { trigger card_targeted owner_source_sha_or_juedou_single_target_hand_nonempty optional { effect reveal_hand_to_player(event_player, self); let diamonds = hand_suit_count(event_player, diamond); let reds = hand_color_count(event_player, red); effect draw_cards(self, diamonds + reds); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_490
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_or_juedou_single_target_hand_nonempty"
    );
    for tag in [
        rule_vm::bytecode::query_tag::HAND_SUIT_COUNT,
        rule_vm::bytecode::query_tag::HAND_COLOR_COUNT,
    ] {
        assert!(package.modules[0]
            .code
            .windows(2)
            .any(|bytes| bytes == [rule_vm::bytecode::op::HOST_QUERY, tag]));
    }
}

#[test]
fn compile_stdfengpo_effect_and_filter_requires_abi230() {
    let package = compile_source(
            "skill stdfengpo(2400535495) { trigger damage_before owner_source_sha_damage_before_endpoint_discardable_hand_equip optional { effect choose_discard_damage_endpoint_card_increase_if_suit(self, event_player, diamond, 1); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_230
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_damage_before_endpoint_discardable_hand_equip"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    legacy_effect_tag(rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_DAMAGE_ENDPOINT_CARD_INCREASE_IF_SUIT),
                ]
        }));
    assert!(compile_source(
            "skill bad(1) { trigger damage_before owner_source { effect choose_discard_damage_endpoint_card_increase_if_suit(self, event_player, purple, 1); } }"
        )
        .is_err());
}

#[test]
fn compile_chongzhen_effect_and_filter_requires_abi231() {
    let package = compile_source(
            "skill chongzhen(4144500448) { trigger card_targeted owner_longdan_view_as_first_counterpart_hand_nonempty optional { effect take_one_event_target_hand(self); } trigger card_responded owner_longdan_view_as_first_counterpart_hand_nonempty optional { effect take_one_event_target_hand(self); } }",
        )
        .unwrap();
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_231
    );
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_longdan_view_as_first_counterpart_hand_nonempty"
    );
    assert!(package.modules[0].code.windows(2).any(|bytes| {
        bytes
            == [
                rule_vm::bytecode::op::HOST_EFFECT,
                legacy_effect_tag(rule_vm::bytecode::effect_tag::TAKE_ONE_EVENT_TARGET_HAND),
            ]
    }));
}

#[test]
fn compile_source_free_conversion_families_require_abi17() {
    let package = compile_source(
            "skill youfeng(1) { view_as ordinary_trick any any from none branch 0 once_per_round next_branch 1 cost gain_max_hp_1; view_as basic any any from none branch 1 once_per_round next_branch 0 cost disable_equip_slot; }",
        )
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 17);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package
        .view_as_index
        .iter()
        .all(|view| { view.source_zone.as_deref() == Some("none") && view.once_per_round }));
}

#[test]
fn reject_partial_source_free_conversion_contracts() {
    assert!(compile_source(
            "skill bad(1) { view_as basic any any from none branch 1 next_branch 0 cost disable_equip_slot; }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { view_as basic any any from hand branch 1 once_per_round next_branch 0 cost disable_equip_slot; }"
        )
        .is_err());
}

#[test]
fn compile_view_as_equipment_filter_requires_abi5() {
    let pkg = compile_source(
        "skill stdyunji(2318160114) { view_as jiedao play_phase card_equip from hand_equip; }",
    )
    .unwrap();
    assert_eq!(pkg.view_as_index[0].filter, "card_equip");
    assert_eq!(pkg.manifest.rule_vm_abi_version, 5);

    let pkg = compile_source(
            "skill duanliang1(3934371847) { view_as bingliang play_phase black_basic_equip from hand_equip; }",
        )
        .unwrap();
    assert_eq!(pkg.view_as_index[0].filter, "black_basic_equip");
    assert_eq!(pkg.manifest.rule_vm_abi_version, 5);

    let pkg =
        compile_source("skill zhanyi_basic_sha(3535942143) { view_as sha response card_basic; }")
            .unwrap();
    assert_eq!(pkg.view_as_index[0].filter, "card_basic");
    assert_eq!(pkg.manifest.rule_vm_abi_version, 5);

    let pkg = compile_source(
        "skill nsgefa(2990982842) { view_as tao response owner_dying_club from hand_equip; }",
    )
    .unwrap();
    assert_eq!(pkg.view_as_index[0].filter, "owner_dying_club");
    assert_eq!(pkg.manifest.rule_vm_abi_version, 5);

    let pkg = compile_source(
        "skill oljieyong(3193662455) { view_as sha response owner_single_red_hand from hand; }",
    )
    .unwrap();
    assert_eq!(pkg.view_as_index[0].filter, "owner_single_red_hand");
    assert_eq!(pkg.manifest.rule_vm_abi_version, 5);

    let pkg = compile_source(
        "skill yanzheng(2899287996) { view_as wuxie response owner_hand_gt_hp from equip; }",
    )
    .unwrap();
    assert_eq!(pkg.view_as_index[0].filter, "owner_hand_gt_hp");
    assert_eq!(pkg.manifest.rule_vm_abi_version, 5);
}

#[test]
fn compile_owner_equip_count_modifier_value() {
    let pkg =
        compile_source("skill spqishe(2801887664) { modifier hand_limit add owner_equip_count; }")
            .unwrap();
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("owner_equip_count")
    );
}

#[test]
fn compile_match_lifetime_damage_history_filter() {
    let pkg = compile_source(
            "skill kuimang(11) locked { trigger death owner_alive_damaged_event_player { effect draw_cards(self, 2); } }",
        )
        .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_alive_damaged_event_player"
    );
}

#[test]
fn compile_not_unique_max_current_hp_filter() {
    let pkg = compile_source(
            "skill olyinji(2633842541) locked { trigger phase_end owner_is_target_not_unique_max_current_hp { effect recover_hp(self, 1); } }",
        )
        .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_is_target_not_unique_max_current_hp"
    );
    assert_eq!(pkg.manifest.rule_vm_abi_version, 3);
}

#[test]
fn compile_owner_or_female_event_player_filter() {
    let pkg = compile_source(
            "skill chengyuan(12) { trigger dying owner_alive_event_player_owner_or_female optional { effect recover_hp(event_player, max_hp(event_player) - hp(event_player)); effect remove_skill(self_skill); } }",
        )
        .unwrap();
    let trigger = &pkg.trigger_index[0];
    assert_eq!(trigger.filter, "owner_alive_event_player_owner_or_female");
    assert!(trigger.optional);
}

#[test]
fn compile_nonnegative_owner_hp_above_other_event_player_filter() {
    let pkg = compile_source(
            "skill yuxin(13) { trigger dying owner_alive_not_target_nonnegative_hp_gt_target optional { effect recover_hp(event_player, hp(self) - hp(event_player)); } }",
        )
        .unwrap();
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_alive_not_target_nonnegative_hp_gt_target"
    );
}

#[test]
fn compile_damage_as_hp_loss_modifier() {
    let pkg = compile_source("skill jueqing(14) locked { modifier damage_as_hp_loss override 1; }")
        .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "damage_as_hp_loss");
    assert_eq!(pkg.modifier_index[0].value, 1);

    let pkg =
        compile_source("skill yinbing(16) locked { modifier damage_as_hp_loss(sha) override 1; }")
            .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "damage_as_hp_loss");
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("sha"));

    let err =
        compile_source("skill bad(17) locked { modifier damage_as_hp_loss(fire) override 1; }")
            .unwrap_err();
    assert!(err.to_string().contains("accepts only the sha selector"));

    let pkg = compile_source(
            "skill gangzhi(15) locked { modifier damage_between_owner_and_other_as_hp_loss override 1; }",
        )
        .unwrap();
    assert_eq!(
        pkg.modifier_index[0].target,
        "damage_between_owner_and_other_as_hp_loss"
    );
}

#[test]
fn compile_sha_use_cancel_modifier() {
    let pkg = compile_source("skill no_sha(7) { modifier sha_use_cancel override 1; }").unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "sha_use_cancel");
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_hand_limit_base_max_hp_modifier() {
    let pkg =
        compile_source("skill max_hp_limit(8) { modifier hand_limit_base_max_hp override 1; }")
            .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "hand_limit_base_max_hp");
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_hand_limit_base_from_turn_damage() {
    let pkg = compile_source(
        "skill damage_limit(9) { modifier hand_limit_base override owner_turn_damage_dealt; }",
    )
    .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "hand_limit_base");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_turn_damage_dealt")
    );
}

#[test]
fn compile_hand_limit_from_sha_hand_count() {
    let pkg =
        compile_source("skill rencheng(11) { modifier hand_limit add owner_sha_hand_count; }")
            .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "hand_limit");
    assert_eq!(modifier.mode, "add");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_sha_hand_count")
    );
}

#[test]
fn compile_card_suit_override() {
    let pkg =
        compile_source("skill hongyan(10) { modifier card_suit(spade) override heart; }").unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_suit");
    assert_eq!(modifier.selector.as_deref(), Some("spade"));
    assert_eq!(modifier.value_source, None);
    assert_eq!(modifier.value, 2);
}

#[test]
fn compile_card_name_override() {
    let pkg = compile_source("skill lieyin(11) { modifier card_name(red) override sha; }").unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_name");
    assert_eq!(modifier.selector.as_deref(), Some("red"));
    assert_eq!(modifier.value_source, None);
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_card_nature_override() {
    let pkg =
        compile_source("skill divine(12) { modifier card_nature(sha) override kami; }").unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_nature");
    assert_eq!(modifier.selector.as_deref(), Some("sha"));
    assert_eq!(modifier.value_source, None);
    assert_eq!(modifier.value, 3);
    assert!(compile_source("skill bad(1) { modifier card_nature override kami; }").is_err());
    assert!(compile_source("skill bad(1) { modifier card_nature(sha) add kami; }").is_err());
}

#[test]
fn compile_card_distance_unlimited_modifier() {
    let pkg =
        compile_source("skill no_distance(9) { modifier card_distance_unlimited override 1; }")
            .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_distance_unlimited");
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_sha_response_cancel_modifier() {
    let pkg =
        compile_source("skill no_sha_response(10) { modifier sha_response_cancel override 1; }")
            .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "sha_response_cancel");
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_directional_card_use_response_lock_modifiers() {
    let pkg = compile_source(
            "skill fuqi(1) { modifier card_use_unrespondable_within_responder_to_owner_distance override 1; } skill refuqi(2) { modifier card_use_unrespondable_within_owner_to_responder_distance override 1; }",
        )
        .unwrap();
    assert_eq!(
        pkg.modifier_index[0].target,
        "card_use_unrespondable_within_responder_to_owner_distance"
    );
    assert_eq!(
        pkg.modifier_index[1].target,
        "card_use_unrespondable_within_owner_to_responder_distance"
    );
}

#[test]
fn compile_armor_link_gain_cancel_modifier() {
    let pkg = compile_source(
        "skill rw_tengjia3(4247798542) locked { modifier armor_link_gain_cancel override 1; }",
    )
    .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "armor_link_gain_cancel");
    assert!(compile_source("skill bad(1) { modifier armor_link_gain_cancel add 1; }").is_err());
}

#[test]
fn compile_per_target_source_distance_filter() {
    let pkg = compile_source(
            "skill yyfuqi(3281236781) locked { modifier card_use_unrespondable_within_responder_to_owner_distance override 1; trigger card_targeted owner_source_target_distance_gt_one { effect draw_cards(self, 1); } }",
        )
        .unwrap();
    assert_eq!(pkg.trigger_index.len(), 1);
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_source_target_distance_gt_one"
    );
}

#[test]
fn compile_card_use_response_lock_and_delayed_target_selector() {
    let pkg = compile_source(
            "skill zhenlue(3469981264) locked { modifier card_use_unrespondable(ordinary_trick) override 1; modifier card_target_cancel(delayed_trick) override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "card_use_unrespondable");
    assert_eq!(
        pkg.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(pkg.modifier_index[1].target, "card_target_cancel");
    assert_eq!(
        pkg.modifier_index[1].selector.as_deref(),
        Some("delayed_trick")
    );
}

#[test]
fn compile_sha_ignored_hand_limit_modifier() {
    let pkg = compile_source("skill keep_sha(11) { modifier sha_ignored_hand_limit override 1; }")
        .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "sha_ignored_hand_limit");
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_card_ignored_hand_limit_selector() {
    let pkg = compile_source(
        "skill keep_tricks(12) { modifier card_ignored_hand_limit(trick) override 1; }",
    )
    .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_ignored_hand_limit");
    assert_eq!(modifier.selector.as_deref(), Some("trick"));
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_card_use_unlimited_modifier() {
    let pkg = compile_source(
            "skill unlimited(13) { modifier card_use_unlimited override 1; modifier card_use_unlimited(black) override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "card_use_unlimited");
    assert_eq!(pkg.modifier_index[0].selector, None);
    assert_eq!(pkg.modifier_index[0].value, 1);
    assert_eq!(pkg.modifier_index[1].target, "card_use_unlimited");
    assert_eq!(pkg.modifier_index[1].selector.as_deref(), Some("black"));
    assert_eq!(pkg.modifier_index[1].value, 1);
}

#[test]
fn compile_card_target_limit_and_lifetime_trigger_requires_abi19() {
    let pkg = compile_source(
            "skill liewu(3721191175) locked { modifier card_target_limit(ordinary_trick) add 1; trigger equip_slot_disabled owner_is_event_player_no_enabled_equip_slot_once_per_game { effect change_max_hp(self, -4); effect add_skill(self, 3695539048); } }",
        )
        .unwrap();
    assert_eq!(pkg.manifest.rule_vm_abi_version, 19);
    assert_eq!(pkg.modifier_index[0].target, "card_target_limit");
    assert_eq!(
        pkg.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(pkg.modifier_index[0].mode, "add");
    assert_eq!(pkg.modifier_index[0].value, 1);
    assert_eq!(pkg.trigger_index[0].event, "equip_slot_disabled");
    assert_eq!(
        pkg.trigger_index[0].filter,
        "owner_is_event_player_no_enabled_equip_slot_once_per_game"
    );
}

#[test]
fn compile_observer_visibility_modifiers_requires_abi20() {
    let pkg = compile_source(
            "skill dongcha(2799956315) locked { modifier observer_foreign_hand_faces_visible override 1; modifier observer_hidden_identities_visible override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.manifest.rule_vm_abi_version, 20);
    assert_eq!(pkg.modifier_index.len(), 2);
    assert_eq!(
        pkg.modifier_index[0].target,
        "observer_foreign_hand_faces_visible"
    );
    assert_eq!(
        pkg.modifier_index[1].target,
        "observer_hidden_identities_visible"
    );
    assert!(pkg
        .modifier_index
        .iter()
        .all(|modifier| modifier.mode == "override" && modifier.value == 1));
}

#[test]
fn card_target_limit_requires_selector_and_add_mode() {
    let missing_selector = compile_source("skill invalid(1) { modifier card_target_limit add 1; }")
        .unwrap_err()
        .to_string();
    assert!(missing_selector.contains("requires a card selector"));

    let wrong_mode = compile_source(
        "skill invalid(1) { modifier card_target_limit(ordinary_trick) override 2; }",
    )
    .unwrap_err()
    .to_string();
    assert!(wrong_mode.contains("requires a constant `add` modifier"));
}

#[test]
fn compile_generic_card_action_cancel_modifiers() {
    let pkg = compile_source(
            "skill xiying2(3555033084) { modifier card_use_cancel override 1; modifier card_response_cancel override 1; modifier card_save_cancel override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 3);
    assert_eq!(pkg.modifier_index[0].target, "card_use_cancel");
    assert_eq!(pkg.modifier_index[1].target, "card_response_cancel");
    assert_eq!(pkg.modifier_index[2].target, "card_save_cancel");
}

#[test]
fn compile_card_use_cancel_selector() {
    let pkg = compile_source(
        "skill qiaoshui2(2280240537) { modifier card_use_cancel(trick) override 1; }",
    )
    .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "card_use_cancel");
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("trick"));
}

#[test]
fn compile_card_save_cancel_tao_selector() {
    let pkg = compile_source(
        "skill reanjian3(4203756175) { modifier card_save_cancel(tao) override 1; }",
    )
    .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "card_save_cancel");
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("tao"));
}

#[test]
fn compile_card_target_cancel_tao_selector() {
    let pkg =
        compile_source("skill ttt(3286554904) { modifier card_target_cancel(tao) override 1; }")
            .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "card_target_cancel");
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("tao"));
}

#[test]
fn compile_discard_phase_equipment_and_legacy_trick_target_selectors() {
    let pkg = compile_source(
            "skill tuipi(3426318690) locked { modifier card_target_cancel(shunshou) override 1; modifier card_target_cancel(guohe) override 1; modifier discard_phase_includes_equip override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 3);
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("shunshou"));
    assert_eq!(pkg.modifier_index[1].selector.as_deref(), Some("guohe"));
    assert_eq!(pkg.modifier_index[2].target, "discard_phase_includes_equip");
    assert_eq!(pkg.modifier_index[2].value, 1);
}

#[test]
fn compile_armor_modifier_channels() {
    let pkg = compile_source(
            "skill armor(3538236867) locked { modifier armor_card_target_cancel(ordinary_trick) override 1; modifier armor_damage_taken_amount(fire) add 1; }\nskill bypass(3538236868) { modifier ignore_armor override 1; }\nskill disabled(3538236869) { modifier armor_effect_disabled override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index[0].target, "armor_card_target_cancel");
    assert_eq!(
        pkg.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(pkg.modifier_index[1].target, "armor_damage_taken_amount");
    assert_eq!(pkg.modifier_index[1].selector.as_deref(), Some("fire"));
    assert_eq!(pkg.modifier_index[2].target, "ignore_armor");
    assert_eq!(pkg.modifier_index[3].target, "armor_effect_disabled");
}

#[test]
fn compile_card_target_cancel_when_owner_hand_empty() {
    let pkg = compile_source(
            "skill kongcheng(3681607931) locked { modifier card_target_cancel(sha) override owner_hand_empty; modifier card_target_cancel(juedou) override owner_hand_empty; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 2);
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("sha"));
    assert_eq!(pkg.modifier_index[1].selector.as_deref(), Some("juedou"));
    assert!(pkg
        .modifier_index
        .iter()
        .all(|modifier| modifier.value_source.as_deref() == Some("owner_hand_empty")));
}

#[test]
fn compile_card_target_cancel_from_current_selector() {
    let pkg = compile_source(
            "skill yoshino_fail(3956929622) { modifier card_target_cancel_from_current(any) override 1; }",
        )
        .unwrap();
    assert_eq!(
        pkg.modifier_index[0].target,
        "card_target_cancel_from_current"
    );
    assert_eq!(pkg.modifier_index[0].selector.as_deref(), Some("any"));
}

#[test]
fn other_user_hp_target_cancel_requires_abi107() {
    let pkg = compile_source(
            "skill maotu(2889501326) locked { modifier card_target_cancel_if_other_user_hp_gte_target override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.manifest.rule_vm_abi_version, 107);
    assert_eq!(
        pkg.modifier_index[0].target,
        "card_target_cancel_if_other_user_hp_gte_target"
    );
    assert!(pkg.modifier_index[0].selector.is_none());
}

#[test]
fn hp_loss_linked_damage_requires_abi108() {
    use rule_api::EffectRequest;

    let pkg = compile_source(
        r#"
skill chenlong(2779773800) {
  active play_phase uses_per_turn 1 targets 1 1 other {
    let amount = choose_range(1, 2);
    effect lose_hp_then_damage_other_lose_max_hp_if_dying(
      self,
      event_source,
      amount,
      self_skill
    );
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_108
    );
    assert!(pkg.manifest.capabilities.request_choice);

    let module = pkg
        .modules
        .iter()
        .find(|module| module.name == pkg.active_index[0].module)
        .unwrap();
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 9,
            self_id: 7,
            event_source: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut vm = RuleVm::new();
    let ExecResult::Paused {
        continuation,
        options,
    } = vm.execute(&vmod(module), &mut host, 1_000).unwrap()
    else {
        panic!("numeric choice did not pause");
    };
    assert_eq!(options, [1, 2]);
    vm.resume(&vmod(module), &mut host, continuation, 2)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount: 2,
            skill,
        }] if player.raw() == 7
            && target.raw() == 8
            && skill.raw() == 9
    ));

    assert!(compile_source(
            "skill bad(1) { trigger dying owner_is_target { effect lose_hp_then_damage_other_lose_max_hp_if_dying(self, event_source, 1, self_skill); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 1 { effect lose_hp_then_damage_other_lose_max_hp_if_dying(self, event_source, 1, self_skill); } }"
        )
        .is_err());
}

#[test]
fn matched_distinct_cost_recovery_requires_abi109() {
    use rule_api::EffectRequest;

    let pkg = compile_source(
        r#"
skill weiyang(4199352428) {
  active play_phase uses_per_turn 1
    targets 1 3 wounded match_selected_card_count
    select_he 1 3 distinct_categories {
    effect discard_selected(self);
    effect recover_selected_targets(1);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_109
    );
    let active = &pkg.active_index[0];
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| targets.wounded && targets.match_selected_card_count));
    assert!(active
        .selection
        .as_ref()
        .is_some_and(|selection| selection.distinct_categories));

    let mut host = CollectingHost {
        ctx: HostContext {
            self_id: 7,
            ..Default::default()
        },
        selected_targets: vec![7, 9],
        selected_cards: vec![21, 22],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::DiscardCards { player, cards },
            EffectRequest::RecoverSelectedTargets {
                source,
                targets,
                amount: 1,
                skill,
            }
        ] if player.raw() == 7
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [21, 22]
            && source.raw() == 7
            && targets.iter().map(|target| target.raw()).collect::<Vec<_>>() == [7, 9]
            && skill.is_none()
    ));

    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 3 wounded match_selected_card_count { effect recover_selected_targets(1); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase targets 1 2 wounded match_selected_card_count select_he 1 3 distinct_categories { effect recover_selected_targets(1); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { trigger phase_end owner_is_target { effect recover_selected_targets(1); } }"
        )
        .is_err());
}

#[test]
fn play_phase_category_recording_requires_abi110() {
    use rule_api::EffectRequest;

    let pkg = compile_source(
        r#"
skill yinhu(4066124366) {
  active play_phase targets 1 1 other
    select_he 1 1 unused_category_by_skill_this_play_phase {
    effect record_selected_play_phase_category(self_skill);
    effect discard_selected(self);
    effect deal_damage_ban_active_if_dying(self, event_source, 1, self_skill);
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        pkg.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_110
    );
    assert!(pkg.active_index[0]
        .selection
        .as_ref()
        .is_some_and(|selection| selection.unused_category_by_skill_this_play_phase));

    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 91,
            self_id: 7,
            event_source: 8,
            ..Default::default()
        },
        selected_cards: vec![21],
        ..Default::default()
    };
    RuleVm::new()
        .execute(&vmod(&pkg.modules[0]), &mut host, 100)
        .unwrap();
    assert!(matches!(
        host.effects.as_slice(),
        [
            EffectRequest::RecordSelectedPlayPhaseCategory { card, skill },
            EffectRequest::DiscardCards { player, cards },
            EffectRequest::DealDamageBanActiveIfDying {
                source,
                target,
                amount: 1,
                skill: damage_skill,
            }
        ] if card.raw() == 21
            && skill.raw() == 91
            && player.raw() == 7
            && cards.iter().map(|card| card.raw()).collect::<Vec<_>>() == [21]
            && source.raw() == 7
            && target.raw() == 8
            && damage_skill.raw() == 91
    ));

    assert!(compile_source(
            "skill bad(1) { active play_phase select_he 1 1 { effect record_selected_play_phase_category(self_skill); } }"
        )
        .is_err());
    assert!(compile_source(
            "skill bad(1) { active play_phase select_he 1 1 unused_category_by_skill_this_turn unused_category_by_skill_this_play_phase { effect discard_selected(self); } }"
        )
        .is_err());
}

#[test]
fn compile_conditional_distance_override() {
    let pkg = compile_source(
        "skill zhuiji(3152159217) { modifier distance_from_if_hp_gte_target override 1; }",
    )
    .unwrap();
    assert_eq!(
        pkg.modifier_index[0].target,
        "distance_from_if_hp_gte_target"
    );
    assert_eq!(pkg.modifier_index[0].value, 1);
}

#[test]
fn compile_card_distance_unlimited_selector() {
    let pkg = compile_source(
        "skill trick_range(14) { modifier card_distance_unlimited(trick) override 1; }",
    )
    .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_distance_unlimited");
    assert_eq!(modifier.selector.as_deref(), Some("trick"));
    assert_eq!(modifier.value, 1);
}

#[test]
fn compile_whole_sha_use_distance_response_lock() {
    let pkg = compile_source(
            "skill fuqi(2567338825) locked { modifier sha_use_unrespondable_within_distance override 1; }",
        )
        .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "sha_use_unrespondable_within_distance");
    assert_eq!(modifier.value, 1);
    assert!(compile_source(
        "skill bad(2567338826) { modifier sha_use_unrespondable_within_distance add 1; }"
    )
    .is_err());
}

#[test]
fn compile_card_discard_protection_selectors() {
    let pkg = compile_source(
            "skill qicai(15) locked { modifier card_discard_protected(equip_non_mount) override 1; modifier card_discard_protected(armor_or_treasure) override 1; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 2);
    assert!(pkg
        .modifier_index
        .iter()
        .all(|modifier| { modifier.target == "card_discard_protected" && modifier.value == 1 }));
}

#[test]
fn compile_bounded_card_distance_selector() {
    let pkg = compile_source(
        "skill duanliang2(2158464053) { modifier card_distance_limit(bingliang) override 2; }",
    )
    .unwrap();
    let modifier = &pkg.modifier_index[0];
    assert_eq!(modifier.target, "card_distance_limit");
    assert_eq!(modifier.selector.as_deref(), Some("bingliang"));
    assert_eq!(modifier.value, 2);
}

#[test]
fn compile_owner_lost_hp_modifier_values() {
    let pkg = compile_source(
            "skill dynamic(15) { modifier distance_from add -owner_lost_hp_min_one; modifier attack_range add owner_lost_hp; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index[0].value, -1);
    assert_eq!(
        pkg.modifier_index[0].value_source.as_deref(),
        Some("owner_lost_hp_min_one")
    );
    assert_eq!(pkg.modifier_index[1].value, 1);
    assert_eq!(
        pkg.modifier_index[1].value_source.as_deref(),
        Some("owner_lost_hp")
    );
}

#[test]
fn compile_parameterized_owner_equip_count_thresholds() {
    let pkg = compile_source(
            "skill yy_baimaxiaoqi_skill(3935591090) locked { modifier attack_range add owner_equip_count; modifier sha_use_limit add owner_equip_count_at_least_2; modifier distance_from add -owner_equip_count_at_least_3; modifier draw_count add owner_equip_count_at_least_4; }",
        )
        .unwrap();
    assert_eq!(pkg.modifier_index.len(), 4);
    assert_eq!(
        pkg.modifier_index[1].value_source.as_deref(),
        Some("owner_equip_count_at_least_2")
    );
    assert_eq!(pkg.modifier_index[2].value, -1);
    assert_eq!(
        pkg.modifier_index[2].value_source.as_deref(),
        Some("owner_equip_count_at_least_3")
    );
    assert_eq!(
        pkg.modifier_index[3].value_source.as_deref(),
        Some("owner_equip_count_at_least_4")
    );

    assert!(compile_source(
        "skill bad(1) { modifier draw_count add owner_equip_count_at_least_0; }",
    )
    .is_err());
}

#[test]
fn compile_contextual_hand_count_damage_modifier_values() {
    let package = compile_source(
            "skill goblin_shiqiang(2912868664) locked { modifier damage_amount add owner_hand_gt_other; modifier damage_taken_amount add owner_hand_lt_other; }",
        )
        .unwrap();
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("owner_hand_gt_other")
    );
    assert_eq!(
        package.modifier_index[1].value_source.as_deref(),
        Some("owner_hand_lt_other")
    );
    assert!(
        compile_source("skill bad(1) { modifier hand_limit add owner_hand_gt_other; }").is_err()
    );

    let package = compile_source(
            "skill pshuoluan(2) locked { modifier damage_amount add owner_qun_other_shu; modifier damage_taken_amount add owner_qun_other_shu; }",
        )
        .unwrap();
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("owner_qun_other_shu")
    );
    assert_eq!(
        package.manifest.rule_vm_abi_version,
        ADDITIVE_RULE_VM_ABI_VERSION_81
    );
    assert!(
        compile_source("skill bad(3) { modifier hand_limit add owner_qun_other_shu; }").is_err()
    );
}

#[test]
fn compile_contextual_turned_over_damage_modifier_value() {
    let package = compile_source(
            "skill lxzhuixi(2387115210) locked { modifier damage_amount add owner_turned_over_diff_other; modifier damage_taken_amount add owner_turned_over_diff_other; }",
        )
        .unwrap();
    assert!(package
        .modifier_index
        .iter()
        .all(|modifier| modifier.value_source.as_deref() == Some("owner_turned_over_diff_other")));
    assert!(compile_source(
        "skill bad(1) { modifier hand_limit add owner_turned_over_diff_other; }"
    )
    .is_err());
}

#[test]
fn compile_contextual_linked_damage_modifier_value() {
    let package = compile_source(
            "skill stdzhongshi(3843853491) locked { modifier damage_amount add owner_linked_diff_other; }",
        )
        .unwrap();
    assert_eq!(
        package.modifier_index[0].value_source.as_deref(),
        Some("owner_linked_diff_other")
    );
    assert!(
        compile_source("skill bad(1) { modifier hand_limit add owner_linked_diff_other; }")
            .is_err()
    );
}

#[test]
fn compile_contextual_sha_hand_equip_damage_modifier_value() {
    let package = compile_source(
            "skill repojun3(2423810510) locked { modifier damage_amount(sha) add owner_hand_equip_gte_other; }",
        )
        .unwrap();
    let modifier = &package.modifier_index[0];
    assert_eq!(modifier.target, "damage_amount");
    assert_eq!(modifier.selector.as_deref(), Some("sha"));
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_hand_equip_gte_other")
    );
    assert!(
        compile_source("skill bad(1) { modifier hand_limit add owner_hand_equip_gte_other; }")
            .is_err()
    );
}

#[test]
fn compile_first_sha_damage_this_turn_modifier_value() {
    let package = compile_source(
            "skill stdduanbing(2170694873) locked { modifier attack_range override 1; modifier damage_amount(sha) add owner_sha_damage_dealt_zero_this_turn; }",
        )
        .unwrap();
    assert_eq!(package.modifier_index.len(), 2);
    assert_eq!(
        package.modifier_index[1].value_source.as_deref(),
        Some("owner_sha_damage_dealt_zero_this_turn")
    );
    assert!(compile_source(
        "skill bad(1) { modifier hand_limit add owner_sha_damage_dealt_zero_this_turn; }"
    )
    .is_err());
}

#[test]
fn compile_global_attack_range_by_queried_hand_and_hp() {
    let package = compile_source(
            "skill stdyinzu(3812447605) locked { modifier global_attack_range add queried_player_hand_gt_hp_signed; }",
        )
        .unwrap();
    let modifier = &package.modifier_index[0];
    assert_eq!(modifier.target, "global_attack_range");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("queried_player_hand_gt_hp_signed")
    );
    assert!(compile_source(
        "skill bad(1) { modifier attack_range add queried_player_hand_gt_hp_signed; }"
    )
    .is_err());
}

//! PA12 Batch 14：Rule-VM 静态修饰器桥接（`PackageModifierEntry` → `VmModifierRule`）。
//!
//! 严格拒绝 `INTENTIONAL_EMPTY_NOOP` 占位（`modifier hand_limit add 0;`），
//! 将真实静态修饰规则编译为 `engine_native::rvm_bridge::VmModifierRule`。

use super::*;
use engine_native::rvm_bridge::{
    VmActiveLimits, VmFilterPolicy, VmHandlerSet, VmModifierMode, VmModifierRule,
    VmModifierSelector, VmModifierTarget, VmModifierValueSource,
};
use std::sync::Arc;

/// 判断是否为 `INTENTIONAL_EMPTY_NOOP` 占位（`modifier hand_limit add 0;`）。
pub(super) fn is_intentional_noop_modifier(m: &content_schema::PackageModifierEntry) -> bool {
    m.target == "hand_limit"
        && m.mode == "add"
        && m.selector.is_none()
        && m.value_source.is_none()
        && m.value == 0
}

fn map_modifier_mode(mode: &str) -> Option<VmModifierMode> {
    Some(match mode {
        "add" => VmModifierMode::Add,
        "override" => VmModifierMode::Override,
        "max" => VmModifierMode::Max,
        "min" => VmModifierMode::Min,
        "multiply" => VmModifierMode::Multiply,
        "veto" => VmModifierMode::Veto,
        _ => return None,
    })
}

fn map_modifier_selector(selector: &str) -> Option<VmModifierSelector> {
    Some(match selector {
        "any" => VmModifierSelector::Any,
        "sha" => VmModifierSelector::Sha,
        "tao" => VmModifierSelector::Tao,
        "bingliang" => VmModifierSelector::Bingliang,
        "lebu" => VmModifierSelector::Lebu,
        "basic" => VmModifierSelector::Basic,
        "non_basic" | "nonbasic" => VmModifierSelector::NonBasic,
        "trick" => VmModifierSelector::Trick,
        "equip" => VmModifierSelector::Equip,
        "spade" => VmModifierSelector::Spade,
        "heart" => VmModifierSelector::Heart,
        "club" => VmModifierSelector::Club,
        "diamond" => VmModifierSelector::Diamond,
        "red" => VmModifierSelector::Red,
        "black" => VmModifierSelector::Black,
        "juedou" => VmModifierSelector::Juedou,
        "black_trick" => VmModifierSelector::BlackTrick,
        "damage_trick" => VmModifierSelector::DamageTrick,
        "sha_fire" => VmModifierSelector::ShaFire,
        "huogong" => VmModifierSelector::Huogong,
        "jiu" => VmModifierSelector::Jiu,
        "physical_jiu" => VmModifierSelector::PhysicalJiu,
        "sha_red" => VmModifierSelector::ShaRed,
        "nanman" => VmModifierSelector::Nanman,
        "ordinary_trick" => VmModifierSelector::OrdinaryTrick,
        "sha_black" => VmModifierSelector::ShaBlack,
        "sha_elemental" => VmModifierSelector::ShaElemental,
        "equip_non_mount" => VmModifierSelector::EquipNonMount,
        "armor_or_treasure" => VmModifierSelector::ArmorOrTreasure,
        "wanjian" => VmModifierSelector::Wanjian,
        "sha_normal" => VmModifierSelector::ShaNormal,
        "sha_heart" => VmModifierSelector::ShaHeart,
        "delayed_trick" => VmModifierSelector::DelayedTrick,
        "wuxie" => VmModifierSelector::Wuxie,
        "guohe" => VmModifierSelector::Guohe,
        "shunshou" => VmModifierSelector::Shunshou,
        "sha_thunder" => VmModifierSelector::ShaThunder,
        "hand_rank_letter" => VmModifierSelector::HandRankLetter,
        "sole_hand_card" => VmModifierSelector::SoleHandCard,
        "sha_diamond" => VmModifierSelector::ShaDiamond,
        "skill_tracked" => VmModifierSelector::SkillTracked,
        "damage_card" => VmModifierSelector::DamageCard,
        "face_up" => VmModifierSelector::FaceUp,
        _ => return None,
    })
}

fn map_modifier_value_source(source: &str) -> Option<(VmModifierValueSource, i32)> {
    if let Some(id_str) = source.strip_prefix("owner_skill_pile_count_negative_") {
        let id = id_str.parse::<u32>().ok().filter(|&v| v > 0)?;
        return Some((VmModifierValueSource::OwnerSkillPileCount(id), -1));
    }
    if source == "owner_skill_pile_count" {
        return Some((VmModifierValueSource::OwnerSkillPileCount(0), 1));
    }
    if let Some(id_str) = source.strip_prefix("owner_skill_pile_count_") {
        let id = id_str.parse::<u32>().ok().filter(|&v| v > 0)?;
        return Some((VmModifierValueSource::OwnerSkillPileCount(id), 1));
    }
    if let Some(mark_str) = source.strip_prefix("owner_mark_") {
        let mark = mark_str.parse::<u32>().ok()?;
        return Some((VmModifierValueSource::OwnerMarkCount(mark), 1));
    }
    if let Some(limit_str) = source.strip_prefix("owner_equip_count_floor_") {
        let n = limit_str.parse::<u32>().ok().filter(|&v| v > 0)?;
        return Some((VmModifierValueSource::OwnerEquipCountFloor(n), 1));
    }
    if let Some(limit_str) = source.strip_prefix("owner_equip_count_at_least_") {
        let n = limit_str.parse::<u32>().ok().filter(|&v| v > 0)?;
        return Some((VmModifierValueSource::OwnerEquipCountAtLeast(n), 1));
    }
    if let Some(limit_str) = source.strip_prefix("owner_skill_play_phase_fire_count_below_") {
        let n = limit_str.parse::<u32>().ok().filter(|&v| v > 0)?;
        return Some((VmModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(n), 1));
    }
    let src = match source {
        "owner_hp" => VmModifierValueSource::OwnerHp,
        "owner_hp_minus_one" => VmModifierValueSource::OwnerHpMinusOne,
        "owner_hp_gt_two" => VmModifierValueSource::OwnerHpGtTwo,
        "owner_hp_le_two" => VmModifierValueSource::OwnerHpLeTwo,
        "owner_max_hp" => VmModifierValueSource::OwnerMaxHp,
        "owner_lost_hp" => VmModifierValueSource::OwnerLostHp,
        "owner_lost_hp_min_one" => VmModifierValueSource::OwnerLostHpMinOne,
        "owner_lost_hp_minus_one" => VmModifierValueSource::OwnerLostHpMinusOne,
        "owner_lost_hp_at_least_two" => VmModifierValueSource::OwnerLostHpAtLeastTwo,
        "owner_lost_hp_negated" => VmModifierValueSource::OwnerLostHpNegated,
        "owner_hand_count" => VmModifierValueSource::OwnerHandCount,
        "owner_hand_empty" => VmModifierValueSource::OwnerHandEmpty,
        "owner_equip_count" | "owner_equipment_count" => VmModifierValueSource::OwnerEquipmentCount,
        "owner_equip_count_exactly_one" => VmModifierValueSource::OwnerEquipCountExactlyOne,
        "owner_equip_count_half_floor" => VmModifierValueSource::OwnerEquipCountHalfFloor,
        "owner_equip_count_odd" => VmModifierValueSource::OwnerEquipCountOdd,
        "owner_equip_count_even" => VmModifierValueSource::OwnerEquipCountEven,
        "field_mount_card_count_ceil_half" => VmModifierValueSource::FieldMountCardCountCeilHalf,
        "owner_shield" => VmModifierValueSource::OwnerShield,
        "living_player_count" => VmModifierValueSource::LivingPlayerCount,
        "alive_other_count_capped_four" => VmModifierValueSource::AliveOtherCountCappedFour,
        "alive_mounted_player_count_min_one" => {
            VmModifierValueSource::AliveMountedPlayerCountMinOne
        }
        "alive_field_rank_thirteen_player_count" => {
            VmModifierValueSource::AliveFieldRankThirteenPlayerCount
        }
        "round_number_capped_five" => VmModifierValueSource::RoundNumberCappedFive,
        "dead_player_count" => VmModifierValueSource::DeadPlayerCount,
        "wounded_player_count" => VmModifierValueSource::WoundedPlayerCount,
        "alive_wounded_or_guiming_wu_count_min_one" => {
            VmModifierValueSource::WoundedPlayerCountMinOne
        }
        "unwounded_player_count" => VmModifierValueSource::UnwoundedPlayerCount,
        "owner_healthy" => VmModifierValueSource::OwnerHealthy,
        "group_count" | "alive_group_count" => VmModifierValueSource::GroupCount,
        "same_group_living_count" | "queried_player_qun_owner_lord_alive" => {
            VmModifierValueSource::SameGroupLivingCount
        }
        "different_group_living_count"
        | "different_group_count"
        | "alive_other_qun_count"
        | "round_at_least_alive_qun_count"
        | "round_below_alive_qun_count" => VmModifierValueSource::DifferentGroupLivingCount,
        "max_group_living_count" => VmModifierValueSource::MaxGroupLivingCount,
        "other_female_living_count" => VmModifierValueSource::OtherFemaleLivingCount,
        "alive_female_count" => VmModifierValueSource::AliveFemaleCount,
        "same_group_count_negated" => VmModifierValueSource::SameGroupCountNegated,
        "different_group_others_count_negated" => {
            VmModifierValueSource::DifferentGroupOthersCountNegated
        }
        "owner_judge_count" => VmModifierValueSource::OwnerJudgeCount,
        "owner_empty_equip_slot_count" => VmModifierValueSource::OwnerEmptyEquipSlotCount,
        "owner_no_mount" => VmModifierValueSource::OwnerNoMount,
        "owner_no_weapon" => VmModifierValueSource::OwnerNoWeapon,
        "owner_armor_empty" => VmModifierValueSource::OwnerArmorEmpty,
        "owner_disabled_equip_slot_count" => VmModifierValueSource::OwnerDisabledEquipSlotCount,
        "owner_enabled_equip_slot_count" => VmModifierValueSource::OwnerEnabledEquipSlotCount,
        "owner_has_equipment" => VmModifierValueSource::OwnerHasEquipment,
        "owner_has_weapon" => VmModifierValueSource::OwnerHasWeapon,
        "owner_has_armor_equip" => VmModifierValueSource::OwnerHasArmorEquip,
        "owner_has_treasure" => VmModifierValueSource::OwnerHasTreasure,
        "owner_has_heart_equip" => VmModifierValueSource::OwnerHasHeartEquip,
        "owner_linked" => VmModifierValueSource::OwnerLinked,
        "owner_all_others_in_attack_range" => VmModifierValueSource::OwnerAllOthersInAttackRange,
        "owner_current_turn_all_others_distance_at_most_one" => {
            VmModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne
        }
        "owner_sha_used_zero" => VmModifierValueSource::OwnerShaUsedZero,
        "owner_sha_used_nonzero" => VmModifierValueSource::OwnerShaUsedNonzero,
        "owner_damaged_not_turn_player" => VmModifierValueSource::OwnerDamagedNotTurnPlayer,
        "owner_hand_count_gt_hp" | "queried_player_hand_gt_hp_signed" => {
            VmModifierValueSource::OwnerHandCountGtHp
        }
        "owner_compare_win_this_turn" => VmModifierValueSource::OwnerCompareWinThisTurn,
        "owner_used_distinct_card_type_count_this_turn"
        | "owner_distinct_hand_card_suits_used_this_turn" => {
            VmModifierValueSource::OwnerUsedDistinctCardTypeCountThisTurn
        }
        "owner_sha_hand_count" => VmModifierValueSource::OwnerShaHandCount,
        "owner_non_damage_hand_type_count" => VmModifierValueSource::OwnerNonDamageHandTypeCount,
        "owner_used_card_count_this_turn_negated" => {
            VmModifierValueSource::OwnerUsedCardCountThisTurnNegated
        }
        "owner_current_turn_card_use_count" => VmModifierValueSource::OwnerCurrentTurnCardUseCount,
        "owner_play_phase_card_use_count_zero" => {
            VmModifierValueSource::OwnerPlayPhaseCardUseCountZero
        }
        "owner_skill_mark" | "owner_mark_count" => VmModifierValueSource::OwnerMarkCount(0),
        "owner_skill_state_u32" | "owner_skill_flag_value" | "owner_skill_use_count" => {
            VmModifierValueSource::OwnerSkillFlagValue(0)
        }
        "owner_skill_state_u32_negative" => {
            return Some((VmModifierValueSource::OwnerSkillFlagValue(0), -1));
        }
        "owner_ren_pile_count" => VmModifierValueSource::OwnerRenPileCount,
        _ => return None,
    };
    Some((src, 1))
}

/// 将单条 `PackageModifierEntry` 编译为 `VmModifierRule`。
pub(super) fn compile_modifier_entry(
    _pkg: &content_schema::ContentPackage,
    m: &content_schema::PackageModifierEntry,
) -> Option<VmModifierRule> {
    if is_intentional_noop_modifier(m) {
        return None;
    }
    if m.skill_def_id == 3_219_705_664
        && m.target == "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
    {
        if m.mode == "override"
            && m.selector.is_none()
            && m.value_source.is_none()
            && m.value == 1
        {
            return Some(VmModifierRule {
                target: VmModifierTarget::CardTargetCancel,
                mode: VmModifierMode::Override,
                selector: None,
                value: 0,
                value_source: None,
            });
        }
        return None;
    }
    let mode = map_modifier_mode(m.mode.as_str())?;
    let selector = match m.selector.as_deref() {
        Some(s) => Some(map_modifier_selector(s)?),
        None => None,
    };
    let (mut value_source, sign) = match m.value_source.as_deref() {
        Some(s) => {
            let (src, mul) = map_modifier_value_source(s)?;
            (Some(src), mul)
        }
        None => (None, 1),
    };
    let target = match m.target.as_str() {
        "hand_limit" => VmModifierTarget::HandLimit,
        "hand_limit_base_max_hp" => VmModifierTarget::HandLimitBaseMaxHp,
        "sha_use_limit" => VmModifierTarget::ShaUseLimit,
        "sha_target_limit" | "sha_extra_targets" => VmModifierTarget::ShaExtraTargets,
        "card_target_limit" if selector == Some(VmModifierSelector::Sha) => {
            VmModifierTarget::ShaExtraTargets
        }
        "attack_range" => VmModifierTarget::AttackRange,
        "distance_from" | "distance_to_others" => VmModifierTarget::DistanceToOthers,
        "distance_to" | "distance_from_others" => VmModifierTarget::DistanceFromOthers,
        "card_distance_unlimited" => VmModifierTarget::CardDistanceUnlimited,
        "card_use_unlimited" => VmModifierTarget::CardUseUnlimited,
        "draw_count" | "draw_phase" | "optional_draw_count" => VmModifierTarget::DrawCount,
        "card_target_cancel" | "armor_card_target_cancel" => VmModifierTarget::CardTargetCancel,
        "sha_use_cancel" => VmModifierTarget::ShaUseCancel,
        "card_use_cancel" => VmModifierTarget::CardUseCancel,
        "hand_card_use_response_cancel" | "card_response_cancel" | "sha_response_cancel" => {
            VmModifierTarget::HandCardUseResponseCancel
        }
        "card_save_cancel" | "dying_peach_target_cancel" => {
            VmModifierTarget::DyingPeachTargetCancel
        }
        "tao_use_save_cancel_for_dying_beyond_distance" => {
            value_source = Some(VmModifierValueSource::TaoSaveCancelBeyondDistance2);
            VmModifierTarget::DyingPeachTargetCancel
        }
        "tao_use_save_cancel_for_non_dying_others_during_owner_turn" => {
            value_source = Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn);
            VmModifierTarget::DyingPeachTargetCancel
        }
        _ => return None,
    };
    Some(VmModifierRule {
        target,
        mode,
        selector,
        value: m.value.saturating_mul(sign),
        value_source,
    })
}

pub(super) fn is_supported_vm_modifier(
    pkg: &content_schema::ContentPackage,
    m: &content_schema::PackageModifierEntry,
) -> bool {
    compile_modifier_entry(pkg, m).is_some()
}

pub(super) fn compile_skill_modifiers(
    pkg: &content_schema::ContentPackage,
    skill_def_id: u32,
) -> Result<Vec<VmModifierRule>, BridgeError> {
    let mut rules = Vec::new();
    for m in pkg
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == skill_def_id)
    {
        let rule = compile_modifier_entry(pkg, m).ok_or_else(|| {
            invalid(
                "modifier_index",
                "native VM modifier declarations are unsupported",
            )
        })?;
        rules.push(rule);
    }
    Ok(rules)
}

/// Bridge pure modifier skills into catalog entries, rejecting `INTENTIONAL_EMPTY_NOOP`.
pub(super) fn bridge_modifier_only_entries(
    pkg: &content_schema::ContentPackage,
    decls: &mut Vec<engine_native::content::extension::VmSkillDecl>,
) -> Result<(), BridgeError> {
    for skill in &pkg.skill_index {
        let has_active = pkg
            .active_index
            .iter()
            .any(|a| a.skill_def_id == skill.skill_def_id);
        let has_trigger = pkg
            .trigger_index
            .iter()
            .any(|t| t.skill_def_id == skill.skill_def_id);
        let has_modifier = pkg
            .modifier_index
            .iter()
            .any(|m| m.skill_def_id == skill.skill_def_id);
        if has_active || has_trigger || !has_modifier {
            continue;
        }
        if skill.lord_only || skill.conversion || skill.non_gainable {
            return Err(invalid(
                "skill_index",
                "modifier-only skill metadata is unsupported",
            ));
        }
        let modifiers = compile_skill_modifiers(pkg, skill.skill_def_id)?;
        let mod_name = format!("{}#modifier", skill.name);
        let code = vec![0x01u8];
        let module = engine_native::rvm_bridge::BytecodeModule {
            logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
                &mod_name, &code, 0, 0, 0,
            ),
            name: mod_name,
            code,
            local_count: 0,
            max_stack: 1,
            entry_pc: 0,
            caps_mask: 0,
        };
        decls.push(engine_native::content::extension::VmSkillDecl {
            declaration_hash: None,
            presentation_binding: None,
            name: skill.name.clone(),
            module,
            trigger: engine_native::catalog::TriggerMeta {
                event: engine_native::event::EventKind::SkillToggled,
                timing: engine_native::event::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            },
            filter: VmFilterPolicy::Handlers(Arc::new(VmHandlerSet {
                skill_def_id: skill.skill_def_id,
                active: None,
                limits: VmActiveLimits::default(),
                triggers: Vec::new(),
                modifiers,
                view_as: Vec::new(),
            })),
            r05_identity: None,
        });
    }
    Ok(())
}

//! PA12 Batch 14 / Batch 16：Rule-VM 静态修饰器桥接（`PackageModifierEntry` → `VmModifierRule`）。
//!
//! 严格拒绝整包纯 `INTENTIONAL_EMPTY_NOOP` 占位（`modifier hand_limit add 0;`），
//! 对含真实 trigger/active/view_as/非空修饰的多技能包跳过伴随 noop 子技，
//! 并将全部静态修饰规则编译为 `engine_native::rvm_bridge::VmModifierRule`。

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

/// 判断单技能包（无 `character_index` 整包混装）内是否含任意真实可执行 trigger/active/view_as 或非空修饰机制。
pub(super) fn package_has_trigger_or_active(pkg: &content_schema::ContentPackage) -> bool {
    pkg.character_index.is_empty()
        && (!pkg.trigger_index.is_empty()
            || !pkg.active_index.is_empty()
            || !pkg.view_as_index.is_empty()
            || pkg
                .modifier_index
                .iter()
                .any(|m| !is_intentional_noop_modifier(m)))
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
        "normal" => VmModifierSelector::DamageNormal,
        "fire" => VmModifierSelector::DamageFire,
        "thunder" => VmModifierSelector::DamageThunder,
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
    if let Some(limit_str) = source.strip_prefix("alive_wounded_count_at_least_") {
        let n = limit_str.parse::<u8>().ok().filter(|&v| v > 0)?;
        return Some((VmModifierValueSource::AliveWoundedCountAtLeast(n), 1));
    }
    if let Some(mark_str) = source.strip_prefix("queried_player_mark_positive_") {
        let mark = mark_str.parse::<u32>().ok()?;
        return Some((VmModifierValueSource::QueriedPlayerMarkPositive(mark), 1));
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
        "alive_wounded_count_zero_or_at_least_four" => {
            VmModifierValueSource::AliveWoundedCountZeroOrAtLeastFour
        }
        "queried_player_linked_first_living_skill_owner" => {
            VmModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner
        }
        "owner_sha_used_exactly_one" => VmModifierValueSource::OwnerShaUsedExactlyOne,
        "owner_sha_used_exactly_two" => VmModifierValueSource::OwnerShaUsedExactlyTwo,
        "owner_qun_other_shu" => VmModifierValueSource::OwnerQunOtherShu,
        "owner_unique_max_hand" => VmModifierValueSource::OwnerUniqueMaxHand,
        "owner_turned_over_diff_other" => VmModifierValueSource::OwnerTurnedOverDiffOther,
        "queried_player_linked_other_than_owner" => {
            VmModifierValueSource::QueriedPlayerLinkedOtherThanOwner
        }
        "queried_player_wounded_in_owner_attack_range_other" => {
            VmModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther
        }
        "queried_player_other_than_owner" => VmModifierValueSource::QueriedPlayerOtherThanOwner,
        "queried_player_judge_nonempty_other_than_owner" => {
            VmModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner
        }
        "queried_source_enemy_owner_ally_target_other_than_owner" => {
            VmModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner
        }
        "owner_hand_gt_other" => VmModifierValueSource::OwnerHandGtOther,
        "owner_hand_lt_other" => VmModifierValueSource::OwnerHandLtOther,
        "owner_hand_equip_gte_other" => VmModifierValueSource::OwnerHandEquipGteOther,
        "owner_sha_damage_dealt_zero_this_turn" => {
            VmModifierValueSource::OwnerShaDamageDealtZeroThisTurn
        }
        "owner_max_hand_before_skill_fires_this_turn" => {
            VmModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn
        }
        "owner_linked_diff_other" => VmModifierValueSource::OwnerLinkedDiffOther,
        "owner_turn_damage_dealt" => VmModifierValueSource::OwnerTurnDamageDealt,
        _ => return None,
    };
    Some((src, 1))
}

fn map_prefix_modifier_target(
    target_str: &str,
    selector: &mut Option<VmModifierSelector>,
) -> Option<VmModifierTarget> {
    if target_str
        .strip_prefix("judgment_rewrite_hand_equip_cost_mark_")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some()
    {
        return Some(VmModifierTarget::JudgmentRewriteHandEquipCostMark);
    }
    if target_str
        .strip_prefix("card_distance_unlimited_target_mark_")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some()
    {
        return Some(VmModifierTarget::CardDistanceUnlimitedTargetMark);
    }
    if target_str
        .strip_prefix("card_distance_unlimited_skill_on_owner_")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some()
    {
        return Some(VmModifierTarget::CardDistanceUnlimitedSkillOnOwner);
    }
    if target_str
        .strip_prefix("card_use_unrespondable_skill_on_owner_")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some()
    {
        return Some(VmModifierTarget::CardUseUnrespondableSkillOnOwner);
    }
    if target_str
        .strip_prefix("sha_use_unlimited_target_mark_")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some()
    {
        return Some(VmModifierTarget::ShaUseUnlimitedTargetMark);
    }
    for (prefix, variant) in [
        ("card_usable_", VmModifierTarget::CardUsable),
        ("card_target_enabled_", VmModifierTarget::CardTargetEnabled),
        ("card_savable_", VmModifierTarget::CardSavable),
        ("card_respondable_", VmModifierTarget::CardRespondable),
        ("card_unrespondable_", VmModifierTarget::CardUnrespondable),
    ] {
        if let Some(rest) = target_str.strip_prefix(prefix) {
            if !rest.is_empty() {
                if selector.is_none() {
                    if let Some(sel) = map_modifier_selector(rest) {
                        *selector = Some(sel);
                    }
                }
                return Some(variant);
            }
        }
    }
    None
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
    let mut selector = match m.selector.as_deref() {
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
        "hand_limit" | "hand_limit_base" => VmModifierTarget::HandLimit,
        "global_hand_limit" => VmModifierTarget::GlobalHandLimit,
        "hand_limit_base_max_hp" => VmModifierTarget::HandLimitBaseMaxHp,
        "sha_ignored_hand_limit" => VmModifierTarget::ShaIgnoredHandLimit,
        "card_ignored_hand_limit" => VmModifierTarget::CardIgnoredHandLimit,
        "sha_use_limit"
        | "sha_use_unlimited_for_skill_bound_target"
        | "sha_no_use_count_if_rank_gt_skill_mark" => VmModifierTarget::ShaUseLimit,
        "global_sha_use_limit" => VmModifierTarget::GlobalShaUseLimit,
        "sha_target_limit" | "sha_extra_targets" => VmModifierTarget::ShaExtraTargets,
        "card_target_limit" if selector == Some(VmModifierSelector::Sha) => {
            VmModifierTarget::ShaExtraTargets
        }
        "card_target_limit" | "card_use_extra_targets" | "card_targets_all" => {
            VmModifierTarget::CardTargetLimit
        }
        "card_use_limit" => VmModifierTarget::CardUseLimit,
        "attack_range" => VmModifierTarget::AttackRange,
        "global_attack_range" => VmModifierTarget::GlobalAttackRange,
        "attack_range_absent" => VmModifierTarget::AttackRangeAbsent,
        "attack_range_fixed" => VmModifierTarget::AttackRangeFixed,
        "distance_from"
        | "distance_to_others"
        | "distance_to_other"
        | "distance_from_skill_bound_target" => VmModifierTarget::DistanceToOthers,
        "distance_from_if_hp_gte_target" => VmModifierTarget::DistanceFromIfHpGteTarget,
        "distance_to" | "distance_from_others" | "distance_from_other" => {
            VmModifierTarget::DistanceFromOthers
        }
        "global_distance_to" => VmModifierTarget::GlobalDistanceTo,
        "card_distance_unlimited"
        | "card_distance_limit"
        | "card_distance_unlimited_if_target_hand_gte_owner"
        | "card_distance_unlimited_for_skill_bound_target"
        | "card_distance_unlimited_for_skill_card_policy"
        | "sha_distance_unlimited_if_rank_lt_skill_mark"
        | "sha_distance_unlimited_if_target_distance_lte_rank" => {
            VmModifierTarget::CardDistanceUnlimited
        }
        "card_use_unlimited" | "first_hand_card_suit_use_unlimited" => {
            VmModifierTarget::CardUseUnlimited
        }
        "draw_count" | "draw_phase" | "optional_draw_count" => VmModifierTarget::DrawCount,
        "global_draw_count" => VmModifierTarget::GlobalDrawCount,
        "card_target_cancel"
        | "armor_card_target_cancel"
        | "card_target_cancel_from_current"
        | "card_target_cancel_owner_turn_banned"
        | "card_target_cancel_if_target_lacks_matching_suit"
        | "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty"
        | "card_target_cancel_if_other_user_hp_gte_target"
        | "card_target_cancel_if_other_user_distance_gt_one"
        | "card_targets_self_only"
        | "immune_shunshou"
        | "immune_le" => VmModifierTarget::CardTargetCancel,
        "sha_use_cancel" => VmModifierTarget::ShaUseCancel,
        "card_use_cancel" | "card_action_cancel_for_skill_color_mask" => {
            VmModifierTarget::CardUseCancel
        }
        "hand_card_use_response_cancel"
        | "card_response_cancel"
        | "sha_response_cancel"
        | "hand_card_gained_this_turn_only" => VmModifierTarget::HandCardUseResponseCancel,
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
        "damage_amount"
        | "sha_damage"
        | "sha_damage_if_outside_target_range"
        | "sha_damage_source_range_threshold" => VmModifierTarget::DamageAmount,
        "damage_taken_amount"
        | "global_damage_taken_amount"
        | "global_damage_taken_final_amount"
        | "damage_taken_source_range_threshold"
        | "sha_damage_taken"
        | "armor_damage_taken_amount"
        | "damage_taken_final_amount" => VmModifierTarget::DamageTakenAmount,
        "damage_taken_cancel" | "damage_cancel" => VmModifierTarget::DamageTakenCancel,
        "ignore_armor"
        | "ignore_armor_for_skill_bound_target"
        | "ignore_armor_for_skill_card_policy"
        | "armor_effect_disabled" => VmModifierTarget::IgnoreArmor,
        "card_use_unrespondable"
        | "card_use_unrespondable_within_responder_to_owner_distance"
        | "card_use_unrespondable_within_owner_to_responder_distance"
        | "card_use_unrespondable_for_skill_card_policy" => VmModifierTarget::CardUseUnrespondable,
        "sha_unrespondable"
        | "sha_unrespondable_within_distance"
        | "sha_use_unrespondable_within_distance"
        | "sha_target_unrespondable" => VmModifierTarget::ShaUnrespondable,
        "dying_save_peach_usable" => VmModifierTarget::DyingSavePeachUsable,
        "card_usable_on_self" => VmModifierTarget::CardUsableOnSelf,
        "card_suit" => VmModifierTarget::CardSuit,
        "card_name" => VmModifierTarget::CardName,
        "card_nature" => VmModifierTarget::CardNature,
        "card_number" => VmModifierTarget::CardNumber,
        "card_usable" => VmModifierTarget::CardUsable,
        "card_target_enabled" => VmModifierTarget::CardTargetEnabled,
        "card_savable" => VmModifierTarget::CardSavable,
        "card_respondable" => VmModifierTarget::CardRespondable,
        "card_unrespondable" => VmModifierTarget::CardUnrespondable,
        "view_as_card_count"
        | "view_as_card_zone"
        | "view_as_card_filter"
        | "view_as_card_kind"
        | "view_as_skill_usable"
        | "active_skill_usable"
        | "judgment_rewrite"
        | "judgment_rewrite_hand"
        | "judgment_rewrite_hand_equip"
        | "judgment_rewrite_hand_red"
        | "judgment_rewrite_hand_equip_spade_or_club"
        | "judgment_rewrite_red_hand_equip_expansion"
        | "judgment_rewrite_black_hand_equip"
        | "judgment_best_from_draw_pile"
        | "nanman_damage_source_as_owner"
        | "card_discard_protected"
        | "card_gain_protected"
        | "observer_foreign_hand_faces_visible"
        | "observer_hidden_identities_visible"
        | "armor_link_gain_cancel"
        | "link_loss_cancel"
        | "response_count"
        | "duel_opponent_response_count"
        | "sha_response_count_if_other_female"
        | "sha_response_count_if_source_owner_target_distance_le_one"
        | "skip_discard_if_no_sha"
        | "skip_discard_phase"
        | "restore_skipped_phase"
        | "turn_face_down_cancel"
        | "turn_over_cancel"
        | "discard_phase_includes_equip"
        | "skip_judge_phase"
        | "draw_replacement_damage"
        | "prepare_scry_count"
        | "draw_phase_scry_four_split_two"
        | "skills_disabled_when_hp_one"
        | "virtual_bagua_if_armor_slot_empty"
        | "damage_as_hp_loss"
        | "damage_between_owner_and_other_as_hp_loss"
        | "damage_nature"
        | "damage_taken_nature"
        | "card_use_cancel_outside_turn"
        | "card_use_cancel_outside_owner_turn"
        | "card_use_cancel_if_owner_flag"
        | "card_use_cancel_if_owner_mark_matches_card_suit"
        | "card_use_response_cancel_if_owner_mark_matches_card_suit"
        | "card_use_cancel_if_owner_marked_by_turn_player_and_not_highest_hp"
        | "card_use_cancel_if_used_card_suit_marked_this_turn"
        | "card_use_cancel_if_target_marked_by_owner"
        | "card_use_response_cancel_if_owner_damaged_and_not_turn_player"
        | "card_target_cancel_if_source_outside_attack_range"
        | "card_target_cancel_if_source_hand_at_least_owner"
        | "card_target_cancel_if_source_hand_gt_owner"
        | "card_target_cancel_if_source_hp_at_least_owner"
        | "card_target_cancel_if_source_damaged"
        | "card_target_cancel_if_other_source_when_owner_hand_empty"
        | "card_target_cancel_if_other_user_during_owner_turn"
        | "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
        | "card_target_cancel_if_non_turn_player"
        | "card_target_cancel_if_owner_no_same_suit_hand_card"
        | "card_target_cancel_if_owner_has_mark"
        | "card_target_cancel_if_user_has_mark"
        | "card_target_cancel_if_owner_linked_first_living_skill_owner"
        | "card_target_cancel_if_owner_mark_matches_card_type"
        | "card_target_cancel_if_owner_mark_matches_card_suit"
        | "card_target_cancel_if_target_has_mark"
        | "card_target_cancel_if_other_target_when_owner_has_mark"
        | "card_target_cancel_if_other_target_during_owner_turn"
        | "card_target_cancel_if_target_outside_attack_range"
        | "card_target_cancel_if_target_distance_gt_one"
        | "card_target_cancel_if_target_hp_lt_owner"
        | "card_target_cancel_if_target_hand_lt_owner"
        | "card_use_unrespondable_if_owner_flag"
        | "card_use_unrespondable_if_target_hand_at_most_owner"
        | "card_use_unrespondable_if_target_hp_at_least_owner"
        | "card_use_unrespondable_if_target_has_mark"
        | "card_use_unrespondable_if_target_marked_by_owner"
        | "card_use_unrespondable_if_user_marked_by_owner"
        | "card_use_unrespondable_if_single_target_damaged"
        | "card_use_unrespondable_if_owner_turn_outside_play_phase"
        | "card_distance_unlimited_if_rank_le_attack_range"
        | "card_distance_unlimited_if_first_use_in_play_phase"
        | "card_distance_unlimited_if_owner_flag"
        | "card_distance_unlimited_if_used_suit_this_turn"
        | "card_distance_unlimited_if_not_used_suit_this_turn"
        | "card_distance_unlimited_if_used_card_type_this_turn"
        | "card_distance_unlimited_if_owner_mark_matches_card_suit"
        | "card_use_unlimited_if_rank_gt_attack_range"
        | "card_use_unlimited_if_owner_flag"
        | "card_use_unlimited_if_used_suit_this_turn"
        | "card_use_unlimited_if_not_used_suit_this_turn"
        | "card_use_unlimited_if_owner_mark_matches_card_type"
        | "card_use_unlimited_if_owner_mark_matches_card_suit"
        | "sha_use_unlimited_if_previous_sha_dealt_damage"
        | "sha_use_unlimited_if_no_trick_used_in_play_phase"
        | "sha_extra_targets_if_no_damage_card_used_in_play_phase"
        | "card_extra_targets_if_used_suit_this_turn"
        | "card_extra_targets_if_owner_mark_matches_card_type"
        | "sha_distance_unlimited_if_slash_used_in_play_phase"
        | "sha_distance_unlimited_if_target_damaged_this_turn"
        | "sha_distance_unlimited_if_target_equipped"
        | "sha_distance_unlimited_if_target_marked_by_owner"
        | "sha_use_unlimited_if_target_damaged_this_turn"
        | "sha_use_unlimited_if_target_marked_by_owner"
        | "card_distance_unlimited_if_target_hand_at_least_owner"
        | "card_distance_unlimited_if_target_hp_at_least_owner"
        | "card_distance_unlimited_if_target_marked_by_owner"
        | "card_use_unlimited_if_target_marked_by_owner"
        | "card_target_allow_outside_attack_range_if_target_hand_eq_hp"
        | "card_target_allow_outside_attack_range_if_target_hand_or_hp_max"
        | "card_target_limit_to_selected_target_within_distance_one"
        | "card_target_limit_to_damaged_this_turn"
        | "card_target_limit_to_last_damaged_by_owner"
        | "card_target_limit_to_marked_by_owner"
        | "card_target_limit_to_min_hp"
        | "shunshou_target_any_distance"
        | "shunshou_from_skill_pile"
        | "jiedao_weapon_target_to_owner"
        | "wuxie_respond_delayed_trick"
        | "dying_save_with_hand_card"
        | "dying_save_self_with_hand_suits"
        | "dying_save_self_with_equip"
        | "initial_hand_count" => VmModifierTarget::SkillConditionalModifier,
        other => map_prefix_modifier_target(other, &mut selector)?,
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
    if is_intentional_noop_modifier(m) {
        return package_has_trigger_or_active(pkg);
    }
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
        if is_intentional_noop_modifier(m) {
            if package_has_trigger_or_active(pkg) {
                continue;
            }
            return Err(invalid(
                "modifier_index",
                "native VM modifier declarations are unsupported",
            ));
        }
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

/// Bridge pure modifier skills into catalog entries, rejecting pure `INTENTIONAL_EMPTY_NOOP` packages.
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
        let modifiers = compile_skill_modifiers(pkg, skill.skill_def_id)?;
        if modifiers.is_empty() {
            continue;
        }
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

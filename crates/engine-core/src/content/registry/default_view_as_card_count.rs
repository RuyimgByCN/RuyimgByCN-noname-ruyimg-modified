//! Content registry operations for `default_view_as_card_count` and related helpers.

use super::*;

pub(crate) fn default_view_as_card_count() -> u8 {
    1
}

pub(crate) fn validate_character_skills<'a>(
    characters: impl IntoIterator<Item = &'a PackageCharacterEntry>,
    available_skills: &BTreeSet<u32>,
) -> Result<(), ContentLoadError> {
    for character in characters {
        if let Some(skill) = character
            .skills
            .iter()
            .find(|skill| !available_skills.contains(skill))
        {
            return Err(ContentLoadError::Invalid(format!(
                "character {} references unavailable skill {}",
                character.id, skill
            )));
        }
    }
    Ok(())
}

pub(crate) fn module_uses_effect_tag(module: &BytecodeModule, target: u16) -> bool {
    use rule_vm::bytecode::op;

    let code = &module.code;
    let mut index = 0usize;
    while let Some(&opcode) = code.get(index) {
        index += 1;
        match opcode {
            op::HOST_EFFECT => {
                let Some(&tag) = code.get(index) else {
                    return false;
                };
                if u16::from(tag) == target {
                    return true;
                }
                index += 1;
            }
            op::HOST_EFFECT_EXT => {
                let Some(bytes) = code.get(index..index + 2) else {
                    return false;
                };
                if u16::from_le_bytes([bytes[0], bytes[1]]) == target {
                    return true;
                }
                index += 2;
            }
            op::CONST_I32 | op::CONST_U32 | op::CONST_BOOL => index += 4,
            op::LOAD | op::STORE | op::HOST_QUERY | op::HOST_CHOICE => index += 1,
            op::JUMP | op::JUMP_IF_FALSE => index += 2,
            op::NOP
            | op::HALT
            | op::TRAP
            | op::DUP
            | op::POP
            | op::NOT
            | op::ADD
            | op::SUB
            | op::MUL
            | op::EQ
            | op::LT
            | op::MIN
            | op::HOST_CHOICE_RANGE
            | op::HOST_SELF
            | op::HOST_EVENT_PLAYER
            | op::HOST_EVENT_SOURCE
            | op::HOST_EVENT_CARD
            | op::HOST_SKILL
            | op::HOST_EVENT_AMOUNT => {}
            _ => return false,
        }
        if index > code.len() {
            return false;
        }
    }
    false
}

/// Stable runtime key for a package-local bytecode module.
pub fn package_module_key(namespace: &str, package_id: &str, module: &str) -> String {
    format!("{namespace}/{package_id}::{module}")
}

pub(crate) fn mod_target_tag(t: ModifierTarget) -> &'static str {
    match t {
        ModifierTarget::AttackRange => "attack_range",
        ModifierTarget::DistanceFrom => "distance_from",
        ModifierTarget::DistanceFromHpGteTarget => "distance_from_if_hp_gte_target",
        ModifierTarget::DistanceFromSkillBoundTarget => "distance_from_skill_bound_target",
        ModifierTarget::LinkLossCancel => "link_loss_cancel",
        ModifierTarget::DistanceTo => "distance_to",
        ModifierTarget::HandLimit => "hand_limit",
        ModifierTarget::HandLimitBase => "hand_limit_base",
        ModifierTarget::DrawCount => "draw_count",
        ModifierTarget::OptionalDrawCount => "optional_draw_count",
        ModifierTarget::ShaDamage => "sha_damage",
        ModifierTarget::ShaUseLimit => "sha_use_limit",
        ModifierTarget::ShaTargetLimit => "sha_target_limit",
        ModifierTarget::CardTargetsAllFor(_) => "card_targets_all",
        ModifierTarget::CardTargetCancelFor(_) => "card_target_cancel",
        ModifierTarget::CardTargetCancelFromCurrentFor(_) => "card_target_cancel_from_current",
        ModifierTarget::CardTargetCancelTurnBannedFor(_) => "card_target_cancel_owner_turn_banned",
        ModifierTarget::CardTargetsSelfOnly => "card_targets_self_only",
        ModifierTarget::ResponseCount => "response_count",
        ModifierTarget::SkipDiscardIfNoSha => "skip_discard_if_no_sha",
        ModifierTarget::DrawReplacementDamage => "draw_replacement_damage",
        ModifierTarget::PrepareScryCount => "prepare_scry_count",
        ModifierTarget::JudgmentRewrite => "judgment_rewrite",
        ModifierTarget::DamageAmount => "damage_amount",
        ModifierTarget::DamageNature => "damage_nature",
        ModifierTarget::DamageCancel => "damage_cancel",
        ModifierTarget::CardUseCancel => "card_use_cancel",
        ModifierTarget::CardUseCancelFor(_) => "card_use_cancel",
        ModifierTarget::CardResponseCancel => "card_response_cancel",
        ModifierTarget::CardResponseCancelFor(_) => "card_response_cancel",
        ModifierTarget::CardSaveCancel => "card_save_cancel",
        ModifierTarget::CardSaveCancelFor(_) => "card_save_cancel",
        ModifierTarget::ImmuneShunshou => "immune_shunshou",
        ModifierTarget::ImmuneLe => "immune_le",
        ModifierTarget::CardAttr(_) => "card_attr",
        ModifierTarget::ShaUseCancel => "sha_use_cancel",
        ModifierTarget::HandLimitBaseMaxHp => "hand_limit_base_max_hp",
        ModifierTarget::CardDistanceUnlimited => "card_distance_unlimited",
        ModifierTarget::CardDistanceUnlimitedTargetMark(_) => "card_distance_unlimited_target_mark",
        ModifierTarget::CardDistanceUnlimitedSkillOnOwner(_) => {
            "card_distance_unlimited_skill_on_owner"
        }
        ModifierTarget::CardUseUnrespondableSkillOnOwner(_) => {
            "card_use_unrespondable_skill_on_owner"
        }
        ModifierTarget::ShaUseUnlimitedTargetMark(_) => "sha_use_unlimited_target_mark",
        ModifierTarget::CardDistanceUnlimitedFor(_) => "card_distance_unlimited",
        ModifierTarget::CardDistanceLimitFor(_) => "card_distance_limit",
        ModifierTarget::ShaResponseCancel => "sha_response_cancel",
        ModifierTarget::ShaIgnoredHandLimit => "sha_ignored_hand_limit",
        ModifierTarget::CardIgnoredHandLimit(_) => "card_ignored_hand_limit",
        ModifierTarget::CardUseUnlimited => "card_use_unlimited",
        ModifierTarget::CardUseUnlimitedFor(_) => "card_use_unlimited",
        ModifierTarget::FirstHandCardSuitUseUnlimited => "first_hand_card_suit_use_unlimited",
        ModifierTarget::CardTargetLimitFor(_) => "card_target_limit",
        ModifierTarget::DamageTakenCancel => "damage_taken_cancel",
        ModifierTarget::DamageTakenCancelForNature(_) => "damage_taken_cancel",
        ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(_) => {
            "card_distance_unlimited_if_target_hand_gte_owner"
        }
        ModifierTarget::HandCardUseResponseCancel => "hand_card_use_response_cancel",
        ModifierTarget::HandCardGainedThisTurnOnly => "hand_card_gained_this_turn_only",
        ModifierTarget::CardTargetCancelIfTargetLacksMatchingSuit => {
            "card_target_cancel_if_target_lacks_matching_suit"
        }
        ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(_) => {
            "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty"
        }
        ModifierTarget::CardSuit(_) => "card_suit",
        ModifierTarget::CardName(_) => "card_name",
        ModifierTarget::ShaUnrespondableFor(_) => "sha_unrespondable",
        ModifierTarget::DamageTakenAmount => "damage_taken_amount",
        ModifierTarget::DamageTakenAmountForNature(_) => "damage_taken_amount",
        ModifierTarget::ShaDamageFor(_) => "sha_damage",
        ModifierTarget::ShaUnrespondableWithinDistance => "sha_unrespondable_within_distance",
        ModifierTarget::ShaTargetUnrespondable => "sha_target_unrespondable",
        ModifierTarget::ShaUseUnrespondableWithinDistance => {
            "sha_use_unrespondable_within_distance"
        }
        ModifierTarget::CardUseUnrespondableWithinResponderToOwnerDistance => {
            "card_use_unrespondable_within_responder_to_owner_distance"
        }
        ModifierTarget::CardUseUnrespondableWithinOwnerToResponderDistance => {
            "card_use_unrespondable_within_owner_to_responder_distance"
        }
        ModifierTarget::CardUseUnrespondableFor(_) => "card_use_unrespondable",
        ModifierTarget::DamageAmountForSha => "damage_amount_sha",
        ModifierTarget::GlobalAttackRange => "global_attack_range",
        ModifierTarget::GlobalDamageTakenAmount => "global_damage_taken_amount",
        ModifierTarget::GlobalDamageTakenAmountForNature(_) => "global_damage_taken_amount",
        ModifierTarget::GlobalDamageTakenFinalAmount => "global_damage_taken_final_amount",
        ModifierTarget::GlobalDamageTakenFinalAmountForNature(_) => {
            "global_damage_taken_final_amount"
        }
        ModifierTarget::GlobalHandLimit => "global_hand_limit",
        ModifierTarget::GlobalDrawCount => "global_draw_count",
        ModifierTarget::GlobalShaUseLimit => "global_sha_use_limit",
        ModifierTarget::JudgmentRewriteHandEquipCostMark(_) => {
            "judgment_rewrite_hand_equip_cost_mark"
        }
        ModifierTarget::CardNatureFor(_) => "card_nature",
        ModifierTarget::GlobalDistanceTo => "global_distance_to",
        ModifierTarget::NanmanDamageSourceAsOwner => "nanman_damage_source_as_owner",
        ModifierTarget::DamageTakenSourceRangeThreshold => "damage_taken_source_range_threshold",
        ModifierTarget::SkipJudgePhase => "skip_judge_phase",
        ModifierTarget::InitialHandCount => "initial_hand_count",
        ModifierTarget::DamageTakenFinalAmount => "damage_taken_final_amount",
        ModifierTarget::DamageTakenFinalAmountForNature(_) => "damage_taken_final_amount",
        ModifierTarget::DamageTakenNature => "damage_taken_nature",
        ModifierTarget::DamageAmountForNature(_) => "damage_amount",
        ModifierTarget::DamageAsHpLoss => "damage_as_hp_loss",
        ModifierTarget::DamageBetweenOwnerAndOtherAsHpLoss => {
            "damage_between_owner_and_other_as_hp_loss"
        }
        ModifierTarget::CardDiscardProtectedFor(_) => "card_discard_protected",
        ModifierTarget::ShaDamageTaken => "sha_damage_taken",
        ModifierTarget::ShaDamageIfOutsideTargetRange => "sha_damage_if_outside_target_range",
        ModifierTarget::ShaDamageSourceRangeThreshold => "sha_damage_source_range_threshold",
        ModifierTarget::DamageAsHpLossForSha => "damage_as_hp_loss",
        ModifierTarget::ArmorCardTargetCancelFor(_) => "armor_card_target_cancel",
        ModifierTarget::ArmorDamageTakenAmountForNature(_) => "armor_damage_taken_amount",
        ModifierTarget::ArmorLinkGainCancel => "armor_link_gain_cancel",
        ModifierTarget::IgnoreArmor => "ignore_armor",
        ModifierTarget::ArmorEffectDisabled => "armor_effect_disabled",
        ModifierTarget::DrawPhaseScryFourSplitTwo => "draw_phase_scry_four_split_two",
        ModifierTarget::SkipDiscardPhase => "skip_discard_phase",
        ModifierTarget::DiscardPhaseIncludesEquip => "discard_phase_includes_equip",
        ModifierTarget::ObserverForeignHandFacesVisible => "observer_foreign_hand_faces_visible",
        ModifierTarget::ObserverHiddenIdentitiesVisible => "observer_hidden_identities_visible",
        ModifierTarget::JudgmentRewriteHandEquip => "judgment_rewrite_hand_equip",
        ModifierTarget::JudgmentRewriteRedHandEquipExpansion => {
            "judgment_rewrite_red_hand_equip_expansion"
        }
        ModifierTarget::JudgmentRewriteBlackHandEquip => "judgment_rewrite_black_hand_equip",
        ModifierTarget::SkillsDisabledWhenHpOne => "skills_disabled_when_hp_one",
        ModifierTarget::CardDistanceUnlimitedForSkillBoundTarget => {
            "card_distance_unlimited_for_skill_bound_target"
        }
        ModifierTarget::ShaUseUnlimitedForSkillBoundTarget => {
            "sha_use_unlimited_for_skill_bound_target"
        }
        ModifierTarget::IgnoreArmorForSkillBoundTarget => "ignore_armor_for_skill_bound_target",
        ModifierTarget::CardActionCancelForSkillColorMask => {
            "card_action_cancel_for_skill_color_mask"
        }
        ModifierTarget::IgnoreArmorForSkillCardPolicy => "ignore_armor_for_skill_card_policy",
        ModifierTarget::CardUseUnrespondableForSkillCardPolicy => {
            "card_use_unrespondable_for_skill_card_policy"
        }
        ModifierTarget::JudgmentBestFromDrawPile => "judgment_best_from_draw_pile",
        ModifierTarget::VirtualBaguaIfArmorSlotEmpty => "virtual_bagua_if_armor_slot_empty",
        ModifierTarget::ShaResponseCountIfOtherFemale => "sha_response_count_if_other_female",
        ModifierTarget::TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn => {
            "tao_use_save_cancel_for_non_dying_others_during_owner_turn"
        }
        ModifierTarget::TaoUseSaveCancelForDyingBeyondDistance => {
            "tao_use_save_cancel_for_dying_beyond_distance"
        }
        ModifierTarget::ShaResponseCountIfSourceOwnerTargetDistanceLeOne => {
            "sha_response_count_if_source_owner_target_distance_le_one"
        }
        ModifierTarget::CardTargetCancelIfOtherUserHpGteTarget => {
            "card_target_cancel_if_other_user_hp_gte_target"
        }
        ModifierTarget::TurnFaceDownCancel => "turn_face_down_cancel",
        ModifierTarget::RestoreSkippedPhase => "restore_skipped_phase",
        ModifierTarget::CardTargetCancelIfOtherUserCardUseCountLtOwnerMark => {
            "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
        }
        ModifierTarget::CardDistanceUnlimitedForSkillCardPolicy => {
            "card_distance_unlimited_for_skill_card_policy"
        }
        ModifierTarget::TurnOverCancel => "turn_over_cancel",
        ModifierTarget::CardTargetCancelIfOtherUserDistanceGtOneFor(_) => {
            "card_target_cancel_if_other_user_distance_gt_one"
        }
        ModifierTarget::DuelOpponentResponseCount => "duel_opponent_response_count",
        ModifierTarget::ShaDistanceUnlimitedIfRankLtSkillMark => {
            "sha_distance_unlimited_if_rank_lt_skill_mark"
        }
        ModifierTarget::ShaNoUseCountIfRankGtSkillMark => "sha_no_use_count_if_rank_gt_skill_mark",
        ModifierTarget::ShaDistanceUnlimitedIfTargetDistanceLteRank => {
            "sha_distance_unlimited_if_target_distance_lte_rank"
        }
        ModifierTarget::CardGainProtectedFor(_) => "card_gain_protected",
        ModifierTarget::CardNumber => "card_number",
    }
}

pub(crate) fn mod_mode_tag(m: CombineMode) -> &'static str {
    match m {
        CombineMode::Add => "add",
        CombineMode::Override => "override",
        CombineMode::Min => "min",
        CombineMode::Max => "max",
        CombineMode::Multiply => "multiply",
        CombineMode::Veto => "veto",
    }
}

pub(crate) fn map_mod_value_source(source: &str) -> Result<ModifierValueSource, String> {
    if let Some(skill_def) = source.strip_prefix("owner_skill_pile_count_negative_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(ModifierValueSource::OwnerSkillPileCountNegative)
            .ok_or_else(|| format!("invalid owner skill-pile value source {source}"));
    }
    if source == "owner_skill_pile_count" {
        return Ok(ModifierValueSource::OwnerSkillPileCount);
    }
    if let Some(skill_def) = source.strip_prefix("owner_skill_pile_count_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(ModifierValueSource::OwnerSkillPileCountOf)
            .ok_or_else(|| format!("invalid owner skill-pile value source {source}"));
    }
    if let Some(mark) = source.strip_prefix("queried_player_mark_positive_") {
        return mark
            .parse::<u32>()
            .map(ModifierValueSource::QueriedPlayerMarkPositive)
            .map_err(|_| format!("invalid queried player mark value source {source}"));
    }
    if let Some(mark) = source.strip_prefix("owner_mark_") {
        return mark
            .parse::<u32>()
            .map(ModifierValueSource::OwnerMark)
            .map_err(|_| format!("invalid owner mark value source {source}"));
    }
    if let Some(threshold) = source.strip_prefix("alive_wounded_count_at_least_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(ModifierValueSource::AliveWoundedCountAtLeast)
            .ok_or_else(|| format!("invalid alive wounded threshold value source {source}"));
    }
    if let Some(threshold) = source.strip_prefix("owner_equip_count_at_least_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(ModifierValueSource::OwnerEquipCountAtLeast)
            .ok_or_else(|| format!("invalid owner equipment threshold value source {source}"));
    }
    if let Some(threshold) = source.strip_prefix("owner_equip_count_floor_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(ModifierValueSource::OwnerEquipCountFloor)
            .ok_or_else(|| format!("invalid owner equipment floor value source {source}"));
    }
    match source {
        "owner_hp" => Ok(ModifierValueSource::OwnerHp),
        "owner_max_hp" => Ok(ModifierValueSource::OwnerMaxHp),
        "owner_hp_minus_one" => Ok(ModifierValueSource::OwnerHpMinusOne),
        "owner_lost_hp" => Ok(ModifierValueSource::OwnerLostHp),
        "owner_lost_hp_minus_one" => Ok(ModifierValueSource::OwnerLostHpMinusOne),
        "owner_lost_hp_min_one" => Ok(ModifierValueSource::OwnerLostHpMinOne),
        "owner_skill_mark" => Ok(ModifierValueSource::OwnerSkillMark),
        "owner_skill_pile_count" => Ok(ModifierValueSource::OwnerSkillPileCount),
        "owner_skill_state_u32" => Ok(ModifierValueSource::OwnerSkillStateU32),
        "owner_skill_state_u32_negative" => Ok(ModifierValueSource::OwnerSkillStateU32Negative),
        "queried_player_qun_owner_lord_alive" => {
            Ok(ModifierValueSource::QueriedPlayerQunOwnerLordAlive)
        }
        "owner_linked" => Ok(ModifierValueSource::OwnerLinked),
        "owner_skill_use_count" => Ok(ModifierValueSource::OwnerSkillUseCount),
        "alive_wounded_or_guiming_wu_count_min_one" => {
            Ok(ModifierValueSource::AliveWoundedOrGuimingWuCountMinOne)
        }
        "alive_group_count" => Ok(ModifierValueSource::AliveGroupCount),
        "alive_female_count" => Ok(ModifierValueSource::AliveFemaleCount),
        "alive_other_count_capped_four" => Ok(ModifierValueSource::AliveOtherCountCappedFour),
        "owner_no_mount" => Ok(ModifierValueSource::OwnerNoMount),
        "owner_hand_empty" => Ok(ModifierValueSource::OwnerHandEmpty),
        "owner_has_treasure" => Ok(ModifierValueSource::OwnerHasTreasure),
        "alive_mounted_player_count_min_one" => {
            Ok(ModifierValueSource::AliveMountedPlayerCountMinOne)
        }
        "owner_equip_count_odd" => Ok(ModifierValueSource::OwnerEquipCountOdd),
        "owner_equip_count_even" => Ok(ModifierValueSource::OwnerEquipCountEven),
        "owner_equip_count_half_floor" => Ok(ModifierValueSource::OwnerEquipCountHalfFloor),
        "owner_equip_count_exactly_one" => Ok(ModifierValueSource::OwnerEquipCountExactlyOne),
        "owner_equip_count" => Ok(ModifierValueSource::OwnerEquipCount),
        "owner_hp_gt_two" => Ok(ModifierValueSource::OwnerHpGtTwo),
        "owner_hp_le_two" => Ok(ModifierValueSource::OwnerHpLeTwo),
        "owner_lost_hp_at_least_two" => Ok(ModifierValueSource::OwnerLostHpAtLeastTwo),
        "owner_all_others_in_attack_range" => Ok(ModifierValueSource::OwnerAllOthersInAttackRange),
        "owner_current_turn_all_others_distance_at_most_one" => {
            Ok(ModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne)
        }
        "owner_sha_used_zero" => Ok(ModifierValueSource::OwnerShaUsedZero),
        "owner_sha_used_nonzero" => Ok(ModifierValueSource::OwnerShaUsedNonzero),
        "owner_sha_used_exactly_one" => Ok(ModifierValueSource::OwnerShaUsedExactlyOne),
        "owner_sha_used_exactly_two" => Ok(ModifierValueSource::OwnerShaUsedExactlyTwo),
        "owner_max_hand_before_skill_fires_this_turn" => {
            Ok(ModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn)
        }
        "owner_non_damage_hand_type_count" => Ok(ModifierValueSource::OwnerNonDamageHandTypeCount),
        "owner_sha_hand_count" => Ok(ModifierValueSource::OwnerShaHandCount),
        "owner_turn_damage_dealt" => Ok(ModifierValueSource::OwnerTurnDamageDealt),
        "owner_distinct_hand_card_suits_used_this_turn" => {
            Ok(ModifierValueSource::OwnerDistinctHandCardSuitsUsedThisTurn)
        }
        "owner_current_turn_card_use_count" => {
            Ok(ModifierValueSource::OwnerCurrentTurnCardUseCount)
        }
        "owner_play_phase_card_use_count_zero" => {
            Ok(ModifierValueSource::OwnerPlayPhaseCardUseCountZero)
        }
        value
            if value
                .strip_prefix("owner_skill_play_phase_fire_count_below_")
                .and_then(|limit| limit.parse::<u32>().ok())
                .is_some_and(|limit| limit > 0) =>
        {
            Ok(ModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(
                value
                    .strip_prefix("owner_skill_play_phase_fire_count_below_")
                    .and_then(|limit| limit.parse::<u32>().ok())
                    .expect("validated positive play-phase fire-count limit"),
            ))
        }
        "owner_has_heart_equip" => Ok(ModifierValueSource::OwnerHasHeartEquip),
        "alive_other_qun_count" => Ok(ModifierValueSource::AliveOtherQunCount),
        "owner_has_weapon" => Ok(ModifierValueSource::OwnerHasWeapon),
        "owner_no_weapon" => Ok(ModifierValueSource::OwnerNoWeapon),
        "owner_has_armor_equip" => Ok(ModifierValueSource::OwnerHasArmorEquip),
        "round_number_capped_five" => Ok(ModifierValueSource::RoundNumberCappedFive),
        "round_at_least_alive_qun_count" => Ok(ModifierValueSource::RoundAtLeastAliveQunCount),
        "round_below_alive_qun_count" => Ok(ModifierValueSource::RoundBelowAliveQunCount),
        "alive_field_rank_thirteen_player_count" => {
            Ok(ModifierValueSource::AliveFieldRankThirteenPlayerCount)
        }
        "field_mount_card_count_ceil_half" => Ok(ModifierValueSource::FieldMountCardCountCeilHalf),
        "owner_unique_max_hand" => Ok(ModifierValueSource::OwnerUniqueMaxHand),
        "owner_hand_gt_other" => Ok(ModifierValueSource::OwnerHandGreaterThanOther),
        "owner_hand_lt_other" => Ok(ModifierValueSource::OwnerHandLessThanOther),
        "owner_qun_other_shu" => Ok(ModifierValueSource::OwnerQunOtherShu),
        "owner_turned_over_diff_other" => Ok(ModifierValueSource::OwnerTurnedOverDiffOther),
        "owner_hand_equip_gte_other" => Ok(ModifierValueSource::OwnerHandEquipGteOther),
        "owner_sha_damage_dealt_zero_this_turn" => {
            Ok(ModifierValueSource::OwnerShaDamageDealtZeroThisTurn)
        }
        "queried_player_hand_gt_hp_signed" => Ok(ModifierValueSource::QueriedPlayerHandGtHpSigned),
        "owner_linked_diff_other" => Ok(ModifierValueSource::OwnerLinkedDiffOther),
        "queried_player_judge_nonempty_other_than_owner" => {
            Ok(ModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner)
        }
        "queried_player_linked_other_than_owner" => {
            Ok(ModifierValueSource::QueriedPlayerLinkedOtherThanOwner)
        }
        "queried_player_linked_first_living_skill_owner" => {
            Ok(ModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner)
        }
        "queried_player_other_than_owner" => Ok(ModifierValueSource::QueriedPlayerOtherThanOwner),
        "queried_player_wounded_in_owner_attack_range_other" => {
            Ok(ModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther)
        }
        "queried_source_enemy_owner_ally_target_other_than_owner" => {
            Ok(ModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner)
        }
        "owner_armor_empty" => Ok(ModifierValueSource::OwnerArmorEmpty),
        "owner_healthy" => Ok(ModifierValueSource::OwnerHealthy),
        "alive_wounded_count_zero_or_at_least_four" => {
            Ok(ModifierValueSource::AliveWoundedCountZeroOrAtLeastFour)
        }
        _ => Err(format!("unknown modifier value source {source}")),
    }
}

pub(crate) fn mod_value_source_tag(source: ModifierValueSource) -> String {
    match source {
        ModifierValueSource::QueriedPlayerMarkPositive(mark) => {
            return format!("queried_player_mark_positive_{mark}");
        }
        ModifierValueSource::OwnerMark(mark) => return format!("owner_mark_{mark}"),
        ModifierValueSource::OwnerSkillPileCount => {
            return format!("owner_skill_pile_count");
        }
        ModifierValueSource::OwnerSkillPileCountNegative(skill_def) => {
            return format!("owner_skill_pile_count_negative_{skill_def}");
        }
        ModifierValueSource::OwnerSkillPileCountOf(skill_def) => {
            return format!("owner_skill_pile_count_of_{skill_def}");
        }
        ModifierValueSource::AliveWoundedCountAtLeast(threshold) => {
            return format!("alive_wounded_count_at_least_{threshold}");
        }
        ModifierValueSource::OwnerEquipCountAtLeast(threshold) => {
            return format!("owner_equip_count_at_least_{threshold}");
        }
        ModifierValueSource::OwnerEquipCountFloor(threshold) => {
            return format!("owner_equip_count_floor_{threshold}");
        }
        ModifierValueSource::OwnerHp => "owner_hp",
        ModifierValueSource::OwnerMaxHp => "owner_max_hp",
        ModifierValueSource::OwnerHpMinusOne => "owner_hp_minus_one",
        ModifierValueSource::OwnerLostHp => "owner_lost_hp",
        ModifierValueSource::OwnerLostHpMinusOne => "owner_lost_hp_minus_one",
        ModifierValueSource::OwnerLostHpMinOne => "owner_lost_hp_min_one",
        ModifierValueSource::OwnerSkillMark => "owner_skill_mark",
        ModifierValueSource::OwnerSkillPileCount => "owner_skill_pile_count",
        ModifierValueSource::OwnerSkillStateU32 => "owner_skill_state_u32",
        ModifierValueSource::OwnerSkillStateU32Negative => "owner_skill_state_u32_negative",
        ModifierValueSource::QueriedPlayerQunOwnerLordAlive => {
            "queried_player_qun_owner_lord_alive"
        }
        ModifierValueSource::OwnerLinked => "owner_linked",
        ModifierValueSource::OwnerSkillUseCount => "owner_skill_use_count",
        ModifierValueSource::AliveWoundedOrGuimingWuCountMinOne => {
            "alive_wounded_or_guiming_wu_count_min_one"
        }
        ModifierValueSource::AliveGroupCount => "alive_group_count",
        ModifierValueSource::AliveFemaleCount => "alive_female_count",
        ModifierValueSource::AliveOtherCountCappedFour => "alive_other_count_capped_four",
        ModifierValueSource::OwnerNoMount => "owner_no_mount",
        ModifierValueSource::OwnerHandEmpty => "owner_hand_empty",
        ModifierValueSource::OwnerHasTreasure => "owner_has_treasure",
        ModifierValueSource::AliveMountedPlayerCountMinOne => "alive_mounted_player_count_min_one",
        ModifierValueSource::OwnerEquipCountOdd => "owner_equip_count_odd",
        ModifierValueSource::OwnerEquipCountEven => "owner_equip_count_even",
        ModifierValueSource::OwnerEquipCountHalfFloor => "owner_equip_count_half_floor",
        ModifierValueSource::OwnerEquipCountExactlyOne => "owner_equip_count_exactly_one",
        ModifierValueSource::OwnerEquipCount => "owner_equip_count",
        ModifierValueSource::OwnerHpGtTwo => "owner_hp_gt_two",
        ModifierValueSource::OwnerHpLeTwo => "owner_hp_le_two",
        ModifierValueSource::OwnerLostHpAtLeastTwo => "owner_lost_hp_at_least_two",
        ModifierValueSource::OwnerAllOthersInAttackRange => "owner_all_others_in_attack_range",
        ModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne => {
            "owner_current_turn_all_others_distance_at_most_one"
        }
        ModifierValueSource::OwnerShaUsedZero => "owner_sha_used_zero",
        ModifierValueSource::OwnerShaUsedNonzero => "owner_sha_used_nonzero",
        ModifierValueSource::OwnerShaUsedExactlyOne => "owner_sha_used_exactly_one",
        ModifierValueSource::OwnerShaUsedExactlyTwo => "owner_sha_used_exactly_two",
        ModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn => {
            "owner_max_hand_before_skill_fires_this_turn"
        }
        ModifierValueSource::OwnerNonDamageHandTypeCount => "owner_non_damage_hand_type_count",
        ModifierValueSource::OwnerShaHandCount => "owner_sha_hand_count",
        ModifierValueSource::OwnerTurnDamageDealt => "owner_turn_damage_dealt",
        ModifierValueSource::OwnerDistinctHandCardSuitsUsedThisTurn => {
            "owner_distinct_hand_card_suits_used_this_turn"
        }
        ModifierValueSource::OwnerCurrentTurnCardUseCount => "owner_current_turn_card_use_count",
        ModifierValueSource::OwnerPlayPhaseCardUseCountZero => {
            "owner_play_phase_card_use_count_zero"
        }
        ModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(limit) => {
            return format!("owner_skill_play_phase_fire_count_below_{limit}");
        }
        ModifierValueSource::OwnerHasHeartEquip => "owner_has_heart_equip",
        ModifierValueSource::AliveOtherQunCount => "alive_other_qun_count",
        ModifierValueSource::OwnerHasWeapon => "owner_has_weapon",
        ModifierValueSource::OwnerNoWeapon => "owner_no_weapon",
        ModifierValueSource::OwnerHasArmorEquip => "owner_has_armor_equip",
        ModifierValueSource::RoundNumberCappedFive => "round_number_capped_five",
        ModifierValueSource::RoundAtLeastAliveQunCount => "round_at_least_alive_qun_count",
        ModifierValueSource::RoundBelowAliveQunCount => "round_below_alive_qun_count",
        ModifierValueSource::AliveFieldRankThirteenPlayerCount => {
            "alive_field_rank_thirteen_player_count"
        }
        ModifierValueSource::FieldMountCardCountCeilHalf => "field_mount_card_count_ceil_half",
        ModifierValueSource::OwnerUniqueMaxHand => "owner_unique_max_hand",
        ModifierValueSource::OwnerHandGreaterThanOther => "owner_hand_gt_other",
        ModifierValueSource::OwnerHandLessThanOther => "owner_hand_lt_other",
        ModifierValueSource::OwnerQunOtherShu => "owner_qun_other_shu",
        ModifierValueSource::OwnerTurnedOverDiffOther => "owner_turned_over_diff_other",
        ModifierValueSource::OwnerHandEquipGteOther => "owner_hand_equip_gte_other",
        ModifierValueSource::OwnerShaDamageDealtZeroThisTurn => {
            "owner_sha_damage_dealt_zero_this_turn"
        }
        ModifierValueSource::QueriedPlayerHandGtHpSigned => "queried_player_hand_gt_hp_signed",
        ModifierValueSource::OwnerLinkedDiffOther => "owner_linked_diff_other",
        ModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner => {
            "queried_player_judge_nonempty_other_than_owner"
        }
        ModifierValueSource::QueriedPlayerLinkedOtherThanOwner => {
            "queried_player_linked_other_than_owner"
        }
        ModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner => {
            "queried_player_linked_first_living_skill_owner"
        }
        ModifierValueSource::QueriedPlayerOtherThanOwner => "queried_player_other_than_owner",
        ModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther => {
            "queried_player_wounded_in_owner_attack_range_other"
        }
        ModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner => {
            "queried_source_enemy_owner_ally_target_other_than_owner"
        }
        ModifierValueSource::OwnerArmorEmpty => "owner_armor_empty",
        ModifierValueSource::OwnerHealthy => "owner_healthy",
        ModifierValueSource::AliveWoundedCountZeroOrAtLeastFour => {
            "alive_wounded_count_zero_or_at_least_four"
        }
    }
    .into()
}

pub(crate) fn map_event(s: &str) -> Option<EventKind> {
    Some(match s {
        "game_started" => EventKind::GameStarted,
        "damage_before" => EventKind::DamageBefore,
        "damage_prevented" => EventKind::DamagePrevented,
        "damage" => EventKind::Damage,
        "hp_lost" => EventKind::HpLost,
        "card_used" | "use_card" => EventKind::CardUsed,
        "card_targeted" => EventKind::CardTargeted,
        "card_responded" | "respond" => EventKind::CardResponded,
        "card_resolved" => EventKind::CardResolved,
        "phase_prepare" => EventKind::PhasePrepare,
        "phase_begin" => EventKind::PhaseBegin,
        "phase_judge" => EventKind::PhaseJudge,
        "phase_draw" => EventKind::PhaseDraw,
        "phase_play" => EventKind::PhasePlay,
        "phase_discard" => EventKind::PhaseDiscard,
        "phase_end" => EventKind::PhaseEnd,
        "phase_draw_end" => EventKind::PhaseDrawEnd,
        "debate_resolved" => EventKind::DebateResolved,
        "debate_started" => EventKind::DebateStarted,
        "phase_play_end" => EventKind::PhasePlayEnd,
        "phase_discard_end" => EventKind::PhaseDiscardEnd,
        "phase_any_end" => EventKind::PhaseAnyEnd,
        "phase_draw_skipped" => EventKind::PhaseDrawSkipped,
        "phase_any_skipped" => EventKind::PhaseAnySkipped,
        "dying" => EventKind::Dying,
        "dying_after" => EventKind::DyingAfter,
        "max_hp_changed" => EventKind::MaxHpChanged,
        "death" => EventKind::Death,
        "cards_drawn" | "draw" => EventKind::CardsDrawn,
        "cards_moved" => EventKind::CardsMoved,
        "cards_given" => EventKind::CardsGiven,
        "skill_added" => EventKind::SkillAdded,
        "turn_ended" => EventKind::TurnEnded,
        "round_started" => EventKind::RoundStart,
        "round_ended" => EventKind::RoundEnd,
        "hp_recovered" => EventKind::HpRecovered,
        "recover_before" => EventKind::RecoverBefore,
        "duel_resolved" => EventKind::DuelResolved,
        "equip_slot_disabled" => EventKind::EquipSlotDisabled,
        "equip_slot_enabled" => EventKind::EquipSlotEnabled,
        "character_state_changed" => EventKind::CharacterStateChanged,
        "sha_dodged" => EventKind::ShaDodged,
        "sha_resolved" => EventKind::ShaResolved,
        "hand_compare_started" => EventKind::HandCompareStarted,
        "hand_compare_finished" => EventKind::HandCompareFinished,
        "judgment_resolved" => EventKind::JudgmentResolved,
        _ => return None,
    })
}

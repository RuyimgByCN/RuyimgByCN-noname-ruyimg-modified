//! Content registry operations for `valid_group_filter_tag` and related helpers.

use super::*;

pub(crate) fn valid_group_filter_tag(group: &str) -> bool {
    !group.is_empty()
        && group
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(crate) fn map_mod_target(s: &str, selector: Option<&str>) -> Option<ModifierTarget> {
    if selector.is_none() {
        if let Some(mark) = s.strip_prefix("judgment_rewrite_hand_equip_cost_mark_") {
            return mark
                .parse::<u32>()
                .ok()
                .filter(|mark| *mark > 0)
                .map(ModifierTarget::JudgmentRewriteHandEquipCostMark);
        }
    }
    if let Some(value) = s.strip_prefix("card_distance_unlimited_target_mark_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::CardDistanceUnlimitedTargetMark);
    }
    if let Some(value) = s.strip_prefix("card_distance_unlimited_skill_on_owner_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::CardDistanceUnlimitedSkillOnOwner);
    }
    if let Some(value) = s.strip_prefix("card_use_unrespondable_skill_on_owner_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::CardUseUnrespondableSkillOnOwner);
    }
    if let Some(value) = s.strip_prefix("sha_use_unlimited_target_mark_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::ShaUseUnlimitedTargetMark);
    }
    if s == "card_suit" {
        return Some(ModifierTarget::CardSuit(map_suit_selector(selector?)?));
    }
    if s == "card_number" && selector.is_none() {
        return Some(ModifierTarget::CardNumber);
    }
    if s == "card_name" {
        return Some(ModifierTarget::CardName(map_card_selector(selector?)?));
    }
    if s == "card_nature" {
        return Some(ModifierTarget::CardNatureFor(map_card_selector(selector?)?));
    }
    if s == "sha_unrespondable" {
        return Some(ModifierTarget::ShaUnrespondableFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_use_unrespondable" {
        return Some(ModifierTarget::CardUseUnrespondableFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "sha_damage" && selector.is_some() {
        return Some(ModifierTarget::ShaDamageFor(map_card_selector(selector?)?));
    }
    if s == "damage_taken_cancel" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::DamageTakenCancel,
            Some("normal") => ModifierTarget::DamageTakenCancelForNature(0),
            Some("fire") => ModifierTarget::DamageTakenCancelForNature(1),
            Some("thunder") => ModifierTarget::DamageTakenCancelForNature(2),
            _ => return None,
        });
    }
    if s == "damage_amount" && selector.is_some() {
        return Some(match selector {
            Some("any") => ModifierTarget::DamageAmount,
            Some("normal") => ModifierTarget::DamageAmountForNature(0),
            Some("fire") => ModifierTarget::DamageAmountForNature(1),
            Some("thunder") => ModifierTarget::DamageAmountForNature(2),
            Some("sha") => ModifierTarget::DamageAmountForSha,
            _ => return None,
        });
    }
    if s == "damage_taken_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::DamageTakenAmount,
            Some("normal") => ModifierTarget::DamageTakenAmountForNature(0),
            Some("fire") => ModifierTarget::DamageTakenAmountForNature(1),
            Some("thunder") => ModifierTarget::DamageTakenAmountForNature(2),
            _ => return None,
        });
    }
    if s == "global_damage_taken_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::GlobalDamageTakenAmount,
            Some("normal") => ModifierTarget::GlobalDamageTakenAmountForNature(0),
            Some("fire") => ModifierTarget::GlobalDamageTakenAmountForNature(1),
            Some("thunder") => ModifierTarget::GlobalDamageTakenAmountForNature(2),
            _ => return None,
        });
    }
    if s == "armor_damage_taken_amount" {
        return Some(match selector {
            Some("normal") => ModifierTarget::ArmorDamageTakenAmountForNature(0),
            Some("fire") => ModifierTarget::ArmorDamageTakenAmountForNature(1),
            Some("thunder") => ModifierTarget::ArmorDamageTakenAmountForNature(2),
            _ => return None,
        });
    }
    if s == "damage_taken_final_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::DamageTakenFinalAmount,
            Some("normal") => ModifierTarget::DamageTakenFinalAmountForNature(0),
            Some("fire") => ModifierTarget::DamageTakenFinalAmountForNature(1),
            Some("thunder") => ModifierTarget::DamageTakenFinalAmountForNature(2),
            _ => return None,
        });
    }
    if s == "global_damage_taken_final_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::GlobalDamageTakenFinalAmount,
            Some("normal") => ModifierTarget::GlobalDamageTakenFinalAmountForNature(0),
            Some("fire") => ModifierTarget::GlobalDamageTakenFinalAmountForNature(1),
            Some("thunder") => ModifierTarget::GlobalDamageTakenFinalAmountForNature(2),
            _ => return None,
        });
    }
    if s == "damage_taken_nature" && selector.is_none() {
        return Some(ModifierTarget::DamageTakenNature);
    }
    if s == "damage_as_hp_loss" && selector.is_some() {
        return match selector {
            Some("sha") => Some(ModifierTarget::DamageAsHpLossForSha),
            _ => None,
        };
    }
    if s == "card_use_cancel" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardUseCancelFor(map_card_selector(selector)?),
            None => ModifierTarget::CardUseCancel,
        });
    }
    if s == "card_response_cancel" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardResponseCancelFor(map_card_selector(selector)?),
            None => ModifierTarget::CardResponseCancel,
        });
    }
    if s == "card_save_cancel" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardSaveCancelFor(map_card_selector(selector)?),
            None => ModifierTarget::CardSaveCancel,
        });
    }
    if s == "card_ignored_hand_limit" {
        return Some(ModifierTarget::CardIgnoredHandLimit(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_distance_unlimited" {
        return Some(match selector {
            Some(selector) => {
                ModifierTarget::CardDistanceUnlimitedFor(map_card_selector(selector)?)
            }
            None => ModifierTarget::CardDistanceUnlimited,
        });
    }
    if s == "card_discard_protected" {
        return Some(ModifierTarget::CardDiscardProtectedFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_gain_protected" {
        return Some(ModifierTarget::CardGainProtectedFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_distance_unlimited_if_target_hand_gte_owner" {
        return Some(
            ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(map_card_selector(
                selector?,
            )?),
        );
    }
    if s == "card_use_unlimited" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardUseUnlimitedFor(map_card_selector(selector)?),
            None => ModifierTarget::CardUseUnlimited,
        });
    }
    if s == "first_hand_card_suit_use_unlimited" && selector.is_none() {
        return Some(ModifierTarget::FirstHandCardSuitUseUnlimited);
    }
    if s == "card_target_limit" {
        return Some(ModifierTarget::CardTargetLimitFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_distance_limit" {
        return Some(ModifierTarget::CardDistanceLimitFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_targets_all" {
        return Some(ModifierTarget::CardTargetsAllFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_target_cancel" {
        return Some(ModifierTarget::CardTargetCancelFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "armor_card_target_cancel" {
        return Some(ModifierTarget::ArmorCardTargetCancelFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_target_cancel_owner_turn_banned" {
        return Some(ModifierTarget::CardTargetCancelTurnBannedFor(
            map_card_selector("any").expect("any selector"),
        ));
    }
    if s == "card_target_cancel_from_current" {
        return Some(ModifierTarget::CardTargetCancelFromCurrentFor(
            map_card_selector(selector?)?,
        ));
    }
    if s == "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty" {
        return Some(
            ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(
                map_card_selector(selector?)?,
            ),
        );
    }
    if s == "card_target_cancel_if_other_user_distance_gt_one" {
        return Some(ModifierTarget::CardTargetCancelIfOtherUserDistanceGtOneFor(
            map_card_selector(selector?)?,
        ));
    }
    if selector.is_some() {
        return None;
    }
    Some(match s {
        "distance_from" => ModifierTarget::DistanceFrom,
        "distance_from_if_hp_gte_target" => ModifierTarget::DistanceFromHpGteTarget,
        "distance_from_skill_bound_target" => ModifierTarget::DistanceFromSkillBoundTarget,
        "link_loss_cancel" => ModifierTarget::LinkLossCancel,
        "distance_to" => ModifierTarget::DistanceTo,
        "global_distance_to" => ModifierTarget::GlobalDistanceTo,
        "nanman_damage_source_as_owner" => ModifierTarget::NanmanDamageSourceAsOwner,
        "attack_range" => ModifierTarget::AttackRange,
        "global_attack_range" => ModifierTarget::GlobalAttackRange,
        "hand_limit" => ModifierTarget::HandLimit,
        "global_hand_limit" => ModifierTarget::GlobalHandLimit,
        "global_draw_count" => ModifierTarget::GlobalDrawCount,
        "global_sha_use_limit" => ModifierTarget::GlobalShaUseLimit,
        "hand_limit_base" => ModifierTarget::HandLimitBase,
        "draw_count" | "draw_phase" => ModifierTarget::DrawCount,
        "optional_draw_count" => ModifierTarget::OptionalDrawCount,
        "sha_damage" => ModifierTarget::ShaDamage,
        "sha_damage_taken" => ModifierTarget::ShaDamageTaken,
        "sha_damage_if_outside_target_range" => ModifierTarget::ShaDamageIfOutsideTargetRange,
        "sha_damage_source_range_threshold" => ModifierTarget::ShaDamageSourceRangeThreshold,
        "sha_unrespondable_within_distance" => ModifierTarget::ShaUnrespondableWithinDistance,
        "sha_target_unrespondable" => ModifierTarget::ShaTargetUnrespondable,
        "sha_use_unrespondable_within_distance" => {
            ModifierTarget::ShaUseUnrespondableWithinDistance
        }
        "card_use_unrespondable_within_responder_to_owner_distance" => {
            ModifierTarget::CardUseUnrespondableWithinResponderToOwnerDistance
        }
        "card_use_unrespondable_within_owner_to_responder_distance" => {
            ModifierTarget::CardUseUnrespondableWithinOwnerToResponderDistance
        }
        "damage_taken_source_range_threshold" => ModifierTarget::DamageTakenSourceRangeThreshold,
        "skip_judge_phase" => ModifierTarget::SkipJudgePhase,
        "skip_discard_phase" => ModifierTarget::SkipDiscardPhase,
        "restore_skipped_phase" => ModifierTarget::RestoreSkippedPhase,
        "turn_face_down_cancel" => ModifierTarget::TurnFaceDownCancel,
        "turn_over_cancel" => ModifierTarget::TurnOverCancel,
        "discard_phase_includes_equip" => ModifierTarget::DiscardPhaseIncludesEquip,
        "observer_foreign_hand_faces_visible" => ModifierTarget::ObserverForeignHandFacesVisible,
        "observer_hidden_identities_visible" => ModifierTarget::ObserverHiddenIdentitiesVisible,
        "draw_phase_scry_four_split_two" => ModifierTarget::DrawPhaseScryFourSplitTwo,
        "initial_hand_count" => ModifierTarget::InitialHandCount,
        "sha_use_limit" => ModifierTarget::ShaUseLimit,
        "sha_target_limit" => ModifierTarget::ShaTargetLimit,
        "card_targets_self_only" => ModifierTarget::CardTargetsSelfOnly,
        "response_count" => ModifierTarget::ResponseCount,
        "duel_opponent_response_count" => ModifierTarget::DuelOpponentResponseCount,
        "skip_discard_if_no_sha" => ModifierTarget::SkipDiscardIfNoSha,
        "draw_replacement_damage" => ModifierTarget::DrawReplacementDamage,
        "prepare_scry_count" => ModifierTarget::PrepareScryCount,
        "judgment_rewrite" => ModifierTarget::JudgmentRewrite,
        "judgment_rewrite_hand_equip" => ModifierTarget::JudgmentRewriteHandEquip,
        "judgment_rewrite_red_hand_equip_expansion" => {
            ModifierTarget::JudgmentRewriteRedHandEquipExpansion
        }
        "judgment_rewrite_black_hand_equip" => ModifierTarget::JudgmentRewriteBlackHandEquip,
        "skills_disabled_when_hp_one" => ModifierTarget::SkillsDisabledWhenHpOne,
        "judgment_best_from_draw_pile" => ModifierTarget::JudgmentBestFromDrawPile,
        "virtual_bagua_if_armor_slot_empty" => ModifierTarget::VirtualBaguaIfArmorSlotEmpty,
        "sha_response_count_if_other_female" => ModifierTarget::ShaResponseCountIfOtherFemale,
        "sha_response_count_if_source_owner_target_distance_le_one" => {
            ModifierTarget::ShaResponseCountIfSourceOwnerTargetDistanceLeOne
        }
        "tao_use_save_cancel_for_non_dying_others_during_owner_turn" => {
            ModifierTarget::TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn
        }
        "tao_use_save_cancel_for_dying_beyond_distance" => {
            ModifierTarget::TaoUseSaveCancelForDyingBeyondDistance
        }
        "damage_amount" => ModifierTarget::DamageAmount,
        "damage_as_hp_loss" => ModifierTarget::DamageAsHpLoss,
        "damage_between_owner_and_other_as_hp_loss" => {
            ModifierTarget::DamageBetweenOwnerAndOtherAsHpLoss
        }
        "damage_nature" => ModifierTarget::DamageNature,
        "damage_cancel" => ModifierTarget::DamageCancel,
        "card_response_cancel" => ModifierTarget::CardResponseCancel,
        "immune_shunshou" => ModifierTarget::ImmuneShunshou,
        "immune_le" => ModifierTarget::ImmuneLe,
        "sha_use_cancel" => ModifierTarget::ShaUseCancel,
        "hand_limit_base_max_hp" => ModifierTarget::HandLimitBaseMaxHp,
        "sha_response_cancel" => ModifierTarget::ShaResponseCancel,
        "sha_ignored_hand_limit" => ModifierTarget::ShaIgnoredHandLimit,
        "hand_card_use_response_cancel" => ModifierTarget::HandCardUseResponseCancel,
        "hand_card_gained_this_turn_only" => ModifierTarget::HandCardGainedThisTurnOnly,
        "card_target_cancel_if_target_lacks_matching_suit" => {
            ModifierTarget::CardTargetCancelIfTargetLacksMatchingSuit
        }
        "card_target_cancel_if_other_user_hp_gte_target" => {
            ModifierTarget::CardTargetCancelIfOtherUserHpGteTarget
        }
        "card_target_cancel_if_other_user_card_use_count_lt_owner_mark" => {
            ModifierTarget::CardTargetCancelIfOtherUserCardUseCountLtOwnerMark
        }
        "ignore_armor" => ModifierTarget::IgnoreArmor,
        "armor_effect_disabled" => ModifierTarget::ArmorEffectDisabled,
        "armor_link_gain_cancel" => ModifierTarget::ArmorLinkGainCancel,
        "card_distance_unlimited_for_skill_bound_target" => {
            ModifierTarget::CardDistanceUnlimitedForSkillBoundTarget
        }
        "sha_use_unlimited_for_skill_bound_target" => {
            ModifierTarget::ShaUseUnlimitedForSkillBoundTarget
        }
        "ignore_armor_for_skill_bound_target" => ModifierTarget::IgnoreArmorForSkillBoundTarget,
        "card_action_cancel_for_skill_color_mask" => {
            ModifierTarget::CardActionCancelForSkillColorMask
        }
        "ignore_armor_for_skill_card_policy" => ModifierTarget::IgnoreArmorForSkillCardPolicy,
        "card_use_unrespondable_for_skill_card_policy" => {
            ModifierTarget::CardUseUnrespondableForSkillCardPolicy
        }
        "card_distance_unlimited_for_skill_card_policy" => {
            ModifierTarget::CardDistanceUnlimitedForSkillCardPolicy
        }
        "sha_distance_unlimited_if_rank_lt_skill_mark" => {
            ModifierTarget::ShaDistanceUnlimitedIfRankLtSkillMark
        }
        "sha_no_use_count_if_rank_gt_skill_mark" => ModifierTarget::ShaNoUseCountIfRankGtSkillMark,
        "sha_distance_unlimited_if_target_distance_lte_rank" => {
            ModifierTarget::ShaDistanceUnlimitedIfTargetDistanceLteRank
        }
        _ => return None,
    })
}

pub(crate) fn map_suit_selector(s: &str) -> Option<engine_api::Suit> {
    Some(match s {
        "spade" => engine_api::Suit::Spade,
        "heart" => engine_api::Suit::Heart,
        "club" => engine_api::Suit::Club,
        "diamond" => engine_api::Suit::Diamond,
        _ => return None,
    })
}

pub(crate) fn mod_suit_tag(suit: engine_api::Suit) -> &'static str {
    match suit {
        engine_api::Suit::None => "none",
        engine_api::Suit::Spade => "spade",
        engine_api::Suit::Heart => "heart",
        engine_api::Suit::Club => "club",
        engine_api::Suit::Diamond => "diamond",
    }
}

pub(crate) fn map_card_selector(s: &str) -> Option<crate::query::CardSelector> {
    use crate::query::CardSelector;
    Some(match s {
        "any" => CardSelector::Any,
        "sha" => CardSelector::Sha,
        "tao" => CardSelector::Tao,
        "bingliang" => CardSelector::Bingliang,
        "lebu" => CardSelector::Lebu,
        "basic" => CardSelector::Basic,
        "non_basic" => CardSelector::NonBasic,
        "trick" => CardSelector::Trick,
        "equip" => CardSelector::Equip,
        "spade" => CardSelector::Spade,
        "heart" => CardSelector::Heart,
        "club" => CardSelector::Club,
        "diamond" => CardSelector::Diamond,
        "red" => CardSelector::Red,
        "black" => CardSelector::Black,
        "juedou" => CardSelector::Juedou,
        "black_trick" => CardSelector::BlackTrick,
        "damage_trick" => CardSelector::DamageTrick,
        "sha_fire" => CardSelector::ShaFire,
        "huogong" => CardSelector::Huogong,
        "jiu" => CardSelector::Jiu,
        "physical_jiu" => CardSelector::PhysicalJiu,
        "sha_red" => CardSelector::ShaRed,
        "nanman" => CardSelector::Nanman,
        "ordinary_trick" => CardSelector::OrdinaryTrick,
        "sha_black" => CardSelector::ShaBlack,
        "sha_elemental" => CardSelector::ShaElemental,
        "equip_non_mount" => CardSelector::EquipNonMount,
        "armor_or_treasure" => CardSelector::ArmorOrTreasure,
        "wanjian" => CardSelector::Wanjian,
        "sha_normal" => CardSelector::ShaNormal,
        "sha_heart" => CardSelector::ShaHeart,
        "delayed_trick" => CardSelector::DelayedTrick,
        "wuxie" => CardSelector::Wuxie,
        "guohe" => CardSelector::Guohe,
        "shunshou" => CardSelector::Shunshou,
        "sha_thunder" => CardSelector::ShaThunder,
        "hand_rank_letter" => CardSelector::HandRankLetter,
        "sole_hand_card" => CardSelector::SoleHandCard,
        "sha_diamond" => CardSelector::ShaDiamond,
        "skill_tracked" => CardSelector::SkillTracked,
        "damage_card" => CardSelector::DamageCard,
        "face_up" => CardSelector::FaceUp,
        _ => return None,
    })
}

pub(crate) fn map_mod_mode(s: &str) -> Option<CombineMode> {
    Some(match s {
        "add" => CombineMode::Add,
        "override" => CombineMode::Override,
        "min" => CombineMode::Min,
        "max" => CombineMode::Max,
        "multiply" => CombineMode::Multiply,
        "veto" => CombineMode::Veto,
        _ => return None,
    })
}

/// Convenience: skill id used by official jianxiong sample.
pub fn jianxiong_skill_def() -> SkillDefId {
    SkillDefId::new(BOOTSTRAP_JIANXIONG)
}

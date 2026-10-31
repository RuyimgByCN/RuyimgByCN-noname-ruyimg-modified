//! 效果语句代码生成：通用效果原语（自 `effect_convert_current_sha_to_fire_lose_hp_if_damage` 分组拆出具名技能结算器之后）。

use super::*;
use crate::abi_versions::ADDITIVE_RULE_VM_ABI_VERSION_758;

impl Assembler {
    pub(super) fn emit_effect_convert_current_sha_to_fire_lose_hp_if_damage(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectConvertCurrentShaToFireLoseHpIfDamage { player, card, span } => {
                self.mark(*span, "convert_current_sha_to_fire_lose_hp_if_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_45);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CONVERT_CURRENT_SHA_TO_FIRE_LOSE_HP_IF_DAMAGE,
                ));
            }
            StmtIr::EffectChooseAddCurrentShaTarget { player, card, span } => {
                self.mark(*span, "choose_add_current_sha_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_45);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET));
            }
            StmtIr::EffectChooseAddCurrentShaTargetWithinDistance {
                player,
                card,
                max_distance,
                span,
            } => {
                self.mark(*span, "choose_add_current_sha_target_within_distance");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_277);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(max_distance)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET_WITHIN_DISTANCE,
                ));
            }
            StmtIr::EffectRefundCurrentShaUse { player, card, span } => {
                self.mark(*span, "refund_current_sha_use");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_278);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REFUND_CURRENT_SHA_USE));
            }
            StmtIr::EffectDamageThenDrawSelectedTargets {
                source,
                damage,
                draw,
                span,
            } => {
                self.mark(*span, "damage_then_draw_selected_targets");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_279);
                self.emit_expr(source)?;
                self.emit_expr(damage)?;
                self.emit_expr(draw)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DAMAGE_THEN_DRAW_SELECTED_TARGETS,
                ));
            }
            StmtIr::EffectChooseRepoluTargetsDraw { player, span } => {
                self.mark(*span, "choose_repolu_targets_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_291);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_REPOLU_TARGETS_DRAW));
            }
            StmtIr::EffectFinishCurrentPhaseAbortResolution { player, span } => {
                self.mark(*span, "finish_current_phase_abort_resolution");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_286);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::FINISH_CURRENT_PHASE_ABORT_RESOLUTION,
                ));
            }
            StmtIr::EffectGainTopCardToSkillPile { player, span } => {
                self.mark(*span, "gain_top_card_to_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_287);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_TOP_CARD_TO_SKILL_PILE));
            }
            StmtIr::EffectChooseDiscardSkillPileCardAddMarkedSkill {
                player,
                target,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_skill_pile_card_add_marked_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_287);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_SKILL_PILE_CARD_ADD_MARKED_SKILL,
                ));
            }
            StmtIr::EffectGainOneHandFromSelectedTargets { player, span } => {
                self.mark(*span, "gain_one_hand_from_selected_targets");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_288);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GAIN_ONE_HAND_FROM_SELECTED_TARGETS,
                ));
            }
            StmtIr::EffectChooseStoreHandShaInSkillPile { player, span } => {
                self.mark(*span, "choose_store_hand_sha_in_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_46);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_STORE_HAND_SHA_IN_SKILL_PILE,
                ));
            }
            StmtIr::EffectChooseSpendSkillPileForDyingJiu {
                player,
                dying,
                span,
            } => {
                self.mark(*span, "choose_spend_skill_pile_for_dying_jiu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_46);
                self.emit_expr(player)?;
                self.emit_expr(dying)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_SPEND_SKILL_PILE_FOR_DYING_JIU,
                ));
            }
            StmtIr::EffectChooseCompareSelectedHandCard {
                player,
                target,
                win_skill_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "choose_compare_selected_hand_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_47);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*win_skill_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD,
                ));
            }
            StmtIr::EffectChooseAddSkillBoundCardTarget { player, card, span } => {
                self.mark(*span, "choose_add_skill_bound_card_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_47);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ADD_SKILL_BOUND_CARD_TARGET,
                ));
            }
            StmtIr::EffectChooseDiscardMatchingEquipColorAddSkill {
                player,
                target,
                skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_matching_equip_color_add_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_48);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_MATCHING_EQUIP_COLOR_ADD_SKILL,
                ));
            }
            StmtIr::EffectUseVirtualShaFromCardColorLockSourceIfNoSha { player, span } => {
                self.mark(
                    *span,
                    "use_virtual_sha_from_card_color_lock_source_if_no_sha",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_48);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::USE_VIRTUAL_SHA_FROM_CARD_COLOR_LOCK_SOURCE_IF_NO_SHA,
                ));
            }
            StmtIr::EffectChooseCurrentCardUseEnhancements {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "choose_current_card_use_enhancements");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_49);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CURRENT_CARD_USE_ENHANCEMENTS,
                ));
            }
            StmtIr::EffectMakeCurrentCardUnrespondable {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "make_current_card_unrespondable");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_220);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::MAKE_CURRENT_CARD_UNRESPONDABLE));
            }
            StmtIr::EffectChooseDrawOrDiscardAddSkill {
                player,
                draw_skill_def,
                discard_skill_def,
                three_way,
                span,
            } => {
                self.mark(
                    *span,
                    if *three_way {
                        "choose_three_way_draw_or_discard_add_skill"
                    } else {
                        "choose_draw_or_discard_add_skill"
                    },
                );
                self.required_abi = self.required_abi.max(if *three_way {
                    ADDITIVE_RULE_VM_ABI_VERSION_315
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_50
                });
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*draw_skill_def as i32));
                self.ops.push(Op::ConstI32(*discard_skill_def as i32));
                self.ops.push(Op::HostEffect(if *three_way {
                    effect_tag::CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL
                } else {
                    effect_tag::CHOOSE_DRAW_OR_DISCARD_ADD_SKILL
                }));
            }
            StmtIr::EffectChooseDiscardedBlackBasicAsBingliang { player, span } => {
                self.mark(*span, "choose_discarded_black_basic_as_bingliang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_51);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARDED_BLACK_BASIC_AS_BINGLIANG,
                ));
            }
            StmtIr::EffectChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_judge_use_virtual_sha_draw_if_no_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_51);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_JUDGE_USE_VIRTUAL_SHA_DRAW_IF_NO_DAMAGE,
                ));
            }
            StmtIr::EffectDiscardSelectedRecordUniqueCategoryTarget {
                player,
                target,
                span,
            } => {
                self.mark(*span, "discard_selected_record_unique_category_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_52);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_SELECTED_RECORD_UNIQUE_CATEGORY_TARGET,
                ));
            }
            StmtIr::EffectChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                player,
                target,
                toggle_link,
                span,
            } => {
                self.mark(
                    *span,
                    if *toggle_link {
                        "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded"
                    } else {
                        "choose_target_draw_or_discard_by_skill_use_count_link_if_wounded"
                    },
                );
                self.required_abi = self.required_abi.max(if *toggle_link {
                    ADDITIVE_RULE_VM_ABI_VERSION_314
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_52
                });
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(if *toggle_link {
                    effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED
                } else {
                    effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_LINK_IF_WOUNDED
                }));
            }
            StmtIr::EffectChooseRevealOtherHandCardRememberCategory { player, span } => {
                self.mark(*span, "choose_reveal_other_hand_card_remember_category");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_53);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_REVEAL_OTHER_HAND_CARD_REMEMBER_CATEGORY,
                ));
            }
            StmtIr::EffectDrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                span,
            } => {
                self.mark(*span, "draw_then_give_exact_record_phase_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_53);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_GIVE_EXACT_RECORD_PHASE_TARGET,
                ));
            }
            StmtIr::EffectClearPhaseGiftTarget { player, span } => {
                self.mark(*span, "clear_phase_gift_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_53);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CLEAR_PHASE_GIFT_TARGET));
            }
            StmtIr::EffectChooseGainResolvedShaOrShanThenOptionalUseSha {
                player,
                sha_card,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_gain_resolved_sha_or_shan_then_optional_use_sha",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_54);
                self.emit_expr(player)?;
                self.emit_expr(sha_card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_RESOLVED_SHA_OR_SHAN_THEN_OPTIONAL_USE_SHA,
                ));
            }
            StmtIr::EffectSkipCurrentPhase { player, span } => {
                self.mark(*span, "skip_current_phase");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_55);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_CURRENT_PHASE));
            }
            StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamage {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_other_targets_by_skipped_phase_count_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_55);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE,
                ));
            }
            StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player,
                amount,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_other_targets_by_skipped_phase_count_damage_draw",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_758);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW,
                ));
            }
            StmtIr::EffectRevealUniformHandDrawSelfChooseOthers { player, span } => {
                self.mark(*span, "reveal_uniform_hand_draw_self_choose_others");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_56);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS,
                ));
            }
            StmtIr::EffectChooseCardUserOrAddTarget { player, card, span } => {
                self.mark(*span, "choose_card_user_or_add_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_57);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_CARD_USER_OR_ADD_TARGET));
            }
            StmtIr::EffectChooseDiscardEquipmentReduceCurrentDamage { player, span } => {
                self.mark(*span, "choose_discard_equipment_reduce_current_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_57);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_EQUIPMENT_REDUCE_CURRENT_DAMAGE,
                ));
            }
            StmtIr::EffectChooseDiscardEquipmentPreventCurrentDamage { player, span } => {
                self.mark(*span, "choose_discard_equipment_prevent_current_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_398);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_EQUIPMENT_PREVENT_CURRENT_DAMAGE,
                ));
            }
            StmtIr::EffectChooseMoveTargetEquipmentOrDraw {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_move_target_equipment_or_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_58);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_OR_DRAW,
                ));
            }
            StmtIr::EffectGiveAllEquipmentChooseRecoverOrDamage {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_equipment_choose_recover_or_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_58);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_ALL_EQUIPMENT_CHOOSE_RECOVER_OR_DAMAGE,
                ));
            }
            StmtIr::EffectChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_discard_weapon_range_hand_or_increase_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_59);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_WEAPON_RANGE_HAND_OR_INCREASE_DAMAGE,
                ));
            }
            StmtIr::EffectChooseHandCountDifferentTargetDrawOrRandomDiscard { player, span } => {
                self.mark(
                    *span,
                    "choose_hand_count_different_target_draw_or_random_discard",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_60);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_HAND_COUNT_DIFFERENT_TARGET_DRAW_OR_RANDOM_DISCARD,
                ));
            }
            StmtIr::EffectIncreaseCurrentDrawCount {
                player,
                count,
                span,
            } => {
                self.mark(*span, "increase_current_draw_count");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_61);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::INCREASE_CURRENT_DRAW_COUNT));
            }
            StmtIr::EffectChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, span } => {
                self.mark(
                    *span,
                    "choose_target_discard_excess_then_self_discard_or_target_draw",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_62);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW,
                ));
            }
            StmtIr::EffectDrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "draw_then_choose_gain_one_from_damage_target_and_neighbors",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_63);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_CHOOSE_GAIN_ONE_FROM_DAMAGE_TARGET_AND_NEIGHBORS,
                ));
            }
            StmtIr::EffectChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                player,
                amount,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_prevent_damage_or_lose_max_hp_increase_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_64);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE,
                ));
            }
            StmtIr::EffectRandomizeCurrentHandCompareOpponent { player, span } => {
                self.mark(*span, "randomize_current_hand_compare_opponent");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_65);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::RANDOMIZE_CURRENT_HAND_COMPARE_OPPONENT,
                ));
            }
            StmtIr::EffectGainHighestRankShaFromEventCards { player, span } => {
                self.mark(*span, "gain_highest_rank_sha_from_event_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_65);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GAIN_HIGHEST_RANK_SHA_FROM_EVENT_CARDS,
                ));
            }
            StmtIr::EffectGainHandCompareResultCard { player, span } => {
                self.mark(*span, "gain_hand_compare_result_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_66);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_HAND_COMPARE_RESULT_CARD));
            }
            StmtIr::EffectChooseAddOrRemoveCurrentCardTarget { player, card, span } => {
                self.mark(*span, "choose_add_or_remove_current_card_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_67);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ADD_OR_REMOVE_CURRENT_CARD_TARGET,
                ));
            }
            StmtIr::EffectChooseHandCompare {
                player,
                win_skill_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "choose_hand_compare");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_68);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*win_skill_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_HAND_COMPARE));
            }
            StmtIr::EffectChooseHandCompareGrantTargetOnWin {
                player,
                win_grant_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "choose_hand_compare_grant_target_on_win");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_678);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*win_grant_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_HAND_COMPARE_GRANT_TARGET_ON_WIN,
                ));
            }
            StmtIr::EffectExcludeSavedShaTarget { card, target, span } => {
                self.mark(*span, "exclude_current_sha_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EXCLUDE_SAVED_SHA_TARGET));
            }
            StmtIr::EffectBanUseAgainst { user, target, span } => {
                self.mark(*span, "ban_use_against");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_571);
                self.emit_expr(user)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::BAN_USE_AGAINST));
            }
            StmtIr::EffectAddFirstMissingSkill {
                player,
                skill_defs,
                span,
            } => {
                self.mark(*span, "add_first_missing_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_575);
                self.emit_expr(player)?;
                for skill_def in skill_defs {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::ADD_FIRST_MISSING_SKILL));
            }
            StmtIr::EffectRecordCurrentShaAllTargets { player, card, span } => {
                self.mark(*span, "record_current_sha_all_targets");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_580);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECORD_CURRENT_SHA_ALL_TARGETS));
            }
            StmtIr::EffectChooseGiveOrDiscardHand {
                player,
                payment_count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_give_or_discard_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_581);
                self.emit_expr(player)?;
                self.emit_expr(payment_count)?;
                self.ops.push(Op::ConstI32(scope.tag() as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GIVE_OR_DISCARD_HAND));
            }
            StmtIr::EffectChooseDiscardHandEquipThenRecoverTarget {
                chooser,
                target,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_recover_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_582);
                self.emit_expr(chooser)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER_TARGET,
                ));
            }
            StmtIr::EffectChooseOtherDamage {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_other_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_69);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DAMAGE));
            }
            StmtIr::EffectChooseOtherDiscardTargetCard {
                player,
                scope,
                span,
            } => {
                self.mark(*span, "choose_other_discard_target_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_70);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DISCARD_TARGET_CARD));
            }
            StmtIr::EffectGainDiscardCardByKind { player, kind, span } => {
                self.mark(*span, "gain_discard_card_by_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_74);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_DISCARD_CARD_BY_KIND));
            }
            StmtIr::EffectRevealUniformHandChooseDamage {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "reveal_uniform_hand_choose_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_76);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_UNIFORM_HAND_CHOOSE_DAMAGE,
                ));
            }
            StmtIr::EffectChooseGainDiscardCardByKind { player, kind, span } => {
                self.mark(*span, "choose_gain_discard_card_by_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_77);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_DISCARD_CARD_BY_KIND));
            }
            StmtIr::EffectDrawForLifetimeCardActionDivisors {
                player,
                divisors,
                span,
            } => {
                self.mark(*span, "draw_for_lifetime_card_action_divisors");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_78);
                self.emit_expr(player)?;
                for divisor in divisors {
                    self.ops.push(Op::ConstI32(*divisor as i32));
                }
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_FOR_LIFETIME_CARD_ACTION_DIVISORS,
                ));
            }
            StmtIr::EffectChooseAddMissingSkill {
                player,
                skill_defs,
                span,
            } => {
                self.mark(*span, "choose_add_missing_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_79);
                self.emit_expr(player)?;
                for skill_def in skill_defs {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ADD_MISSING_SKILL));
            }
            StmtIr::EffectChooseTargetsDraw {
                player,
                count,
                target_count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_targets_draw");
                self.required_abi = self.required_abi.max(match scope {
                    content_schema::DrawTargetScopeIr::HandLteChooser => {
                        ADDITIVE_RULE_VM_ABI_VERSION_338
                    }
                    content_schema::DrawTargetScopeIr::HandEmpty => {
                        ADDITIVE_RULE_VM_ABI_VERSION_162
                    }
                    content_schema::DrawTargetScopeIr::Wounded => ADDITIVE_RULE_VM_ABI_VERSION_195,
                    content_schema::DrawTargetScopeIr::SameHp => ADDITIVE_RULE_VM_ABI_VERSION_135,
                    content_schema::DrawTargetScopeIr::MinHand => ADDITIVE_RULE_VM_ABI_VERSION_83,
                    _ => ADDITIVE_RULE_VM_ABI_VERSION_80,
                });
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(i32::from(*target_count)));
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGETS_DRAW));
            }
            StmtIr::EffectChooseTargetsDamage {
                player,
                min_count,
                max_count,
                scope,
                amount,
                span,
            } => {
                self.mark(*span, "choose_targets_damage");
                self.required_abi = self.required_abi.max(match scope {
                    content_schema::DrawTargetScopeIr::HandEmpty => {
                        ADDITIVE_RULE_VM_ABI_VERSION_162
                    }
                    content_schema::DrawTargetScopeIr::Wounded => ADDITIVE_RULE_VM_ABI_VERSION_195,
                    _ => ADDITIVE_RULE_VM_ABI_VERSION_135,
                });
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*min_count)));
                self.ops.push(Op::ConstI32(i32::from(*max_count)));
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGETS_DAMAGE));
            }
            StmtIr::EffectChooseOptionalTargetsDraw {
                player,
                count,
                min_count,
                max_count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_optional_targets_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_195);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(i32::from(*min_count)));
                self.ops.push(Op::ConstI32(i32::from(*max_count)));
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OPTIONAL_TARGETS_DRAW));
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

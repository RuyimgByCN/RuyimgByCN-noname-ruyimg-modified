//! 效果语句代码生成：通用效果原语（自 `effect_gain_whole_skill_pile` 分组拆出具名技能结算器之后）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_gain_whole_skill_pile(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectGainWholeSkillPile {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "gain_whole_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_WHOLE_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_687);
            }
            StmtIr::EffectPlaceHandCardsToDeckTop {
                player,
                skill,
                count,
                span,
            } => {
                self.mark(*span, "place_hand_cards_to_deck_top");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_HAND_CARDS_TO_DECK_TOP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_689);
            }
            StmtIr::EffectRearrangePileToDeckTop {
                player,
                skill,
                pile_def,
                count,
                span,
            } => {
                self.mark(*span, "rearrange_pile_to_deck_top");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REARRANGE_PILE_TO_DECK_TOP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_690);
            }
            StmtIr::EffectChooseHandCardsToSkillPile {
                player,
                skill,
                min_count,
                max_count,
                span,
            } => {
                self.mark(*span, "choose_hand_cards_to_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(min_count)?;
                self.emit_expr(max_count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_HAND_CARDS_TO_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_691);
            }
            StmtIr::EffectChooseTargetHandCardToPile {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "choose_target_hand_card_to_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_HAND_CARD_TO_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_688);
            }
            StmtIr::EffectChooseOneFromSkillPileDiscard {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "choose_one_from_skill_pile_discard");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ONE_FROM_SKILL_PILE_DISCARD,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_686);
            }
            StmtIr::EffectPlaceTopDeckToExpansion {
                player,
                skill,
                pile_def,
                count,
                span,
            } => {
                self.mark(*span, "place_top_deck_to_expansion");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_TOP_DECK_TO_EXPANSION));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_685);
            }
            StmtIr::EffectDiscardSkillPile {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "discard_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_684);
            }
            StmtIr::EffectGainFromSkillPile {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "gain_from_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_FROM_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_651);
            }
            StmtIr::EffectPlaceHandCardToExpansion {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "place_hand_card_to_expansion");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_HAND_CARD_TO_EXPANSION));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_650);
            }
            StmtIr::EffectExtraResolveCurrentTrick {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "extra_resolve_current_trick");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EXTRA_RESOLVE_CURRENT_TRICK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_649);
            }
            StmtIr::EffectChooseClanNotMaxHandLimitGain {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_clan_not_max_hand_limit_gain");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CLAN_NOT_MAX_HAND_LIMIT_GAIN,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_648);
            }
            StmtIr::EffectGainTargetHandCard {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "gain_target_hand_card");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_TARGET_HAND_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_647);
            }
            StmtIr::EffectIncreaseCurrentShaBaseDamage {
                player,
                card,
                amount,
                span,
            } => {
                self.mark(*span, "increase_current_sha_base_damage");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::INCREASE_CURRENT_SHA_BASE_DAMAGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_99);
            }
            StmtIr::EffectSkipNextPlayPhase { player, span } => {
                self.mark(*span, "skip_next_play_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_PLAY_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_100);
            }
            StmtIr::EffectSkipNextJudgePhase { player, span } => {
                self.mark(*span, "skip_next_judge_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_JUDGE_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_228);
            }
            StmtIr::EffectSkipJudgeAndDrawPhases { player, span } => {
                self.mark(*span, "skip_judge_and_draw_phases");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_JUDGE_AND_DRAW_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_450);
            }
            StmtIr::EffectChooseDiscardDamageEndpointCardIncreaseIfSuit {
                player,
                target,
                suit,
                amount,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_damage_endpoint_card_increase_if_suit",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_DAMAGE_ENDPOINT_CARD_INCREASE_IF_SUIT,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_230);
            }
            StmtIr::EffectChooseOtherDrawOrRecover {
                player,
                draw_count,
                recover_count,
                times,
                span,
            } => {
                self.mark(*span, "choose_other_draw_or_recover");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(recover_count)?;
                self.emit_expr(times)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DRAW_OR_RECOVER));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_101);
            }
            StmtIr::EffectChooseOtherRecover {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_other_recover");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_RECOVER));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_160);
            }
            StmtIr::EffectEnableEquipSlot {
                player,
                slot,
                source,
                span,
            } => {
                self.mark(*span, "enable_equip_slot");
                self.emit_expr(player)?;
                self.emit_expr(slot)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::ENABLE_EQUIP_SLOT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_161);
            }
            StmtIr::EffectChooseDiscardHandEquipThenRemoveSelfSkill {
                player,
                count,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_remove_self_skill");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_REMOVE_SELF_SKILL,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_102);
            }
            StmtIr::EffectResetLinkedSelectedTargets { source, span } => {
                self.mark(*span, "reset_linked_selected_targets");
                self.emit_expr(source)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESET_LINKED_SELECTED_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_719);
            }
            StmtIr::EffectDrawSelectedTargets { count, span } => {
                self.mark(*span, "draw_selected_targets");
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_SELECTED_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_103);
            }
            StmtIr::EffectRecoverIfMinCurrentHp {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "recover_if_min_current_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECOVER_IF_MIN_CURRENT_HP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_103);
            }
            StmtIr::EffectIncreaseCurrentCardBaseDamage {
                player,
                card,
                amount,
                span,
            } => {
                self.mark(*span, "increase_current_card_base_damage");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::INCREASE_CURRENT_CARD_BASE_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_104);
            }
            StmtIr::EffectModifyCurrentDamage {
                player,
                delta,
                span,
            } => {
                self.mark(*span, "modify_current_damage");
                self.emit_expr(player)?;
                self.emit_expr(delta)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::MODIFY_CURRENT_DAMAGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_562);
            }
            StmtIr::EffectLoseHpThenDamageOtherLoseMaxHpIfDying {
                player,
                target,
                amount,
                skill,
                span,
            } => {
                self.mark(*span, "lose_hp_then_damage_other_lose_max_hp_if_dying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::LOSE_HP_THEN_DAMAGE_OTHER_LOSE_MAX_HP_IF_DYING,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_108);
            }
            StmtIr::EffectRecoverSelectedTargets { amount, span } => {
                self.mark(*span, "recover_selected_targets");
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECOVER_SELECTED_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_109);
            }
            StmtIr::EffectRecordSelectedPlayPhaseCategory { skill, span } => {
                self.mark(*span, "record_selected_play_phase_category");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::RECORD_SELECTED_PLAY_PHASE_CATEGORY,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_110);
            }
            StmtIr::EffectTakeTargetHandThenRepeatUntilMax {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(*span, "take_target_hand_then_repeat_until_max");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TAKE_TARGET_HAND_THEN_REPEAT_UNTIL_MAX,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_113);
            }
            StmtIr::EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                player,
                card,
                draw_count,
                damage_amount,
                mark_def,
                mark_count,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_draw_or_increase_current_card_damage_or_both_mark",
                );
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(damage_amount)?;
                self.ops.push(Op::ConstU32(*mark_def));
                self.emit_expr(mark_count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_114);
            }
            StmtIr::EffectDrawThenChooseGiveOrDiscardHand {
                player,
                draw_count,
                payment_count,
                span,
            } => {
                self.mark(*span, "draw_then_choose_give_or_discard_hand");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(payment_count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_116);
            }
            StmtIr::EffectGainTargetEquipment {
                player,
                target,
                slot,
                span,
            } => {
                self.mark(*span, "gain_target_equipment");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*slot)));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_TARGET_EQUIPMENT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_117);
            }
            StmtIr::EffectDiscardAllTargetEquipment {
                actor,
                target,
                span,
            } => {
                self.mark(*span, "discard_all_target_equipment");
                self.emit_expr(actor)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_TARGET_EQUIPMENT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_118);
            }
            StmtIr::EffectIncreaseCurrentShaTargetDamage {
                source,
                card,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "increase_current_sha_target_damage");
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::INCREASE_CURRENT_SHA_TARGET_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_119);
            }
            StmtIr::EffectChooseTargetDrawThenOwnerIfBelowMaxHp {
                player,
                count,
                span,
            } => {
                self.mark(*span, "choose_target_draw_then_owner_if_below_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DRAW_THEN_OWNER_IF_BELOW_MAX_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_120);
            }
            StmtIr::EffectChooseBoundSourceAttackRangeDamage {
                player,
                source,
                amount,
                span,
            } => {
                self.mark(*span, "choose_bound_source_attack_range_damage");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_BOUND_SOURCE_ATTACK_RANGE_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_121);
            }
            StmtIr::EffectChooseTargetDrawMaxHpCapThenDiscardExcess { player, cap, span } => {
                self.mark(*span, "choose_target_draw_max_hp_cap_then_discard_excess");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*cap as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_DISCARD_EXCESS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_122);
            }
            StmtIr::EffectChooseTargetDrawMaxHpCapThenAddSkill {
                player,
                cap,
                skill_def,
                span,
            } => {
                self.mark(*span, "choose_target_draw_to_max_hp_cap_then_add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*cap as i32));
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_455);
            }
            StmtIr::EffectDiscardHandEquipCount {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand_equip_count");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*count as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_HAND_EQUIP_COUNT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_460);
            }
            StmtIr::EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
                player,
                reveal_count,
                recipient_cap,
                span,
            } => {
                self.mark(
                    *span,
                    "reveal_choose_category_distribute_self_threshold_hp_loss",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*reveal_count)));
                self.ops.push(Op::ConstI32(i32::from(*recipient_cap)));
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_CHOOSE_CATEGORY_DISTRIBUTE_SELF_THRESHOLD_HP_LOSS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_491);
            }
            StmtIr::EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                player,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_other_random_gain_region_then_draw_compare_color_lose_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_RANDOM_GAIN_REGION_THEN_DRAW_COMPARE_COLOR_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_492);
            }
            StmtIr::EffectChooseGainTargetHandCeilHalfDifference {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_gain_target_hand_ceil_half_difference");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_TARGET_HAND_CEIL_HALF_DIFFERENCE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_494);
            }
            StmtIr::EffectChooseDiscardHandRange {
                player,
                min,
                max,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_range");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*min as i32));
                self.ops.push(Op::ConstI32(*max as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_HAND_RANGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_596);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

//! 效果语句代码生成：通用效果原语（自 `effect_resolve_wdb_yangyi` 分组拆出具名技能结算器之后）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_resolve_wdb_yangyi(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectGuozhanUseZhulianBihe { player, span } => {
                self.mark(*span, "guozhan_use_zhulian_bihe");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GUOZHAN_USE_ZHULIAN_BIHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_641);
            }
            StmtIr::EffectCancelIncomingCardUse {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "cancel_incoming_card_use");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CANCEL_INCOMING_CARD_USE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_643);
            }
            StmtIr::EffectChooseOtherGainOneOfTwoSkills {
                player,
                target,
                first_def,
                second_def,
                span,
            } => {
                self.mark(*span, "choose_other_gain_one_of_two_skills");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*first_def));
                self.ops.push(Op::ConstU32(*second_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_644);
            }
            StmtIr::EffectGwyewuRandomDiscardChain {
                player,
                wuyao_def,
                lang_def,
                span,
            } => {
                self.mark(*span, "gwyewu_random_discard_chain");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*wuyao_def));
                self.ops.push(Op::ConstU32(*lang_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::GWYEWU_RANDOM_DISCARD_CHAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_645);
            }
            StmtIr::EffectGiveSelectedThenTargetMayUseDrawOwner {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_then_target_may_use_draw_owner");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_TARGET_MAY_USE_DRAW_OWNER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_497);
            }
            StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_category_then_target_give_matching_or_damage_plus_one",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_DAMAGE_PLUS_ONE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_498);
            }
            StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip",
                );
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_EXCLUDING_SOURCE_RANDOM_GAIN_HAND_EQUIP_THEN_USE_IF_EQUIP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_499);
            }
            StmtIr::EffectSkipNextDiscardPhase { player, span } => {
                self.mark(*span, "skip_next_discard_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_DISCARD_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_471);
            }
            StmtIr::EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "give_selected_then_target_choose_discard_same_suit_or_lose_hp",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_123);
            }
            StmtIr::EffectGainEquipmentFromDrawPile { player, span } => {
                self.mark(*span, "gain_equipment_from_draw_pile");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_EQUIPMENT_FROM_DRAW_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_124);
            }
            StmtIr::EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "target_choose_suit_gain_owner_hand_damage_if_mismatch",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_168);
            }
            StmtIr::EffectJudgeSourceDiscardTwoHandOrDamage {
                player,
                source,
                span,
            } => {
                self.mark(*span, "judge_source_discard_two_hand_or_damage");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_171);
            }
            StmtIr::EffectGiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_recover_after_phase_total_two");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_172);
            }
            StmtIr::EffectChooseOtherHandTargetsGainOneReplaceDraw { player, span } => {
                self.mark(*span, "choose_other_hand_targets_gain_one_replace_draw");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_173);
            }
            StmtIr::EffectChooseOtherHandTargetsGainOneReduceDraw { player, span } => {
                self.mark(*span, "choose_other_hand_targets_gain_one_reduce_draw");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_311);
            }
            StmtIr::EffectDistributeTopTwoCards { player, span } => {
                self.mark(*span, "distribute_top_two_cards");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISTRIBUTE_TOP_TWO_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_174);
            }
            StmtIr::EffectGiveAllHandRecoverToOne {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand_recover_to_one");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GIVE_ALL_HAND_RECOVER_TO_ONE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_125);
            }
            StmtIr::EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { player, span } => {
                self.mark(
                    *span,
                    "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_127);
            }
            StmtIr::EffectChooseGiveOneHandEquipOrRecover {
                owner,
                giver,
                target,
                span,
            } => {
                self.mark(*span, "choose_give_one_hand_equip_or_recover");
                self.emit_expr(owner)?;
                self.emit_expr(giver)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_129);
            }
            StmtIr::EffectRemoveSkillByDef {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "remove_skill_by_def");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::REMOVE_SKILL_BY_DEF));
            }
            StmtIr::EffectChooseDiscardHandAddSkillByColor {
                player,
                red_skill_def,
                black_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_add_skill_by_color");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*red_skill_def));
                self.ops.push(Op::ConstU32(*black_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_ADD_SKILL_BY_COLOR,
                ));
            }
            StmtIr::EffectRevealTopGainNonBasicOrSwapBasic { player, span } => {
                self.mark(*span, "reveal_top_gain_non_basic_or_swap_basic");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC,
                ));
            }
            StmtIr::EffectRevealTopGainIfColor {
                player,
                color_mask,
                span,
            } => {
                self.mark(*span, "reveal_top_gain_if_color");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*color_mask)));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_TOP_GAIN_IF_COLOR));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_352);
            }
            StmtIr::EffectPreventCurrentDamage { span } => {
                self.mark(*span, "prevent_current_damage");
                self.ops
                    .push(Op::HostEffect(effect_tag::PREVENT_CURRENT_DAMAGE));
            }
            StmtIr::EffectJudgeCancelCurrentShaTargetOnRed {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "judge_cancel_current_sha_target_on_red");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_CANCEL_CURRENT_SHA_TARGET_ON_RED,
                ));
            }
            StmtIr::EffectChooseDrawOrRecover {
                player,
                draw_count,
                recover_count,
                allow_cancel,
                span,
            } => {
                self.mark(*span, "choose_draw_or_recover");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(recover_count)?;
                self.emit_expr(allow_cancel)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DRAW_OR_RECOVER));
            }
            StmtIr::EffectChooseUseVirtualSha {
                player,
                ignore_distance,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_sha");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*ignore_distance)));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_SHA));
            }
            StmtIr::EffectTakeHand { from, to, span } => {
                self.mark(*span, "take_hand");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::HostEffect(effect_tag::TAKE_HAND));
            }
            StmtIr::EffectLoseHp {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "lose_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::LOSE_HP));
            }
            StmtIr::EffectAddSkill {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::ADD_SKILL));
            }
            StmtIr::EffectAddBoundSkill {
                player,
                skill_def,
                target,
                span,
            } => {
                self.mark(*span, "add_bound_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::ADD_BOUND_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_130);
            }
            StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                player,
                skill,
                skill_def,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "replace_skill_with_bound_after_resolution_if_target_alive",
                );
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_132);
            }
            StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { player, span } => {
                self.mark(
                    *span,
                    "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_133);
            }
            StmtIr::EffectChooseDiscardHandEquipThenRecover {
                player,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_recover");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_134);
            }
            StmtIr::EffectRemoveSkill { skill, span } => {
                self.mark(*span, "remove_skill");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_SKILL));
            }
            StmtIr::EffectRefreshSkill { skill, span } => {
                self.mark(*span, "refresh_skill");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::REFRESH_SKILL));
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

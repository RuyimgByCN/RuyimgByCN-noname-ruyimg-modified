//! 效果语句代码生成：通用效果原语（自 `effect_ban_active_until_turn_end` 分组拆出具名技能结算器之后）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_ban_active_until_turn_end(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectBanActiveUntilTurnEnd {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "ban_active_until_turn_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::BAN_ACTIVE_UNTIL_TURN_END));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_400);
            }
            StmtIr::EffectSpwanweiActive {
                player,
                target,
                span,
            } => {
                self.mark(*span, "spwanwei_active");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::SPWANWEI_ACTIVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_401);
            }
            StmtIr::EffectSpwanweiDying {
                player,
                target,
                span,
            } => {
                self.mark(*span, "spwanwei_dying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::SPWANWEI_DYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_401);
            }
            StmtIr::EffectEquipWeaponFromDrawPile { player, span } => {
                self.mark(*span, "equip_weapon_from_draw_pile");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EQUIP_WEAPON_FROM_DRAW_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_366);
            }
            StmtIr::EffectSetSkillContentFlag {
                player,
                bit,
                value,
                span,
            } => {
                self.mark(*span, "set_skill_content_flag");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*bit)));
                self.ops.push(Op::ConstI32(i32::from(*value)));
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_SKILL_CONTENT_FLAG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
            }
            StmtIr::EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_reclaim_equipment_draw_if_range_reduced",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_97);
            }
            StmtIr::EffectChooseUseVirtualCard {
                player,
                kind,
                skill,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_card");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_98);
            }
            StmtIr::EffectChooseUseVirtualNanman {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_nanman");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_NANMAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_604);
            }
            StmtIr::EffectDiscardAllOwnJudge {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_all_own_judge");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_OWN_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_606);
            }
            StmtIr::EffectStoreEventCardInSkillPile {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "store_event_card_in_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::STORE_EVENT_CARD_IN_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_607);
            }
            StmtIr::EffectChooseTargetGainMaxHp {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_target_gain_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_GAIN_MAX_HP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_646);
            }
            StmtIr::EffectDiscardBasicDrawEqual {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_basic_draw_equal");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_BASIC_DRAW_EQUAL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_657);
            }
            StmtIr::EffectStartDebateAllOthers {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "start_debate_all_others");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::START_DEBATE_ALL_OTHERS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_655);
            }
            StmtIr::EffectStartDebateTargets {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "start_debate_targets");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::START_DEBATE_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_656);
            }
            StmtIr::EffectPinDebateOpinions {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "pin_debate_opinions");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PIN_DEBATE_OPINIONS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_658);
            }
            StmtIr::EffectAlignDebateOpinions {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "align_debate_opinions");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::ALIGN_DEBATE_OPINIONS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_659);
            }
            StmtIr::EffectGainShadowCards {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "gain_shadow_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::GAIN_SHADOW_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_660);
            }
            StmtIr::EffectChangeGroup {
                player,
                group,
                skill,
                span,
            } => {
                self.mark(*span, "change_group");
                self.emit_expr(player)?;
                let code = match group.as_str() {
                    "wei" => 0,
                    "shu" => 1,
                    "wu" => 2,
                    "qun" => 3,
                    "jin" => 4,
                    _ => 0,
                };
                self.ops.push(Op::ConstI32(code));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CHANGE_GROUP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_661);
            }
            StmtIr::EffectDistributeShadowCards {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "distribute_shadow_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISTRIBUTE_SHADOW_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_662);
            }
            StmtIr::EffectRegisterTurnEndDebate {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "register_turn_end_debate");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REGISTER_TURN_END_DEBATE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_663);
            }
            StmtIr::EffectChooseGroup {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_group");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CHOOSE_GROUP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_664);
            }
            StmtIr::EffectChooseGainOtherSkill {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_gain_other_skill");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_OTHER_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_675);
            }
            StmtIr::EffectLoseGainedSkill {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "lose_gained_skill");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::LOSE_GAINED_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_676);
            }
            StmtIr::EffectPlaceXumou {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "place_xumou");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::PLACE_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_665);
            }
            StmtIr::EffectPlaceDeckTopXumou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "place_deck_top_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_DECK_TOP_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectChoosePlaceHandXumou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_place_hand_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_PLACE_HAND_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectDiscardOneOwnXumou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_one_own_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ONE_OWN_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectPlaceDeckTopExpansion {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "place_deck_top_expansion");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*count));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_DECK_TOP_EXPANSION));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_667);
            }
            StmtIr::EffectCityConsume {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "city_consume");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CITY_CONSUME));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_668);
            }
            StmtIr::EffectQueueExtraTurn {
                player,
                check,
                skill,
                span,
            } => {
                self.mark(*span, "queue_extra_turn");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*check));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QUEUE_EXTRA_TURN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_669);
            }
            StmtIr::EffectChooseOthersInsertExtraTurn {
                player,
                check,
                skill,
                span,
            } => {
                self.mark(*span, "choose_others_insert_extra_turn");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*check));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHERS_INSERT_EXTRA_TURN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_671);
            }
            StmtIr::EffectQueueExtraPhases {
                player,
                phases_mask,
                skill,
                span,
            } => {
                self.mark(*span, "queue_extra_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*phases_mask));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::QUEUE_EXTRA_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_672);
            }
            StmtIr::EffectSkipRemainingPhases {
                player,
                phases_mask,
                skill,
                span,
            } => {
                self.mark(*span, "skip_remaining_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*phases_mask));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_REMAINING_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_674);
            }
            StmtIr::EffectChooseOthersAddMark {
                player,
                mark,
                delta,
                max_count,
                skill,
                span,
            } => {
                self.mark(*span, "choose_others_add_mark");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(*delta));
                self.ops.push(Op::ConstU32(*max_count));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHERS_ADD_MARK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_670);
            }
            StmtIr::EffectChooseOtherAddMark {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "choose_other_add_mark");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(*delta));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_ADD_MARK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_652);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

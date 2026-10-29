//! 效果语句代码生成：通用效果原语（自 `effect_set_draw_phase_from_bottom` 分组拆出具名技能结算器之后）。

use super::*;
use crate::abi_versions::ADDITIVE_RULE_VM_ABI_VERSION_756;

impl Assembler {
    pub(super) fn emit_effect_set_draw_phase_from_bottom(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectSetDrawPhaseFromBottom { player, span } => {
                self.mark(*span, "set_draw_phase_bottom");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_DRAW_PHASE_FROM_BOTTOM));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectDrawFromBottom {
                player,
                count,
                span,
            } => {
                self.mark(*span, "draw_cards_from_bottom");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_CARDS_FROM_BOTTOM));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectDraw {
                player,
                count,
                span,
            } => {
                self.mark(*span, "draw_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_CARDS));
            }
            StmtIr::EffectRecover {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "recover_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RECOVER_HP));
            }
            StmtIr::EffectNote { code, span } => {
                self.mark(*span, "emit_note");
                self.ops.push(Op::ConstI32(*code));
                self.ops.push(Op::HostEffect(effect_tag::EMIT_NOTE));
            }
            StmtIr::EffectDamage {
                source,
                target,
                amount,
                nature,
                span,
            } => {
                self.mark(*span, "deal_damage");
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                if *nature == 0 {
                    self.ops.push(Op::HostEffect(effect_tag::DEAL_DAMAGE));
                } else {
                    self.ops.push(Op::ConstI32(i32::from(*nature)));
                    self.ops
                        .push(Op::HostEffect(effect_tag::DEAL_NATURE_DAMAGE));
                }
            }
            StmtIr::EffectChangeMaxHp {
                player,
                delta,
                span,
            } => {
                self.mark(*span, "change_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(delta)?;
                self.ops.push(Op::HostEffect(effect_tag::CHANGE_MAX_HP));
            }
            StmtIr::EffectAddMark {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "add_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.ops.push(Op::ConstI32(*delta));
                self.ops.push(Op::HostEffect(effect_tag::ADD_MARK));
            }
            StmtIr::EffectAddMarkExpr {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "add_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.emit_expr(delta)?;
                self.ops.push(Op::HostEffect(effect_tag::ADD_MARK));
            }
            StmtIr::EffectRemoveMark {
                player,
                mark,
                amount,
                span,
            } => {
                self.mark(*span, "remove_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.ops.push(Op::ConstI32(*amount));
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_MARK));
            }
            StmtIr::EffectRemoveMarkExpr {
                player,
                mark,
                amount,
                span,
            } => {
                self.mark(*span, "remove_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_MARK));
            }
            StmtIr::EffectDiscardHand {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*count));
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_HAND));
            }
            StmtIr::EffectDiscardHandExpr {
                player,
                count,
                span,
            } => {
                // Same HostEffect op as literal-count discard_hand; the count is
                // pushed by an ordinary expression, so no new VM capability is
                // required and the ABI stays unchanged.
                self.mark(*span, "discard_hand_expr");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_HAND));
            }
            StmtIr::EffectDiscardHandEquipExpr {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand_equip_expr");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_HAND_EQUIP));
            }
            StmtIr::EffectDiscardHandEquip {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_350);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*count));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_HAND_EQUIP));
            }
            StmtIr::EffectAdjustHandTo {
                player,
                count,
                span,
            } => {
                self.mark(*span, "adjust_hand_to");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_9);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::ADJUST_HAND_TO));
            }
            StmtIr::EffectDiscardSelected { player, span } => {
                self.mark(*span, "discard_selected");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_SELECTED));
            }
            StmtIr::EffectRecastSelected { player, span } => {
                self.mark(*span, "recast_selected");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_16);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RECAST_SELECTED));
            }
            StmtIr::EffectDisableAllEquipSlots { player, span } => {
                self.mark(*span, "disable_all_equip_slots");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_18);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISABLE_ALL_EQUIP_SLOTS));
            }
            StmtIr::EffectMoveEquipmentThenDraw { player, span } => {
                self.mark(*span, "move_equipment_then_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_30);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::MOVE_EQUIPMENT_THEN_DRAW));
            }
            StmtIr::EffectChooseUseVirtualShaOrDraw { player, span } => {
                self.mark(*span, "choose_use_virtual_sha_or_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_31);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_SHA_OR_DRAW));
            }
            StmtIr::EffectDiscardEquipmentPairWithArmor { player, span } => {
                self.mark(*span, "discard_equipment_pair_with_armor");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_32);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_EQUIPMENT_PAIR_WITH_ARMOR,
                ));
            }
            StmtIr::EffectDiscardTargetEquipmentGainMount {
                player,
                target,
                span,
            } => {
                self.mark(*span, "discard_target_equipment_gain_mount");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_33);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_TARGET_EQUIPMENT_GAIN_MOUNT,
                ));
            }
            StmtIr::EffectDiscardTargetEquipmentDrawWeaponRange {
                player,
                target,
                span,
            } => {
                self.mark(*span, "discard_target_equipment_draw_weapon_range");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_136);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_TARGET_EQUIPMENT_DRAW_WEAPON_RANGE,
                ));
            }
            StmtIr::EffectChooseTargetRevealHandDamageIfSha {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_target_reveal_hand_damage_if_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_137);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_REVEAL_HAND_DAMAGE_IF_SHA,
                ));
            }
            StmtIr::EffectChooseGiveOrRecastOneHand {
                player,
                require_skill_def,
                grant_skill_def,
                grant_mark_def,
                span,
            } => {
                self.mark(*span, "choose_give_or_recast_one_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_756);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*require_skill_def));
                self.ops.push(Op::ConstU32(*grant_skill_def));
                self.ops.push(Op::ConstU32(*grant_mark_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GIVE_OR_RECAST_ONE_HAND));
            }
            StmtIr::EffectChooseOtherGiveOneHandEquip { player, span } => {
                self.mark(*span, "choose_other_give_one_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_138);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_GIVE_ONE_HAND_EQUIP));
            }
            StmtIr::EffectChooseRevealHandCardBindCategoryTarget {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_reveal_hand_card_bind_category_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_140);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_REVEAL_HAND_CARD_BIND_CATEGORY_TARGET,
                ));
            }
            StmtIr::EffectRecoverToMaxThenOptionalExchangeSkill {
                player,
                removable_skill_def,
                span,
            } => {
                self.mark(*span, "recover_to_max_then_optional_exchange_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_142);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*removable_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::RECOVER_TO_MAX_THEN_OPTIONAL_EXCHANGE_SKILL,
                ));
            }
            StmtIr::EffectRevealHandDiscardKind { player, kind, span } => {
                self.mark(*span, "reveal_hand_discard_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_144);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_HAND_DISCARD_KIND));
            }
            StmtIr::EffectChooseMoveTargetEquipmentAndOrDraw {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_move_target_equipment_and_or_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_146);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_AND_OR_DRAW,
                ));
            }
            StmtIr::EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_147);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_ATTACK_RANGE_CARDS_OR_INCREASE_DAMAGE_WEAPON_FORCES_BOTH,
                ));
            }
            StmtIr::EffectChooseDrawOrRecastAddSkill {
                player,
                draw_skill_def,
                recast_skill_def,
                span,
            } => {
                self.mark(*span, "choose_draw_or_recast_add_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_148);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*draw_skill_def as i32));
                self.ops.push(Op::ConstI32(*recast_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DRAW_OR_RECAST_ADD_SKILL));
            }
            StmtIr::EffectChooseGainShaForOpeningExtraPlay { player, span } => {
                self.mark(*span, "choose_gain_sha_for_opening_extra_play");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_149);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_SHA_FOR_OPENING_EXTRA_PLAY,
                ));
            }
            StmtIr::EffectFinishOpeningExtraPlayShaGain { player, span } => {
                self.mark(*span, "finish_opening_extra_play_sha_gain");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_149);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::FINISH_OPENING_EXTRA_PLAY_SHA_GAIN,
                ));
            }
            StmtIr::EffectRevealTargetHandGainShanArmorOrDrawTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "reveal_target_hand_gain_shan_armor_or_draw_two");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_150);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TARGET_HAND_GAIN_SHAN_ARMOR_OR_DRAW_TWO,
                ));
            }
            StmtIr::EffectGiveAllHand {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_151);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::GIVE_ALL_HAND));
            }
            StmtIr::EffectGainCardIfDiscard { player, card, span } => {
                self.mark(*span, "gain_card_if_discard");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_151);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_CARD_IF_DISCARD));
            }
            StmtIr::EffectChooseDrawOrDiscardTargetCardRecordSha {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "choose_draw_or_discard_target_card_record_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_152);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DRAW_OR_DISCARD_TARGET_CARD_RECORD_SHA,
                ));
            }
            StmtIr::EffectRevealHandToPlayer {
                player,
                observer,
                span,
            } => {
                self.mark(*span, "reveal_hand_to_player");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_153);
                self.emit_expr(player)?;
                self.emit_expr(observer)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_HAND_TO_PLAYER));
            }
            StmtIr::EffectChooseOptionalDiscardTargetCard {
                player,
                target,
                scope,
                span,
            } => {
                self.mark(*span, "choose_optional_discard_target_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_153);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OPTIONAL_DISCARD_TARGET_CARD,
                ));
            }
            StmtIr::EffectRevealHandChooseDiscardColor { player, span } => {
                self.mark(*span, "reveal_hand_choose_discard_color");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_154);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_HAND_CHOOSE_DISCARD_COLOR));
            }
            StmtIr::EffectChooseDiscardTargetCardsRecoverSameCategory {
                player,
                target,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_target_cards_recover_same_category");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_155);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_TARGET_CARDS_RECOVER_SAME_CATEGORY,
                ));
            }
            StmtIr::EffectChooseScskuijiDiscard {
                player,
                target,
                count,
                span,
            } => {
                self.mark(*span, "choose_scskuiji_discard");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_348);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_SCSKUIJI_DISCARD));
            }
            StmtIr::EffectChoosePskuijiDiscard {
                player,
                target,
                count,
                span,
            } => {
                self.mark(*span, "choose_pskuiji_discard");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_349);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_PSKKUIJI_DISCARD));
            }
            StmtIr::EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
                player,
                target,
                draw_count,
                failure_count,
                lose_hp,
                damage,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_compare_selected_hand_card_second_failure_seat_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_156);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(failure_count)?;
                self.emit_expr(lose_hp)?;
                self.emit_expr(damage)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD_SECOND_FAILURE_SEAT_DAMAGE,
                ));
            }
            StmtIr::EffectDiscardTargetCardGainMount {
                player,
                target,
                scope,
                span,
            } => {
                self.mark(*span, "discard_target_card_gain_mount");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_34);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_TARGET_CARD_GAIN_MOUNT));
            }
            StmtIr::EffectMakeCurrentShaTargetUnrespondable {
                source,
                card,
                target,
                span,
            } => {
                self.mark(*span, "make_current_sha_target_unrespondable");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_35);
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE,
                ));
            }
            StmtIr::EffectJudgeMakeCurrentShaTargetUnrespondableOnRed {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "judge_make_current_sha_target_unrespondable_on_red");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_177);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE_ON_RED,
                ));
            }
            StmtIr::EffectChooseTargetDiscardDistanceHandOrIncreaseDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_discard_distance_hand_or_increase_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_322);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_DISTANCE_HAND_OR_INCREASE_DAMAGE,
                ));
            }
            StmtIr::EffectChooseDrawViewTwoGainOrMoveEquipment {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_draw_view_two_gain_or_move_equipment");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_323);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DRAW_VIEW_TWO_GAIN_OR_MOVE_EQUIPMENT,
                ));
            }
            StmtIr::EffectGiveAllHandThenChooseUseOneOrDamage {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand_then_choose_use_one_or_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_359);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_ALL_HAND_THEN_CHOOSE_USE_ONE_OR_DAMAGE,
                ));
            }
            StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                player,
                card,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_category_then_target_give_matching_or_sha_unrespondable",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_360);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_SHA_UNRESPONDABLE,
                ));
            }
            StmtIr::EffectChooseLoseHpUseVirtualShaByBasicVariants { player, span } => {
                self.mark(*span, "choose_lose_hp_use_virtual_sha_by_basic_variants");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_361);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS,
                ));
            }
            StmtIr::EffectSuppressNonLockedSkillsUntilTurnEnd { player, span } => {
                self.mark(*span, "suppress_non_locked_skills_until_turn_end");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_356);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::SUPPRESS_NON_LOCKED_SKILLS_UNTIL_TURN_END,
                ));
            }
            StmtIr::EffectChooseRecoverTarget {
                chooser,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "choose_recover_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_356);
                self.emit_expr(chooser)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_RECOVER_TARGET));
            }
            StmtIr::EffectChooseGiveTwoDifferentColorsOrRecover {
                owner,
                giver,
                target,
                span,
            } => {
                self.mark(*span, "choose_give_two_different_colors_or_recover");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_330);
                self.emit_expr(owner)?;
                self.emit_expr(giver)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GIVE_TWO_DIFFERENT_COLORS_OR_RECOVER,
                ));
            }
            StmtIr::EffectApplyMbjinzuNextSha { player, card, span } => {
                self.mark(*span, "apply_mbjinzu_next_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_333);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(0));
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(0));
                self.ops.push(Op::ConstU32(2));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBJINZU));
            }
            StmtIr::EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_basic_then_target_discard_equip_or_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_178);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_BASIC_THEN_TARGET_DISCARD_EQUIP_OR_DAMAGE,
                ));
            }
            StmtIr::EffectChooseCurrentShaTargetDiscardRedirectToOwner {
                owner,
                source,
                card,
                target,
                span,
            } => {
                self.mark(*span, "choose_current_sha_target_discard_redirect_to_owner");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_180);
                self.emit_expr(owner)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CURRENT_SHA_TARGET_DISCARD_REDIRECT_TO_OWNER,
                ));
            }
            StmtIr::EffectChooseTargetGainBottomCard { player, span } => {
                self.mark(*span, "choose_target_gain_bottom_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_181);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_GAIN_BOTTOM_CARD));
            }
            StmtIr::EffectJudgeBlackGainRepeat { player, span } => {
                self.mark(*span, "judge_black_gain_repeat");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_182);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_BLACK_GAIN_REPEAT));
            }
            StmtIr::EffectUseVirtualJuedouSecondOnFirst { span } => {
                self.mark(*span, "use_virtual_juedou_second_on_first");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_183);
                self.ops.push(Op::HostEffect(
                    effect_tag::USE_VIRTUAL_JUEDOU_SECOND_ON_FIRST,
                ));
            }
            StmtIr::EffectRecoverAs {
                source,
                player,
                amount,
                span,
            } => {
                self.mark(*span, "recover_hp_as");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_185);
                self.emit_expr(source)?;
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RECOVER_HP_AS));
            }
            StmtIr::EffectChooseDiscardRedirectCurrentSha {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "choose_discard_redirect_current_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_186);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_REDIRECT_CURRENT_SHA,
                ));
            }
            StmtIr::EffectJudgeSetSkillMarkToRank {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "judge_set_skill_mark_to_rank");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_188);
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_SET_SKILL_MARK_TO_RANK));
            }
            StmtIr::EffectChooseDiscardOneFromEachLinked {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_discard_one_from_each_linked");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_189);
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_FROM_EACH_LINKED,
                ));
            }
            StmtIr::EffectDisableSkillByDefUntilTurnEnd {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "disable_skill_by_def_until_turn_end");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_191);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::DISABLE_SKILL_BY_DEF_UNTIL_TURN_END,
                ));
            }
            StmtIr::EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "target_discard_then_duel_if_non_sha_and_hp_gte_owner",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_192);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TARGET_DISCARD_THEN_DUEL_IF_NON_SHA_AND_HP_GTE_OWNER,
                ));
            }
            StmtIr::EffectChooseTargetLoseHpDiscardHandShaIfBasic { player, span } => {
                self.mark(*span, "choose_target_lose_hp_discard_hand_sha_if_basic");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_194);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_LOSE_HP_DISCARD_HAND_SHA_IF_BASIC,
                ));
            }
            StmtIr::EffectChooseDiscardTargetCard {
                player,
                target,
                scope,
                span,
            } => {
                self.mark(*span, "choose_discard_target_card");
                self.required_abi = self.required_abi.max(
                    if matches!(scope, content_schema::TargetCardScopeIr::Hand) {
                        ADDITIVE_RULE_VM_ABI_VERSION_153
                    } else {
                        ADDITIVE_RULE_VM_ABI_VERSION_36
                    },
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_TARGET_CARD));
            }
            StmtIr::EffectJudgePreventDamageLoseMaxHpOnNonHeart {
                player,
                target,
                span,
            } => {
                self.mark(*span, "judge_prevent_damage_lose_max_hp_on_nonheart");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_37);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_PREVENT_DAMAGE_LOSE_MAX_HP_ON_NONHEART,
                ));
            }
            StmtIr::EffectChooseGainTopCardsBySuit {
                player,
                count,
                suit,
                span,
            } => {
                self.mark(*span, "choose_gain_top_cards_by_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_38);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*count)));
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_TOP_CARDS_BY_SUIT));
            }
            StmtIr::EffectChooseOtherDrawByHandCount {
                player,
                normal_count,
                empty_count,
                times,
                span,
            } => {
                self.mark(*span, "choose_other_draw_by_hand_count");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_39);
                self.emit_expr(player)?;
                self.emit_expr(normal_count)?;
                self.emit_expr(empty_count)?;
                self.emit_expr(times)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DRAW_BY_HAND_COUNT));
            }
            StmtIr::EffectChooseDiscardOneThenGainTargetEquipment {
                player,
                target,
                slot,
                span,
            } => {
                self.mark(*span, "choose_discard_one_then_gain_target_equipment");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_40);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*slot)));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_THEN_GAIN_TARGET_EQUIPMENT,
                ));
            }
            StmtIr::EffectChooseTargetDraw {
                player,
                count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_target_draw");
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
                    _ => ADDITIVE_RULE_VM_ABI_VERSION_41,
                });
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_DRAW));
            }
            StmtIr::EffectDrawByPlayPhaseCardCategoryCount { player, span } => {
                self.mark(*span, "draw_by_play_phase_card_category_count");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_42);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_BY_PLAY_PHASE_CARD_CATEGORY_COUNT,
                ));
            }
            StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRed {
                player,
                sha_user,
                card,
                span,
            } => {
                self.mark(*span, "choose_discard_one_for_sha_refund_and_draw_if_red");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_43);
                self.emit_expr(player)?;
                self.emit_expr(sha_user)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED,
                ));
            }
            StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                player,
                sha_user,
                card,
                restore_skill_def,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_295);
                self.emit_expr(player)?;
                self.emit_expr(sha_user)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*restore_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED_RESTORE_IF_EQUAL_RANK,
                ));
            }
            StmtIr::EffectQueueOpeningExtraPlayPhase { player, span } => {
                self.mark(*span, "queue_opening_extra_play_phase");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_44);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::QUEUE_OPENING_EXTRA_PLAY_PHASE));
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

use super::*;

impl Assembler {
    pub(super) fn emit_effect_reveal_target_hand_sha_or_discard_bind(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectRevealTargetHandShaOrDiscardBind {
                player,
                target,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "reveal_target_hand_sha_or_discard_bind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_196);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TARGET_HAND_SHA_OR_DISCARD_BIND,
                ));
            }
            StmtIr::EffectDrawRememberedHandCards {
                player,
                count,
                span,
            } => {
                self.mark(*span, "draw_remembered_hand_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_197);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_REMEMBERED_HAND_CARDS));
            }
            StmtIr::EffectDiscardRememberedHandCards { player, span } => {
                self.mark(*span, "discard_remembered_hand_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_197);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_REMEMBERED_HAND_CARDS));
            }
            StmtIr::EffectGainDrawCardsForLifetimeCardActionDivisors {
                player,
                divisors,
                groups,
                span,
            } => {
                self.mark(*span, "gain_draw_cards_for_lifetime_card_action_divisors");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_198);
                self.emit_expr(player)?;
                for index in 0..3 {
                    self.ops.push(Op::ConstI32(divisors[index] as i32));
                    self.ops.push(Op::ConstI32(i32::from(groups[index])));
                }
                self.ops.push(Op::HostEffect(
                    effect_tag::GAIN_DRAW_CARDS_FOR_LIFETIME_CARD_ACTION_DIVISORS,
                ));
            }
            StmtIr::EffectLoseHpGainDiscardCardForOpeningExtraPlay {
                player,
                kind,
                mandatory_skill_def,
                span,
            } => {
                self.mark(*span, "lose_hp_gain_discard_card_for_opening_extra_play");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_200);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops.push(Op::ConstU32(*mandatory_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::LOSE_HP_GAIN_DISCARD_CARD_FOR_OPENING_EXTRA_PLAY,
                ));
            }
            StmtIr::EffectChooseGainOwnLostDiscardCardThisTurn { player, span } => {
                self.mark(*span, "choose_gain_own_lost_discard_card_this_turn");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_201);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_OWN_LOST_DISCARD_CARD_THIS_TURN,
                ));
            }
            StmtIr::EffectChooseMoveFieldCardTo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_move_field_card_to");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_202);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_MOVE_FIELD_CARD_TO));
            }
            StmtIr::EffectDrawThenOptionallyDistributeHandCards {
                player,
                draw_count,
                give_cap,
                span,
            } => {
                self.mark(*span, "draw_then_optionally_distribute_hand_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_203);
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(give_cap)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_OPTIONALLY_DISTRIBUTE_HAND_CARDS,
                ));
            }
            StmtIr::EffectGiveSelectedThenContinueDistinctRecoverAtTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_then_continue_distinct_recover_at_two");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_204);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO,
                ));
            }
            StmtIr::EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                player,
                mark_def,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_205);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_MAX_MARK_OTHER_JUDGE_KILL_UNLESS_TAO_OR_TAOYUAN,
                ));
            }
            StmtIr::EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_zone_priority_gain_random_card_from_each_other_then_turn_over",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_206);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ZONE_PRIORITY_GAIN_RANDOM_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
                ));
            }
            StmtIr::EffectChooseGainCardFromEachOtherThenTurnOver { player, span } => {
                self.mark(*span, "choose_gain_card_from_each_other_then_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_237);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
                ));
            }
            StmtIr::EffectResolveHiddenSkillPile {
                player,
                pile_skill_def,
                action,
                card,
                span,
            } => {
                self.mark(*span, "resolve_hidden_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_238);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*pile_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_HIDDEN_SKILL_PILE));
            }
            StmtIr::EffectSetNonLockedSkillSuppression {
                player,
                enabled,
                span,
            } => {
                self.mark(*span, "set_non_locked_skill_suppression");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_239);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*enabled)));
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_NON_LOCKED_SKILL_SUPPRESSION));
            }
            StmtIr::EffectResolveDiliYuheng {
                player,
                action,
                card,
                span,
            } => {
                self.mark(*span, "resolve_dili_yuheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_240);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DILI_YUHENG));
            }
            StmtIr::EffectDiscardSelectedDrawAllHandBonus { player, span } => {
                self.mark(*span, "discard_selected_draw_all_hand_bonus");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_241);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_SELECTED_DRAW_ALL_HAND_BONUS,
                ));
            }
            StmtIr::EffectChooseOtherYinghun { player, span } => {
                self.mark(*span, "choose_other_yinghun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_242);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_YINGHUN));
            }
            StmtIr::EffectChooseOtherBizheng { player, span } => {
                self.mark(*span, "choose_other_bizheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_243);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_BIZHENG));
            }
            StmtIr::EffectChooseOtherXiashu { player, span } => {
                self.mark(*span, "choose_other_xiashu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_244);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_XIASHU));
            }
            StmtIr::EffectResolveGuanchaoRank { player, card, span } => {
                self.mark(*span, "resolve_guanchao_rank");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_245);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GUANCHAO_RANK));
            }
            StmtIr::EffectResolveGuanwei {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_guanwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_246);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUANWEI));
            }
            StmtIr::EffectResolveLanjiang { player, span } => {
                self.mark(*span, "resolve_lanjiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_247);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LANJIANG));
            }
            StmtIr::EffectResolveRebingyi { player, span } => {
                self.mark(*span, "resolve_rebingyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_248);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REBINGYI));
            }
            StmtIr::EffectResolveRebotu { player, span } => {
                self.mark(*span, "resolve_rebotu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_249);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REBOTU));
            }
            StmtIr::EffectResolveOldimeng {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_oldimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_250);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDIMENG));
            }
            StmtIr::EffectResolveRejieyin { player, span } => {
                self.mark(*span, "resolve_rejieyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_251);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REJIEYIN));
            }
            StmtIr::EffectResolveReanxu { player, span } => {
                self.mark(*span, "resolve_reanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_252);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REANXU));
            }
            StmtIr::EffectResolveDrltJueyan {
                player,
                sha_skill_def,
                weapon_skill_def,
                mount_skill_def,
                armor_skill_def,
                treasure_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_drlt_jueyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_253);
                self.emit_expr(player)?;
                for skill_def in [
                    sha_skill_def,
                    weapon_skill_def,
                    mount_skill_def,
                    armor_skill_def,
                    treasure_skill_def,
                ] {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_JUEYAN));
            }
            StmtIr::EffectResolveRejizhi { player, span } => {
                self.mark(*span, "resolve_rejizhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_254);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REJIZHI));
            }
            StmtIr::EffectResolveJiexun {
                player,
                funan_helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_jiexun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_255);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*funan_helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIEXUN));
            }
            StmtIr::EffectResolveWenguaSetup {
                player,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_wengua_setup");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_256);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WENGUA_SETUP));
            }
            StmtIr::EffectResolveWengua {
                player,
                main_skill_def,
                marker_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_wengua");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_257);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*main_skill_def as i32));
                self.ops.push(Op::ConstI32(*marker_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WENGUA));
            }
            StmtIr::EffectResolveXinanguo { player, span } => {
                self.mark(*span, "resolve_xinanguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_258);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XINANGUO));
            }
            StmtIr::EffectResolveGanglieThree { player, span } => {
                self.mark(*span, "resolve_ganglie_three");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_259);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GANGLIE_THREE));
            }
            StmtIr::EffectResolveZhongyi {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_zhongyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_260);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGYI));
            }
            StmtIr::EffectDiscardEquipmentSlot { player, slot, span } => {
                self.mark(*span, "discard_equipment_slot");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_261);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*slot)));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_EQUIPMENT_SLOT));
            }
            StmtIr::EffectResolveOldAnxu { player, span } => {
                self.mark(*span, "resolve_oldanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_262);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDANXU));
            }
            StmtIr::EffectStoreSelectedInSkillPile { player, span } => {
                self.mark(*span, "store_selected_in_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_263);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::STORE_SELECTED_IN_SKILL_PILE));
            }
            StmtIr::EffectEndMatchForCampIfSkillPileAtLeast {
                player,
                pile_skill_def,
                count,
                span,
            } => {
                self.mark(*span, "end_match_for_player_camp_if_skill_pile_at_least");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_264);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*pile_skill_def as i32));
                self.ops.push(Op::ConstI32(*count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::END_MATCH_FOR_CAMP_IF_SKILL_PILE_AT_LEAST,
                ));
            }
            StmtIr::EffectResolveOldQinqing { player, span } => {
                self.mark(*span, "resolve_oldqinqing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_265);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDQINQING));
            }
            StmtIr::EffectResolveOldMingjian { player, span } => {
                self.mark(*span, "resolve_oldmingjian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_266);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDMINGJIAN));
            }
            StmtIr::EffectResolveOldXuanfeng { player, span } => {
                self.mark(*span, "resolve_oldxuanfeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_267);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDXUANFENG));
            }
            StmtIr::EffectResolveJianrou { player, span } => {
                self.mark(*span, "resolve_jianrou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_268);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANROU));
            }
            StmtIr::EffectResolveOldHuisheng { player, span } => {
                self.mark(*span, "resolve_oldhuisheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_269);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDHUISHENG));
            }
            StmtIr::EffectResolveOldJiefan {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_oldjiefan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_270);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDJIEFAN));
            }
            StmtIr::EffectResolveChegu { player, span } => {
                self.mark(*span, "resolve_chegu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_271);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHEGU));
            }
            StmtIr::EffectResolveJunkguixin { player, span } => {
                self.mark(*span, "resolve_junkguixin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_272);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JUNKGUIXIN));
            }
            StmtIr::EffectResolveYaoChenwei { player, span } => {
                self.mark(*span, "resolve_yao_chenwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_275);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_YAO_CHENWEI));
            }
            StmtIr::EffectAddSkillUntilTurnEnd {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "add_skill_until_turn_end");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::ADD_SKILL_UNTIL_TURN_END));
            }
            StmtIr::EffectResolveMassDamageDiscardTurnOver {
                player,
                mark_def,
                mark_count,
                damage_amount,
                hand_count,
                span,
            } => {
                self.mark(*span, "resolve_mass_damage_discard_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mark_count as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(*hand_count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_TURN_OVER,
                ));
            }
            StmtIr::EffectResolveMassDamageDiscardHandEquip {
                player,
                mark_def,
                mark_count,
                damage_amount,
                discard_count,
                span,
            } => {
                self.mark(*span, "resolve_mass_damage_discard_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_235);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mark_count as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(*discard_count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP,
                ));
            }
            StmtIr::EffectRecordActiveTarget {
                player,
                target,
                span,
            } => {
                self.mark(*span, "record_active_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECORD_ACTIVE_TARGET));
            }
            StmtIr::EffectChooseDiscardWeaponOrLoseHp { player, span } => {
                self.mark(*span, "choose_discard_weapon_or_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_222);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_WEAPON_OR_LOSE_HP));
            }
            StmtIr::EffectResolveMarkParityControl {
                player,
                mark_def,
                mass_threshold,
                damage_amount,
                span,
            } => {
                self.mark(*span, "resolve_mark_parity_control");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mass_threshold as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MARK_PARITY_CONTROL));
            }
            StmtIr::EffectResolveSelectedEquipmentDiscardThenChooseDamage {
                player,
                damage_amount,
                nature,
                span,
            } => {
                self.mark(
                    *span,
                    "resolve_selected_equipment_discard_then_choose_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(i32::from(*nature)));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE,
                ));
            }
            StmtIr::EffectReplaceDrawWithDistinctSuitSelection {
                player,
                count,
                span,
            } => {
                self.mark(*span, "replace_draw_with_distinct_suit_selection");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_209);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*count)));
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_DRAW_WITH_DISTINCT_SUIT_SELECTION,
                ));
            }
            StmtIr::EffectInspectTargetHandChooseHeartDiscardOrTop {
                player,
                target,
                span,
            } => {
                self.mark(*span, "inspect_target_hand_choose_heart_discard_or_top");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_209);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::INSPECT_TARGET_HAND_CHOOSE_HEART_DISCARD_OR_TOP,
                ));
            }
            StmtIr::EffectResolveYeyan { player, span } => {
                self.mark(*span, "resolve_yeyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_210);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YEYAN));
            }
            StmtIr::EffectChooseAllLoseHpOrRecover { player, span } => {
                self.mark(*span, "choose_all_lose_hp_or_recover");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_210);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ALL_LOSE_HP_OR_RECOVER));
            }
            StmtIr::EffectResolveQixing {
                player,
                initialize,
                span,
            } => {
                self.mark(*span, "resolve_qixing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_212);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*initialize)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIXING));
            }
            StmtIr::EffectResolveQixingWeather {
                player,
                qixing_skill_def,
                helper_skill_def,
                max_targets,
                span,
            } => {
                self.mark(*span, "resolve_qixing_weather");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_212);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qixing_skill_def as i32));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*max_targets)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_QIXING_WEATHER));
            }
            StmtIr::EffectResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_nzry_longnu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*fire_skill_def as i32));
                self.ops.push(Op::ConstI32(*thunder_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NZRY_LONGNU));
            }
            StmtIr::EffectChooseLinkOther { player, span } => {
                self.mark(*span, "choose_link_other");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::CHOOSE_LINK_OTHER));
            }
            StmtIr::EffectResolveDrltPoxi {
                player,
                target,
                hand_limit_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_drlt_poxi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_214);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*hand_limit_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DRLT_POXI));
            }
            StmtIr::EffectResolveDrltJieying {
                player,
                event_player,
                mark,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_drlt_jieying");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_215);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::ConstI32(*mark as i32));
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_JIEYING));
            }
            StmtIr::EffectResolveDrltDuorui {
                player,
                target,
                helper_skill_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_drlt_duorui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_216);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_DUORUI));
            }
            StmtIr::EffectResolveDrltZhiti {
                player,
                event_player,
                event_source,
                event_amount,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_drlt_zhiti");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_217);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_source)?;
                self.emit_expr(event_amount)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_ZHITI));
            }
            StmtIr::EffectResolveShenSimayi {
                player,
                event_player,
                event_amount,
                mark,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_shen_simayi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_218);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_amount)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHEN_SIMAYI));
            }
            StmtIr::EffectResolveBeige {
                player,
                target,
                source,
                span,
            } => {
                self.mark(*span, "resolve_beige");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_219);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BEIGE));
            }
            StmtIr::EffectResolveBeigeConfigured {
                player,
                target,
                source,
                heart_recover_amount,
                diamond_draw_count,
                span,
            } => {
                self.mark(*span, "resolve_beige_configured");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_296);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(heart_recover_amount)?;
                self.emit_expr(diamond_draw_count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_BEIGE_CONFIGURED));
            }
            StmtIr::EffectResolveZhuiyi {
                player,
                killer,
                span,
            } => {
                self.mark(*span, "resolve_zhuiyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_297);
                self.emit_expr(player)?;
                self.emit_expr(killer)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHUIYI));
            }
            StmtIr::EffectResolveDecadezhenjun { player, span } => {
                self.mark(*span, "resolve_decadezhenjun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_298);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DECADEZHENJUN));
            }
            StmtIr::EffectResolveSpXiaoguo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_sp_xiaoguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_299);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SP_XIAOGUO));
            }
            StmtIr::EffectChooseGainCardFromEachOtherReplaceDraw { player, span } => {
                self.mark(*span, "choose_gain_card_from_each_other_replace_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_300);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW,
                ));
            }
            StmtIr::EffectGiveSelectedThenRecoverSelf {
                player,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "give_selected_then_recover_self");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_679);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GIVE_SELECTED_THEN_RECOVER_SELF));
            }
            StmtIr::EffectChooseHandCompareBetween {
                player,
                first,
                span,
            } => {
                self.mark(*span, "choose_hand_compare_between");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_680);
                self.emit_expr(player)?;
                self.emit_expr(first)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_HAND_COMPARE_BETWEEN));
            }
            StmtIr::EffectGuanxingTop {
                player,
                count,
                span,
            } => {
                self.mark(*span, "guanxing_top");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_681);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::GUANXING_TOP));
            }
            StmtIr::EffectGuanxingTopByFaluMarks {
                player,
                marks,
                span,
            } => {
                self.mark(*span, "guanxing_top_by_falu_marks");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_683);
                self.emit_expr(player)?;
                for def in marks {
                    self.ops.push(Op::ConstI32(*def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::GUANXING_TOP_BY_FALU_MARKS));
            }
            StmtIr::EffectChooseOtherGiveThenRecover { player, span } => {
                self.mark(*span, "choose_other_give_then_recover");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_682);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_GIVE_THEN_RECOVER));
            }
            StmtIr::EffectGiveSelectedThenRecoverTarget {
                player,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "give_selected_then_recover_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_301);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_RECOVER_TARGET,
                ));
            }
            StmtIr::EffectResolveDingpan {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dingpan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_302);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGPAN));
            }
            StmtIr::EffectResolveTanlu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_tanlu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_303);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TANLU));
            }
            StmtIr::EffectJudgeSameColorGainRepeat { player, span } => {
                self.mark(*span, "judge_same_color_gain_repeat");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_307);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT));
            }
            StmtIr::EffectChooseCurrentTrickTargetsToNullify {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "choose_current_trick_targets_to_nullify");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_308);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY,
                ));
            }
            StmtIr::EffectJudgeBlackGainRepeatProtectHandLimit { player, span } => {
                self.mark(*span, "judge_black_gain_repeat_protect_hand_limit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_309);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT,
                ));
            }
            StmtIr::EffectChooseGainAllResolvedShaShanThenColorRewards {
                player,
                sha_card,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_gain_all_resolved_sha_shan_then_color_rewards",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_310);
                self.emit_expr(player)?;
                self.emit_expr(sha_card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS,
                ));
            }
            StmtIr::EffectGainEventCards { player, span } => {
                self.mark(*span, "gain_event_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_82);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::GAIN_EVENT_CARDS));
            }
            StmtIr::EffectChooseTargetDrawToMaxHpCap { player, cap, span } => {
                self.mark(*span, "choose_target_draw_to_max_hp_cap");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_84);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*cap as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_DRAW_TO_MAX_HP_CAP));
            }
            StmtIr::EffectChooseOtherDrawOwnerLostHpTurnOver { player, span } => {
                self.mark(*span, "choose_other_draw_owner_lost_hp_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_85);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_DRAW_OWNER_LOST_HP_TURN_OVER,
                ));
            }
            StmtIr::EffectDiscardOneHandOrEquip { player, span } => {
                self.mark(*span, "discard_one_hand_or_equip");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ONE_HAND_OR_EQUIP));
            }
            StmtIr::EffectDiscardAllHandAndEquip { player, span } => {
                self.mark(*span, "discard_all_hand_and_equip");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_HAND_AND_EQUIP));
            }
            StmtIr::EffectRemoveAllSkills { player, span } => {
                self.mark(*span, "remove_all_skills");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_159);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_ALL_SKILLS));
            }
            StmtIr::EffectJudgeGainEquipmentByColor { player, span } => {
                self.mark(*span, "judge_gain_equipment_by_color");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_GAIN_EQUIPMENT_BY_COLOR));
            }
            StmtIr::EffectJudgeRecoverOnSuit {
                player,
                suit,
                amount,
                span,
            } => {
                self.mark(*span, "judge_recover_on_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_12);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_RECOVER_ON_SUIT));
            }
            StmtIr::EffectJudgeRecoverOtherOnSuit {
                judge_player,
                recover_player,
                suit,
                amount,
                span,
            } => {
                self.mark(*span, "judge_recover_other_on_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_88);
                self.emit_expr(judge_player)?;
                self.emit_expr(recover_player)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_RECOVER_OTHER_ON_SUIT));
            }
            StmtIr::EffectForceOthersUseShaOnNearestOrLoseHp { player, span } => {
                self.mark(*span, "force_others_use_sha_on_nearest_or_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_90);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::FORCE_OTHERS_USE_SHA_ON_NEAREST_OR_LOSE_HP,
                ));
            }
            StmtIr::EffectJudgeDamageOtherOnSuit {
                player,
                suit,
                amount,
                nature,
                span,
            } => {
                self.mark(*span, "judge_damage_other_on_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_15);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops.push(Op::ConstI32(i32::from(*nature)));
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_DAMAGE_OTHER_ON_SUIT));
            }
            StmtIr::EffectRevealDiscardSuitGainOrUse {
                player,
                mark,
                base_count,
                max_mark,
                span,
            } => {
                self.mark(*span, "reveal_discard_suit_gain_or_use");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.ops.push(Op::ConstI32(i32::from(*base_count)));
                self.ops.push(Op::ConstI32(i32::from(*max_mark)));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_DISCARD_SUIT_GAIN_OR_USE));
            }
            StmtIr::EffectChooseDiscardOrFireDamageSkipPhases {
                player,
                max_discard,
                damage,
                span,
            } => {
                self.mark(*span, "choose_discard_or_fire_damage_skip_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*max_discard)));
                self.ops.push(Op::ConstI32(i32::from(*damage)));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_OR_FIRE_DAMAGE_SKIP_PHASES,
                ));
            }
            StmtIr::EffectDrawSelected { player, span } => {
                self.mark(*span, "draw_selected");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_SELECTED));
            }
            StmtIr::EffectDrawEachBySeat {
                first,
                second,
                count,
                span,
            } => {
                self.mark(*span, "draw_each_by_seat");
                self.emit_expr(first)?;
                self.emit_expr(second)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_EACH_BY_SEAT));
            }
            StmtIr::EffectDrawThenGiveOne {
                player,
                target,
                count,
                drawn_only,
                use_given_equipment,
                span,
            } => {
                self.mark(*span, "draw_then_give_one");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(
                    i32::from(*drawn_only) | (i32::from(*use_given_equipment) << 1),
                ));
                if *use_given_equipment {
                    self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_227);
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_THEN_GIVE_ONE));
            }
            StmtIr::EffectDrawThenTakeOne {
                player,
                taker,
                count,
                span,
            } => {
                self.mark(*span, "draw_then_take_one");
                self.emit_expr(player)?;
                self.emit_expr(taker)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_THEN_TAKE_ONE));
            }
            StmtIr::EffectTakeOneCard {
                from,
                to,
                equip_only,
                span,
            } => {
                self.mark(*span, "take_one_card");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::ConstI32(i32::from(*equip_only)));
                self.ops.push(Op::HostEffect(effect_tag::TAKE_ONE_CARD));
            }
            StmtIr::EffectTakeOneScopedCard {
                from,
                to,
                scope,
                span,
            } => {
                self.mark(*span, "take_one_card");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::TAKE_ONE_SCOPED_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_334);
            }
            StmtIr::EffectRevealHand { player, span } => {
                self.mark(*span, "reveal_hand");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::REVEAL_HAND));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_336);
            }
            StmtIr::EffectDrawLinkedPlayers { count, span } => {
                self.mark(*span, "draw_linked_players");
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_LINKED_PLAYERS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_344);
            }
            StmtIr::EffectTakeOneEventTargetHand { player, span } => {
                self.mark(*span, "take_one_event_target_hand");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::TAKE_ONE_EVENT_TARGET_HAND));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_231);
            }
            StmtIr::EffectTakeOneHandCard { player, span } => {
                self.mark(*span, "take_one_event_target_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_231);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::TAKE_ONE_EVENT_TARGET_HAND));
            }
            StmtIr::EffectChooseOtherHandStoreSkillPile {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_other_hand_store_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_567);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_STORE_SKILL_PILE,
                ));
            }
            StmtIr::EffectGainAllSkillPileCards { player, span } => {
                self.mark(*span, "gain_all_skill_pile_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_569);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_ALL_SKILL_PILE_CARDS));
            }
            StmtIr::EffectGainOtherSkillPileCards {
                player,
                target,
                skill_def,
                span,
            } => {
                self.mark(*span, "gain_other_skill_pile_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_600);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_OTHER_SKILL_PILE_CARDS));
            }
            StmtIr::EffectChooseDiscardOneHeOrLoseHp {
                player,
                lose_amount,
                span,
            } => {
                self.mark(*span, "choose_discard_one_he_or_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_601);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*lose_amount));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_ONE_HE_OR_LOSE_HP));
            }
            StmtIr::EffectChooseEndExtraDrawOrPlayByUsedSuits { player, span } => {
                self.mark(*span, "choose_end_extra_draw_or_play_by_used_suits");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_END_EXTRA_DRAW_OR_PLAY_BY_USED_SUITS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_232);
            }
            StmtIr::EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_602);
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_SPADE_TWO_TO_NINE_DRAW_TWO_OR_THUNDER_SHA_TWICE,
                ));
            }
            StmtIr::EffectChooseSourceConfirmOwnerLoseHp {
                owner,
                source,
                amount,
                span,
            } => {
                self.mark(*span, "choose_source_confirm_owner_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_603);
                self.emit_expr(owner)?;
                self.emit_expr(source)?;
                self.ops.push(Op::ConstU32(*amount));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_SOURCE_CONFIRM_OWNER_LOSE_HP,
                ));
            }
            StmtIr::EffectLoseHpDoubleCurrentDamageThenReplace {
                player,
                replacement_skill_def,
                span,
            } => {
                self.mark(*span, "lose_hp_double_current_damage_then_replace");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*replacement_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::LOSE_HP_DOUBLE_CURRENT_DAMAGE_THEN_REPLACE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_233);
            }
            StmtIr::EffectTurnOver { player, span } => {
                self.mark(*span, "turn_over");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::TURN_OVER));
            }
            StmtIr::EffectRestoreCharacterState { player, span } => {
                self.mark(*span, "restore_character_state");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESTORE_CHARACTER_STATE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_93);
            }
            StmtIr::EffectDiscardAllHandEquipJudge { player, span } => {
                self.mark(*span, "discard_all_hand_equip_judge");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_HAND_EQUIP_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_93);
            }
            StmtIr::EffectDamageThenDiscardAllJudge { player, span } => {
                self.mark(*span, "damage_then_discard_all_judge");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DAMAGE_THEN_DISCARD_ALL_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_224);
            }
            StmtIr::EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
                span,
            } => {
                self.mark(
                    *span,
                    "replace_draw_gain_top_two_add_skills_if_different_colors",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*first_skill_def));
                self.ops.push(Op::ConstU32(*second_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_DRAW_GAIN_TOP_TWO_ADD_SKILLS_IF_DIFFERENT_COLORS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_225);
            }
            StmtIr::EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_gain_target_card_alternate_zone_then_target_draw",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_TARGET_CARD_ALTERNATE_ZONE_THEN_TARGET_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_226);
            }
            StmtIr::EffectChooseDiscardTargetCardResolveCurrentSha {
                source,
                card,
                target,
                span,
            } => {
                self.mark(*span, "choose_discard_target_card_resolve_current_sha");
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_94);
            }
            StmtIr::EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                source,
                card,
                target,
                bonus_skill_def,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_target_card_resolve_current_sha_nonbasic_bonus",
                );
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*bonus_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_312);
            }
            StmtIr::EffectResolveDcqiaomeng {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcqiaomeng");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCQIAOMENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_313);
            }
            StmtIr::EffectUseVirtualCardOnSelectedTargets { player, kind, span } => {
                self.mark(*span, "use_virtual_card_on_selected_targets");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops.push(Op::HostEffect(
                    effect_tag::USE_VIRTUAL_CARD_ON_SELECTED_TARGETS,
                ));
                self.required_abi = self.required_abi.max(if *kind == 34 {
                    ADDITIVE_RULE_VM_ABI_VERSION_343
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_95
                });
            }
            StmtIr::EffectDealDamageBanActiveIfDying {
                source,
                target,
                amount,
                skill,
                span,
            } => {
                self.mark(*span, "deal_damage_ban_active_if_dying");
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DEAL_DAMAGE_BAN_ACTIVE_IF_DYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_96);
            }
            StmtIr::EffectSpjungongDiscardOrLoseThenSha {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(*span, "spjungong_discard_or_lose_then_sha");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::SPJUNGONG_DISCARD_OR_LOSE_THEN_SHA,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_400);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

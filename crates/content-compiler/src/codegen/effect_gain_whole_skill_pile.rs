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
            StmtIr::EffectResolveQianlong {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_qianlong");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANLONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_697);
            }
            StmtIr::EffectResolveJsrgzuozhan {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgzuozhan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGZUOZHAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_698);
            }
            StmtIr::EffectResolveJsrgzuozhanGain {
                player,
                skill,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_jsrgzuozhan_gain");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGZUOZHAN_GAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_699);
            }
            StmtIr::EffectResolveTwmouzhi {
                player,
                skill,
                mode,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_twmouzhi");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMOUZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_700);
            }
            StmtIr::EffectResolveTwdaigui {
                player,
                skill,
                stage,
                count,
                span,
            } => {
                self.mark(*span, "resolve_twdaigui");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWDAIGUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_701);
            }
            StmtIr::EffectResolveTwmitu {
                player,
                skill,
                stage,
                count,
                span,
            } => {
                self.mark(*span, "resolve_twmitu");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMITU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_702);
            }
            StmtIr::EffectResolveQiantun {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_qiantun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANTUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_703);
            }
            StmtIr::EffectResolveYangming {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_yangming");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YANGMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_704);
            }
            StmtIr::EffectResolveCanyun {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_canyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CANYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_705);
            }
            StmtIr::EffectResolveHengwu {
                player,
                skill,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_hengwu");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HENGWU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_706);
            }
            StmtIr::EffectResolveBiaozhao3 {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_biaozhao3");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIAOZHAO3));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_707);
            }
            StmtIr::EffectResolveChuanyun {
                player,
                skill,
                event_player,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_chuanyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHUANYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_708);
            }
            StmtIr::EffectResolveBiaozhao2 {
                player,
                skill,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_biaozhao2");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIAOZHAO2));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_709);
            }
            StmtIr::EffectResolveDcniji {
                player,
                skill,
                mode,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_dcniji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCNIJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_710);
            }
            StmtIr::EffectResolveDcmoukui {
                player,
                skill,
                stage,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcmoukui");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMOUKUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_711);
            }
            StmtIr::EffectResolveZhongyan {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_zhongyan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_712);
            }
            StmtIr::EffectResolveChumei {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_chumei");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHUMEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_713);
            }
            StmtIr::EffectResolveShuaiyan {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_shuaiyan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHUAIYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_715);
            }
            StmtIr::EffectResolveMbbeiming {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbbeiming");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBBEIMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_718);
            }
            StmtIr::EffectResolveReliangyin {
                player,
                skill,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_reliangyin");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*mode));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_RELIANGYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_717);
            }
            StmtIr::EffectResolveZhengjian {
                player,
                skill,
                stage,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_zhengjian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_716);
            }
            StmtIr::EffectResolveQinmian {
                player,
                skill,
                mode,
                amount,
                span,
            } => {
                self.mark(*span, "resolve_qinmian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINMIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_714);
            }
            StmtIr::EffectResolveJsrgdaoren {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgdaoren");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGDAOREN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_696);
            }
            StmtIr::EffectResolveDuwang {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_duwang");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUWANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_695);
            }
            StmtIr::EffectResolveQuanjia {
                player,
                skill,
                source,
                target,
                span,
            } => {
                self.mark(*span, "resolve_quanjia");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QUANJIA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_694);
            }
            StmtIr::EffectResolvePxqixian {
                player,
                skill,
                stage,
                kind,
                span,
            } => {
                self.mark(*span, "resolve_pxqixian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(kind)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_PXQIXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_693);
            }
            StmtIr::EffectResolveDcbingji {
                player,
                skill,
                stage,
                kind,
                span,
            } => {
                self.mark(*span, "resolve_dcbingji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(kind)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCBINGJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_692);
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
            StmtIr::EffectResolveTianxiang {
                player,
                draw_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_heart_redirect_current_damage");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*draw_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIANXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_456);
            }
            StmtIr::EffectResolveRetianxiang { player, span } => {
                self.mark(*span, "choose_discard_heart_cancel_damage_branch");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_RETIANXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_457);
            }
            StmtIr::EffectResolveLianzhou { player, span } => {
                self.mark(*span, "choose_link_other_same_hp");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIANZHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_458);
            }
            StmtIr::EffectResolveStddechong { player, span } => {
                self.mark(*span, "choose_give_hand_equip_to_event_player");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STDDECHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_459);
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
            StmtIr::EffectResolveJieliangGain { player, span } => {
                self.mark(*span, "resolve_jieliang_gain");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JIELIANG_GAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_461);
            }
            StmtIr::EffectResolveStdjujian { player, span } => {
                self.mark(*span, "choose_other_gain_event_cards");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_STDJUJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_462);
            }
            StmtIr::EffectResolveLulve { player, span } => {
                self.mark(*span, "resolve_lulve");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LULVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_463);
            }
            StmtIr::EffectResolveDctingxian { player, span } => {
                self.mark(*span, "resolve_dctingxian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCTINGXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_464);
            }
            StmtIr::EffectResolveHfjieying {
                player,
                grant_def,
                span,
            } => {
                self.mark(*span, "choose_other_add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*grant_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HFJIEYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_465);
            }
            StmtIr::EffectResolveDcnuchen { player, span } => {
                self.mark(*span, "resolve_dcnuchen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCNUCHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_466);
            }
            StmtIr::EffectResolveDcdanji {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_dcdanji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_466);
            }
            StmtIr::EffectResolveDczhiheng { player, span } => {
                self.mark(*span, "resolve_dczhiheng");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCZHIHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_467);
            }
            StmtIr::EffectResolveStdgaobian { player, span } => {
                self.mark(*span, "resolve_stdgaobian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STDGAOBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_468);
            }
            StmtIr::EffectResolveSpjiedao { player, span } => {
                self.mark(*span, "resolve_spjiedao");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPJIEDAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_469);
            }
            StmtIr::EffectResolveSibian { player, span } => {
                self.mark(*span, "resolve_sibian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SIBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_470);
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
            StmtIr::EffectResolveRefuhai { player, span } => {
                self.mark(*span, "resolve_refuhai");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REFUHAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_493);
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
            StmtIr::EffectResolveGzjinfa {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_gzjinfa");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GZJINFA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_495);
            }
            StmtIr::EffectResolveWnQifeng {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_wn_qifeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WN_QIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_496);
            }
            StmtIr::EffectResolveDcjue {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcjue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCJUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_583);
            }
            StmtIr::EffectResolveRuanji {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ruanji");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RUANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_584);
            }
            StmtIr::EffectResolveZhengbi {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhengbi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGBI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_585);
            }
            StmtIr::EffectResolveFengying {
                player,
                helper_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_fengying");
                self.emit_expr(player)?;
                self.emit_expr(helper_def)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FENGYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_585);
            }
            StmtIr::EffectResolveQiaojian {
                player,
                target,
                helper_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_qiaojian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(helper_def)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIAOJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_586);
            }
            StmtIr::EffectResolveXicha {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_xicha");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XICHA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_586);
            }
            StmtIr::EffectResolveHuzun {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_huzun");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HUZUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_587);
            }
            StmtIr::EffectResolveZhugemengxue {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhugemengxue");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_ZHUGEMENGXUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_588);
            }
            StmtIr::EffectResolveBailingyun {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_bailingyun");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_BAILINGYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_593);
            }
            StmtIr::EffectResolveDcshaowei {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dcshaowei");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCSHAOWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_594);
            }
            StmtIr::EffectResolveDcdichou {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcdichou");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDICHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_594);
            }
            StmtIr::EffectResolveDcdufeng {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcdufeng");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDUFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_595);
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
            StmtIr::EffectResolveDcctjiuxian {
                player,
                event_player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dcctjiuxian");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCCTJIUXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_597);
            }
            StmtIr::EffectResolveBianyue {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_bianyue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIANYUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_598);
            }
            StmtIr::EffectResolveZhupeilan {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhupeilan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHUPEILAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_600);
            }
            StmtIr::EffectResolveDcSbZhangxiu {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_zhangxiu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_ZHANGXIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_601);
            }
            StmtIr::EffectResolveDcSbZhouyu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_zhouyu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_ZHOUYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_602);
            }
            StmtIr::EffectResolveDcSbLusu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_lusu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_LUSU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_603);
            }
            StmtIr::EffectResolveZhangjian {
                player,
                card,
                grant_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhangjian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*grant_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHANGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_604);
            }
            StmtIr::EffectResolveZhugeruoxue {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhugeruoxue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_ZHUGERUOXUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_605);
            }
            StmtIr::EffectResolveCaoyi {
                player,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_caoyi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CAOYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_606);
            }
            StmtIr::EffectResolveMalingli {
                player,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_malingli");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MALINGLI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_607);
            }
            StmtIr::EffectResolveNsshuaiyan {
                player,
                recovered_player,
                span,
            } => {
                self.mark(*span, "resolve_nsshuaiyan");
                self.emit_expr(player)?;
                self.emit_expr(recovered_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NSSHUAIYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_608);
            }
            StmtIr::EffectResolveWdbHanlong {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_hanlong");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_HANLONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_609);
            }
            StmtIr::EffectResolveWdbLiufu {
                player,
                target,
                card,
                zhucheng_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liufu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*zhucheng_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUFU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_610);
            }
            StmtIr::EffectResolveWdbLiuyan {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liuyan");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_611);
            }
            StmtIr::EffectResolveWdbLiuzan {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liuzan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUZAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_612);
            }
            StmtIr::EffectResolveWdbTianyu {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_tianyu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_TIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_613);
            }
            StmtIr::EffectResolveWdbXizhenxihong {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_xizhenxihong");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_XIZHENXIHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_614);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}

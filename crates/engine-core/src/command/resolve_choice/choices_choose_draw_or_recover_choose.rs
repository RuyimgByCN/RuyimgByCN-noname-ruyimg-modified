//! 选择续行（resolve_choice）：choose 族效果。

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    _vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    _optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::ChooseDrawOrRecover => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let draw_count = *payload.get(1).unwrap_or(&1);
            let recover_count = *payload.get(2).unwrap_or(&1);
            let allow_cancel = *payload.get(3).unwrap_or(&0) != 0;
            let recover_target = PlayerId::new(payload.get(4).copied().unwrap_or(player.raw()));
            if cancelled {
                return if allow_cancel {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "draw-or-recover choice cannot cancel".into(),
                    ))
                };
            }
            let effect = match selections {
                [1] => EffectRequest::DrawCards {
                    player,
                    count: draw_count,
                },
                [2] => EffectRequest::RecoverHp {
                    source: recover_target,
                    player: recover_target,
                    amount: recover_count,
                },
                _ => {
                    return Err(EffectError::Illegal(
                        "choose draw or recover exactly once".into(),
                    ));
                }
            };
            run_effects(tx, vec![effect]).map(|_| ())
        }

        ContinuationKind::ChooseDingpanBranch => {
            resolve_choose_dingpan_branch(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic => {
            resolve_choose_target_lose_hp_discard_hand_sha_if_basic(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetRevealHandDamageIfSha => {
            resolve_choose_target_reveal_hand_damage_if_sha(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherGiveOneHandEquip => {
            resolve_choose_other_give_one_hand_equip(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGiveOrRecastOneHand => {
            resolve_choose_give_or_recast_one_hand(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherGiveThenRecover => {
            resolve_choose_other_give_then_recover(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardTargetCard => {
            resolve_choose_discard_target_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha => {
            resolve_choose_discard_target_card_current_sha(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable => {
            resolve_choose_category_then_target_give_matching_or_sha_unrespondable(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDrawChoice => {
            // Delegate to the concurrent process's resolve_choose_target_draw
            crate::command::resolve_choice::group_post_3::resolve_choose_target_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetReclaimEquipmentDrawIfRangeReduced => {
            resolve_choose_target_reclaim_equipment_draw_if_range_reduced(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseUseVirtualCard => {
            resolve_choose_use_virtual_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardOneThenGainTargetEquipment => {
            resolve_choose_discard_one_then_gain_target_equipment(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDraw => {
            resolve_choose_target_draw(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetsDamage => {
            resolve_choose_targets_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp => {
            resolve_choose_target_draw_then_owner_if_below_max_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess => {
            resolve_choose_target_draw_max_hp_cap_then_discard_excess(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp => {
            resolve_choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGiveOneHandEquipOrRecover => {
            resolve_choose_give_one_hand_equip_or_recover(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover => {
            resolve_choose_give_two_different_colors_or_recover(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp => {
            resolve_choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver => {
            resolve_choose_other_draw_owner_lost_hp_turn_over(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed => {
            resolve_choose_discard_one_for_sha_refund_and_draw_if_red(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetGainMaxHp => {
            resolve_choose_target_gain_max_hp(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherAddMark => {
            resolve_choose_other_add_mark(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGroup => {
            crate::effect::resolve_choose_group(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainOtherSkill => crate::effect::resolve_choose_gain_other_skill(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ChoosePlaceHandXumou => crate::effect::resolve_choose_place_hand_xumou(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ChooseOthersAddMark => crate::effect::resolve_choose_others_add_mark(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ChooseOthersInsertExtraTurn => {
            crate::effect::resolve_choose_others_insert_extra_turn(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseClanNotMaxHandLimitGain => {
            resolve_choose_clan_not_max_hand_limit_gain(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseAddCurrentShaTarget => {
            resolve_choose_add_current_sha_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseStoreHandShaInSkillPile => {
            resolve_choose_store_hand_sha_in_skill_pile(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseSpendSkillPileForDyingJiu => {
            resolve_choose_spend_skill_pile_for_dying_jiu(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCompareTargetHandCard => {
            resolve_choose_compare_target_hand_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseHandCompareTarget => {
            resolve_choose_hand_compare_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseHandComparePlayerCard => {
            resolve_choose_hand_compare_player_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherDamage => {
            resolve_choose_other_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseBoundSourceAttackRangeDamage => {
            resolve_choose_bound_source_attack_range_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherDiscardTargetCard => {
            resolve_choose_other_discard_target_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainDiscardCardByKind => {
            resolve_choose_gain_discard_card_by_kind(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn => {
            resolve_choose_gain_own_lost_discard_card_this_turn(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseMoveFieldCardTo => {
            resolve_choose_move_field_card_to(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan => {
            resolve_choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver => {
            resolve_choose_zone_priority_gain_random_card_from_each_other_then_turn_over(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver => {
            resolve_choose_gain_card_from_each_other_then_turn_over(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseHiddenSkillPileHandCards => {
            resolve_choose_hidden_skill_pile_hand_cards(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseHiddenSkillPileCards => {
            resolve_choose_hidden_skill_pile_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseChangandajianTarget => {
            resolve_choose_changandajian_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseChangandajianCard => {
            resolve_choose_changandajian_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTiaojiyanmeiGainTarget => {
            resolve_choose_tiaojiyanmei_gain_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherYinghun => {
            resolve_choose_other_yinghun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherBizheng => {
            resolve_choose_other_bizheng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherXiashu => {
            resolve_choose_other_xiashu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardSkillPileCardAddMarkedSkill => {
            let [player, target, helper_skill_def, skill] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad skill-pile discard continuation".into(),
                ));
            };
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "skill-pile discard requires one card".into(),
                ));
            };
            if cancelled || observer != PlayerId::new(*player) {
                return Err(EffectError::Illegal(
                    "invalid skill-pile discard choice".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*target),
                    helper_skill_def: engine_api::SkillDefId::new(*helper_skill_def),
                    selected: CardId::new(*selected),
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::ChooseRepoluTargetsDraw => {
            resolve_choose_repolu_targets_draw(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseMarkParityControlTarget => {
            resolve_choose_mark_parity_control_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSelectedTargetDamage => {
            resolve_choose_selected_target_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDistinctSuitCards => {
            resolve_choose_distinct_suit_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseInspectedHeartCard => {
            resolve_choose_inspected_heart_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseInspectedHeartDestination => {
            resolve_choose_inspected_heart_destination(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseYeyanMode => {
            resolve_choose_yeyan_mode(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanBigCards => {
            resolve_choose_yeyan_big_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanTargets => {
            resolve_choose_yeyan_targets(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanSingleDamage => {
            resolve_choose_yeyan_single_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanDoubleTarget => {
            resolve_choose_yeyan_double_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAllLoseHpOrRecover => {
            resolve_choose_all_lose_hp_or_recover(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardWeaponOrLoseHp => {
            resolve_choose_discard_weapon_or_lose_hp(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardOneHeOrLoseHp => {
            resolve_choose_discard_one_he_or_lose_hp(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSourceConfirmOwnerLoseHp => {
            resolve_choose_source_confirm_owner_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw => {
            resolve_choose_gain_target_card_alternate_zone_then_target_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseQixingHandCards => {
            resolve_choose_qixing_hand_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseQixingPileCards => {
            resolve_choose_qixing_pile_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseQixingWeatherCards => {
            resolve_choose_qixing_weather_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseQixingWeatherTargets => {
            resolve_choose_qixing_weather_targets(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseLinkOther => {
            resolve_choose_link_other(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltPoxiCards => {
            resolve_choose_drlt_poxi_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltJieyingTransfer => {
            resolve_choose_drlt_jieying_transfer(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltDuoruiDisableSlot => {
            resolve_choose_drlt_duorui_disable_slot(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltDuoruiSkill => {
            resolve_choose_drlt_duorui_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltZhitiEnableSlot => {
            resolve_choose_drlt_zhiti_enable_slot(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseJilueJizhiDiscard => {
            resolve_choose_jilue_jizhi_discard(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAddMissingSkill => {
            resolve_choose_add_missing_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAddSkillBoundCardTarget => {
            resolve_choose_add_skill_bound_card_target(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardMatchingEquipColorAddSkill => {
            resolve_choose_discard_matching_equip_color_add_skill(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCurrentCardUseEnhancements => {
            resolve_choose_current_card_use_enhancements(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseAddCurrentCardTarget => {
            resolve_choose_add_current_card_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget => {
            resolve_choose_add_or_remove_current_card_target(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDrawOrDiscardAddSkill => {
            resolve_choose_draw_or_discard_add_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang => {
            resolve_choose_discarded_black_basic_as_bingliang(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage => {
            resolve_choose_discard_judge_use_virtual_sha_draw_if_no_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded => {
            resolve_choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseRevealOtherHandCardRememberCategory => {
            resolve_choose_reveal_other_hand_card_remember_category(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha => {
            resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherTargetsBySkippedPhaseCountDamage => {
            resolve_choose_other_targets_by_skipped_phase_count_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOthersAfterUniformHandReveal => {
            resolve_choose_others_after_uniform_hand_reveal(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCardUserOrAddTarget => {
            resolve_choose_card_user_or_add_target(tx, observer, &payload, selections, cancelled)
        }
        _ => unreachable!("choice routed to a different continuation group"),
    }
}

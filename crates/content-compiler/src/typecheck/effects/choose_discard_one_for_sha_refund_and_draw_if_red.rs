//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `choose_discard_one_for_sha_refund_and_draw_if_red_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "choose_discard_one_for_sha_refund_and_draw_if_red" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_discard_one_for_sha_refund_and_draw_if_red(player, sha_user, card)"
                            .into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_one_for_sha_refund_and_draw_if_red card must be a card operand"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRed {
                player: check_expr(&args[0], locals)?,
                sha_user: check_expr(&args[1], locals)?,
                card,
                span: *span,
            })
        }
        "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(player, sha_user, card, restore_skill_def)".into(),
                    });
            }
            let card = check_expr(&args[2], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank card must be a card operand".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                    player: check_expr(&args[0], locals)?,
                    sha_user: check_expr(&args[1], locals)?,
                    card,
                    restore_skill_def: expect_u32(&args[3], span.start as usize)?,
                    span: *span,
                },
            )
        }
        "queue_opening_extra_play_phase" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_opening_extra_play_phase(player)".into(),
                });
            }
            Ok(StmtIr::EffectQueueOpeningExtraPlayPhase {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "convert_current_sha_to_fire_lose_hp_if_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "convert_current_sha_to_fire_lose_hp_if_damage(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "convert-current-Sha card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectConvertCurrentShaToFireLoseHpIfDamage {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        "choose_add_current_sha_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_add_current_sha_target(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose-add-current-Sha-target card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectChooseAddCurrentShaTarget {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        "choose_add_current_sha_target_within_distance" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_add_current_sha_target_within_distance(player, card, max_distance)"
                            .into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "distance-capped additional-Sha-target card must be a card operand"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseAddCurrentShaTargetWithinDistance {
                player: check_expr(&args[0], locals)?,
                card,
                max_distance: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "refund_current_sha_use" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "refund_current_sha_use(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "refund-current-Sha-use card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectRefundCurrentShaUse {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        "damage_then_draw_selected_targets" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "damage_then_draw_selected_targets(source, damage, draw)".into(),
                });
            }
            Ok(StmtIr::EffectDamageThenDrawSelectedTargets {
                source: check_expr(&args[0], locals)?,
                damage: check_expr(&args[1], locals)?,
                draw: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_repolu_targets_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_repolu_targets_draw(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseRepoluTargetsDraw {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_store_hand_sha_in_skill_pile" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_store_hand_sha_in_skill_pile(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseStoreHandShaInSkillPile {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_spend_skill_pile_for_dying_jiu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_spend_skill_pile_for_dying_jiu(player, dying)".into(),
                });
            }
            Ok(StmtIr::EffectChooseSpendSkillPileForDyingJiu {
                player: check_expr(&args[0], locals)?,
                dying: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_compare_selected_hand_card" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_compare_selected_hand_card(player, target, win_skill_def, lose_skill_def)"
                            .into(),
                    });
            }
            Ok(StmtIr::EffectChooseCompareSelectedHandCard {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                win_skill_def: expect_u32(&args[2], span.start as usize)?,
                lose_skill_def: expect_u32(&args[3], span.start as usize)?,
                span: *span,
            })
        }
        "choose_add_skill_bound_card_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_add_skill_bound_card_target(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose-add-skill-bound-card-target card must be a card operand"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseAddSkillBoundCardTarget {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        "choose_discard_matching_equip_color_add_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_discard_matching_equip_color_add_skill(player, target, skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardMatchingEquipColorAddSkill {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "use_virtual_sha_from_card_color_lock_source_if_no_sha" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_sha_from_card_color_lock_source_if_no_sha(player)".into(),
                });
            }
            Ok(StmtIr::EffectUseVirtualShaFromCardColorLockSourceIfNoSha {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_current_card_use_enhancements" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_current_card_use_enhancements(player, card, helper_skill_def)"
                        .into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "card-use-enhancement card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectChooseCurrentCardUseEnhancements {
                player: check_expr(&args[0], locals)?,
                card,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "make_current_card_unrespondable" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "make_current_card_unrespondable(player, card, helper_skill_def)"
                        .into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "current-card unrespondable card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectMakeCurrentCardUnrespondable {
                player: check_expr(&args[0], locals)?,
                card,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "choose_draw_or_discard_add_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectChooseDrawOrDiscardAddSkill {
                player: check_expr(&args[0], locals)?,
                draw_skill_def: expect_u32(&args[1], span.start as usize)?,
                discard_skill_def: expect_u32(&args[2], span.start as usize)?,
                three_way: false,
                span: *span,
            })
        }
        "choose_three_way_draw_or_discard_add_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_three_way_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectChooseDrawOrDiscardAddSkill {
                player: check_expr(&args[0], locals)?,
                draw_skill_def: expect_u32(&args[1], span.start as usize)?,
                discard_skill_def: expect_u32(&args[2], span.start as usize)?,
                three_way: true,
                span: *span,
            })
        }
        "choose_discarded_black_basic_as_bingliang" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discarded_black_basic_as_bingliang(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardedBlackBasicAsBingliang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_judge_use_virtual_sha_draw_if_no_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_discard_judge_use_virtual_sha_draw_if_no_damage(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "discard_selected_record_unique_category_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_selected_record_unique_category_target(player, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectDiscardSelectedRecordUniqueCategoryTarget {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        name @ ("choose_target_draw_or_discard_by_skill_use_count_link_if_wounded"
        | "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded") => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: format!("{name}(player, target)"),
                });
            }
            Ok(
                StmtIr::EffectChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    toggle_link: name.contains("_toggle_link_"),
                    span: *span,
                },
            )
        }
        "choose_reveal_other_hand_card_remember_category" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_reveal_other_hand_card_remember_category(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseRevealOtherHandCardRememberCategory {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_then_give_exact_record_phase_target" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_then_give_exact_record_phase_target(player, target, count)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectDrawThenGiveExactRecordPhaseTarget {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "clear_phase_gift_target" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clear_phase_gift_target(player)".into(),
                });
            }
            Ok(StmtIr::EffectClearPhaseGiftTarget {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_gain_resolved_sha_or_shan_then_optional_use_sha" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_gain_resolved_sha_or_shan_then_optional_use_sha(player, sha_card)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseGainResolvedShaOrShanThenOptionalUseSha {
                    player: check_expr(&args[0], locals)?,
                    sha_card: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_gain_all_resolved_sha_shan_then_color_rewards" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_gain_all_resolved_sha_shan_then_color_rewards(player, sha_card)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseGainAllResolvedShaShanThenColorRewards {
                player: check_expr(&args[0], locals)?,
                sha_card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "skip_current_phase" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_current_phase(player)".into(),
                });
            }
            Ok(StmtIr::EffectSkipCurrentPhase {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "finish_current_phase_abort_resolution" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "finish_current_phase_abort_resolution(player)".into(),
                });
            }
            Ok(StmtIr::EffectFinishCurrentPhaseAbortResolution {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "gain_top_card_to_skill_pile" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_top_card_to_skill_pile(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainTopCardToSkillPile {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_skill_pile_card_add_marked_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_skill_pile_card_add_marked_skill(player, target, helper_skill_def)"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardSkillPileCardAddMarkedSkill {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "gain_one_hand_from_selected_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_one_hand_from_selected_targets(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainOneHandFromSelectedTargets {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_targets_by_skipped_phase_count_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_targets_by_skipped_phase_count_damage(player, amount)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamage {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_other_targets_by_skipped_phase_count_damage_draw" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_targets_by_skipped_phase_count_damage_draw(player, amount)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "reveal_uniform_hand_draw_self_choose_others" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_uniform_hand_draw_self_choose_others(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealUniformHandDrawSelfChooseOthers {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_card_user_or_add_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_card_user_or_add_target(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectChooseCardUserOrAddTarget {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_discard_equipment_reduce_current_damage" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_equipment_reduce_current_damage(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardEquipmentReduceCurrentDamage {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_equipment_prevent_current_damage" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_equipment_prevent_current_damage(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardEquipmentPreventCurrentDamage {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_move_target_equipment_or_draw" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_move_target_equipment_or_draw(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseMoveTargetEquipmentOrDraw {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_move_target_equipment_and_or_draw" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_move_target_equipment_and_or_draw(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseMoveTargetEquipmentAndOrDraw {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(player, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_draw_or_recast_add_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseDrawOrRecastAddSkill {
                player: check_expr(&args[0], locals)?,
                draw_skill_def: expect_u32(&args[1], span.start as usize)?,
                recast_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "choose_gain_sha_for_opening_extra_play" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_sha_for_opening_extra_play(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainShaForOpeningExtraPlay {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "finish_opening_extra_play_sha_gain" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "finish_opening_extra_play_sha_gain(player)".into(),
                });
            }
            Ok(StmtIr::EffectFinishOpeningExtraPlayShaGain {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reveal_target_hand_gain_shan_armor_or_draw_two" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_target_hand_gain_shan_armor_or_draw_two(player, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectRevealTargetHandGainShanArmorOrDrawTwo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_all_equipment_choose_recover_or_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_all_equipment_choose_recover_or_damage(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveAllEquipmentChooseRecoverOrDamage {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_discard_weapon_range_hand_or_increase_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_target_discard_weapon_range_hand_or_increase_damage(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_hand_count_different_target_draw_or_random_discard" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_hand_count_different_target_draw_or_random_discard(player)"
                        .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseHandCountDifferentTargetDrawOrRandomDiscard {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "increase_current_draw_count" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "increase_current_draw_count(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectIncreaseCurrentDrawCount {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_discard_excess_then_self_discard_or_target_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_target_discard_excess_then_self_discard_or_target_draw(player)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "draw_then_choose_gain_one_from_damage_target_and_neighbors" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "draw_then_choose_gain_one_from_damage_target_and_neighbors(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectDrawThenChooseGainOneFromDamageTargetAndNeighbors {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_prevent_damage_or_lose_max_hp_increase_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_prevent_damage_or_lose_max_hp_increase_damage(player, amount)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "randomize_current_hand_compare_opponent" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "randomize_current_hand_compare_opponent(player)".into(),
                });
            }
            Ok(StmtIr::EffectRandomizeCurrentHandCompareOpponent {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "gain_highest_rank_sha_from_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_highest_rank_sha_from_event_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainHighestRankShaFromEventCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "gain_hand_compare_result_card" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_hand_compare_result_card(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainHandCompareResultCard {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_add_or_remove_current_card_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_add_or_remove_current_card_target(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectChooseAddOrRemoveCurrentCardTarget {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_hand_compare" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_hand_compare(player, win_skill_def, lose_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectChooseHandCompare {
                player: check_expr(&args[0], locals)?,
                win_skill_def: expect_u32(&args[1], span.start as usize)?,
                lose_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "choose_hand_compare_grant_target_on_win" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_hand_compare_grant_target_on_win(player, win_grant_def, lose_skill_def)"
                            .into(),
                    });
            }
            Ok(StmtIr::EffectChooseHandCompareGrantTargetOnWin {
                player: check_expr(&args[0], locals)?,
                win_grant_def: expect_u32(&args[1], span.start as usize)?,
                lose_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "exclude_current_sha_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "exclude_current_sha_target(card, target)".into(),
                });
            }
            Ok(StmtIr::EffectExcludeSavedShaTarget {
                card: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "ban_use_against" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "ban_use_against(user, target)".into(),
                });
            }
            Ok(StmtIr::EffectBanUseAgainst {
                user: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::choose_discard_one_for_sha_refund_and_draw_if_red_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

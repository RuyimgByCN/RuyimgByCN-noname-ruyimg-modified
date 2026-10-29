//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `gain_draw_cards_for_lifetime_card_action_divisors_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "gain_draw_cards_for_lifetime_card_action_divisors" => {
            if args.len() != 7 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_draw_cards_for_lifetime_card_action_divisors(player, divisor, group, divisor, group, divisor, group)".into(),
                    });
            }
            let mut divisors = [0_u32; 3];
            let mut groups = [0_u8; 3];
            for index in 0..3 {
                let ExprAst::Int(divisor) = &args[index * 2 + 1] else {
                    return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "gain_draw_cards_for_lifetime_card_action_divisors expects positive integer divisors".into(),
                        });
                };
                if *divisor <= 0 {
                    return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "gain_draw_cards_for_lifetime_card_action_divisors divisors must be positive".into(),
                        });
                }
                let ExprAst::Ident(group) = &args[index * 2 + 2] else {
                    return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "gain_draw_cards_for_lifetime_card_action_divisors expects reward group identifiers".into(),
                        });
                };
                divisors[index] = *divisor as u32;
                groups[index] =
                    lifetime_card_reward_group_value(group).ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: format!("unknown lifetime card reward group `{group}`"),
                    })?;
            }
            Ok(StmtIr::EffectGainDrawCardsForLifetimeCardActionDivisors {
                player: check_expr(&args[0], locals)?,
                divisors,
                groups,
                span: *span,
            })
        }
        "lose_hp_gain_discard_card_for_opening_extra_play" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_hp_gain_discard_card_for_opening_extra_play(player, card_kind, mandatory_skill_def)".into(),
                    });
            }
            let ExprAst::Ident(kind) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "lose_hp_gain_discard_card_for_opening_extra_play expects a card-kind name"
                            .into(),
                });
            };
            let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown card kind {kind}"),
            })?;
            let mandatory_skill_def = expect_u32(&args[2], span.start as usize)?;
            if mandatory_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "mandatory_skill_def must be a positive stable id".into(),
                });
            }
            Ok(StmtIr::EffectLoseHpGainDiscardCardForOpeningExtraPlay {
                player: check_expr(&args[0], locals)?,
                kind,
                mandatory_skill_def,
                span: *span,
            })
        }
        "choose_gain_own_lost_discard_card_this_turn" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_own_lost_discard_card_this_turn(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainOwnLostDiscardCardThisTurn {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_move_field_card_to" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_move_field_card_to(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseMoveFieldCardTo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "draw_then_optionally_distribute_hand_cards" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "draw_then_optionally_distribute_hand_cards(player, draw_count, give_cap)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectDrawThenOptionallyDistributeHandCards {
                player: check_expr(&args[0], locals)?,
                draw_count: check_expr(&args[1], locals)?,
                give_cap: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "give_selected_then_continue_distinct_recover_at_two" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_selected_then_continue_distinct_recover_at_two(player, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectGiveSelectedThenContinueDistinctRecoverAtTwo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_selected_then_recover_target" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_selected_then_recover_target(player, target, amount)".into(),
                });
            }
            Ok(StmtIr::EffectGiveSelectedThenRecoverTarget {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_other_give_then_recover" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_give_then_recover(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherGiveThenRecover {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "guanxing_top_by_falu_marks" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "guanxing_top_by_falu_marks(player, spade, heart, club, diamond)"
                        .into(),
                });
            }
            let mut parsed = [0u32; 4];
            for (slot, arg) in args[1..].iter().enumerate() {
                parsed[slot] = expect_u32(arg, span.start as usize)?;
            }
            Ok(StmtIr::EffectGuanxingTopByFaluMarks {
                player: check_expr(&args[0], locals)?,
                marks: parsed,
                span: *span,
            })
        }
        "guanxing_top" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "guanxing_top(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectGuanxingTop {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_hand_compare_between" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_hand_compare_between(player, first)".into(),
                });
            }
            Ok(StmtIr::EffectChooseHandCompareBetween {
                player: check_expr(&args[0], locals)?,
                first: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_selected_then_recover_self" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_selected_then_recover_self(player, target, amount)".into(),
                });
            }
            Ok(StmtIr::EffectGiveSelectedThenRecoverSelf {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(player, mark_def)"
                            .into(),
                });
            }
            let mark_def = expect_u32(&args[1], span.start as usize)?;
            if mark_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "mark definition must be non-zero".into(),
                });
            }
            Ok(
                StmtIr::EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                    player: check_expr(&args[0], locals)?,
                    mark_def,
                    span: *span,
                },
            )
        }
        "choose_zone_priority_gain_random_card_from_each_other_then_turn_over" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_zone_priority_gain_random_card_from_each_other_then_turn_over(player)"
                                .into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "choose_gain_card_from_each_other_then_turn_over" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_card_from_each_other_then_turn_over(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainCardFromEachOtherThenTurnOver {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "set_non_locked_skill_suppression" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "set_non_locked_skill_suppression(player, enabled)".into(),
                });
            }
            let enabled = match &args[1] {
                ExprAst::Int(0) => false,
                ExprAst::Int(1) => true,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "non-locked skill suppression enabled must be 0 or 1".into(),
                    });
                }
            };
            Ok(StmtIr::EffectSetNonLockedSkillSuppression {
                player: check_expr(&args[0], locals)?,
                enabled,
                span: *span,
            })
        }
        "discard_selected_draw_all_hand_bonus" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_selected_draw_all_hand_bonus(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardSelectedDrawAllHandBonus {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_yinghun" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_yinghun(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherYinghun {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_bizheng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_bizheng(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherBizheng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_xiashu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_xiashu(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherXiashu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "discard_equipment_slot" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "discard_equipment_slot(player, weapon|armor|horse_def|horse_off|treasure)"
                            .into(),
                });
            }
            let ExprAst::Ident(slot) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_equipment_slot expects an equipment-slot name".into(),
                });
            };
            let slot = match slot.as_str() {
                "weapon" => 1,
                "armor" => 2,
                "horse_def" => 3,
                "horse_off" => 4,
                "treasure" => 5,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: format!("unknown equipment slot {slot}"),
                    });
                }
            };
            Ok(StmtIr::EffectDiscardEquipmentSlot {
                player: check_expr(&args[0], locals)?,
                slot,
                span: *span,
            })
        }
        "store_selected_in_skill_pile" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "store_selected_in_skill_pile(player)".into(),
                });
            }
            Ok(StmtIr::EffectStoreSelectedInSkillPile {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "end_match_for_player_camp_if_skill_pile_at_least" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "end_match_for_player_camp_if_skill_pile_at_least(player, pile_skill_def, count)"
                                .into(),
                    });
            }
            let pile_skill_def = expect_u32(&args[1], span.start as usize)?;
            let count = expect_u32(&args[2], span.start as usize)?;
            if pile_skill_def == 0 || count == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "pile skill definition and count must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectEndMatchForCampIfSkillPileAtLeast {
                player: check_expr(&args[0], locals)?,
                pile_skill_def,
                count,
                span: *span,
            })
        }
        "add_skill_until_turn_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_skill_until_turn_end(player, skill_def)".into(),
                });
            }
            let skill_def = expect_u32(&args[1], span.start as usize)?;
            if skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skill definition must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectAddSkillUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                skill_def,
                span: *span,
            })
        }
        "record_active_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "record_active_target(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectRecordActiveTarget {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_discard_weapon_or_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_weapon_or_lose_hp(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardWeaponOrLoseHp {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "replace_draw_with_distinct_suit_selection" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "replace_draw_with_distinct_suit_selection(player, count)".into(),
                });
            }
            let count = u8::try_from(expect_u32(&args[1], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "distinct-suit reveal count must be 1..255".into(),
                })?;
            Ok(StmtIr::EffectReplaceDrawWithDistinctSuitSelection {
                player: check_expr(&args[0], locals)?,
                count,
                span: *span,
            })
        }
        "inspect_target_hand_choose_heart_discard_or_top" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "inspect_target_hand_choose_heart_discard_or_top(player, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectInspectTargetHandChooseHeartDiscardOrTop {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_all_lose_hp_or_recover" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_all_lose_hp_or_recover(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseAllLoseHpOrRecover {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_link_other" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_link_other(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseLinkOther {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_gain_card_from_each_other_replace_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_card_from_each_other_replace_draw(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainCardFromEachOtherReplaceDraw {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_targets_damage" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_targets_damage(player, min_count, max_count, other|male_other|female_other|any|min_hand|same_hp|hand_empty|wounded|hand_lte_chooser, amount)"
                                .into(),
                    });
            }
            let min_count = u8::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "damage target minimum must be 1..255".into(),
                })?;
            let max_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                .ok()
                .filter(|count| *count >= min_count)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "damage target maximum must be within min_count..255".into(),
                })?;
            Ok(StmtIr::EffectChooseTargetsDamage {
                player: check_expr(&args[0], locals)?,
                min_count,
                max_count,
                scope: expect_draw_target_scope(&args[3], span.start as usize)?,
                amount: check_expr(&args[4], locals)?,
                span: *span,
            })
        }
        "draw_by_play_phase_card_category_count" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_by_play_phase_card_category_count(player)".into(),
                });
            }
            Ok(StmtIr::EffectDrawByPlayPhaseCardCategoryCount {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        _ => super::gain_draw_cards_for_lifetime_card_action_divisors_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

//! 效果语句类型检查链：通用效果原语（选择与判定类见下一环 `resolve_zhizheng_choices`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "discard_all_hand_and_equip" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_hand_and_equip(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardAllHandAndEquip {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "remove_all_skills" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_all_skills(player)".into(),
                });
            }
            Ok(StmtIr::EffectRemoveAllSkills {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "force_others_use_sha_on_nearest_or_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "force_others_use_sha_on_nearest_or_lose_hp(player)".into(),
                });
            }
            Ok(StmtIr::EffectForceOthersUseShaOnNearestOrLoseHp {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reveal_discard_suit_gain_or_use" => {
            check_reveal_discard_suit_gain_or_use(args, *span, locals)
        }
        "draw_selected" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_selected(player)".into(),
                });
            }
            Ok(StmtIr::EffectDrawSelected {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_each_by_seat" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_each_by_seat(first, second, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawEachBySeat {
                first: check_expr(&args[0], locals)?,
                second: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "draw_then_give_one" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_one(player, target, count, any_hand_equip|drawn_only|any_hand_equip_use_equipment)".into(),
                    });
            }
            let ExprAst::Ident(scope) = &args[3] else {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_one scope must be any_hand_equip, drawn_only, or any_hand_equip_use_equipment".into(),
                    });
            };
            let (drawn_only, use_given_equipment) = match scope.as_str() {
                    "any_hand_equip" => (false, false),
                    "drawn_only" => (true, false),
                    "any_hand_equip_use_equipment" => (false, true),
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "draw_then_give_one scope must be any_hand_equip, drawn_only, or any_hand_equip_use_equipment".into(),
                        })
                    }
                };
            Ok(StmtIr::EffectDrawThenGiveOne {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                drawn_only,
                use_given_equipment,
                span: *span,
            })
        }
        "draw_then_take_one" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_then_take_one(player, taker, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawThenTakeOne {
                player: check_expr(&args[0], locals)?,
                taker: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "take_one_card" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_one_card(from, to, hand_equip|equip|hand_equip_judge)".into(),
                });
            }
            let ExprAst::Ident(scope) = &args[2] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_one_card scope must be hand_equip, equip, or hand_equip_judge"
                        .into(),
                });
            };
            if scope == "hand_equip_judge" {
                return Ok(StmtIr::EffectTakeOneScopedCard {
                    from: check_expr(&args[0], locals)?,
                    to: check_expr(&args[1], locals)?,
                    scope: TargetCardScopeIr::HandEquipJudge,
                    span: *span,
                });
            }
            let equip_only = match scope.as_str() {
                "hand_equip" => false,
                "equip" => true,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "take_one_card scope must be hand_equip, equip, or hand_equip_judge"
                                .into(),
                    })
                }
            };
            Ok(StmtIr::EffectTakeOneCard {
                from: check_expr(&args[0], locals)?,
                to: check_expr(&args[1], locals)?,
                equip_only,
                span: *span,
            })
        }
        "take_one_event_target_hand" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_one_event_target_hand(player)".into(),
                });
            }
            Ok(StmtIr::EffectTakeOneHandCard {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "gain_all_skill_pile_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_all_skill_pile_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainAllSkillPileCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "gain_other_skill_pile_cards" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_other_skill_pile_cards(player, target, skill_def)".into(),
                });
            }
            let skill_def = expect_u32(&args[2], span.start as usize)?;
            if skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_other_skill_pile_cards skill_def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectGainOtherSkillPileCards {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill_def,
                span: *span,
            })
        }
        "lose_hp_double_current_damage_then_replace" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "lose_hp_double_current_damage_then_replace(player, replacement_skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectLoseHpDoubleCurrentDamageThenReplace {
                player: check_expr(&args[0], locals)?,
                replacement_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "take_target_hand_then_repeat_until_max" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_target_hand_then_repeat_until_max(player, target, skill)".into(),
                });
            }
            Ok(StmtIr::EffectTakeTargetHandThenRepeatUntilMax {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "gain_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_event_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainEventCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "turn_over" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "turn_over(player)".into(),
                });
            }
            Ok(StmtIr::EffectTurnOver {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "restore_character_state" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "restore_character_state(player)".into(),
                });
            }
            Ok(StmtIr::EffectRestoreCharacterState {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "discard_all_hand_equip_judge" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_hand_equip_judge(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardAllHandEquipJudge {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "damage_then_discard_all_judge" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "damage_then_discard_all_judge(player)".into(),
                });
            }
            Ok(StmtIr::EffectDamageThenDiscardAllJudge {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "replace_draw_gain_top_two_add_skills_if_different_colors" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "replace_draw_gain_top_two_add_skills_if_different_colors(player, first_skill_def, second_skill_def)"
                                .into(),
                    });
            }
            Ok(
                StmtIr::EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player: check_expr(&args[0], locals)?,
                    first_skill_def: expect_u32(&args[1], span.start as usize)?,
                    second_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                },
            )
        }
        "remove_skill_by_def" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_skill_by_def(player, skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectRemoveSkillByDef {
                player: check_expr(&args[0], locals)?,
                skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "reveal_top_gain_non_basic_or_swap_basic" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_top_gain_non_basic_or_swap_basic(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealTopGainNonBasicOrSwapBasic {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reveal_top_gain_if_color" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_top_gain_if_color(player, red|black)".into(),
                });
            }
            let color_mask = match &args[1] {
                ExprAst::Ident(color) if color == "red" => 1,
                ExprAst::Ident(color) if color == "black" => 2,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_top_gain_if_color color must be red or black".into(),
                    });
                }
            };
            Ok(StmtIr::EffectRevealTopGainIfColor {
                player: check_expr(&args[0], locals)?,
                color_mask,
                span: *span,
            })
        }
        "prevent_current_damage" => {
            if !args.is_empty() {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "prevent_current_damage()".into(),
                });
            }
            Ok(StmtIr::EffectPreventCurrentDamage { span: *span })
        }
        "take_hand" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_hand(from, to)".into(),
                });
            }
            Ok(StmtIr::EffectTakeHand {
                from: check_expr(&args[0], locals)?,
                to: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_all_hand" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_all_hand(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveAllHand {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "gain_card_if_discard" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_card_if_discard(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectGainCardIfDiscard {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "reveal_hand_to_player" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand_to_player(player, observer)".into(),
                });
            }
            Ok(StmtIr::EffectRevealHandToPlayer {
                player: check_expr(&args[0], locals)?,
                observer: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "lose_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "lose_hp(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectLoseHp {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "add_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_skill(player, skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectAddSkill {
                player: check_expr(&args[0], locals)?,
                skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "add_bound_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_bound_skill(player, skill_def, target)".into(),
                });
            }
            Ok(StmtIr::EffectAddBoundSkill {
                player: check_expr(&args[0], locals)?,
                skill_def: expect_u32(&args[1], span.start as usize)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "replace_skill_with_bound_after_resolution_if_target_alive" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "replace_skill_with_bound_after_resolution_if_target_alive(player, skill, skill_def, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    skill_def: expect_u32(&args[2], span.start as usize)?,
                    target: check_expr(&args[3], locals)?,
                    span: *span,
                },
            )
        }
        "remove_skill" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_skill(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectRemoveSkill {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "refresh_skill" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "refresh_skill(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectRefreshSkill {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "tag_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "tag_event_cards(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectTagEventCards {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "untag_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "untag_event_cards(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectUntagEventCards {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "storage_set" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "storage_set(skill_instance, key, value)".into(),
                });
            }
            Ok(StmtIr::EffectStorageSet {
                skill: check_expr(&args[0], locals)?,
                key: check_expr(&args[1], locals)?,
                value: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "draw_all_event_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_all_event_targets(count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawAllEventTargets {
                count: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "clanshenjun_reveal" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanshenjun_reveal(self, event_card, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanshenjunReveal {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "clanshenjun_use" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanshenjun_use(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanshenjunUse {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "clanyuzhi_reveal" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanyuzhi_reveal(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanyuzhiReveal {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "clanyuzhi_round_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanyuzhi_round_end(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanyuzhiRoundEnd {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_zhizheng_choices::check_effect_stmt(name, args, span, locals),
    }
}

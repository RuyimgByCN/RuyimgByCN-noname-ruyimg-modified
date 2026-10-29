//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `resolve_beizhu_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "give_all_hand_then_choose_use_one_or_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_all_hand_then_choose_use_one_or_damage(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveAllHandThenChooseUseOneOrDamage {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_category_then_target_give_matching_or_sha_unrespondable" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_category_then_target_give_matching_or_sha_unrespondable(player, card, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                },
            )
        }
        "choose_category_then_target_give_matching_or_damage_plus_one" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_category_then_target_give_matching_or_damage_plus_one(player, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(player, source)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_lose_hp_use_virtual_sha_by_basic_variants" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_lose_hp_use_virtual_sha_by_basic_variants(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseLoseHpUseVirtualShaByBasicVariants {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "suppress_non_locked_skills_until_turn_end" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "suppress_non_locked_skills_until_turn_end(player)".into(),
                });
            }
            Ok(StmtIr::EffectSuppressNonLockedSkillsUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_recover_target" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_recover_target(chooser, target, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseRecoverTarget {
                chooser: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_basic_then_target_discard_equip_or_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_discard_basic_then_target_discard_equip_or_damage(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_current_sha_target_discard_redirect_to_owner" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectChooseCurrentShaTargetDiscardRedirectToOwner {
                owner: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                target: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_target_gain_bottom_card" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_gain_bottom_card(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetGainBottomCard {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_black_gain_repeat" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_black_gain_repeat(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeBlackGainRepeat {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_black_gain_repeat_protect_hand_limit" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_black_gain_repeat_protect_hand_limit(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeBlackGainRepeatProtectHandLimit {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_same_color_gain_repeat" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_same_color_gain_repeat(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeSameColorGainRepeat {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_current_trick_targets_to_nullify" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_current_trick_targets_to_nullify(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectChooseCurrentTrickTargetsToNullify {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "use_virtual_juedou_second_on_first" => {
            if !args.is_empty() {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_juedou_second_on_first()".into(),
                });
            }
            Ok(StmtIr::EffectUseVirtualJuedouSecondOnFirst { span: *span })
        }
        effect @ ("choose_discard_target_card" | "choose_optional_discard_target_card") => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: format!(
                        "{effect}(player, target, hand|equip|hand_equip|hand_equip_judge)"
                    ),
                });
            }
            let ExprAst::Ident(scope) = &args[2] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: format!(
                        "{effect} scope must be hand, equip, hand_equip, or hand_equip_judge"
                    ),
                });
            };
            let scope = match scope.as_str() {
                "hand" => TargetCardScopeIr::Hand,
                "equip" => TargetCardScopeIr::Equip,
                "hand_equip" => TargetCardScopeIr::HandEquip,
                "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                "equip_judge" => TargetCardScopeIr::EquipJudge,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: format!(
                            "{effect} scope must be hand, equip, hand_equip, or hand_equip_judge"
                        ),
                    });
                }
            };
            let player = check_expr(&args[0], locals)?;
            let target = check_expr(&args[1], locals)?;
            Ok(if effect == "choose_discard_target_card" {
                StmtIr::EffectChooseDiscardTargetCard {
                    player,
                    target,
                    scope,
                    span: *span,
                }
            } else {
                StmtIr::EffectChooseOptionalDiscardTargetCard {
                    player,
                    target,
                    scope,
                    span: *span,
                }
            })
        }
        "reveal_hand_choose_discard_color" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand_choose_discard_color(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealHandChooseDiscardColor {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reveal_hand" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealHand {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_linked_players" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_linked_players(count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawLinkedPlayers {
                count: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_target_cards_recover_same_category" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_target_cards_recover_same_category(player, target, count, amount)".into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardTargetCardsRecoverSameCategory {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_scskuiji_discard" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_scskuiji_discard(player, target, count)".into(),
                });
            }
            Ok(StmtIr::EffectChooseScskuijiDiscard {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_pskuiji_discard" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_pskuiji_discard(player, target, count)".into(),
                });
            }
            Ok(StmtIr::EffectChoosePskuijiDiscard {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_compare_selected_hand_card_second_failure_seat_damage" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_compare_selected_hand_card_second_failure_seat_damage(player, target, draw_count, failure_count, lose_hp, damage)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    draw_count: check_expr(&args[2], locals)?,
                    failure_count: check_expr(&args[3], locals)?,
                    lose_hp: check_expr(&args[4], locals)?,
                    damage: check_expr(&args[5], locals)?,
                    span: *span,
                },
            )
        }
        "judge_prevent_damage_lose_max_hp_on_nonheart" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_prevent_damage_lose_max_hp_on_nonheart(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectJudgePreventDamageLoseMaxHpOnNonHeart {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_gain_top_cards_by_suit" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_gain_top_cards_by_suit(player, count, spade|heart|club|diamond)"
                            .into(),
                });
            }
            let count = u8::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "top-card view count must be 1..255".into(),
                })?;
            let suit = match &args[2] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "top-card selection suit must be spade, heart, club, or diamond"
                            .into(),
                    });
                }
            };
            Ok(StmtIr::EffectChooseGainTopCardsBySuit {
                player: check_expr(&args[0], locals)?,
                count,
                suit,
                span: *span,
            })
        }
        "choose_other_draw_by_hand_count" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_draw_by_hand_count(player, normal_count, empty_count, times)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDrawByHandCount {
                player: check_expr(&args[0], locals)?,
                normal_count: check_expr(&args[1], locals)?,
                empty_count: check_expr(&args[2], locals)?,
                times: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "discard_one_hand_or_equip" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_one_hand_or_equip(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardOneHandOrEquip {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_one_then_gain_target_equipment" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_then_gain_target_equipment(player, target, weapon|armor|horse_def|horse_off|treasure)".into(),
                    });
            }
            let ExprAst::Ident(slot) = &args[2] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "equipment slot must be weapon, armor, horse_def, horse_off, or treasure"
                            .into(),
                });
            };
            let slot = match slot.as_str() {
                "weapon" => 1,
                "armor" => 2,
                "horse_def" => 3,
                "horse_off" => 4,
                "treasure" => 5,
                _ => return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "equipment slot must be weapon, armor, horse_def, horse_off, or treasure"
                            .into(),
                }),
            };
            Ok(StmtIr::EffectChooseDiscardOneThenGainTargetEquipment {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                slot,
                span: *span,
            })
        }
        "choose_target_draw" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_draw(player, count, other|male_other|female_other|any|min_hand|same_hp)"
                                .into(),
                    });
            }
            let scope = expect_draw_target_scope(&args[2], span.start as usize)?;
            Ok(StmtIr::EffectChooseTargetDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                scope,
                span: *span,
            })
        }
        "choose_targets_draw" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_targets_draw(player, count, target_count, other|male_other|female_other|any|min_hand|same_hp)".into(),
                    });
            }
            let target_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "draw target count must be 1..255".into(),
                })?;
            let scope = expect_draw_target_scope(&args[3], span.start as usize)?;
            Ok(StmtIr::EffectChooseTargetsDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                target_count,
                scope,
                span: *span,
            })
        }
        "choose_optional_targets_draw" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_optional_targets_draw(player, count, min_count, max_count, scope)"
                            .into(),
                });
            }
            let min_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "draw target minimum must be 1..255".into(),
                })?;
            let max_count = u8::try_from(expect_int(&args[3], span.start as usize)?)
                .ok()
                .filter(|count| *count >= min_count)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "draw target maximum must be min_count..255".into(),
                })?;
            Ok(StmtIr::EffectChooseOptionalTargetsDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                min_count,
                max_count,
                scope: expect_draw_target_scope(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "reveal_target_hand_sha_or_discard_bind" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectRevealTargetHandShaOrDiscardBind {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "draw_remembered_hand_cards" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_remembered_hand_cards(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawRememberedHandCards {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_remembered_hand_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_remembered_hand_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardRememberedHandCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_beizhu_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

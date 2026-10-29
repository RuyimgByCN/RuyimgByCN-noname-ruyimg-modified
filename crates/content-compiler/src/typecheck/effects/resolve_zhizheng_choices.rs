//! 效果语句类型检查链：选择（choose_*）与判定（judge_*）类效果原语。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "choose_other_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_damage(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDamage {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_other_discard_target_card" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_discard_target_card(player, equip|hand_equip|hand_equip_judge|equip_judge)".into(),
                    });
            }
            let ExprAst::Ident(scope) = &args[1] else {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_discard_target_card scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                    });
            };
            let scope = match scope.as_str() {
                "equip" => TargetCardScopeIr::Equip,
                "hand_equip" => TargetCardScopeIr::HandEquip,
                "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                "equip_judge" => TargetCardScopeIr::EquipJudge,
                _ => {
                    return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "choose_other_discard_target_card scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                        });
                }
            };
            Ok(StmtIr::EffectChooseOtherDiscardTargetCard {
                player: check_expr(&args[0], locals)?,
                scope,
                span: *span,
            })
        }
        "judge_gain_equipment_by_color" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_gain_equipment_by_color(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeGainEquipmentByColor {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_recover_on_suit" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_recover_on_suit(player, spade|heart|club|diamond, amount)"
                        .into(),
                });
            }
            let suit = match &args[1] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judgment suit must be spade, heart, club, or diamond".into(),
                    });
                }
            };
            Ok(StmtIr::EffectJudgeRecoverOnSuit {
                player: check_expr(&args[0], locals)?,
                suit,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "judge_recover_other_on_suit" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_recover_other_on_suit(judge_player, recover_player, spade|heart|club|diamond, amount)"
                            .into(),
                    });
            }
            let suit = match &args[2] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judgment suit must be spade, heart, club, or diamond".into(),
                    });
                }
            };
            Ok(StmtIr::EffectJudgeRecoverOtherOnSuit {
                judge_player: check_expr(&args[0], locals)?,
                recover_player: check_expr(&args[1], locals)?,
                suit,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "judge_damage_other_on_suit" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_damage_other_on_suit(player, spade|heart|club|diamond, amount, normal|fire|thunder)"
                            .into(),
                    });
            }
            let suit = match &args[1] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judgment suit must be spade, heart, club, or diamond".into(),
                    });
                }
            };
            let nature = match &args[3] {
                ExprAst::Ident(name) if name == "normal" => 0,
                ExprAst::Ident(name) if name == "fire" => 1,
                ExprAst::Ident(name) if name == "thunder" => 2,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage nature must be normal, fire, or thunder".into(),
                    });
                }
            };
            Ok(StmtIr::EffectJudgeDamageOtherOnSuit {
                player: check_expr(&args[0], locals)?,
                suit,
                amount: check_expr(&args[2], locals)?,
                nature,
                span: *span,
            })
        }
        "choose_discard_or_fire_damage_skip_phases" => {
            check_choose_discard_or_fire_damage_skip_phases(args, *span, locals)
        }
        "choose_other_hand_store_skill_pile" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_hand_store_skill_pile(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherHandStoreSkillPile {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_discard_one_he_or_lose_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_one_he_or_lose_hp(player, lose_amount)".into(),
                });
            }
            let lose_amount = expect_u32(&args[1], span.start as usize)?;
            if lose_amount == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_one_he_or_lose_hp lose_amount must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardOneHeOrLoseHp {
                player: check_expr(&args[0], locals)?,
                lose_amount,
                span: *span,
            })
        }
        "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(source, target)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_source_confirm_owner_lose_hp" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_source_confirm_owner_lose_hp(owner, source, amount)".into(),
                });
            }
            let amount = expect_u32(&args[2], span.start as usize)?;
            if amount == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_source_confirm_owner_lose_hp amount must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectChooseSourceConfirmOwnerLoseHp {
                owner: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                amount,
                span: *span,
            })
        }
        "choose_end_extra_draw_or_play_by_used_suits" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_end_extra_draw_or_play_by_used_suits(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseEndExtraDrawOrPlayByUsedSuits {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_target_draw_to_max_hp_cap" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_draw_to_max_hp_cap(player, cap)".into(),
                });
            }
            let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|cap| *cap > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "maximum-HP draw cap must be a positive integer".into(),
                })?;
            Ok(StmtIr::EffectChooseTargetDrawToMaxHpCap {
                player: check_expr(&args[0], locals)?,
                cap,
                span: *span,
            })
        }
        "choose_other_draw_owner_lost_hp_turn_over" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_draw_owner_lost_hp_turn_over(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDrawOwnerLostHpTurnOver {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_gain_target_card_alternate_zone_then_target_draw" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_gain_target_card_alternate_zone_then_target_draw(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_discard_target_card_resolve_current_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_target_card_resolve_current_sha(source, card, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardTargetCardResolveCurrentSha {
                source: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_target_card_resolve_current_sha_nonbasic_bonus" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_target_card_resolve_current_sha_nonbasic_bonus(source, card, target, bonus_skill_def)"
                                .into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                    source: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    bonus_skill_def: expect_u32(&args[3], span.start as usize)?,
                    span: *span,
                },
            )
        }
        "choose_discard_hand_add_skill_by_color" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_add_skill_by_color(player, red_skill_def, black_skill_def)".into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardHandAddSkillByColor {
                player: check_expr(&args[0], locals)?,
                red_skill_def: expect_u32(&args[1], span.start as usize)?,
                black_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "judge_cancel_current_sha_target_on_red" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_cancel_current_sha_target_on_red(player, card, target)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeCancelCurrentShaTargetOnRed {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_draw_or_recover" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_draw_or_recover(player, draw_count, recover_count, allow_cancel)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseDrawOrRecover {
                player: check_expr(&args[0], locals)?,
                draw_count: check_expr(&args[1], locals)?,
                recover_count: check_expr(&args[2], locals)?,
                allow_cancel: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_draw_or_discard_target_card_record_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_draw_or_discard_target_card_record_sha(player, card, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseDrawOrDiscardTargetCardRecordSha {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)"
                        .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "choose_discard_hand_equip_then_recover" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_hand_equip_then_recover(player, count, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardHandEquipThenRecover {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_zhizheng_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

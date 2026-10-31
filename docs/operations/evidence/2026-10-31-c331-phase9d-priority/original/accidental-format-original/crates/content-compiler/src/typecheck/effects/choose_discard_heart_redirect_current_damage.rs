//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `choose_discard_heart_redirect_current_damage_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "choose_discard_heart_redirect_current_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_heart_redirect_current_damage(player, draw_skill_def)"
                        .into(),
                });
            }
            let draw_skill_def = expect_u32(&args[1], span.start as usize)?;
            if draw_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw skill def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveTianxiang {
                player: check_expr(&args[0], locals)?,
                draw_skill_def,
                span: *span,
            })
        }
        "choose_discard_heart_cancel_damage_branch" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_heart_cancel_damage_branch(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRetianxiang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_link_other_same_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_link_other_same_hp(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLianzhou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_give_hand_equip_to_event_player" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_hand_equip_to_event_player(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStddechong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_gain_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_gain_event_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStdjujian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_add_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_add_skill(player, skill_def)".into(),
                });
            }
            let grant_def = expect_u32(&args[1], span.start as usize)?;
            if grant_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "granted skill def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveHfjieying {
                player: check_expr(&args[0], locals)?,
                grant_def,
                span: *span,
            })
        }
        "reveal_choose_category_distribute_self_threshold_hp_loss" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)".into(),
                    });
            }
            let reveal_count = bounded_u8(&args[1], "reveal_count")?;
            let recipient_cap = bounded_u8(&args[2], "recipient_cap")?;
            if reveal_count == 0 || recipient_cap == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_count and recipient_cap must be positive".into(),
                });
            }
            Ok(
                StmtIr::EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
                    player: check_expr(&args[0], locals)?,
                    reveal_count,
                    recipient_cap,
                    span: *span,
                },
            )
        }
        "choose_other_random_gain_region_then_draw_compare_color_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "choose_gain_target_hand_ceil_half_difference" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_target_hand_ceil_half_difference(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainTargetHandCeilHalfDifference {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_discard_hand_range" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_hand_range(player, min, max)".into(),
                });
            }
            let min = expect_u32(&args[1], span.start as usize)?;
            let max = expect_u32(&args[2], span.start as usize)?;
            if min == 0 || max < min {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard range requires 0 < min <= max".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardHandRange {
                player: check_expr(&args[0], locals)?,
                min,
                max,
                span: *span,
            })
        }
        _ => super::choose_discard_heart_redirect_current_damage_resolvers::check_effect_stmt(
            name, args, span, locals,
        ),
    }
}

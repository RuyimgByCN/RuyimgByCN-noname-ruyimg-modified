//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `set_draw_phase_bottom_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "set_draw_phase_bottom" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "set_draw_phase_bottom(player)".into(),
                });
            }
            Ok(StmtIr::EffectSetDrawPhaseFromBottom {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_cards_from_bottom" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_cards_from_bottom(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawFromBottom {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "draw_cards" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_cards(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "recover_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recover_hp(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectRecover {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "recover_hp_as" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recover_hp_as(source, player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectRecoverAs {
                source: check_expr(&args[0], locals)?,
                player: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_redirect_current_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_redirect_current_sha(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardRedirectCurrentSha {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "judge_set_skill_mark_to_rank" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_set_skill_mark_to_rank(player, skill)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeSetSkillMarkToRank {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_discard_one_from_each_linked" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_one_from_each_linked(player, skill)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardOneFromEachLinked {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "disable_skill_by_def_until_turn_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "disable_skill_by_def_until_turn_end(player, skill_def)".into(),
                });
            }
            let skill_def = match &args[1] {
                ExprAst::Int(value) if *value >= 0 => *value as u32,
                ExprAst::U32(value) => *value,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skill_def must be a nonnegative stable id".into(),
                    });
                }
            };
            Ok(StmtIr::EffectDisableSkillByDefUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                skill_def,
                span: *span,
            })
        }
        "target_discard_then_duel_if_non_sha_and_hp_gte_owner" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_lose_hp_discard_hand_sha_if_basic" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_lose_hp_discard_hand_sha_if_basic(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetLoseHpDiscardHandShaIfBasic {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reset_linked_selected_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reset_linked_selected_targets(source)".into(),
                });
            }
            Ok(StmtIr::EffectResetLinkedSelectedTargets {
                source: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "deal_damage" => {
            if !(3..=4).contains(&args.len()) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "deal_damage(source, target, amount[, normal|fire|thunder])".into(),
                });
            }
            let nature = match args.get(3) {
                None => 0,
                Some(ExprAst::Ident(name)) if name == "normal" => 0,
                Some(ExprAst::Ident(name)) if name == "fire" => 1,
                Some(ExprAst::Ident(name)) if name == "thunder" => 2,
                Some(_) => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage nature must be normal, fire, or thunder".into(),
                    });
                }
            };
            Ok(StmtIr::EffectDamage {
                source: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                nature,
                span: *span,
            })
        }
        "deal_damage_ban_active_if_dying" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "deal_damage_ban_active_if_dying(source, target, amount, self_skill)"
                        .into(),
                });
            }
            if !matches!(&args[3], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "damage-linked active ban must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectDealDamageBanActiveIfDying {
                source: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                skill: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "spjungong_discard_or_lose_then_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spjungong_discard_or_lose_then_sha(player, target, self_skill)"
                        .into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "峻攻 must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectSpjungongDiscardOrLoseThenSha {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "ban_active_until_turn_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "ban_active_until_turn_end(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "turn-end active ban must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectBanActiveUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "spwanwei_active" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spwanwei_active(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectSpwanweiActive {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "spwanwei_dying" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spwanwei_dying(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectSpwanweiDying {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::set_draw_phase_bottom_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

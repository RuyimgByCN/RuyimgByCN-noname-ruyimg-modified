//! 效果语句类型检查链：具名技能结算器（resolve_*）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_zhizheng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhizheng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhizheng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_lijun" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_lijun(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLijun {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_liangyin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_liangyin(player, stage, kongsheng_def)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if !matches!(stage, 0 | 3) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_liangyin stage must be 0 or 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveLiangyin {
                player: check_expr(&args[0], locals)?,
                kongsheng_def: expect_u32(&args[2], span.start as usize)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_reliangyin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_reliangyin(player, self_skill, mode)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_reliangyin must reference self_skill".into(),
                });
            }
            let mode = expect_u32(&args[2], span.start as usize)?;
            Ok(StmtIr::EffectResolveReliangyin {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                mode,
                span: *span,
            })
        }
        "resolve_kongsheng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_kongsheng(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if !matches!(stage, 0 | 2) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_kongsheng stage must be 0 or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveKongsheng {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_pottuntian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pottuntian(player, action, zaoxian_def)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pottuntian action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolvePottuntian {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                zaoxian_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potjixi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potjixi(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potjixi action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotjixi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potceduan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potceduan(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potceduan action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotceduan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_pottaishici" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "resolve_pottaishici(player, target, action, hanzhan_def, zhanlie_def)"
                            .into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pottaishici action must be 0 through 5".into(),
                });
            }
            Ok(StmtIr::EffectResolvePottaishici {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                hanzhan_def: expect_u32(&args[3], span.start as usize)?,
                zhanlie_def: expect_u32(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_mbzhonghui" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhonghui(player, target, action, amount, mbsizi_def, mbkechang_def)".into(),
                    });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 7 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbzhonghui action must be 0 through 7".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbzhonghui {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                amount: check_expr(&args[3], locals)?,
                mbsizi_def: expect_u32(&args[4], span.start as usize)?,
                mbkechang_def: expect_u32(&args[5], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potjiejie" => {
            if args.len() != 8 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjiejie(player, actor, used_def, spade_def, heart_def, club_def, diamond_def, qingshi_def)".into(),
                    });
            }
            Ok(StmtIr::EffectResolvePotjiejie {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                used_def: expect_u32(&args[2], span.start as usize)?,
                suit_defs: [
                    expect_u32(&args[3], span.start as usize)?,
                    expect_u32(&args[4], span.start as usize)?,
                    expect_u32(&args[5], span.start as usize)?,
                    expect_u32(&args[6], span.start as usize)?,
                ],
                qingshi_def: expect_u32(&args[7], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potqingshi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potqingshi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotqingshi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_potweiyan" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potweiyan(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 10 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potweiyan action must be 0 through 10".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotweiyan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potzhouyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potzhouyu(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 7 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potzhouyu action must be 0 through 7".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotzhouyu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potlvyi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potlvyi(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 9 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potlvyi action must be 0 through 9".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotlvyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potcaoshuang" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potcaoshuang(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 11 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potcaoshuang action must be 0 through 11".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotcaoshuang {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potyuji" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potyuji(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 8 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potyuji action must be 0 through 8".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotyuji {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_rehengjiang" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def)"
                        .into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rehengjiang action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveRehengjiang {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                hand_limit_def: expect_u32(&args[3], span.start as usize)?,
                draw_def: expect_u32(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_dcqiaomeng" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcqiaomeng(player, card, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcqiaomeng {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        _ => super::qieyi_phase_start::check_effect_stmt(name, args, span, locals),
    }
}

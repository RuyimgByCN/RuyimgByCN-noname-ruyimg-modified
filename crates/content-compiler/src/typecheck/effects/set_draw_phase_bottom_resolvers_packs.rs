//! 效果语句类型检查链：带包前缀（sp/dc/ol/xin/gz/tw/mb/sb 等）的具名技能结算器。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_sbbiyue" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sbbiyue(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSbbiyue {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_gzbuqu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gzbuqu(player, delta, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGzbuqu {
                player: check_expr(&args[0], locals)?,
                delta: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_gzbuqu_recover" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gzbuqu_recover(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGzbuquRecover {
                player: check_expr(&args[0], locals)?,
                stage: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_twjuxiang" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjuxiang(player, event_player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwjuxiang {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_twxiongjun" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twxiongjun(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwxiongjun {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcqiaolve" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcqiaolve(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcqiaolve {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xinfu_qiai" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinfu_qiai(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinfuQiai {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_olwenren" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olwenren(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlwenren {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_spolaige" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spolaige(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpolaige {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_olzhijie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olzhijie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlzhijie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_olsuyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olsuyi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlsuyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_olqiaozhi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olqiaozhi(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlqiaozhi {
                player: check_expr(&args[0], locals)?,
                stage: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dczhee" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dczhee(player, mode)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDczhee {
                player: check_expr(&args[0], locals)?,
                mode: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dchaoxian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dchaoxian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDchaoxian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbzhenann" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbzhenann(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbzhenann {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcanxu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcanxu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcanxu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_spxizhan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spxizhan(player, phase_player, fangzong_skill_def)".into(),
                });
            }
            let fangzong = expect_u32(&args[2], span.start as usize)?;
            if fangzong == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spxizhan fangzong_skill_def must be nonzero".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpxizhan {
                player: check_expr(&args[0], locals)?,
                phase_player: check_expr(&args[1], locals)?,
                fangzong_skill_def: fangzong,
                span: *span,
            })
        }
        "resolve_spsongshu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "resolve_spsongshu(player, phase_player, renku_skill_def, block_skill_def)"
                            .into(),
                });
            }
            let renku = expect_u32(&args[2], span.start as usize)?;
            let block = expect_u32(&args[3], span.start as usize)?;
            if renku == 0 || block == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spsongshu skill defs must be nonzero".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpsongshu {
                player: check_expr(&args[0], locals)?,
                phase_player: check_expr(&args[1], locals)?,
                renku_skill_def: renku,
                block_skill_def: block,
                span: *span,
            })
        }
        "resolve_spcunsi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spcunsi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpcunsi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xinmingshi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinmingshi(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinmingshi {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spyinju" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spyinju(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpyinju {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spchijie" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spchijie(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpchijie {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_spshangyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshangyi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpshangyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spyajun" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spyajun(player, less_def, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpyajun {
                player: check_expr(&args[0], locals)?,
                less_def: expect_u32(&args[1], span.start as usize)?,
                stage: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_spzundi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spzundi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpzundi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xinyangjie" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinyangjie(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinyangjie {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbdiancai" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbdiancai(player, actor)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbdiancai {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spqiai" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spqiai(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpqiai {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spshanxi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi(player, target, stage)".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpshanxi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_spshanxi_suoming" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi_suoming(player, actor, stage)".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi_suoming stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpshanxiSuoming {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_rechuhai" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rechuhai(player, target, card, target_card, stage)".into(),
                });
            }
            let stage = expect_u32(&args[4], span.start as usize)?;
            if stage > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rechuhai stage must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveRechuhai {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                target_card: check_expr(&args[3], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_spjianyi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spjianyi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpjianyi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_heji::check_effect_stmt(name, args, span, locals),
    }
}

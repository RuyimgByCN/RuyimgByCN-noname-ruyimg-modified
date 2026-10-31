//! 效果语句类型检查链：具名技能结算器（resolve_*，无包前缀者）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_chenglve" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chenglve(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveChenglve {
                player: check_expr(&args[0], locals)?,
                stage: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_shicai" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shicai(player, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShicai {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_sxrmqiewang" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sxrmqiewang(player, event_player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSxrmqiewang {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_sxrmqianliu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sxrmqianliu(player, source, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSxrmqianliu {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_xiongluan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongluan(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiongluan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_duoduan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duoduan(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDuoduan {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_qiangyong" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiangyong(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveQiangyong {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_yixiang" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yixiang(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYixiang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zhongjie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongjie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhongjie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_starcuxia_weiming" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_starcuxia_weiming(player, source, variant)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStarcuxiaWeiming {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                variant: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_sanshou" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sanshou(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSanshou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_nhlundao" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_nhlundao(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveNhlundao {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xiangshu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiangshu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiangshu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dingce" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dingce(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDingce {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_meiyong" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_meiyong(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMeiyong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_friendbiwei" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_friendbiwei(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFriendbiwei {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_chulao" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chulao(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveChulao {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_gaoyuan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gaoyuan(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGaoyuan {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_yuli" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yuli(player, mode)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yuli mode must be 0 (source) or 1 (target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYuli {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                span: *span,
            })
        }
        "resolve_tingwei" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tingwei(player, mode)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tingwei mode must be 0 (select) or 1 (resolve)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTingwei {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                span: *span,
            })
        }
        "resolve_xunyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xunyi(player, mode)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xunyi mode must be 0 (select) or 1 (damage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXunyi {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                span: *span,
            })
        }
        "resolve_boming" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_boming(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveBoming {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_ejian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ejian(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveEjian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_yiyong" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yiyong(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYiyong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_shanxie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shanxie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShanxie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_sheyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sheyi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSheyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_tianyin" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tianyin(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTianyin {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dbzhuifeng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dbzhuifeng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDbzhuifeng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dbquedi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dbquedi(player, target, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDbquedi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_qingyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qingyu(player, action, xuancun_def, mbyongjue_def)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qingyu action must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveQingyu {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                xuancun_def: expect_u32(&args[2], span.start as usize)?,
                mbyongjue_def: expect_u32(&args[3], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_zhengsu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengsu(player, mode, stage, card)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengsu mode must be 0 through 2".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengsu stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhengsu {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                stage: stage as u8,
                card: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_xiongshu" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongshu(player, target, card, action, stage)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongshu action must be 0 or 1".into(),
                });
            }
            let stage = expect_u32(&args[4], span.start as usize)?;
            if stage > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongshu stage must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiongshu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_jianhui" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jianhui(player, source, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJianhui {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_tongduo" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tongduo(player, target, card, stage)".into(),
                });
            }
            let stage = expect_u32(&args[3], span.start as usize)?;
            if stage > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tongduo stage must be 0 through 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTongduo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_yuanqing" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yuanqing(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYuanqing {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_shuchen" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shuchen(player, dying, renku_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShuchen {
                player: check_expr(&args[0], locals)?,
                dying: check_expr(&args[1], locals)?,
                renku_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_duanbi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duanbi(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if stage > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duanbi stage must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveDuanbi {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_zaoli" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zaoli(player, mark)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZaoli {
                player: check_expr(&args[0], locals)?,
                mark: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_guying" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guying(player, actor, card, stage)".into(),
                });
            }
            let stage = expect_u32(&args[3], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guying stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveGuying {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_houfeng" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_houfeng(player, target, mode, stage)".into(),
                });
            }
            let mode = expect_u32(&args[2], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_houfeng mode must be 0 or 1".into(),
                });
            }
            let stage = expect_u32(&args[3], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_houfeng stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveHoufeng {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                mode: mode as u8,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_fyjianyu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_fyjianyu(player, a, b)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFyjianyu {
                player: check_expr(&args[0], locals)?,
                a: check_expr(&args[1], locals)?,
                b: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_liubing" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_liubing(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLiubing {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::set_draw_phase_bottom_resolvers_packs::check_effect_stmt(
            name, args, span, locals,
        ),
    }
}

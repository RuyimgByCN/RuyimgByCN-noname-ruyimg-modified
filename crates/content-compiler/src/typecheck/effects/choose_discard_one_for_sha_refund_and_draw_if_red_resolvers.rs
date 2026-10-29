//! 效果语句类型检查链：具名技能结算器（resolve_*）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_youdi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_youdi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYoudi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_kuangfu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_kuangfu(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveKuangfu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_shanjia" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shanjia(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShanjia {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_new_yajiao" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_new_yajiao(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_new_yajiao second argument must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectResolveNewYajiao {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        "resolve_dingpin" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dingpin(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDingpin {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xinfu_guolun" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinfu_guolun(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinfuGuolun {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_lieren" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_lieren(player, target, win_skill_def, lose_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLieren {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                win_skill_def: expect_u32(&args[2], span.start as usize)?,
                lose_skill_def: expect_u32(&args[3], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_tuntian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tuntian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTuntian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zaoxian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zaoxian(player, jixi_skill_def, tuntian_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZaoxian {
                player: check_expr(&args[0], locals)?,
                jixi_def: expect_u32(&args[1], span.start as usize)?,
                tuntian_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_tiaoxin" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tiaoxin(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTiaoxin {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zhiji" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhiji(player, guanxing_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhiji {
                player: check_expr(&args[0], locals)?,
                guanxing_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_zaiqi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zaiqi(player, extra)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZaiqi {
                player: check_expr(&args[0], locals)?,
                extra: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_twqiushou" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twqiushou(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwqiushou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zhengrong" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengrong(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhengrong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_hongju" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_hongju(player, qingjian_skill_def, rong_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveHongju {
                player: check_expr(&args[0], locals)?,
                qingjian_def: expect_u32(&args[1], span.start as usize)?,
                rong_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_qingjian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qingjian(player, rong_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveQingjian {
                player: check_expr(&args[0], locals)?,
                rong_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_releiji" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_releiji(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveReleiji {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_huangtian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_huangtian(player, actor)".into(),
                });
            }
            Ok(StmtIr::EffectResolveHuangtian {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_haoshi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_haoshi(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveHaoshi {
                player: check_expr(&args[0], locals)?,
                stage: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_dimeng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dimeng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDimeng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_qiaobian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiaobian(player, phase)".into(),
                });
            }
            let phase = expect_u32(&args[1], span.start as usize)?;
            if phase > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiaobian phase must be 0..=3".into(),
                });
            }
            Ok(StmtIr::EffectResolveQiaobian {
                player: check_expr(&args[0], locals)?,
                phase: phase as u8,
                span: *span,
            })
        }
        "resolve_xiangle" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiangle(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiangle {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_fangquan_skip" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_fangquan_skip(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFangquanSkip {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_fangquan_grant" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_fangquan_grant(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFangquanGrant {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_ruoyu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ruoyu(player, jijiang_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRuoyu {
                player: check_expr(&args[0], locals)?,
                jijiang_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_zuilun" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zuilun(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZuilun {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_fuyin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_fuyin(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFuyin {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_juzhan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_juzhan(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJuzhan {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_qizhi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qizhi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveQizhi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_jinqu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jinqu(player, qizhi_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJinqu {
                player: check_expr(&args[0], locals)?,
                qizhi_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_jianxiang" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jianxiang(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJianxiang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_shenshi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shenshi(player, source, stage)".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if !matches!(stage, 0 | 1 | 4) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shenshi stage must be 0, 1, or 4".into(),
                });
            }
            Ok(StmtIr::EffectResolveShenshi {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_mingren" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mingren(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if !matches!(stage, 0 | 2) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mingren stage must be 0 or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveMingren {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_zhenliang" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhenliang(player, source, card, mingren_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhenliang {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                mingren_def: expect_u32(&args[3], span.start as usize)?,
                stage: 0,
                span: *span,
            })
        }
        "resolve_kuizhu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_kuizhu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveKuizhu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_zhizheng::check_effect_stmt(name, args, span, locals),
    }
}

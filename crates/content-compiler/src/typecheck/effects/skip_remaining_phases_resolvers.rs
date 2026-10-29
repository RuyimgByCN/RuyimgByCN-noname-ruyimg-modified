//! 效果语句类型检查链：具名技能结算器（resolve_*）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_jsrgchaozheng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgchaozheng(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgchaozheng must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveJsrgchaozheng {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_jsrgsheju" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgsheju(player, counterpart, self_skill)".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgsheju must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveJsrgsheju {
                player: check_expr(&args[0], locals)?,
                counterpart: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_qianlong" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qianlong(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qianlong must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveQianlong {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_jsrgzuozhan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgzuozhan(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgzuozhan must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveJsrgzuozhan {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_twmouzhi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twmouzhi(player, self_skill, mode, event_card)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twmouzhi must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwmouzhi {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                mode: check_expr(&args[2], locals)?,
                event_card: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_twdaigui" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twdaigui(player, self_skill, stage, count)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twdaigui must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwdaigui {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                count: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_twmitu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twmitu(player, self_skill, stage, count)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twmitu must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwmitu {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                count: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_qiantun" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiantun(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiantun must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveQiantun {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_yangming" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yangming(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yangming must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveYangming {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_canyun" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_canyun(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_canyun must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveCanyun {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_hengwu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_hengwu(player, self_skill, event_card)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_hengwu must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveHengwu {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                event_card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_biaozhao3" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_biaozhao3(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_biaozhao3 must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveBiaozhao3 {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_biaozhao2" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_biaozhao2(player, self_skill, event_player)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_biaozhao2 must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveBiaozhao2 {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                event_player: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_dcniji" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcniji(player, self_skill, mode, event_card)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcniji must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcniji {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                mode: check_expr(&args[2], locals)?,
                event_card: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_dcmoukui" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcmoukui(player, self_skill, stage, target)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcmoukui must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcmoukui {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                target: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_zhongyan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongyan(player, self_skill, stage)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongyan must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhongyan {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_chumei" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chumei(player, self_skill, stage)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chumei must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveChumei {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_shuaiyan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shuaiyan(player, self_skill, stage)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shuaiyan must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveShuaiyan {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_mbbeiming" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbbeiming(player, self_skill, stage)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbbeiming must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbbeiming {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_zhengjian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengjian(player, self_skill, stage, event_player)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengjian must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhengjian {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                event_player: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_qinmian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qinmian(player, self_skill, mode, amount)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qinmian must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveQinmian {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                mode: check_expr(&args[2], locals)?,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_chuanyun" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chuanyun(player, self_skill, event_player, event_card)"
                        .into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chuanyun must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveChuanyun {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                event_player: check_expr(&args[2], locals)?,
                event_card: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_jsrgzuozhan_gain" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgzuozhan_gain(player, self_skill, event_player)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgzuozhan_gain must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveJsrgzuozhanGain {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                event_player: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_jsrgdaoren" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgdaoren(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgdaoren must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveJsrgdaoren {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_duwang" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duwang(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duwang must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveDuwang {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_quanjia" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_quanjia(player, self_skill, source, target)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_quanjia must reference self_skill".into(),
                });
            }
            let source = check_expr(&args[2], locals)?;
            let target = check_expr(&args[3], locals)?;
            Ok(StmtIr::EffectResolveQuanjia {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                source,
                target,
                span: *span,
            })
        }
        "resolve_pxqixian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pxqixian(player, self_skill, stage, kind)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pxqixian must reference self_skill".into(),
                });
            }
            let stage = check_expr(&args[2], locals)?;
            let kind = check_expr(&args[3], locals)?;
            Ok(StmtIr::EffectResolvePxqixian {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage,
                kind,
                span: *span,
            })
        }
        "resolve_dcbingji" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcbingji(player, self_skill, stage, kind)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcbingji must reference self_skill".into(),
                });
            }
            let stage = check_expr(&args[2], locals)?;
            let kind = check_expr(&args[3], locals)?;
            Ok(StmtIr::EffectResolveDcbingji {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                stage,
                kind,
                span: *span,
            })
        }
        _ => super::choose_discard_heart_redirect_current_damage::check_effect_stmt(
            name, args, span, locals,
        ),
    }
}

//! 效果语句类型检查链：具名技能结算器（resolve_*）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_ddd_guanning" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_guanning(player, actor, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_guanning public action must be 0..=2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddGuanning {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_caoshuang" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_caoshuang(player, actor, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_caoshuang public action must be 0..=3".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddCaoshuang {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_xuelingyun" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xuelingyun(player, actor, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xuelingyun public action must be 0..=5".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXuelingyun {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_liuhong" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuhong(player, actor, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuhong public action must be 0..=5".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddLiuhong {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_xiahouxuan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xiahouxuan(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xiahouxuan public action must be 0..=2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXiahouxuan {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_zhangkai" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_zhangkai(player, actor, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_zhangkai public action must be 0..=4".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddZhangkai {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_liangxi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liangxi(player, actor, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liangxi public action must be 0..=2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddLiangxi {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_shoulijian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shoulijian(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShoulijian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spfengyin" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spfengyin(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpfengyin {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xinfu_lianpian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinfu_lianpian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinfuLianpian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_gongjian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gongjian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGongjian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xinfu_qinguo" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinfu_qinguo(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinfuQinguo {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcmanzhi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcmanzhi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcmanzhi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xinxingluan" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinxingluan(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinxingluan {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dckuizhen" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dckuizhen(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDckuizhen {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dckuizhen_resolve" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dckuizhen_resolve(player, winner, loser)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDckuizhenResolve {
                player: check_expr(&args[0], locals)?,
                winner: check_expr(&args[1], locals)?,
                loser: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_nsdingzhou" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_nsdingzhou(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveNsdingzhou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_sxrmlanjiao" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sxrmlanjiao(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSxrmlanjiao {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_rekuanshi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rekuanshi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRekuanshi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_rekuanshi_resolve" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rekuanshi_resolve(player, damaged)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRekuanshiResolve {
                player: check_expr(&args[0], locals)?,
                damaged: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dcmoyu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcmoyu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcmoyu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xizhen" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xizhen(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXizhen {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xuxie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xuxie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXuxie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xuxie_add" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xuxie_add(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXuxieAdd {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcchiying" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcchiying(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcchiying {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xinkuangfu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinkuangfu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinkuangfu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcxiangshu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcxiangshu(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcxiangshu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_shuangxiong" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shuangxiong(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShuangxiong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zhijian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhijian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhijian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_guzheng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guzheng(player, event_player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGuzheng {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_zhiba" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhiba(player, actor)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhiba {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_drlt_congjian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_congjian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDrltCongjian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_nzry_feijun" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_nzry_feijun(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveNzryFeijun {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_drlt_yongsi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_yongsi(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_yongsi stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDrltYongsi {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_drlt_weidi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_weidi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDrltWeidi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcxiangshu_lottery" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcxiangshu_lottery(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcxiangshuLottery {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dcyingtu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcyingtu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcyingtu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbzhuguo" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbzhuguo(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbzhuguo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbanxian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbanxian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbanxian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbjinzu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbjinzu(player, target, helper_skill_def)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            if helper_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "mbjinzu helper_skill_def must be positive".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbjinzu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def,
                span: *span,
            })
        }
        "resolve_ol_jie_danshou" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ol_jie_danshou(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlJieDanshou {
                player: check_expr(&args[0], locals)?,
                skill: ExprIr::SelfSkill,
                span: *span,
            })
        }
        "resolve_retieji" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_retieji(player, card, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRetieji {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_new_reluoyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_new_reluoyi(player, buff_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveNewReluoyi {
                player: check_expr(&args[0], locals)?,
                buff_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_remoshi" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_remoshi(player, target, card, helper_skill_def, action)"
                        .into(),
                });
            }
            let action = expect_u32(&args[4], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_remoshi action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveRemoshi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                helper_skill_def: expect_u32(&args[3], span.start as usize)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_xinyaoming" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinyaoming(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinyaoming {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_rehuaiyi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rehuaiyi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRehuaiyi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_reanguo" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_reanguo(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveReanguo {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_beizhu::check_effect_stmt(name, args, span, locals),
    }
}

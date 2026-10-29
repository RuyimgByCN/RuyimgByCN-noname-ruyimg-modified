//! 效果语句类型检查链：具名技能结算器（resolve_*）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_heji" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_heji(player, target, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveHeji {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_twjiaohua" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjiaohua(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwjiaohua {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_twkujian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twkujian(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twkujian action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwkujian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twruilian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twruilian(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twruilian action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwruilian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twjianwei" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjianwei(player, actor, stage)".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjianwei stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwjianwei {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_twtanfeng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twtanfeng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwtanfeng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_twzhibian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twzhibian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwzhibian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_twyuyan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twyuyan(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwyuyan {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_twguoyi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twguoyi(player, target, card, hand_lock_skill_def)".into(),
                });
            }
            let hand_lock_skill_def = expect_u32(&args[3], span.start as usize)?;
            Ok(StmtIr::EffectResolveTwguoyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                hand_lock_skill_def,
                span: *span,
            })
        }
        "resolve_twchuhai" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchuhai(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchuhai action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwchuhai {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twyizhu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twyizhu(player, source, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twyizhu action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwyizhu {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twluanchou" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twluanchou(player, helper_skill_def, action)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[1], span.start as usize)?;
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twluanchou action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwluanchou {
                player: check_expr(&args[0], locals)?,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twxiawei" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twxiawei(player, wangxing_skill_def, action)".into(),
                });
            }
            let wangxing_skill_def = expect_u32(&args[1], span.start as usize)?;
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twxiawei action must be 0 through 4".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwxiawei {
                player: check_expr(&args[0], locals)?,
                wangxing_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twbianfuren" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twbianfuren(player, target, helper_skill_def, action)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twbianfuren action must be 0 through 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwbianfuren {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twfeiyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfeiyi(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfeiyi action must be 0 through 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwfeiyi {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twchenzhen" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchenzhen(player, target, helper_skill_def, record_skill_def, action)".into(),
                    });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let record_skill_def = expect_u32(&args[3], span.start as usize)?;
            let action = expect_u32(&args[4], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchenzhen action must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwchenzhen {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def,
                record_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twqianju" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twqianju(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwqianju {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_twqingxi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twqingxi(player, target, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwqingxi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_twzaoli" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twzaoli(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwzaoli {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_twzhengjian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twzhengjian(player, target, helper_skill_def, action)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twzhengjian action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwzhengjian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twtianyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twtianyu(player, target, helper_skill_def, action)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twtianyu action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwtianyu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twwujing" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twwujing(player, target_count, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twwujing action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwwujing {
                player: check_expr(&args[0], locals)?,
                target_count: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twshanghe" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twshanghe(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwshanghe {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_twwangchang" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twwangchang(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twwangchang action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwwangchang {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twcaozhao" => {
            if args.len() != 7 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twcaozhao(player, target, source, card, twfuzuan_def, twfeifu_def, action)".into(),
                    });
            }
            let action = expect_u32(&args[6], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twcaozhao action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwcaozhao {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                card: check_expr(&args[3], locals)?,
                twfuzuan_skill_def: expect_u32(&args[4], span.start as usize)?,
                twfeifu_skill_def: expect_u32(&args[5], span.start as usize)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twguohuai" => {
            if args.len() != 7 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twguohuai(player, target, source, card, mark_def, helper_def, action)".into(),
                    });
            }
            let action = expect_u32(&args[6], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twguohuai action must be 0..=4".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwguohuai {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                card: check_expr(&args[3], locals)?,
                mark_def: expect_u32(&args[4], span.start as usize)?,
                helper_skill_def: expect_u32(&args[5], span.start as usize)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twchengpu" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchengpu(player, target, source, card, chunlao_def, action)"
                        .into(),
                });
            }
            let action = expect_u32(&args[5], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchengpu action must be 0..=3".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwchengpu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                card: check_expr(&args[3], locals)?,
                chunlao_skill_def: expect_u32(&args[4], span.start as usize)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twcaoang" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twcaoang(player, target, source, card, helper_def, action)"
                        .into(),
                });
            }
            let action = expect_u32(&args[5], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twcaoang action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwcaoang {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                card: check_expr(&args[3], locals)?,
                helper_skill_def: expect_u32(&args[4], span.start as usize)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twniujin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twniujin(player, limited_skill_def, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twniujin action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwniujin {
                player: check_expr(&args[0], locals)?,
                limited_skill_def: expect_u32(&args[1], span.start as usize)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twmoukui" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twmoukui(player, target, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwmoukui {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_twfuman" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfuman(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfuman action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwfuman {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_cuijin" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_cuijin(player, source, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_cuijin action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveCuijin {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twqingkou" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twqingkou(player, winner, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twqingkou action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwqingkou {
                player: check_expr(&args[0], locals)?,
                winner: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twchenxun" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchenxun(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twchenxun action must be 0 through 4".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwchenxun {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_twfenxian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfenxian(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfenxian action must be 0 through 4".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwfenxian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_qiluan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiluan(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiluan must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveQiluan {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_ollinjie" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ollinjie(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOllinjie {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_turn_end_debates" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_turn_end_debates(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_turn_end_debates must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveTurnEndDebates {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_potheyun" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potheyun(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potheyun must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotheyun {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_twjieqiu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjieqiu(player, target, subaction, self_skill)".into(),
                });
            }
            let subaction = expect_u32(&args[2], span.start as usize)?;
            if subaction != 0 && subaction != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjieqiu subaction must be 0 or 2".into(),
                });
            }
            if !matches!(&args[3], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjieqiu must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwjieqiu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                subaction,
                skill: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        _ => super::skip_remaining_phases::check_effect_stmt(name, args, span, locals),
    }
}

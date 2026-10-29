//! 效果语句类型检查链：具名技能结算器（resolve_*，DDD 与玩点论杀之外）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_jieliang_gain" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jieliang_gain(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJieliangGain {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_lulve" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_lulve(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLulve {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dctingxian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dctingxian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDctingxian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcnuchen" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcnuchen(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcnuchen {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcdanji" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcdanji(player, skill)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcdanji {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dczhiheng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dczhiheng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDczhiheng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_stdgaobian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_stdgaobian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStdgaobian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_spjiedao" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spjiedao(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpjiedao {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_sibian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sibian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSibian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_refuhai" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_refuhai(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRefuhai {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_gzjinfa" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gzjinfa(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGzjinfa {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_wn_qifeng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wn_qifeng(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveWnQifeng {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dcjue" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcjue(player, event_player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcjue {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_ruanji" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ruanji(player, source, card, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRuanji {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_zhengbi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengbi(player, target, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhengbi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_fengying" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_fengying(player, helper_def, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFengying {
                player: check_expr(&args[0], locals)?,
                helper_def: check_expr(&args[1], locals)?,
                action: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_qiaojian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiaojian(player, target, helper_def, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveQiaojian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_def: check_expr(&args[2], locals)?,
                action: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_xicha" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xicha(player, source, card, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXicha {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_huzun" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_huzun(player, source, card, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveHuzun {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_zhugemengxue" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhugemengxue(player, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhugemengxue {
                player: check_expr(&args[0], locals)?,
                action: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_bailingyun" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_bailingyun(player, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveBailingyun {
                player: check_expr(&args[0], locals)?,
                action: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dcshaowei" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcshaowei(player, action)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcshaowei {
                player: check_expr(&args[0], locals)?,
                action: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dcdichou" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcdichou(player, action, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcdichou {
                player: check_expr(&args[0], locals)?,
                action: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_dcdufeng" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcdufeng(player, action, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcdufeng {
                player: check_expr(&args[0], locals)?,
                action: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_dcctjiuxian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcctjiuxian(player, event_player, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 2) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcctjiuxian action must be 0 or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcctjiuxian {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_bianyue" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_bianyue(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 10) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_bianyue action must be 0 or 10".into(),
                });
            }
            Ok(StmtIr::EffectResolveBianyue {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_zhupeilan" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhupeilan(player, target, card, helper_def, action)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[3], span.start as usize)?;
            let action = expect_u32(&args[4], span.start as usize)?;
            if !matches!(action, 0 | 1 | 2 | 10) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhupeilan action must be 0, 1, 2, or 10".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhupeilan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_dc_sb_zhangxiu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_zhangxiu(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 10) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_zhangxiu action must be 0 or 10".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcSbZhangxiu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_dc_sb_zhouyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_zhouyu(player, source, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_zhouyu card must be a card".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 1 | 10 | 11 | 20) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_zhouyu action must be 0, 1, 10, 11, or 20".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcSbZhouyu {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_dc_sb_lusu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_lusu(player, source, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_lusu card must be a card".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 10 | 11 | 20 | 21) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dc_sb_lusu action must be 0, 10, 11, 20, or 21".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcSbLusu {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_zhangjian" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhangjian(player, card, grant_skill_def, action)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhangjian card must be a card".into(),
                });
            }
            let grant_skill_def = expect_u32(&args[2], span.start as usize)?;
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 1) || (action == 1 && grant_skill_def == 0) {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhangjian action must be 0, or 1 with a non-zero granted skill definition".into(),
                    });
            }
            Ok(StmtIr::EffectResolveZhangjian {
                player: check_expr(&args[0], locals)?,
                card,
                grant_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_zhugeruoxue" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhugeruoxue(player, target, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhugeruoxue card must be a card".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 10) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhugeruoxue action must be 0 or 10".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhugeruoxue {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_caoyi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_caoyi(player, card, helper_skill_def, action)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_caoyi card must be a card".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 10 | 11 | 20 | 21) || helper_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "resolve_caoyi requires a non-zero helper and action 0, 10, 11, 20, or 21"
                            .into(),
                });
            }
            Ok(StmtIr::EffectResolveCaoyi {
                player: check_expr(&args[0], locals)?,
                card,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_malingli" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_malingli(player, card, helper_skill_def, action)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_malingli card must be a card".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 10 | 20) || helper_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_malingli requires a non-zero helper and action 0, 10, or 20"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolveMalingli {
                player: check_expr(&args[0], locals)?,
                card,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_nsshuaiyan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_nsshuaiyan(player, recovered_player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveNsshuaiyan {
                player: check_expr(&args[0], locals)?,
                recovered_player: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dddxianxi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dddxianxi(player, card, action)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dddxianxi card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dddxianxi action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXianxi {
                player: check_expr(&args[0], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_dddxiaheng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dddxiaheng(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action != 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dddxiaheng public action must be 0".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXiaheng {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        _ => super::choose_discard_heart_redirect_current_damage_resolvers_ddd_wdb::check_effect_stmt(name, args, span, locals),
    }
}

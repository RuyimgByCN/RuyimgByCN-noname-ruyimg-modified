//! 效果语句类型检查链：DDD 与玩点论杀（Wdb）具名技能结算器。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_wdb_hanlong" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_hanlong(player, target, card, helper_skill_def, action)"
                        .into(),
                });
            }
            let target = check_expr(&args[1], locals)?;
            let card = check_expr(&args[2], locals)?;
            if !matches!(
                target,
                ExprIr::EventPlayer | ExprIr::U32(_) | ExprIr::Int(_)
            ) || !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_))
            {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_hanlong target/card operands have invalid types".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[3], span.start as usize)?;
            let action = expect_u32(&args[4], span.start as usize)?;
            if helper_skill_def == 0 || action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_hanlong requires a non-zero helper and action 0 or 1"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbHanlong {
                player: check_expr(&args[0], locals)?,
                target,
                card,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_liufu" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action)"
                        .into(),
                });
            }
            let target = check_expr(&args[1], locals)?;
            let card = check_expr(&args[2], locals)?;
            if !matches!(
                target,
                ExprIr::EventPlayer | ExprIr::EventSource | ExprIr::U32(_) | ExprIr::Int(_)
            ) || !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_))
            {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liufu target/card operands have invalid types".into(),
                });
            }
            let zhucheng_skill_def = expect_u32(&args[3], span.start as usize)?;
            let action = expect_u32(&args[4], span.start as usize)?;
            if zhucheng_skill_def == 0 || action > 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_wdb_liufu requires a non-zero 筑城 definition and action 0, 1, or 2"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectResolveWdbLiufu {
                player: check_expr(&args[0], locals)?,
                target,
                card,
                zhucheng_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_liuyan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liuyan(player, card, action)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liuyan card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liuyan action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbLiuyan {
                player: check_expr(&args[0], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_liuzan" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liuzan(player, target, helper_skill_def, action)".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            if helper_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liuzan requires a helper skill definition".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action != 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_liuzan public action must be 0".into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbLiuzan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_tianyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_tianyu(player, target, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_tianyu card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action != 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_tianyu public action must be 0".into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbTianyu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_xizhenxihong" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_xizhenxihong(player, target, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_xizhenxihong card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_xizhenxihong public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbXizhenxihong {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_yangyi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_yangyi(player, target, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_yangyi card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_yangyi public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbYangyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_wdb_zaozhirenjun" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_zaozhirenjun(player, target, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_zaozhirenjun card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wdb_zaozhirenjun public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveWdbZaozhiRenjun {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_liuchong" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuchong(player, actor, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuchong card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuchong public action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddLiuchong {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_duanbing" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_duanbing(player, target, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_duanbing card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_duanbing public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddDuanbing {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_wuzhi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_wuzhi(player, card, action)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_wuzhi card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 10) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_wuzhi public action must be 0 or 10".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddWuzhi {
                player: check_expr(&args[0], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_xujing" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xujing(player, source, card, action)".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xujing card operand has invalid type".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if !matches!(action, 0 | 1 | 2 | 10 | 11 | 12) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xujing public action must be 0, 1, 2, 10, 11, or 12"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXujing {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_caomao" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_caomao(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if !matches!(action, 0 | 10) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_caomao public action must be 0 or 10".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddCaomao {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_xinxianying" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xinxianying(player, actor, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 1) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xinxianying public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXinxianying {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_xianglang" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xianglang(player, actor, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 1 | 2) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_xianglang public action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddXianglang {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_yujin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_yujin(player, actor, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if !matches!(action, 0 | 1) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_yujin public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddYujin {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_liuye" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuye(player, actor, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuye public action must be 0..=4".into(),
                });
            }
            let card = check_expr(&args[2], locals)?;
            if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuye card operand has invalid type".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddLiuye {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_baosanniang" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_baosanniang(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_baosanniang public action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddBaosanniang {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_zhenji" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_zhenji(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_zhenji public action must be 0..=2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddZhenji {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_zhaoang" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_zhaoang(player, action)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_zhaoang public action must be 0..=1".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddZhaoang {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_liuba" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuba(player, actor, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_liuba public action must be 0..=4".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddLiuba {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_ddd_jianshuo" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_jianshuo(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ddd_jianshuo public action must be 0..=2".into(),
                });
            }
            Ok(StmtIr::EffectResolveDddJianshuo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        _ => super::resolve_ddd_guanning::check_effect_stmt(name, args, span, locals),
    }
}

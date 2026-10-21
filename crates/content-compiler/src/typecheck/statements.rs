use super::*;

pub(super) fn card_kind_value(name: &str) -> Option<i32> {
    Some(match name {
        "sha" => 1,
        "tao" => 2,
        "shan" => 3,
        "jiu" => 4,
        "sha_fire" => 5,
        "sha_thunder" => 6,
        "weapon" => 10,
        "armor" => 11,
        "horse_off" => 12,
        "horse_def" => 13,
        "treasure" => 14,
        "wuzhong" => 20,
        "guohe" => 21,
        "shunshou" => 22,
        "juedou" => 23,
        "nanman" => 24,
        "wanjian" => 25,
        "taoyuan" => 26,
        "wugu" => 27,
        "le" => 30,
        "bingliang" => 31,
        "shandian" => 32,
        "jiedao" => 33,
        "tiesuo" => 34,
        "huogong" => 35,
        "wuxie" => 36,
        "yuanjiao" => 37,
        "tiaojiyanmei" => 38,
        "mantianguohai" => 39,
        "youdishenru" => 40,
        "diaobingqianjiang" => 41,
        "wangmeizhike" => 42,
        _ => return None,
    })
}

pub(super) fn lifetime_card_reward_group_value(name: &str) -> Option<u8> {
    Some(match name {
        "sha_or_shan" => rule_api::lifetime_card_reward_group::SHA_OR_SHAN,
        "tao_or_jiu" => rule_api::lifetime_card_reward_group::TAO_OR_JIU,
        "juedou_or_wuzhong" => rule_api::lifetime_card_reward_group::JUE_DOU_OR_WU_ZHONG,
        _ => return None,
    })
}

pub(super) fn check_stmts(
    stmts: &[StmtAst],
    locals: &mut Vec<String>,
) -> Result<Vec<StmtIr>, CompileError> {
    let mut out = Vec::new();
    for s in stmts {
        out.push(check_stmt(s, locals)?);
    }
    Ok(out)
}

pub(super) fn check_reveal_discard_suit_gain_or_use(
    args: &[ExprAst],
    span: content_schema::Span,
    locals: &[String],
) -> Result<StmtIr, CompileError> {
    if args.len() != 4 {
        return Err(CompileError::Type {
            pos: span.start as usize,
            message: "reveal_discard_suit_gain_or_use(player, mark_def, base_count, max_mark)"
                .into(),
        });
    }
    let bounded_u8 = |arg: &ExprAst, name: &str| {
        u8::try_from(expect_int(arg, span.start as usize)?).map_err(|_| CompileError::Type {
            pos: span.start as usize,
            message: format!("{name} must be 0..255"),
        })
    };
    Ok(StmtIr::EffectRevealDiscardSuitGainOrUse {
        player: check_expr(&args[0], locals)?,
        mark: expect_u32(&args[1], span.start as usize)?,
        base_count: bounded_u8(&args[2], "base_count")?,
        max_mark: bounded_u8(&args[3], "max_mark")?,
        span,
    })
}

pub(super) fn check_choose_discard_or_fire_damage_skip_phases(
    args: &[ExprAst],
    span: content_schema::Span,
    locals: &[String],
) -> Result<StmtIr, CompileError> {
    if args.len() != 3 {
        return Err(CompileError::Type {
            pos: span.start as usize,
            message: "choose_discard_or_fire_damage_skip_phases(player, max_discard, damage)"
                .into(),
        });
    }
    let positive_u8 = |arg: &ExprAst, name: &str| {
        let value = expect_int(arg, span.start as usize)?;
        u8::try_from(value)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("{name} must be 1..255"),
            })
    };
    Ok(StmtIr::EffectChooseDiscardOrFireDamageSkipPhases {
        player: check_expr(&args[0], locals)?,
        max_discard: positive_u8(&args[1], "max_discard")?,
        damage: positive_u8(&args[2], "damage")?,
        span,
    })
}

pub(super) fn check_stmt(s: &StmtAst, locals: &mut Vec<String>) -> Result<StmtIr, CompileError> {
    match s {
        StmtAst::Effect { name, args, span } => {
            check_effect_stmt(name.as_str(), args, span, locals)
        }

        StmtAst::If {
            cond,
            then_body,
            else_body,
            span,
        } => {
            let cond = check_expr(cond, locals)?;
            let then_body = check_stmts(then_body, locals)?;
            let else_body = check_stmts(else_body, locals)?;
            Ok(StmtIr::If {
                cond,
                then_body,
                else_body,
                span: *span,
            })
        }
        StmtAst::Loop { times, body, span } => {
            if *times == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "loop count must be > 0".into(),
                });
            }
            Ok(StmtIr::Loop {
                times: *times,
                body: check_stmts(body, locals)?,
                span: *span,
            })
        }
        StmtAst::Let { name, value, span } => {
            let value = check_expr(value, locals)?;
            if !locals.iter().any(|l| l == name) {
                locals.push(name.clone());
            }
            Ok(StmtIr::Let {
                name: name.clone(),
                value,
                span: *span,
            })
        }
    }
}

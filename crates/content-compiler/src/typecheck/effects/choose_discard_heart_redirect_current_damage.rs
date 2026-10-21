use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "choose_discard_heart_redirect_current_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_heart_redirect_current_damage(player, draw_skill_def)"
                        .into(),
                });
            }
            let draw_skill_def = expect_u32(&args[1], span.start as usize)?;
            if draw_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw skill def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveTianxiang {
                player: check_expr(&args[0], locals)?,
                draw_skill_def,
                span: *span,
            })
        }
        "choose_discard_heart_cancel_damage_branch" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_heart_cancel_damage_branch(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRetianxiang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_link_other_same_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_link_other_same_hp(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLianzhou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_give_hand_equip_to_event_player" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_hand_equip_to_event_player(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStddechong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
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
        "choose_other_gain_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_gain_event_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStdjujian {
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
        "choose_other_add_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_add_skill(player, skill_def)".into(),
                });
            }
            let grant_def = expect_u32(&args[1], span.start as usize)?;
            if grant_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "granted skill def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveHfjieying {
                player: check_expr(&args[0], locals)?,
                grant_def,
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
        "reveal_choose_category_distribute_self_threshold_hp_loss" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)".into(),
                    });
            }
            let reveal_count = bounded_u8(&args[1], "reveal_count")?;
            let recipient_cap = bounded_u8(&args[2], "recipient_cap")?;
            if reveal_count == 0 || recipient_cap == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_count and recipient_cap must be positive".into(),
                });
            }
            Ok(
                StmtIr::EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
                    player: check_expr(&args[0], locals)?,
                    reveal_count,
                    recipient_cap,
                    span: *span,
                },
            )
        }
        "choose_other_random_gain_region_then_draw_compare_color_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
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
        "choose_gain_target_hand_ceil_half_difference" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_target_hand_ceil_half_difference(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainTargetHandCeilHalfDifference {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
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
        "choose_discard_hand_range" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_hand_range(player, min, max)".into(),
                });
            }
            let min = expect_u32(&args[1], span.start as usize)?;
            let max = expect_u32(&args[2], span.start as usize)?;
            if min == 0 || max < min {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard range requires 0 < min <= max".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardHandRange {
                player: check_expr(&args[0], locals)?,
                min,
                max,
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

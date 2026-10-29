//! 效果语句类型检查链：具名技能结算器（resolve_*）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_dingpan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dingpan(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDingpan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_tanlu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tanlu(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTanlu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_hidden_skill_pile" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_hidden_skill_pile(player, pile_skill_def, action, card)"
                        .into(),
                });
            }
            let pile_skill_def = expect_u32(&args[1], span.start as usize)?;
            let action = u8::try_from(expect_u32(&args[2], span.start as usize)?)
                .ok()
                .filter(|action| *action <= 4)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "hidden skill-pile action must be 0..4".into(),
                })?;
            if pile_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "hidden skill-pile definition must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveHiddenSkillPile {
                player: check_expr(&args[0], locals)?,
                pile_skill_def,
                action,
                card: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_dili_yuheng" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dili_yuheng(player, action, card)".into(),
                });
            }
            let action = match &args[1] {
                ExprAst::Int(value) if (0..=u8::MAX as i32).contains(value) => *value as u8,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "Dili/Yuheng action must be a u8 literal".into(),
                    });
                }
            };
            let card = check_expr(&args[2], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "Dili/Yuheng card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectResolveDiliYuheng {
                player: check_expr(&args[0], locals)?,
                action,
                card,
                span: *span,
            })
        }
        "resolve_guanchao_rank" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guanchao_rank(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "Guanchao card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectResolveGuanchaoRank {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        "resolve_guanwei" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guanwei(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGuanwei {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_lanjiang" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_lanjiang(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLanjiang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_rebingyi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rebingyi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRebingyi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_rebotu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rebotu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRebotu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_oldimeng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldimeng(player, 0|1)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "oldimeng action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldimeng {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_rejieyin" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rejieyin(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRejieyin {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_reanxu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_reanxu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveReanxu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_drlt_jueyan" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_jueyan(player, sha_skill_def, weapon_skill_def, mount_skill_def, armor_skill_def, treasure_skill_def)".into(),
                    });
            }
            Ok(StmtIr::EffectResolveDrltJueyan {
                player: check_expr(&args[0], locals)?,
                sha_skill_def: expect_u32(&args[1], span.start as usize)?,
                weapon_skill_def: expect_u32(&args[2], span.start as usize)?,
                mount_skill_def: expect_u32(&args[3], span.start as usize)?,
                armor_skill_def: expect_u32(&args[4], span.start as usize)?,
                treasure_skill_def: expect_u32(&args[5], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_rejizhi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rejizhi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveRejizhi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_jiexun" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jiexun(player, funan_helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJiexun {
                player: check_expr(&args[0], locals)?,
                funan_helper_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_wengua_setup" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wengua_setup(player, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveWenguaSetup {
                player: check_expr(&args[0], locals)?,
                helper_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_wengua" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_wengua(player, main_skill_def, marker_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveWengua {
                player: check_expr(&args[0], locals)?,
                main_skill_def: expect_u32(&args[1], span.start as usize)?,
                marker_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_xinanguo" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinanguo(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinanguo {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_ganglie_three" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ganglie_three(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGanglieThree {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zhongyi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongyi(player, action, helper_skill_def)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongyi action must be 0, 1, or 2".into(),
                });
            }
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            if helper_skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongyi helper skill must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhongyi {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                helper_skill_def,
                span: *span,
            })
        }
        "resolve_oldanxu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldanxu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldAnxu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_oldqinqing" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldqinqing(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldQinqing {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_oldmingjian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldmingjian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldMingjian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_oldxuanfeng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldxuanfeng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldXuanfeng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_jianrou" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jianrou(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJianrou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_oldhuisheng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldhuisheng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldHuisheng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_oldjiefan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldjiefan(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if stage > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_oldjiefan stage must be 0..=2".into(),
                });
            }
            Ok(StmtIr::EffectResolveOldJiefan {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_chegu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chegu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveChegu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_junkguixin" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_junkguixin(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJunkguixin {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_yao_chenwei" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yao_chenwei(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYaoChenwei {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mass_damage_discard_turn_over" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mass_damage_discard_turn_over(player, mark_def, mark_count, damage_amount, hand_count)".into(),
                    });
            }
            let mark_def = expect_u32(&args[1], span.start as usize)?;
            let mark_count = expect_u32(&args[2], span.start as usize)?;
            let damage_amount = expect_u32(&args[3], span.start as usize)?;
            let hand_count = expect_u32(&args[4], span.start as usize)?;
            if mark_def == 0 || mark_count == 0 || damage_amount == 0 || hand_count == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "mass damage/discard arguments must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveMassDamageDiscardTurnOver {
                player: check_expr(&args[0], locals)?,
                mark_def,
                mark_count,
                damage_amount,
                hand_count,
                span: *span,
            })
        }
        "resolve_mass_damage_discard_hand_equip" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mass_damage_discard_hand_equip(player, mark_def, mark_count, damage_amount, discard_count)".into(),
                    });
            }
            let mark_def = expect_u32(&args[1], span.start as usize)?;
            let mark_count = expect_u32(&args[2], span.start as usize)?;
            let damage_amount = expect_u32(&args[3], span.start as usize)?;
            let discard_count = expect_u32(&args[4], span.start as usize)?;
            if mark_def == 0 || mark_count == 0 || damage_amount == 0 || discard_count == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "mass damage/discard arguments must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveMassDamageDiscardHandEquip {
                player: check_expr(&args[0], locals)?,
                mark_def,
                mark_count,
                damage_amount,
                discard_count,
                span: *span,
            })
        }
        "resolve_mark_parity_control" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_mark_parity_control(player, mark_def, mass_threshold, damage_amount)"
                                .into(),
                    });
            }
            let mark_def = expect_u32(&args[1], span.start as usize)?;
            let mass_threshold = expect_u32(&args[2], span.start as usize)?;
            let damage_amount = expect_u32(&args[3], span.start as usize)?;
            if mark_def == 0 || mass_threshold == 0 || damage_amount == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "mark parity control arguments must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectResolveMarkParityControl {
                player: check_expr(&args[0], locals)?,
                mark_def,
                mass_threshold,
                damage_amount,
                span: *span,
            })
        }
        "resolve_selected_equipment_discard_then_choose_damage" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_selected_equipment_discard_then_choose_damage(player, damage_amount, normal|fire|thunder)"
                                .into(),
                    });
            }
            let damage_amount = expect_u32(&args[1], span.start as usize)?;
            if damage_amount == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "selected-target damage amount must be non-zero".into(),
                });
            }
            let nature = match &args[2] {
                ExprAst::Ident(name) if name == "normal" => 0,
                ExprAst::Ident(name) if name == "fire" => 1,
                ExprAst::Ident(name) if name == "thunder" => 2,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage nature must be normal, fire, or thunder".into(),
                    });
                }
            };
            Ok(
                StmtIr::EffectResolveSelectedEquipmentDiscardThenChooseDamage {
                    player: check_expr(&args[0], locals)?,
                    damage_amount,
                    nature,
                    span: *span,
                },
            )
        }
        "resolve_yeyan" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yeyan(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYeyan {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_qixing" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qixing(player, initialize)".into(),
                });
            }
            let initialize = match &args[1] {
                ExprAst::Int(0) => false,
                ExprAst::Int(1) => true,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qixing initialize must be 0 or 1".into(),
                    });
                }
            };
            Ok(StmtIr::EffectResolveQixing {
                player: check_expr(&args[0], locals)?,
                initialize,
                span: *span,
            })
        }
        "resolve_qixing_weather" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qixing_weather(player, qixing_skill_def, helper_skill_def, max_targets)".into(),
                    });
            }
            let qixing_skill_def = expect_u32(&args[1], span.start as usize)?;
            let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
            let max_targets = u8::try_from(expect_u32(&args[3], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "Qixing weather max_targets must be 1..255".into(),
                })?;
            Ok(StmtIr::EffectResolveQixingWeather {
                player: check_expr(&args[0], locals)?,
                qixing_skill_def,
                helper_skill_def,
                max_targets,
                span: *span,
            })
        }
        "resolve_nzry_longnu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_nzry_longnu(player, fire_skill_def, thunder_skill_def)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolveNzryLongnu {
                player: check_expr(&args[0], locals)?,
                fire_skill_def: expect_u32(&args[1], span.start as usize)?,
                thunder_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_drlt_poxi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_poxi(player, target, hand_limit_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDrltPoxi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                hand_limit_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_drlt_jieying" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_jieying(player, event_player, mark_def, mode)".into(),
                });
            }
            let mode = u8::try_from(expect_u32(&args[3], span.start as usize)?)
                .ok()
                .filter(|mode| *mode <= 2)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "Jieying mode must be 0, 1, or 2".into(),
                })?;
            Ok(StmtIr::EffectResolveDrltJieying {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                mark: expect_u32(&args[2], span.start as usize)?,
                mode,
                span: *span,
            })
        }
        "resolve_drlt_duorui" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_drlt_duorui(player, target, helper_skill_def, stage)".into(),
                });
            }
            let stage = u8::try_from(expect_u32(&args[3], span.start as usize)?)
                .ok()
                .filter(|stage| matches!(stage, 0 | 3))
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "Duorui package stage must be 0 or 3".into(),
                })?;
            Ok(StmtIr::EffectResolveDrltDuorui {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                stage,
                span: *span,
            })
        }
        "resolve_drlt_zhiti" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "resolve_drlt_zhiti(player, event_player, event_source, event_amount, mode)"
                            .into(),
                });
            }
            let mode = u8::try_from(expect_u32(&args[4], span.start as usize)?)
                .ok()
                .filter(|mode| *mode <= 2)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "Zhiti mode must be 0, 1, or 2".into(),
                })?;
            Ok(StmtIr::EffectResolveDrltZhiti {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                event_source: check_expr(&args[2], locals)?,
                event_amount: check_expr(&args[3], locals)?,
                mode,
                span: *span,
            })
        }
        "resolve_shen_simayi" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shen_simayi(player, event_player, event_amount, mark, mode)"
                        .into(),
                });
            }
            let mode = u8::try_from(expect_u32(&args[4], span.start as usize)?)
                .ok()
                .filter(|mode| *mode <= 4)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "Shen Simayi mode must be 0..4".into(),
                })?;
            Ok(StmtIr::EffectResolveShenSimayi {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                event_amount: check_expr(&args[2], locals)?,
                mark: expect_u32(&args[3], span.start as usize)?,
                mode,
                span: *span,
            })
        }
        "resolve_beige" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_beige(player, target, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveBeige {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_beige_configured" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_beige_configured(player, target, source, heart_recover_amount, diamond_draw_count)".into(),
                    });
            }
            Ok(StmtIr::EffectResolveBeigeConfigured {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                heart_recover_amount: check_expr(&args[3], locals)?,
                diamond_draw_count: check_expr(&args[4], locals)?,
                span: *span,
            })
        }
        "resolve_zhuiyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhuiyi(player, killer)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhuiyi {
                player: check_expr(&args[0], locals)?,
                killer: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_decadezhenjun" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_decadezhenjun(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDecadezhenjun {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_sp_xiaoguo" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sp_xiaoguo(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpXiaoguo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::choose_discard_one_for_sha_refund_and_draw_if_red::check_effect_stmt(
            name, args, span, locals,
        ),
    }
}

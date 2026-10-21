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
        "guozhan_use_zhulian_bihe" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "guozhan_use_zhulian_bihe(player)".into(),
                });
            }
            Ok(StmtIr::EffectGuozhanUseZhulianBihe {
                player: check_expr(&args[0], locals)?,
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
        "cancel_incoming_card_use" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "cancel_incoming_card_use(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectCancelIncomingCardUse {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_other_gain_one_of_two_skills" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_gain_one_of_two_skills(player, target, first_def, second_def)"
                            .into(),
                });
            }
            let first_def = expect_u32(&args[2], span.start as usize)?;
            let second_def = expect_u32(&args[3], span.start as usize)?;
            if first_def == 0 || second_def == 0 || first_def == second_def {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_gain_one_of_two_skills expects two distinct non-zero skill definitions".into(),
                    });
            }
            Ok(StmtIr::EffectChooseOtherGainOneOfTwoSkills {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                first_def,
                second_def,
                span: *span,
            })
        }
        "gwyewu_random_discard_chain" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gwyewu_random_discard_chain(player, wuyao_def, lang_def)".into(),
                });
            }
            let wuyao_def = expect_u32(&args[1], span.start as usize)?;
            let lang_def = expect_u32(&args[2], span.start as usize)?;
            if wuyao_def == 0 || lang_def == 0 || wuyao_def == lang_def {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gwyewu_random_discard_chain expects two distinct non-zero skill definitions".into(),
                    });
            }
            Ok(StmtIr::EffectGwyewuRandomDiscardChain {
                player: check_expr(&args[0], locals)?,
                wuyao_def,
                lang_def,
                span: *span,
            })
        }
        "give_selected_then_target_may_use_draw_owner" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_selected_then_target_may_use_draw_owner(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveSelectedThenTargetMayUseDrawOwner {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "skip_next_discard_phase" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_next_discard_phase(player)".into(),
                });
            }
            Ok(StmtIr::EffectSkipNextDiscardPhase {
                player: check_expr(&args[0], locals)?,
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
        "discard_hand_equip_count" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_hand_equip_count(player, count)".into(),
                });
            }
            let count = u32::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "discard count must be a positive integer".into(),
                })?;
            Ok(StmtIr::EffectDiscardHandEquipCount {
                player: check_expr(&args[0], locals)?,
                count,
                span: *span,
            })
        }
        "give_selected_then_target_choose_discard_same_suit_or_lose_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)"
                                .into(),
                    });
            }
            Ok(
                StmtIr::EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "gain_equipment_from_draw_pile" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_equipment_from_draw_pile(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainEquipmentFromDrawPile {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "target_choose_suit_gain_owner_hand_damage_if_mismatch" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "target_choose_suit_gain_owner_hand_damage_if_mismatch(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "judge_source_discard_two_hand_or_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_source_discard_two_hand_or_damage(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeSourceDiscardTwoHandOrDamage {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_selected_recover_after_phase_total_two" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_selected_recover_after_phase_total_two(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveSelectedRecoverAfterPhaseTotalTwo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_other_hand_targets_gain_one_replace_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_hand_targets_gain_one_replace_draw(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherHandTargetsGainOneReplaceDraw {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_hand_targets_gain_one_reduce_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_hand_targets_gain_one_reduce_draw(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherHandTargetsGainOneReduceDraw {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "distribute_top_two_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "distribute_top_two_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectDistributeTopTwoCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "give_all_hand_recover_to_one" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_all_hand_recover_to_one(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveAllHandRecoverToOne {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(player)"
                        .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "choose_give_one_hand_equip_or_recover" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_one_hand_equip_or_recover(owner, giver, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseGiveOneHandEquipOrRecover {
                owner: check_expr(&args[0], locals)?,
                giver: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_give_two_different_colors_or_recover" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_two_different_colors_or_recover(owner, giver, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseGiveTwoDifferentColorsOrRecover {
                owner: check_expr(&args[0], locals)?,
                giver: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
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
        "apply_mbjinzu_next_sha" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "apply_mbjinzu_next_sha(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectApplyMbjinzuNextSha {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "lose_hp_then_damage_other_lose_max_hp_if_dying" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, self_skill)"
                                .into(),
                    });
            }
            if !matches!(&args[3], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "HP-loss-linked damage must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectLoseHpThenDamageOtherLoseMaxHpIfDying {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                skill: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "emit_note" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "emit_note(code)".into(),
                });
            }
            Ok(StmtIr::EffectNote {
                code: expect_int(&args[0], span.start as usize)?,
                span: *span,
            })
        }
        "change_max_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "change_max_hp(player, delta)".into(),
                });
            }
            Ok(StmtIr::EffectChangeMaxHp {
                player: check_expr(&args[0], locals)?,
                delta: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "add_mark" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_mark(player, mark_def, delta)".into(),
                });
            }
            let player = check_expr(&args[0], locals)?;
            let mark = expect_u32(&args[1], span.start as usize)?;
            match expect_int(&args[2], span.start as usize) {
                Ok(delta) => Ok(StmtIr::EffectAddMark {
                    player,
                    mark,
                    delta,
                    span: *span,
                }),
                Err(_) => Ok(StmtIr::EffectAddMarkExpr {
                    player,
                    mark,
                    delta: check_expr(&args[2], locals)?,
                    span: *span,
                }),
            }
        }
        "remove_mark" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_mark(player, mark_def, amount)".into(),
                });
            }
            let player = check_expr(&args[0], locals)?;
            let mark = expect_u32(&args[1], span.start as usize)?;
            match expect_int(&args[2], span.start as usize) {
                Ok(amount) => Ok(StmtIr::EffectRemoveMark {
                    player,
                    mark,
                    amount,
                    span: *span,
                }),
                Err(_) => Ok(StmtIr::EffectRemoveMarkExpr {
                    player,
                    mark,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                }),
            }
        }
        "discard_hand" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_hand(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardHand {
                player: check_expr(&args[0], locals)?,
                count: expect_int(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "discard_hand_expr" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_hand_expr(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardHandExpr {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_hand_equip_expr" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_hand_equip_expr(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardHandEquipExpr {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_hand_equip" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_hand_equip(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardHandEquip {
                player: check_expr(&args[0], locals)?,
                count: expect_int(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "adjust_hand_to" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "adjust_hand_to(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectAdjustHandTo {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_selected" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_selected(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardSelected {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "recast_selected" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recast_selected(player)".into(),
                });
            }
            Ok(StmtIr::EffectRecastSelected {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "disable_all_equip_slots" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "disable_all_equip_slots(player)".into(),
                });
            }
            Ok(StmtIr::EffectDisableAllEquipSlots {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "move_equipment_then_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "move_equipment_then_draw(player)".into(),
                });
            }
            Ok(StmtIr::EffectMoveEquipmentThenDraw {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_use_virtual_sha_or_draw" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_use_virtual_sha_or_draw(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseUseVirtualShaOrDraw {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "discard_equipment_pair_with_armor" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_equipment_pair_with_armor(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardEquipmentPairWithArmor {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "discard_target_equipment_gain_mount" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_target_equipment_gain_mount(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardTargetEquipmentGainMount {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_target_equipment_draw_weapon_range" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_target_equipment_draw_weapon_range(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardTargetEquipmentDrawWeaponRange {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_reveal_hand_damage_if_sha" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_reveal_hand_damage_if_sha(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetRevealHandDamageIfSha {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_other_give_one_hand_equip" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_give_one_hand_equip(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherGiveOneHandEquip {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_reveal_hand_card_bind_category_target" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_reveal_hand_card_bind_category_target(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseRevealHandCardBindCategoryTarget {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "recover_to_max_then_optional_exchange_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "recover_to_max_then_optional_exchange_skill(player, removable_skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectRecoverToMaxThenOptionalExchangeSkill {
                player: check_expr(&args[0], locals)?,
                removable_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "reveal_hand_discard_kind" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand_discard_kind(player, card_kind)".into(),
                });
            }
            let ExprAst::Ident(kind) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand_discard_kind expects a card-kind name".into(),
                });
            };
            let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown card kind {kind}"),
            })?;
            Ok(StmtIr::EffectRevealHandDiscardKind {
                player: check_expr(&args[0], locals)?,
                kind,
                span: *span,
            })
        }
        "discard_target_card_gain_mount" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_target_card_gain_mount(player, target, equip|hand_equip|hand_equip_judge|equip_judge)".into(),
                    });
            }
            let ExprAst::Ident(scope) = &args[2] else {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_target_card_gain_mount scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                    });
            };
            let scope = match scope.as_str() {
                    "equip" => TargetCardScopeIr::Equip,
                    "hand_equip" => TargetCardScopeIr::HandEquip,
                    "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                    "equip_judge" => TargetCardScopeIr::EquipJudge,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "discard_target_card_gain_mount scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                        })
                    }
                };
            Ok(StmtIr::EffectDiscardTargetCardGainMount {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                scope,
                span: *span,
            })
        }
        "make_current_sha_target_unrespondable" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "make_current_sha_target_unrespondable(source, card, target)".into(),
                });
            }
            Ok(StmtIr::EffectMakeCurrentShaTargetUnrespondable {
                source: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "judge_make_current_sha_target_unrespondable_on_red" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "judge_make_current_sha_target_unrespondable_on_red(player, card, target)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectJudgeMakeCurrentShaTargetUnrespondableOnRed {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
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
        "choose_target_discard_distance_hand_or_increase_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_target_discard_distance_hand_or_increase_damage(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseTargetDiscardDistanceHandOrIncreaseDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_draw_view_two_gain_or_move_equipment" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_draw_view_two_gain_or_move_equipment(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDrawViewTwoGainOrMoveEquipment {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_beizhu::check_effect_stmt(name, args, span, locals),
    }
}

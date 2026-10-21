use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "set_draw_phase_bottom" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "set_draw_phase_bottom(player)".into(),
                });
            }
            Ok(StmtIr::EffectSetDrawPhaseFromBottom {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_cards_from_bottom" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_cards_from_bottom(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawFromBottom {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_chenglve" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chenglve(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveChenglve {
                player: check_expr(&args[0], locals)?,
                stage: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_shicai" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shicai(player, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShicai {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_sbbiyue" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sbbiyue(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSbbiyue {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_sxrmqiewang" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sxrmqiewang(player, event_player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSxrmqiewang {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_sxrmqianliu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sxrmqianliu(player, source, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSxrmqianliu {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_gzbuqu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gzbuqu(player, delta, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGzbuqu {
                player: check_expr(&args[0], locals)?,
                delta: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_gzbuqu_recover" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gzbuqu_recover(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGzbuquRecover {
                player: check_expr(&args[0], locals)?,
                stage: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xiongluan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongluan(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiongluan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_twjuxiang" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twjuxiang(player, event_player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwjuxiang {
                player: check_expr(&args[0], locals)?,
                event_player: check_expr(&args[1], locals)?,
                stage: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "draw_cards" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_cards(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "recover_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recover_hp(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectRecover {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "recover_hp_as" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recover_hp_as(source, player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectRecoverAs {
                source: check_expr(&args[0], locals)?,
                player: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_duoduan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duoduan(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDuoduan {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_qiangyong" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qiangyong(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveQiangyong {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_twxiongjun" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twxiongjun(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwxiongjun {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcqiaolve" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcqiaolve(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcqiaolve {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_yixiang" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yixiang(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYixiang {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_xinfu_qiai" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinfu_qiai(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinfuQiai {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_olwenren" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olwenren(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlwenren {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_spolaige" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spolaige(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpolaige {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_olzhijie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olzhijie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlzhijie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_olsuyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olsuyi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlsuyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_olqiaozhi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_olqiaozhi(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveOlqiaozhi {
                player: check_expr(&args[0], locals)?,
                stage: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dczhee" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dczhee(player, mode)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDczhee {
                player: check_expr(&args[0], locals)?,
                mode: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_dchaoxian" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dchaoxian(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDchaoxian {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_zhongjie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhongjie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhongjie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_starcuxia_weiming" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_starcuxia_weiming(player, source, variant)".into(),
                });
            }
            Ok(StmtIr::EffectResolveStarcuxiaWeiming {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                variant: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_sanshou" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sanshou(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSanshou {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_nhlundao" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_nhlundao(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveNhlundao {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xiangshu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiangshu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiangshu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dingce" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dingce(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDingce {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_meiyong" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_meiyong(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMeiyong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbzhenann" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbzhenann(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbzhenann {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_friendbiwei" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_friendbiwei(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFriendbiwei {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_chulao" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_chulao(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveChulao {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dcanxu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcanxu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcanxu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_gaoyuan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_gaoyuan(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveGaoyuan {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_redirect_current_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_redirect_current_sha(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardRedirectCurrentSha {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "judge_set_skill_mark_to_rank" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_set_skill_mark_to_rank(player, skill)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeSetSkillMarkToRank {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_discard_one_from_each_linked" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_one_from_each_linked(player, skill)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardOneFromEachLinked {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "disable_skill_by_def_until_turn_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "disable_skill_by_def_until_turn_end(player, skill_def)".into(),
                });
            }
            let skill_def = match &args[1] {
                ExprAst::Int(value) if *value >= 0 => *value as u32,
                ExprAst::U32(value) => *value,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skill_def must be a nonnegative stable id".into(),
                    });
                }
            };
            Ok(StmtIr::EffectDisableSkillByDefUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                skill_def,
                span: *span,
            })
        }
        "target_discard_then_duel_if_non_sha_and_hp_gte_owner" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_target_lose_hp_discard_hand_sha_if_basic" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_lose_hp_discard_hand_sha_if_basic(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetLoseHpDiscardHandShaIfBasic {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reset_linked_selected_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reset_linked_selected_targets(source)".into(),
                });
            }
            Ok(StmtIr::EffectResetLinkedSelectedTargets {
                source: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "deal_damage" => {
            if !(3..=4).contains(&args.len()) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "deal_damage(source, target, amount[, normal|fire|thunder])".into(),
                });
            }
            let nature = match args.get(3) {
                None => 0,
                Some(ExprAst::Ident(name)) if name == "normal" => 0,
                Some(ExprAst::Ident(name)) if name == "fire" => 1,
                Some(ExprAst::Ident(name)) if name == "thunder" => 2,
                Some(_) => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage nature must be normal, fire, or thunder".into(),
                    });
                }
            };
            Ok(StmtIr::EffectDamage {
                source: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                nature,
                span: *span,
            })
        }
        "deal_damage_ban_active_if_dying" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "deal_damage_ban_active_if_dying(source, target, amount, self_skill)"
                        .into(),
                });
            }
            if !matches!(&args[3], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "damage-linked active ban must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectDealDamageBanActiveIfDying {
                source: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                skill: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "spjungong_discard_or_lose_then_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spjungong_discard_or_lose_then_sha(player, target, self_skill)"
                        .into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "峻攻 must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectSpjungongDiscardOrLoseThenSha {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "ban_active_until_turn_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "ban_active_until_turn_end(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "turn-end active ban must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectBanActiveUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "spwanwei_active" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spwanwei_active(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectSpwanweiActive {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "spwanwei_dying" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spwanwei_dying(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectSpwanweiDying {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spxizhan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spxizhan(player, phase_player, fangzong_skill_def)".into(),
                });
            }
            let fangzong = expect_u32(&args[2], span.start as usize)?;
            if fangzong == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spxizhan fangzong_skill_def must be nonzero".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpxizhan {
                player: check_expr(&args[0], locals)?,
                phase_player: check_expr(&args[1], locals)?,
                fangzong_skill_def: fangzong,
                span: *span,
            })
        }
        "resolve_spsongshu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "resolve_spsongshu(player, phase_player, renku_skill_def, block_skill_def)"
                            .into(),
                });
            }
            let renku = expect_u32(&args[2], span.start as usize)?;
            let block = expect_u32(&args[3], span.start as usize)?;
            if renku == 0 || block == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spsongshu skill defs must be nonzero".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpsongshu {
                player: check_expr(&args[0], locals)?,
                phase_player: check_expr(&args[1], locals)?,
                renku_skill_def: renku,
                block_skill_def: block,
                span: *span,
            })
        }
        "resolve_spcunsi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spcunsi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpcunsi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_yuli" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yuli(player, mode)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yuli mode must be 0 (source) or 1 (target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYuli {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                span: *span,
            })
        }
        "resolve_tingwei" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tingwei(player, mode)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tingwei mode must be 0 (select) or 1 (resolve)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTingwei {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                span: *span,
            })
        }
        "resolve_xinmingshi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinmingshi(player, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinmingshi {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xunyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xunyi(player, mode)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xunyi mode must be 0 (select) or 1 (damage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXunyi {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                span: *span,
            })
        }
        "resolve_boming" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_boming(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveBoming {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_ejian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_ejian(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveEjian {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_yiyong" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yiyong(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYiyong {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_shanxie" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shanxie(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShanxie {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_sheyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_sheyi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSheyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_tianyin" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tianyin(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTianyin {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dbzhuifeng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dbzhuifeng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDbzhuifeng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_dbquedi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dbquedi(player, target, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDbquedi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_spyinju" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spyinju(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpyinju {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spchijie" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spchijie(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpchijie {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_spshangyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshangyi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpshangyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spyajun" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spyajun(player, less_def, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpyajun {
                player: check_expr(&args[0], locals)?,
                less_def: expect_u32(&args[1], span.start as usize)?,
                stage: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_spzundi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spzundi(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpzundi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xinyangjie" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xinyangjie(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveXinyangjie {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_qingyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qingyu(player, action, xuancun_def, mbyongjue_def)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_qingyu action must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveQingyu {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                xuancun_def: expect_u32(&args[2], span.start as usize)?,
                mbyongjue_def: expect_u32(&args[3], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_zhengsu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengsu(player, mode, stage, card)".into(),
                });
            }
            let mode = expect_u32(&args[1], span.start as usize)?;
            if mode > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengsu mode must be 0 through 2".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhengsu stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhengsu {
                player: check_expr(&args[0], locals)?,
                mode: mode as u8,
                stage: stage as u8,
                card: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_mbdiancai" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbdiancai(player, actor)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbdiancai {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_xiongshu" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongshu(player, target, card, action, stage)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongshu action must be 0 or 1".into(),
                });
            }
            let stage = expect_u32(&args[4], span.start as usize)?;
            if stage > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_xiongshu stage must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveXiongshu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_jianhui" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jianhui(player, source, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJianhui {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_spqiai" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spqiai(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpqiai {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_spshanxi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi(player, target, stage)".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpshanxi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_tongduo" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tongduo(player, target, card, stage)".into(),
                });
            }
            let stage = expect_u32(&args[3], span.start as usize)?;
            if stage > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tongduo stage must be 0 through 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveTongduo {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_yuanqing" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_yuanqing(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveYuanqing {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_shuchen" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_shuchen(player, dying, renku_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveShuchen {
                player: check_expr(&args[0], locals)?,
                dying: check_expr(&args[1], locals)?,
                renku_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_duanbi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duanbi(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if stage > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_duanbi stage must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveDuanbi {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_zaoli" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zaoli(player, mark)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZaoli {
                player: check_expr(&args[0], locals)?,
                mark: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_guying" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guying(player, actor, card, stage)".into(),
                });
            }
            let stage = expect_u32(&args[3], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_guying stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveGuying {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_spshanxi_suoming" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi_suoming(player, actor, stage)".into(),
                });
            }
            let stage = expect_u32(&args[2], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spshanxi_suoming stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpshanxiSuoming {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_rechuhai" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rechuhai(player, target, card, target_card, stage)".into(),
                });
            }
            let stage = expect_u32(&args[4], span.start as usize)?;
            if stage > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rechuhai stage must be 0 through 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveRechuhai {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                target_card: check_expr(&args[3], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_houfeng" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_houfeng(player, target, mode, stage)".into(),
                });
            }
            let mode = expect_u32(&args[2], span.start as usize)?;
            if mode > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_houfeng mode must be 0 or 1".into(),
                });
            }
            let stage = expect_u32(&args[3], span.start as usize)?;
            if stage > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_houfeng stage must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveHoufeng {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                mode: mode as u8,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_spjianyi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spjianyi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpjianyi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_fyjianyu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_fyjianyu(player, a, b)".into(),
                });
            }
            Ok(StmtIr::EffectResolveFyjianyu {
                player: check_expr(&args[0], locals)?,
                a: check_expr(&args[1], locals)?,
                b: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_liubing" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_liubing(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLiubing {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_heji::check_effect_stmt(name, args, span, locals),
    }
}

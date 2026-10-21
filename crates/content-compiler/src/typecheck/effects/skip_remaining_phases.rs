use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "skip_remaining_phases" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_remaining_phases(player, phases_mask, self_skill)".into(),
                });
            }
            let phases_mask = expect_u32(&args[1], span.start as usize)?;
            if phases_mask == 0 || phases_mask > 0b0001_1100 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_remaining_phases mask limited to Draw/Play/Discard".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_remaining_phases must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectSkipRemainingPhases {
                player: check_expr(&args[0], locals)?,
                phases_mask,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_others_add_mark" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_others_add_mark(player, mark_def, delta, max_count, self_skill)"
                            .into(),
                });
            }
            let mark = expect_u32(&args[1], span.start as usize)?;
            let delta = expect_int(&args[2], span.start as usize)?;
            let max_count = expect_u32(&args[3], span.start as usize)?;
            if !matches!(&args[4], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_others_add_mark must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseOthersAddMark {
                player: check_expr(&args[0], locals)?,
                mark,
                delta,
                max_count,
                skill: check_expr(&args[4], locals)?,
                span: *span,
            })
        }
        "register_turn_end_debate" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "register_turn_end_debate(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "register_turn_end_debate must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectRegisterTurnEndDebate {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
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
        "choose_other_add_mark" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_add_mark(player, mark_def, delta)".into(),
                });
            }
            let player = check_expr(&args[0], locals)?;
            let mark = expect_u32(&args[1], span.start as usize)?;
            let delta =
                expect_int(&args[2], span.start as usize).map_err(|e| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("choose_other_add_mark delta must be an integer literal: {e}"),
                })?;
            Ok(StmtIr::EffectChooseOtherAddMark {
                player,
                mark,
                delta,
                span: *span,
            })
        }
        "gain_whole_skill_pile" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_whole_skill_pile(player, self_skill, pile_def)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_whole_skill_pile must reference self_skill".into(),
                });
            }
            let pile_def = expect_u32(&args[2], 0)?;
            Ok(StmtIr::EffectGainWholeSkillPile {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                pile_def,
                span: *span,
            })
        }
        "place_hand_cards_to_deck_top" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_hand_cards_to_deck_top(player, self_skill, count)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_hand_cards_to_deck_top must reference self_skill".into(),
                });
            }
            let count = check_expr(&args[2], locals)?;
            Ok(StmtIr::EffectPlaceHandCardsToDeckTop {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                count,
                span: *span,
            })
        }
        "rearrange_pile_to_deck_top" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "rearrange_pile_to_deck_top(player, self_skill, pile_def, count)"
                        .into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "rearrange_pile_to_deck_top must reference self_skill".into(),
                });
            }
            let pile_def = expect_u32(&args[2], 0)?;
            let count = check_expr(&args[3], locals)?;
            Ok(StmtIr::EffectRearrangePileToDeckTop {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                pile_def,
                count,
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
        "choose_hand_cards_to_skill_pile" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_hand_cards_to_skill_pile(player, self_skill, min_count, max_count)"
                            .into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_hand_cards_to_skill_pile must reference self_skill".into(),
                });
            }
            let min_count = check_expr(&args[2], locals)?;
            let max_count = check_expr(&args[3], locals)?;
            Ok(StmtIr::EffectChooseHandCardsToSkillPile {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                min_count,
                max_count,
                span: *span,
            })
        }
        "choose_target_hand_card_to_pile" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_hand_card_to_pile(player, self_skill, pile_def)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_hand_card_to_pile must reference self_skill".into(),
                });
            }
            let pile_def = expect_u32(&args[2], 0)?;
            Ok(StmtIr::EffectChooseTargetHandCardToPile {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                pile_def,
                span: *span,
            })
        }
        "choose_one_from_skill_pile_discard" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_one_from_skill_pile_discard(player, self_skill, pile_def)"
                        .into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_one_from_skill_pile_discard must reference self_skill".into(),
                });
            }
            let pile_def = expect_u32(&args[2], 0)?;
            Ok(StmtIr::EffectChooseOneFromSkillPileDiscard {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                pile_def,
                span: *span,
            })
        }
        "place_top_deck_to_expansion" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_top_deck_to_expansion(player, self_skill, pile_def, count)"
                        .into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_top_deck_to_expansion must reference self_skill".into(),
                });
            }
            let pile_def = expect_u32(&args[2], 0)?;
            let count = check_expr(&args[3], locals)?;
            Ok(StmtIr::EffectPlaceTopDeckToExpansion {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                pile_def,
                count,
                span: *span,
            })
        }
        "discard_skill_pile" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_skill_pile(player, self_skill, pile_def)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_skill_pile must reference self_skill".into(),
                });
            }
            let pile_def = expect_u32(&args[2], 0)?;
            Ok(StmtIr::EffectDiscardSkillPile {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                pile_def,
                span: *span,
            })
        }
        "gain_from_skill_pile" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_from_skill_pile(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_from_skill_pile must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectGainFromSkillPile {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "place_hand_card_to_expansion" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_hand_card_to_expansion(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_hand_card_to_expansion must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectPlaceHandCardToExpansion {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "extra_resolve_current_trick" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "extra_resolve_current_trick(player, card, self_skill)".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "extra_resolve_current_trick must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectExtraResolveCurrentTrick {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_clan_not_max_hand_limit_gain" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_clan_not_max_hand_limit_gain(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_clan_not_max_hand_limit_gain must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseClanNotMaxHandLimitGain {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "gain_target_hand_card" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_target_hand_card(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_target_hand_card must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectGainTargetHandCard {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "increase_current_sha_base_damage" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "increase_current_sha_base_damage(player, card, amount)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "increase-current-Sha card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectIncreaseCurrentShaBaseDamage {
                player: check_expr(&args[0], locals)?,
                card,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "skip_next_play_phase" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_next_play_phase(player)".into(),
                });
            }
            Ok(StmtIr::EffectSkipNextPlayPhase {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "skip_next_judge_phase" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_next_judge_phase(player)".into(),
                });
            }
            Ok(StmtIr::EffectSkipNextJudgePhase {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "skip_judge_and_draw_phases" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "skip_judge_and_draw_phases(player)".into(),
                });
            }
            Ok(StmtIr::EffectSkipJudgeAndDrawPhases {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_damage_endpoint_card_increase_if_suit" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_damage_endpoint_card_increase_if_suit(player, target, spade|heart|club|diamond, amount)".into(),
                    });
            }
            let suit = match &args[2] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "damage-endpoint discard suit must be spade, heart, club, or diamond"
                                .into(),
                    });
                }
            };
            Ok(
                StmtIr::EffectChooseDiscardDamageEndpointCardIncreaseIfSuit {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    suit,
                    amount: check_expr(&args[3], locals)?,
                    span: *span,
                },
            )
        }
        "choose_other_draw_or_recover" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_draw_or_recover(player, draw_count, recover_count, times)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDrawOrRecover {
                player: check_expr(&args[0], locals)?,
                draw_count: check_expr(&args[1], locals)?,
                recover_count: check_expr(&args[2], locals)?,
                times: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_other_recover" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_recover(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherRecover {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "enable_equip_slot" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "enable_equip_slot(player, slot, source)".into(),
                });
            }
            Ok(StmtIr::EffectEnableEquipSlot {
                player: check_expr(&args[0], locals)?,
                slot: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_hand_equip_then_remove_self_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_hand_equip_then_remove_self_skill(player, count)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardHandEquipThenRemoveSelfSkill {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "draw_selected_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_selected_targets(count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawSelectedTargets {
                count: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "recover_selected_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recover_selected_targets(amount)".into(),
                });
            }
            Ok(StmtIr::EffectRecoverSelectedTargets {
                amount: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "record_selected_play_phase_category" => {
            if args.len() != 1 || !matches!(&args[0], ExprAst::Ident(name) if name == "self_skill")
            {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "record_selected_play_phase_category(self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectRecordSelectedPlayPhaseCategory {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "recover_if_min_current_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "recover_if_min_current_hp(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectRecoverIfMinCurrentHp {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "increase_current_card_base_damage" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "increase_current_card_base_damage(player, card, amount)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "increase-current-card card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectIncreaseCurrentCardBaseDamage {
                player: check_expr(&args[0], locals)?,
                card,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "modify_current_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "modify_current_damage(player, delta)".into(),
                });
            }
            Ok(StmtIr::EffectModifyCurrentDamage {
                player: check_expr(&args[0], locals)?,
                delta: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_draw_or_increase_current_card_damage_or_both_mark" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_increase_current_card_damage_or_both_mark(player, card, draw_count, damage_amount, mark_def, mark_count)"
                                .into(),
                    });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "current-card branch card must be a card operand".into(),
                });
            }
            Ok(
                StmtIr::EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                    player: check_expr(&args[0], locals)?,
                    card,
                    draw_count: check_expr(&args[2], locals)?,
                    damage_amount: check_expr(&args[3], locals)?,
                    mark_def: expect_u32(&args[4], span.start as usize)?,
                    mark_count: check_expr(&args[5], locals)?,
                    span: *span,
                },
            )
        }
        "draw_then_choose_give_or_discard_hand" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "draw_then_choose_give_or_discard_hand(player, draw_count, payment_count)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectDrawThenChooseGiveOrDiscardHand {
                player: check_expr(&args[0], locals)?,
                draw_count: check_expr(&args[1], locals)?,
                payment_count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_give_or_discard_hand" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_or_discard_hand(player, payment_count, hand_empty)"
                        .into(),
                });
            }
            let scope = expect_draw_target_scope(&args[2], span.start as usize)?;
            if scope != DrawTargetScopeIr::HandEmpty {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_or_discard_hand currently requires hand_empty scope"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseGiveOrDiscardHand {
                player: check_expr(&args[0], locals)?,
                payment_count: check_expr(&args[1], locals)?,
                scope,
                span: *span,
            })
        }
        "choose_discard_hand_equip_then_recover_target" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_equip_then_recover_target(chooser, target, count, amount)".into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardHandEquipThenRecoverTarget {
                chooser: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "gain_target_equipment" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "gain_target_equipment(player, target, weapon|armor|horse_def|horse_off|treasure)"
                                .into(),
                    });
            }
            let ExprAst::Ident(slot) = &args[2] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "equipment slot must be weapon, armor, horse_def, horse_off, or treasure"
                            .into(),
                });
            };
            let slot = match slot.as_str() {
                "weapon" => 1,
                "armor" => 2,
                "horse_def" => 3,
                "horse_off" => 4,
                "treasure" => 5,
                _ => return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "equipment slot must be weapon, armor, horse_def, horse_off, or treasure"
                            .into(),
                }),
            };
            Ok(StmtIr::EffectGainTargetEquipment {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                slot,
                span: *span,
            })
        }
        "discard_all_target_equipment" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_target_equipment(actor, target)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardAllTargetEquipment {
                actor: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "increase_current_sha_target_damage" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "increase_current_sha_target_damage(source, card, target, amount)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectIncreaseCurrentShaTargetDamage {
                source: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_target_draw_then_owner_if_below_max_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_draw_then_owner_if_below_max_hp(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetDrawThenOwnerIfBelowMaxHp {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_bound_source_attack_range_damage" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_bound_source_attack_range_damage(player, source, amount)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseBoundSourceAttackRangeDamage {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_target_draw_max_hp_cap_then_discard_excess" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_draw_max_hp_cap_then_discard_excess(player, cap)"
                        .into(),
                });
            }
            let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|cap| *cap > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "maximum-HP draw/discard cap must be a positive integer".into(),
                })?;
            Ok(StmtIr::EffectChooseTargetDrawMaxHpCapThenDiscardExcess {
                player: check_expr(&args[0], locals)?,
                cap,
                span: *span,
            })
        }
        "choose_target_draw_to_max_hp_cap_then_add_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)"
                            .into(),
                });
            }
            let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|cap| *cap > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "maximum-HP draw cap must be a positive integer".into(),
                })?;
            let skill_def = expect_u32(&args[2], span.start as usize)?;
            if skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "granted skill def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetDrawMaxHpCapThenAddSkill {
                player: check_expr(&args[0], locals)?,
                cap,
                skill_def,
                span: *span,
            })
        }
        _ => super::choose_discard_heart_redirect_current_damage::check_effect_stmt(
            name, args, span, locals,
        ),
    }
}

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_zhizheng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_zhizheng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveZhizheng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_lijun" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_lijun(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveLijun {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_liangyin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_liangyin(player, stage, kongsheng_def)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if !matches!(stage, 0 | 3) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_liangyin stage must be 0 or 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveLiangyin {
                player: check_expr(&args[0], locals)?,
                kongsheng_def: expect_u32(&args[2], span.start as usize)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_reliangyin" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_reliangyin(player, self_skill, mode)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_reliangyin must reference self_skill".into(),
                });
            }
            let mode = expect_u32(&args[2], span.start as usize)?;
            Ok(StmtIr::EffectResolveReliangyin {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                mode,
                span: *span,
            })
        }
        "resolve_kongsheng" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_kongsheng(player, stage)".into(),
                });
            }
            let stage = expect_u32(&args[1], span.start as usize)?;
            if !matches!(stage, 0 | 2) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_kongsheng stage must be 0 or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolveKongsheng {
                player: check_expr(&args[0], locals)?,
                stage: stage as u8,
                span: *span,
            })
        }
        "resolve_pottuntian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pottuntian(player, action, zaoxian_def)".into(),
                });
            }
            let action = expect_u32(&args[1], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pottuntian action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolvePottuntian {
                player: check_expr(&args[0], locals)?,
                action: action as u8,
                zaoxian_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potjixi" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potjixi(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potjixi action must be 0, 1, or 2".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotjixi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potceduan" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potceduan(player, target, action)".into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potceduan action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotceduan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_pottaishici" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "resolve_pottaishici(player, target, action, hanzhan_def, zhanlie_def)"
                            .into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pottaishici action must be 0 through 5".into(),
                });
            }
            Ok(StmtIr::EffectResolvePottaishici {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                hanzhan_def: expect_u32(&args[3], span.start as usize)?,
                zhanlie_def: expect_u32(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_mbzhonghui" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhonghui(player, target, action, amount, mbsizi_def, mbkechang_def)".into(),
                    });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 7 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbzhonghui action must be 0 through 7".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbzhonghui {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                action: action as u8,
                amount: check_expr(&args[3], locals)?,
                mbsizi_def: expect_u32(&args[4], span.start as usize)?,
                mbkechang_def: expect_u32(&args[5], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potjiejie" => {
            if args.len() != 8 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjiejie(player, actor, used_def, spade_def, heart_def, club_def, diamond_def, qingshi_def)".into(),
                    });
            }
            Ok(StmtIr::EffectResolvePotjiejie {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                used_def: expect_u32(&args[2], span.start as usize)?,
                suit_defs: [
                    expect_u32(&args[3], span.start as usize)?,
                    expect_u32(&args[4], span.start as usize)?,
                    expect_u32(&args[5], span.start as usize)?,
                    expect_u32(&args[6], span.start as usize)?,
                ],
                qingshi_def: expect_u32(&args[7], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potqingshi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potqingshi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotqingshi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_potweiyan" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potweiyan(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 10 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potweiyan action must be 0 through 10".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotweiyan {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potzhouyu" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potzhouyu(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 7 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potzhouyu action must be 0 through 7".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotzhouyu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potlvyi" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potlvyi(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 9 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potlvyi action must be 0 through 9".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotlvyi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potcaoshuang" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potcaoshuang(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 11 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potcaoshuang action must be 0 through 11".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotcaoshuang {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_potyuji" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potyuji(player, target, card, action)".into(),
                });
            }
            let action = expect_u32(&args[3], span.start as usize)?;
            if action > 8 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potyuji action must be 0 through 8".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotyuji {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                action: action as u8,
                span: *span,
            })
        }
        "resolve_rehengjiang" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def)"
                        .into(),
                });
            }
            let action = expect_u32(&args[2], span.start as usize)?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_rehengjiang action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveRehengjiang {
                player: check_expr(&args[0], locals)?,
                actor: check_expr(&args[1], locals)?,
                action: action as u8,
                hand_limit_def: expect_u32(&args[3], span.start as usize)?,
                draw_def: expect_u32(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "choose_other_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_damage(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDamage {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_other_discard_target_card" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_discard_target_card(player, equip|hand_equip|hand_equip_judge|equip_judge)".into(),
                    });
            }
            let ExprAst::Ident(scope) = &args[1] else {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_discard_target_card scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
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
                            message: "choose_other_discard_target_card scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                        });
                }
            };
            Ok(StmtIr::EffectChooseOtherDiscardTargetCard {
                player: check_expr(&args[0], locals)?,
                scope,
                span: *span,
            })
        }
        "discard_all_hand_and_equip" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_hand_and_equip(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardAllHandAndEquip {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "remove_all_skills" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_all_skills(player)".into(),
                });
            }
            Ok(StmtIr::EffectRemoveAllSkills {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_gain_equipment_by_color" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_gain_equipment_by_color(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeGainEquipmentByColor {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_recover_on_suit" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_recover_on_suit(player, spade|heart|club|diamond, amount)"
                        .into(),
                });
            }
            let suit = match &args[1] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judgment suit must be spade, heart, club, or diamond".into(),
                    });
                }
            };
            Ok(StmtIr::EffectJudgeRecoverOnSuit {
                player: check_expr(&args[0], locals)?,
                suit,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "judge_recover_other_on_suit" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_recover_other_on_suit(judge_player, recover_player, spade|heart|club|diamond, amount)"
                            .into(),
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
                        message: "judgment suit must be spade, heart, club, or diamond".into(),
                    });
                }
            };
            Ok(StmtIr::EffectJudgeRecoverOtherOnSuit {
                judge_player: check_expr(&args[0], locals)?,
                recover_player: check_expr(&args[1], locals)?,
                suit,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "force_others_use_sha_on_nearest_or_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "force_others_use_sha_on_nearest_or_lose_hp(player)".into(),
                });
            }
            Ok(StmtIr::EffectForceOthersUseShaOnNearestOrLoseHp {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_damage_other_on_suit" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_damage_other_on_suit(player, spade|heart|club|diamond, amount, normal|fire|thunder)"
                            .into(),
                    });
            }
            let suit = match &args[1] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judgment suit must be spade, heart, club, or diamond".into(),
                    });
                }
            };
            let nature = match &args[3] {
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
            Ok(StmtIr::EffectJudgeDamageOtherOnSuit {
                player: check_expr(&args[0], locals)?,
                suit,
                amount: check_expr(&args[2], locals)?,
                nature,
                span: *span,
            })
        }
        "reveal_discard_suit_gain_or_use" => {
            check_reveal_discard_suit_gain_or_use(args, *span, locals)
        }
        "choose_discard_or_fire_damage_skip_phases" => {
            check_choose_discard_or_fire_damage_skip_phases(args, *span, locals)
        }
        "draw_selected" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_selected(player)".into(),
                });
            }
            Ok(StmtIr::EffectDrawSelected {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_each_by_seat" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_each_by_seat(first, second, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawEachBySeat {
                first: check_expr(&args[0], locals)?,
                second: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "draw_then_give_one" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_one(player, target, count, any_hand_equip|drawn_only|any_hand_equip_use_equipment)".into(),
                    });
            }
            let ExprAst::Ident(scope) = &args[3] else {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_one scope must be any_hand_equip, drawn_only, or any_hand_equip_use_equipment".into(),
                    });
            };
            let (drawn_only, use_given_equipment) = match scope.as_str() {
                    "any_hand_equip" => (false, false),
                    "drawn_only" => (true, false),
                    "any_hand_equip_use_equipment" => (false, true),
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "draw_then_give_one scope must be any_hand_equip, drawn_only, or any_hand_equip_use_equipment".into(),
                        })
                    }
                };
            Ok(StmtIr::EffectDrawThenGiveOne {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                drawn_only,
                use_given_equipment,
                span: *span,
            })
        }
        "draw_then_take_one" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_then_take_one(player, taker, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawThenTakeOne {
                player: check_expr(&args[0], locals)?,
                taker: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "take_one_card" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_one_card(from, to, hand_equip|equip|hand_equip_judge)".into(),
                });
            }
            let ExprAst::Ident(scope) = &args[2] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_one_card scope must be hand_equip, equip, or hand_equip_judge"
                        .into(),
                });
            };
            if scope == "hand_equip_judge" {
                return Ok(StmtIr::EffectTakeOneScopedCard {
                    from: check_expr(&args[0], locals)?,
                    to: check_expr(&args[1], locals)?,
                    scope: TargetCardScopeIr::HandEquipJudge,
                    span: *span,
                });
            }
            let equip_only = match scope.as_str() {
                "hand_equip" => false,
                "equip" => true,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "take_one_card scope must be hand_equip, equip, or hand_equip_judge"
                                .into(),
                    })
                }
            };
            Ok(StmtIr::EffectTakeOneCard {
                from: check_expr(&args[0], locals)?,
                to: check_expr(&args[1], locals)?,
                equip_only,
                span: *span,
            })
        }
        "take_one_event_target_hand" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_one_event_target_hand(player)".into(),
                });
            }
            Ok(StmtIr::EffectTakeOneHandCard {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_other_hand_store_skill_pile" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_hand_store_skill_pile(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherHandStoreSkillPile {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "gain_all_skill_pile_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_all_skill_pile_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainAllSkillPileCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "gain_other_skill_pile_cards" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_other_skill_pile_cards(player, target, skill_def)".into(),
                });
            }
            let skill_def = expect_u32(&args[2], span.start as usize)?;
            if skill_def == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_other_skill_pile_cards skill_def must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectGainOtherSkillPileCards {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill_def,
                span: *span,
            })
        }
        "choose_discard_one_he_or_lose_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_one_he_or_lose_hp(player, lose_amount)".into(),
                });
            }
            let lose_amount = expect_u32(&args[1], span.start as usize)?;
            if lose_amount == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_one_he_or_lose_hp lose_amount must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardOneHeOrLoseHp {
                player: check_expr(&args[0], locals)?,
                lose_amount,
                span: *span,
            })
        }
        "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(source, target)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_source_confirm_owner_lose_hp" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_source_confirm_owner_lose_hp(owner, source, amount)".into(),
                });
            }
            let amount = expect_u32(&args[2], span.start as usize)?;
            if amount == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_source_confirm_owner_lose_hp amount must be non-zero".into(),
                });
            }
            Ok(StmtIr::EffectChooseSourceConfirmOwnerLoseHp {
                owner: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                amount,
                span: *span,
            })
        }
        "choose_end_extra_draw_or_play_by_used_suits" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_end_extra_draw_or_play_by_used_suits(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseEndExtraDrawOrPlayByUsedSuits {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "lose_hp_double_current_damage_then_replace" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "lose_hp_double_current_damage_then_replace(player, replacement_skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectLoseHpDoubleCurrentDamageThenReplace {
                player: check_expr(&args[0], locals)?,
                replacement_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "take_target_hand_then_repeat_until_max" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_target_hand_then_repeat_until_max(player, target, skill)".into(),
                });
            }
            Ok(StmtIr::EffectTakeTargetHandThenRepeatUntilMax {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "gain_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_event_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectGainEventCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_target_draw_to_max_hp_cap" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_draw_to_max_hp_cap(player, cap)".into(),
                });
            }
            let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|cap| *cap > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "maximum-HP draw cap must be a positive integer".into(),
                })?;
            Ok(StmtIr::EffectChooseTargetDrawToMaxHpCap {
                player: check_expr(&args[0], locals)?,
                cap,
                span: *span,
            })
        }
        "choose_other_draw_owner_lost_hp_turn_over" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_other_draw_owner_lost_hp_turn_over(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDrawOwnerLostHpTurnOver {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "turn_over" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "turn_over(player)".into(),
                });
            }
            Ok(StmtIr::EffectTurnOver {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "restore_character_state" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "restore_character_state(player)".into(),
                });
            }
            Ok(StmtIr::EffectRestoreCharacterState {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "discard_all_hand_equip_judge" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_hand_equip_judge(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardAllHandEquipJudge {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "damage_then_discard_all_judge" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "damage_then_discard_all_judge(player)".into(),
                });
            }
            Ok(StmtIr::EffectDamageThenDiscardAllJudge {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "replace_draw_gain_top_two_add_skills_if_different_colors" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "replace_draw_gain_top_two_add_skills_if_different_colors(player, first_skill_def, second_skill_def)"
                                .into(),
                    });
            }
            Ok(
                StmtIr::EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player: check_expr(&args[0], locals)?,
                    first_skill_def: expect_u32(&args[1], span.start as usize)?,
                    second_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                },
            )
        }
        "choose_gain_target_card_alternate_zone_then_target_draw" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_gain_target_card_alternate_zone_then_target_draw(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_discard_target_card_resolve_current_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_target_card_resolve_current_sha(source, card, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardTargetCardResolveCurrentSha {
                source: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_discard_target_card_resolve_current_sha_nonbasic_bonus" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_target_card_resolve_current_sha_nonbasic_bonus(source, card, target, bonus_skill_def)"
                                .into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                    source: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    bonus_skill_def: expect_u32(&args[3], span.start as usize)?,
                    span: *span,
                },
            )
        }
        "resolve_dcqiaomeng" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dcqiaomeng(player, card, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDcqiaomeng {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "remove_skill_by_def" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_skill_by_def(player, skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectRemoveSkillByDef {
                player: check_expr(&args[0], locals)?,
                skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "choose_discard_hand_add_skill_by_color" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_add_skill_by_color(player, red_skill_def, black_skill_def)".into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardHandAddSkillByColor {
                player: check_expr(&args[0], locals)?,
                red_skill_def: expect_u32(&args[1], span.start as usize)?,
                black_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "reveal_top_gain_non_basic_or_swap_basic" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_top_gain_non_basic_or_swap_basic(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealTopGainNonBasicOrSwapBasic {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reveal_top_gain_if_color" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_top_gain_if_color(player, red|black)".into(),
                });
            }
            let color_mask = match &args[1] {
                ExprAst::Ident(color) if color == "red" => 1,
                ExprAst::Ident(color) if color == "black" => 2,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_top_gain_if_color color must be red or black".into(),
                    });
                }
            };
            Ok(StmtIr::EffectRevealTopGainIfColor {
                player: check_expr(&args[0], locals)?,
                color_mask,
                span: *span,
            })
        }
        "prevent_current_damage" => {
            if !args.is_empty() {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "prevent_current_damage()".into(),
                });
            }
            Ok(StmtIr::EffectPreventCurrentDamage { span: *span })
        }
        "judge_cancel_current_sha_target_on_red" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_cancel_current_sha_target_on_red(player, card, target)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeCancelCurrentShaTargetOnRed {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_draw_or_recover" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_draw_or_recover(player, draw_count, recover_count, allow_cancel)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseDrawOrRecover {
                player: check_expr(&args[0], locals)?,
                draw_count: check_expr(&args[1], locals)?,
                recover_count: check_expr(&args[2], locals)?,
                allow_cancel: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "take_hand" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "take_hand(from, to)".into(),
                });
            }
            Ok(StmtIr::EffectTakeHand {
                from: check_expr(&args[0], locals)?,
                to: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_all_hand" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_all_hand(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveAllHand {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "gain_card_if_discard" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_card_if_discard(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectGainCardIfDiscard {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_draw_or_discard_target_card_record_sha" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_draw_or_discard_target_card_record_sha(player, card, target)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectChooseDrawOrDiscardTargetCardRecordSha {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "reveal_hand_to_player" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand_to_player(player, observer)".into(),
                });
            }
            Ok(StmtIr::EffectRevealHandToPlayer {
                player: check_expr(&args[0], locals)?,
                observer: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "lose_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "lose_hp(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectLoseHp {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "add_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_skill(player, skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectAddSkill {
                player: check_expr(&args[0], locals)?,
                skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "add_bound_skill" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_bound_skill(player, skill_def, target)".into(),
                });
            }
            Ok(StmtIr::EffectAddBoundSkill {
                player: check_expr(&args[0], locals)?,
                skill_def: expect_u32(&args[1], span.start as usize)?,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "replace_skill_with_bound_after_resolution_if_target_alive" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "replace_skill_with_bound_after_resolution_if_target_alive(player, skill, skill_def, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    skill_def: expect_u32(&args[2], span.start as usize)?,
                    target: check_expr(&args[3], locals)?,
                    span: *span,
                },
            )
        }
        "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)"
                        .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                },
            )
        }
        "choose_discard_hand_equip_then_recover" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_discard_hand_equip_then_recover(player, count, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseDiscardHandEquipThenRecover {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "remove_skill" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_skill(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectRemoveSkill {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "refresh_skill" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "refresh_skill(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectRefreshSkill {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "tag_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "tag_event_cards(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectTagEventCards {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "untag_event_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "untag_event_cards(skill_instance)".into(),
                });
            }
            Ok(StmtIr::EffectUntagEventCards {
                skill: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "storage_set" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "storage_set(skill_instance, key, value)".into(),
                });
            }
            Ok(StmtIr::EffectStorageSet {
                skill: check_expr(&args[0], locals)?,
                key: check_expr(&args[1], locals)?,
                value: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "draw_all_event_targets" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_all_event_targets(count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawAllEventTargets {
                count: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "clanshenjun_reveal" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanshenjun_reveal(self, event_card, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanshenjunReveal {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "clanshenjun_use" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanshenjun_use(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanshenjunUse {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "clanyuzhi_reveal" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanyuzhi_reveal(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanyuzhiReveal {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "clanyuzhi_round_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "clanyuzhi_round_end(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectClanyuzhiRoundEnd {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        _ => super::qieyi_phase_start::check_effect_stmt(name, args, span, locals),
    }
}

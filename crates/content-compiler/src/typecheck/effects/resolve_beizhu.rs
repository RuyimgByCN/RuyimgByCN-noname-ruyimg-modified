use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "resolve_beizhu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_beizhu(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveBeizhu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbshanjia" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbshanjia(player, lost_equipment_count, distance_skill_def)"
                        .into(),
                });
            }
            let distance_skill_def = expect_u32(&args[2], span.start as usize)?;
            Ok(StmtIr::EffectResolveMbshanjia {
                player: check_expr(&args[0], locals)?,
                lost_equipment_count: check_expr(&args[1], locals)?,
                distance_skill_def,
                span: *span,
            })
        }
        "resolve_daoji" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_daoji(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveDaoji {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_tw_daoji" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_tw_daoji(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwDaoji {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "give_all_hand_then_choose_use_one_or_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "give_all_hand_then_choose_use_one_or_damage(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectGiveAllHandThenChooseUseOneOrDamage {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_category_then_target_give_matching_or_sha_unrespondable" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_category_then_target_give_matching_or_sha_unrespondable(player, card, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                },
            )
        }
        "choose_category_then_target_give_matching_or_damage_plus_one" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_category_then_target_give_matching_or_damage_plus_one(player, target)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(player, source)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "resolve_mobiledanshou" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mobiledanshou(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMobiledanshou {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_jsrgchiying" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jsrgchiying(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJsrgchiying {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbqingdao" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbqingdao(player, source, card, damaged)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbqingdao {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                damaged: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "resolve_mbxiugeng" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbxiugeng(player, helper_def, hand_limit_def, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbxiugeng {
                player: check_expr(&args[0], locals)?,
                helper_def: expect_u32(&args[1], span.start as usize)?,
                hand_limit_def: expect_u32(&args[2], span.start as usize)?,
                stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "修耕 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbchenshe" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbchenshe(player, dying, source)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbchenshe {
                player: check_expr(&args[0], locals)?,
                dying: check_expr(&args[1], locals)?,
                source: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_mbquanchong" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbquanchong(player, helper_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbquanchong {
                player: check_expr(&args[0], locals)?,
                helper_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_mbrenxing" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbrenxing(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbrenxing {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbchizhang" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbchizhang(player, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbchizhang {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbduanyang" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbduanyang(player, target, card, mode)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbduanyang {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                mode: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "断鞅 mode must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbrunwei" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbrunwei(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbrunwei {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "润微 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbshuanghuai" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbshuanghuai(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbshuanghuai {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbxiongtu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbxiongtu(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbxiongtu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "凶图 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbganggeng" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbganggeng(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbganggeng {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "刚鲠 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbsijian" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbsijian(player, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbsijian {
                player: check_expr(&args[0], locals)?,
                helper_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_mbjianji" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbjianji(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbjianji {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbyuanmo" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbyuanmo(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbyuanmo {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_mbfeijing" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbfeijing(player, target, card)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbfeijing {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_mbxiaoge" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbxiaoge(player, target, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbxiaoge {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "骁戈 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbxuye" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbxuye(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbxuye {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "resolve_mbkuangxiang" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbkuangxiang(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbkuangxiang {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "狂想 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potchengpu" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potchengpu(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotchengpu {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势程普 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potjugu" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potjugu(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotjugu {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_potsifeng" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potsifeng(player, target, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotsifeng {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "伺锋 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potchenqun" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potchenqun(player, target, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotchenqun {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势陈群 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potwanglie" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potwanglie(player, card, helper_skill_def, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotwanglie {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势陈到往烈 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_pothongyi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pothongyi(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePothongyi {
                player: check_expr(&args[0], locals)?,
                stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势陈到弘毅 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potguansha" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potguansha(player, helper_skill_def)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotguansha {
                player: check_expr(&args[0], locals)?,
                helper_skill_def: expect_u32(&args[1], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potjiyu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potjiyu(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotjiyu {
                player: check_expr(&args[0], locals)?,
                stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势娄圭急御 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_mbzhuji" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_mbzhuji(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolveMbzhuji {
                player: check_expr(&args[0], locals)?,
                stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "兵势孙韶筑墼 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potlibing" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potlibing(player, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotlibing {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势夏侯霸砺兵 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potpoxi" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potpoxi(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotpoxi {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_potfuan" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potfuan(player, card, stage, range_helper_def, yinxian_def)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolvePotfuan {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势张任伏暗 stage must fit u8".into(),
                    }
                })?,
                range_helper_def: expect_u32(&args[3], span.start as usize)?,
                yinxian_def: expect_u32(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potyinxian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potyinxian(player, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotyinxian {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势张任引弦 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_pothuanjie" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pothuanjie(player, target, stage, qice_def, kanpo_def)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolvePothuanjie {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势桓阶 resolver stage must fit u8".into(),
                    }
                })?,
                qice_def: expect_u32(&args[3], span.start as usize)?,
                kanpo_def: expect_u32(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "resolve_potnigu" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potnigu(player, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotnigu {
                player: check_expr(&args[0], locals)?,
                stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势孙綝逆固 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potlulian" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potlulian(player, card, stage)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotlulian {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "势孙綝戮连 stage must fit u8".into(),
                    }
                })?,
                span: *span,
            })
        }
        "resolve_potdimeng" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_potdimeng(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolvePotdimeng {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_pothaoshi" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_pothaoshi(player, action)".into(),
                });
            }
            let action =
                u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "pothaoshi action must fit u8".into(),
                    }
                })?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "pothaoshi content action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolvePothaoshi {
                player: check_expr(&args[0], locals)?,
                action,
                span: *span,
            })
        }
        "resolve_spmiaolve" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spmiaolve(player, action)".into(),
                });
            }
            let action =
                u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "spmiaolve action must fit u8".into(),
                    }
                })?;
            if action > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spmiaolve action must be 0 or 1".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpmiaolve {
                player: check_expr(&args[0], locals)?,
                action,
                span: *span,
            })
        }
        "resolve_spyingjia" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_spyingjia(player, action, helper_def)".into(),
                });
            }
            let action =
                u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                    CompileError::Type {
                        pos: span.start as usize,
                        message: "spyingjia action must fit u8".into(),
                    }
                })?;
            if action > 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "spyingjia action must be 0, 1, 2, or 3".into(),
                });
            }
            Ok(StmtIr::EffectResolveSpyingjia {
                player: check_expr(&args[0], locals)?,
                action,
                helper_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "choose_lose_hp_use_virtual_sha_by_basic_variants" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_lose_hp_use_virtual_sha_by_basic_variants(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseLoseHpUseVirtualShaByBasicVariants {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_twfupan" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_twfupan(player, damage_amount)".into(),
                });
            }
            Ok(StmtIr::EffectResolveTwfupan {
                player: check_expr(&args[0], locals)?,
                damage_amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "suppress_non_locked_skills_until_turn_end" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "suppress_non_locked_skills_until_turn_end(player)".into(),
                });
            }
            Ok(StmtIr::EffectSuppressNonLockedSkillsUntilTurnEnd {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_recover_target" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_recover_target(chooser, target, amount)".into(),
                });
            }
            Ok(StmtIr::EffectChooseRecoverTarget {
                chooser: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                amount: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "resolve_jiaohua" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_jiaohua(player)".into(),
                });
            }
            Ok(StmtIr::EffectResolveJiaohua {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "resolve_luanqun" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_luanqun(player, effect_skill_def, direct_hit_skill_def)"
                        .into(),
                });
            }
            Ok(StmtIr::EffectResolveLuanqun {
                player: check_expr(&args[0], locals)?,
                effect_skill_def: expect_u32(&args[1], span.start as usize)?,
                direct_hit_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "choose_discard_basic_then_target_discard_equip_or_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_discard_basic_then_target_discard_equip_or_damage(player, target)"
                            .into(),
                });
            }
            Ok(
                StmtIr::EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                },
            )
        }
        "choose_current_sha_target_discard_redirect_to_owner" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)"
                                .into(),
                    });
            }
            Ok(StmtIr::EffectChooseCurrentShaTargetDiscardRedirectToOwner {
                owner: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                target: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_target_gain_bottom_card" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_gain_bottom_card(player)".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetGainBottomCard {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_black_gain_repeat" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_black_gain_repeat(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeBlackGainRepeat {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_black_gain_repeat_protect_hand_limit" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_black_gain_repeat_protect_hand_limit(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeBlackGainRepeatProtectHandLimit {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "judge_same_color_gain_repeat" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_same_color_gain_repeat(player)".into(),
                });
            }
            Ok(StmtIr::EffectJudgeSameColorGainRepeat {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_current_trick_targets_to_nullify" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_current_trick_targets_to_nullify(player, source, card)".into(),
                });
            }
            Ok(StmtIr::EffectChooseCurrentTrickTargetsToNullify {
                player: check_expr(&args[0], locals)?,
                source: check_expr(&args[1], locals)?,
                card: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "use_virtual_juedou_second_on_first" => {
            if !args.is_empty() {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_juedou_second_on_first()".into(),
                });
            }
            Ok(StmtIr::EffectUseVirtualJuedouSecondOnFirst { span: *span })
        }
        effect @ ("choose_discard_target_card" | "choose_optional_discard_target_card") => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: format!(
                        "{effect}(player, target, hand|equip|hand_equip|hand_equip_judge)"
                    ),
                });
            }
            let ExprAst::Ident(scope) = &args[2] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: format!(
                        "{effect} scope must be hand, equip, hand_equip, or hand_equip_judge"
                    ),
                });
            };
            let scope = match scope.as_str() {
                "hand" => TargetCardScopeIr::Hand,
                "equip" => TargetCardScopeIr::Equip,
                "hand_equip" => TargetCardScopeIr::HandEquip,
                "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                "equip_judge" => TargetCardScopeIr::EquipJudge,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: format!(
                            "{effect} scope must be hand, equip, hand_equip, or hand_equip_judge"
                        ),
                    });
                }
            };
            let player = check_expr(&args[0], locals)?;
            let target = check_expr(&args[1], locals)?;
            Ok(if effect == "choose_discard_target_card" {
                StmtIr::EffectChooseDiscardTargetCard {
                    player,
                    target,
                    scope,
                    span: *span,
                }
            } else {
                StmtIr::EffectChooseOptionalDiscardTargetCard {
                    player,
                    target,
                    scope,
                    span: *span,
                }
            })
        }
        "reveal_hand_choose_discard_color" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand_choose_discard_color(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealHandChooseDiscardColor {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "reveal_hand" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_hand(player)".into(),
                });
            }
            Ok(StmtIr::EffectRevealHand {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "draw_linked_players" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_linked_players(count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawLinkedPlayers {
                count: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_target_cards_recover_same_category" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_target_cards_recover_same_category(player, target, count, amount)".into(),
                    });
            }
            Ok(StmtIr::EffectChooseDiscardTargetCardsRecoverSameCategory {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                amount: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "choose_scskuiji_discard" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_scskuiji_discard(player, target, count)".into(),
                });
            }
            Ok(StmtIr::EffectChooseScskuijiDiscard {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_pskuiji_discard" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_pskuiji_discard(player, target, count)".into(),
                });
            }
            Ok(StmtIr::EffectChoosePskuijiDiscard {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                count: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_compare_selected_hand_card_second_failure_seat_damage" => {
            if args.len() != 6 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_compare_selected_hand_card_second_failure_seat_damage(player, target, draw_count, failure_count, lose_hp, damage)".into(),
                    });
            }
            Ok(
                StmtIr::EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    draw_count: check_expr(&args[2], locals)?,
                    failure_count: check_expr(&args[3], locals)?,
                    lose_hp: check_expr(&args[4], locals)?,
                    damage: check_expr(&args[5], locals)?,
                    span: *span,
                },
            )
        }
        "judge_prevent_damage_lose_max_hp_on_nonheart" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "judge_prevent_damage_lose_max_hp_on_nonheart(player, target)".into(),
                });
            }
            Ok(StmtIr::EffectJudgePreventDamageLoseMaxHpOnNonHeart {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_gain_top_cards_by_suit" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_gain_top_cards_by_suit(player, count, spade|heart|club|diamond)"
                            .into(),
                });
            }
            let count = u8::try_from(expect_int(&args[1], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "top-card view count must be 1..255".into(),
                })?;
            let suit = match &args[2] {
                ExprAst::Ident(name) if name == "spade" => 1,
                ExprAst::Ident(name) if name == "heart" => 2,
                ExprAst::Ident(name) if name == "club" => 3,
                ExprAst::Ident(name) if name == "diamond" => 4,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "top-card selection suit must be spade, heart, club, or diamond"
                            .into(),
                    });
                }
            };
            Ok(StmtIr::EffectChooseGainTopCardsBySuit {
                player: check_expr(&args[0], locals)?,
                count,
                suit,
                span: *span,
            })
        }
        "choose_other_draw_by_hand_count" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_other_draw_by_hand_count(player, normal_count, empty_count, times)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseOtherDrawByHandCount {
                player: check_expr(&args[0], locals)?,
                normal_count: check_expr(&args[1], locals)?,
                empty_count: check_expr(&args[2], locals)?,
                times: check_expr(&args[3], locals)?,
                span: *span,
            })
        }
        "discard_one_hand_or_equip" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_one_hand_or_equip(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardOneHandOrEquip {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "choose_discard_one_then_gain_target_equipment" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_then_gain_target_equipment(player, target, weapon|armor|horse_def|horse_off|treasure)".into(),
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
            Ok(StmtIr::EffectChooseDiscardOneThenGainTargetEquipment {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                slot,
                span: *span,
            })
        }
        "choose_target_draw" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_draw(player, count, other|male_other|female_other|any|min_hand|same_hp)"
                                .into(),
                    });
            }
            let scope = expect_draw_target_scope(&args[2], span.start as usize)?;
            Ok(StmtIr::EffectChooseTargetDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                scope,
                span: *span,
            })
        }
        "choose_targets_draw" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_targets_draw(player, count, target_count, other|male_other|female_other|any|min_hand|same_hp)".into(),
                    });
            }
            let target_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "draw target count must be 1..255".into(),
                })?;
            let scope = expect_draw_target_scope(&args[3], span.start as usize)?;
            Ok(StmtIr::EffectChooseTargetsDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                target_count,
                scope,
                span: *span,
            })
        }
        "choose_optional_targets_draw" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_optional_targets_draw(player, count, min_count, max_count, scope)"
                            .into(),
                });
            }
            let min_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "draw target minimum must be 1..255".into(),
                })?;
            let max_count = u8::try_from(expect_int(&args[3], span.start as usize)?)
                .ok()
                .filter(|count| *count >= min_count)
                .ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "draw target maximum must be min_count..255".into(),
                })?;
            Ok(StmtIr::EffectChooseOptionalTargetsDraw {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                min_count,
                max_count,
                scope: expect_draw_target_scope(&args[4], span.start as usize)?,
                span: *span,
            })
        }
        "reveal_target_hand_sha_or_discard_bind" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)"
                            .into(),
                });
            }
            Ok(StmtIr::EffectRevealTargetHandShaOrDiscardBind {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                span: *span,
            })
        }
        "draw_remembered_hand_cards" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_remembered_hand_cards(player, count)".into(),
                });
            }
            Ok(StmtIr::EffectDrawRememberedHandCards {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_remembered_hand_cards" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_remembered_hand_cards(player)".into(),
                });
            }
            Ok(StmtIr::EffectDiscardRememberedHandCards {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        _ => super::gain_draw_cards_for_lifetime_card_action_divisors::check_effect_stmt(
            name, args, span, locals,
        ),
    }
}

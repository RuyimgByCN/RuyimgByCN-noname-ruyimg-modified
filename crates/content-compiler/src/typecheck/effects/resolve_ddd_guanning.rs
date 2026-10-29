//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `resolve_ddd_guanning_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
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
        "choose_give_or_recast_one_hand" => {
            if args.len() != 1 && args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_give_or_recast_one_hand(player)".into(),
                });
            }
            let (require_skill_def, grant_skill_def, grant_mark_def) = if args.len() == 4 {
                (
                    expect_u32(&args[1], span.start as usize)?,
                    expect_u32(&args[2], span.start as usize)?,
                    expect_u32(&args[3], span.start as usize)?,
                )
            } else {
                (0, 0, 0)
            };
            Ok(StmtIr::EffectChooseGiveOrRecastOneHand {
                player: check_expr(&args[0], locals)?,
                require_skill_def,
                grant_skill_def,
                grant_mark_def,
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
        _ => super::resolve_ddd_guanning_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

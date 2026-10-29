//! 效果语句类型检查链：通用效果原语（具名技能结算器见下一环 `resolve_heji_resolvers`）。

use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "equip_weapon_from_draw_pile" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "equip_weapon_from_draw_pile(player)".into(),
                });
            }
            Ok(StmtIr::EffectEquipWeaponFromDrawPile {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "set_skill_content_flag" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "set_skill_content_flag(player, bit, 0|1)".into(),
                });
            }
            let bit = literal_i32(&args[1]).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: "skill content flag bit must be a literal".into(),
            })?;
            let value = literal_i32(&args[2]).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: "skill content flag value must be 0 or 1".into(),
            })?;
            if !(0..64).contains(&bit) || !matches!(value, 0 | 1) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "set_skill_content_flag requires bit 0..63 and value 0|1".into(),
                });
            }
            Ok(StmtIr::EffectSetSkillContentFlag {
                player: check_expr(&args[0], locals)?,
                bit: bit as u8,
                value: value != 0,
                span: *span,
            })
        }
        "choose_target_reclaim_equipment_draw_if_range_reduced" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_reclaim_equipment_draw_if_range_reduced(player, target, self_skill)"
                                .into(),
                    });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "target-equipment reclamation must reference self_skill".into(),
                });
            }
            Ok(
                StmtIr::EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                },
            )
        }
        "choose_use_virtual_card" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_use_virtual_card(player, wanjian|juedou, self_skill)".into(),
                });
            }
            let ExprAst::Ident(kind_name) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_use_virtual_card second argument must be a card kind".into(),
                });
            };
            if !matches!(kind_name.as_str(), "wanjian" | "juedou") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_use_virtual_card supports wanjian or juedou".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "virtual-card use must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseUseVirtualCard {
                player: check_expr(&args[0], locals)?,
                kind: card_kind_value(kind_name).expect("validated card kind"),
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_use_virtual_nanman" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_use_virtual_nanman(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "virtual-card use must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseUseVirtualNanman {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_all_own_judge" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_own_judge(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_all_own_judge must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectDiscardAllOwnJudge {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "store_event_card_in_skill_pile" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "store_event_card_in_skill_pile(player, card, self_skill)".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "store_event_card_in_skill_pile must reference self_skill".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "store-event-card card must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectStoreEventCardInSkillPile {
                player: check_expr(&args[0], locals)?,
                card,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_target_gain_max_hp" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_gain_max_hp(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_target_gain_max_hp must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseTargetGainMaxHp {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_basic_draw_equal" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_basic_draw_equal(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_basic_draw_equal must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectDiscardBasicDrawEqual {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "start_debate_all_others" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "start_debate_all_others(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "start_debate_all_others must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectStartDebateAllOthers {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "start_debate_targets" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "start_debate_targets(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "start_debate_targets must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectStartDebateTargets {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "pin_debate_opinions" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "pin_debate_opinions(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "pin_debate_opinions must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectPinDebateOpinions {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "align_debate_opinions" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "align_debate_opinions(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "align_debate_opinions must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectAlignDebateOpinions {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "gain_shadow_cards" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_shadow_cards(player, count, self_skill)".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_shadow_cards must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectGainShadowCards {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "change_group" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "change_group(player, \"wei|shu|wu|qun|jin\", self_skill)".into(),
                });
            }
            let ExprAst::Ident(group) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "change_group group must be a faction ident (wei/shu/wu/qun/jin)"
                        .into(),
                });
            };
            if !matches!(group.as_str(), "wei" | "shu" | "wu" | "qun" | "jin") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "change_group unknown faction".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "change_group must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChangeGroup {
                player: check_expr(&args[0], locals)?,
                group: group.clone(),
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "distribute_shadow_cards" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "distribute_shadow_cards(player, count, self_skill)".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "distribute_shadow_cards must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectDistributeShadowCards {
                player: check_expr(&args[0], locals)?,
                count: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_group" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_group(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_group must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseGroup {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_gain_other_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_other_skill(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_other_skill must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseGainOtherSkill {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "lose_gained_skill" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "lose_gained_skill(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "lose_gained_skill must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectLoseGainedSkill {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "place_xumou" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_xumou(player, card, self_skill)".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_xumou must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectPlaceXumou {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "place_deck_top_xumou" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_deck_top_xumou(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_deck_top_xumou must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectPlaceDeckTopXumou {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_place_hand_xumou" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_place_hand_xumou(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_place_hand_xumou must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChoosePlaceHandXumou {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "discard_one_own_xumou" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_one_own_xumou(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "discard_one_own_xumou must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectDiscardOneOwnXumou {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "place_deck_top_expansion" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_deck_top_expansion(player, count, self_skill)".into(),
                });
            }
            let count = expect_u32(&args[1], span.start as usize)?;
            if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "place_deck_top_expansion must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectPlaceDeckTopExpansion {
                player: check_expr(&args[0], locals)?,
                count,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }

        "city_consume" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "city_consume(player, self_skill)".into(),
                });
            }
            if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "city_consume must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectCityConsume {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "queue_extra_turn" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_extra_turn(player, check, self_skill)".into(),
                });
            }
            let check = expect_u32(&args[1], span.start as usize)?;
            if check > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_extra_turn check must be 0 or 1".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_extra_turn must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectQueueExtraTurn {
                player: check_expr(&args[0], locals)?,
                check,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "choose_others_insert_extra_turn" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_others_insert_extra_turn(player, check, self_skill)".into(),
                });
            }
            let check = expect_u32(&args[1], span.start as usize)?;
            if check > 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_others_insert_extra_turn check must be 0 or 1".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_others_insert_extra_turn must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectChooseOthersInsertExtraTurn {
                player: check_expr(&args[0], locals)?,
                check,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "queue_extra_phases" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_extra_phases(player, phases_mask, self_skill)".into(),
                });
            }
            let phases_mask = expect_u32(&args[1], span.start as usize)?;
            if phases_mask == 0 || phases_mask > 0b0111_110 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_extra_phases mask out of range".into(),
                });
            }
            if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "queue_extra_phases must reference self_skill".into(),
                });
            }
            Ok(StmtIr::EffectQueueExtraPhases {
                player: check_expr(&args[0], locals)?,
                phases_mask,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        _ => super::resolve_heji_resolvers::check_effect_stmt(name, args, span, locals),
    }
}

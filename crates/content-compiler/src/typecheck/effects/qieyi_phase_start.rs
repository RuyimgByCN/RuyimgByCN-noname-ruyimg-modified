use super::*;

pub(in super::super) fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
        "qieyi_phase_start" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "qieyi_phase_start(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectQieyiPhaseStart {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "qieyi_use_after" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "qieyi_use_after(self, event_card, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectQieyiUseAfter {
                player: check_expr(&args[0], locals)?,
                card: check_expr(&args[1], locals)?,
                skill: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "jianzhi_phase_end" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "jianzhi_phase_end(self, self_skill)".into(),
                });
            }
            Ok(StmtIr::EffectJianzhiPhaseEnd {
                player: check_expr(&args[0], locals)?,
                skill: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "move_card" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "move_card(card, to_zone, from_zone_or_zero)".into(),
                });
            }
            let card = check_expr(&args[0], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "move_card first argument must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectMoveCard {
                card,
                to: check_expr(&args[1], locals)?,
                from: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "set_turn_state" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "set_turn_state(current, phase_tag)".into(),
                });
            }
            let phase = expect_int(&args[1], span.start as usize)?;
            if !(0..=6).contains(&phase) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "phase_tag must be in 0..=6".into(),
                });
            }
            Ok(StmtIr::EffectSetTurnState {
                current: check_expr(&args[0], locals)?,
                phase,
                span: *span,
            })
        }
        "use_card_on" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_card_on(player, card, target)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_card_on second argument must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectUseCardOn {
                player: check_expr(&args[0], locals)?,
                card,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "use_virtual_card_on" => {
            if args.len() != 3 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_card_on(player, card_kind, target)".into(),
                });
            }
            let ExprAst::Ident(kind_name) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_card_on second argument must be a card kind".into(),
                });
            };
            let kind = card_kind_value(kind_name).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown virtual card kind {kind_name}"),
            })?;
            Ok(StmtIr::EffectUseVirtualCardOn {
                player: check_expr(&args[0], locals)?,
                kind,
                target: check_expr(&args[2], locals)?,
                span: *span,
            })
        }
        "use_virtual_card_on_selected_targets" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_card_on_selected_targets(player, card_kind)".into(),
                });
            }
            let ExprAst::Ident(kind_name) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "use_virtual_card_on_selected_targets second argument must be a card kind"
                            .into(),
                });
            };
            if !matches!(kind_name.as_str(), "juedou" | "tiesuo") {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_virtual_card_on_selected_targets supports juedou or tiesuo"
                        .into(),
                });
            }
            let kind = card_kind_value(kind_name).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown virtual card kind {kind_name}"),
            })?;
            Ok(StmtIr::EffectUseVirtualCardOnSelectedTargets {
                player: check_expr(&args[0], locals)?,
                kind,
                span: *span,
            })
        }
        "enter_dying" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "enter_dying(player)".into(),
                });
            }
            Ok(StmtIr::EffectEnterDying {
                player: check_expr(&args[0], locals)?,
                span: *span,
            })
        }
        "kill_player" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "kill_player(player, killer)".into(),
                });
            }
            Ok(StmtIr::EffectKillPlayer {
                player: check_expr(&args[0], locals)?,
                killer: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "reveal_card" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_card(card)".into(),
                });
            }
            let card = check_expr(&args[0], locals)?;
            // Card operand must be a card-typed expression, not a player/int.
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_card expects a card operand (e.g. event_card)".into(),
                });
            }
            Ok(StmtIr::EffectRevealCard { card, span: *span })
        }
        "use_card" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_card(player, card)".into(),
                });
            }
            let player = check_expr(&args[0], locals)?;
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "use_card second argument must be a card operand".into(),
                });
            }
            Ok(StmtIr::EffectUseCard {
                player,
                card,
                span: *span,
            })
        }
        "choose_use_virtual_sha" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_use_virtual_sha(player, normal_distance|ignore_distance)"
                        .into(),
                });
            }
            let ExprAst::Ident(policy) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "virtual Sha distance policy must be normal_distance or ignore_distance"
                            .into(),
                });
            };
            let ignore_distance = match policy.as_str() {
                "normal_distance" => false,
                "ignore_distance" => true,
                _ => {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "virtual Sha distance policy must be normal_distance or ignore_distance"
                                .into(),
                    });
                }
            };
            Ok(StmtIr::EffectChooseUseVirtualSha {
                player: check_expr(&args[0], locals)?,
                ignore_distance,
                span: *span,
            })
        }
        "remove_cards_by_kind" => {
            if args.len() != 1 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_cards_by_kind(card_kind)".into(),
                });
            }
            let ExprAst::Ident(kind) = &args[0] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "remove_cards_by_kind expects a card-kind name".into(),
                });
            };
            let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown card kind {kind}"),
            })?;
            Ok(StmtIr::EffectRemoveCardsByKind { kind, span: *span })
        }
        "gain_card_by_kind" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_card_by_kind(player, card_kind)".into(),
                });
            }
            let ExprAst::Ident(kind) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_card_by_kind expects a card-kind name".into(),
                });
            };
            let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown card kind {kind}"),
            })?;
            Ok(StmtIr::EffectGainCardByKind {
                player: check_expr(&args[0], locals)?,
                kind,
                span: *span,
            })
        }
        "gain_discard_card_by_kind" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_discard_card_by_kind(player, card_kind)".into(),
                });
            }
            let ExprAst::Ident(kind) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "gain_discard_card_by_kind expects a card-kind name".into(),
                });
            };
            let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown card kind {kind}"),
            })?;
            Ok(StmtIr::EffectGainDiscardCardByKind {
                player: check_expr(&args[0], locals)?,
                kind,
                span: *span,
            })
        }
        "reveal_uniform_hand_choose_damage" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "reveal_uniform_hand_choose_damage(player, amount)".into(),
                });
            }
            Ok(StmtIr::EffectRevealUniformHandChooseDamage {
                player: check_expr(&args[0], locals)?,
                amount: check_expr(&args[1], locals)?,
                span: *span,
            })
        }
        "choose_gain_discard_card_by_kind" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_discard_card_by_kind(player, card_kind)".into(),
                });
            }
            let ExprAst::Ident(kind) = &args[1] else {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_gain_discard_card_by_kind expects a card-kind name".into(),
                });
            };
            let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown card kind {kind}"),
            })?;
            Ok(StmtIr::EffectChooseGainDiscardCardByKind {
                player: check_expr(&args[0], locals)?,
                kind,
                span: *span,
            })
        }
        "draw_for_lifetime_card_action_divisors" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "draw_for_lifetime_card_action_divisors(player, first, second, third)"
                        .into(),
                });
            }
            let mut divisors = [0_u32; 3];
            for (index, arg) in args[1..].iter().enumerate() {
                let ExprAst::Int(value) = arg else {
                    return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "draw_for_lifetime_card_action_divisors expects three positive integer literals".into(),
                        });
                };
                if *value <= 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_for_lifetime_card_action_divisors divisors must be positive"
                            .into(),
                    });
                }
                divisors[index] = *value as u32;
            }
            Ok(StmtIr::EffectDrawForLifetimeCardActionDivisors {
                player: check_expr(&args[0], locals)?,
                divisors,
                span: *span,
            })
        }
        "choose_add_missing_skill" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "choose_add_missing_skill(player, first, second, third)".into(),
                });
            }
            let skill_defs = [
                expect_u32(&args[1], span.start as usize)?,
                expect_u32(&args[2], span.start as usize)?,
                expect_u32(&args[3], span.start as usize)?,
            ];
            if skill_defs.contains(&0)
                || skill_defs[0] == skill_defs[1]
                || skill_defs[0] == skill_defs[2]
                || skill_defs[1] == skill_defs[2]
            {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "choose_add_missing_skill expects three distinct non-zero skill definitions"
                            .into(),
                });
            }
            Ok(StmtIr::EffectChooseAddMissingSkill {
                player: check_expr(&args[0], locals)?,
                skill_defs,
                span: *span,
            })
        }
        "add_first_missing_skill" => {
            if args.len() != 4 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "add_first_missing_skill(player, first, second, third)".into(),
                });
            }
            let skill_defs = [
                expect_u32(&args[1], span.start as usize)?,
                expect_u32(&args[2], span.start as usize)?,
                expect_u32(&args[3], span.start as usize)?,
            ];
            if skill_defs.contains(&0)
                || skill_defs[0] == skill_defs[1]
                || skill_defs[0] == skill_defs[2]
                || skill_defs[1] == skill_defs[2]
            {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "add_first_missing_skill expects three distinct non-zero skill definitions"
                            .into(),
                });
            }
            Ok(StmtIr::EffectAddFirstMissingSkill {
                player: check_expr(&args[0], locals)?,
                skill_defs,
                span: *span,
            })
        }
        "resolve_dccunsi" => {
            if args.len() != 5 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dccunsi(player, target, guixiu_def, yongjue_def, stage)"
                        .into(),
                });
            }
            let guixiu_def = expect_u32(&args[2], span.start as usize)?;
            let yongjue_def = expect_u32(&args[3], span.start as usize)?;
            let stage = expect_u32(&args[4], span.start as usize)?;
            if guixiu_def == 0 || yongjue_def == 0 || guixiu_def == yongjue_def {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dccunsi expects two distinct non-zero skill definitions"
                        .into(),
                });
            }
            if stage != 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "resolve_dccunsi public stage must be 0".into(),
                });
            }
            Ok(StmtIr::EffectResolveDccunsi {
                player: check_expr(&args[0], locals)?,
                target: check_expr(&args[1], locals)?,
                guixiu_def,
                yongjue_def,
                stage: stage as u8,
                span: *span,
            })
        }
        "record_current_sha_all_targets" => {
            if args.len() != 2 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "record_current_sha_all_targets(player, card)".into(),
                });
            }
            let card = check_expr(&args[1], locals)?;
            if !is_card_operand(&card) {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message:
                        "record_current_sha_all_targets second argument must be a card operand"
                            .into(),
                });
            }
            Ok(StmtIr::EffectRecordCurrentShaAllTargets {
                player: check_expr(&args[0], locals)?,
                card,
                span: *span,
            })
        }
        other => Err(CompileError::Type {
            pos: span.start as usize,
            message: format!("unknown effect {other}"),
        }),
    }
}

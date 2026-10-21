use super::*;

pub(super) fn check_expr(e: &ExprAst, locals: &[String]) -> Result<ExprIr, CompileError> {
    match e {
        ExprAst::Int(n) => Ok(ExprIr::Int(*n)),
        ExprAst::U32(_) => Err(CompileError::Type {
            pos: 0,
            message: "u32 literal is only valid for an identifier argument".into(),
        }),
        ExprAst::Ident(name) => match name.as_str() {
            "self" => Ok(ExprIr::SelfPlayer),
            "self_skill" => Ok(ExprIr::SelfSkill),
            "event_player" => Ok(ExprIr::EventPlayer),
            "event_source" => Ok(ExprIr::EventSource),
            "event_card" => Ok(ExprIr::EventCard),
            "event_amount" => Ok(ExprIr::EventAmount),
            "event_target_count" => Ok(ExprIr::EventTargetCount),
            "event_card_count" => Ok(ExprIr::EventCardCount),
            "turn_damaged_player_count" => Ok(ExprIr::TurnDamagedPlayerCount),
            other => {
                if locals.iter().any(|l| l == other) {
                    Ok(ExprIr::Local(other.into()))
                } else {
                    Err(CompileError::Type {
                        pos: 0,
                        message: format!("unknown name {other}"),
                    })
                }
            }
        },
        ExprAst::Call { name, args } => match name.as_str() {
            "distance"
            | "attack_range"
            | "hand_limit"
            | "hp"
            | "hand_count"
            | "empty_equip_slots"
            | "max_hp"
            | "relation"
            | "card_usable"
            | "target_legal"
            | "card_attribute"
            | "skill_uses_remaining"
            | "zone"
            | "card_zone"
            | "equipped_count"
            | "mark_count"
            | "skill_pile_count"
            | "alive_count"
            | "alive_group_count"
            | "alive_wounded_or_guiming_wu_count"
            | "unique_max_current_hp"
            | "unique_max_current_hp_at"
            | "lifetime_damage_dealt"
            | "public_card_kind_count"
            | "field_equipment_kind_count"
            | "hand_equip_suit_count"
            | "hand_suit_count"
            | "hand_color_count"
            | "skill_content_flag"
            | "round_number_capped"
            | "card_action_count"
            | "cards_drawn_this_turn"
            | "damage_dealt_this_turn"
            | "has_skill"
            | "card_name_length"
            | "in_play_phase"
            | "card_has_tag"
            | "storage_get"
            | "storage_has"
            | "cards_used_this_round"
            | "current_phase_player" => {
                // Per-query arity + which arg positions are card-typed (0-based).
                let (arity, card_positions): (usize, &[usize]) = match name.as_str() {
                    "alive_count"
                    | "alive_group_count"
                    | "current_phase_player"
                    | "in_play_phase" => (0, &[]),
                    "equipped_count" => (1, &[]),
                    "unique_max_current_hp_at" => (2, &[]),
                    "hand_equip_suit_count" | "hand_suit_count" | "hand_color_count" => (2, &[]),
                    "skill_content_flag" => (3, &[]),
                    "distance" | "relation" | "mark_count" | "skill_pile_count" => (2, &[]),
                    "has_skill" => (2, &[]),
                    "card_name_length" => (2, &[1]),
                    "card_has_tag" => (2, &[0]), // (card, skill_instance)
                    "storage_get" | "storage_has" => (2, &[]), // (skill_instance, key)
                    "cards_used_this_round" => (1, &[]), // (player)
                    "card_usable" => (2, &[1]),  // (player, card)
                    "target_legal" => (3, &[1]), // (user, card, target)
                    "card_attribute" => (2, &[0]), // (card, attr)
                    "skill_uses_remaining" => (1, &[]),
                    "zone" => (2, &[]),
                    "card_zone" => (1, &[0]),
                    "round_number_capped" => (1, &[]),
                    _ => (1, &[]),
                };
                if args.len() != arity {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: format!("{name} expects {arity} argument(s)"),
                    });
                }
                let mut a = Vec::new();
                for (i, x) in args.iter().enumerate() {
                    let ir = if matches!(name.as_str(), "hand_equip_suit_count" | "hand_suit_count")
                        && i == 1
                    {
                        let ExprAst::Ident(suit) = x else {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: format!("{name} expects a suit name"),
                            });
                        };
                        ExprIr::Int(match suit.as_str() {
                            "spade" => 1,
                            "heart" => 2,
                            "club" => 3,
                            "diamond" => 4,
                            _ => {
                                return Err(CompileError::Type {
                                    pos: 0,
                                    message: format!("unknown suit {suit}"),
                                });
                            }
                        })
                    } else if name == "hand_color_count" && i == 1 {
                        let ExprAst::Ident(color) = x else {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: "hand_color_count expects a color name".into(),
                            });
                        };
                        ExprIr::Int(match color.as_str() {
                            "red" => 1,
                            "black" => 2,
                            _ => {
                                return Err(CompileError::Type {
                                    pos: 0,
                                    message: format!("unknown color {color}"),
                                });
                            }
                        })
                    } else if matches!(
                        name.as_str(),
                        "public_card_kind_count" | "field_equipment_kind_count"
                    ) {
                        let ExprAst::Ident(kind) = x else {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: format!("{name} expects a card-kind name"),
                            });
                        };
                        ExprIr::Int(card_kind_value(kind).ok_or_else(|| CompileError::Type {
                            pos: 0,
                            message: format!("unknown card kind {kind}"),
                        })?)
                    } else if name == "mark_count" && i == 1 {
                        let mark = expect_u32(x, 0)?;
                        i32::try_from(mark).map_or(ExprIr::U32(mark), ExprIr::Int)
                    } else if name == "skill_pile_count" && i == 1 {
                        let skill_def = expect_u32(x, 0)?;
                        i32::try_from(skill_def).map_or(ExprIr::U32(skill_def), ExprIr::Int)
                    } else if name == "skill_content_flag" && i == 1 {
                        let skill_def = expect_u32(x, 0)?;
                        i32::try_from(skill_def).map_or(ExprIr::U32(skill_def), ExprIr::Int)
                    } else if name == "skill_content_flag" && i == 2 {
                        let bit = literal_i32(x).ok_or_else(|| CompileError::Type {
                            pos: 0,
                            message: "skill_content_flag bit must be a literal in 0..63".into(),
                        })?;
                        if !(0..64).contains(&bit) {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: "skill_content_flag bit must be in 0..63".into(),
                            });
                        }
                        ExprIr::Int(bit)
                    } else {
                        check_expr(x, locals)?
                    };
                    if card_positions.contains(&i) && !is_card_operand(&ir) {
                        return Err(CompileError::Type {
                            pos: 0,
                            message: format!(
                                "{name} argument {i} must be a card operand (e.g. event_card)"
                            ),
                        });
                    }
                    a.push(ir);
                }
                Ok(ExprIr::Query {
                    name: name.clone(),
                    args: a,
                })
            }
            "choose" => {
                if args.len() < 2 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "choose expects at least 2 option tokens".into(),
                    });
                }
                if args.len() > 255 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "choose supports at most 255 options".into(),
                    });
                }
                let mut opts = Vec::new();
                for x in args {
                    opts.push(check_expr(x, locals)?);
                }
                Ok(ExprIr::Choose { options: opts })
            }
            "choose_range" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "choose_range expects inclusive min and max bounds".into(),
                    });
                }
                Ok(ExprIr::ChooseRange {
                    min: Box::new(check_expr(&args[0], locals)?),
                    max: Box::new(check_expr(&args[1], locals)?),
                })
            }
            "min" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "min expects exactly two arguments".into(),
                    });
                }
                Ok(ExprIr::Min {
                    left: Box::new(check_expr(&args[0], locals)?),
                    right: Box::new(check_expr(&args[1], locals)?),
                })
            }
            other => Err(CompileError::Type {
                pos: 0,
                message: format!("unknown function {other}"),
            }),
        },
        ExprAst::BinOp { op, left, right } => Ok(ExprIr::BinOp {
            op: op.clone(),
            left: Box::new(check_expr(left, locals)?),
            right: Box::new(check_expr(right, locals)?),
        }),
    }
}

/// Whether an expression is a card-typed operand (currently `event_card` or a
/// local, since locals are untyped for now). Rejects players / integers.
pub(super) fn is_card_operand(e: &ExprIr) -> bool {
    matches!(e, ExprIr::EventCard | ExprIr::Local(_))
}

pub(super) fn expect_int(e: &ExprAst, pos: usize) -> Result<i32, CompileError> {
    match e {
        ExprAst::Int(n) => Ok(*n),
        _ => Err(CompileError::Type {
            pos,
            message: "expected integer literal".into(),
        }),
    }
}

pub(super) fn bounded_u8(arg: &ExprAst, name: &str) -> Result<u8, CompileError> {
    u8::try_from(expect_int(arg, 0).map_err(|_| CompileError::Type {
        pos: 0,
        message: format!("{name} must be an integer literal"),
    })?)
    .map_err(|_| CompileError::Type {
        pos: 0,
        message: format!("{name} must be 0..255"),
    })
}

pub(super) fn literal_i32(e: &ExprAst) -> Option<i32> {
    match e {
        ExprAst::Int(n) => Some(*n),
        _ => None,
    }
}

pub(super) fn expect_draw_target_scope(
    expression: &ExprAst,
    pos: usize,
) -> Result<DrawTargetScopeIr, CompileError> {
    match expression {
        ExprAst::Ident(scope) if scope == "other" => Ok(DrawTargetScopeIr::Other),
        ExprAst::Ident(scope) if scope == "male_other" => Ok(DrawTargetScopeIr::MaleOther),
        ExprAst::Ident(scope) if scope == "female_other" => Ok(DrawTargetScopeIr::FemaleOther),
        ExprAst::Ident(scope) if scope == "any" => Ok(DrawTargetScopeIr::Any),
        ExprAst::Ident(scope) if scope == "min_hand" => Ok(DrawTargetScopeIr::MinHand),
        ExprAst::Ident(scope) if scope == "same_hp" => Ok(DrawTargetScopeIr::SameHp),
        ExprAst::Ident(scope) if scope == "hand_empty" => Ok(DrawTargetScopeIr::HandEmpty),
        ExprAst::Ident(scope) if scope == "wounded" => Ok(DrawTargetScopeIr::Wounded),
        ExprAst::Ident(scope) if scope == "hand_lte_chooser" => {
            Ok(DrawTargetScopeIr::HandLteChooser)
        }
        _ => Err(CompileError::Type {
            pos,
            message:
                "target scope must be other, male_other, female_other, any, min_hand, same_hp, hand_empty, wounded, or hand_lte_chooser"
                    .into(),
        }),
    }
}

pub(super) fn expect_u32(e: &ExprAst, pos: usize) -> Result<u32, CompileError> {
    match e {
        ExprAst::Int(n) if *n >= 0 => Ok(*n as u32),
        ExprAst::U32(n) => Ok(*n),
        _ => Err(CompileError::Type {
            pos,
            message: "expected u32 literal".into(),
        }),
    }
}

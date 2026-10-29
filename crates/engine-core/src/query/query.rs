//! Query operations for `alive_wounded_count` and related helpers.

use super::*;

/// Run a typed query (read-only).
pub fn query(state: &GameState, req: &QueryRequest, ctx: &mut QueryContext) -> QueryResponse {
    query_explain(state, req, ctx).response
}

/// Run query with full explanation.
pub fn query_explain(
    state: &GameState,
    req: &QueryRequest,
    ctx: &mut QueryContext,
) -> QueryExplanation {
    if ctx.depth >= ctx.max_depth {
        return QueryExplanation {
            response: QueryResponse::Error {
                code: QueryErrorCode::DepthExceeded,
            },
            base: None,
            sources: vec![],
        };
    }

    // 依赖版本 = 提交版本 + 结算内单调哨兵（ADR 0703 第②部分）：
    // 结算内推进伤害/恢复/选择序号或消耗 RNG 都会使缓存键失效，
    // 防止「revision 未变但状态已变」的脏命中。
    let key = CacheKey::from_request(req, state.query_dep_version());
    if let Some(cache) = ctx.cache.as_mut() {
        if let Some(hit) = cache.get(&key) {
            return QueryExplanation {
                response: hit,
                base: None,
                sources: vec![],
            };
        }
    }

    let cycle_key = key.raw.clone();
    if !ctx.in_flight.insert(cycle_key.clone()) {
        return QueryExplanation {
            response: QueryResponse::Error {
                code: QueryErrorCode::Cycle,
            },
            base: None,
            sources: vec![],
        };
    }

    ctx.depth += 1;
    let explained = match req {
        QueryRequest::Distance { from, to } => distance_explain(state, *from, *to),
        QueryRequest::AttackRange { player } => attack_range_explain(state, *player),
        QueryRequest::HandLimit { player } => hand_limit_explain(state, *player),
        QueryRequest::Relation { a, b } => QueryExplanation {
            response: relation(state, *a, *b),
            base: None,
            sources: vec![],
        },
        QueryRequest::CardUsable { player, card } => card_usable_explain(state, *player, *card),
        QueryRequest::TargetLegal { user, card, target } => {
            target_legal_explain(state, *user, *card, *target, ctx)
        }
        QueryRequest::SkillUsesRemaining { skill } => skill_uses_explain(state, *skill),
        QueryRequest::CardAttribute { card, attr } => card_attr_explain(state, *card, *attr),
        QueryRequest::Hp { player } => player_int_explain(state, *player, |p| p.hp),
        QueryRequest::MaxHp { player } => player_int_explain(state, *player, |p| p.max_hp as i32),
        QueryRequest::HandCount { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                QueryResponse::Int(state.hand_count(*player) as i32)
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::EmptyEquipSlots { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                let occupied = state
                    .zones
                    .find(ZoneKind::Equip, *player)
                    .and_then(|zone| state.zones.get(zone))
                    .map_or(0, |zone| zone.cards.len());
                QueryResponse::Int(5_i32.saturating_sub(occupied as i32))
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::EquippedCount { player } => QueryExplanation {
            response: state
                .zones
                .find(ZoneKind::Equip, *player)
                .and_then(|zone| state.zones.get(zone))
                .map(|zone| QueryResponse::Int(zone.cards.len() as i32))
                .unwrap_or(QueryResponse::Int(0)),
            base: None,
            sources: vec![],
        },
        QueryRequest::Zone { player, kind } => zone_explain(state, *player, *kind),
        QueryRequest::CardZone { card } => QueryExplanation {
            response: state
                .cards
                .get(*card)
                .map(|card| QueryResponse::Int(card.zone.raw() as i32))
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::MarkCount { player, mark } => QueryExplanation {
            response: match state.players.get(*player) {
                Some(player) => QueryResponse::Int(
                    player
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .map(|(_, count)| *count)
                        .unwrap_or(0),
                ),
                None => QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                },
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::SkillPileCount { player, skill_def } => QueryExplanation {
            response: match state.players.get(*player) {
                Some(_) => QueryResponse::SkillPileCount {
                    count: state.skill_pile_cards(*player, *skill_def).len() as i32,
                },
                None => QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                },
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::AliveCount => QueryExplanation {
            response: QueryResponse::Int(
                state
                    .players
                    .iter_live()
                    .filter(|player| player.alive)
                    .count()
                    .min(i32::MAX as usize) as i32,
            ),
            base: None,
            sources: vec![],
        },
        QueryRequest::CardActionCount { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                QueryResponse::Int(state.turn.flag(
                    crate::state::turn_flag::card_action_count_by_player(*player),
                ))
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::CardsDrawnThisTurn { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                QueryResponse::Int(state.turn.card_gain_count(*player) as i32)
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::DamageDealtThisTurn { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                QueryResponse::Int(state.turn.damage_dealt_count(*player) as i32)
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::HasSkill { player, skill_def } => QueryExplanation {
            response: QueryResponse::Bool(state.players.get(*player).is_some_and(|_| {
                state
                    .skills
                    .sorted_live()
                    .into_iter()
                    .any(|skill| skill.owner == *player && skill.def == *skill_def)
            })),
            base: None,
            sources: vec![],
        },
        QueryRequest::CardNameLength { player, card } => QueryExplanation {
            response: QueryResponse::Int(
                crate::query::card_name_length(state, *player, *card)
                    .map(|n| n as i32)
                    .unwrap_or(0),
            ),
            base: None,
            sources: vec![],
        },
        QueryRequest::CardHasTag { card, tag } => {
            // The tag the VM pushes is a skill instance id; resolve to its definition id to
            // match the def-id-as-tag convention used by tag_cards / hardcoded card tags.
            let resolved_tag = state
                .skills
                .get(engine_api::SkillId::new(*tag))
                .map(|inst| inst.def.raw())
                .unwrap_or(*tag);
            QueryExplanation {
                response: QueryResponse::Bool(state.card_has_content_tag(*card, resolved_tag)),
                base: None,
                sources: vec![],
            }
        }
        QueryRequest::StorageGet { skill, key } => QueryExplanation {
            response: QueryResponse::Int(state.skills.storage_get(*skill, *key) as i32),
            base: None,
            sources: vec![],
        },
        QueryRequest::StorageHas { skill, key } => QueryExplanation {
            response: QueryResponse::Bool(state.skills.storage_has(*skill, *key)),
            base: None,
            sources: vec![],
        },
        QueryRequest::CardsUsedThisRound { player } => QueryExplanation {
            response: QueryResponse::Int(state.turn.round_card_use_count(*player) as i32),
            base: None,
            sources: vec![],
        },
        QueryRequest::InPlayPhase => QueryExplanation {
            response: QueryResponse::Bool(matches!(state.turn.phase, crate::state::Phase::Play)),
            base: None,
            sources: vec![],
        },
        QueryRequest::AliveGroupCount => QueryExplanation {
            response: QueryResponse::Int(alive_group_count(state)),
            base: None,
            sources: vec![],
        },
        QueryRequest::UniqueMaxCurrentHp { player } => QueryExplanation {
            response: unique_max_current_hp(state, *player)
                .map(QueryResponse::Bool)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::UniqueMaxCurrentHpAt { player, value } => QueryExplanation {
            response: unique_max_current_hp_at(state, *player, *value)
                .map(QueryResponse::Bool)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::LifetimeDamageDealt { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                QueryResponse::Int(
                    state
                        .damage_dealt_totals
                        .get(player)
                        .copied()
                        .unwrap_or(0)
                        .min(i32::MAX as u32) as i32,
                )
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::PublicCardKindCount { kind } => QueryExplanation {
            response: public_card_kind_count(state, *kind)
                .map(QueryResponse::Int)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::RoundNumberCapped { cap } => QueryExplanation {
            response: QueryResponse::Int(state.turn.round.min(*cap).min(i32::MAX as u32) as i32),
            base: None,
            sources: vec![],
        },
        QueryRequest::FieldEquipmentKindCount { kind } => QueryExplanation {
            response: field_equipment_kind_count(state, *kind)
                .map(QueryResponse::Int)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::AliveWoundedOrGuimingWuCount { player } => QueryExplanation {
            response: if state.players.get(*player).is_some() {
                QueryResponse::Int(alive_wounded_or_guiming_wu_count(state, *player))
            } else {
                QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }
            },
            base: None,
            sources: vec![],
        },
        QueryRequest::HandEquipSuitCount { player, suit } => QueryExplanation {
            response: hand_equip_suit_count(state, *player, *suit)
                .map(QueryResponse::Int)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::CurrentPhasePlayer => QueryExplanation {
            response: QueryResponse::Int(state.turn.current.raw() as i32),
            base: None,
            sources: vec![],
        },
        QueryRequest::HandSuitCount { player, suit } => QueryExplanation {
            response: hand_suit_count(state, *player, *suit)
                .map(QueryResponse::Int)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::HandColorCount { player, color } => QueryExplanation {
            response: hand_color_count(state, *player, *color)
                .map(QueryResponse::Int)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
        QueryRequest::SkillContentFlag {
            player,
            skill_def,
            bit,
        } => QueryExplanation {
            response: skill_content_flag(state, *player, *skill_def, *bit)
                .map(QueryResponse::Int)
                .unwrap_or(QueryResponse::Error {
                    code: QueryErrorCode::InvalidId,
                }),
            base: None,
            sources: vec![],
        },
    };
    ctx.depth -= 1;
    ctx.in_flight.remove(&cycle_key);

    if let Some(cache) = ctx.cache.as_mut() {
        cache.insert(key, explained.response.clone());
    }
    explained
}

pub(crate) fn zone_explain(state: &GameState, player: PlayerId, kind: u8) -> QueryExplanation {
    let kind = match kind {
        1 => ZoneKind::DrawPile,
        2 => ZoneKind::DiscardPile,
        3 => ZoneKind::Hand,
        4 => ZoneKind::Equip,
        5 => ZoneKind::Judge,
        6 => ZoneKind::Processing,
        7 => ZoneKind::Removed,
        _ => return err(QueryErrorCode::InvalidId),
    };
    QueryExplanation {
        response: state
            .zones
            .find(kind, player)
            .map(|zone| QueryResponse::Int(zone.raw() as i32))
            .unwrap_or(QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }),
        base: None,
        sources: vec![],
    }
}

pub(crate) fn player_int_explain(
    state: &GameState,
    player: PlayerId,
    f: impl FnOnce(&crate::state::Player) -> i32,
) -> QueryExplanation {
    match state.players.get(player) {
        Some(p) => QueryExplanation {
            response: QueryResponse::Int(f(p)),
            base: None,
            sources: vec![],
        },
        None => QueryExplanation {
            response: QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            },
            base: None,
            sources: vec![],
        },
    }
}

pub(crate) fn distance_explain(
    state: &GameState,
    from: PlayerId,
    to: PlayerId,
) -> QueryExplanation {
    let base = match base_distance(state, from, to) {
        Ok(d) => d,
        Err(code) => {
            return QueryExplanation {
                response: QueryResponse::Error { code },
                base: None,
                sources: vec![],
            };
        }
    };
    if from == to {
        return QueryExplanation {
            response: QueryResponse::Int(0),
            base: Some(0),
            sources: vec![],
        };
    }
    // 雄乱（张绣）：对限定目标无距离限制。
    if state
        .turn
        .flag(crate::state::turn_flag::xiongluan_target_by_player(from))
        .max(0) as u32
        == to.raw()
    {
        return QueryExplanation {
            response: QueryResponse::Int(0),
            base: Some(0),
            sources: vec![],
        };
    }
    let (mut value, mut sources, final_override) = apply_distance_modifiers(state, base, from, to);
    // Equipment horses
    if has_equip_kind(state, from, CardKind::HorseOff) {
        value -= 1;
        sources.push(ModifierSource {
            source_id: 0,
            label_key: "equip.horse_off".into(),
            value: -1,
        });
    }
    if has_equip_kind(state, to, CardKind::HorseDef) {
        value += 1;
        sources.push(ModifierSource {
            source_id: 0,
            label_key: "equip.horse_def".into(),
            value: 1,
        });
    }
    for (owner, adjustment, label) in [
        (from, true, "equip.package_from"),
        (to, false, "equip.package_to"),
    ] {
        let delta = state
            .zones
            .find(ZoneKind::Equip, owner)
            .and_then(|zone| state.zones.get(zone))
            .into_iter()
            .flat_map(|zone| zone.cards.iter())
            .filter_map(|card| state.cards.get(*card))
            .filter_map(|card| state.content.card_metadata(card.def))
            .map(|card| {
                i32::from(if adjustment {
                    card.distance_from
                } else {
                    card.distance_to
                })
            })
            .sum::<i32>();
        if delta != 0 {
            value += delta;
            sources.push(ModifierSource {
                source_id: 0,
                label_key: label.into(),
                value: delta,
            });
        }
    }
    if let Some(overridden) = final_override {
        value = overridden;
    }
    if value < 1 {
        value = 1;
    }
    QueryExplanation {
        response: QueryResponse::Int(value),
        base: Some(base),
        sources,
    }
}

pub(crate) fn base_distance(
    state: &GameState,
    from: PlayerId,
    to: PlayerId,
) -> Result<i32, QueryErrorCode> {
    let Some(a) = state.players.get(from) else {
        return Err(QueryErrorCode::InvalidId);
    };
    let Some(b) = state.players.get(to) else {
        return Err(QueryErrorCode::InvalidId);
    };
    if !a.alive || !b.alive {
        return Ok(99);
    }
    let alive = state.players.alive_by_seat();
    let n = alive.len() as i32;
    if n == 0 {
        return Ok(0);
    }
    let ia = alive.iter().position(|p| *p == from).map(|i| i as i32);
    let ib = alive.iter().position(|p| *p == to).map(|i| i as i32);
    match (ia, ib) {
        (Some(ia), Some(ib)) => {
            let d1 = (ia - ib).abs();
            let d2 = n - d1;
            Ok(d1.min(d2))
        }
        _ => Ok(99),
    }
}

pub(crate) fn attack_range_explain(state: &GameState, player: PlayerId) -> QueryExplanation {
    if state.players.get(player).is_none() {
        return QueryExplanation {
            response: QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            },
            base: None,
            sources: vec![],
        };
    }
    // Base 1, or equipped weapon range
    let mut base = 1i32;
    let mut sources = Vec::new();
    if let Some(range) = equipped_weapon_range(state, player) {
        base = range as i32;
        sources.push(ModifierSource {
            source_id: 0,
            label_key: "equip.weapon".into(),
            value: range as i32,
        });
    }
    let mods = state.modifiers.for_attack_range(player, |owner| {
        state.players.get(owner).is_some_and(|player| player.alive)
    });
    let (value, mut skill_sources, vetoed) =
        apply_int_modifiers_for_state_with_context(state, base, &mods, None, Some(player), None);
    sources.append(&mut skill_sources);
    if vetoed {
        return QueryExplanation {
            response: QueryResponse::Int(0),
            base: Some(base),
            sources,
        };
    }
    QueryExplanation {
        response: QueryResponse::Int(value.max(0)),
        base: Some(base),
        sources,
    }
}

pub(crate) fn has_equip_kind(state: &GameState, player: PlayerId, kind: CardKind) -> bool {
    let Some(eq) = state.zones.find(ZoneKind::Equip, player) else {
        return false;
    };
    let Some(z) = state.zones.get(eq) else {
        return false;
    };
    z.cards.iter().any(|cid| {
        state
            .cards
            .get(*cid)
            .map(|c| c.kind == kind)
            .unwrap_or(false)
    })
}

/// Whether a living player has an enabled, empty armor slot and an active virtual-Bagua grant.
pub fn virtual_bagua_available_for(state: &GameState, player: PlayerId) -> bool {
    let Some(owner) = state.players.get(player) else {
        return false;
    };
    if !owner.alive
        || !owner.equip_slot_enabled(EquipSlot::Armor)
        || has_equip_kind(state, player, CardKind::Armor)
    {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::VirtualBaguaIfArmorSlotEmpty);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

pub(crate) fn equipped_weapon_range(state: &GameState, player: PlayerId) -> Option<u8> {
    let eq = state.zones.find(ZoneKind::Equip, player)?;
    let z = state.zones.get(eq)?;
    z.cards.iter().find_map(|cid| {
        let c = state.cards.get(*cid)?;
        if c.kind == CardKind::Weapon && c.weapon_range > 0 {
            Some(c.weapon_range)
        } else {
            None
        }
    })
}

pub(crate) fn hand_limit_explain(state: &GameState, player: PlayerId) -> QueryExplanation {
    let Some(p) = state.players.get(player) else {
        return QueryExplanation {
            response: QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            },
            base: None,
            sources: vec![],
        };
    };
    let max_hp_base_mods = state
        .modifiers
        .for_target(ModifierTarget::HandLimitBaseMaxHp);
    let (use_max_hp, mut sources, max_hp_base_vetoed) =
        apply_int_modifiers_for_state(state, 0, &max_hp_base_mods, Some(player));
    let raw_base = if use_max_hp != 0 {
        p.max_hp.min(i32::MAX as u32) as i32
    } else {
        p.hp.max(0)
    };
    let base_mods = state.modifiers.for_target(ModifierTarget::HandLimitBase);
    let (base, base_sources, base_vetoed) =
        apply_int_modifiers_for_state(state, raw_base, &base_mods, Some(player));
    sources.extend(base_sources);
    let mods = state.modifiers.for_hand_limit(player, |owner| {
        state.players.get(owner).is_some_and(|player| player.alive)
    });
    let (value, extra_sources, vetoed) =
        apply_int_modifiers_for_state_with_context(state, base, &mods, None, Some(player), None);
    sources.extend(extra_sources);
    QueryExplanation {
        response: QueryResponse::Int(if max_hp_base_vetoed || base_vetoed || vetoed {
            0
        } else {
            let bonus = state
                .hand_limit_bonus
                .get(&player.raw())
                .copied()
                .unwrap_or(0);
            (value.max(0) + bonus).max(0)
        }),
        base: Some(raw_base),
        sources,
    }
}

/// Draw-phase card count (base 2 + `draw_count` modifiers, e.g. 英姿).
pub fn draw_count_for(state: &GameState, player: PlayerId) -> u32 {
    use crate::state::DEFAULT_DRAW_COUNT;
    let intent = crate::intent::resolve_draw_intent(
        state,
        crate::intent::DrawIntent {
            player,
            count: DEFAULT_DRAW_COUNT,
            cancelled: false,
        },
    );
    if intent.cancelled {
        0
    } else {
        intent.count
    }
}

/// Damage dealt by `source`'s 杀/决斗, including ordinary and active
/// draw-replacement modifiers. Clamped to at least 1.
pub fn sha_damage_for(state: &GameState, source: PlayerId, base_amount: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ShaDamage);
    sha_damage_from_modifiers(state, source, base_amount, &mods)
}

/// Damage dealt by one concrete Sha, including generic and card-selector modifiers.
pub fn sha_damage_for_card(
    state: &GameState,
    source: PlayerId,
    card: CardId,
    base_amount: u32,
) -> u32 {
    let Some(card) = state.cards.get(card) else {
        return sha_damage_for(state, source, base_amount);
    };
    let kind = effective_card_kind(state, source, card);
    let suit = effective_card_suit(state, source, card);
    let mods = state.modifiers.for_sha_damage(kind, suit);
    sha_damage_from_modifiers(state, source, base_amount, &mods)
}

pub(crate) fn sha_damage_from_modifiers(
    state: &GameState,
    source: PlayerId,
    base_amount: u32,
    mods: &[&modifier::ActiveModifier],
) -> u32 {
    use crate::state::turn_flag;
    let (mut value, _, _) =
        apply_int_modifiers_for_state(state, base_amount as i32, mods, Some(source));
    if source == state.turn.current && state.turn.flag(turn_flag::DRAW_REPLACED) != 0 {
        let draw_replacement = state
            .modifiers
            .for_target(ModifierTarget::DrawReplacementDamage);
        value = apply_int_modifiers(value, &draw_replacement, Some(source)).0;
    }
    value.max(1) as u32
}

/// Proactive 杀 uses allowed in the current turn (base 1).
pub fn sha_use_limit_for(state: &GameState, player: PlayerId) -> u32 {
    sha_use_limit(state, player, card_use_unlimited_for(state, player))
}

/// Proactive 杀 uses allowed for one concrete source card.
pub fn sha_use_limit_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> u32 {
    sha_use_limit(
        state,
        player,
        card_use_unlimited_for_card(state, player, card),
    )
}

pub(crate) fn sha_use_limit(state: &GameState, player: PlayerId, unlimited: bool) -> u32 {
    let cancel = state.modifiers.for_target(ModifierTarget::ShaUseCancel);
    let (cancel, _, cancel_vetoed) = apply_int_modifiers(0, &cancel, Some(player));
    if cancel_vetoed || cancel != 0 {
        return 0;
    }
    // `card_use_cancel(sha)`（并行会话 DSL 形态）同样取消【杀】的使用额度。
    let selector_cancel = state
        .modifiers
        .for_target(ModifierTarget::CardUseCancelFor(CardSelector::Sha));
    let (selector_cancel, _, selector_vetoed) =
        apply_int_modifiers(0, &selector_cancel, Some(player));
    if selector_vetoed || selector_cancel != 0 {
        return 0;
    }
    if unlimited {
        return u32::MAX;
    }
    let mods = state.modifiers.for_sha_use_limit(player, |owner| {
        state
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    });
    let (value, _, vetoed) =
        apply_int_modifiers_for_state_with_context(state, 1, &mods, None, Some(player), None);
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Whether cards used by `player` ignore ordinary per-turn use limits.
pub fn card_use_unlimited_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::CardUseUnlimited);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card ignores its ordinary per-turn use limit.
pub fn card_use_unlimited_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if sha_use_does_not_count_for_card(state, player, card) {
        return true;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if first_hand_card_suit_use_unlimited(state, player, card) {
        return true;
    }
    if chenglve_suit_unlimited_for(state, player, card) {
        return true;
    }
    // 雄乱（张绣）：本回合对限定目标使用牌无次数限制。
    if state
        .turn
        .flag(crate::state::turn_flag::xiongluan_target_by_player(player))
        != 0
    {
        return true;
    }
    let kind = effective_card_kind(state, player, card);
    let mods = state.modifiers.for_card_use_limit(
        card.kind,
        kind,
        effective_card_suit(state, player, card),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// 成略（许攸）：本回合弃置花色的实体牌无距离与次数限制（位 1..=4 对应黑桃/红桃/梅花/方块）。
pub fn chenglve_suit_unlimited_for(state: &GameState, player: PlayerId, card: &Card) -> bool {
    if player.is_none() {
        return false;
    }
    let suits = state
        .turn
        .flag(crate::state::turn_flag::CHENGLVE_UNLIMITED_SUITS)
        .max(0) as u32;
    if suits == 0 {
        return false;
    }
    let bit = match card.suit {
        engine_api::Suit::Spade => 1u32 << 1,
        engine_api::Suit::Heart => 1u32 << 2,
        engine_api::Suit::Club => 1u32 << 3,
        engine_api::Suit::Diamond => 1u32 << 4,
        _ => 0,
    };
    bit != 0 && suits & bit != 0
}

/// Whether a card ignores ordinary use limits after an explicit view-as conversion.
pub fn card_use_unlimited_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    logical_kind: CardKind,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if first_hand_card_suit_use_unlimited(state, player, card) {
        return true;
    }
    let mods = state.modifiers.for_card_use_limit(
        card.kind,
        logical_kind,
        effective_card_suit(state, player, card),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

pub(crate) fn first_hand_card_suit_use_unlimited(
    state: &GameState,
    player: PlayerId,
    card: &Card,
) -> bool {
    if state.turn.current != player
        || state
            .zones
            .get(card.zone)
            .is_none_or(|zone| zone.kind != ZoneKind::Hand || zone.owner != player)
    {
        return false;
    }
    let bit = match effective_card_suit(state, player, card) {
        Suit::Spade => 1,
        Suit::Heart => 1 << 1,
        Suit::Club => 1 << 2,
        Suit::Diamond => 1 << 3,
        Suit::None => return false,
    };
    state
        .turn
        .flag(crate::state::turn_flag::hand_card_suit_history_by_player(
            player,
        ))
        & bit
        == 0
        && {
            let mods = state
                .modifiers
                .for_target(ModifierTarget::FirstHandCardSuitUseUnlimited);
            let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
            vetoed || value != 0
        }
}

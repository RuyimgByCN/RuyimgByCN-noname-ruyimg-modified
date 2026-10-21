//! Query operations for `alive_wounded_count` and related helpers.

use super::*;

/// Number of living players whose current HP is below maximum HP.
pub fn alive_wounded_count(state: &GameState) -> usize {
    state
        .players
        .iter_live()
        .filter(|player| player.alive && player.hp < player.max_hp as i32)
        .count()
}

/// Number of distinct non-empty character groups among living players.
pub fn alive_group_count(state: &GameState) -> i32 {
    state
        .players
        .iter_live()
        .filter(|player| player.alive)
        .filter_map(|player| state.player_group(player.id))
        .collect::<BTreeSet<_>>()
        .len()
        .min(i32::MAX as usize) as i32
}

/// 好施 authorization covering one lender/borrower pair.
pub(crate) fn pothaoshi_grant(
    state: &GameState,
    lender: PlayerId,
    borrower: PlayerId,
) -> Option<engine_api::SkillId> {
    if lender == borrower
        || !state.players.get(lender).is_some_and(|player| player.alive)
        || !state
            .players
            .get(borrower)
            .is_some_and(|player| player.alive)
    {
        return None;
    }
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| {
            skill.owner == lender
                && state.skill_effectively_enabled(skill)
                && skill
                    .pothaoshi_grants()
                    .is_some_and(|grants| grants.iter().any(|(target, _)| *target == borrower))
        })
        .map(|skill| skill.id)
}

/// Lender and originating 好施 skill for a borrowed physical hand card.
pub(crate) fn borrowed_hand_source(
    state: &GameState,
    borrower: PlayerId,
    card: CardId,
) -> Option<(PlayerId, engine_api::SkillId)> {
    let card = state.cards.get(card)?;
    let zone = state.zones.get(card.zone)?;
    let lender = zone.owner;
    (zone.kind == ZoneKind::Hand && card.owner == lender)
        .then(|| pothaoshi_grant(state, lender, borrower).map(|skill| (lender, skill)))?
}

/// Legal unused 界邀名 actions for one current target.
pub(crate) fn xinyaoming_available_actions(
    state: &GameState,
    owner: PlayerId,
    target: PlayerId,
    used_mask: u8,
) -> Vec<u8> {
    if !state
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Vec::new();
    }
    let mut actions = Vec::new();
    if target != owner {
        if used_mask & (1 << (XINYAOMING_DRAW - 1)) == 0 {
            actions.push(XINYAOMING_DRAW);
        }
        if used_mask & (1 << (XINYAOMING_DISCARD_HAND - 1)) == 0 && state.hand_count(target) > 0 {
            actions.push(XINYAOMING_DISCARD_HAND);
        }
    }
    if used_mask & (1 << (XINYAOMING_DISCARD_DRAW - 1)) == 0 {
        actions.push(XINYAOMING_DISCARD_DRAW);
    }
    actions
}

/// Gainable lord skills found on unused loaded character definitions.
pub(crate) fn gainable_lord_skill_defs(
    state: &GameState,
    player: PlayerId,
) -> Vec<engine_api::SkillDefId> {
    let active_characters: BTreeSet<_> = state
        .players
        .iter_live()
        .map(|player| player.character_def)
        .collect();
    let owned: BTreeSet<_> = state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .map(|skill| skill.def)
        .collect();
    state
        .content
        .characters()
        .filter(|character| !active_characters.contains(&character.id))
        .flat_map(|character| character.skills.iter().copied())
        .map(engine_api::SkillDefId::new)
        .filter(|skill| {
            !owned.contains(skill)
                && state
                    .content
                    .skill_metadata(*skill)
                    .is_some_and(|metadata| metadata.lord_only && !metadata.non_gainable)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Conversion skill definitions currently owned by a living player.
pub fn conversion_skill_defs(state: &GameState, player: PlayerId) -> Vec<engine_api::SkillDefId> {
    if !state.players.get(player).is_some_and(|owner| owner.alive) {
        return Vec::new();
    }
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|owner| owner.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .map(|skill| skill.def)
        .filter(|skill| {
            state
                .content
                .skill_metadata(*skill)
                .is_some_and(|metadata| metadata.conversion)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Legal living destinations for relocating one equipment card.
///
/// The current owner and `forbidden_destination` are excluded. An occupied matching slot is
/// legal and will be replaced; only the destination's disabled-slot state blocks equipping.
pub fn equipment_move_destinations(
    state: &GameState,
    forbidden_destination: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some(card_state) = state.cards.get(card) else {
        return Vec::new();
    };
    let source = card_state.owner;
    let Some(slot) = card_state.kind.equip_slot() else {
        return Vec::new();
    };
    if !state.players.get(source).is_some_and(|player| player.alive)
        || state.zones.find(ZoneKind::Equip, source) != Some(card_state.zone)
    {
        return Vec::new();
    }
    let mut destinations: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| {
            player.alive
                && player.id != source
                && player.id != forbidden_destination
                && player.equip_slot_enabled(slot)
        })
        .map(|player| player.id)
        .collect();
    destinations.sort_by_key(|player| player.raw());
    destinations
}

/// Equipment cards that `chooser` can legally relocate while remaining forbidden as recipient.
pub fn movable_equipment_cards(state: &GameState, chooser: PlayerId) -> Vec<CardId> {
    if !state
        .players
        .get(chooser)
        .is_some_and(|player| player.alive)
    {
        return Vec::new();
    }
    let mut cards: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.alive)
        .filter_map(|player| state.zones.find(ZoneKind::Equip, player.id))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| !equipment_move_destinations(state, chooser, *card).is_empty())
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Equipment and delayed-trick cards another living player can move to `destination`.
///
/// Equipment requires an enabled, empty matching slot. Delayed tricks require that the
/// destination does not already contain the same kind.
pub fn movable_field_cards_to(state: &GameState, destination: PlayerId) -> Vec<CardId> {
    if !state
        .players
        .get(destination)
        .is_some_and(|player| player.alive)
    {
        return Vec::new();
    }
    let mut cards: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != destination)
        .flat_map(|player| [ZoneKind::Equip, ZoneKind::Judge].map(|kind| (player.id, kind)))
        .filter_map(|(owner, kind)| state.zones.find(kind, owner))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| field_card_move_destinations(state, *card).contains(&destination))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Living destinations to which one equipment or delayed-trick field card can move.
pub fn field_card_move_destinations(state: &GameState, card: CardId) -> Vec<PlayerId> {
    let Some(card_state) = state.cards.get(card) else {
        return Vec::new();
    };
    let Some(from) = state.zones.get(card_state.zone) else {
        return Vec::new();
    };
    if !state
        .players
        .get(from.owner)
        .is_some_and(|player| player.alive)
    {
        return Vec::new();
    }
    let mut destinations: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != from.owner)
        .filter(|destination| match from.kind {
            ZoneKind::Equip => card_state.kind.equip_slot().is_some_and(|slot| {
                destination.equip_slot_enabled(slot)
                    && state
                        .zones
                        .find(ZoneKind::Equip, destination.id)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().all(|existing| {
                                state
                                    .cards
                                    .get(*existing)
                                    .and_then(|card| card.kind.equip_slot())
                                    != Some(slot)
                            })
                        })
            }),
            ZoneKind::Judge => {
                card_state.kind.is_delayed_trick()
                    && state
                        .zones
                        .find(ZoneKind::Judge, destination.id)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().all(|existing| {
                                state.cards.get(*existing).map(|card| card.kind)
                                    != Some(card_state.kind)
                            })
                        })
            }
            _ => false,
        })
        .map(|player| player.id)
        .collect();
    destinations.sort_by_key(|player| player.raw());
    destinations
}

/// Equipment and delayed-trick cards in one player's field that can move elsewhere.
pub fn movable_field_cards_from(state: &GameState, source: PlayerId) -> Vec<CardId> {
    let mut cards: Vec<_> = [ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, source))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| !field_card_move_destinations(state, *card).is_empty())
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Whether every other living player is currently within `owner`'s attack range.
pub fn all_other_players_in_attack_range(state: &GameState, owner: PlayerId) -> bool {
    let range = match attack_range_explain(state, owner).response {
        rule_api::QueryResponse::Int(value) => value,
        _ => return false,
    };
    state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != owner)
        .all(
            |player| match distance_explain(state, owner, player.id).response {
                rule_api::QueryResponse::Int(value) => value <= range,
                _ => false,
            },
        )
}

/// Distinct known groups among other living players for which no member can currently target
/// `owner` with Sha by attack range.
pub fn other_group_count_outside_owner_attack_range(state: &GameState, owner: PlayerId) -> u32 {
    let mut groups: std::collections::BTreeSet<String> = state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != owner)
        .filter_map(|player| state.player_group(player.id))
        .filter(|group| *group != "unknown")
        .map(str::to_owned)
        .collect();
    for player in state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != owner)
    {
        if player_in_attack_range(state, player.id, owner) {
            if let Some(group) = state.player_group(player.id) {
                groups.remove(group);
            }
        }
    }
    groups.len().min(u32::MAX as usize) as u32
}

/// Equipment cards in `target`'s equipment zone that `player` can move into the matching slot.
/// An occupied slot is legal and will be replaced; a disabled slot is not.
pub fn movable_target_equipment_cards(
    state: &GameState,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    if player == target
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Vec::new();
    }
    let Some(equip) = state.zones.find(ZoneKind::Equip, target) else {
        return Vec::new();
    };
    let mut cards: Vec<_> = state
        .zones
        .get(equip)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                .is_some_and(|slot| {
                    state
                        .players
                        .get(player)
                        .is_some_and(|participant| participant.equip_slot_enabled(slot))
                })
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards
}

/// Whether a distinct living `target` is currently inside `player`'s attack range.
pub fn player_in_attack_range(state: &GameState, player: PlayerId, target: PlayerId) -> bool {
    if player == target
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let mut ctx = QueryContext::new();
    matches!(
        (
            query(
                state,
                &QueryRequest::Distance {
                    from: player,
                    to: target,
                },
                &mut ctx,
            ),
            query(
                state,
                &QueryRequest::AttackRange { player },
                &mut ctx,
            ),
        ),
        (QueryResponse::Int(distance), QueryResponse::Int(range)) if range >= 1 && distance <= range
    )
}

/// Whether `to` is within the requested effective distance from `from`.
pub(crate) fn player_within_distance(
    state: &GameState,
    from: PlayerId,
    to: PlayerId,
    max_distance: i32,
) -> bool {
    let mut ctx = QueryContext::new();
    matches!(
        query(state, &QueryRequest::Distance { from, to }, &mut ctx),
        QueryResponse::Int(distance) if distance <= max_distance
    )
}

/// Number of physical cards of `kind` in the shared draw and discard piles.
pub fn public_card_kind_count(state: &GameState, kind: u8) -> Option<i32> {
    let kind = CardKind::from_tag(kind)?;
    Some(
        [ZoneKind::DrawPile, ZoneKind::DiscardPile]
            .into_iter()
            .filter_map(|zone_kind| state.zones.find(zone_kind, PlayerId::NONE))
            .filter_map(|zone| state.zones.get(zone))
            .flat_map(|zone| zone.cards.iter())
            .filter(|card| {
                state
                    .cards
                    .get(**card)
                    .is_some_and(|card| card.kind == kind)
            })
            .count()
            .min(i32::MAX as usize) as i32,
    )
}

/// Number of physical cards of `kind` in living players' equipment zones.
pub fn field_equipment_kind_count(state: &GameState, kind: u8) -> Option<i32> {
    let kind = CardKind::from_tag(kind)?;
    Some(
        state
            .players
            .alive_by_seat()
            .into_iter()
            .filter_map(|player| state.zones.find(ZoneKind::Equip, player))
            .filter_map(|zone| state.zones.get(zone))
            .flat_map(|zone| zone.cards.iter())
            .filter(|card| {
                state
                    .cards
                    .get(**card)
                    .is_some_and(|card| card.kind == kind)
            })
            .count()
            .min(i32::MAX as usize) as i32,
    )
}

/// Number of cards in `player`'s hand/equipment region with the requested effective suit.
pub fn hand_equip_suit_count(state: &GameState, player: PlayerId, suit: u8) -> Option<i32> {
    let suit = match suit {
        1 => engine_api::Suit::Spade,
        2 => engine_api::Suit::Heart,
        3 => engine_api::Suit::Club,
        4 => engine_api::Suit::Diamond,
        _ => return None,
    };
    state.players.get(player)?;
    Some(
        [ZoneKind::Hand, ZoneKind::Equip]
            .into_iter()
            .filter_map(|kind| state.zones.find(kind, player))
            .filter_map(|zone| state.zones.get(zone))
            .flat_map(|zone| zone.cards.iter())
            .filter(|card| {
                state
                    .cards
                    .get(**card)
                    .is_some_and(|card| effective_card_suit(state, player, card) == suit)
            })
            .count()
            .min(i32::MAX as usize) as i32,
    )
}

/// Count cards in one player's hand by effective suit.
pub fn hand_suit_count(state: &GameState, player: PlayerId, suit: u8) -> Option<i32> {
    let suit = match suit {
        1 => engine_api::Suit::Spade,
        2 => engine_api::Suit::Heart,
        3 => engine_api::Suit::Club,
        4 => engine_api::Suit::Diamond,
        _ => return None,
    };
    state.players.get(player)?;
    Some(
        state
            .hand_cards(player)
            .into_iter()
            .filter(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| effective_card_suit(state, player, card) == suit)
            })
            .count()
            .min(i32::MAX as usize) as i32,
    )
}

/// Count cards in one player's hand by effective red/black color.
pub fn hand_color_count(state: &GameState, player: PlayerId, color: u8) -> Option<i32> {
    let color = match color {
        1 => 0b01,
        2 => 0b10,
        _ => return None,
    };
    state.players.get(player)?;
    Some(
        state
            .hand_cards(player)
            .into_iter()
            .filter(|card| effective_card_color_mask(state, player, *card) == color)
            .count()
            .min(i32::MAX as usize) as i32,
    )
}

/// Read one persistent content flag from an owned skill definition.
pub fn skill_content_flag(
    state: &GameState,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
    bit: u8,
) -> Option<i32> {
    state.players.get(player)?;
    if bit >= 64 {
        return None;
    }
    let flag = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == player && skill.def == skill_def)?
        .content_flag(bit)?;
    Some(i32::from(flag))
}

/// Whether `player` is alive and has strictly more current HP than every other living player.
pub fn unique_max_current_hp(state: &GameState, player: PlayerId) -> Option<bool> {
    let subject = state.players.get(player)?;
    unique_max_current_hp_at(state, player, subject.hp)
}

/// Whether `player` would be the unique current-HP leader at `value`.
pub fn unique_max_current_hp_at(state: &GameState, player: PlayerId, value: i32) -> Option<bool> {
    let subject = state.players.get(player)?;
    Some(
        subject.alive
            && state
                .players
                .iter_live()
                .filter(|other| other.alive && other.id != player)
                .all(|other| other.hp < value),
    )
}

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

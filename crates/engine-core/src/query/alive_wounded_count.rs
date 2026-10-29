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

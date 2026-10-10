//! Typed query system with modifiers, explain, cycle detection, optional cache.
//!
//! Queries never emit effects or consume RuleRng (architecture §9).

mod cache;
mod modifier;

pub use cache::{CacheKey, QueryCache};
pub use modifier::{
    alive_wounded_or_guiming_wu_count, apply_distance_modifiers, apply_int_modifiers,
    apply_int_modifiers_for_state, apply_int_modifiers_for_state_with_context,
    apply_int_modifiers_for_state_with_other, bootstrap_modifier_defs, optional_draw_bonuses_for,
    sources_by_label, ActiveModifier, ActiveModifierIndex, CardSelector, CombineMode,
    ModifierDefinition, ModifierTarget, ModifierValueSource,
};

use crate::state::{Card, CardKind, EquipSlot, GameState, Phase, ZoneKind};
// CardKind used for equip queries
use engine_api::{CardId, PlayerId, Suit};
use rule_api::{ModifierSource, QueryErrorCode, QueryRequest, QueryResponse, RelationTag};
use std::collections::{BTreeSet, HashSet};

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

pub(crate) const XINYAOMING_DRAW: u8 = 1;
pub(crate) const XINYAOMING_DISCARD_HAND: u8 = 2;
pub(crate) const XINYAOMING_DISCARD_DRAW: u8 = 3;

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

/// Explained query result for debugger / AI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExplanation {
    /// Final response.
    pub response: QueryResponse,
    /// Base value before modifiers (if integer path).
    pub base: Option<i32>,
    /// Modifier contributions.
    pub sources: Vec<ModifierSource>,
}

/// Query context for recursion / cycle detection.
#[derive(Debug)]
pub struct QueryContext {
    /// Depth.
    pub depth: u32,
    /// Max depth.
    pub max_depth: u32,
    /// In-flight request keys (cycle detection).
    in_flight: BTreeSet<Vec<u8>>,
    /// Optional cache.
    cache: Option<QueryCache>,
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryContext {
    /// Default context (no cache).
    ///
    /// # 缓存边界契约（ADR 0703 第②部分）
    ///
    /// 缓存键的依赖版本为 `GameState::query_dep_version()`：提交版本 +
    /// 结算内伤害/恢复/选择序号 + RNG 调用数。结算内这些哨兵推进后
    /// 同键查询不会再命中旧值。**残余边界**：不经过上述哨兵的静默
    /// 字段写入（如宿主直改状态字段）不使键失效——跨命令/跨事务复用
    /// 本上下文时，宿主必须以提交边界为准调用 [`QueryContext::clear`]
    /// （或按命令重建上下文）。内核自身（vm_host）按命令新建无缓存
    /// 上下文，不受影响。
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 32,
            in_flight: BTreeSet::new(),
            cache: None,
        }
    }

    /// 缓存命中次数（宿主侧可观察缓存边界；无缓存时恒为 0）。
    pub fn cache_hits(&self) -> u64 {
        self.cache.as_ref().map(|c| c.stats().0).unwrap_or(0)
    }

    /// Context with cache control.
    pub fn with_cache(enabled: bool) -> Self {
        let mut c = QueryCache::new();
        c.set_enabled(enabled);
        Self {
            depth: 0,
            max_depth: 32,
            in_flight: BTreeSet::new(),
            cache: Some(c),
        }
    }

    /// Enable/disable cache.
    pub fn set_cache_enabled(&mut self, on: bool) {
        if let Some(c) = &mut self.cache {
            c.set_enabled(on);
        } else if on {
            let mut c = QueryCache::new();
            c.set_enabled(true);
            self.cache = Some(c);
        }
    }

    /// Clear cache.
    pub fn clear_cache(&mut self) {
        if let Some(c) = &mut self.cache {
            c.clear();
        }
    }
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
            response: QueryResponse::Bool(matches!(
                state.turn.phase,
                crate::state::Phase::Play
            )),
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

fn zone_explain(state: &GameState, player: PlayerId, kind: u8) -> QueryExplanation {
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

fn player_int_explain(
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

fn distance_explain(state: &GameState, from: PlayerId, to: PlayerId) -> QueryExplanation {
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

fn base_distance(state: &GameState, from: PlayerId, to: PlayerId) -> Result<i32, QueryErrorCode> {
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

fn attack_range_explain(state: &GameState, player: PlayerId) -> QueryExplanation {
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

fn has_equip_kind(state: &GameState, player: PlayerId, kind: CardKind) -> bool {
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

fn equipped_weapon_range(state: &GameState, player: PlayerId) -> Option<u8> {
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

fn hand_limit_explain(state: &GameState, player: PlayerId) -> QueryExplanation {
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

fn sha_damage_from_modifiers(
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

fn sha_use_limit(state: &GameState, player: PlayerId, unlimited: bool) -> u32 {
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

fn first_hand_card_suit_use_unlimited(state: &GameState, player: PlayerId, card: &Card) -> bool {
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

fn sha_rank_skill_mark_modifier_active(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    target: ModifierTarget,
    rank_is_less: bool,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if !effective_card_kind(state, player, card).is_sha() {
        return false;
    }
    let rank = i32::from(card.rank);
    let mods: Vec<_> = state
        .modifiers
        .for_target(target)
        .into_iter()
        .filter(|modifier| {
            if modifier.owner != player {
                return false;
            }
            let threshold = state
                .players
                .get(player)
                .and_then(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == modifier.skill_def.raw())
                })
                .map(|(_, count)| *count)
                .unwrap_or(0);
            threshold > 0
                && if rank_is_less {
                    rank < threshold
                } else {
                    rank == 0 || rank > threshold
                }
        })
        .collect();
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether this exact logical Sha bypasses and does not consume its ordinary use count.
pub fn sha_use_does_not_count_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    sha_rank_skill_mark_modifier_active(
        state,
        player,
        card,
        ModifierTarget::ShaNoUseCountIfRankGtSkillMark,
        false,
    )
}

/// Marks carried by `target` as `(mark_def, count)` pairs.
fn target_marks(state: &GameState, target: PlayerId) -> Vec<(u32, i32)> {
    state
        .players
        .get(target)
        .map(|player| player.marks.clone())
        .unwrap_or_default()
}

fn skill_bound_target_modifier_applies(
    state: &GameState,
    modifier: &&ActiveModifier,
    owner: PlayerId,
    target: PlayerId,
) -> bool {
    modifier.owner == owner
        && state
            .skills
            .get(modifier.skill_id)
            .and_then(|skill| skill.bound_target())
            == Some(target)
}

fn skill_card_policy_modifier_applies(
    state: &GameState,
    modifier: &&ActiveModifier,
    owner: PlayerId,
    card: engine_api::CardId,
    policy: u8,
) -> bool {
    modifier.owner == owner
        && state
            .skills
            .get(modifier.skill_id)
            .is_some_and(|skill| skill.card_use_policy(card) & policy != 0)
}

/// Whether a proactive Sha may exceed its ordinary use count for this specific target.
pub fn sha_use_unlimited_for_target(state: &GameState, player: PlayerId, target: PlayerId) -> bool {
    let mut mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaUseUnlimitedForSkillBoundTarget)
        .into_iter()
        .filter(|modifier| skill_bound_target_modifier_applies(state, modifier, player, target))
        .collect();
    // 政略②（jsrgzhenglve）：目标持有任一被声明豁免的标记即不计数。
    for (mark, _) in target_marks(state, target) {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::ShaUseUnlimitedTargetMark(mark))
                .into_iter()
                .filter(|modifier| modifier.owner == player),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card, when resolved as Sha, cannot be answered by its target.
pub fn sha_unrespondable_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    sha_unrespondable_for_logical_kind(state, player, card, kind)
}

/// Whether one concrete Sha cannot be answered by this specific target.
pub fn sha_unrespondable_for_card_target(
    state: &GameState,
    player: PlayerId,
    target: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    if !kind.is_sha() {
        return false;
    }
    if sha_unrespondable_for_logical_kind(state, player, card, kind) {
        return true;
    }
    if state.turn.current == player
        && state
            .players
            .get(player)
            .into_iter()
            .flat_map(|player| player.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .filter(|skill| state.skill_effectively_enabled(skill))
            .filter_map(|skill| skill.luanqun_helper())
            .any(|(behavior, sources)| behavior == 2 && sources.contains(&target))
    {
        return true;
    }
    let target_mods = state
        .modifiers
        .for_target(ModifierTarget::ShaTargetUnrespondable);
    let (target_locked, _, target_vetoed) =
        apply_int_modifiers_for_state(state, 0, &target_mods, Some(target));
    if target_vetoed || target_locked != 0 {
        return true;
    }
    let distance = match distance_explain(state, player, target).response {
        QueryResponse::Int(distance) => distance,
        _ => return false,
    };
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaUnrespondableWithinDistance);
    let (threshold, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || (threshold > 0 && distance <= threshold)
}

/// Whether every target of the source's next Sha is one of its Luanqun owners.
pub(crate) fn luanqun_sha_targets_allowed(
    state: &GameState,
    player: PlayerId,
    targets: &[PlayerId],
) -> bool {
    if state.turn.current != player || state.turn.phase != crate::state::Phase::Play {
        return true;
    }
    let allowed: std::collections::BTreeSet<_> = state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .filter(|skill| state.skill_effectively_enabled(skill))
        .filter_map(|skill| skill.luanqun_helper())
        .filter(|(behavior, _)| *behavior == 1)
        .flat_map(|(_, sources)| sources)
        .collect();
    allowed.is_empty() || targets.iter().all(|target| allowed.contains(target))
}

/// Whether one whole Sha use is unrespondable because its source or any target owns a matching
/// mutual-distance modifier. Distance is measured from the other participant to the owner, which
/// preserves directional distance modifiers such as defensive mounts.
pub fn sha_use_unrespondable_within_distance(
    state: &GameState,
    source: PlayerId,
    targets: &[PlayerId],
) -> bool {
    state
        .modifiers
        .for_target(ModifierTarget::ShaUseUnrespondableWithinDistance)
        .into_iter()
        .any(|modifier| {
            let owner = modifier.owner;
            let threshold = modifier.value;
            if threshold <= 0 {
                return false;
            }
            let within = |from, to| {
                matches!(
                    distance_explain(state, from, to).response,
                    QueryResponse::Int(distance) if distance <= threshold
                )
            };
            if owner == source {
                targets.iter().any(|target| within(*target, owner))
            } else {
                targets.contains(&owner) && within(source, owner)
            }
        })
}

/// Whether `responder` may not answer the current card used by `source` because a source-owned
/// modifier covers the final directional distance between them.
pub fn card_use_unrespondable_within_distance(
    state: &GameState,
    source: PlayerId,
    responder: PlayerId,
) -> bool {
    if source == responder {
        return false;
    }
    let covered = |target, from, to| {
        let mods = state.modifiers.for_target(target);
        let (threshold, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
        if vetoed {
            return true;
        }
        if threshold <= 0 {
            return false;
        }
        matches!(
            distance_explain(state, from, to).response,
            QueryResponse::Int(distance) if distance <= threshold
        )
    };
    covered(
        ModifierTarget::CardUseUnrespondableWithinResponderToOwnerDistance,
        responder,
        source,
    ) || covered(
        ModifierTarget::CardUseUnrespondableWithinOwnerToResponderDistance,
        source,
        responder,
    )
}

/// Whether the current logical card used by `source` cannot be answered by another player.
pub fn card_use_unrespondable_for_kind(
    state: &GameState,
    source: PlayerId,
    logical_kind: CardKind,
    suit: engine_api::Suit,
) -> bool {
    let mods = state
        .modifiers
        .for_card_use_unrespondable(logical_kind, suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

/// tanhu3（批 170，内容层变体）：目标集合中存在「拥有某 def 技能、且任意角色挂有
/// 对应 `CardUseUnrespondableSkillOnOwner(def)` 声明」的角色时，以该角色为目标的
/// 锦囊不可被无懈响应。以目标侧 def 反查声明（与 tanhu3 距离变体同构）。
pub fn wuxie_blocked_by_target_skill_declarations(
    state: &GameState,
    targets: &[PlayerId],
) -> bool {
    targets.iter().any(|target| {
        state
            .skills
            .sorted_live()
            .iter()
            .filter(|instance| instance.owner == *target)
            .any(|instance| {
                !state
                    .modifiers
                    .for_target(ModifierTarget::CardUseUnrespondableSkillOnOwner(
                        instance.def.raw(),
                    ))
                    .is_empty()
            })
    })
}

/// Whether 势施 froze `responder` into the direct-hit set of this exact physical card use.
pub(crate) fn potshishi_blocks_response(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
    responder: PlayerId,
) -> bool {
    card.is_some()
        && source != responder
        && state
            .players
            .get(source)
            .into_iter()
            .flat_map(|player| player.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .filter(|skill| state.skill_effectively_enabled(skill))
            .any(|skill| skill.potshishi_blocks(card, responder))
}

/// Colors that `player` discarded this round for the exact 颂谳 Wuzhong resolution.
pub(crate) fn potsongyan_response_colors(
    state: &GameState,
    card: engine_api::CardId,
    player: PlayerId,
) -> u8 {
    if card.is_none() {
        return 0;
    }
    state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| state.skill_effectively_enabled(skill))
        .fold(0, |mask, skill| {
            mask | skill.potsongyan_response_colors(state.turn.round, card, player)
        })
        & 0b11
}

/// Whether `card`, when converted to `logical_kind`, cannot be answered as Sha.
pub fn sha_unrespondable_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
    logical_kind: CardKind,
) -> bool {
    if !logical_kind.is_sha() {
        return false;
    }
    let mut mods = state
        .modifiers
        .for_sha_unrespondable(logical_kind, effective_card_suit(state, player, card));
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardUseUnrespondableForSkillCardPolicy)
            .into_iter()
            .filter(|modifier| {
                skill_card_policy_modifier_applies(
                    state,
                    modifier,
                    player,
                    card.id,
                    crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE,
                )
            }),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card use cannot be answered by Wuxie.
pub fn card_use_unrespondable_for_card(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
    logical_kind: CardKind,
    suit: engine_api::Suit,
) -> bool {
    let mut mods = state
        .modifiers
        .for_card_use_unrespondable(logical_kind, suit);
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardUseUnrespondableForSkillCardPolicy)
            .into_iter()
            .filter(|modifier| {
                skill_card_policy_modifier_applies(
                    state,
                    modifier,
                    source,
                    card,
                    crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE,
                )
            }),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

/// Whether cards used by `player` ignore their ordinary target-distance limit.
pub fn card_distance_unlimited_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardDistanceUnlimited);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether a use resolved as 杀 ignores distance, including view-as 杀.
pub fn sha_distance_unlimited_for(state: &GameState, player: PlayerId) -> bool {
    let mut mods = state
        .modifiers
        .for_target(ModifierTarget::CardDistanceUnlimited);
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardDistanceUnlimitedFor(CardSelector::Sha)),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card used by `player` ignores its target-distance limit.
/// 祸首：南蛮实体牌伤害的来源重定向为修正器持有者（ABI 541）。
pub fn nanman_damage_source_for(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
) -> Option<PlayerId> {
    let is_nanman = state.cards.get(card).is_some_and(|value| {
        crate::query::effective_card_kind(state, source, value) == CardKind::TrickNanman
    });
    if !is_nanman {
        return None;
    }
    state
        .modifiers
        .for_target(ModifierTarget::NanmanDamageSourceAsOwner)
        .into_iter()
        .filter(|modifier| {
            modifier.owner != source
                && state
                    .players
                    .get(modifier.owner)
                    .is_some_and(|owner| owner.alive)
        })
        .map(|modifier| modifier.owner)
        .next()
}

/// Whether one concrete card ignores its target-distance limit.
pub fn card_distance_unlimited_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if sha_rank_skill_mark_modifier_active(
        state,
        player,
        card,
        ModifierTarget::ShaDistanceUnlimitedIfRankLtSkillMark,
        true,
    ) {
        return true;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if chenglve_suit_unlimited_for(state, player, card) {
        return true;
    }
    let kind = effective_card_kind(state, player, card);
    let mut mods = state
        .modifiers
        .for_card_distance(kind, effective_card_suit(state, player, card));
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardDistanceUnlimitedForSkillCardPolicy)
            .into_iter()
            .filter(|modifier| {
                skill_card_policy_modifier_applies(
                    state,
                    modifier,
                    player,
                    card.id,
                    crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED,
                )
            }),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card ignores distance to this specific target.
pub fn card_distance_unlimited_for_target(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    if card_distance_unlimited_for_card(state, player, card) {
        return true;
    }
    if sha_target_distance_lte_rank_modifier_active(state, player, card, target) {
        return true;
    }
    let bound_mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::CardDistanceUnlimitedForSkillBoundTarget)
        .into_iter()
        .filter(|modifier| skill_bound_target_modifier_applies(state, modifier, player, target))
        .collect();
    let (bound_value, _, bound_vetoed) =
        apply_int_modifiers_for_state(state, 0, &bound_mods, Some(player));
    if bound_vetoed || bound_value != 0 {
        return true;
    }
    // 政略②（jsrgzhenglve）：目标持有任一被声明豁免的标记即无视距离。
    let mut mark_mods = Vec::new();
    for (mark, _) in target_marks(state, target) {
        mark_mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardDistanceUnlimitedTargetMark(mark))
                .into_iter()
                .filter(|modifier| modifier.owner == player),
        );
    }
    let (mark_value, _, mark_vetoed) =
        apply_int_modifiers_for_state(state, 0, &mark_mods, Some(player));
    if mark_vetoed || mark_value != 0 {
        return true;
    }
    // tanhu3（批 168，ABI 679）：任意目标持有被声明的技能（def）时，任何使用者
    // 对其无视距离（与政略②方向相反：技能在目标侧；modifier 为全局声明，
    // 不限定使用方或持有方）。
    let mut on_owner_mods = Vec::new();
    for skill_def in state
        .skills
        .sorted_live()
        .iter()
        .filter(|instance| instance.owner == target)
        .map(|instance| instance.def.raw())
    {
        on_owner_mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardDistanceUnlimitedSkillOnOwner(skill_def)),
        );
    }
    // 目标侧声明：不做 owner_filter（modifier 挂在声明技持有者身上，作用于任意使用方）。
    let (on_owner_value, _, on_owner_vetoed) =
        apply_int_modifiers_for_state(state, 0, &on_owner_mods, None);
    if on_owner_vetoed || on_owner_value != 0 {
        return true;
    }
    if state.hand_cards(target).len() < state.hand_cards(player).len() {
        return false;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    let mods = state
        .modifiers
        .for_card_distance_if_target_hand_gte_owner(kind, effective_card_suit(state, player, card));
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

fn sha_target_distance_lte_rank_modifier_active(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if card.rank == 0 || !effective_card_kind(state, player, card).is_sha() {
        return false;
    }
    let within_rank = matches!(
        query(
            state,
            &QueryRequest::Distance {
                from: player,
                to: target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(distance) if distance <= i32::from(card.rank)
    );
    if !within_rank {
        return false;
    }
    let mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaDistanceUnlimitedIfTargetDistanceLteRank)
        .into_iter()
        .filter(|modifier| modifier.owner == player)
        .collect();
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether another player is forbidden from discarding this physical equipment card.
pub fn card_discard_protected_from(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if actor == target {
        return false;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let Some(equip) = state.zones.find(ZoneKind::Equip, target) else {
        return false;
    };
    if card.owner != target || card.zone != equip {
        return false;
    }
    let mods = state
        .modifiers
        .for_card_discard_protection(card.kind, card.suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}

/// Whether another player is forbidden from gaining this physical card from the target.
pub fn card_gain_protected_from(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    card: CardId,
) -> bool {
    if actor == target {
        return false;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let in_target_personal_zone = card.owner == target
        && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
            .into_iter()
            .any(|kind| state.zones.find(kind, target) == Some(card.zone));
    if !in_target_personal_zone {
        return false;
    }
    let mods = state
        .modifiers
        .for_card_gain_protection(card.kind, card.suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}

/// Cards the owner may choose for a self-discard from hand or equipment.
pub fn discardable_hand_equip_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let mut cards = state.hand_cards(player);
    if let Some(equip) = state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone))
    {
        cards.extend(equip.cards.iter().copied());
    }
    cards
}

/// Current hand cards eligible for Yeyan's four-distinct-suit cost.
pub fn yeyan_hand_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            state.cards.get(*card).is_some_and(|card| {
                effective_card_suit(state, player, card) != engine_api::Suit::None
            })
        })
        .collect()
}

/// Effective basic cards in a player's hand that may pay Standard Xiaoguo's cost.
pub fn standard_xiaoguo_basic_hand_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| effective_card_kind(state, player, card).is_basic())
        })
        .collect()
}

/// Effective equipment cards in a player's hand/equipment zones for Standard Xiaoguo.
pub fn standard_xiaoguo_equipment_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    discardable_hand_equip_cards(state, player)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| effective_card_kind(state, player, card).is_equip())
        })
        .collect()
}

/// Hand cards that may pay SP Xiaoguo's unrestricted self-discard cost.
pub fn sp_xiaoguo_hand_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state.hand_cards(player)
}

/// Effective equipment cards the phase player may self-discard for SP Xiaoguo.
pub fn sp_xiaoguo_equipment_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    discardable_hand_equip_cards(state, player)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| effective_card_kind(state, player, card).is_equip())
        })
        .collect()
}

/// Discardable non-basic hand/equipment cards whose effective color matches at least one of the
/// player's currently equipped cards.
pub fn discardable_non_basic_matching_equip_color_cards(
    state: &GameState,
    player: PlayerId,
) -> Vec<CardId> {
    let equip_color_mask = state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| {
            zone.cards.iter().fold(0, |mask, card| {
                mask | effective_card_color_mask(state, player, *card)
            })
        })
        .unwrap_or(0);
    if equip_color_mask == 0 {
        return Vec::new();
    }
    discardable_hand_equip_cards(state, player)
        .into_iter()
        .filter(|card| {
            state.cards.get(*card).is_some_and(|card_state| {
                !effective_card_kind(state, player, card_state).is_basic()
                    && effective_card_color_mask(state, player, *card) & equip_color_mask != 0
            })
        })
        .collect()
}

/// Equipment cards that `actor` may discard from `target`.
pub fn discardable_equipment_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    state
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| !card_discard_protected_from(state, actor, target, *card))
                .collect()
        })
        .unwrap_or_default()
}

/// Cards that `actor` may discard from `target` within a canonical target-card scope.
///
/// The returned order is stable by zone order (hand, equipment, judgment) and then canonical
/// zone order. Equipment discard protection is applied through the shared modifier query.
pub fn discardable_target_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Vec<CardId> {
    let kinds = [
        scope.includes_hand().then_some(ZoneKind::Hand),
        scope.includes_equip().then_some(ZoneKind::Equip),
        scope.includes_judge().then_some(ZoneKind::Judge),
    ];
    let mut cards: Vec<_> = kinds
        .into_iter()
        .flatten()
        .filter_map(|kind| state.zones.find(kind, target))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| !card_discard_protected_from(state, actor, target, *card))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Cards that `actor` may gain from `target` within a canonical target-card scope.
///
/// The returned order is stable by physical card id. Target-owned gain protection is applied
/// through the shared modifier query for hand, equipment, and judgment cards alike.
pub fn gainable_target_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Vec<CardId> {
    let kinds = [
        scope.includes_hand().then_some(ZoneKind::Hand),
        scope.includes_equip().then_some(ZoneKind::Equip),
        scope.includes_judge().then_some(ZoneKind::Judge),
    ];
    let mut cards: Vec<_> = kinds
        .into_iter()
        .flatten()
        .filter_map(|kind| state.zones.find(kind, target))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| !card_gain_protected_from(state, actor, target, *card))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Living previous/next seats around `player`, deduplicated in a two-player game.
pub fn alive_seat_neighbors(state: &GameState, player: PlayerId) -> Vec<PlayerId> {
    let alive = state.players.alive_by_seat();
    let Some(index) = alive.iter().position(|candidate| *candidate == player) else {
        return Vec::new();
    };
    if alive.len() < 2 {
        return Vec::new();
    }
    let previous = alive[(index + alive.len() - 1) % alive.len()];
    let next = alive[(index + 1) % alive.len()];
    if previous == next {
        vec![previous]
    } else {
        vec![previous, next]
    }
}

/// Discardable armor equipment cards that `actor` may discard from `target`.
pub fn discardable_armor_equipment_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    discardable_equipment_cards(state, actor, target)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                == Some(crate::state::EquipSlot::Armor)
        })
        .collect()
}

/// Whether two targets form a legal equipment-discard pair containing armor.
pub fn equipment_pair_with_armor_legal(
    state: &GameState,
    actor: PlayerId,
    targets: &[PlayerId],
) -> bool {
    targets.len() == 2
        && targets[0] != targets[1]
        && targets.iter().all(|target| {
            state
                .players
                .get(*target)
                .is_some_and(|player| player.alive)
                && !discardable_equipment_cards(state, actor, *target).is_empty()
        })
        && targets
            .iter()
            .any(|target| !discardable_armor_equipment_cards(state, actor, *target).is_empty())
}

/// Whether `player` is currently forbidden from responding with 杀.
pub fn sha_response_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaResponseCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is currently forbidden from proactively using any card.
pub fn card_use_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::CardUseCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is forbidden from using one concrete card.
pub fn card_use_cancelled_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return card_use_cancelled_for(state, player);
    };
    if ddd_lianer_card_blocked(state, player, card) {
        return true;
    }
    card_use_cancelled_for_logical_kind(
        state,
        player,
        card,
        effective_card_kind(state, player, card),
    )
}

pub(crate) fn card_use_cancelled_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
    logical_kind: crate::state::CardKind,
) -> bool {
    if logical_kind == crate::state::CardKind::Tao
        && tao_cancelled_for_non_dying_other_during_owner_turn(state, player, None)
    {
        return true;
    }
    let mut mods = state
        .modifiers
        .for_card_use(logical_kind, effective_card_suit(state, player, card));
    mods.extend(card_color_lock_modifiers_for_suit(
        state,
        player,
        effective_card_suit(state, player, card),
    ));
    if state
        .zones
        .find(ZoneKind::Hand, player)
        .is_some_and(|hand| card.zone == hand)
    {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::HandCardUseResponseCancel),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Stable red/black bit used by versioned skill color-lock state.
pub(crate) const fn card_color_mask_for_suit(suit: engine_api::Suit) -> u8 {
    match suit {
        engine_api::Suit::Heart | engine_api::Suit::Diamond => 0b01,
        engine_api::Suit::Spade | engine_api::Suit::Club => 0b10,
        engine_api::Suit::None => 0,
    }
}

/// Effective red/black bit of one physical card for one player.
pub(crate) fn effective_card_color_mask(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> u8 {
    state
        .cards
        .get(card)
        .map(|card| card_color_mask_for_suit(effective_card_suit(state, player, card)))
        .unwrap_or(0)
}

/// Whether a physical response material matches the color lock on the innermost card use.
pub(crate) fn response_color_locked_for_current_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(record) = state.resolution.pending_card_resolutions.last() else {
        return false;
    };
    player != record.source
        && record.response_color_lock_mask != 0
        && effective_card_color_mask(state, player, card) & record.response_color_lock_mask != 0
}

fn card_color_lock_modifiers_for_suit(
    state: &GameState,
    player: PlayerId,
    suit: engine_api::Suit,
) -> Vec<&ActiveModifier> {
    let color = card_color_mask_for_suit(suit);
    if color == 0 {
        return Vec::new();
    }
    state
        .modifiers
        .for_target(ModifierTarget::CardActionCancelForSkillColorMask)
        .into_iter()
        .filter(|modifier| {
            modifier.owner == player
                && state
                    .skills
                    .get(modifier.skill_id)
                    .and_then(|skill| skill.card_color_lock())
                    .is_some_and(|(_, mask)| mask & color != 0)
        })
        .collect()
}

/// Whether a physical card is blocked by a temporary skill-instance color or rank lock.
pub(crate) fn card_color_locked_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if ddd_lianer_card_blocked(state, player, card) {
        return true;
    }
    let mods =
        card_color_lock_modifiers_for_suit(state, player, effective_card_suit(state, player, card));
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

fn ddd_lianer_card_blocked(state: &GameState, player: PlayerId, card: &crate::state::Card) -> bool {
    state.players.get(player).is_some_and(|owner| {
        owner.skills.iter().any(|skill| {
            state.skills.get(*skill).is_some_and(|instance| {
                instance.def.raw() == 1_083_711_762
                    && state.skill_effectively_enabled(instance)
                    && instance.mark > 0
                    && (card.rank == 0 || u32::from(card.rank) >= instance.mark)
            })
        })
    })
}

/// Whether `player` is forbidden from using hand cards as use/response inputs.
pub fn hand_card_use_response_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let current = state.turn.current;
    if player != current
        && state.players.get(current).is_some_and(|participant| {
            participant.alive
                && participant.skills.iter().any(|skill| {
                    state.skills.get(*skill).is_some_and(|instance| {
                        instance.def.raw() == 1_632_230_116
                            && state.skill_effectively_enabled(instance)
                    })
                })
                && state
                    .hand_cards(current)
                    .into_iter()
                    .all(|card| state.cards.get(card).is_some_and(|card| card.face_up))
        })
    {
        return true;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::HandCardUseResponseCancel);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether a Play-phase modifier restricts this player to hand cards gained this turn.
pub fn hand_card_gained_this_turn_only_for(state: &GameState, player: PlayerId) -> bool {
    if state.turn.current != player || state.turn.phase != crate::state::Phase::Play {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::HandCardGainedThisTurnOnly);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is currently forbidden from responding with any card.
pub fn card_response_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardResponseCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one physical card is forbidden as a response under its supplied logical kind.
pub(crate) fn card_response_cancelled_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
    logical_kind: CardKind,
) -> bool {
    if ddd_lianer_card_blocked(state, player, card) {
        return true;
    }
    let mods = state
        .modifiers
        .for_card_response(logical_kind, effective_card_suit(state, player, card));
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is currently forbidden from using a card to save a dying player.
pub fn card_save_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    card_save_cancelled_for_kind(state, player, CardKind::Tao)
}

/// Whether `player` is forbidden from using one logical card kind to save themselves.
pub fn card_save_cancelled_for_kind(state: &GameState, player: PlayerId, kind: CardKind) -> bool {
    card_save_cancelled_for_kind_to(state, player, player, kind)
}

/// Whether `player` is forbidden from using one logical card kind to save `dying`.
pub fn card_save_cancelled_for_kind_to(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
    kind: CardKind,
) -> bool {
    card_save_cancelled_without_hand_source(state, player, dying, kind)
        || hand_card_use_response_cancelled_for(state, player)
}

pub(crate) fn card_save_cancelled_without_hand_source(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
    kind: CardKind,
) -> bool {
    if kind == CardKind::Tao
        && tao_cancelled_for_non_dying_other_during_owner_turn(state, player, Some(dying))
    {
        return true;
    }
    if kind == CardKind::Tao && tao_cancelled_for_dying_beyond_distance(state, player, dying) {
        return true;
    }
    let mut mods = state.modifiers.for_target(ModifierTarget::CardSaveCancel);
    if let Some(selector) = match kind {
        CardKind::Tao => Some(CardSelector::Tao),
        CardKind::Jiu => Some(CardSelector::Jiu),
        _ => None,
    } {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardSaveCancelFor(selector)),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0 || card_target_cancelled_for_kind(state, player, dying, kind)
}

fn tao_cancelled_for_non_dying_other_during_owner_turn(
    state: &GameState,
    player: PlayerId,
    dying: Option<PlayerId>,
) -> bool {
    let Some(participant) = state.players.get(player) else {
        return false;
    };
    let is_current_dying_player = dying.map_or(participant.dying, |dying| dying == player);
    if !participant.alive || is_current_dying_player {
        return false;
    }
    let owner = state.turn.current;
    if owner.is_none()
        || owner == player
        || !state
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(owner));
    vetoed || value != 0
}

/// Whether `player`'s 桃 save is cancelled because the dying target is beyond distance 2 and
/// `player` carries a `tao_use_save_cancel_for_dying_beyond_distance` modifier. Powers 「泣梓」.
fn tao_cancelled_for_dying_beyond_distance(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
) -> bool {
    if player == dying {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::TaoUseSaveCancelForDyingBeyondDistance);
    if mods.is_empty() {
        return false;
    }
    let (_value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    if !(vetoed || _value != 0) {
        return false;
    }
    base_distance(state, player, dying).map_or(false, |d| d > 2)
}

/// Whether one physical card satisfies a declarative view-as filter.
pub(crate) fn view_as_filter_matches(
    state: &GameState,
    player: PlayerId,
    filter: &str,
    card: &Card,
) -> bool {
    let suit = effective_card_suit(state, player, card);
    let kind = effective_card_kind(state, player, card);
    match filter {
        "any" => true,
        "red" => matches!(suit, Suit::Heart | Suit::Diamond),
        "black" => matches!(suit, Suit::Spade | Suit::Club),
        "heart" => suit == Suit::Heart,
        "spade" => suit == Suit::Spade,
        "club" => suit == Suit::Club,
        "diamond" => suit == Suit::Diamond,
        "owner_dying_club" => {
            state.players.get(player).is_some_and(|owner| owner.hp <= 0) && suit == Suit::Club
        }
        "owner_outside_turn" => state.turn.current != player,
        "owner_outside_turn_red" => {
            state.turn.current != player && matches!(suit, Suit::Heart | Suit::Diamond)
        }
        "owner_single_hand" => state.hand_count(player) == 1,
        "owner_single_red_hand" => {
            matches!(suit, Suit::Heart | Suit::Diamond)
                && state
                    .hand_cards(player)
                    .into_iter()
                    .filter_map(|card| state.cards.get(card))
                    .filter(|card| {
                        matches!(
                            effective_card_suit(state, player, card),
                            Suit::Heart | Suit::Diamond
                        )
                    })
                    .count()
                    == 1
        }
        "owner_hand_gt_hp" => state
            .players
            .get(player)
            .is_some_and(|owner| state.hand_count(player) as i32 > owner.hp),
        "owner_hp_le_two_black_trick" => {
            state.players.get(player).is_some_and(|owner| owner.hp <= 2)
                && matches!(suit, Suit::Spade | Suit::Club)
                && kind.is_trick()
        }
        "card_sha" => kind.is_sha(),
        "card_shan" => kind == CardKind::Shan,
        "card_tao" => kind == CardKind::Tao,
        "card_jiu" => kind == CardKind::Jiu,
        "card_ying" => kind == CardKind::Ying,
        "card_basic" => kind.is_basic(),
        "card_equip" => kind.is_equip(),
        "card_trick" => kind.is_trick(),
        "damage_trick" => kind.is_damage_trick(),
        "non_basic" => !kind.is_basic(),
        "color_ne_shuangxiong" => {
            let recorded = state
                .players
                .get(player)
                .map(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == crate::effect::SHUANGXIONG_COLOR_MARK)
                        .map(|(_, count)| *count)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            match recorded {
                // Red recorded: black cards qualify.
                1 => matches!(suit, Suit::Spade | Suit::Club),
                // Black recorded: red cards qualify.
                2 => matches!(suit, Suit::Heart | Suit::Diamond),
                _ => false,
            }
        }
        "black_basic_equip" => {
            matches!(suit, Suit::Spade | Suit::Club) && (kind.is_basic() || kind.is_equip())
        }
        _ => false,
    }
}

/// Whether a physical card is in the source zone declared by a view-as rule.
pub(crate) fn view_as_source_zone_matches(
    state: &GameState,
    player: PlayerId,
    source_zone: Option<&str>,
    card: &Card,
) -> bool {
    let borrowed_hand = borrowed_hand_source(state, player, card.id).is_some();
    let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone) || borrowed_hand;
    let in_equip =
        card.owner == player && state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = state.zones.get(card.zone).is_some_and(|zone| {
        card.owner == player && zone.kind == ZoneKind::Expansion && zone.owner == player
    });
    match source_zone.unwrap_or("hand") {
        "hand" => in_hand,
        "equip" => in_equip,
        "hand_equip" => in_hand || in_equip,
        "hand_expansion" => in_hand || in_expansion,
        "hand_equip_expansion" => in_hand || in_equip || in_expansion,
        _ => false,
    }
}

/// First deterministic self-save card: logical 桃 before 酒, then natural/skill/source order.
pub fn dying_save_card(state: &GameState, player: PlayerId) -> Option<CardId> {
    dying_save_card_for(state, player, player)
}

/// First deterministic card with which `player` may save `dying`. Other players may use Tao;
/// only the dying player may additionally self-save with Jiu.
pub fn dying_save_card_for(state: &GameState, player: PlayerId, dying: PlayerId) -> Option<CardId> {
    dying_save_card_and_kind_for(state, player, dying).map(|(card, _)| card)
}

/// First deterministic save card together with the logical Tao/Jiu kind it supplies.
pub(crate) fn dying_save_card_and_kind_for(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
) -> Option<(CardId, CardKind)> {
    let hand = state.playable_hand_cards(player);
    [(CardKind::Tao, "tao"), (CardKind::Jiu, "jiu")]
        .into_iter()
        .filter(|(kind, _)| *kind != CardKind::Jiu || player == dying)
        .find_map(|(kind, as_name)| {
            if let Some(card) = hand.iter().copied().find(|card| {
                !card_save_cancelled_for_kind_to(state, player, dying, kind)
                    && !card_color_locked_for_card(state, player, *card)
                    && state
                        .cards
                        .get(*card)
                        .is_some_and(|card| effective_card_kind(state, player, card) == kind)
            }) {
                return Some((card, kind));
            }
            let owner = state.players.get(player)?;
            owner.skills.iter().find_map(|skill_id| {
                let skill = state.skills.get(*skill_id)?;
                if !state.skill_effectively_enabled(skill) {
                    return None;
                }
                state
                    .content
                    .view_as_defs_for_skill(skill.def)
                    .into_iter()
                    .filter(|view| {
                        view.as_kind == as_name
                            && matches!(view.window.as_str(), "any" | "response")
                            && view.max_cards == 1
                            && !view.owner_hp_min_one_cards
                            && !view.owner_basic_uses_plus_one_cards
                    })
                    .find_map(|view| {
                        state
                            .cards
                            .sorted_live()
                            .into_iter()
                            .find(|card| {
                                let in_hand =
                                    state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
                                !card_save_cancelled_without_hand_source(state, player, dying, kind)
                                    && !card_color_locked_for_card(state, player, card.id)
                                    && (!in_hand
                                        || !hand_card_use_response_cancelled_for(state, player))
                                    && view_as_source_zone_matches(
                                        state,
                                        player,
                                        view.source_zone.as_deref(),
                                        card,
                                    )
                                    && view_as_filter_matches(state, player, &view.filter, card)
                            })
                            .map(|card| (card.id, kind))
                    })
            })
        })
}

/// Whether a logical card kind may select `target`.
pub fn card_target_cancelled_for_kind(
    state: &GameState,
    user: PlayerId,
    target: PlayerId,
    logical_kind: CardKind,
) -> bool {
    if logical_kind != CardKind::Tao {
        return false;
    }
    let mut mods = state
        .modifiers
        .for_target(ModifierTarget::CardTargetCancelFor(CardSelector::Tao));
    if user == state.turn.current {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelFromCurrentFor(
                    CardSelector::Tao,
                )),
        );
    }
    if user != target && state.turn.use_ban_pairs.contains(&(user, target)) {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelTurnBannedFor(
                    CardSelector::Tao,
                ))
                .into_iter()
                .chain(
                    state
                        .modifiers
                        .for_target(ModifierTarget::CardTargetCancelTurnBannedFor(
                            CardSelector::Any,
                        )),
                ),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}

/// Whether one concrete/logical card may select `target`.
pub(crate) fn card_target_cancelled_for_logical_kind(
    state: &GameState,
    user: PlayerId,
    target: PlayerId,
    card: &Card,
    logical_kind: CardKind,
) -> bool {
    let mut mods = state.modifiers.for_card_target(
        user,
        target,
        logical_kind,
        effective_card_suit(state, user, card),
        user == state.turn.current,
        &state.turn.use_ban_pairs,
    );
    let card_suit = effective_card_suit(state, user, card);
    if user != target && !armor_effect_ignored_for(state, user, target) {
        mods.extend(
            state
                .modifiers
                .for_armor_card_target(logical_kind, card_suit),
        );
    }
    if !state.hand_cards(target).into_iter().any(|hand_card| {
        state
            .cards
            .get(hand_card)
            .is_some_and(|held| effective_card_suit(state, target, held) == card_suit)
    }) {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelIfTargetLacksMatchingSuit),
        );
    }
    if user != target
        && state.hand_count(user) >= state.hand_count(target)
        && !has_equip_kind(state, target, CardKind::Armor)
    {
        mods.extend(
            state
                .modifiers
                .for_card_target_if_other_user_hand_gte_target_and_armor_empty(
                    logical_kind,
                    card_suit,
                ),
        );
    }
    if user != target
        && state
            .players
            .get(user)
            .zip(state.players.get(target))
            .is_some_and(|(user, target)| user.hp >= target.hp)
    {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelIfOtherUserHpGteTarget),
        );
    }
    if user != target {
        let distance_mods = state
            .modifiers
            .for_card_target_if_other_user_distance_gt_one(logical_kind, card_suit);
        if !distance_mods.is_empty()
            && matches!(
                distance_explain(state, user, target).response,
                QueryResponse::Int(distance) if distance > 1
            )
        {
            mods.extend(distance_mods);
        }
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}

/// Whether source-side armor bypass or target-side armor invalidation suppresses armor effects.
pub(crate) fn armor_effect_ignored_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    let source_ignores = if source.is_some() {
        let mut mods = state.modifiers.for_target(ModifierTarget::IgnoreArmor);
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::IgnoreArmorForSkillBoundTarget)
                .into_iter()
                .filter(|modifier| {
                    skill_bound_target_modifier_applies(state, modifier, source, target)
                }),
        );
        let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
        vetoed || value != 0
    } else {
        false
    };
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ArmorEffectDisabled);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    source_ignores || vetoed || value != 0
}

/// Whether armor is suppressed for one concrete card's source-side effects.
pub(crate) fn armor_effect_ignored_for_card(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if armor_effect_ignored_for(state, source, target) {
        return true;
    }
    if source.is_none() || card.is_none() {
        return false;
    }
    let mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::IgnoreArmorForSkillCardPolicy)
        .into_iter()
        .filter(|modifier| {
            skill_card_policy_modifier_applies(
                state,
                modifier,
                source,
                card,
                crate::state::SkillInstance::CARD_USE_POLICY_IGNORE_ARMOR,
            )
        })
        .collect();
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

/// Whether an unlinked player is protected by active armor from gaining the chain mark.
pub fn armor_link_gain_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    if armor_effect_ignored_for(state, PlayerId::NONE, player) {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ArmorLinkGainCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether a linked player is prevented from losing the chain mark.
pub fn link_loss_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let mods = state.modifiers.for_target(ModifierTarget::LinkLossCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether 杀 cards are excluded from this player's hand-limit accounting.
pub fn sha_ignored_for_hand_limit(state: &GameState, player: PlayerId) -> bool {
    let mut mods = state
        .modifiers
        .for_target(ModifierTarget::ShaIgnoredHandLimit);
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardIgnoredHandLimit(CardSelector::Sha)),
    );
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card is excluded from this player's hand-limit accounting.
pub fn card_ignored_for_hand_limit(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if state.turn.hand_limit_exempt_cards.contains(&(player, card)) {
        return true;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    if kind == CardKind::TrickMantianguohai {
        return true;
    }
    let mods = state.modifiers.for_hand_limit_card(
        state,
        player,
        card,
        kind,
        effective_card_suit(state, player, card),
    );
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Hand cards eligible for the ordinary discard-to-limit choice.
pub fn discard_to_limit_tokens(state: &GameState, player: PlayerId) -> Vec<engine_api::CardId> {
    let mut cards = state.hand_cards(player);
    if discard_phase_includes_equip(state, player) {
        if let Some(equip) = state
            .zones
            .find(ZoneKind::Equip, player)
            .and_then(|zone| state.zones.get(zone))
        {
            cards.extend(equip.cards.iter().copied());
        }
    }
    cards
        .into_iter()
        .filter(|card| !card_ignored_for_hand_limit(state, player, *card))
        .collect()
}

/// Effective hand size used by the discard-to-limit rule.
pub fn hand_count_for_limit(state: &GameState, player: PlayerId) -> u32 {
    discard_to_limit_tokens(state, player).len() as u32
}

/// Suit seen by rules for one player; the physical card remains unchanged.
pub fn effective_card_suit(state: &GameState, player: PlayerId, card: &Card) -> engine_api::Suit {
    if player.is_none() {
        return card.suit;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardSuit(card.suit));
    let base = match card.suit {
        engine_api::Suit::None => 0,
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 2,
        engine_api::Suit::Club => 3,
        engine_api::Suit::Diamond => 4,
    };
    let (value, _, vetoed) = apply_int_modifiers(base, &mods, Some(player));
    if vetoed {
        return card.suit;
    }
    match value {
        0 => engine_api::Suit::None,
        1 => engine_api::Suit::Spade,
        2 => engine_api::Suit::Heart,
        3 => engine_api::Suit::Club,
        4 => engine_api::Suit::Diamond,
        _ => card.suit,
    }
}

/// Available physical cards grouped by their non-virtual effective suit.
pub fn effective_suit_card_groups(
    state: &GameState,
    player: PlayerId,
    cards: &[engine_api::CardId],
) -> [Vec<engine_api::CardId>; 4] {
    let mut groups = std::array::from_fn(|_| Vec::new());
    for card_id in cards {
        let Some(card) = state.cards.get(*card_id) else {
            continue;
        };
        let index = match effective_card_suit(state, player, card) {
            engine_api::Suit::Spade => 0,
            engine_api::Suit::Heart => 1,
            engine_api::Suit::Club => 2,
            engine_api::Suit::Diamond => 3,
            engine_api::Suit::None => continue,
        };
        groups[index].push(*card_id);
    }
    groups
}

/// Number seen by rules for one player's card; the physical rank remains unchanged.
pub fn effective_card_number(state: &GameState, player: PlayerId, card: &Card) -> u8 {
    if player.is_none() {
        return card.rank;
    }
    let tagged_in_hand = state.card_has_content_tag(card.id, crate::state::SHEQUAN_CARD_TAG)
        && state.zones.get(card.zone).is_some_and(|zone| {
            zone.kind == ZoneKind::Hand && zone.owner == player && card.owner == player
        });
    if tagged_in_hand {
        let has_jianzhuan = state
            .players
            .get(player)
            .into_iter()
            .flat_map(|owner| owner.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .any(|skill| {
                skill.def.raw() == crate::state::POTJIANZHUAN_DEF
                    && state.skill_effectively_enabled(skill)
            });
        return if has_jianzhuan { 13 } else { 1 };
    }
    let mods = state.modifiers.for_target(ModifierTarget::CardNumber);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, i32::from(card.rank), &mods, Some(player));
    if vetoed {
        return card.rank;
    }
    u8::try_from(value)
        .ok()
        .filter(|number| (1..=13).contains(number))
        .unwrap_or(card.rank)
}

/// Physical number used by converted-card rules: sum every material for a multi-card use,
/// otherwise use the carrier's printed rank.
pub fn physical_card_number_sum(
    state: &GameState,
    carrier: engine_api::CardId,
    sources: &[engine_api::CardId],
) -> Option<u32> {
    if sources.len() > 1 {
        return sources.iter().try_fold(0u32, |sum, card| {
            let rank = state.cards.get(*card)?.rank;
            (rank > 0).then(|| sum.saturating_add(u32::from(rank)))
        });
    }
    let rank = state.cards.get(carrier)?.rank;
    (rank > 0).then_some(u32::from(rank))
}

/// Rule-visible length of the card's localized name, matching upstream `get.cardNameLength`.
pub fn card_name_length(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> Option<u8> {
    let card = state.cards.get(card)?;
    let kind = effective_card_kind(state, player, card);
    let fixed = match kind {
        CardKind::Ying | CardKind::TrickChuqibuyi => 0,
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => 1,
        CardKind::Tao | CardKind::Shan | CardKind::Jiu => 1,
        CardKind::TrickJuedou | CardKind::TrickShandian | CardKind::TrickHuogong => 2,
        CardKind::TrickWuzhong
        | CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickNanman
        | CardKind::TrickWanjian
        | CardKind::TrickTaoyuan
        | CardKind::TrickWugu
        | CardKind::TrickLe
        | CardKind::TrickBingliang
        | CardKind::TrickJiedao
        | CardKind::TrickTiesuo
        | CardKind::TrickWuxie
        | CardKind::TrickYuanjiao
        | CardKind::TrickTiaojiyanmei
        | CardKind::TrickMantianguohai
        | CardKind::TrickYoudishenru => 4,
        CardKind::TrickDiaobingqianjiang | CardKind::TrickWangmeizhike => 4,
        CardKind::TrickYiyi => 4,
        CardKind::TrickShuiyanqijunx => 4,
        CardKind::TrickLulitongxin => 4,
        CardKind::TrickKefuzhongyuan => 4,
        CardKind::TrickGuguoanbang => 4,
        CardKind::TrickHuoshaolianying => 4,
        CardKind::TrickLianjunshengyan | CardKind::TrickWenheluanwu => 4,
        CardKind::TrickHaolingtianxia => 4,
        CardKind::TrickChiling => 4,
        CardKind::Weapon | CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef => {
            match card.def.raw() {
                100 => 5,                   // 雌雄双股剑
                101 => 4,                   // 诸葛连弩
                102 | 105 | 110 => 3,       // 青釭剑、麒麟弓、八卦阵
                103 | 104 | 122 => 4,       // 丈八蛇矛、方天画戟、爪黄飞电
                111 | 120 | 121 | 123 => 2, // 藤甲与标准坐骑
                _ => 0,
            }
        }
        CardKind::Treasure | CardKind::Other => 0,
    };
    (fixed > 0).then_some(fixed).or_else(|| {
        state
            .content
            .card_metadata(card.def)
            .map(|card| card.name_length)
            .filter(|n| *n > 0)
    })
}

/// Whether a physical card carries the upstream `gifts` tag.
pub fn card_is_gift(state: &GameState, card: engine_api::CardId) -> bool {
    state
        .cards
        .get(card)
        .and_then(|card| state.content.card_metadata(card.def))
        .is_some_and(|metadata| metadata.gift)
}

/// Enumerate bounded, stable card subsets whose effective numbers total exactly `sum`.
pub fn effective_number_sum_subsets(
    state: &GameState,
    player: PlayerId,
    cards: &[engine_api::CardId],
    min_count: usize,
    max_count: usize,
    sum: u16,
    limit: usize,
) -> Vec<Vec<u32>> {
    fn visit(
        entries: &[(u32, u16)],
        index: usize,
        min_count: usize,
        max_count: usize,
        remaining: u16,
        limit: usize,
        current: &mut Vec<u32>,
        out: &mut Vec<Vec<u32>>,
        dead: &mut HashSet<(usize, usize, u16)>,
    ) {
        if out.len() >= limit {
            return;
        }
        let state = (index, current.len(), remaining);
        if dead.contains(&state) {
            return;
        }
        if remaining == 0 {
            if current.len() >= min_count {
                out.push(current.clone());
            }
            return;
        }
        if current.len() == max_count {
            dead.insert(state);
            return;
        }
        let before = out.len();
        for next in index..entries.len() {
            let (card, number) = entries[next];
            if number > remaining {
                continue;
            }
            current.push(card);
            visit(
                entries,
                next + 1,
                min_count,
                max_count,
                remaining - number,
                limit,
                current,
                out,
                dead,
            );
            current.pop();
            if out.len() >= limit {
                return;
            }
        }
        if out.len() == before {
            dead.insert(state);
        }
    }

    if limit == 0 || min_count > max_count || sum == 0 {
        return Vec::new();
    }
    let entries: Vec<_> = cards
        .iter()
        .filter_map(|card| {
            state.cards.get(*card).map(|state_card| {
                (
                    card.raw(),
                    u16::from(effective_card_number(state, player, state_card)),
                )
            })
        })
        .collect();
    let mut out = Vec::new();
    visit(
        &entries,
        0,
        min_count,
        max_count.min(entries.len()),
        sum,
        limit,
        &mut Vec::new(),
        &mut out,
        &mut HashSet::new(),
    );
    out
}

/// Rule category seen by one player; the physical card remains unchanged.
pub fn effective_card_kind(state: &GameState, player: PlayerId, card: &Card) -> CardKind {
    if player.is_none() {
        return card.kind;
    }
    let mods = state.modifiers.for_card_name(
        state,
        player,
        card,
        effective_card_suit(state, player, card),
    );
    let (tracked, owned): (Vec<_>, Vec<_>) = mods.into_iter().partition(|modifier| {
        matches!(
            modifier.target,
            ModifierTarget::CardName(CardSelector::SkillTracked)
        )
    });
    let (value, _, tracked_vetoed) = apply_int_modifiers(card.kind.tag() as i32, &tracked, None);
    let (value, _, owned_vetoed) = apply_int_modifiers(value, &owned, Some(player));
    let vetoed = tracked_vetoed || owned_vetoed;
    if vetoed {
        return card.kind;
    }
    u8::try_from(value)
        .ok()
        .and_then(CardKind::from_tag)
        .unwrap_or(card.kind)
}

/// Damage nature seen for one card used by a player; the physical card remains unchanged.
pub fn effective_card_nature(
    state: &GameState,
    player: PlayerId,
    card: &Card,
    logical_kind: CardKind,
) -> u8 {
    if player.is_none() {
        return logical_kind.sha_nature();
    }
    let mods = state
        .modifiers
        .for_card_nature(logical_kind, effective_card_suit(state, player, card));
    let base = i32::from(logical_kind.sha_nature());
    let (value, _, vetoed) = apply_int_modifiers(base, &mods, Some(player));
    if vetoed {
        return logical_kind.sha_nature();
    }
    u8::try_from(value).unwrap_or_else(|_| logical_kind.sha_nature())
}

/// Check a fixed card target-distance limit through the shared modifier surface.
pub fn card_target_within_distance(
    state: &GameState,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
    limit: i32,
) -> bool {
    if card_distance_unlimited_for_target(state, user, card, target) {
        return true;
    }
    let Some(card_state) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, user, card_state);
    let limit_mods = state
        .modifiers
        .for_card_distance_limit(kind, effective_card_suit(state, user, card_state));
    let (limit, _, vetoed) = apply_int_modifiers(limit, &limit_mods, Some(user));
    if vetoed {
        return false;
    }
    matches!(
        query(
            state,
            &QueryRequest::Distance {
                from: user,
                to: target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(distance) if distance <= limit.max(0)
    )
}

/// Whether the player has a content-defined draw-replacement damage offer.
pub fn has_draw_replacement(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DrawReplacementDamage);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Content-defined prepare-phase scry count, capped by alive players.
pub fn prepare_scry_count(state: &GameState, player: PlayerId) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::PrepareScryCount);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    if vetoed {
        0
    } else {
        (value.max(0) as u32).min(state.players.alive_by_seat().len() as u32)
    }
}

/// Whether package content permits this player to rewrite a judgment.
pub fn can_rewrite_judgment(state: &GameState, player: PlayerId) -> bool {
    judgment_rewrite_unrestricted(state, player)
        || judgment_rewrite_red_hand_equip_expansion(state, player)
        || state
            .modifiers
            .for_target(ModifierTarget::JudgmentRewriteBlackHandEquip)
            .iter()
            .any(|modifier| {
                modifier.owner == player
                    && !crate::query::judgment_black_rewrite_cards(state, player).is_empty()
            })
}

/// Black hand/equipment cards eligible for a 鬼道 rewrite.
fn judgment_black_rewrite_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let in_zone = |card: &Card| {
        let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
        let in_equip = state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
        (in_hand || in_equip)
            && matches!(
                effective_card_suit(state, player, card),
                Suit::Spade | Suit::Club
            )
    };
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.owner == player && in_zone(card))
        .map(|card| card.id)
        .collect()
}

fn judgment_rewrite_unrestricted(state: &GameState, player: PlayerId) -> bool {
    let hand_mods = state.modifiers.for_target(ModifierTarget::JudgmentRewrite);
    let hand_equip_mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteHandEquip);
    let (hand_value, _, hand_vetoed) = apply_int_modifiers(0, &hand_mods, Some(player));
    let (hand_equip_value, _, hand_equip_vetoed) =
        apply_int_modifiers(0, &hand_equip_mods, Some(player));
    let paid_hand_equip = judgment_rewrite_mark_cost(state, player).is_some_and(|mark| {
        state
            .players
            .get(player)
            .into_iter()
            .flat_map(|player| player.marks.iter())
            .any(|(definition, count)| *definition == mark && *count > 0)
    });
    (!hand_vetoed && hand_value > 0)
        || (!hand_equip_vetoed && hand_equip_value > 0)
        || paid_hand_equip
}

fn judgment_rewrite_red_hand_equip_expansion(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteRedHandEquipExpansion);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether judgment rewrite may also consume one of the player's equipment cards.
pub fn judgment_rewrite_includes_equip(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteHandEquip);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    (!vetoed && value > 0)
        || judgment_rewrite_mark_cost(state, player).is_some()
        || judgment_rewrite_red_hand_equip_expansion(state, player)
}

/// Whether one owned personal-zone card may currently replace a judgment.
pub(crate) fn judgment_rewrite_card_matches(
    state: &GameState,
    player: PlayerId,
    card: &Card,
) -> bool {
    if card.owner != player {
        return false;
    }
    let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = state.zones.find(ZoneKind::Expansion, player) == Some(card.zone);
    let hand_mods = state.modifiers.for_target(ModifierTarget::JudgmentRewrite);
    let hand_equip_mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteHandEquip);
    let (hand, _, hand_vetoed) = apply_int_modifiers(0, &hand_mods, Some(player));
    let (hand_equip, _, hand_equip_vetoed) = apply_int_modifiers(0, &hand_equip_mods, Some(player));
    let black_hand_equip_mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteBlackHandEquip);
    let (black_hand_equip, _, black_hand_equip_vetoed) =
        apply_int_modifiers(0, &black_hand_equip_mods, Some(player));
    let free = (in_hand && !hand_vetoed && hand > 0)
        || ((in_hand || in_equip) && !hand_equip_vetoed && hand_equip > 0)
        || ((in_hand || in_equip)
            && !black_hand_equip_vetoed
            && black_hand_equip > 0
            && matches!(
                effective_card_suit(state, player, card),
                Suit::Spade | Suit::Club
            ))
        || ((in_hand || in_equip || in_expansion)
            && judgment_rewrite_red_hand_equip_expansion(state, player)
            && matches!(
                effective_card_suit(state, player, card),
                Suit::Heart | Suit::Diamond
            ));
    let paid = (in_hand || in_equip)
        && judgment_rewrite_mark_cost(state, player).is_some_and(|mark| {
            state
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.marks.iter())
                .any(|(definition, count)| *definition == mark && *count > 0)
        });
    free || paid
}

/// Mark consumed by this exact rewrite card, if it is legal only through a paid channel.
pub(crate) fn judgment_rewrite_mark_cost_for_card(
    state: &GameState,
    player: PlayerId,
    card: &Card,
) -> Option<u32> {
    let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let red_free = judgment_rewrite_red_hand_equip_expansion(state, player)
        && matches!(
            effective_card_suit(state, player, card),
            Suit::Heart | Suit::Diamond
        );
    let hand_free = in_hand && {
        let mods = state.modifiers.for_target(ModifierTarget::JudgmentRewrite);
        let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
        !vetoed && value > 0
    };
    let hand_equip_free = (in_hand || in_equip) && {
        let mods = state
            .modifiers
            .for_target(ModifierTarget::JudgmentRewriteHandEquip);
        let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
        !vetoed && value > 0
    };
    if hand_free || hand_equip_free || red_free {
        None
    } else {
        judgment_rewrite_mark_cost(state, player)
    }
}

/// Mark consumed when this player successfully rewrites a judgment, if any.
pub fn judgment_rewrite_mark_cost(state: &GameState, player: PlayerId) -> Option<u32> {
    for target in [
        ModifierTarget::JudgmentRewrite,
        ModifierTarget::JudgmentRewriteHandEquip,
    ] {
        let mods = state.modifiers.for_target(target);
        let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
        if !vetoed && value > 0 {
            return None;
        }
    }
    state.modifiers.judgment_rewrite_cost_mark(player)
}

/// Whether the player's judgment should use the most favorable card in the draw pile.
pub fn judgment_uses_best_draw_pile_card(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentBestFromDrawPile);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Required response-card count after source-owned content modifiers.
pub fn response_count_for(state: &GameState, source: PlayerId, base_count: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ResponseCount);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_count as i32, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Required Sha count for one Duel exchange after modifiers owned by the responder's opponent.
pub fn duel_response_count_for(
    state: &GameState,
    responder_opponent: PlayerId,
    base_count: u32,
) -> u32 {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DuelOpponentResponseCount);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_count as i32, &mods, Some(responder_opponent));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Required Shan count for one Sha target after source-owned general modifiers and modifiers
/// whose owner participates opposite a female character.
pub fn sha_response_count_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    base_count: u32,
) -> u32 {
    let base_count = response_count_for(state, source, base_count);
    let has_female_sex = |player: PlayerId| {
        state
            .players
            .get(player)
            .filter(|player| player.alive)
            .and_then(|player| state.content.character(player.character_def))
            .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
    };
    let modifiers: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaResponseCountIfOtherFemale)
        .into_iter()
        .filter(|modifier| {
            (modifier.owner == source && has_female_sex(target))
                || (modifier.owner == target && has_female_sex(source))
        })
        .collect();
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_count as i32, &modifiers, None);
    let nearby_modifiers: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaResponseCountIfSourceOwnerTargetDistanceLeOne)
        .into_iter()
        .filter(|modifier| {
            modifier.owner == source
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: source,
                            to: target,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        })
        .collect();
    let (value, _, nearby_vetoed) =
        apply_int_modifiers_for_state(state, value, &nearby_modifiers, None);
    if vetoed || nearby_vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Damage received from a logical Sha after target-owned Sha-specific modifiers.
pub fn sha_damage_taken_for(state: &GameState, target: PlayerId, base_amount: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ShaDamageTaken);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_amount as i32, &mods, Some(target));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Sha damage after source-owned modifiers that require being outside the target's attack range.
pub fn sha_damage_outside_target_range_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    base_amount: u32,
) -> u32 {
    let (distance, range) = match (
        distance_explain(state, target, source).response,
        attack_range_explain(state, target).response,
    ) {
        (QueryResponse::Int(distance), QueryResponse::Int(range)) => (distance, range),
        _ => return base_amount,
    };
    if distance <= range {
        return base_amount;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaDamageIfOutsideTargetRange);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_amount as i32, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

fn sha_damage_source_range_threshold(state: &GameState, source: PlayerId) -> Option<(i32, i32)> {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaDamageSourceRangeThreshold);
    let (threshold, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    if !vetoed && threshold <= 0 {
        return None;
    }
    let range = match attack_range_explain(state, source).response {
        QueryResponse::Int(range) => range,
        _ => return None,
    };
    Some((range, threshold))
}

/// Early source-range bonus for logical Sha damage.
pub fn sha_damage_source_range_bonus_for(
    state: &GameState,
    source: PlayerId,
    base_amount: u32,
) -> u32 {
    if sha_damage_source_range_threshold(state, source)
        .is_some_and(|(range, threshold)| range > threshold)
    {
        base_amount.saturating_add(1)
    } else {
        base_amount
    }
}

/// Late source-range cap for logical Sha damage.
pub fn sha_damage_source_range_cap_for(
    state: &GameState,
    source: PlayerId,
    base_amount: u32,
) -> u32 {
    if base_amount > 1
        && sha_damage_source_range_threshold(state, source)
            .is_some_and(|(range, threshold)| range < threshold)
    {
        1
    } else {
        base_amount
    }
}

/// Damage received after target-owned amount modifiers for the resolved nature.
pub fn damage_taken_amount_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
    base_amount: u32,
) -> u32 {
    let range_mods = state
        .modifiers
        .for_target(ModifierTarget::DamageTakenSourceRangeThreshold);
    let (threshold, _, range_vetoed) =
        apply_int_modifiers_for_state(state, 0, &range_mods, Some(target));
    let source_range = if source.is_some() && (range_vetoed || threshold > 0) {
        match attack_range_explain(state, source).response {
            QueryResponse::Int(range) => Some(range),
            _ => None,
        }
    } else {
        None
    };
    let base_amount = if source_range.is_some_and(|range| range > threshold) {
        base_amount.saturating_add(1)
    } else {
        base_amount
    };
    let mods = state
        .modifiers
        .for_damage_taken_amount(nature, target, |owner| {
            state.players.get(owner).is_some_and(|player| player.alive)
        });
    let (mut value, _, vetoed) = apply_int_modifiers_for_state_with_context(
        state,
        base_amount as i32,
        &mods,
        None,
        Some(target),
        Some(source),
    );
    if vetoed {
        return 0;
    }
    if !armor_effect_ignored_for(state, source, target) {
        let mods = state.modifiers.for_armor_damage_taken_amount(nature);
        let (armor_value, _, vetoed) =
            apply_int_modifiers_for_state(state, value, &mods, Some(target));
        if vetoed {
            return 0;
        }
        value = armor_value;
    }
    if value > 1 && source_range.is_some_and(|range| range < threshold) {
        1
    } else {
        value.max(0) as u32
    }
}

/// Final damage received after equipment and ordinary target-owned amount modifiers.
pub fn damage_taken_final_amount_for(
    state: &GameState,
    target: PlayerId,
    nature: u8,
    base_amount: u32,
) -> u32 {
    let mods = state
        .modifiers
        .for_damage_taken_final_amount(nature, target, |owner| {
            state.players.get(owner).is_some_and(|player| player.alive)
        });
    let (value, _, vetoed) = apply_int_modifiers_for_state_with_context(
        state,
        base_amount as i32,
        &mods,
        None,
        Some(target),
        None,
    );
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Resolved damage nature after target-owned nature modifiers.
pub fn damage_taken_nature_for(state: &GameState, target: PlayerId, base_nature: u8) -> u8 {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DamageTakenNature);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_nature as i32, &mods, Some(target));
    if vetoed {
        base_nature
    } else {
        value.clamp(0, u8::MAX as i32) as u8
    }
}

/// Maximum number of distinct targets one proactive 杀 may select.
pub fn sha_target_limit_for(state: &GameState, source: PlayerId) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ShaTargetLimit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 1, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Maximum number of independently resolved targets for one concrete card use.
pub fn card_target_limit_for_card(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
    base: u32,
) -> u32 {
    let Some(card) = state.cards.get(card) else {
        return base;
    };
    let effective_kind = effective_card_kind(state, source, card);
    card_target_limit(
        state,
        source,
        card.kind,
        effective_kind,
        effective_card_suit(state, source, card),
        base,
    )
}

/// Maximum target count for a source-free logical card use.
pub fn card_target_limit_for_kind(
    state: &GameState,
    source: PlayerId,
    kind: CardKind,
    base: u32,
) -> u32 {
    card_target_limit(state, source, kind, kind, engine_api::Suit::None, base)
}

fn card_target_limit(
    state: &GameState,
    source: PlayerId,
    physical_kind: CardKind,
    effective_kind: CardKind,
    effective_suit: engine_api::Suit,
    base: u32,
) -> u32 {
    let mods = state
        .modifiers
        .for_card_target_limit(physical_kind, effective_kind, effective_suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, base as i32, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Whether one proactive 杀 must target every otherwise-legal player.
pub fn sha_targets_all_for(state: &GameState, source: PlayerId) -> bool {
    let mods = state.modifiers.for_card_targets_all(CardSelector::Sha);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(source));
    vetoed || value != 0
}

/// Sha uses charged against the current Play phase's allowance.
///
/// ABI-44 content separates this bounded phase counter from the turn-wide Sha history used by
/// skills such as Keji. Older content keeps the original single-counter state and hashes.
pub fn sha_play_phase_use_count(state: &GameState) -> u32 {
    let key = if state.content.tracks_opening_extra_play_lifecycle() {
        crate::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE
    } else {
        crate::state::turn_flag::SHA_PLAYED
    };
    state.turn.flag(key).max(0) as u32
}

/// Whether cards used by `source` may only target that same player.
pub fn card_targets_self_only_for(state: &GameState, source: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardTargetsSelfOnly);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(source));
    vetoed || value != 0
}

/// Whether content policy skips this player's discard phase after a no-杀 play phase.
pub fn skip_discard_if_no_sha(state: &GameState, player: PlayerId) -> bool {
    if state.turn.flag(crate::state::turn_flag::SHA_PLAYED) != 0 {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::SkipDiscardIfNoSha);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether content policy skips this player's judgment phase.
pub fn skip_judge_phase(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::SkipJudgePhase);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether content policy skips this player's discard phase unconditionally at query time.
pub fn skip_discard_phase(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::SkipDiscardPhase);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether a skipped or cancelled phase is restored for this player after its audit event.
pub fn restore_skipped_phase(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::RestoreSkippedPhase);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Black non-trick hand/equipment cards that can pay 3D精选丁奉「短兵」.
pub(crate) fn ddd_duanbing_cost_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    if !ddd_duanbing_can_receive_bingliang(state, player) {
        return Vec::new();
    }
    let hand = state.zones.find(ZoneKind::Hand, player);
    let equip = state.zones.find(ZoneKind::Equip, player);
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| Some(card.zone) == hand || Some(card.zone) == equip)
        .filter(|card| effective_card_color_mask(state, player, card.id) == 0b10)
        .filter(|card| !effective_card_kind(state, player, card).is_trick())
        .map(|card| card.id)
        .collect()
}

/// Owner judgment cards moved by 3D精选丁奉「短兵」 after card damage.
pub(crate) fn ddd_duanbing_transfer_cards(
    state: &GameState,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    if !ddd_duanbing_can_receive_bingliang(state, target) {
        return Vec::new();
    }
    let Some(judge) = state.zones.find(ZoneKind::Judge, owner) else {
        return Vec::new();
    };
    state
        .zones
        .get(judge)
        .into_iter()
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == CardKind::TrickBingliang)
        })
        .collect()
}

fn ddd_duanbing_can_receive_bingliang(state: &GameState, player: PlayerId) -> bool {
    if state.disabled_judge_zones.contains(&player)
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    state
        .zones
        .find(ZoneKind::Judge, player)
        .and_then(|zone| state.zones.get(zone))
        .is_some_and(|zone| {
            zone.cards.iter().all(|card| {
                state
                    .cards
                    .get(*card)
                    .is_none_or(|card| card.kind != CardKind::TrickBingliang)
            })
        })
}

/// Whether a face-up player is protected from being turned face-down.
pub fn turn_face_down_cancelled(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::TurnFaceDownCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether content policy prevents either face-up/face-down transition.
pub fn turn_over_cancelled(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::TurnOverCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether the ordinary discard phase counts and may discard the owner's equipment cards.
pub fn discard_phase_includes_equip(state: &GameState, player: PlayerId) -> bool {
    let mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::DiscardPhaseIncludesEquip)
        .into_iter()
        .filter(|modifier| {
            modifier.owner == player
                || modifier.value_source
                    == Some(ModifierValueSource::QueriedPlayerLinkedOtherThanOwner)
        })
        .collect();
    let (value, _, vetoed) =
        apply_int_modifiers_for_state_with_context(state, 0, &mods, None, Some(player), None);
    !vetoed && value > 0
}

/// Whether this seated observer may see the faces of cards in other players' hands.
pub fn observer_foreign_hand_faces_visible(state: &GameState, observer: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ObserverForeignHandFacesVisible);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(observer));
    !vetoed && value > 0
}

/// Whether this seated observer may see identities that are otherwise private.
pub fn observer_hidden_identities_visible(state: &GameState, observer: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ObserverHiddenIdentitiesVisible);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(observer));
    !vetoed && value > 0
}

/// Whether the player may perform the optional four-card draw-phase reorder.
pub fn has_draw_phase_scry_four_split_two(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DrawPhaseScryFourSplitTwo);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Initial hand size after package modifiers.
pub fn initial_hand_count_for(state: &GameState, player: PlayerId, base: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::InitialHandCount);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, base as i32, &mods, Some(player));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Whether `target` is immune to being chosen by a trick of the given
/// immunity channel (e.g. 谦逊 → 顺手牵羊 / 乐不思蜀). `channel` is one of
/// [`ModifierTarget::ImmuneShunshou`] / [`ModifierTarget::ImmuneLe`].
pub fn target_immune(state: &GameState, target: PlayerId, channel: ModifierTarget) -> bool {
    let mods = state.modifiers.for_target(channel);
    let (value, _, _) = apply_int_modifiers(0, &mods, Some(target));
    value > 0
}

fn relation(state: &GameState, a: PlayerId, b: PlayerId) -> QueryResponse {
    if a == b {
        return QueryResponse::Relation(RelationTag::Self_);
    }
    let Some(pa) = state.players.get(a) else {
        return QueryResponse::Error {
            code: QueryErrorCode::InvalidId,
        };
    };
    let Some(pb) = state.players.get(b) else {
        return QueryResponse::Error {
            code: QueryErrorCode::InvalidId,
        };
    };
    use crate::state::Identity::*;
    let tag = match (pa.identity, pb.identity) {
        (Lord, Loyalist) | (Loyalist, Lord) | (Loyalist, Loyalist) | (Lord, Lord) => {
            RelationTag::Ally
        }
        (Rebel, Rebel) => RelationTag::Ally,
        (Renegade, _) | (_, Renegade) => RelationTag::Enemy,
        _ => RelationTag::Enemy,
    };
    QueryResponse::Relation(tag)
}

fn card_usable_explain(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> QueryExplanation {
    let Some(p) = state.players.get(player) else {
        return err(QueryErrorCode::InvalidId);
    };
    if !p.alive || p.dying {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    if state.turn.phase != Phase::Play || state.turn.current != player {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    let Some(c) = state.cards.get(card) else {
        return err(QueryErrorCode::InvalidId);
    };
    let hand = state.zones.find(ZoneKind::Hand, player);
    if hand != Some(c.zone) || c.owner != player {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    if card_use_cancelled_for_card(state, player, card) {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: Some(0),
            sources: vec![],
        };
    }
    let kind = effective_card_kind(state, player, c);
    let ok = match kind {
        CardKind::Ying => false,
        CardKind::Shan => false,
        // 无懈：仅响应窗口打出
        CardKind::TrickWuxie | CardKind::TrickYoudishenru => false,
        CardKind::Tao => {
            p.hp < p.max_hp as i32
                && !card_target_cancelled_for_logical_kind(state, player, player, c, CardKind::Tao)
        }
        // 酒：满血也可出（获得 JIU mark）
        CardKind::Jiu => true,
        CardKind::Sha
        | CardKind::ShaFire
        | CardKind::ShaThunder
        | CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorseOff
        | CardKind::HorseDef
        | CardKind::Treasure
        | CardKind::TrickWuzhong
        | CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickJuedou
        | CardKind::TrickNanman
        | CardKind::TrickWanjian
        | CardKind::TrickTaoyuan
        | CardKind::TrickWugu
        | CardKind::TrickLe
        | CardKind::TrickBingliang
        | CardKind::TrickShandian
        | CardKind::TrickJiedao
        | CardKind::TrickTiesuo
        | CardKind::TrickHuogong
        | CardKind::TrickYuanjiao
        | CardKind::TrickTiaojiyanmei
        | CardKind::TrickMantianguohai
        | CardKind::TrickDiaobingqianjiang
        | CardKind::TrickWangmeizhike
        | CardKind::TrickYiyi
        | CardKind::TrickShuiyanqijunx
        | CardKind::TrickLulitongxin
        | CardKind::TrickKefuzhongyuan
        | CardKind::TrickGuguoanbang
        | CardKind::TrickChuqibuyi
        | CardKind::TrickHuoshaolianying
        | CardKind::TrickLianjunshengyan
        | CardKind::TrickWenheluanwu
        | CardKind::TrickHaolingtianxia
        | CardKind::TrickChiling
        | CardKind::Other => true,
    };
    QueryExplanation {
        response: QueryResponse::Bool(ok),
        base: None,
        sources: vec![],
    }
}

fn target_legal_explain(
    state: &GameState,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
    ctx: &mut QueryContext,
) -> QueryExplanation {
    let Some(c) = state.cards.get(card) else {
        return err(QueryErrorCode::InvalidId);
    };
    let Some(t) = state.players.get(target) else {
        return err(QueryErrorCode::InvalidId);
    };
    if !t.alive {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    // 雄乱（张绣）：本回合使用者只能对限定目标使用牌。
    let restricted = state
        .turn
        .flag(crate::state::turn_flag::xiongluan_target_by_player(user))
        .max(0) as u32;
    if restricted != 0 && target.raw() != restricted {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    let kind = effective_card_kind(state, user, c);
    if card_target_cancelled_for_logical_kind(state, user, target, c, kind) {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    match kind {
        CardKind::Sha
        | CardKind::ShaFire
        | CardKind::ShaThunder
        | CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickJuedou
        | CardKind::TrickHuogong => {
            if target == user && kind.is_sha() {
                return QueryExplanation {
                    response: QueryResponse::Bool(false),
                    base: None,
                    sources: vec![],
                };
            }
            if target == user
                && matches!(
                    kind,
                    CardKind::TrickGuohe | CardKind::TrickShunshou | CardKind::TrickJuedou
                )
            {
                return QueryExplanation {
                    response: QueryResponse::Bool(false),
                    base: None,
                    sources: vec![],
                };
            }
            // 火攻可指定自己（有手牌时）
            if kind.is_sha() {
                if card_distance_unlimited_for_target(state, user, card, target) {
                    return QueryExplanation {
                        response: QueryResponse::Bool(true),
                        base: None,
                        sources: vec![],
                    };
                }
                let dist = query(
                    state,
                    &QueryRequest::Distance {
                        from: user,
                        to: target,
                    },
                    ctx,
                );
                let range = query(state, &QueryRequest::AttackRange { player: user }, ctx);
                let ok = matches!(
                    (dist, range),
                    (QueryResponse::Int(d), QueryResponse::Int(r)) if d <= r
                );
                return QueryExplanation {
                    response: QueryResponse::Bool(ok),
                    base: None,
                    sources: vec![],
                };
            }
            if kind == CardKind::TrickShunshou {
                return QueryExplanation {
                    response: QueryResponse::Bool(card_target_within_distance(
                        state, user, card, target, 1,
                    )),
                    base: None,
                    sources: vec![],
                };
            }
            if kind == CardKind::TrickHuogong {
                let alive = state.players.get(target).map(|p| p.alive).unwrap_or(false);
                let has_hand = !state.hand_cards(target).is_empty();
                return QueryExplanation {
                    response: QueryResponse::Bool(alive && has_hand),
                    base: None,
                    sources: vec![],
                };
            }
            QueryExplanation {
                response: QueryResponse::Bool(
                    state.players.get(target).map(|p| p.alive).unwrap_or(false),
                ),
                base: None,
                sources: vec![],
            }
        }
        CardKind::TrickLe | CardKind::TrickBingliang => {
            let ok = target != user
                && t.alive
                && (kind != CardKind::TrickBingliang
                    || card_target_within_distance(state, user, card, target, 1));
            QueryExplanation {
                response: QueryResponse::Bool(ok),
                base: None,
                sources: vec![],
            }
        }
        CardKind::Tao | CardKind::Jiu => QueryExplanation {
            response: QueryResponse::Bool(target == user),
            base: None,
            sources: vec![],
        },
        _ => QueryExplanation {
            response: QueryResponse::Bool(true),
            base: None,
            sources: vec![],
        },
    }
}

fn skill_uses_explain(state: &GameState, skill: engine_api::SkillId) -> QueryExplanation {
    let Some(s) = state.skills.get(skill) else {
        return err(QueryErrorCode::InvalidId);
    };
    // Phase 3: unlimited unless state_blob encodes max; report 99 - use_count
    let remaining = 99i32.saturating_sub(s.use_count as i32);
    QueryExplanation {
        response: QueryResponse::Int(remaining.max(0)),
        base: Some(99),
        sources: vec![],
    }
}

fn card_attr_explain(state: &GameState, card: engine_api::CardId, attr: u32) -> QueryExplanation {
    let Some(c) = state.cards.get(card) else {
        return err(QueryErrorCode::InvalidId);
    };
    // attr 0 = kind tag, 1 = rank, 2 = suit tag
    let base = match attr {
        0 => c.kind.tag() as i32,
        1 => c.rank as i32,
        2 => match c.suit {
            engine_api::Suit::None => 0,
            engine_api::Suit::Spade => 1,
            engine_api::Suit::Heart => 2,
            engine_api::Suit::Club => 3,
            engine_api::Suit::Diamond => 4,
        },
        _ => 0,
    };
    let mods = state.modifiers.for_target(ModifierTarget::CardAttr(attr));
    // Card mods: no owner filter
    let (value, sources, _) = apply_int_modifiers(base, &mods, None);
    QueryExplanation {
        response: QueryResponse::Int(value),
        base: Some(base),
        sources,
    }
}

fn err(code: QueryErrorCode) -> QueryExplanation {
    QueryExplanation {
        response: QueryResponse::Error { code },
        base: None,
        sources: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;
    use crate::transaction::Transaction;
    use crate::trigger::{BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
    use engine_api::{CardDefId, SkillDefId, SkillId, ZoneId};
    use rule_api::EffectRequest;

    #[test]
    fn cache_on_off_same_result() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 3, 1, "bootstrap").unwrap();
        let state = tx.state().clone();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let req = QueryRequest::Distance { from: p1, to: p2 };

        let mut ctx_off = QueryContext::with_cache(false);
        let a = query(&state, &req, &mut ctx_off);

        let mut ctx_on = QueryContext::with_cache(true);
        let b1 = query(&state, &req, &mut ctx_on);
        let b2 = query(&state, &req, &mut ctx_on);
        assert_eq!(a, b1);
        assert_eq!(b1, b2);
    }

    #[test]
    fn exact_number_subsets_bound_unsatisfiable_large_hand() {
        let mut state = GameState::empty();
        let cards: Vec<_> = (0..64)
            .map(|_| {
                state.cards.alloc(Card {
                    id: CardId::NONE,
                    def: CardDefId::NONE,
                    kind: CardKind::Sha,
                    suit: Suit::Spade,
                    rank: 1,
                    weapon_range: 0,
                    zone: ZoneId::NONE,
                    owner: PlayerId::NONE,
                    face_up: true,
                })
            })
            .collect();

        assert!(
            effective_number_sum_subsets(&state, PlayerId::NONE, &cards, 1, 12, 13, 64,).is_empty()
        );
    }

    #[test]
    fn unique_max_current_hp_requires_one_strictly_highest_living_player() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 3, 1, "bootstrap").unwrap();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let p3 = PlayerId::new(3);
        tx.state_mut().players.get_mut(p1).unwrap().hp = 3;
        tx.state_mut().players.get_mut(p2).unwrap().hp = 3;
        tx.state_mut().players.get_mut(p3).unwrap().hp = 2;

        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp { player: p1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(false)
        );

        tx.state_mut().players.get_mut(p2).unwrap().hp = 1;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp { player: p1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(true)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHpAt {
                    player: p2,
                    value: 4,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(true)
        );

        tx.state_mut().players.get_mut(p1).unwrap().alive = false;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp { player: p1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(false)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp {
                    player: PlayerId::new(99),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }
        );
    }

    #[test]
    fn lifetime_damage_dealt_is_saturating_and_validates_player() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        let player = PlayerId::new(1);

        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::LifetimeDamageDealt { player },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(0)
        );
        tx.state_mut().damage_dealt_totals.insert(player, u32::MAX);
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::LifetimeDamageDealt { player },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(i32::MAX)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::LifetimeDamageDealt {
                    player: PlayerId::new(99),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }
        );
    }

    #[test]
    fn public_card_kind_count_reads_only_draw_and_discard() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        let player = PlayerId::new(1);
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let card = tx
            .state()
            .zones
            .get(draw)
            .unwrap()
            .cards
            .iter()
            .copied()
            .find(|card| tx.state().cards.get(*card).unwrap().kind != CardKind::Tao)
            .unwrap();
        let before = public_card_kind_count(tx.state(), CardKind::Tao.tag()).unwrap();

        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Tao;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::PublicCardKindCount {
                    kind: CardKind::Tao.tag(),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(before + 1)
        );

        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .retain(|candidate| *candidate != card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        let card_state = tx.state_mut().cards.get_mut(card).unwrap();
        card_state.zone = hand;
        card_state.owner = player;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::PublicCardKindCount {
                    kind: CardKind::Tao.tag(),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(before)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::PublicCardKindCount { kind: u8::MAX - 1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }
        );
    }

    #[test]
    fn mashu_reduces_distance_explain() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 4, 1, "bootstrap").unwrap();
        // Give p1 马术
        let p1 = PlayerId::new(1);
        let sid = tx.state_mut().skills.alloc(crate::state::SkillInstance {
            id: SkillId::NONE,
            def: SkillDefId::new(BOOTSTRAP_MASHU),
            owner: p1,
            enabled: true,
            use_count: 0,
            turn_fire_count: 0,
            content_flags: 0,
            play_phase_fire_count: 0,
            play_phase_use_count: 0,
            play_phase_category_mask: 0,
            active_turn_banned: false,
            turn_disabled: false,
            disabled_by_skills: Vec::new(),
            temporary_until_turn_end_of: PlayerId::NONE,
            round_fire_round: 0,
            lifetime_fire_count: 0,
            view_as_branch: 0,
            mark: 0,
            view_as_use_round: 0,
            view_as_used_branches: 0,
            state_blob: vec![],
            source_free_physical_card: CardId::NONE,
            general_slot: crate::state::GENERAL_SLOT_NONE,
            content_storage: std::collections::BTreeMap::new(),
        });
        tx.state_mut().players.get_mut(p1).unwrap().skills.push(sid);
        tx.state_mut().modifiers.register_skill(
            sid,
            SkillDefId::new(BOOTSTRAP_MASHU),
            p1,
            &crate::content::ContentRegistry::new(),
        );

        let p3 = PlayerId::new(3);
        let mut ctx = QueryContext::new();
        let base_only = {
            // without would be seat distance
            base_distance(tx.state(), p1, p3).unwrap()
        };
        let exp = query_explain(
            tx.state(),
            &QueryRequest::Distance { from: p1, to: p3 },
            &mut ctx,
        );
        assert_eq!(exp.base, Some(base_only));
        assert!(!exp.sources.is_empty());
        if let QueryResponse::Int(v) = exp.response {
            assert!(v <= base_only);
            assert!(v >= 1);
        } else {
            panic!("expected int");
        }
        let _ = EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        };
    }
}

/// 缓存边界（ADR 0703 第②部分）：依赖版本覆盖结算内单调哨兵，
/// 防止「revision 未变但状态已变」的脏命中。
#[cfg(test)]
mod cache_boundary_tests {
    use super::*;
    use crate::engine::Engine;
    use crate::engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};

    fn started_engine() -> Engine {
        let mut e = Engine::new();
        e.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 42,
                ruleset: "identity-standard".into(),
            },
        });
        e
    }

    /// 依赖版本哨兵：revision / 伤害序号 / 恢复序号 / 选择序号 / RNG 调用数
    /// 任一推进都必须改变依赖版本。
    #[test]
    fn dep_version_senses_intra_settlement_sentinels() {
        let mut e = started_engine();
        let state = e.state_mut_for_tests();
        let v0 = state.query_dep_version();
        state.resolution.next_damage_seq += 1;
        assert_ne!(state.query_dep_version(), v0, "伤害序号推进应改变依赖版本");
        state.resolution.next_recovery_seq += 1;
        assert_ne!(state.query_dep_version(), v0, "恢复序号推进应改变依赖版本");
        state.resolution.next_choice_seq += 1;
        assert_ne!(state.query_dep_version(), v0, "选择序号推进应改变依赖版本");
        state.rng.call_count += 1;
        assert_ne!(state.query_dep_version(), v0, "RNG 消耗应改变依赖版本");
        state.revision += 1;
        assert_ne!(state.query_dep_version(), v0, "提交版本推进应改变依赖版本");
    }

    /// 缓存脏读修复：结算内推进哨兵后，同键查询不得命中旧缓存。
    #[test]
    fn cached_query_recomputes_after_intra_settlement_progress() {
        let mut e = started_engine();
        let p1 = PlayerId::new(1);
        let mut ctx = QueryContext::with_cache(true);

        let state = e.state();
        let r1 = query_explain(state, &QueryRequest::Hp { player: p1 }, &mut ctx);
        assert_eq!(ctx.cache_hits(), 0, "首次查询未命中");

        let state = e.state();
        let r2 = query_explain(state, &QueryRequest::Hp { player: p1 }, &mut ctx);
        assert_eq!(ctx.cache_hits(), 1, "同键二次查询应命中缓存");
        assert_eq!(r1.response, r2.response);

        // 模拟结算内推进（revision 不变）：伤害序号 +1
        e.state_mut_for_tests().resolution.next_damage_seq += 1;

        let state = e.state();
        let _r3 = query_explain(state, &QueryRequest::Hp { player: p1 }, &mut ctx);
        assert_eq!(
            ctx.cache_hits(),
            1,
            "依赖版本推进后不得命中旧缓存（脏读边界）"
        );
    }
}

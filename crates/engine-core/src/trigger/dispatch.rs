//! Collect and plan trigger execution for domain events.

use super::catalog::{TriggerDefId, TriggerFilter, TriggerHandler};
use super::index::ActiveTrigger;
use super::{seat_rel_to_current, EventKind, TimingBand, TriggerOrderKey};
use crate::effect::{hand_compare_result_card, EffectError};
use crate::query::{query, QueryContext};
use crate::state::{turn_flag, CardKind, EquipSlot, Identity, Phase, ZoneKind};
use crate::transaction::Transaction;
use crate::vm_host::run_module_with_event_cards;
use content_schema::TriggerChoiceActor;
use engine_api::{CardId, CardMoveReason, DomainEvent, PlayerId, SkillDefId, SkillId};
use rule_api::{EffectRequest, QueryRequest, QueryResponse, RelationTag};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

const CHARACTER_STATE_TURNED_FACE_UP: u32 = 1;
const CHARACTER_STATE_TURNED_FACE_DOWN: u32 = 2;
const CHARACTER_STATE_BECAME_LINKED: u32 = 3;
const CHARACTER_STATE_BECAME_UNLINKED: u32 = 4;

const DDD_ZHENGJUN_HP: u8 = 1;
const DDD_ZHENGJUN_HAND: u8 = 2;
const DDD_ZHENGJUN_EQUIP: u8 = 4;

fn ddd_shixing_female_available(state: &crate::state::GameState) -> bool {
    state.players.alive_by_seat().into_iter().any(|player| {
        state.players.get(player).is_some_and(|participant| {
            participant.hp >= i32::try_from(participant.max_hp).unwrap_or(i32::MAX)
                && state
                    .content
                    .character(participant.character_def)
                    .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
        })
    })
}

fn ddd_player_linked(state: &crate::state::GameState, player: PlayerId) -> bool {
    state.players.get(player).is_some_and(|participant| {
        participant
            .marks
            .iter()
            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    })
}

fn ddd_zhangkai_unique_max_equip(state: &crate::state::GameState) -> Option<PlayerId> {
    let counts: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| {
            let count = state
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| state.zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            (player, count)
        })
        .collect();
    let max = counts.iter().map(|(_, count)| *count).max()?;
    let mut leaders = counts.into_iter().filter(|(_, count)| *count == max);
    let leader = leaders.next()?.0;
    leaders.next().is_none().then_some(leader)
}

fn ddd_zhangkai_can_take_equipment(
    state: &crate::state::GameState,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
) -> bool {
    let processing = state.zones.find(ZoneKind::Processing, PlayerId::NONE);
    let actor_hand = state.zones.find(ZoneKind::Hand, actor);
    state.cards.get(card).is_some_and(|card| {
        (Some(card.zone) == processing || Some(card.zone) == actor_hand)
            && card.kind.equip_slot().is_some_and(|slot| {
                state
                    .players
                    .get(player)
                    .is_some_and(|owner| owner.equip_slot_enabled(slot))
            })
    })
}

fn ddd_zhengjun_move_mask(state: &crate::state::GameState, ctx: &TriggerContext) -> u8 {
    let mut mask = match ctx.from_zone {
        Some(ZoneKind::Hand) => 1,
        Some(ZoneKind::Equip) => 4,
        _ => 0,
    };
    if let Some(kind) = ctx
        .to_zone
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.kind)
    {
        mask |= match kind {
            ZoneKind::Hand => 2,
            ZoneKind::Equip => 8,
            _ => 0,
        };
    }
    mask
}

fn no_player() -> PlayerId {
    PlayerId::NONE
}

/// Context extracted from a domain event for filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerContext {
    /// Event kind.
    pub kind: EventKind,
    /// Primary player (damage target, dying, killed, drawer, from-zone owner, …).
    pub player: PlayerId,
    /// Source (damage source, card user).
    pub source: PlayerId,
    /// Player whose card a `CardResponded` event answered.
    #[serde(default = "no_player")]
    pub respond_to: PlayerId,
    /// Damage/recovery amount, or number of cards for draw/move events.
    #[serde(default)]
    pub amount: u32,
    /// Damage nature tag; zero for normal damage and non-damage events.
    #[serde(default)]
    pub nature: u8,
    /// First end-context history mask, encoded by `PlayerId - 1` bits: turn damage sources for
    /// ordinary end contexts, phase card targets for `PhaseAnyEnd`.
    #[serde(default)]
    pub turn_damage_sources: u32,
    /// Second end-context history mask, encoded by `PlayerId - 1` bits: turn damage recipients
    /// for ordinary end contexts, phase HP changes for `PhaseAnyEnd`.
    #[serde(default)]
    pub turn_damaged_players: u32,
    /// Semantic cause for `CardsMoved`; unspecified for all other events.
    #[serde(default)]
    pub move_reason: CardMoveReason,
    /// Whether a `CardsMoved` batch contains at least one basic card.
    #[serde(default)]
    pub moved_has_basic: bool,
    /// For `CardsMoved`: kind of the source zone (if known).
    pub from_zone: Option<crate::state::ZoneKind>,
    /// For `CardsMoved`: destination zone id (if known).
    #[serde(default)]
    pub to_zone: Option<engine_api::ZoneId>,
    /// For `CardUsed`: the card instance id.
    pub card: Option<engine_api::CardId>,
    /// Logical card kind after view-as conversion.
    #[serde(default)]
    pub card_kind: Option<u8>,
    /// Card-use provenance flags attached to damage.
    #[serde(default)]
    pub card_flags: u8,
    /// Number of declared targets for `CardUsed`; zero for other events.
    #[serde(default)]
    pub target_count: u32,
    /// Ordered declared targets for `CardUsed`; empty for other events.
    #[serde(default)]
    pub targets: Vec<PlayerId>,
    /// Targets damaged by the resolved card (求首 filter; filled by the card-resolution tail).
    #[serde(default)]
    pub damaged_targets: Vec<PlayerId>,
    /// Physical cards carried by a batched domain event such as `CardsMoved`.
    #[serde(default)]
    pub cards: Vec<engine_api::CardId>,
    /// Def id of the skill instance carried by a `SkillAdded` event; zero otherwise.
    #[serde(default)]
    pub added_skill_def: u32,
    /// Debate snapshot (ADR 0644 P2): red-opinion players in the resolved debate.
    #[serde(default)]
    pub debate_red: Vec<PlayerId>,
    /// Debate snapshot: black-opinion players in the resolved debate.
    #[serde(default)]
    pub debate_black: Vec<PlayerId>,
}

/// Event-time history needed by filters after card counters advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TriggerHistorySnapshot {
    pub(crate) previous_card_meta: u32,
    pub(crate) previous_card_kind: u8,
    pub(crate) previous_card_def: u32,
    pub(crate) previous_play_card_meta: u32,
    pub(crate) previous_game_card_meta: u32,
    pub(crate) owner_sha_used: u32,
    pub(crate) owner_juedou_used: u32,
    pub(crate) source_card_used: i32,
    pub(crate) owner_card_action_count: i32,
    pub(crate) owner_play_phase_card_used: i32,
    pub(crate) event_player_play_phase_card_used: u32,
    pub(crate) event_player_play_phase_suit_count: u32,
    pub(crate) view_as_skill_def: u32,
    pub(crate) view_as_skill_owner: u32,
    pub(crate) event_player_had_other_red_card_target: bool,
    #[serde(default)]
    pub(crate) event_player_hand_discard_events: u32,
    #[serde(default)]
    pub(crate) event_player_hp_change_count: u32,
    #[serde(default)]
    pub(crate) owner_used_card_suit_history: u32,
}

impl TriggerHistorySnapshot {
    fn capture(
        state: &crate::state::GameState,
        active: &ActiveTrigger,
        ctx: &TriggerContext,
        view_as_skill: u32,
    ) -> Self {
        let view_as_source = state.skills.get(SkillId::new(view_as_skill));
        Self {
            previous_card_meta: state.turn.flag(turn_flag::LAST_CARD_META) as u32,
            previous_card_kind: u8::try_from(state.turn.flag(turn_flag::LAST_CARD_KIND))
                .unwrap_or(0),
            previous_card_def: state.turn.flag(turn_flag::LAST_CARD_DEF) as u32,
            previous_play_card_meta: state.turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32,
            previous_game_card_meta: state
                .turn
                .flag(turn_flag::last_game_card_meta_by_player(active.owner))
                as u32,
            owner_sha_used: state
                .turn
                .flag(turn_flag::sha_used_by_player(active.owner))
                .max(0) as u32,
            owner_juedou_used: state
                .turn
                .flag(turn_flag::juedou_used_by_player(active.owner))
                .max(0) as u32,
            source_card_used: state.turn.flag(turn_flag::card_used_by_player(ctx.source)),
            owner_card_action_count: state
                .turn
                .flag(turn_flag::card_action_count_by_player(active.owner)),
            owner_play_phase_card_used: state
                .turn
                .flag(turn_flag::play_phase_card_used_by_player(active.owner)),
            event_player_play_phase_card_used: state.turn.play_phase_card_use_count(ctx.player),
            event_player_play_phase_suit_count: state
                .turn
                .distinct_play_phase_card_suits(ctx.player),
            view_as_skill_def: view_as_source.map_or(0, |skill| skill.def.raw()),
            view_as_skill_owner: view_as_source.map_or(0, |skill| skill.owner.raw()),
            event_player_had_other_red_card_target: ctx
                .card
                .is_some_and(|card| state.turn.was_targeted_by_other_red_card(ctx.player, card)),
            event_player_hand_discard_events: state
                .turn
                .flag(turn_flag::hand_discard_event_by_player(ctx.player))
                .max(0) as u32,
            event_player_hp_change_count: state.turn.hp_change_count(ctx.player),
            owner_used_card_suit_history: state
                .turn
                .flag(turn_flag::used_card_suit_history_by_player(active.owner))
                .max(0) as u32,
        }
    }
}

/// A trigger already selected and ordered for an event, but not yet executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingTrigger {
    /// Active skill/trigger instance fixed at dispatch time.
    pub active: ActiveTrigger,
    /// Event context fixed at dispatch time.
    pub context: TriggerContext,
    /// View-as skill provenance captured while the originating card event was active.
    #[serde(default)]
    pub view_as_skill: u32,
    /// View-as card provenance captured while the originating card event was active.
    #[serde(default)]
    pub view_as_card: u32,
    /// Event-time card history used by filters; absent only in legacy saved states.
    #[serde(default)]
    pub(crate) history: Option<TriggerHistorySnapshot>,
}

/// Per-segment fire counts for anti-loop (skill_instance, trigger_def) → count.
#[derive(Debug, Default, Clone)]
pub struct TriggerFireRecord {
    counts: BTreeMap<(u32, u32), u32>,
}

impl TriggerFireRecord {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current count.
    pub fn get(&self, skill: SkillId, def: TriggerDefId) -> u32 {
        self.counts
            .get(&(skill.raw(), def.raw()))
            .copied()
            .unwrap_or(0)
    }

    /// Increment and return new count.
    pub fn inc(&mut self, skill: SkillId, def: TriggerDefId) -> u32 {
        let e = self.counts.entry((skill.raw(), def.raw())).or_insert(0);
        *e = e.saturating_add(1);
        *e
    }
}

/// Map domain events to trigger contexts (needs state for zone owner / kind).
pub fn contexts_from_event(
    state: &crate::state::GameState,
    ev: &DomainEvent,
) -> Option<TriggerContext> {
    match ev {
        DomainEvent::GameStarted { player_count } => Some(TriggerContext {
            kind: EventKind::GameStarted,
            player: PlayerId::NONE,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: u32::from(*player_count),
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::SkillAdded { player, skill } => Some(TriggerContext {
            kind: EventKind::SkillAdded,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
            added_skill_def: state
                .skills
                .get(*skill)
                .map(|instance| instance.def.raw())
                .unwrap_or(0),
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::DamageDealt {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => Some(TriggerContext {
            kind: if *is_lose_hp {
                EventKind::HpLost
            } else {
                EventKind::Damage
            },
            player: *target,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: *amount,
            nature: *nature,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
                added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::DamagePrevented {
            source,
            target,
            current,
        } => Some(TriggerContext {
            kind: EventKind::DamagePrevented,
            player: *current,
            source: *source,
            respond_to: *target,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::HpRecovered {
            source,
            player,
            amount,
        } => Some(TriggerContext {
            kind: EventKind::HpRecovered,
            player: *player,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: *amount,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardUsed {
            player,
            card,
            targets,
        } => Some(TriggerContext {
            kind: EventKind::CardUsed,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: card.is_some().then_some(*card),
            card_kind: state
                .cards
                .get(*card)
                .map(|value| crate::query::effective_card_kind(state, *player, value).tag()),
            card_flags: 0,
            target_count: targets.len() as u32,
            targets: targets.clone(),
            damaged_targets: Vec::new(),
            cards: state.resolution.card_source_cards(*player, *card),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => {
            let damaged_mask = targets
                .iter()
                .enumerate()
                .filter(|(_, target)| damaged_targets.contains(target))
                .fold(0u32, |mask, (index, _)| {
                    mask | 1u32.checked_shl(index as u32).unwrap_or(0)
                });
            Some(TriggerContext {
                kind: EventKind::CardResolved,
                player: *source,
                source: *source,
                respond_to: PlayerId::NONE,
                amount: damaged_mask,
                // CardResolved has no damage nature; carry its bounded (2–8 seats) damaged-target
                // mask separately from `amount`, whose legacy VM bridge also carries target count.
                nature: damaged_mask as u8,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: Some(*card),
                card_kind: Some(*kind),
                // CardResolved uses bit 0 for the event-local all-materials-from-hand fact.
                card_flags: u8::from(*all_sources_were_hand),
                target_count: targets.len() as u32,
                targets: targets.clone(),
                damaged_targets: Vec::new(),
                cards: source_cards.clone(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
            })
        }
        DomainEvent::DuelResolved {
            card,
            card_flags,
            initiator,
            target,
            winner,
            loser,
        } => Some(TriggerContext {
            kind: EventKind::DuelResolved,
            player: *winner,
            source: *loser,
            respond_to: *initiator,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*card),
            card_kind: Some(CardKind::TrickJuedou.tag()),
            card_flags: *card_flags,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::EquipSlotDisabled { player, slot } => Some(TriggerContext {
            kind: EventKind::EquipSlotDisabled,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: u32::from(*slot),
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::EquipSlotEnabled {
            source,
            player,
            slot,
        } => Some(TriggerContext {
            kind: EventKind::EquipSlotEnabled,
            player: *player,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: u32::from(*slot),
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over,
        } => Some(TriggerContext {
            kind: EventKind::CharacterStateChanged,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: if *turned_over {
                CHARACTER_STATE_TURNED_FACE_DOWN
            } else {
                CHARACTER_STATE_TURNED_FACE_UP
            },
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
                added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::PlayerLinkedChanged { player, linked } => Some(TriggerContext {
            kind: EventKind::CharacterStateChanged,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: if *linked {
                CHARACTER_STATE_BECAME_LINKED
            } else {
                CHARACTER_STATE_BECAME_UNLINKED
            },
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
                added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardResponded {
            player,
            respond_to,
            card,
            response_kind,
        } => Some(TriggerContext {
            kind: EventKind::CardResponded,
            player: *player,
            source: *player,
            respond_to: *respond_to,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: card.is_some().then_some(*card),
            card_kind: Some(*response_kind),
            card_flags: 0,
            target_count: 0,
            targets: vec![*respond_to],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::ShaDodged {
            source,
            target,
            card,
        } => Some(TriggerContext {
            kind: EventKind::ShaDodged,
            player: *target,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*card),
            card_kind: Some(CardKind::Sha.tag()),
            card_flags: 0,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::ShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => Some(TriggerContext {
            kind: EventKind::ShaResolved,
            player: *source,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: response_cards.len() as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*card),
            card_kind: Some(CardKind::Sha.tag()),
            card_flags: *card_flags,
            target_count: targets.len() as u32,
            targets: targets.clone(),
            damaged_targets: Vec::new(),
            cards: response_cards.clone(),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::HandCompareStarted {
            player,
            target,
            player_card,
        } => Some(TriggerContext {
            kind: EventKind::HandCompareStarted,
            player: *player,
            source: *target,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*player_card),
            card_kind: state.cards.get(*player_card).map(|card| card.kind.tag()),
            card_flags: 0,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![*player_card],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::DebateResolved {
            initiator,
            red,
            black,
            result,
        } => Some(TriggerContext {
            kind: EventKind::DebateResolved,
            player: *initiator,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: *result as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: (red.len() + black.len()) as u32,
            targets: red.iter().chain(black.iter()).copied().collect(),
            damaged_targets: Vec::new(),
            cards: Vec::new(),
        added_skill_def: 0,
            debate_red: red.clone(),
            debate_black: black.clone(),
        }),
        DomainEvent::DebateStarted {
            initiator,
            participants,
        } => Some(TriggerContext {
            kind: EventKind::DebateStarted,
            player: *initiator,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: participants.len() as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: participants.len() as u32,
            targets: participants.clone(),
            damaged_targets: Vec::new(),
            cards: Vec::new(),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::HandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => Some(TriggerContext {
            kind: EventKind::HandCompareFinished,
            player: *player,
            source: *target,
            respond_to: *winner,
            // `run_pending_triggers` shares this host scalar with target count. Offset by one so
            // a tie (`NONE == 0`) survives the `max(target_count)` compatibility path.
            amount: winner.raw().saturating_add(1),
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*player_card),
            card_kind: state.cards.get(*player_card).map(|card| card.kind.tag()),
            card_flags: 0,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![*player_card, *target_card],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::JudgmentResolved {
            player,
            result_card,
            cause_card,
            ..
        } => Some(TriggerContext {
            kind: EventKind::JudgmentResolved,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            // Legacy `judgeEnd.card` is the delayed trick/equipment cause, while
            // `judgeEnd.result.card` is carried in `cards` for result-card effects.
            card: cause_card.is_some().then_some(*cause_card),
            card_kind: state.cards.get(*cause_card).map(|card| card.kind.tag()),
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![*result_card],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::EnteredDying { player, source } => Some(TriggerContext {
            kind: EventKind::Dying,
            player: *player,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::LeftDying { player } => Some(TriggerContext {
            kind: EventKind::DyingAfter,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::MaxHpChanged { player, .. } => Some(TriggerContext {
            kind: EventKind::MaxHpChanged,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::PlayerKilled {
            player,
            killer,
            hand_equip_cards,
        } => Some(TriggerContext {
            kind: EventKind::Death,
            player: *player,
            source: *killer,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: hand_equip_cards.clone(),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardsDrawn { player, cards } => Some(TriggerContext {
            kind: EventKind::CardsDrawn,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: cards.len() as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: cards.clone(),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardsGiven {
            giver,
            recipient,
            cards,
        } => Some(TriggerContext {
            kind: EventKind::CardsGiven,
            player: *recipient,
            source: *giver,
            respond_to: PlayerId::NONE,
            amount: cards.len() as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Give,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: cards.clone(),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardsMoved {
            cards,
            from,
            to,
            reason,
        } => {
            let z = state.zones.get(*from);
            Some(TriggerContext {
                kind: EventKind::CardsMoved,
                player: z.map(|z| z.owner).unwrap_or(PlayerId::NONE),
                source: z.map(|z| z.owner).unwrap_or(PlayerId::NONE),
                respond_to: PlayerId::NONE,
                amount: cards.len() as u32,
                nature: 0,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: *reason,
                moved_has_basic: cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.is_basic())
                }),
                from_zone: z.map(|z| z.kind),
                to_zone: Some(*to),
                card: None,
                card_kind: None,
                card_flags: 0,
                target_count: 0,
                targets: vec![],
                damaged_targets: Vec::new(),
                cards: cards.clone(),
                    added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
            })
        }
        DomainEvent::RoundEnded { round, current } => Some(TriggerContext {
            kind: EventKind::RoundEnd,
            player: *current,
            source: *current,
            respond_to: PlayerId::NONE,
            amount: *round,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::RoundStarted { round, current } => Some(TriggerContext {
            kind: EventKind::RoundStart,
            player: *current,
            source: *current,
            respond_to: PlayerId::NONE,
            amount: *round,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::TurnBegan { player } => Some(TriggerContext {
            kind: EventKind::PhaseBegin,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![],
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::TurnStateChanged { current, phase } => {
            let kind = match *phase {
                engine_api::TurnPhaseTag::Prepare => EventKind::PhasePrepare,
                engine_api::TurnPhaseTag::Judge => EventKind::PhaseJudge,
                engine_api::TurnPhaseTag::Draw => EventKind::PhaseDraw,
                engine_api::TurnPhaseTag::Play => EventKind::PhasePlay,
                engine_api::TurnPhaseTag::Discard => EventKind::PhaseDiscard,
                engine_api::TurnPhaseTag::End => EventKind::PhaseEnd,
                engine_api::TurnPhaseTag::Idle => return None,
            };
            Some(TriggerContext {
                kind,
                player: *current,
                source: *current,
                respond_to: PlayerId::NONE,
                amount: if *phase == engine_api::TurnPhaseTag::End {
                    state.turn.damaged_player_count()
                } else {
                    0
                },
                nature: 0,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: None,
                card_kind: None,
                card_flags: 0,
                target_count: 0,
                targets: vec![],
                damaged_targets: Vec::new(),
                cards: vec![],
                    added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
            })
        }
        DomainEvent::PhaseEnded {
            current,
            phase,
            damage_sources,
            damaged_players,
            card_targeted_other,
            current_dealt_damage_other,
            ..
        } => {
            let kind = match *phase {
                engine_api::TurnPhaseTag::Draw => EventKind::PhaseDrawEnd,
                engine_api::TurnPhaseTag::Play => EventKind::PhasePlayEnd,
                engine_api::TurnPhaseTag::Discard => EventKind::PhaseDiscardEnd,
                engine_api::TurnPhaseTag::End => EventKind::TurnEnded,
                _ => return None,
            };
            Some(TriggerContext {
                kind,
                player: *current,
                source: *current,
                respond_to: PlayerId::NONE,
                amount: 0,
                nature: 0,
                turn_damage_sources: *damage_sources,
                turn_damaged_players: *damaged_players,
                move_reason: CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: None,
                card_kind: None,
                card_flags: u8::from(*card_targeted_other)
                    | (u8::from(*current_dealt_damage_other) << 1),
                target_count: 0,
                targets: vec![],
                damaged_targets: Vec::new(),
                cards: vec![],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
            })
        }
        DomainEvent::PhaseSkipped { current, phase } => {
            if *phase != engine_api::TurnPhaseTag::Draw {
                return None;
            }
            Some(TriggerContext {
                kind: EventKind::PhaseDrawSkipped,
                player: *current,
                source: *current,
                respond_to: PlayerId::NONE,
                amount: 0,
                nature: 0,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: None,
                card_kind: None,
                card_flags: 0,
                target_count: 0,
                targets: vec![],
                damaged_targets: Vec::new(),
                cards: vec![],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
            })
        }
        _ => None,
    }
}

fn target_contexts_from_event(
    state: &crate::state::GameState,
    ev: &DomainEvent,
) -> Vec<TriggerContext> {
    let (player, card, targets, added_target) = match ev {
        DomainEvent::CardUsed {
            player,
            card,
            targets,
        } => (*player, *card, targets, None),
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            targets,
        } => (*player, *card, targets, Some(*target)),
        _ => return Vec::new(),
    };
    let card_kind = state
        .cards
        .get(card)
        .map(|value| crate::query::effective_card_kind(state, player, value).tag());
    let mut source_cards = state.resolution.card_source_cards(player, card);
    if source_cards.is_empty() {
        source_cards.push(card);
    }
    targets
        .iter()
        .filter(|target| added_target.is_none_or(|added| added == **target))
        .map(|target| TriggerContext {
            kind: EventKind::CardTargeted,
            player: *target,
            source: player,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(card),
            card_kind,
            card_flags: 0,
            target_count: targets.len() as u32,
            targets: targets.clone(),
            damaged_targets: Vec::new(),
            cards: source_cards.clone(),
        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        })
        .collect()
}

/// Plan effect requests from triggers reacting to a batch of domain events.
pub fn dispatch_for_events(
    tx: &mut Transaction,
    events: &[DomainEvent],
    fires: &mut TriggerFireRecord,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events_with_damage_context(tx, events, fires, None)
}

/// Dispatch events while attaching scheduler-owned card provenance to damage.
pub(crate) fn dispatch_for_events_with_damage_context(
    tx: &mut Transaction,
    events: &[DomainEvent],
    fires: &mut TriggerFireRecord,
    damage_context: Option<(engine_api::CardId, u8, u8)>,
) -> Result<Vec<EffectRequest>, EffectError> {
    let mut pending = Vec::new();
    for ev in events {
        let planned = (|| -> Result<(), EffectError> {
            if let Some(mut ctx) = contexts_from_event(tx.state(), ev) {
                if ctx.kind == EventKind::Damage {
                    if let Some((card, kind, flags)) = damage_context {
                        if card.is_some() {
                            ctx.card = Some(card);
                        }
                        if kind != 0 {
                            ctx.card_kind = Some(kind);
                        }
                        ctx.card_flags = flags;
                        if card.is_some() {
                            ctx.cards = tx.state().resolution.card_source_cards(ctx.source, card);
                        }
                    }
                }
                if matches!(
                    ev,
                    DomainEvent::TurnStateChanged {
                        phase: engine_api::TurnPhaseTag::Prepare,
                        ..
                    }
                ) && tx.state().turn.flag(turn_flag::EXPLICIT_TURN_BEGIN_EVENT) == 0
                {
                    let mut phase_begin = ctx.clone();
                    phase_begin.kind = EventKind::PhaseBegin;
                    pending.extend(plan_triggers(tx, phase_begin, fires)?);
                }
                pending.extend(plan_triggers(tx, ctx, fires)?);
            }
            if let DomainEvent::PhaseSkipped { current, phase } = ev {
                let phase_any_skipped = TriggerContext {
                    kind: EventKind::PhaseAnySkipped,
                    player: *current,
                    source: *current,
                    respond_to: PlayerId::NONE,
                    amount: match phase {
                        engine_api::TurnPhaseTag::Prepare => 1,
                        engine_api::TurnPhaseTag::Judge => 2,
                        engine_api::TurnPhaseTag::Draw => 3,
                        engine_api::TurnPhaseTag::Play => 4,
                        engine_api::TurnPhaseTag::Discard => 5,
                        engine_api::TurnPhaseTag::End => 6,
                        engine_api::TurnPhaseTag::Idle => 0,
                    },
                    nature: 0,
                    turn_damage_sources: 0,
                    turn_damaged_players: 0,
                    move_reason: CardMoveReason::Unspecified,
                    moved_has_basic: false,
                    from_zone: None,
                    to_zone: None,
                    card: None,
                    card_kind: None,
                    card_flags: 0,
                    target_count: 0,
                    targets: vec![],
                    damaged_targets: Vec::new(),
                    cards: vec![],
                        added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
                };
                pending.extend(plan_triggers(tx, phase_any_skipped, fires)?);
            }
            if let DomainEvent::PhaseEnded {
                current,
                phase_targeted_players,
                phase_hp_changed_players,
                ..
            } = ev
            {
                let phase_any_end = TriggerContext {
                    kind: EventKind::PhaseAnyEnd,
                    player: *current,
                    source: *current,
                    respond_to: PlayerId::NONE,
                    amount: 0,
                    nature: 0,
                    turn_damage_sources: *phase_targeted_players,
                    turn_damaged_players: *phase_hp_changed_players,
                    move_reason: CardMoveReason::Unspecified,
                    moved_has_basic: false,
                    from_zone: None,
                    to_zone: None,
                    card: None,
                    card_kind: None,
                    card_flags: 0,
                    target_count: 0,
                    targets: vec![],
                    damaged_targets: Vec::new(),
                    cards: vec![],
                added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
                };
                pending.extend(plan_triggers(tx, phase_any_end, fires)?);
            }
            for ctx in target_contexts_from_event(tx.state(), ev) {
                if ctx.kind == EventKind::CardTargeted
                    && tx.state().content.tracks_card_target_history()
                {
                    tx.state_mut().turn.record_card_targeted(ctx.player);
                }
                pending.extend(plan_triggers(tx, ctx, fires)?);
            }
            Ok(())
        })();
        if let DomainEvent::CardUsed { card, .. } = ev {
            if tx.state().content.tracks_basic_use_history()
                && tx
                    .state()
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| card_state.kind.is_basic())
            {
                tx.state_mut().lifetime_basic_card_uses += 1;
            }
        }
        if let DomainEvent::CardUsed { card, .. } = ev {
            if tx.state().content.tracks_used_type_groups() {
                if let Some(card_state) = tx.state().cards.get(*card) {
                    let bit = 1u8 << card_state.kind.card_type_group().min(2);
                    tx.state_mut().lifetime_used_type_groups |= bit;
                }
            }
        }
        remember_card_action(tx, ev);
        planned?;
    }
    run_pending_triggers(tx, pending)
}

fn remember_card_action(tx: &mut Transaction, event: &DomainEvent) {
    if tx.state().turn.phase != Phase::Discard
        && matches!(
            event,
            DomainEvent::CardsMoved {
                cards,
                reason: CardMoveReason::Discard,
                ..
            } if !cards.is_empty()
        )
    {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::NON_DISCARD_PHASE_DISCARD_SEEN, 1);
    }
    if tx.state().content.tracks_owner_card_category_history() {
        match event {
            DomainEvent::CardUsed { player, card, .. } => {
                let category = tx.state().cards.get(*card).and_then(|card| {
                    let kind = crate::query::effective_card_kind(tx.state(), *player, card);
                    if kind.is_basic() {
                        Some(0)
                    } else if kind.is_trick() {
                        Some(1)
                    } else if kind.is_equip() {
                        Some(2)
                    } else {
                        None
                    }
                });
                if let Some(category) = category {
                    tx.state_mut()
                        .turn
                        .remember_owner_card_category(*player, category);
                }
            }
            DomainEvent::CardsMoved {
                cards,
                from,
                reason: CardMoveReason::Discard,
                ..
            } => {
                let owner = tx.state().zones.get(*from).and_then(|zone| {
                    matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip).then_some(zone.owner)
                });
                if let Some(owner) = owner.filter(|owner| !owner.is_none()) {
                    let categories: Vec<_> = cards
                        .iter()
                        .filter_map(|card| {
                            tx.state().cards.get(*card).and_then(|card| {
                                let kind =
                                    crate::query::effective_card_kind(tx.state(), owner, card);
                                if kind.is_basic() {
                                    Some(0)
                                } else if kind.is_trick() {
                                    Some(1)
                                } else if kind.is_equip() {
                                    Some(2)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();
                    for category in categories {
                        tx.state_mut()
                            .turn
                            .remember_owner_card_category(owner, category);
                    }
                }
            }
            _ => {}
        }
    }
    if tx.state().content.tracks_discard_phase_discard_count()
        && tx.state().turn.phase == Phase::Discard
    {
        if let DomainEvent::CardsMoved {
            cards,
            from,
            reason: CardMoveReason::Discard,
            ..
        } = event
        {
            let counts = tx.state().zones.get(*from).is_some_and(|zone| {
                zone.owner == tx.state().turn.current
                    && matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)
            });
            if counts {
                let current = tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED);
                tx.state_mut().turn.set_flag(
                    turn_flag::DISCARD_PHASE_DISCARDED,
                    current.saturating_add(i32::try_from(cards.len()).unwrap_or(i32::MAX)),
                );
            }
        }
    }
    // twjieqiu 劫囚（§7 装备栏编排）：BUFF 标记目标的每次弃牌即时等量恢复
    // 禁用槽——独立顶层块，不受 tracks_discard_phase_discard_count 门控。
    if let DomainEvent::CardsMoved {
        cards,
        reason: CardMoveReason::Discard,
        ..
    } = event
    {
        let mover = tx.state().turn.current;
        let buffed = tx.state().players.get(mover).is_some_and(|p| {
            p.alive
                && p.marks
                    .iter()
                    .any(|(id, count)| *id == crate::content::TWJIEQIU_BUFF_MARK && *count > 0)
        });
        if buffed {
            let mut remaining = cards.len() as u32;
            for tag in 1u8..=5u8 {
                if remaining == 0 {
                    break;
                }
                let Some(slot) = crate::state::EquipSlot::from_tag(tag) else {
                    continue;
                };
                let index = slot.index();
                let still_disabled = tx
                    .state()
                    .players
                    .get(mover)
                    .and_then(|p| p.disabled_equip_slots.get(index))
                    .copied()
                    .unwrap_or(0)
                    > 0;
                if still_disabled {
                    if let Some(p) = tx.state_mut().players.get_mut(mover) {
                        if let Some(count) = p.disabled_equip_slots.get_mut(index) {
                            *count -= 1;
                        }
                    }
                    remaining -= 1;
                }
            }
        }
    }
    if tx.state().content.tracks_turn_discarded_cards() {
        if let DomainEvent::CardsMoved {
            cards,
            from,
            to,
            reason: CardMoveReason::Discard,
            ..
        } = event
        {
            let from_player = tx
                .state()
                .zones
                .get(*from)
                .is_some_and(|zone| zone.owner.is_some());
            let to_discard = tx
                .state()
                .zones
                .get(*to)
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
            if from_player && to_discard {
                tx.state_mut()
                    .turn
                    .record_discarded_cards(cards.iter().copied());
            }
        }
    }
    if tx.state().content.tracks_turn_haoyi_cards() {
        if let DomainEvent::CardsMoved { cards, to, .. } = event {
            let to_discard = tx
                .state()
                .zones
                .get(*to)
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
            if to_discard {
                tx.state_mut()
                    .turn
                    .record_entered_discard_cards(cards.iter().copied());
            }
        }
    }
    if tx.state().content.tracks_phase_target_hp_history()
        && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
    {
        match event {
            DomainEvent::CardUsed { targets, .. } => {
                for target in targets {
                    tx.state_mut().turn.mark_phase_card_target(*target);
                }
            }
            DomainEvent::CardTargetAdded { target, .. } => {
                tx.state_mut().turn.mark_phase_card_target(*target);
            }
            _ => {}
        }
    }
    if tx.state().content.tracks_discard_pile_suit_history() {
        if let DomainEvent::CardsMoved { cards, to, .. } = event {
            if tx
                .state()
                .zones
                .get(*to)
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
            {
                let suits: Vec<_> = cards
                    .iter()
                    .filter_map(|card| tx.state().cards.get(*card).map(|card| card.suit))
                    .collect();
                tx.state_mut().turn.remember_discard_pile_suits(suits);
            }
        }
    }
    if let DomainEvent::CardsMoved {
        from,
        reason: CardMoveReason::Discard,
        ..
    } = event
    {
        if let Some(owner) = tx.state().zones.get(*from).and_then(|zone| {
            (zone.kind == ZoneKind::Hand && zone.owner.is_some()).then_some(zone.owner)
        }) {
            let key = turn_flag::hand_discard_event_by_player(owner);
            let count = tx.state().turn.flag(key);
            tx.state_mut().turn.set_flag(key, count.saturating_add(1));
        }
    }
    if let DomainEvent::CardUsed {
        player,
        card,
        targets,
    } = event
    {
        if tx.state().content.tracks_ddd_chengjing_history()
            && *player == tx.state().turn.current
            && tx.state().turn.flag(turn_flag::VIEW_AS_SKILL) == 0
        {
            let materials = tx.state().resolution.card_source_cards(*player, *card);
            if let [material] = materials.as_slice() {
                let eligible = tx.state().cards.get(*material).is_some_and(|physical| {
                    physical.def.is_some()
                        && (physical.kind.is_basic()
                            || (physical.kind.is_trick()
                                && !physical.kind.is_delayed_trick()
                                && !matches!(
                                    physical.kind,
                                    CardKind::TrickWuxie | CardKind::TrickYoudishenru
                                )))
                });
                if eligible {
                    tx.state_mut()
                        .turn
                        .ddd_chengjing_eligible_uses
                        .push(*material);
                }
            }
        }
        if tx.state().content.tracks_turn_card_target_counts() {
            for target in targets
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
            {
                tx.state_mut().turn.record_card_target(*player, target);
            }
        }
        let tracks_red_targets = tx.state().content.tracks_turn_red_card_target_history();
        let card_is_red = tracks_red_targets
            && tx.state().cards.get(*card).is_some_and(|value| {
                matches!(
                    crate::query::effective_card_suit(tx.state(), *player, value),
                    engine_api::Suit::Heart | engine_api::Suit::Diamond
                )
            });
        if card_is_red {
            for target in targets {
                tx.state_mut().turn.remember_red_card_target(*target, *card);
            }
        }
    }
    let (action_player, action_card) = match event {
        DomainEvent::CardUsed { player, card, .. }
        | DomainEvent::CardResponded { player, card, .. } => (*player, *card),
        _ => return,
    };
    if tx.state().content.tracks_turn_use_response_discard_count() && action_card.is_some() {
        let in_discard = tx
            .state()
            .cards
            .get(action_card)
            .and_then(|card| tx.state().zones.get(card.zone))
            .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
        if in_discard {
            let count = tx.state().turn.flag(turn_flag::USE_RESPONSE_DISCARDED);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::USE_RESPONSE_DISCARDED, count.saturating_add(1));
        }
    }
    if tx
        .state()
        .content
        .tracks_per_player_turn_card_action_count()
    {
        let key = turn_flag::card_action_count_by_player(action_player);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
    }
    if tx.state().content.tracks_lifetime_card_action_count() {
        let total = tx
            .state_mut()
            .card_action_totals
            .entry(action_player)
            .or_insert(0);
        *total = total.saturating_add(1);
    }
    let DomainEvent::CardUsed {
        player,
        card,
        targets,
    } = event
    else {
        return;
    };
    if tx.state().content.tracks_turn_used_card_suit_history() {
        if let Some(suit) = tx
            .state()
            .cards
            .get(*card)
            .map(|value| crate::query::effective_card_suit(tx.state(), *player, value))
        {
            tx.state_mut().turn.remember_used_card_suit(*player, suit);
        }
    }
    if tx.state().content.tracks_turn_basic_card_variant_history() {
        let variant = tx.state().cards.get(*card).and_then(|value| {
            match crate::query::effective_card_kind(tx.state(), *player, value) {
                CardKind::Sha => Some(0),
                CardKind::ShaFire => Some(1),
                CardKind::ShaThunder => Some(2),
                CardKind::Shan => Some(3),
                CardKind::Tao => Some(4),
                CardKind::Jiu => Some(5),
                _ => None,
            }
        });
        if let Some(variant) = variant {
            tx.state_mut()
                .turn
                .remember_basic_card_variant(*player, variant);
        }
    }
    if *player == tx.state().turn.current
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_per_player_play_phase_card_count()
    {
        let key = turn_flag::play_phase_card_used_by_player(*player);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
    }
    if *player == tx.state().turn.current
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_play_phase_card_suit_history()
    {
        if let Some(suit) = tx
            .state()
            .cards
            .get(*card)
            .map(|value| crate::query::effective_card_suit(tx.state(), *player, value))
        {
            tx.state_mut()
                .turn
                .remember_play_phase_card_suit_used(*player, suit);
        }
    }
    if tx.state().turn.phase == Phase::Play
        && (*player == tx.state().turn.current
            || tx
                .state()
                .content
                .tracks_play_card_category_history_all_users())
        && (tx.state().content.tracks_play_card_category_history()
            || tx.state().content.tracks_play_card_category_duplicates())
    {
        let category = tx.state().cards.get(*card).and_then(|card| {
            let kind = crate::query::effective_card_kind(tx.state(), *player, card);
            if kind.is_basic() {
                Some(0)
            } else if kind.is_trick() {
                Some(1)
            } else if kind.is_equip() {
                Some(2)
            } else {
                None
            }
        });
        if let Some(category) = category {
            let track_duplicate = tx.state().content.tracks_play_card_category_duplicates();
            tx.state_mut().turn.remember_play_card_category_used(
                *player,
                category,
                track_duplicate,
            );
        }
    }
    if tx.state().content.tracks_per_player_turn_sha_use_count() {
        let key = tx.state().cards.get(*card).and_then(|card| {
            match crate::query::effective_card_kind(tx.state(), *player, card) {
                kind if kind.is_sha() => Some(turn_flag::sha_used_by_player(*player)),
                CardKind::TrickJuedou => Some(turn_flag::juedou_used_by_player(*player)),
                _ => None,
            }
        });
        if let Some(key) = key {
            let count = tx.state().turn.flag(key);
            tx.state_mut().turn.set_flag(key, count.saturating_add(1));
        }
    }
    let track_turn =
        *player == tx.state().turn.current && tx.state().content.tracks_turn_card_history();
    let track_turn_name =
        *player == tx.state().turn.current && tx.state().content.tracks_turn_card_name_history();
    let track_play = *player == tx.state().turn.current
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_play_card_history();
    let track_count = tx.state().content.tracks_per_player_turn_card_count();
    let track_game = tx.state().content.tracks_game_card_history();
    if !track_turn && !track_turn_name && !track_play && !track_count && !track_game {
        return;
    }
    if track_turn_name {
        let (kind, def) = {
            let state = tx.state();
            let Some(card) = state.cards.get(*card) else {
                return;
            };
            (
                crate::query::effective_card_kind(state, *player, card).tag(),
                card.def.raw(),
            )
        };
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_CARD_KIND, i32::from(kind));
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_CARD_DEF, def as i32);
        let mut mask: i32 = 0;
        for target in targets {
            if let Some(bit) = target
                .raw()
                .checked_sub(1)
                .and_then(|shift| 1_i32.checked_shl(shift))
            {
                mask |= bit;
            }
        }
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_CARD_TARGETS, mask);
        let is_sha = tx.state().cards.get(*card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), *player, value).is_sha()
        });
        if is_sha {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::LAST_SHA_TARGETS, mask);
        }
    }
    if track_turn || track_play || track_game {
        let meta = {
            let state = tx.state();
            let Some(card) = state.cards.get(*card) else {
                return;
            };
            let suit = crate::query::effective_card_suit(state, *player, card);
            u32::from(crate::command::judge::suit_tag(suit))
                | (u32::from(card.rank) << 8)
                | (u32::from(state.turn.phase.to_u8()) << 16)
        };
        if track_turn {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::LAST_CARD_META, meta as i32);
        }
        if track_play {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::LAST_PLAY_CARD_META, meta as i32);
        }
        if track_game {
            tx.state_mut().turn.set_flag(
                turn_flag::last_game_card_meta_by_player(*player),
                meta as i32,
            );
        }
    }
    if track_count {
        let key = turn_flag::card_used_by_player(*player);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
        // NOTE: round-scoped card-use counting (cards_used_this_round) is recorded lazily by
        // content that opts into the round-history ABI, not here, to keep the canonical state
        // hash stable for packages that pre-date the round-history query.
    }
}

pub(crate) fn plan_triggers(
    tx: &mut Transaction,
    ctx: TriggerContext,
    fires: &mut TriggerFireRecord,
) -> Result<Vec<PendingTrigger>, EffectError> {
    if tx.state().resolution.trigger_budget == 0 {
        return Err(EffectError::BudgetExceeded);
    }

    let alive = tx.state().players.alive_by_seat();
    let current = tx.state().turn.current;
    let kind = ctx.kind;

    // Clone actives so we can mutably update skill turn_fire_count later.
    let ordered: Vec<(TriggerOrderKey, ActiveTrigger)> = tx
        .state()
        .triggers
        .collect_sorted(kind, |active| {
            let seat_rel = seat_rel_to_current(&alive, current, active.owner);
            TriggerOrderKey::new(
                timing_from_u16(active.timing),
                active.priority,
                active.optional,
                seat_rel,
                active.skill_id.raw(),
                active.trigger_def.raw(),
            )
        })
        .into_iter()
        .map(|(k, a)| (k, a.clone()))
        .collect();

    let mut pending = Vec::new();
    for (_key, active) in ordered {
        let view_as_skill = tx.state().turn.flag(turn_flag::VIEW_AS_SKILL).max(0) as u32;
        let view_as_card = tx.state().turn.flag(turn_flag::VIEW_AS_CARD).max(0) as u32;
        let history = TriggerHistorySnapshot::capture(tx.state(), &active, &ctx, view_as_skill);
        if !filter_match(
            tx.state(),
            &active,
            &ctx,
            view_as_skill,
            view_as_card,
            Some(&history),
        ) {
            continue;
        }
        let def = tx
            .state()
            .content
            .find_def(active.trigger_def)
            .ok_or_else(|| {
                EffectError::EngineFault(format!(
                    "missing trigger def {}",
                    active.trigger_def.raw()
                ))
            })?;

        let count = fires.get(active.skill_id, active.trigger_def);
        if count >= def.max_per_segment {
            continue;
        }
        if let Some(limit) = turn_limit(&def.filter) {
            let fired = tx
                .state()
                .skills
                .get(active.skill_id)
                .map(|skill| skill.turn_fire_count)
                .unwrap_or(0);
            if fired.saturating_add(count) >= limit {
                continue;
            }
        }
        if let Some(limit) = play_phase_limit(&def.filter) {
            let fired = tx
                .state()
                .skills
                .get(active.skill_id)
                .map(|skill| skill.play_phase_fire_count)
                .unwrap_or(0);
            if fired.saturating_add(count) >= limit {
                continue;
            }
        }
        if once_per_round(&def.filter) {
            let fired_this_round = tx
                .state()
                .skills
                .get(active.skill_id)
                .is_some_and(|skill| skill.round_fire_round == tx.state().turn.round);
            if fired_this_round || count > 0 {
                continue;
            }
        }
        if let Some(limit) = lifetime_limit(&def.filter) {
            let fired = tx
                .state()
                .skills
                .get(active.skill_id)
                .map(|skill| skill.lifetime_fire_count)
                .unwrap_or(0);
            if fired.saturating_add(count) >= limit {
                continue;
            }
        }

        let repeat = if matches!(
            def.filter,
            TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint
                | TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
                | TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint
                | TriggerFilter::OwnerIsEventPlayerPerDamagePoint
                | TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty
                | TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint
                | TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint
                | TriggerFilter::OwnerFromEquipPerCard
        ) {
            ctx.amount
        } else {
            1
        }
        .min(def.max_per_segment.saturating_sub(count));
        for _ in 0..repeat {
            fires.inc(active.skill_id, active.trigger_def);
            if tx.state().resolution.trigger_budget == 0 {
                return Err(EffectError::BudgetExceeded);
            }
            pending.push(PendingTrigger {
                active: active.clone(),
                context: ctx.clone(),
                view_as_skill,
                view_as_card,
                history: Some(history),
            });
        }
    }
    Ok(pending)
}

/// Execute an ordered trigger queue until completion or a choice barrier.
///
/// A queued trigger keeps its original event context, but its filter is re-evaluated against the
/// current state immediately before execution. Effects from an earlier trigger are committed
/// before the next trigger is considered so replacement/exclusion filters can invalidate stale
/// queued work without opening an optional prompt.
pub fn run_pending_triggers(
    tx: &mut Transaction,
    pending: Vec<PendingTrigger>,
) -> Result<Vec<EffectRequest>, EffectError> {
    let mut queue: VecDeque<_> = pending.into();
    let mut effects = Vec::new();
    loop {
        if !effects.is_empty() && !queue.is_empty() {
            match crate::effect::run_effects(tx, std::mem::take(&mut effects)) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {
                    tx.state_mut().resolution.deferred_triggers.extend(queue);
                    return Err(EffectError::ChoiceRequested);
                }
                Err(error) => return Err(error),
            }
        }

        let Some(task) = queue.pop_front() else {
            break;
        };
        if !filter_match(
            tx.state(),
            &task.active,
            &task.context,
            task.view_as_skill,
            task.view_as_card,
            task.history.as_ref(),
        ) {
            continue;
        }
        if task.active.optional {
            let owner = task.active.owner;
            let choice_actor = tx
                .state()
                .content
                .find_def(task.active.trigger_def)
                .map(|definition| definition.choice_actor)
                .ok_or_else(|| {
                    EffectError::EngineFault(format!(
                        "missing trigger def {}",
                        task.active.trigger_def.raw()
                    ))
                })?;
            let observer = match choice_actor {
                TriggerChoiceActor::Owner => owner,
                TriggerChoiceActor::EventPlayer => task.context.player,
                TriggerChoiceActor::EventSource => task.context.source,
            };
            if observer.is_none() {
                return Err(EffectError::ContentFault(format!(
                    "optional trigger {} has no {:?} choice actor",
                    task.active.trigger_def.raw(),
                    choice_actor
                )));
            }
            let mut confirmed = task;
            confirmed.active.optional = false;
            let r = &mut tx.state_mut().resolution;
            r.optional_trigger = Some(confirmed);
            r.deferred_effects.extend(effects);
            r.deferred_triggers.extend(queue);
            crate::effect::open_choice(
                tx,
                observer,
                1,
                1,
                false,
                vec![0, 1],
                engine_api::ChoiceKind::Confirm,
                crate::state::ContinuationKind::OptionalTrigger,
                Vec::new(),
            )?;
            return Err(EffectError::ChoiceRequested);
        }
        spend_trigger_budget(tx, 1)?;
        let marker = [
            task.active.skill_id.raw(),
            task.active.trigger_def.raw(),
            task.context.kind.tag(),
        ];
        let handler = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .ok_or_else(|| {
                EffectError::EngineFault(format!(
                    "missing trigger def {}",
                    task.active.trigger_def.raw()
                ))
            })?
            .handler
            .clone();
        let turn_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .and_then(|definition| turn_limit(&definition.filter))
            .is_some();
        if turn_limited {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.turn_fire_count = skill.turn_fire_count.saturating_add(1);
            }
        }
        let play_phase_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .and_then(|definition| play_phase_limit(&definition.filter))
            .is_some();
        if play_phase_limited {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.play_phase_fire_count = skill.play_phase_fire_count.saturating_add(1);
            }
        }
        let round_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .is_some_and(|definition| once_per_round(&definition.filter));
        if round_limited {
            let round = tx.state().turn.round;
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.round_fire_round = round;
            }
        }
        let lifetime_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .and_then(|definition| lifetime_limit(&definition.filter))
            .is_some();
        if lifetime_limited {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.lifetime_fire_count = skill.lifetime_fire_count.saturating_add(1);
            }
        }
        let first_for_target = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .is_some_and(|definition| {
                definition.filter == TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
            });
        if first_for_target {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.record_lifetime_target(task.context.player);
            }
        }

        if let TriggerHandler::RunVm { module, entry_pc } = handler {
            let Some(bytecode) = tx.state().content.module(&module).cloned() else {
                effects.push(EffectRequest::EmitRuleEvent {
                    code: 101,
                    payload: vec![task.active.skill_id.raw()],
                });
                effects.push(trigger_marker(marker));
                continue;
            };
            let _ = entry_pc;
            let fuel = tx.state().resolution.effect_budget.min(10_000);
            let ctx = [
                task.active.owner.raw(),
                task.context.player.raw(),
                task.context.source.raw(),
                task.context.card.map(|c| c.raw()).unwrap_or(0),
            ];
            let expose_only_raw_black_discards = tx
                .state()
                .content
                .find_def(task.active.trigger_def)
                .is_some_and(|definition| {
                    matches!(
                        definition.filter,
                        TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard
                    )
                });
            let event_cards: Vec<_> = task
                .context
                .cards
                .iter()
                .filter(|card| {
                    !expose_only_raw_black_discards
                        || tx.state().cards.get(**card).is_some_and(|card_state| {
                            crate::query::card_color_mask_for_suit(card_state.suit) == 0b10
                        })
                })
                .map(|card| card.raw())
                .collect();
            let event_targets: Vec<_> = task
                .context
                .targets
                .iter()
                .map(|target| target.raw())
                .collect();
            let outcome = run_module_with_event_cards(
                tx.state(),
                &bytecode,
                task.active.skill_id.raw(),
                ctx,
                task.context.amount.max(task.context.target_count),
                &event_cards,
                &event_targets,
                task.context.nature,
                task.context.card_kind.unwrap_or(0),
                task.context.card_flags,
                task.context
                    .to_zone
                    .and_then(|zone| tx.state().zones.get(zone))
                    .map(|zone| zone.owner)
                    .unwrap_or(PlayerId::NONE),
                ddd_zhengjun_move_mask(tx.state(), &task.context),
                fuel,
            )
            .map_err(|e| EffectError::ContentFault(format!("vm: {e}")))?;
            effects.extend(outcome.effects);
            if let Some(pause) = outcome.paused {
                let r = &mut tx.state_mut().resolution;
                r.deferred_effects.extend(effects);
                r.deferred_triggers.extend(queue);
                crate::command::resolve_choice::open_vm_choice(
                    tx,
                    task.active.owner,
                    &module,
                    task.active.skill_id.raw(),
                    ctx,
                    marker,
                    pause,
                )?;
                return Err(EffectError::ChoiceRequested);
            }
        } else {
            effects.extend(native_handler_effects(&task.active, &handler));
        }
        effects.push(trigger_marker(marker));
    }
    Ok(effects)
}

fn trigger_marker(marker: [u32; 3]) -> EffectRequest {
    EffectRequest::EmitRuleEvent {
        code: 100,
        payload: marker.to_vec(),
    }
}

/// Card-name-scoped filter helper (Phase 3): returns true when the event's originating card
/// (from `ctx.card`, resolved against `state`) has a logical kind/suit matching `selector`.
/// Falls back to `ctx.card_kind` when the physical card is unavailable.
fn event_card_matches_selector(
    state: &crate::state::GameState,
    owner: engine_api::PlayerId,
    ctx: &TriggerContext,
    selector: crate::query::CardSelector,
) -> bool {
    if let Some(card_id) = ctx.card {
        if let Some(card) = state.cards.get(card_id) {
            let suit = crate::query::effective_card_suit(state, owner, card);
            return selector.matches_physical_card(state, owner, card, suit);
        }
    }
    // Fallback: use the event's resolved card kind tag and no suit constraint.
    matches!(selector, crate::query::CardSelector::Any)
        || ctx.card_kind
            .and_then(CardKind::from_tag)
            .is_some_and(|kind| selector.matches_kind_and_suit(kind, engine_api::Suit::Spade))
}

/// 神君: whether the owner's hand holds a card sharing the used card's
/// effective kind (同名牌). The used card is 杀 or a trick per the skill text.
fn hand_has_same_kind_as_event_card(
    state: &crate::state::GameState,
    owner: engine_api::PlayerId,
    ctx: &TriggerContext,
) -> bool {
    let Some(event_card) = ctx.card.and_then(|card| state.cards.get(card)) else {
        return false;
    };
    let kind = crate::query::effective_card_kind(state, owner, event_card);
    if !(kind.is_sha() || kind.is_trick()) {
        return false;
    }
    state.hand_cards(owner).into_iter().any(|card| {
        state.cards.get(card).is_some_and(|candidate| {
            crate::query::effective_card_kind(state, owner, candidate) == kind
        })
    })
}

fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    view_as_skill: u32,
    view_as_card: u32,
    history: Option<&TriggerHistorySnapshot>,
) -> bool {
    use crate::state::ZoneKind;
    let Some(def) = state.content.find_def(active.trigger_def) else {
        return false;
    };
    let owner_is_player = active.owner == ctx.player;
    let owner_is_source = active.owner == ctx.source;
    let owner_source_distance_le_one = owner_is_source
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.alive)
        && matches!(
            query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(distance) if distance <= 1
        );
    let owner_source_target_distance_gt_one = owner_is_source
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.alive)
        && matches!(
            query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(distance) if distance > 1
        );
    let owner_source_wounded_distance_le_one = owner_source_distance_le_one
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.hp < player.max_hp as i32);
    let source_alive = state
        .players
        .get(ctx.source)
        .map(|player| player.alive)
        .unwrap_or(false);
    let owner_alive = state
        .players
        .get(active.owner)
        .map(|player| player.alive)
        .unwrap_or(false);
    let owner_hand_below_alive_count_capped_seven = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(7) as u32;
    let owner_hand_below_alive_count_capped_eight = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(8) as u32;
    let event_player_alive = state
        .players
        .get(ctx.player)
        .map(|player| player.alive)
        .unwrap_or(false);
    let event_player_distance_le_owner_hp = state
        .players
        .get(active.owner)
        .filter(|owner| owner.alive)
        .is_some_and(|owner| {
            event_player_alive
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: ctx.player,
                            to: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= owner.hp
                )
        });
    let unique_max_hand = |player_id| {
        state
            .players
            .get(player_id)
            .filter(|player| player.alive)
            .is_some_and(|_| {
                let hand = state.hand_count(player_id);
                state
                    .players
                    .iter_live()
                    .filter(|other| other.alive && other.id != player_id)
                    .all(|other| state.hand_count(other.id) < hand)
            })
    };
    let equip_count = |player_id| {
        state
            .zones
            .find(ZoneKind::Equip, player_id)
            .and_then(|zone| state.zones.get(zone))
            .map_or(0, |zone| zone.cards.len())
    };
    let source_is_max_equip = state
        .players
        .get(ctx.source)
        .filter(|source| source.alive)
        .is_some_and(|_| {
            let source_count = equip_count(ctx.source);
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| equip_count(player.id) <= source_count)
        });
    let owner_is_min_hp = state
        .players
        .get(active.owner)
        .filter(|player| player.alive)
        .map(|owner| {
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| owner.hp <= player.hp)
        })
        .unwrap_or(false);
    let owner_is_min_hand = state
        .players
        .get(active.owner)
        .filter(|player| player.alive)
        .map(|_| {
            let owner_hand = state.hand_count(active.owner);
            state
                .players
                .iter_live()
                .all(|player| owner_hand <= state.hand_count(player.id))
        })
        .unwrap_or(false);
    let hand_empty = state.hand_count(active.owner) == 0;
    let turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let play_phase_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.play_phase_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The currently modelled ordinary tricks mirror upstream cards whose `multitarget`
    // property is absent/false; `selectTarget: -1` alone does not make a card multitarget.
    let card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let single_sha_source_kind = ctx
        .card
        .and_then(|card| {
            state
                .resolution
                .sha_resolution_records
                .iter()
                .find(|record| record.source == ctx.source && record.card == card)
                .and_then(|record| match record.source_cards.as_slice() {
                    [] => Some(card),
                    [source] => Some(*source),
                    _ => None,
                })
        })
        .and_then(|card| state.cards.get(card))
        .map(|card| card.kind);
    let card_nature = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .zip(card_kind)
        .map(|(card, kind)| crate::query::effective_card_nature(state, ctx.source, card, kind));
    let card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let owner_was_targeted = ctx.targets.contains(&active.owner);
    let owner_equip_empty = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_none_or(|zone| zone.cards.is_empty());
    let owner_hand_below_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.hp.max(0) as u32);
    let owner_hand_above_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) > player.hp.max(0) as u32);
    let owner_hand_below_max_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.max_hp);
    let owner_hand = state.hand_cards(active.owner);
    let owner_hand_basic_count = owner_hand
        .iter()
        .filter(|card_id| {
            state.cards.get(**card_id).is_some_and(|card| {
                crate::query::effective_card_kind(state, active.owner, card).is_basic()
            })
        })
        .count();
    let owner_hand_basic_equals_non_basic = owner_hand_basic_count * 2 == owner_hand.len();
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let card_is_red = matches!(
        card_suit,
        Some(engine_api::Suit::Heart | engine_api::Suit::Diamond)
    );
    let card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let judgment_is_black = ctx.kind == EventKind::JudgmentResolved
        && ctx.cards.first().is_some_and(|result_card| {
            state.cards.get(*result_card).is_some_and(|card| {
                matches!(
                    crate::query::effective_card_suit(state, ctx.player, card),
                    engine_api::Suit::Spade | engine_api::Suit::Club
                )
            })
        });
    let card_is_diamond = card_suit == Some(engine_api::Suit::Diamond);
    let card_is_club = card_suit == Some(engine_api::Suit::Club);
    let previous_card_meta = history
        .map(|snapshot| snapshot.previous_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_META) as u32);
    let previous_card_kind = history
        .map(|snapshot| snapshot.previous_card_kind)
        .or_else(|| u8::try_from(state.turn.flag(turn_flag::LAST_CARD_KIND)).ok())
        .and_then(CardKind::from_tag);
    let previous_card_def = history
        .map(|snapshot| snapshot.previous_card_def)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_DEF) as u32);
    let previous_suit = previous_card_meta & 0xff;
    let previous_play_card_meta = history
        .map(|snapshot| snapshot.previous_play_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32);
    let previous_play_suit = previous_play_card_meta & 0xff;
    let previous_play_rank = (previous_play_card_meta >> 8) & 0xff;
    let previous_game_card_meta = history
        .map(|snapshot| snapshot.previous_game_card_meta)
        .unwrap_or_else(|| {
            state
                .turn
                .flag(turn_flag::last_game_card_meta_by_player(active.owner)) as u32
        });
    let previous_game_suit = previous_game_card_meta & 0xff;
    let previous_game_rank = (previous_game_card_meta >> 8) & 0xff;
    let current_suit = card_suit.map(crate::command::judge::suit_tag);
    let current_rank = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| u32::from(card.rank));
    let current_card_def = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| card.def.raw());
    let repeats_previous_card_name = previous_card_kind
        .zip(card_kind)
        .zip(current_card_def)
        .is_some_and(|((previous_kind, current_kind), current_def)| {
            same_logical_card_name(previous_kind, previous_card_def, current_kind, current_def)
        });
    let suit_is_red = |suit| matches!(suit, 2 | 4);
    let event_players_equal_hp = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .is_some_and(|(source, target)| source.hp == target.hp);
    let event_players_same_sex = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .and_then(|(source, target)| {
            state
                .content
                .character(source.character_def)
                .zip(state.content.character(target.character_def))
        })
        .is_some_and(|(source, target)| same_sex(&source.sex, &target.sex));
    let event_player_has_female_sex = state
        .players
        .get(ctx.player)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"));
    let event_source_has_male_sex = state
        .players
        .get(ctx.source)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"));
    let owner_has_discardable_equipment =
        crate::query::discardable_hand_equip_cards(state, active.owner)
            .into_iter()
            .any(|card| {
                state.cards.get(card).is_some_and(|card_state| {
                    crate::query::effective_card_kind(state, active.owner, card_state).is_equip()
                })
            });
    let owner_has_weapon = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_some_and(|zone| {
            zone.cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == CardKind::Weapon)
            })
        });
    let owner_event_player_same_group = state
        .player_group(active.owner)
        .is_some_and(|group| state.player_group(ctx.player) == Some(group));
    let owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    let event_player_group = state.player_group(ctx.player);
    let event_source_group = state.player_group(ctx.source);
    let owner_damaged_hp_le_source = state
        .players
        .get(active.owner)
        .zip(state.players.get(ctx.source))
        .is_some_and(|(owner, source)| owner.hp < owner.max_hp as i32 && owner.hp <= source.hp);
    let owner_alive_other_discarded_basic_outside_turn = owner_alive
        && ctx.kind == EventKind::CardsMoved
        && ctx.player.is_some()
        && ctx.player != active.owner
        && state.turn.current != active.owner
        && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
        && ctx.move_reason == CardMoveReason::Discard
        && ctx.moved_has_basic;
    let owner_received_gift_from_other = owner_alive
        && ctx.kind == EventKind::CardsMoved
        && ctx.source.is_some()
        && ctx.source != active.owner
        && ctx.move_reason == CardMoveReason::Give
        && ctx
            .to_zone
            .and_then(|zone| state.zones.get(zone))
            .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand);
    let owner_dealt_or_received_damage_this_turn = owner_alive
        && active
            .owner
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_u32.checked_shl(shift))
            .is_some_and(|bit| (ctx.turn_damage_sources | ctx.turn_damaged_players) & bit != 0);
    let owner_history_bit = active
        .owner
        .raw()
        .checked_sub(1)
        .and_then(|shift| 1_u32.checked_shl(shift));

    match &def.filter {
        TriggerFilter::Always => true,
        TriggerFilter::OwnerIsEventPlayerOtherHandNonempty => {
            owner_alive
                && owner_is_player
                && matches!(ctx.kind, EventKind::PhasePlay | EventKind::PhaseDraw)
                && (ctx.kind != EventKind::PhaseDraw || !state.turn.draw_phase_is_fixed())
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }
        TriggerFilter::OwnerIsEventPlayerPositiveDrawOtherHandNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
                && crate::query::draw_count_for(state, active.owner).saturating_add(
                    state.turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS).max(0) as u32,
                ) > 0
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }
        TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip => {
            let black = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| crate::query::effective_card_suit(state, active.owner, card))
                .is_some_and(|suit| {
                    matches!(suit, engine_api::Suit::Spade | engine_api::Suit::Club)
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
                && black
                && ctx.targets.iter().any(|target| {
                    *target != active.owner
                        && state
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                        && !crate::query::discardable_target_cards(
                            state,
                            active.owner,
                            *target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
        }
        TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable => {
            let card_gainable = ctx.card.is_some_and(|card| {
                state
                    .resolution
                    .sha_source_cards(ctx.source, card)
                    .into_iter()
                    .any(|card| {
                        state
                            .cards
                            .get(card)
                            .filter(|card| card.def != engine_api::CardDefId::NONE)
                            .and_then(|card| state.zones.get(card.zone))
                            .is_some_and(|zone| {
                                matches!(zone.kind, ZoneKind::Processing | ZoneKind::DiscardPile)
                            })
                    })
            });
            let target_has_armor_or_mount = state
                .zones
                .find(ZoneKind::Equip, ctx.player)
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| {
                    zone.cards.iter().any(|card| {
                        state.cards.get(*card).is_some_and(|card| {
                            matches!(
                                card.kind,
                                CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                            )
                        })
                    })
                });
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
                && card_gainable
                && target_has_armor_or_mount
        }
        TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Equip)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card| {
                        matches!(
                            card.kind,
                            CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                        )
                    })
                })
                && !state
                    .skill_pile_cards(ctx.player, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory => {
            let category = card_kind.map(|kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    4
                } else {
                    0
                }
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.remembered_category())
                    .zip(category)
                    .is_some_and(|(remembered, used)| remembered == used)
        }
        TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive && state.hand_count(player.id) > 0)
        }
        TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase => {
            let category = card_kind.map(|kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    4
                } else {
                    0
                }
            });
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && state.turn.phase == Phase::Play
                && state.turn.current == ctx.source
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.bound_category_target())
                    .zip(category)
                    .is_some_and(|((target, remembered), used)| {
                        target == ctx.source && remembered == used
                    })
        }
        TriggerFilter::OwnerPhaseGiftTargetIsEventSource => {
            owner_alive
                && ctx.kind == EventKind::Death
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.phase_gift_target())
                    == Some(ctx.source)
        }
        TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer => {
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.phase_gift_target())
                    == Some(ctx.player)
        }
        TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let sha_gainable = ctx.card.is_some_and(|card| {
                state.cards.get(card).is_some_and(|card_state| {
                    card_state.def != engine_api::CardDefId::NONE
                        && Some(card_state.zone) == discard
                })
            });
            let shan_gainable = ctx.cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| Some(card_state.zone) == discard)
            });
            let recipient_exists =
                state.players.alive_by_seat().into_iter().any(|recipient| {
                    recipient != active.owner && !ctx.targets.contains(&recipient)
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && (sha_gainable || shan_gainable)
                && recipient_exists
        }
        TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.source.is_some()
                && ctx.source != active.owner
                && !ctx.targets.contains(&active.owner)
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        card_state.def != engine_api::CardDefId::NONE
                            && Some(card_state.zone) == discard
                    })
                })
                && turn_fires < 1
        }
        TriggerFilter::OwnerIsEventPlayerTiedMaxHand => {
            let owner_hand = state.hand_count(active.owner);
            owner_alive
                && owner_is_player
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .all(|player| state.hand_count(player) <= owner_hand)
        }
        TriggerFilter::OwnerIsEventPlayerTiedMaxHp => {
            let owner_hp = state
                .players
                .get(active.owner)
                .map(|player| player.hp)
                .unwrap_or(i32::MIN);
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().all(|player| {
                    state
                        .players
                        .get(player)
                        .is_some_and(|participant| participant.hp <= owner_hp)
                })
        }
        TriggerFilter::OwnerIsEventSourceOrPlayerXinyaomingAvailable => {
            let used_mask = state
                .skills
                .get(active.skill_id)
                .map_or(u8::MAX, |skill| skill.xinyaoming_used_mask());
            ctx.kind == EventKind::Damage
                && owner_alive
                && (owner_is_source || owner_is_player)
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::xinyaoming_available_actions(
                        state,
                        active.owner,
                        target,
                        used_mask,
                    )
                    .is_empty()
                })
        }
        TriggerFilter::OwnerIsEventPlayerTiedMaxEquipNonempty => {
            let owner_equip = equip_count(active.owner);
            owner_alive
                && owner_is_player
                && owner_equip > 0
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .all(|player| equip_count(player) <= owner_equip)
        }
        TriggerFilter::OwnerIsEventPlayerSkippedPhaseCountPositive => {
            owner_alive && owner_is_player && state.turn.skipped_phase_count() > 0
        }
        TriggerFilter::OwnerDiscardedHasHandOncePerPhase => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.amount > 0
                && state.hand_count(active.owner) > 0
                && state.turn.phase != Phase::Idle
                && play_phase_fires == 0
        }
        TriggerFilter::OwnerPhaseEndLostCards => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.card_loss_count(active.owner) > 0
        }
        TriggerFilter::OwnerAliveSecondGlobalDying => {
            // 全局第二次濒死结算后（DyingAfter 时刻计数已含本次进入）。
            owner_alive
                && ctx.kind == EventKind::DyingAfter
                && state.lifetime_dying_events == 2
        }
        TriggerFilter::OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner => {
            // 粗遐：CardTargeted 逐目标 ctx（player=目标 owner，source=使用者）。
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && (state
                    .players
                    .get(ctx.source)
                    .is_some_and(|source| {
                        source.hp > state.players.get(active.owner).map(|o| o.hp).unwrap_or(0)
                    })
                    || state.damage_history.contains(&(ctx.source, active.owner)))
        }
        TriggerFilter::OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner => {
            // 未命：owner 体力更高，或使用者伤害过 owner。
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && (state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp > state.players.get(ctx.source).map(|s| s.hp).unwrap_or(0))
                    || state.damage_history.contains(&(ctx.source, active.owner)))
        }
        TriggerFilter::OwnerIsTargetSanshouTypeTracked => owner_alive && owner_is_player,
        TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp => {
            // 威临：自己回合内、目标本局未受过伤、owner 本局用牌数 ≥ 目标体力。
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && state.turn.current == active.owner
                && !state
                    .damage_history
                    .iter()
                    .any(|(_, damaged)| *damaged == ctx.player)
                && state
                    .card_action_totals
                    .get(&active.owner)
                    .copied()
                    .unwrap_or(0)
                    >= state
                        .players
                        .get(ctx.player)
                        .map(|p| p.hp.max(0) as u32)
                        .unwrap_or(0)
        }
        TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange => {
            // 直谏：任意角色结束阶段，owner 本回合用过/打出过牌、被指定为
            // 目标过且射程内有角色（上游 useCard||respond 双历史）。
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .turn
                    .flag(turn_flag::card_action_count_by_player(active.owner))
                    > 0
                && state.turn.card_targeted_count(active.owner) > 0
                && crate::query::all_other_players_in_attack_range(state, active.owner)
        }
        TriggerFilter::OwnerAliveLostHandThisTurn => {
            owner_alive
                && !owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state.turn.card_loss_count(active.owner) > 0
        }
        TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo => {
            // 精练：任意角色结束阶段，若你本回合至少失去过两张牌（回合级失牌计数）。
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.card_loss_count(active.owner) >= 2
        }
        TriggerFilter::OwnerSourceFirstShaThisPlayPhase => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.card_kind.is_some_and(|kind| {
                    crate::state::CardKind::from_tag(kind).is_some_and(|kind| kind.is_sha())
                })
                && state.turn.flag(turn_flag::sha_used_by_player(active.owner)) == 0
        }
        TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused => {
            let empty_slot = state.players.get(active.owner).is_some_and(|owner| {
                let equip = state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone));
                (1..=5).any(|tag| {
                    let slot = crate::state::EquipSlot::from_tag(tag).expect("valid equip slot");
                    owner.equip_slot_enabled(slot)
                        && equip.is_none_or(|zone| {
                            zone.cards.iter().all(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .and_then(|card| card.kind.equip_slot())
                                    != Some(slot)
                            })
                        })
                })
            });
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_source_distance_le_one
                && event_player_alive
                && empty_slot
                && turn_fires == 0
        }
        TriggerFilter::OwnerSourceFirstShaTargetThisTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && history.map_or_else(
                    || state.turn.flag(turn_flag::sha_used_by_player(active.owner)) == 1,
                    |history| history.owner_sha_used == 1,
                )
        }
        TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition => {
            let (branch, awakened, _) = state
                .skills
                .get(active.skill_id)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            let failed = match branch {
                Some(0) => {
                    state
                        .turn
                        .flag(turn_flag::play_card_category_history_by_player(ctx.player))
                        & 0b110
                        == 0
                }
                Some(1) => !state.turn.gained_card_in_play_phase(ctx.player),
                _ => false,
            };
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && ctx.player != active.owner
                && event_player_alive
                && failed
                && (awakened
                    || !crate::query::discardable_hand_equip_cards(state, ctx.player).is_empty())
        }
        TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && event_player_alive
                && ctx.player != active.owner
                && card_is_sha
                && turn_fires == 0
                && (!crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::Hand,
                )
                .is_empty()
                    || !crate::query::movable_field_cards_from(state, ctx.player).is_empty())
        }
        TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.targets.first() == Some(&ctx.player)
                && ctx.target_count > 0
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick())
        }
        TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && state.turn.current != active.owner
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_color_mask(state, active.owner, *card)
                                == 0b01
                    })
                })
        }
        TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.twkujian_cards());
            owner_alive
                && event_player_alive
                && ctx.player != active.owner
                && match ctx.kind {
                    EventKind::CardUsed | EventKind::CardResponded => {
                        ctx.card.is_some_and(|card| tracked.contains(&card))
                    }
                    EventKind::CardsMoved => {
                        ctx.from_zone == Some(ZoneKind::Hand)
                            && ctx.move_reason.is_use_or_respond()
                            && ctx.cards.iter().any(|card| tracked.contains(card))
                    }
                    _ => false,
                }
        }
        TriggerFilter::OwnerAliveOtherTwkujianLost => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.twkujian_cards());
            owner_alive
                && event_player_alive
                && ctx.player != active.owner
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && !ctx.move_reason.is_use_or_respond()
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }
        TriggerFilter::OwnerAliveTwruilianRoundUnprompted => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.round > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.twruilian_state().0 != state.turn.round)
        }
        TriggerFilter::OwnerAliveTwruilianSelectedTargetDiscarded => {
            let selected = state
                .skills
                .get(active.skill_id)
                .map_or(PlayerId::NONE, |skill| skill.twruilian_state().1);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && selected.is_some()
                && ctx.player == selected
                && state.turn.current == selected
                && ctx.move_reason == CardMoveReason::Discard
                && !ctx.cards.is_empty()
        }
        TriggerFilter::OwnerAliveTwruilianSelectedTargetTurnEnded => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.twruilian_state().1 == ctx.player)
        }
        TriggerFilter::OwnerAliveLostWeaponOutsideTurn => {
            let to_own_hand = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == active.owner);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && state.turn.current != active.owner
                && ctx.from_zone == Some(ZoneKind::Equip)
                && !to_own_hand
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.equip_slot() == Some(EquipSlot::Weapon))
                })
        }
        TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && ctx.player != active.owner
                && event_player_alive
                && !state.hand_cards(ctx.player).is_empty()
                && !state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_was_targeted
                && ctx.source != active.owner
                && card_is_sha
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card| !card.def.is_none())
                && state
                    .players
                    .get(ctx.source)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(source, owner)| source.alive && source.hp > owner.hp)
        }
        TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition => {
            let target_is_max_hp = state.players.alive_by_seat().iter().all(|player| {
                state.players.get(*player).map_or(true, |candidate| {
                    state
                        .players
                        .get(ctx.player)
                        .is_some_and(|target| target.hp >= candidate.hp)
                })
            });
            let target_is_max_hand = state
                .players
                .alive_by_seat()
                .iter()
                .all(|player| state.hand_count(ctx.player) >= state.hand_count(*player));
            let owner_low_hand = state.players.get(active.owner).is_some_and(|owner| {
                state.hand_count(active.owner) as i32 <= (owner.max_hp as i32 - owner.hp).max(0) + 1
            });
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.player != active.owner
                && event_player_alive
                && (card_is_sha || card_is_ordinary_trick)
                && state.resolution.effect_source_skill != active.skill_id.raw()
                && (target_is_max_hp || target_is_max_hand || owner_low_hand)
        }
        TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.dying_caused_count(active.owner) >= 2
        }
        TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.source.is_some()
                && ctx.source != active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == active.owner)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card| {
                        card.owner == active.owner
                            && state
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                    })
                })
        }
        TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map(|skill| skill.twkujian_cards())
                .unwrap_or_default();
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source.is_some()
                && ctx.source != active.owner
                && ctx.target_count == 1
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }
        TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map(|skill| skill.twkujian_cards())
                .unwrap_or_default();
            ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }
        TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        | TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            let dealt_damage =
                ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0;
            owner_alive
                && source_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.source != active.owner
                && ctx.card.is_some_and(|card| tracked.contains(&card))
                && (dealt_damage
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
                    ))
        }
        TriggerFilter::OwnerTrackedSkillCardLeftHandNotUseOrRespond => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            ctx.kind == EventKind::CardsMoved
                && ctx.player != active.owner
                && ctx.from_zone == Some(ZoneKind::Hand)
                && !ctx.move_reason.is_use_or_respond()
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }
        TriggerFilter::OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_sha)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && (ctx.source == active.owner
                    || crate::query::player_in_attack_range(state, active.owner, ctx.source))
        }
        TriggerFilter::OwnerAliveTrackedCuijinShaResolvedNoDamage
        | TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.source_free_physical_card());
            let dealt_damage =
                ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0;
            owner_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.card == tracked
                && (dealt_damage
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage
                    ))
                && (dealt_damage || source_alive)
        }
        TriggerFilter::OwnerAliveTrackedTwqingkouDuelResolved => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.source_free_physical_card());
            owner_alive
                && ctx.kind == EventKind::DuelResolved
                && ctx.respond_to == active.owner
                && ctx.card == tracked
                && ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0
        }
        TriggerFilter::OwnerAliveTwchenxunRoundUnprompted => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.round > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.twchenxun_round() != state.turn.round)
                && crate::command::play::source_free_card_has_legal_use(
                    state,
                    active.owner,
                    CardKind::TrickJuedou,
                )
        }
        TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
        | TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetNotDamaged => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.twchenxun_card());
            let target_damaged = ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE
                != 0
                && ctx.targets.first().copied() == Some(ctx.source);
            owner_alive
                && ctx.kind == EventKind::DuelResolved
                && ctx.respond_to == active.owner
                && ctx.card == tracked
                && (target_damaged
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
                    ))
        }
        TriggerFilter::OwnerAliveTrackedTwchenxunCardLeftProcessing => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.twchenxun_card());
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Processing)
                && tracked.is_some_and(|card| ctx.cards.contains(&card))
        }
        TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused => {
            let target_has_helper = state.skills.sorted_live().into_iter().any(|instance| {
                instance.owner == ctx.player
                    && instance.def == active.skill_def
                    && state.skill_effectively_enabled(instance)
            });
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && ctx.player != active.owner
                && ctx.amount > 0
                && !state.resolution.damage_redirect_in_progress
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
                && target_has_helper
                && state
                    .players
                    .get(ctx.player)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(target, owner)| target.alive && target.hp <= owner.hp)
        }
        TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && ctx.player == active.owner
                && state.skill_piles.iter().any(|pile| {
                    pile.owner == active.owner
                        && pile.usable_as_hand
                        && state
                            .zones
                            .get(pile.zone)
                            .is_some_and(|zone| !zone.cards.is_empty())
                })
        }
        TriggerFilter::OwnerUsedPlayableSkillPileCardUnused => {
            owner_alive
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && (ctx.source == active.owner || ctx.player == active.owner)
                && ctx.card.is_some_and(|card| {
                    state.turn.used_playable_skill_pile_card(active.owner, card)
                })
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }
        TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && ctx.player == active.owner
                && !state.skill_piles.iter().any(|pile| {
                    pile.owner == active.owner
                        && pile.usable_as_hand
                        && state
                            .zones
                            .get(pile.zone)
                            .is_some_and(|zone| !zone.cards.is_empty())
                })
        }
        TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused => {
            let min_hp = state
                .players
                .alive_by_seat()
                .into_iter()
                .filter_map(|player| state.players.get(player).map(|p| p.hp))
                .min();
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && Some(target.hp) == min_hp)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }
        TriggerFilter::OwnerAliveGlobalPhaseEnd => owner_alive && ctx.kind == EventKind::PhaseEnd,
        TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou => {
            let count = state.turn.card_target_count(ctx.player, active.owner);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && (count == 0
                    || (event_player_alive
                        && count as usize
                            <= crate::query::discardable_hand_equip_cards(state, active.owner)
                                .len()))
        }
        TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state.turn.damage_pair_count(ctx.player, active.owner) == 0
        }
        TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state.turn.card_target_count(ctx.player, active.owner) > 0
                && state.turn.damage_taken_count(active.owner) == 0
        }
        TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive)
                && state.turn.card_loss_count(active.owner) > 0
        }
        TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let has_discarded_card = state.turn.discarded_cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == discard)
            });
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && event_player_alive
                && (ctx.player != active.owner
                    || state
                        .players
                        .iter_live()
                        .any(|player| player.alive && player.id != active.owner))
                && has_discarded_card
        }
        TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed => {
            let token = state
                .turn
                .round
                .saturating_mul(16)
                .saturating_add(state.turn.current.raw());
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.view_as_use_round == token
                        && ctx.card.is_some_and(|card| {
                            skill.tracked_cards().contains(&card)
                                && state.cards.get(card).is_some_and(|card| {
                                    matches!(
                                        crate::query::effective_card_suit(
                                            state,
                                            active.owner,
                                            card
                                        ),
                                        engine_api::Suit::Heart | engine_api::Suit::Diamond
                                    )
                                })
                        })
                })
        }
        TriggerFilter::OwnerIsTargetEventCardJiu => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Jiu)
        }
        TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable => {
            let has_jiu = state.players.get(active.owner).is_some_and(|player| {
                player
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_JIU && *count > 0)
            });
            let has_discardable_hand =
                crate::query::discardable_hand_equip_cards(state, active.owner)
                    .into_iter()
                    .any(|card| {
                        state.cards.get(card).is_some_and(|card| {
                            state
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                        })
                    });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && ctx.source.is_some()
                && ctx.source != active.owner
                && card_is_black
                && (!has_jiu || has_discardable_hand)
        }
        TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player == active.owner
                && state
                    .turn
                    .flag(turn_flag::card_used_by_player(active.owner))
                    > 0
                && state.turn.damage_dealt_count(active.owner) == 0
        }
        TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused => {
            let to_discard = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && to_discard
                && !ctx.move_reason.is_use_or_respond()
                && !ctx.cards.is_empty()
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }
        TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDiscardEnd
                && state.turn.flag(turn_flag::DISCARD_PHASE_DISCARDED) >= 2
        }
        TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase => {
            owner_alive
                && ctx.kind == EventKind::PhaseDiscardEnd
                && ctx.player != active.owner
                && crate::effect::discard_phase_discards_for(state, ctx.player)
                    .next()
                    .is_some()
        }
        TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange => {
            // 整肃三技能定义（朱儁侯封 / 吕范燕集共用，ABI 432）。
            const ZHENGSU_DEFS: [u32; 3] = [2_584_022_884, 3_044_720_297, 3_824_072_820];
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && zhengsu_missing_count(state, ctx.player, &ZHENGSU_DEFS) > 0
                && crate::effect::dcchiying_in_range(state, active.owner, ctx.player)
        }
        TriggerFilter::OwnerIsTargetMissingZhengsu => {
            const ZHENGSU_DEFS: [u32; 3] = [2_584_022_884, 3_044_720_297, 3_824_072_820];
            owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && zhengsu_missing_count(state, active.owner, &ZHENGSU_DEFS) > 0
        }
        TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && state.player_group(ctx.player) == Some("wu")
                && state.player_group(active.owner).is_some()
                && !state.hand_cards(ctx.player).is_empty()
                && !state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.target_count > 1
                && card_is_trick
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerIsTargetFeijunNewTarget => {
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && owner_is_player
                && state
                    .turn
                    .flag(turn_flag::feijun_new_target_by_player(active.owner))
                    != 0
        }
        TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && card_is_sha
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|participant| participant.alive)
                && !state.hand_cards(active.owner).is_empty()
                && !state.hand_cards(ctx.player).is_empty()
        }
        TriggerFilter::OwnerAliveOtherNanmanCardsDiscard => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source != active.owner
                && ctx.card_kind == Some(24)
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        state
                            .zones
                            .find(ZoneKind::DiscardPile, PlayerId::NONE)
                            .is_some_and(|zone| card_state.zone == zone)
                    })
                })
        }
        TriggerFilter::OwnerAliveSkillBoundTargetIsEventSource => {
            owner_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|instance| instance.bound_target() == Some(ctx.source))
        }
        TriggerFilter::OwnerLostCardsOutsideTurn => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && state.turn.current != active.owner
                && ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && ctx.to_zone.is_some_and(|to| {
                    state
                        .zones
                        .get(to)
                        .is_some_and(|zone| zone.owner != active.owner)
                })
                && !ctx.cards.is_empty()
        }
        TriggerFilter::OwnerPhaseBeginSkillPileGteThree(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && owner_is_player
                && state.turn.current == active.owner
                && state.skill_pile_cards(active.owner, *skill_def).len() >= 3
        }
        TriggerFilter::OwnerPhaseBeginHandEmpty => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && owner_is_player
                && state.turn.current == active.owner
                && state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerRespondedShan => {
            owner_alive
                && ctx.kind == EventKind::CardResponded
                && owner_is_player
                && ctx.card_kind == Some(3)
        }
        TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && state.player_group(ctx.player) == Some("qun")
                && state.hand_cards(ctx.player).iter().any(|card| {
                    state.cards.get(*card).is_some_and(|value| {
                        let kind = crate::query::effective_card_kind(state, ctx.player, value);
                        matches!(kind, CardKind::Shan | CardKind::TrickShandian)
                    })
                })
        }
        TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused => {
            const POTJIEJIE_USED_DEF: u32 = 4_200_000_554;
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player == state.turn.current
                && event_player_alive
                && !state.hand_cards(ctx.player).is_empty()
                && !state
                    .players
                    .get(ctx.player)
                    .into_iter()
                    .flat_map(|player| player.skills.iter())
                    .any(|skill| {
                        state
                            .skills
                            .get(*skill)
                            .is_some_and(|instance| instance.def.raw() == POTJIEJIE_USED_DEF)
                    })
        }
        TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && card_is_sha
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && state.players.get(active.owner).is_some_and(|owner| {
                            owner.hp <= target.hp
                                || state.hand_count(active.owner) <= state.hand_count(ctx.player)
                        })
                })
        }
        TriggerFilter::OwnerSourceDamageWithinOne => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && ctx.amount > 0
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive)
                && matches!(
                    crate::query::query(
                        state,
                        &QueryRequest::Distance {
                            from: active.owner,
                            to: ctx.player,
                        },
                        &mut QueryContext::default(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        }
        TriggerFilter::OwnerGainedCardsPotzhouyu => {
            let gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive
                && gained
                && ctx.amount > 0
                && state.hand_count(active.owner) > 0
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.content_flags & 0b110 == 0 || state.turn.current == active.owner
                })
        }
        TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.targets.first() == Some(&ctx.player)
                && ctx
                    .targets
                    .iter()
                    .any(|target| state.hand_count(*target) > 0)
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.content_flags & 0b110 == 0 || state.turn.current == active.owner
                })
        }
        TriggerFilter::OwnerAliveFireDamagePotfentao => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && ctx.nature == rule_api::damage_nature::FIRE
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.content_flags & 0b110 == 0 || state.turn.current == active.owner
                })
        }
        TriggerFilter::OwnerAlivePotfentaoPendingDamage => {
            owner_alive
                && matches!(ctx.kind, EventKind::Damage | EventKind::DamagePrevented)
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.starts_with(b"PFT1")
                        && skill.state_blob.get(4..8).is_some_and(|raw| {
                            u32::from_le_bytes(raw.try_into().unwrap()) == ctx.player.raw()
                        })
                })
        }
        TriggerFilter::OwnerAliveTurnEndedRehengjiangPending => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.len() == 12
                        && &skill.state_blob[..4] == b"RHJ1"
                        && u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap())
                            == ctx.player.raw()
                        && u32::from_le_bytes(skill.state_blob[8..12].try_into().unwrap()) > 0
                })
        }
        TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending => {
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && ctx.player == active.owner
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.len() == 8 && &skill.state_blob[..4] == b"ZBI1" && {
                        let target = PlayerId::new(u32::from_le_bytes(
                            skill.state_blob[4..8].try_into().unwrap(),
                        ));
                        target.is_some()
                    }
                })
        }
        TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && source_alive
                && !state.hand_cards(ctx.source).is_empty()
        }
        TriggerFilter::OwnerSourceQiaojianExtraAvailable => {
            let category = card_kind.and_then(|kind| {
                if kind.is_sha() {
                    Some(1u8)
                } else if kind.is_trick() && !kind.is_delayed_trick() {
                    Some(2u8)
                } else {
                    None
                }
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && state.skills.get(active.skill_id).is_some_and(|helper| {
                    helper.state_blob.len() == 6
                        && helper.temporary_until_turn_end_of == active.owner
                        && helper.state_blob[4] & 1 != 0
                        && category.is_some_and(|bit| helper.state_blob[5] & bit == 0)
                })
        }
        TriggerFilter::OwnerXichaWatchedSourceCardUsed => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.source.is_some()
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| xicha_state_has_source(&skill.state_blob, ctx.source))
        }
        TriggerFilter::OwnerQiaojianTrackedCardLeftHand => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            let owner_hand = state.zones.find(ZoneKind::Hand, active.owner);
            ctx.kind == EventKind::CardsMoved
                && ctx.cards.iter().any(|card| {
                    tracked.contains(card)
                        && state
                            .cards
                            .get(*card)
                            .is_some_and(|card| Some(card.zone) != owner_hand)
                })
        }
        TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive)
                && (ctx.player == active.owner
                    || crate::query::player_in_attack_range(state, active.owner, ctx.player))
                && ctx.source != active.owner
                && ctx.card.is_some_and(|card| {
                    crate::query::physical_card_number_sum(state, card, &ctx.cards).is_some()
                })
        }
        TriggerFilter::OwnerSourceWoundedExistsOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && turn_fires == 0
                && state
                    .players
                    .iter_live()
                    .any(|target| target.alive && target.hp < target.max_hp as i32)
        }
        TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner => {
            let damage_tagged = ctx.card.is_some_and(|card| {
                state.cards.get(card).is_some_and(|value| {
                    let kind = crate::query::effective_card_kind(state, active.owner, value);
                    kind.is_sha() || kind.is_damage_trick()
                })
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
                && damage_tagged
                && ctx.targets.iter().any(|target| {
                    *target != active.owner
                        && state
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                        && state.hand_count(*target) >= state.hand_count(active.owner)
                        && !crate::query::discardable_hand_equip_cards(state, *target).is_empty()
                })
        }
        TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.card_kind == Some(24)
                && (ctx.targets.iter().any(|target| {
                    state
                        .players
                        .get(*target)
                        .is_some_and(|participant| !participant.alive)
                }) || {
                    // amount 为受创目标位掩码（dispatch 对 CardResolved 的编码）。
                    let damaged_count = (0..ctx.targets.len())
                        .filter(|index| ctx.amount & (1u32 << index) != 0)
                        .count();
                    damaged_count > 3
                })
        }
        TriggerFilter::OwnerLostNonDamageCardPottuntianChargeNotFull => {
            let max_charge = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.state_blob.get(..4))
                .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
                .unwrap_or(0);
            let charge = state
                .players
                .get(active.owner)
                .and_then(|player| {
                    player
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map(|(_, count)| (*count).max(0) as u32)
                })
                .unwrap_or(0);
            let moved: Vec<_> = ctx
                .cards
                .iter()
                .filter_map(|card| state.cards.get(*card))
                .collect();
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && ctx.to_zone.is_some_and(|to| {
                    state
                        .zones
                        .get(to)
                        .is_some_and(|zone| zone.owner != active.owner)
                })
                && charge < max_charge
                && moved
                    .iter()
                    .any(|card| !card.kind.is_sha() && !card.kind.is_damage_trick())
                && !(ctx.move_reason == CardMoveReason::Use
                    && moved.iter().all(|card| card.kind.is_equip()))
        }
        TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.state_blob.get(..4))
                .map(|bytes| CardId::new(u32::from_le_bytes(bytes.try_into().unwrap())));
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0
                && tracked.is_some_and(|tracked| ctx.card == Some(tracked))
        }
        TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable => {
            let (quota, seen) = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| {
                    (skill.state_blob.len() >= 3)
                        .then(|| (skill.state_blob[1], skill.state_blob[2]))
                })
                .unwrap_or((0, 0));
            let marks = state
                .players
                .get(active.owner)
                .and_then(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map(|(_, count)| (*count).max(0) as u32)
                })
                .unwrap_or(0);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && seen < quota
                && marks < 6
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card| {
                        card.kind.is_sha()
                            && state
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                    })
                })
        }
        TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw => {
            let tracked = state.skills.get(active.skill_id).and_then(|skill| {
                (skill.state_blob.len() >= 8 && skill.state_blob[3] != 0).then(|| {
                    CardId::new(u32::from_le_bytes(
                        skill.state_blob[4..8].try_into().unwrap(),
                    ))
                })
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && tracked.is_some_and(|tracked| ctx.card == Some(tracked))
        }
        TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun => {
            owner_alive
                && ctx.kind == EventKind::PhaseDiscard
                && owner_is_player
                && ctx.player == state.turn.current
                && {
                    let limit = crate::query::query(
                        state,
                        &rule_api::QueryRequest::HandLimit {
                            player: active.owner,
                        },
                        &mut crate::query::QueryContext::new(),
                    );
                    let limit = match limit {
                        rule_api::QueryResponse::Int(value) => value.max(0) as u32,
                        _ => 0,
                    };
                    state.hand_count(active.owner) > limit
                        && state.players.alive_by_seat().into_iter().any(|candidate| {
                            candidate != active.owner
                                && state.player_group(candidate) == Some("qun")
                        })
                }
        }
        TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.target_count == 1
                && (card_is_sha || (card_is_ordinary_trick && card_is_black))
                && play_phase_fires == 0
        }
        TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_player
                && ctx.source.is_some()
                && event_source_has_male_sex
                && owner_has_discardable_equipment
        }
        TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && ctx.player != active.owner
                && event_player_alive
                && ctx.card_flags & 1 == 0
        }
        TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && ctx.player != active.owner
                && event_player_alive
                && ctx.card_flags & 2 == 0
        }
        TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && ctx.player != active.owner
                && event_player_alive
                && ctx.card_flags & 1 != 0
                && ctx.card_flags & 2 == 0
        }
        TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian => {
            owner_alive
                && ctx.kind == EventKind::PhaseAnyEnd
                && owner_history_bit.is_some_and(|bit| {
                    ctx.turn_damage_sources & bit != 0 && ctx.turn_damaged_players & bit != 0
                })
                && crate::command::play::source_free_card_has_legal_use(
                    state,
                    active.owner,
                    CardKind::TrickWanjian,
                )
        }
        TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou => {
            owner_alive
                && ctx.kind == EventKind::PhaseAnyEnd
                && owner_history_bit.is_some_and(|bit| {
                    ctx.turn_damage_sources & bit != 0 && ctx.turn_damaged_players & bit == 0
                })
                && crate::command::play::source_free_card_has_legal_use(
                    state,
                    active.owner,
                    CardKind::TrickJuedou,
                )
        }
        TriggerFilter::OwnerSourceShaHasWeapon => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
                && owner_has_weapon
        }
        TriggerFilter::OwnerSourceShaDirectDamageBefore => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
        }
        TriggerFilter::OwnerSourceOtherDamageBefore => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
        }
        TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && turn_fires == 0
        }
        TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && turn_fires == 0
                && state.players.get(active.owner).is_some_and(|owner| {
                    state.players.get(ctx.player).is_some_and(|target| {
                        owner.seat.raw() > target.seat.raw()
                            && state.hand_count(active.owner) < target.max_hp
                    })
                })
        }
        TriggerFilter::OwnerSourceOtherDirectDamageBefore => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
        }
        TriggerFilter::OwnerIsTargetDiscardableHandEquip => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_player
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && (owner_is_source || owner_is_player)
                && turn_fires == 0
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| state.hand_count(target) != state.hand_count(active.owner))
        }
        TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| state.hand_count(target) > 0)
        }
        TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| state.hand_count(target) == 0)
        }
        TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.players.alive_by_seat().into_iter().any(|target| {
                    target != active.owner
                        && state.hand_count(target) <= state.hand_count(active.owner)
                })
        }
        TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded => {
            owner_alive
                && owner_is_player
                && owner_wounded
                && ctx.kind == EventKind::PhasePrepare
                && state.draw_reshuffle_count == 0
        }
        TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state.draw_reshuffle_count > 0
        }
        TriggerFilter::OwnerHandCompareParticipant => {
            owner_alive
                && ctx.kind == EventKind::HandCompareStarted
                && (owner_is_player || owner_is_source)
        }
        TriggerFilter::OwnerHandCompareFinishedParticipant => {
            owner_alive
                && ctx.kind == EventKind::HandCompareFinished
                && (owner_is_player || owner_is_source)
        }
        TriggerFilter::OwnerHandCompareParticipantHasSha => {
            owner_alive
                && ctx.kind == EventKind::HandCompareFinished
                && (owner_is_player || owner_is_source)
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.is_sha())
                })
        }
        TriggerFilter::OwnerHandCompareParticipantResultCardInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::HandCompareFinished
                && hand_compare_result_card(
                    active.owner,
                    ctx.player,
                    ctx.source,
                    ctx.respond_to,
                    &ctx.cards,
                )
                .and_then(|card| state.cards.get(card))
                .is_some_and(|card| Some(card.zone) == discard)
        }
        TriggerFilter::OwnerDiscardedBlackBasicToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && owner_is_player
                && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_basic()
                            && crate::query::effective_card_color_mask(state, active.owner, *card)
                                == 0b10
                    })
                })
        }
        TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_basic()
                            && crate::query::effective_card_color_mask(state, active.owner, *card)
                                == 0b10
                    })
                })
        }
        TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard => {
            if ctx.kind != EventKind::PhaseJudge
                || !owner_alive
                || !event_player_alive
                || owner_is_player
                || crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::Judge,
                )
                .is_empty()
            {
                return false;
            }
            let distance = query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            );
            let range = query(
                state,
                &QueryRequest::AttackRange {
                    player: active.owner,
                },
                &mut QueryContext::new(),
            );
            matches!(
                (distance, range),
                (QueryResponse::Int(distance), QueryResponse::Int(range)) if distance <= range
            )
        }
        TriggerFilter::OwnerCardRespondedToByPlayerInRange => {
            if ctx.kind != EventKind::CardResponded
                || !owner_alive
                || !event_player_alive
                || ctx.respond_to != active.owner
            {
                return false;
            }
            let mut query_ctx = QueryContext::new();
            let distance = query(
                state,
                &QueryRequest::Distance {
                    from: ctx.player,
                    to: active.owner,
                },
                &mut query_ctx,
            );
            let range = query(
                state,
                &QueryRequest::AttackRange { player: ctx.player },
                &mut query_ctx,
            );
            matches!(
                (distance, range),
                (QueryResponse::Int(distance), QueryResponse::Int(range)) if distance <= range
            )
        }
        TriggerFilter::OwnerAliveFirstCardsMovedToHand => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                && turn_fires == 0
        }
        TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn => {
            let target = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .and_then(|zone| {
                    (zone.kind == ZoneKind::Hand && zone.owner != active.owner)
                        .then_some(zone.owner)
                });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && turn_fires == 0
                && target.is_some_and(|target| {
                    state
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                        && crate::query::alive_seat_neighbors(state, active.owner).contains(&target)
                        && !(state.turn.current == target && state.turn.phase == Phase::Draw)
                        && !crate::query::gainable_target_cards(
                            state,
                            active.owner,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
        }
        TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf => {
            owner_alive
                && ctx.kind == EventKind::CardsDrawn
                && owner_is_player
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && state.resolution.effect_source_skill != active.skill_id.raw()
        }
        TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && ctx.target_count > 0
                && card_kind.is_some_and(|kind| !kind.is_equip())
                && owner_hand_basic_count == 0
        }
        TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded => {
            let entered_owner_equip = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::Equip && zone.owner == active.owner);
            let left_owner_equip = owner_is_player && ctx.from_zone == Some(ZoneKind::Equip);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && crate::query::alive_wounded_count(state) >= 2
                && (entered_owner_equip || left_owner_equip)
        }
        TriggerFilter::OwnerAliveEquipUsed => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_equip)
        }
        TriggerFilter::OwnerAliveEquipUsedSkillMarkBelowThree => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_equip)
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map_or(0, |(_, count)| *count)
                        < 3
                })
        }
        TriggerFilter::OwnerAliveSourceAliveOncePerRound => owner_alive && source_alive,
        TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.dying && target.hp <= 0)
        }
        TriggerFilter::OwnerAliveOncePerRound => owner_alive,
        TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound => {
            owner_alive
                && owner_is_player
                && crate::query::discardable_hand_equip_cards(state, active.owner).len() >= 2
        }
        TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| !skill.oldhuisheng_sources().contains(&ctx.source))
        }
        TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state.turn.current != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive && player.dying && player.hp <= 0)
                && !crate::command::play::oldjiefan_sha_source_cards(
                    state,
                    active.owner,
                    state.turn.current,
                )
                .is_empty()
        }
        TriggerFilter::OwnerSourceOldJiefanSha => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::OLDJIEFAN != 0
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.oldjiefan_dying())
                    .is_some()
        }
        TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip => {
            owner_alive
                && owner_is_source
                && state.turn.current == active.owner
                && ctx.kind == EventKind::CardTargeted
                && !card_kind.is_some_and(CardKind::is_equip)
                && ctx.targets.first() == Some(&ctx.player)
        }
        TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable => {
            let source = history
                .map(|snapshot| {
                    (
                        SkillDefId::new(snapshot.view_as_skill_def),
                        PlayerId::new(snapshot.view_as_skill_owner),
                    )
                })
                .or_else(|| {
                    state
                        .skills
                        .get(SkillId::new(view_as_skill))
                        .map(|skill| (skill.def, skill.owner))
                });
            let used_face_down = source.is_some_and(|(skill_def, skill_owner)| {
                skill_owner == active.owner
                    && state
                        .content
                        .view_as_defs_for_skill(skill_def)
                        .iter()
                        .any(|rule| rule.cost.as_deref() == Some("face_down_hand"))
            });
            let mode = state
                .skills
                .get(active.skill_id)
                .map_or(0, |skill| skill.view_as_branch);
            let branch_available = if mode == 0 {
                !state.hand_cards(active.owner).is_empty()
            } else {
                state.players.iter_live().any(|target| {
                    target.alive
                        && target.id != active.owner
                        && !crate::query::discardable_target_cards(
                            state,
                            active.owner,
                            target.id,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            };
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && used_face_down
                && branch_available
        }
        TriggerFilter::OwnerAlive => owner_alive,
        TriggerFilter::OwnerAliveKilledThisTurn => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .turn
                    .flag(turn_flag::kill_count_by_player(active.owner))
                    > 0
        }
        TriggerFilter::OwnerAliveHandBelowAliveCountCappedSeven => {
            owner_hand_below_alive_count_capped_seven
        }
        TriggerFilter::OwnerAliveHandBelowAliveCountCappedEight => {
            owner_hand_below_alive_count_capped_eight
        }
        TriggerFilter::OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_kind.is_some_and(CardKind::is_delayed_trick)
                && !state.pending_judge_skips.contains(&active.owner)
        }
        TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && ctx.target_count == 1
                && card_kind.is_some_and(|kind| {
                    if kind.is_sha() {
                        history
                            .map(|snapshot| snapshot.owner_sha_used)
                            .unwrap_or_else(|| {
                                state
                                    .turn
                                    .flag(turn_flag::sha_used_by_player(active.owner))
                                    .max(0) as u32
                            })
                            == 0
                    } else if kind == CardKind::TrickJuedou {
                        history
                            .map(|snapshot| snapshot.owner_juedou_used)
                            .unwrap_or_else(|| {
                                state
                                    .turn
                                    .flag(turn_flag::juedou_used_by_player(active.owner))
                                    .max(0) as u32
                            })
                            == 0
                    } else {
                        false
                    }
                })
        }
        TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.target_count == 1
                && card_kind.is_some_and(|kind| kind.is_sha() || kind == CardKind::TrickJuedou)
                && state.hand_count(ctx.player) > 0
        }
        TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && [active.owner, ctx.player].into_iter().any(|endpoint| {
                    !crate::query::discardable_target_cards(
                        state,
                        active.owner,
                        endpoint,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
        }
        TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty => {
            let view_card = view_as_card;
            let source = history
                .map(|snapshot| {
                    (
                        SkillDefId::new(snapshot.view_as_skill_def),
                        PlayerId::new(snapshot.view_as_skill_owner),
                    )
                })
                .or_else(|| {
                    state
                        .skills
                        .get(SkillId::new(view_as_skill))
                        .map(|skill| (skill.def, skill.owner))
                });
            let converted_by_longdan = source.is_some_and(|(skill_def, skill_owner)| {
                skill_owner == active.owner
                    && state
                        .content
                        .skill_metadata(skill_def)
                        .is_some_and(|metadata| {
                            metadata.name.contains("longdan") || metadata.name.contains("fanghun")
                        })
            });
            let event_card_matches = ctx
                .card
                .map_or(view_card == 0, |card| card.raw() == view_card);
            let first_counterpart = ctx.targets.first().copied();
            owner_alive
                && event_player_alive
                && converted_by_longdan
                && event_card_matches
                && matches!(card_kind, Some(CardKind::Sha | CardKind::Shan))
                && first_counterpart.is_some_and(|counterpart| {
                    counterpart != active.owner
                        && state
                            .players
                            .get(counterpart)
                            .is_some_and(|player| player.alive)
                        && state.hand_count(counterpart) > 0
                })
                && match ctx.kind {
                    EventKind::CardTargeted => {
                        owner_is_source && first_counterpart == Some(ctx.player)
                    }
                    EventKind::CardResponded => {
                        owner_is_player && first_counterpart == Some(ctx.respond_to)
                    }
                    _ => false,
                }
        }
        TriggerFilter::OwnerAliveEventPlayerAlive => owner_alive && event_player_alive,
        TriggerFilter::OwnerAliveEventPlayerEquipEmpty => {
            owner_alive && event_player_alive && equip_count(ctx.player) == 0
        }
        TriggerFilter::OwnerAliveHandHasSameKindAsEventCard => {
            owner_alive && hand_has_same_kind_as_event_card(state, active.owner, ctx)
        }
        TriggerFilter::OwnerAliveHasSkillTaggedCards => owner_alive
            && state
                .hand_cards(active.owner)
                .into_iter()
                .any(|card| state.card_has_content_tag(card, active.skill_def.raw())),
        TriggerFilter::OwnerAliveTaggedCardLostThisTurn => owner_alive
            && state.turn.lost_card_tag(active.skill_def.raw()),
        TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty => {
            owner_alive
                && event_player_alive
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Equip))
                && equip_count(ctx.player) == 0
        }
        TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !history
                    .map(|snapshot| snapshot.event_player_had_other_red_card_target)
                    .unwrap_or_else(|| {
                        ctx.card.is_some_and(|card| {
                            state.turn.was_targeted_by_other_red_card(ctx.player, card)
                        })
                    })
        }
        TriggerFilter::OwnerAliveEventPlayerHandEmpty => {
            owner_alive && event_player_alive && state.hand_count(ctx.player) == 0
        }
        TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty => {
            owner_alive
                && event_player_alive
                && ctx.player != state.turn.current
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && state.hand_count(ctx.player) == 0
        }
        TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn => {
            owner_alive
                && event_player_alive
                && ctx.player != state.turn.current
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && state.hand_count(ctx.player) == 0
                && turn_fires == 0
        }
        TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp => event_player_distance_le_owner_hp,
        TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && ctx.nature == rule_api::damage_nature::NORMAL
                && state.players.get(ctx.player).is_some_and(|player| {
                    !player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }
        TriggerFilter::OwnerAliveEquipSourceMaxEquip => {
            owner_alive && card_kind.is_some_and(CardKind::is_equip) && source_is_max_equip
        }
        TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName => {
            owner_alive
                && !owner_is_source
                && ctx.source == state.turn.current
                && repeats_previous_card_name
        }
        TriggerFilter::OwnerAliveNotEventPlayerNeitherUniqueMaxHand => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && !unique_max_hand(active.owner)
                && !unique_max_hand(ctx.player)
        }
        TriggerFilter::OwnerAliveSourceTargetSameSex => owner_alive && event_players_same_sex,
        TriggerFilter::OwnerAliveEventPlayerSameGroup => {
            owner_alive && owner_event_player_same_group
        }
        TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty => {
            owner_alive && owner_event_player_same_group && state.hand_count(active.owner) > 0
        }
        TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange => {
            let tracked = state.skills.get(active.skill_id).is_some_and(|instance| {
                ctx.card
                    .is_some_and(|card| instance.tracked_cards().contains(&card))
                    && instance
                        .active_play_phase_targets()
                        .is_some_and(|targets| targets.contains(&ctx.player))
            });
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && card_kind == Some(CardKind::TrickJuedou)
                && tracked
                && state.players.iter_live().any(|candidate| {
                    candidate.alive
                        && candidate.id != active.owner
                        && candidate.hp < candidate.max_hp as i32
                        && crate::query::player_in_attack_range(state, ctx.player, candidate.id)
                })
        }
        TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.hand_count(active.owner) > 0
                && state.players.iter_live().any(|candidate| {
                    candidate.alive
                        && candidate.id != active.owner
                        && state.hand_count(candidate.id) > 0
                })
        }
        TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_is_ordinary_trick
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|instance| crate::effect::cilv_used_mask(instance) != 0b111)
        }
        TriggerFilter::CilvTrackedCardDamageBefore => {
            ctx.kind == EventKind::DamageBefore
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::cilv_tracks_card(instance, card, 0b010)
                    })
                })
        }
        TriggerFilter::CilvTrackedCardResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::cilv_tracks_card(instance, card, 0b110)
                    })
                })
        }
        TriggerFilter::OwnerDyingZhupeilanOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.dying && owner.hp <= 0)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|instance| instance.lifetime_fire_count == 0)
        }
        TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable => {
            let event_hand = state.hand_count(ctx.player);
            let max_hand = state
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| state.hand_count(player))
                .max()
                .unwrap_or(0);
            let owner_cards = !state.hand_cards(active.owner).is_empty()
                || state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty());
            let target_cards = !crate::query::discardable_target_cards(
                state,
                active.owner,
                ctx.player,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty();
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && event_player_alive
                && event_hand == max_hand
                && (owner_cards
                    || target_cards
                    || crate::command::play::source_free_sha_can_target_without_distance(
                        state,
                        active.owner,
                        ctx.player,
                    ))
        }
        TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.entered_discard_cards.iter().any(|card| {
                    !state.turn.damage_card_materials.contains(card)
                        && state.cards.get(*card).is_some_and(|card| {
                            Some(card.zone) == discard
                                && (card.kind.is_sha() || card.kind.is_damage_trick())
                        })
                })
        }
        TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && ctx.nature == rule_api::damage_nature::FIRE
        }
        TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        let kind =
                            crate::query::effective_card_kind(state, active.owner, card_state);
                        kind == CardKind::TrickHuogong
                            || kind.is_sha()
                                && crate::query::effective_card_nature(
                                    state,
                                    active.owner,
                                    card_state,
                                    kind,
                                ) == rule_api::damage_nature::FIRE
                    })
                })
        }
        TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn => {
            let other_targets: Vec<_> = ctx
                .targets
                .iter()
                .copied()
                .filter(|target| {
                    *target != active.owner
                        && state
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                })
                .collect();
            let yin = state
                .skills
                .get(active.skill_id)
                .is_some_and(|instance| instance.view_as_branch.min(1) == 1);
            let has_yin_pair = !yin
                || other_targets.iter().any(|target| {
                    let max_hand = state
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .map(|player| state.hand_count(player))
                        .max()
                        .unwrap_or(0);
                    state
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .any(|source| source != *target && state.hand_count(source) == max_hand)
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && turn_fires == 0
                && !other_targets.is_empty()
                && has_yin_pair
        }
        TriggerFilter::DcsbyingmouTrackedCardResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::dcsbyingmou_tracks_card(instance, card)
                    })
                })
        }
        TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable => {
            let destination = ctx.to_zone.and_then(|zone| state.zones.get(zone));
            let destination_owner = destination.map(|zone| zone.owner).unwrap_or(PlayerId::NONE);
            let counterpart = if ctx.source == active.owner {
                destination_owner
            } else if destination_owner == active.owner {
                ctx.source
            } else {
                PlayerId::NONE
            };
            let branch = state
                .skills
                .get(active.skill_id)
                .map_or(0, |instance| instance.view_as_branch.min(1));
            let used = state
                .turn
                .flag(crate::state::turn_flag::dcsbmengmou_used_by_player(
                    active.owner,
                )) as u32;
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && destination.is_some_and(|zone| zone.kind == ZoneKind::Hand)
                && counterpart.is_some()
                && counterpart != active.owner
                && state
                    .players
                    .get(counterpart)
                    .is_some_and(|participant| participant.alive)
                && used & (1u32 << branch) == 0
                && !ctx.cards.is_empty()
        }
        TriggerFilter::DcsbmengmouTrackedShaDamage => {
            ctx.kind == EventKind::Damage
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::dcsbmengmou_tracks_sha(instance, ctx.source, card)
                    })
                })
        }
        TriggerFilter::DcsbmengmouTrackedShaResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::dcsbmengmou_tracks_sha(instance, ctx.source, card)
                    })
                })
        }
        TriggerFilter::OwnerIsTargetCardDamageBefore => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && (ctx.card.is_some() || ctx.card_kind.is_some())
        }
        TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha => {
            let target = ctx.targets.first().copied().unwrap_or(PlayerId::NONE);
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && ctx.target_count == 1
                && ctx.card_flags & 1 != 0
                && !ctx.cards.is_empty()
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_trick())
                && crate::command::play::source_free_sha_can_target_without_distance(
                    state,
                    active.owner,
                    target,
                )
        }
        TriggerFilter::DcyinjunTrackedShaResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        instance.state_blob.len() == 8
                            && instance.state_blob.get(..4) == Some(b"CYJ1")
                            && u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap())
                                == card.raw()
                    })
                })
        }
        TriggerFilter::OwnerSourceOwnViewAsCardTargetedTargetSkillPile(skill_def) => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && view_as_skill == active.skill_id.raw()
                && !state.skill_pile_cards(ctx.player, *skill_def).is_empty()
        }
        TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn => {
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && ctx.player != active.owner
                && event_player_alive
                && state.turn.current != active.owner
        }
        TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDiscardEnd
                && crate::effect::discard_phase_discarded_cards_for(state, active.owner).any(
                    |card| {
                        state.cards.get(card).is_some_and(|card_state| {
                            crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_sha()
                        })
                    },
                )
        }
        TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && event_player_alive
                && card_kind.is_some_and(|kind| kind.is_sha())
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.hp > 0 && target.hp == target.max_hp as i32)
        }
        TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerIsTargetShaSkillPileNonempty(skill_def) => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind.is_some_and(|kind| kind.is_sha())
                && !state.skill_pile_cards(active.owner, *skill_def).is_empty()
        }
        TriggerFilter::OwnerAliveOtherPlayDiscardSkillPileNonempty(skill_def) => {
            owner_alive
                && state.turn.current != active.owner
                && state.turn.phase == crate::state::Phase::Play
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && !ctx.cards.is_empty()
                && !state.skill_pile_cards(active.owner, *skill_def).is_empty()
        }
        TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.current == active.owner
                && state.turn.phase == crate::state::Phase::Prepare
                && crate::query::other_group_count_outside_owner_attack_range(state, active.owner)
                    > 0
        }
        TriggerFilter::OwnerAliveEventCardsNonempty => owner_alive && !ctx.cards.is_empty(),
        TriggerFilter::OwnerAliveTargetedByOther => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
        }
        TriggerFilter::OwnerAliveTargetedByOtherBlackCard => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_black
        }
        TriggerFilter::OwnerAliveTargetedByOtherRedCard => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_red
        }
        TriggerFilter::OwnerAliveOtherSourceDeath => {
            owner_alive
                && ctx.kind == EventKind::Death
                && ctx.source != active.owner
                && ctx.source != PlayerId::NONE
        }
        TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && state.turn.current == active.owner
                && !state.hand_cards(active.owner).into_iter().any(|card| {
                    state.cards.get(card).is_some_and(|candidate| {
                        crate::query::effective_card_kind(state, active.owner, candidate)
                            .is_sha()
                    })
                })
        }
        TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && !crate::effect::dcchiying_in_range(state, ctx.player, active.owner)
        }
        TriggerFilter::OwnerAliveEventPlayerGroup(group) => {
            owner_alive
                && event_player_alive
                && event_player_group.is_some_and(|event_group| event_group == group)
        }
        TriggerFilter::OwnerIsEventPlayerNoClanLostHandThisTurn(clan) => {
            // Clan 中流 (clanzhongliu): owner is the event player, and no OTHER living member
            // of `clan` lost hand cards this turn. Per-turn approximation of upstream's
            // per-useEvent `relatedEvent` correlation (the engine tracks per-turn card loss,
            // not per-event-parent loss; see docs/migration/upstream-1.11.5-delta.md).
            owner_is_player
                && owner_alive
                && !state.players.iter_live().any(|other| {
                    other.id != active.owner
                        && state.player_has_clan(other.id, clan)
                        && state.turn.card_loss_count(other.id) > 0
                })
        }
        TriggerFilter::OwnerIsRespondTo => {
            // 矢仄 (clanshize): someone responded to the owner's card. For CardResponded
            // events ctx.source is the responder and ctx.respond_to is the answered player.
            owner_alive && active.owner == ctx.respond_to
        }
        TriggerFilter::OwnerAliveEventPlayerSelfOrClan(clan) => {
            // 保族 (clanbaozu): the dying/event player is the owner or a member of `clan`.
            // The upstream hp<=0 gate is inherent to the dying event; the "not linked" gate
            // is expressed as a body-side mark guard in the DSL.
            owner_alive
                && event_player_alive
                && (ctx.player == active.owner || state.player_has_clan(ctx.player, clan))
        }
        TriggerFilter::OwnerAliveOtherDamageSourceGroup(group) => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::Damage
                && !owner_is_source
                && event_source_group.is_some_and(|event_group| event_group == group)
        }
        TriggerFilter::OwnerDyingOtherTaoResponderGroup(group) => {
            owner_alive
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.dying && owner.hp <= 0)
                && source_alive
                && ctx.kind == EventKind::CardResponded
                && ctx.respond_to == active.owner
                && ctx.source != active.owner
                && card_kind == Some(CardKind::Tao)
                && event_source_group.is_some_and(|event_group| event_group == group)
        }
        TriggerFilter::OwnerAliveOtherJudgmentBlackEventPlayerGroup(group) => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && judgment_is_black
                && event_player_group.is_some_and(|event_group| event_group == group)
        }
        TriggerFilter::OwnerWoundedNotEventSource => owner_wounded && !owner_is_source,
        TriggerFilter::OwnerIsEventPlayerWounded => owner_is_player && owner_wounded,
        TriggerFilter::OwnerIsEventPlayerLinked => {
            owner_alive
                && owner_is_player
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }
        TriggerFilter::OwnerIsTargetChainDamageLinked => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED != 0
        }
        TriggerFilter::OwnerIsTargetWoundedSha => {
            owner_alive
                && owner_is_player
                && owner_wounded
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
        }
        TriggerFilter::OwnerIsTargetHealthyDrawNotFixed => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.alive && owner.hp >= owner.max_hp as i32)
        }
        TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && crate::query::alive_wounded_or_guiming_wu_count(state, active.owner) > 0
        }
        TriggerFilter::OwnerWoundedEventPlayerGroup(group) => {
            owner_wounded && event_player_group.is_some_and(|event_group| event_group == group)
        }
        TriggerFilter::OwnerAliveNotEventPlayer => owner_alive && !owner_is_player,
        TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && (state.hand_count(active.owner) > 0 || equip_count(active.owner) > 0)
        }
        TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && (card_is_sha || card_is_ordinary_trick)
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.hp < target.max_hp as i32)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && (card_is_sha || card_is_ordinary_trick)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerSourceShaOrOrdinaryTrick => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && (card_is_sha || card_is_ordinary_trick)
        }
        TriggerFilter::OwnerSourceWuxieUsed => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind == Some(CardKind::TrickWuxie)
                && turn_fires == 0
        }
        TriggerFilter::OwnerSourceShaOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && card_is_sha
                && turn_fires == 0
        }
        TriggerFilter::OwnerEquipZoneChanged => {
            let entered_owner_equip = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::Equip && zone.owner == active.owner);
            let left_owner_equip = owner_is_player && ctx.from_zone == Some(ZoneKind::Equip);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && (entered_owner_equip || left_owner_equip)
        }
        TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_is_sha
                && turn_fires == 0
                && {
                    let previous = state.turn.flag(turn_flag::LAST_SHA_TARGETS).max(0) as u32;
                    previous != 0
                        && !ctx.targets.is_empty()
                        && ctx.targets.iter().any(|target| {
                            target
                                .raw()
                                .checked_sub(1)
                                .and_then(|shift| 1_u32.checked_shl(shift))
                                .is_some_and(|bit| previous & bit != 0)
                        })
                }
        }
        TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && turn_fires < 3
                && {
                    let previous = state.turn.flag(turn_flag::LAST_CARD_TARGETS).max(0) as u32;
                    previous != 0
                        && !ctx.targets.is_empty()
                        && ctx.targets.iter().any(|target| {
                            target
                                .raw()
                                .checked_sub(1)
                                .and_then(|shift| 1_u32.checked_shl(shift))
                                .is_some_and(|bit| previous & bit != 0)
                        })
                }
        }
        TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && (state.hand_count(active.owner)
                    + state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|id| state.zones.get(id))
                        .map_or(0, |zone| zone.cards.len() as u32)
                    + state
                        .zones
                        .find(ZoneKind::Judge, active.owner)
                        .and_then(|id| state.zones.get(id))
                        .map_or(0, |zone| zone.cards.len() as u32))
                    > state
                        .players
                        .get(active.owner)
                        .map_or(0, |participant| participant.hp.max(0) as u32)
        }
        TriggerFilter::OwnerSourceJiuBoostedShaDamageSkillEnabled(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::JIU_BOOSTED != 0
                && state.skills.sorted_live().into_iter().any(|skill| {
                    skill.owner == active.owner
                        && skill.def == *skill_def
                        && state.skill_effectively_enabled(skill)
                })
        }
        TriggerFilter::OwnerAliveNotEventPlayerSourceSameGroup => {
            owner_alive
                && !owner_is_player
                && state
                    .player_group(active.owner)
                    .is_some_and(|group| state.player_group(ctx.source) == Some(group))
        }
        TriggerFilter::OwnerAliveNotEventPlayerEqualHand => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && state.hand_count(active.owner) == state.hand_count(ctx.player)
        }
        TriggerFilter::OwnerAliveNotEventPlayerHandBelowHp => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| state.hand_count(active.owner) < owner.hp.max(0) as u32)
        }
        TriggerFilter::OwnerAliveNotEventPlayerNonnegativeHpGtEventPlayer => {
            owner_alive
                && !owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .zip(state.players.get(ctx.player))
                    .is_some_and(|(owner, event_player)| owner.hp.max(0) > event_player.hp)
        }
        TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayer => {
            owner_alive
                && active.owner == state.turn.current
                && !owner_is_player
                && event_player_alive
        }
        TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty => {
            owner_alive
                && active.owner == state.turn.current
                && !owner_is_player
                && event_player_alive
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && state.hand_count(ctx.player) == 0
        }
        TriggerFilter::OwnerGainsMultipleOutsideOwnDraw => {
            let owner_gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive
                && owner_gained
                && ctx.amount > 1
                && !(state.turn.current == active.owner && state.turn.phase == Phase::Draw)
        }
        TriggerFilter::OwnerGainedOrLostAtLeastTwo => {
            let owner_gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner),
                _ => false,
            };
            let owner_lost = ctx.kind == EventKind::CardsMoved && owner_is_player;
            owner_alive && ctx.amount > 1 && (owner_gained || owner_lost)
        }
        TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(mark) => {
            let owner_gained_to_hand = match ctx.kind {
                EventKind::CardsDrawn => ctx.player == active.owner,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            let alive_count = state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(i32::MAX as usize) as i32;
            let mark_count = state
                .players
                .get(active.owner)
                .and_then(|owner| owner.marks.iter().find(|(id, _)| id == mark))
                .map(|(_, count)| *count)
                .unwrap_or(0);
            owner_alive
                && owner_gained_to_hand
                && ctx.amount > 0
                && active.owner != state.turn.current
                && mark_count < alive_count
                && turn_fires == 0
        }
        TriggerFilter::OwnerCurrentTurnOtherEquipmentLost => {
            owner_alive
                && active.owner == state.turn.current
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && ctx.from_zone == Some(ZoneKind::Equip)
                && ctx.player.is_some()
                && ctx.player != active.owner
        }
        TriggerFilter::OwnerMarkPositive(mark) => {
            owner_alive
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }
        TriggerFilter::OwnerAliveDamagedEventPlayer => {
            owner_alive && state.damage_history.contains(&(active.owner, ctx.player))
        }
        TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale => {
            owner_alive && event_player_alive && (owner_is_player || event_player_has_female_sex)
        }
        TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn => {
            owner_alive && !owner_is_player && turn_fires == 0
        }
        TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp => {
            owner_alive
                && (active.owner == state.turn.current
                    || state
                        .players
                        .get(active.owner)
                        .is_some_and(|owner| turn_fires < owner.hp.max(0) as u32))
        }
        TriggerFilter::OwnerAliveCardIsSha => owner_alive && card_is_sha,
        TriggerFilter::OwnerAliveOtherRespondedShaGainable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardResponded
                && !owner_is_player
                && card_is_sha
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card| Some(card.zone) == discard)
        }
        TriggerFilter::OwnerAliveCardTargetsGtOne => owner_alive && ctx.target_count > 1,
        TriggerFilter::OwnerAliveNonMultitargetTrickTargetsGtOne => {
            owner_alive && ctx.target_count > 1 && card_is_non_multitarget_trick
        }
        TriggerFilter::OwnerAliveWoundedCardTargetsGtOne => {
            owner_alive
                && ctx.target_count > 1
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp < owner.max_hp as i32)
        }
        TriggerFilter::OwnerAliveTargetedByOtherSingleCardDiscardable => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.source != active.owner
                && ctx.targets.as_slice() == [active.owner]
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveTargetedByTrickSingleOrDiscardable => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && card_is_trick
                && (ctx.target_count == 1
                    || !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty())
        }
        TriggerFilter::OwnerAliveTargetedByOtherTrick => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_trick
        }
        TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player != PlayerId::NONE
                && !owner_is_player
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::card_color_mask_for_suit(card_state.suit) == 0b10
                    })
                })
        }
        TriggerFilter::OwnerAliveTargetedByOtherMatchingEquipSuit => {
            let raw_suit = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| card.suit);
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && raw_suit.is_some_and(|suit| {
                    state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state.cards.get(*card).is_some_and(|card| {
                                    crate::query::effective_card_suit(state, active.owner, card)
                                        == suit
                                })
                            })
                        })
                })
        }
        TriggerFilter::OwnerMinHp => owner_is_min_hp,
        TriggerFilter::OwnerIsEventPlayerNotMinHp => {
            owner_is_player && owner_alive && !owner_is_min_hp
        }
        TriggerFilter::OwnerIsEventPlayerMinHand => {
            owner_alive && owner_is_player && owner_is_min_hand
        }
        TriggerFilter::OwnerIsEventPlayerNotMinHand => {
            owner_alive && owner_is_player && !owner_is_min_hand
        }
        TriggerFilter::OwnerShaUsedAtLeastTwo => {
            owner_alive
                && owner_is_player
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) >= 2
        }
        TriggerFilter::OwnerAliveUsedShaThisTurn => {
            owner_alive
                && state
                    .turn
                    .flag(crate::state::turn_flag::sha_used_by_player(active.owner))
                    > 0
        }
        TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.max_hp > 1)
                && (hand_empty
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_none_or(|zone| zone.cards.is_empty()))
        }
        TriggerFilter::OwnerIsEventPlayer => owner_is_player,
        TriggerFilter::OwnerAliveIsEventPlayerCurrentPlayPhase => {
            owner_alive
                && owner_is_player
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
        }
        TriggerFilter::OwnerIsEventPlayerOtherAlive => {
            owner_is_player
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }
        TriggerFilter::OwnerIsEventPlayerNotUniqueMaxCurrentHp => {
            owner_is_player
                && crate::query::unique_max_current_hp(state, active.owner) == Some(false)
        }
        TriggerFilter::OwnerIsEventPlayerHandNonempty => owner_is_player && !hand_empty,
        TriggerFilter::OwnerIsEventPlayerHpBelow(threshold) => {
            owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.alive && i64::from(owner.hp) < i64::from(*threshold))
        }
        TriggerFilter::OwnerIsEventPlayerMarkPositive(mark) => {
            owner_alive
                && owner_is_player
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }
        TriggerFilter::OwnerSourceTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
        }
        TriggerFilter::OwnerIsEventPlayerHandBelowMaxHp => {
            owner_alive && owner_is_player && owner_hand_below_max_hp
        }
        TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority => {
            let mut categories = [0_u32; 3];
            for card in state.hand_cards(active.owner) {
                if let Some(card) = state.cards.get(card) {
                    let kind = crate::query::effective_card_kind(state, active.owner, card);
                    let index = if kind.is_basic() {
                        Some(0)
                    } else if kind.is_trick() {
                        Some(1)
                    } else if kind.is_equip() {
                        Some(2)
                    } else {
                        None
                    };
                    if let Some(index) = index {
                        categories[index] = categories[index].saturating_add(1);
                    }
                }
            }
            owner_alive
                && owner_is_player
                && matches!(
                    ctx.kind,
                    EventKind::Damage
                        | EventKind::HpLost
                        | EventKind::HpRecovered
                        | EventKind::MaxHpChanged
                )
                && history.is_some_and(|snapshot| snapshot.event_player_hp_change_count == 1)
                && categories[1] > categories[0]
                && categories[1] > categories[2]
        }
        TriggerFilter::OwnerIsEventPlayerHandAboveHp => {
            owner_alive && owner_is_player && owner_hand_above_hp
        }
        TriggerFilter::OwnerIsEventPlayerHandAboveHpNoLiubeiLord => {
            let liubei_lord_exists = state.players.iter_live().any(|player| {
                player.alive
                    && player.identity == Identity::Lord
                    && state
                        .content
                        .character(player.character_def)
                        .is_some_and(|character| {
                            character
                                .name_key
                                .rsplit('_')
                                .next()
                                .is_some_and(|name| name.starts_with("liubei"))
                        })
            });
            owner_alive && owner_is_player && owner_hand_above_hp && !liubei_lord_exists
        }
        TriggerFilter::OwnerIsEventPlayerHandAtLeastHpOtherAlive => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| state.hand_count(active.owner) >= owner.hp.max(0) as u32)
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }
        TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable => {
            let count = state.turn.card_loss_count(active.owner);
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && count > 0
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| count <= owner.hp.max(0) as u32)
                && state.turn.lost_cards_for(active.owner).any(|card| {
                    state
                        .cards
                        .get(card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
        }
        TriggerFilter::OwnerIsEventPlayerSourceMovableFieldCard => {
            owner_alive
                && owner_is_player
                && source_alive
                && !crate::query::movable_field_cards_to(state, ctx.source).is_empty()
        }
        TriggerFilter::OwnerIsEventPlayerNoTurnDamage => {
            owner_is_player
                && active.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::DAMAGE_DEALT) == 0
        }
        TriggerFilter::OwnerIsEventPlayerTurnDamage => {
            owner_is_player
                && active.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::DAMAGE_DEALT) > 0
        }
        TriggerFilter::OwnerIsEventPlayerTurnDamageAtLeastThree => {
            owner_is_player
                && active.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::DAMAGE_DEALT) >= 3
        }
        TriggerFilter::OwnerIsEventPlayerCardUsedAtLeastHp => {
            owner_is_player
                && active.owner == state.turn.current
                && state.players.get(active.owner).is_some_and(|player| {
                    state.turn.flag(crate::state::turn_flag::CARD_USED) >= player.hp
                })
        }
        TriggerFilter::OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive => {
            owner_alive
                && owner_is_player
                && active.owner == state.turn.current
                && state.turn.play_card_category_count(active.owner) > 0
        }
        TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive => {
            owner_alive && state.turn.play_card_category_count(active.owner) > 0
        }
        TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && state.turn.current == ctx.player
                && state.turn.phase == Phase::Play
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerIsEventPlayerActiveUsedAtLeastThisPlayPhase(threshold) => {
            owner_alive
                && owner_is_player
                && active.owner == state.turn.current
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.play_phase_use_count >= *threshold)
        }
        TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp => {
            owner_is_player
                && active.owner == state.turn.current
                && state.players.get(active.owner).is_some_and(|player| {
                    state
                        .turn
                        .flag(crate::state::turn_flag::USE_RESPONSE_DISCARDED)
                        >= player.hp
                })
        }
        TriggerFilter::OwnerIsEventPlayerDamagedOtherQun => {
            owner_is_player
                && active.owner == state.turn.current
                && state
                    .turn
                    .flag(crate::state::turn_flag::DAMAGE_DEALT_OTHER_QUN)
                    != 0
        }
        TriggerFilter::OwnerIsEventPlayerSourceAlive => owner_is_player && source_alive,
        TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn => {
            owner_is_player && source_alive && turn_fires == 0
        }
        TriggerFilter::OwnerIsEventPlayerSourceAliveDisabledWeapon => {
            owner_is_player
                && source_alive
                && state.players.get(ctx.source).is_some_and(|source| {
                    source.disabled_equip_slots[crate::state::EquipSlot::Weapon.index()] > 0
                })
        }
        TriggerFilter::OwnerIsEventPlayerSourceHasWeapon => {
            owner_is_player
                && ctx.source.is_some()
                && state
                    .zones
                    .find(ZoneKind::Equip, ctx.source)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state
                                .cards
                                .get(*card)
                                .and_then(|card| card.kind.equip_slot())
                                == Some(crate::state::EquipSlot::Weapon)
                        })
                    })
        }
        TriggerFilter::OwnerIsEventPlayerOtherSourceAlive => {
            owner_is_player && source_alive && !owner_is_source
        }
        TriggerFilter::OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && event_source_group == Some("qun")
                && crate::query::discardable_hand_equip_cards(state, active.owner).len() >= 2
        }
        TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint => {
            ctx.amount > 0 && owner_is_player && source_alive && !owner_is_source
        }
        TriggerFilter::OwnerIsEventPlayerPerDamagePoint => ctx.amount > 0 && owner_is_player,
        TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty => {
            ctx.amount > 0
                && owner_alive
                && owner_is_player
                && state.players.iter_live().any(|player| {
                    player.alive
                        && player.id != active.owner
                        && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                            .into_iter()
                            .any(|kind| {
                                state
                                    .zones
                                    .find(kind, player.id)
                                    .and_then(|zone| state.zones.get(zone))
                                    .is_some_and(|zone| !zone.cards.is_empty())
                            })
                })
        }
        TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint => {
            ctx.amount > 0
                && owner_is_player
                && source_alive
                && if owner_is_source {
                    equip_count(ctx.source) > 0
                } else {
                    state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0
                }
        }
        TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCard => {
            ctx.amount > 0
                && owner_is_player
                && source_alive
                && if owner_is_source {
                    equip_count(ctx.source) > 0
                } else {
                    state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0
                }
        }
        TriggerFilter::OwnerIsEventPlayerCardInProcessing => {
            owner_is_player
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        state
                            .zones
                            .get(card_state.zone)
                            .is_some_and(|zone| zone.kind == ZoneKind::Processing)
                    })
                })
        }
        TriggerFilter::OwnerIsEventPlayerRedShaSourceAlive => {
            owner_is_player && source_alive && card_is_sha && card_is_red
        }
        TriggerFilter::OwnerIsEventPlayerNonRedSha => {
            owner_is_player && card_is_sha && !card_is_red
        }
        TriggerFilter::OwnerIsEventPlayerRedOrJiuSha => {
            owner_is_player
                && card_is_sha
                && (card_is_red || ctx.card_flags & rule_api::damage_card_flag::JIU_BOOSTED != 0)
        }
        TriggerFilter::OwnerIsEventPlayerShaSourceHasDiscardableHandEquip => {
            owner_is_player
                && ctx.source.is_some()
                && card_is_sha
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerIsEventPlayerRedCardSourceAlive => {
            owner_is_player
                && source_alive
                && (ctx.card.is_some() || ctx.card_kind.is_some())
                && card_is_red
        }
        TriggerFilter::OwnerIsEventPlayerNonRedCard => {
            owner_is_player && (ctx.card.is_some() || ctx.card_kind.is_some()) && !card_is_red
        }
        TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn => owner_is_player && turn_fires == 0,
        TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn => owner_is_player && turn_fires == 1,
        TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn => {
            owner_is_player && turn_fires > 0
        }
        TriggerFilter::OwnerIsEventSource => owner_is_source,
        TriggerFilter::OwnerAliveSourceTargetWasLinked => {
            owner_alive
                && owner_is_source
                && ctx.card_flags & rule_api::damage_card_flag::TARGET_WAS_LINKED != 0
        }
        TriggerFilter::OwnerAliveIsEventSource => owner_alive && owner_is_source,
        TriggerFilter::OwnerSourceHandBelowAliveCountCappedSeven => {
            owner_is_source && owner_hand_below_alive_count_capped_seven
        }
        TriggerFilter::OwnerIsEventSourceOrPlayer => owner_is_source || owner_is_player,
        TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint => {
            ctx.amount > 0
                && owner_alive
                && event_player_alive
                && source_alive
                && ctx.player != ctx.source
                && (owner_is_source || owner_is_player)
        }
        TriggerFilter::OwnerSourceHealthy => {
            owner_is_source
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp == player.max_hp as i32)
        }
        TriggerFilter::OwnerSourceAmountGtOne => owner_is_source && ctx.amount > 1,
        TriggerFilter::OwnerSourceFire => {
            owner_is_source && ctx.nature == rule_api::damage_nature::FIRE
        }
        TriggerFilter::OwnerTargetFire => {
            owner_is_player && ctx.nature == rule_api::damage_nature::FIRE
        }
        TriggerFilter::OwnerTargetThunder => {
            owner_is_player && ctx.nature == rule_api::damage_nature::THUNDER
        }
        TriggerFilter::OwnerSourceThunder => {
            owner_is_source && ctx.nature == rule_api::damage_nature::THUNDER
        }
        TriggerFilter::OwnerSourceOtherTwicePerTurn => {
            owner_is_source && !owner_is_player && turn_fires < 2
        }
        TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn => {
            owner_alive
                && source_alive
                && ctx.source != active.owner
                && card_is_sha
                && turn_fires < 2
        }
        TriggerFilter::OwnerSourceWoundedDistanceLeOne => owner_source_wounded_distance_le_one,
        TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint => owner_source_distance_le_one,
        TriggerFilter::OwnerSourceTargetDistanceGtOne => owner_source_target_distance_gt_one,
        TriggerFilter::OwnerSourceTargetNotGroup(group) => {
            owner_is_source && event_player_group.is_some_and(|event_group| event_group != group)
        }
        TriggerFilter::OwnerHandEmpty => owner_is_player && hand_empty,
        TriggerFilter::OwnerTurnedOver => {
            owner_is_player
                && ctx.kind == EventKind::CharacterStateChanged
                && matches!(
                    ctx.amount,
                    CHARACTER_STATE_TURNED_FACE_UP | CHARACTER_STATE_TURNED_FACE_DOWN
                )
        }
        TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && !owner_is_player
                && state.player_group(ctx.player) == Some("wei")
                && state.turn.current == ctx.player
        }
        TriggerFilter::OwnerFromHand => {
            owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Hand))
        }
        TriggerFilter::OwnerFromEquip => {
            owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Equip))
        }
        TriggerFilter::OwnerFromEquipPerCard => {
            ctx.amount > 0 && owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Equip))
        }
        TriggerFilter::OwnerFromHandEmpty => {
            owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Hand)) && hand_empty
        }
        TriggerFilter::OwnerFromHandEmptyOtherDiscardableHandEquip => {
            owner_alive
                && owner_is_player
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && hand_empty
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|target| *target != active.owner)
                    .any(|target| {
                        !crate::query::discardable_target_cards(
                            state,
                            active.owner,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                    })
        }
        TriggerFilter::OwnerFromFieldEmpty => {
            owner_is_player
                && matches!(ctx.from_zone, Some(ZoneKind::Equip | ZoneKind::Judge))
                && [ZoneKind::Equip, ZoneKind::Judge].into_iter().all(|kind| {
                    state
                        .zones
                        .find(kind, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_none_or(|zone| zone.cards.is_empty())
                })
        }
        TriggerFilter::OwnerTargetedByTrickEquipEmpty => {
            owner_alive && owner_was_targeted && card_is_trick && owner_equip_empty
        }
        TriggerFilter::OwnerTargetedByShaHandBelowHp => {
            owner_alive && owner_was_targeted && card_is_sha && owner_hand_below_hp
        }
        TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce => {
            owner_alive
                && owner_was_targeted
                && (card_is_sha || card_is_juedou)
                && owner_damaged_hp_le_source
                && turn_fires == 0
        }
        TriggerFilter::OwnerTargetedOutsideTurnByShaOrTrickHandEmpty => {
            owner_alive
                && owner_was_targeted
                && state.turn.current != active.owner
                && hand_empty
                && (card_is_sha || card_is_trick)
        }
        TriggerFilter::OwnerTargetedHandBelowMaxHp => {
            owner_alive && owner_was_targeted && owner_hand_below_max_hp
        }
        TriggerFilter::OwnerTargetedByOtherSingleTargetTrick => {
            owner_alive
                && owner_is_player
                && !owner_is_source
                && ctx.target_count == 1
                && card_is_trick
        }
        TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha => {
            owner_alive
                && (owner_is_source || owner_was_targeted)
                && (card_is_juedou || (card_is_sha && card_is_red))
        }
        TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed => {
            owner_alive && (card_is_juedou || (card_is_sha && card_is_red))
        }
        TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed => {
            owner_alive
                && ctx.player != active.owner
                && (card_is_juedou || (card_is_sha && card_is_red))
        }
        TriggerFilter::OwnerSkillPileNonempty(pile_def) => {
            owner_is_player && !state.skill_pile_cards(active.owner, *pile_def).is_empty()
        }
        TriggerFilter::OwnerSourceOrTargetedByJuedou => {
            owner_alive && (owner_is_source || owner_was_targeted) && card_is_juedou
        }
        TriggerFilter::OwnerSourceOrTargetedByShaEqualHp => {
            owner_alive
                && (owner_is_source || owner_is_player)
                && card_is_sha
                && event_players_equal_hp
        }
        TriggerFilter::OwnerSourceOrTargetedByOtherDiamond => {
            owner_alive
                && card_is_diamond
                && (owner_is_source || (owner_was_targeted && !owner_is_source))
        }
        TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn => {
            owner_alive
                && card_is_club
                && (owner_is_source || owner_was_targeted)
                && turn_fires == 0
                && state.players.iter_live().any(|player| {
                    player.alive
                        && player
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }
        TriggerFilter::OwnerSourceShaSingleTrickSourceCard => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && single_sha_source_kind.is_some_and(CardKind::is_trick)
        }
        TriggerFilter::OwnerSourceShaSingleEquipSourceCard => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && single_sha_source_kind.is_some_and(CardKind::is_equip)
        }
        TriggerFilter::OwnerSourceRed => owner_alive && owner_is_source && card_is_red,
        TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce => {
            owner_alive
                && owner_is_player
                && ctx.amount == 1
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && state.turn.current != active.owner
                && ctx.player == state.turn.current
                && turn_fires == 0
        }
        TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.player == active.owner
                && ctx.source != active.owner
                && ctx.target_count == 1
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }
        TriggerFilter::OwnerAliveOtherMarkedXi => {
            // 檄标记键 = 擅袭技能 def id（ABI 440）。
            const SPSHANXI_DEF: u32 = 2_378_382_374;
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|participant| participant.hp > 0)
                && state.players.get(ctx.player).is_some_and(|participant| {
                    participant
                        .marks
                        .iter()
                        .any(|(m, n)| *m == SPSHANXI_DEF && *n > 0)
                })
        }
        TriggerFilter::OwnerAliveOtherTrickFirstTargetSourceMaxHand => {
            let source_is_max_hand = state
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != ctx.source)
                .all(|candidate| state.hand_count(candidate) <= state.hand_count(ctx.source));
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && ctx.player == active.owner
                && ctx.targets.first() == Some(&ctx.player)
                && card_is_trick
                && source_is_max_hand
        }
        TriggerFilter::OwnerAliveOtherPlayPhaseDiscardableGteMark => {
            // 凶竖标记键 = 凶竖技能 def id（ABI 436）。
            const XIONGSHU_DEF: u32 = 3_731_458_110;
            let mark = state
                .players
                .get(active.owner)
                .and_then(|p| {
                    p.marks
                        .iter()
                        .find(|(m, _)| *m == XIONGSHU_DEF)
                        .map(|(_, n)| *n)
                })
                .unwrap_or(0)
                .max(0) as u32;
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && !state.hand_cards(ctx.player).is_empty()
                && crate::query::discardable_hand_equip_cards(state, active.owner).len() as u32
                    >= mark
        }
        TriggerFilter::OwnerSourceClub => {
            owner_alive
                && owner_is_source
                && current_suit == Some(crate::command::judge::suit_tag(engine_api::Suit::Club))
        }
        TriggerFilter::OwnerSourceAlternatesColor => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && previous_card_meta != 0
                && current_suit.is_some_and(|suit| suit_is_red(suit) != suit_is_red(previous_suit))
        }
        TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && previous_card_meta != 0
                && matches!(previous_suit, 1..=4)
                && current_suit.is_some_and(|suit| {
                    matches!(suit, 1..=4) && suit_is_red(suit) == suit_is_red(previous_suit)
                })
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer => {
            owner_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.bound_target())
                    == Some(ctx.player)
        }
        TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && previous_play_card_meta != 0
                && (current_suit == Some(previous_play_suit)
                    || current_rank == Some(previous_play_rank))
        }
        TriggerFilter::OwnerSourcePlaySkillMarkMode(mark) => {
            let mode = state
                .players
                .get(active.owner)
                .and_then(|owner| owner.marks.iter().find(|(id, _)| id == mark))
                .map(|(_, count)| *count)
                .unwrap_or(0);
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && matches!(mode, 1 | 2)
        }
        TriggerFilter::OwnerSourceHandBasicEqualsNonBasic => {
            owner_is_source && owner_hand_basic_equals_non_basic
        }
        TriggerFilter::OwnerIsEventPlayerEventCardSha => owner_is_player && card_is_sha,
        TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick => {
            owner_is_player && (card_is_sha || card_is_trick)
        }
        TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou => {
            owner_alive
                && (card_is_sha || card_is_juedou)
                && (owner_is_source
                    || (ctx.source.is_some()
                        && crate::query::player_in_attack_range(state, ctx.source, active.owner)))
        }
        TriggerFilter::OwnerSourceSha => owner_is_source && card_is_sha,
        TriggerFilter::OwnerTargetSha => owner_is_player && card_is_sha,
        TriggerFilter::OwnerSourceHeartSha => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && current_suit == Some(crate::command::judge::suit_tag(engine_api::Suit::Heart))
        }
        TriggerFilter::OwnerSourceShaFirstTarget => {
            owner_is_source
                && card_is_sha
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
        }
        TriggerFilter::OwnerSourceShaOrJuedouSingleTarget => {
            owner_is_source
                && (card_is_sha || card_is_juedou)
                && ctx.kind == EventKind::CardTargeted
                && ctx.target_count == 1
        }
        TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther => {
            let gained = state.turn.gained_hand_cards.iter().any(|(owner, card)| {
                *owner == active.owner && state.hand_cards(active.owner).contains(card)
            });
            let has_other = state.players.alive_by_seat().into_iter().any(|candidate| {
                candidate != active.owner && !state.hand_cards(candidate).is_empty()
            });
            owner_alive && gained && has_other
        }
        TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && !owner_is_source
                && ctx.target_count == 1
                && ctx.player != active.owner
                && (card_is_juedou || (card_is_sha && card_is_red))
        }
        TriggerFilter::OwnerAliveCardTargetedMarkedPeer(mark_def) => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source == state.turn.current
                && ctx.source != ctx.player
                && state
                    .players
                    .get(ctx.source)
                    .is_some_and(|p| p.marks.iter().any(|(m, n)| *m == *mark_def && *n > 0))
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.marks.iter().any(|(m, n)| *m == *mark_def && *n > 0))
        }
        TriggerFilter::OwnerSourceNormalSha => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_is_sha
                && card_nature == Some(0)
        }
        TriggerFilter::OwnerSourceFireSha => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_is_sha
                && card_nature == Some(1)
        }
        TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
                && state.hand_cards(active.owner).into_iter().any(|card| {
                    state.cards.get(card).is_some_and(|card| {
                        crate::query::effective_card_kind(state, active.owner, card).is_sha()
                    })
                })
        }
        TriggerFilter::OwnerAliveDyingHasSkillPileCards => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive && player.dying && player.hp <= 0)
                && !state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && ctx.target_count == 1
                && (card_is_sha || card_is_ordinary_trick)
        }
        TriggerFilter::OwnerSourceEventCardPolicyDraw => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|skill| {
                        skill.card_use_policy(card)
                            & crate::state::SkillInstance::CARD_USE_POLICY_DRAW_ON_DAMAGE
                            != 0
                    })
                })
        }
        TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.player != active.owner
                && event_player_alive
                && card_is_sha
                && play_phase_fires == 0
        }
        TriggerFilter::OwnerSourcePlayCardCountEquals(expected) => {
            let next_count = history
                .map(|snapshot| snapshot.owner_play_phase_card_used)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::play_phase_card_used_by_player(active.owner))
                })
                .saturating_add(1);
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && u32::try_from(next_count).ok() == Some(*expected)
        }
        TriggerFilter::OwnerSourcePlayCardCountEqualsHp => {
            let next_count = history
                .map(|snapshot| snapshot.owner_play_phase_card_used)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::play_phase_card_used_by_player(active.owner))
                })
                .saturating_add(1);
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp > 0 && next_count == owner.hp)
        }
        TriggerFilter::OwnerIsEventPlayerSourceAliveMarkPositive(mark) => {
            owner_is_player
                && source_alive
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(found, _)| found == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }
        TriggerFilter::OwnerIsEventPlayerAnyFieldCardNoSkillPile(skill_def) => {
            owner_is_player
                && owner_alive
                && state.players.alive_by_seat().into_iter().any(|target| {
                    [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                        .into_iter()
                        .filter_map(|kind| state.zones.find(kind, target))
                        .filter_map(|zone| state.zones.get(zone))
                        .any(|zone| !zone.cards.is_empty())
                })
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .all(|target| state.skill_pile_cards(target, *skill_def).is_empty())
        }
        TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile(skill_def) => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && (state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0)
                && !state.skill_pile_cards(ctx.source, *skill_def).is_empty()
        }
        TriggerFilter::OwnerAliveDyingSkillPile(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && event_player_alive
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.dying && target.hp <= 0)
                && !state.skill_pile_cards(ctx.player, *skill_def).is_empty()
        }
        TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && ctx.player != active.owner
                && ctx.target_count == 1
                && card_is_sha
        }
        TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && ctx.player != active.owner
                && ctx.target_count == 1
                && card_is_sha
                && crate::query::player_in_attack_range(state, active.owner, ctx.source)
        }
        TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.source != active.owner
                && ctx.card.is_some_and(|card| {
                    state
                        .skills
                        .get(active.skill_id)
                        .and_then(|skill| skill.twxiaolian_redirect())
                        .is_some_and(|(remembered, _)| remembered == card)
                })
                && (state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0)
        }
        TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage => {
            owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.card.is_some_and(|card| {
                    state
                        .skills
                        .get(active.skill_id)
                        .and_then(|skill| skill.twxiaolian_redirect())
                        .is_some_and(|(remembered, _)| remembered == card)
                })
        }
        TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state.players.alive_by_seat().into_iter().any(|player| {
                    player != active.owner
                        && state.hand_count(player) > state.hand_count(active.owner)
                })
        }
        TriggerFilter::OwnerSourceJiu => owner_is_source && card_kind == Some(CardKind::Jiu),
        TriggerFilter::OwnerIsEventPlayerHpOdd => {
            owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.alive && owner.hp % 2 != 0)
        }
        TriggerFilter::OwnerIsEventPlayerHpEvenWounded => {
            owner_is_player
                && owner_wounded
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp % 2 == 0)
        }
        TriggerFilter::OwnerSourceTrick => owner_is_source && card_is_trick,
        TriggerFilter::OwnerSourceShaOrTrickCurrentPlayPhase => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && (card_is_sha || card_is_trick)
        }
        TriggerFilter::OwnerSourceShaCurrentPlayPhase => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && card_is_sha
        }
        TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.targets.iter().any(|target| *target != active.owner)
        }
        TriggerFilter::OwnerSourceOrdinaryTrick => owner_is_source && card_is_ordinary_trick,
        TriggerFilter::OwnerSourceOrdinaryTrickMarkPositive(mark) => {
            owner_is_source
                && card_is_ordinary_trick
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }
        TriggerFilter::OwnerSourceShaOrBlackTrickMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && (card_is_sha || (card_is_ordinary_trick && card_is_black))
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }
        TriggerFilter::OwnerSourceBasic => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(CardKind::is_basic)
        }
        TriggerFilter::OwnerSourceNonBasic => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| !kind.is_basic())
        }
        TriggerFilter::OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && owner_wounded
                && (matches!(
                    card_kind,
                    Some(CardKind::TrickNanman | CardKind::TrickWanjian)
                ) || (card_is_sha
                    && card_nature == Some(rule_api::damage_nature::NORMAL)
                    && card_suit == Some(engine_api::Suit::Spade)))
        }
        TriggerFilter::OwnerSourceDamageCardOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick())
        }
        TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.play_card_category_repeated(ctx.player)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.flag(turn_flag::DAMAGE_DEALT) > 1
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveOtherPhaseEndHasDiscardableBasicHand => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && !crate::query::standard_xiaoguo_basic_hand_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveOtherPhaseEndHandNonempty => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && !state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerAlivePreparePlayerHpGtOwner
        | TriggerFilter::OwnerAlivePrepareLordHpGtOwner => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .get(active.owner)
                    .zip(state.players.get(ctx.player))
                    .is_some_and(|(owner, event_player)| {
                        event_player.hp > owner.hp
                            && (!matches!(
                                def.filter,
                                TriggerFilter::OwnerAlivePrepareLordHpGtOwner
                            ) || event_player.identity == Identity::Lord)
                    })
        }
        TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source.is_some()
                && ctx.source != active.owner
                && ctx.player != active.owner
                && !ctx.targets.contains(&active.owner)
                && crate::query::player_in_attack_range(state, ctx.player, active.owner)
                && !crate::query::discardable_hand_equip_cards(state, ctx.player).is_empty()
        }
        TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card_state| {
                        state.players.iter_live().any(|candidate| {
                            candidate.id != active.owner
                                && candidate.id != ctx.source
                                && !ctx.targets.contains(&candidate.id)
                                && crate::query::player_in_attack_range(
                                    state,
                                    active.owner,
                                    candidate.id,
                                )
                                && !crate::query::card_target_cancelled_for_logical_kind(
                                    state,
                                    ctx.source,
                                    candidate.id,
                                    card_state,
                                    CardKind::Sha,
                                )
                        })
                    })
        }
        TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card_state| {
                        state.players.iter_live().any(|candidate| {
                            candidate.id != active.owner
                                && candidate.id != ctx.source
                                && !ctx.targets.contains(&candidate.id)
                                && candidate
                                    .marks
                                    .iter()
                                    .any(|(mark, count)| {
                                        *mark == crate::effect::ZHENGJIAN_MARK && *count > 0
                                    })
                                && crate::query::player_in_attack_range(
                                    state,
                                    ctx.source,
                                    candidate.id,
                                )
                                && !crate::query::card_target_cancelled_for_logical_kind(
                                    state,
                                    ctx.source,
                                    candidate.id,
                                    card_state,
                                    CardKind::Sha,
                                )
                        })
                    })
        }
        TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && turn_fires == 0
        }
        TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind.is_some_and(|kind| kind.is_trick())
                && !ctx.targets.is_empty()
                && ctx.targets.len()
                    > state
                        .players
                        .get(ctx.source)
                        .map(|source| source.hp.max(1))
                        .unwrap_or(1) as usize
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerIsSourceShaUsed => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::Sha)
        }
        TriggerFilter::OwnerIsResponderSha => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardResponded
                && card_kind == Some(CardKind::Sha)
        }
        TriggerFilter::OwnerIsSourceShaCardDamageBefore => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && ctx.card.is_some()
                && card_kind == Some(CardKind::Sha)
        }
        TriggerFilter::OwnerIsTargetDamageAmountGtOne => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && ctx.amount > 1
        }
        TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx
                    .source
                    .ne(&active.owner)
                && state
                    .players
                    .get(ctx.source)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(src, own)| src.hp > own.hp)
                && turn_fires == 0
        }
        TriggerFilter::OwnerSourceBasicOrTrick => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| kind.is_basic() || kind.is_trick())
        }
        TriggerFilter::OwnerIsEventPlayerCanCompareOther => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && state.hand_count(active.owner) > 0
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }
        TriggerFilter::OwnerSourcePhysicalOrdinaryTrick => {
            owner_is_source
                && card_is_ordinary_trick
                && ctx.card.is_some_and(|card| view_as_card != card.raw())
        }
        TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange => {
            let moved_from_owner = ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                );
            let moved_to_owner = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| {
                    zone.owner == active.owner
                        && matches!(
                            zone.kind,
                            ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge
                        )
                });
            let personal_cards_changed =
                ctx.kind == EventKind::CardsMoved && (moved_from_owner || moved_to_owner);
            let owner_vitals_changed = owner_is_player
                && matches!(
                    ctx.kind,
                    EventKind::Damage
                        | EventKind::HpLost
                        | EventKind::HpRecovered
                        | EventKind::MaxHpChanged
                );
            owner_alive
                && (personal_cards_changed || owner_vitals_changed)
                && state.players.get(active.owner).is_some_and(|owner| {
                    state.hand_count(active.owner)
                        < owner.max_hp.saturating_sub(owner.hp.max(0) as u32)
                })
        }
        TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick => {
            owner_alive
                && card_is_ordinary_trick
                && history
                    .map(|snapshot| snapshot.source_card_used)
                    .unwrap_or_else(|| state.turn.flag(turn_flag::card_used_by_player(ctx.source)))
                    == 0
        }
        TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank => {
            owner_alive
                && owner_is_source
                && previous_game_card_meta != 0
                && (current_suit == Some(previous_game_suit)
                    || current_rank == Some(previous_game_rank))
        }
        TriggerFilter::OncePerTurn => owner_alive && turn_fires == 0,
        TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn => {
            owner_alive
                && event_player_alive
                && turn_fires == 0
                && history
                    .map(|snapshot| snapshot.event_player_play_phase_card_used)
                    .unwrap_or_else(|| state.turn.play_phase_card_use_count(ctx.player))
                    > 1
                && history
                    .map(|snapshot| snapshot.event_player_play_phase_suit_count)
                    .unwrap_or_else(|| state.turn.distinct_play_phase_card_suits(ctx.player))
                    == 1
        }
        TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap => {
            let (round, count) = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| {
                    (skill.state_blob.len() == 12 && &skill.state_blob[..4] == b"BTU1").then(|| {
                        (
                            u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap()),
                            u32::from_le_bytes(skill.state_blob[8..12].try_into().unwrap()),
                        )
                    })
                })
                .unwrap_or((0, 0));
            let count = if round == state.turn.round { count } else { 0 };
            owner_is_player
                && state.turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) & 0b1111 == 0b1111
                && count < state.players.alive_by_seat().len().min(3) as u32
        }
        TriggerFilter::OwnerSourceOncePerTurn => owner_is_source && turn_fires == 0,
        TriggerFilter::OwnerIsEventPlayerOncePerTurn => owner_is_player && turn_fires == 0,
        TriggerFilter::OwnerSourceTargetAliveHasEquip => {
            owner_alive && owner_is_source && event_player_alive && equip_count(ctx.player) > 0
        }
        TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && matches!(
                    card_suit,
                    Some(engine_api::Suit::Spade | engine_api::Suit::Club)
                )
                && !crate::query::discardable_equipment_cards(state, active.owner, ctx.player)
                    .is_empty()
        }
        TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquipJudge,
                )
                .is_empty()
        }
        TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }
        TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::TrickTiesuo)
                && (state.hand_count(ctx.player) > 0 || equip_count(ctx.player) > 0)
                && state.players.get(ctx.player).is_some_and(|player| {
                    !player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }
        TriggerFilter::OwnerSourceShaTargetAliveHandNonempty => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && state.hand_count(ctx.player) > 0
        }
        TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne => {
            owner_source_distance_le_one && event_player_alive && card_is_sha
        }
        TriggerFilter::OwnerAliveShaTargetDistanceLeOne => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: active.owner,
                            to: ctx.player,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        }
        TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && crate::query::discardable_hand_equip_cards(state, active.owner)
                    .into_iter()
                    .any(|card| {
                        state
                            .cards
                            .get(card)
                            .is_some_and(|card| card.kind.is_equip())
                    })
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: active.owner,
                            to: ctx.player,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        }
        TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty => {
            owner_alive
                && owner_is_player
                && source_alive
                && card_is_sha
                && (state.hand_count(active.owner) > 0
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()))
        }
        TriggerFilter::OwnerAliveWdbFuchouPendingLiving => {
            owner_alive
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.starts_with(b"WFU1")
                        && (skill.state_blob.len() - 4) % 4 == 0
                        && skill.state_blob[4..].chunks_exact(4).any(|raw| {
                            let source = PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap()));
                            state
                                .players
                                .get(source)
                                .is_some_and(|participant| participant.alive)
                        })
                })
        }
        TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable => {
            let recipient = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .filter(|zone| zone.kind == ZoneKind::Hand)
                .map(|zone| zone.owner);
            let gained = ctx.cards.first().copied();
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.cards.len() == 1
                && recipient.is_some_and(|recipient| {
                    recipient != active.owner
                        && recipient != state.turn.current
                        && state.players.get(recipient).is_some_and(|player| {
                            player.alive
                                && !player.dying
                                && gained.is_some_and(|card| {
                                    state.cards.get(card).is_some_and(|card_state| {
                                        card_state.owner == recipient
                                            && Some(card_state.zone) == ctx.to_zone
                                            && card_state.kind.is_equip()
                                            && card_state
                                                .kind
                                                .equip_slot()
                                                .is_some_and(|slot| player.equip_slot_enabled(slot))
                                            && !crate::query::card_use_cancelled_for_card(
                                                state, recipient, card,
                                            )
                                    })
                                })
                        })
                })
        }
        TriggerFilter::OwnerAliveWuguRemainedInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::WuguRemained
                && !ctx.cards.is_empty()
                && discard.is_some_and(|discard| {
                    ctx.cards.iter().any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.zone == discard)
                    })
                })
        }
        TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_is_non_multitarget_trick
                && ctx.target_count >= 2
        }
        TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && state.turn.current == active.owner
                && state.turn.phase == crate::state::Phase::Play
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        state.players.alive_by_seat().into_iter().any(|target| {
                            target != active.owner
                                && !ctx.targets.contains(&target)
                                && !crate::query::card_target_cancelled_for_logical_kind(
                                    state,
                                    active.owner,
                                    target,
                                    card_state,
                                    CardKind::Sha,
                                )
                        })
                    })
                })
        }
        TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.state_blob.get(4..8))
                .and_then(|raw| raw.try_into().ok())
                .map(u32::from_le_bytes);
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && ctx.card.map(engine_api::CardId::raw) == tracked
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.get(0..4).is_some_and(|raw| {
                        u32::from_le_bytes(raw.try_into().unwrap()) == 0xDDD0_6170
                    })
                })
        }
        TriggerFilter::OwnerSourceCardResolvedDddXuyuPending => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && crate::effect::ddd_xuyu_pending(state, active.skill_id)
        }
        TriggerFilter::DddXuyuPending => {
            ctx.kind == EventKind::TurnEnded
                && crate::effect::ddd_xuyu_pending(state, active.skill_id)
        }
        TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty => {
            let owner_has_card = state
                .zones
                .find(ZoneKind::Hand, active.owner)
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| !zone.cards.is_empty())
                || state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty());
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source != active.owner
                && event_player_alive
                && state.turn.current == ctx.source
                && state.turn.phase == Phase::Play
                && history
                    .map(|snapshot| snapshot.event_player_play_phase_card_used)
                    .unwrap_or_else(|| state.turn.play_phase_card_use_count(ctx.source))
                    == 2
                && owner_has_card
        }
        TriggerFilter::DddShijianWatchedSourceCardUsedTargeted => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.target_count > 0
                && crate::effect::ddd_shijian_watched(state, active.skill_id) == Some(ctx.source)
        }
        TriggerFilter::DddShijianPending => {
            ctx.kind == EventKind::TurnEnded
                && crate::effect::ddd_shijian_watched(state, active.skill_id).is_some()
        }
        TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.hp_change_count(active.owner) > 0
                && state.turn.entered_discard_cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
        }
        TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && !state.hand_cards(active.owner).is_empty()
                && !state.hand_cards(ctx.player).is_empty()
        }
        TriggerFilter::OwnerAliveDddDongchaPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && !state
                    .skill_pile_cards(active.owner, crate::effect::DDD_DONGCHA_DEF)
                    .is_empty()
        }
        TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp => {
            let need = state
                .players
                .get(ctx.player)
                .map(|participant| participant.hp.max(0) as usize)
                .unwrap_or(0);
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && need > 0
                && state
                    .hand_cards(active.owner)
                    .into_iter()
                    .filter(|card| state.cards.get(*card).is_some_and(|card| !card.face_up))
                    .count()
                    >= need
        }
        TriggerFilter::OwnerPlayDddZhengjunHpAvailable => {
            let used = state
                .skills
                .get(active.skill_id)
                .map(|skill| skill.play_phase_category_mask)
                .unwrap_or(u8::MAX);
            owner_alive
                && event_player_alive
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && used & DDD_ZHENGJUN_HP == 0
                && matches!(
                    ctx.kind,
                    EventKind::Damage | EventKind::HpLost | EventKind::HpRecovered
                )
                && state
                    .players
                    .get(ctx.player)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(changed, owner)| changed.hp == owner.hp)
        }
        TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable => {
            if !owner_alive
                || ctx.kind != EventKind::CardsMoved
                || state.turn.current != active.owner
                || state.turn.phase != Phase::Play
            {
                false
            } else {
                let used = state
                    .skills
                    .get(active.skill_id)
                    .map(|skill| skill.play_phase_category_mask)
                    .unwrap_or(u8::MAX);
                let move_mask = ddd_zhengjun_move_mask(state, ctx);
                let destination = ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .map(|zone| zone.owner)
                    .unwrap_or(PlayerId::NONE);
                let affected = |from_bit: u8, to_bit: u8| {
                    let mut players = Vec::new();
                    if move_mask & from_bit != 0 && ctx.player.is_some() {
                        players.push(ctx.player);
                    }
                    if move_mask & to_bit != 0
                        && destination.is_some()
                        && !players.contains(&destination)
                    {
                        players.push(destination);
                    }
                    players
                };
                let hand = used & DDD_ZHENGJUN_HAND == 0
                    && affected(1, 2).into_iter().any(|changed| {
                        state
                            .players
                            .get(changed)
                            .is_some_and(|participant| participant.alive)
                            && state.hand_count(changed) == state.hand_count(active.owner)
                    });
                let equip = used & DDD_ZHENGJUN_EQUIP == 0
                    && affected(4, 8).into_iter().any(|changed| {
                        state
                            .players
                            .get(changed)
                            .is_some_and(|participant| participant.alive)
                            && equip_count(changed) == equip_count(active.owner)
                    });
                hand || equip
            }
        }
        TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePlay
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.phase == Phase::Play
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::DddChashiMatchingCard => {
            let pending = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.ddd_chashi_state());
            let suit = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| crate::query::effective_card_suit(state, ctx.source, card))
                .map(crate::command::judge::suit_tag)
                .and_then(|value| u8::try_from(value).ok())
                .unwrap_or(0);
            let category = card_kind.map_or(0, |kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    3
                } else {
                    0
                }
            });
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && pending.is_some_and(|(target, expected_suit, expected_category)| {
                    target == ctx.source
                        && state.turn.current == target
                        && state.turn.phase == Phase::Play
                        && ((suit != 0 && suit == expected_suit)
                            || (category != 0 && category == expected_category))
                })
        }
        TriggerFilter::OwnerAliveDddQiceReady => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner == ctx.player
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.content_flag(0) == Some(false))
        }
        TriggerFilter::OwnerAliveDddFenjiAvailable => {
            let phase_matches = (ctx.kind == EventKind::PhaseDraw
                && state.turn.phase == Phase::Draw
                && state.turn.draw_phase_fixed_count().is_none())
                || (ctx.kind == EventKind::PhaseDiscard && state.turn.phase == Phase::Discard);
            let used_mask = state
                .skills
                .get(active.skill_id)
                .map_or(0, |skill| skill.content_flags);
            owner_alive
                && active.owner == ctx.player
                && state.turn.current == active.owner
                && phase_matches
                && !crate::command::play::source_free_ordinary_trick_kinds(
                    state,
                    active.owner,
                    used_mask,
                )
                .is_empty()
        }
        TriggerFilter::OwnerAliveDddFengzhengRoundPending => {
            const GLOBAL: u32 = crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF;
            let round = state.turn.round;
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && round > 1
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.round_fire_round != round)
                && state.skills.sorted_live().iter().any(|skill| {
                    skill.def.raw() == GLOBAL
                        && skill.round_fire_round != 0
                        && skill.round_fire_round.saturating_add(1) == round
                })
        }
        TriggerFilter::OwnerAliveDddYulvDiscardMatches => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let yulv = state
                .skill_pile_cards(active.owner, 694_263_366)
                .first()
                .and_then(|card| state.cards.get(*card))
                .map(|card| card.rank);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.to_zone == discard
                && state.turn.phase != Phase::Idle
                && state
                    .players
                    .get(state.turn.current)
                    .is_some_and(|current| current.alive)
                && yulv.is_some_and(|rank| {
                    rank > 0
                        && ctx.cards.iter().any(|card| {
                            state
                                .cards
                                .get(*card)
                                .is_some_and(|card| card.rank == rank && Some(card.zone) == discard)
                        })
                })
        }
        TriggerFilter::OwnerAliveDddYouxueRoundPending => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.round > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.round_fire_round != state.turn.round)
        }
        TriggerFilter::OwnerAliveDddChengjingAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player.is_some()
                && state.players.get(ctx.player).is_some_and(|player| {
                    player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == 2_449_657_606 && *count > 0)
                })
                && state
                    .turn
                    .ddd_chengjing_eligible_uses
                    .iter()
                    .rev()
                    .any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| Some(card.zone) == discard)
                    })
        }
        TriggerFilter::DddChashiPendingPhasePlayEnd => {
            ctx.kind == EventKind::PhasePlayEnd
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.ddd_chashi_state())
                    .is_some_and(|(target, _, _)| target == ctx.player)
        }
        TriggerFilter::OwnerAliveDddZhilianAvailable => {
            let mut actors: Vec<_> = state
                .players
                .iter_live()
                .filter(|actor| {
                    state
                        .turn
                        .flag(turn_flag::card_action_count_by_player(actor.id))
                        > 0
                })
                .map(|actor| (actor.seat.raw(), actor.id, actor.alive))
                .collect();
            actors.sort_by_key(|(seat, _, _)| *seat);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && actors.len() == 2
                && actors.iter().all(|(_, _, alive)| *alive)
                && actors.iter().any(|(_, actor, _)| *actor == active.owner)
                && actors.iter().any(|(_, actor, _)| {
                    *actor != active.owner
                        && state
                            .players
                            .get(*actor)
                            .and_then(|participant| {
                                state.content.character(participant.character_def)
                            })
                            .is_some_and(|character| {
                                matches!(character.sex.as_str(), "male" | "double")
                            })
                })
        }
        TriggerFilter::OwnerAliveDddZhuansheGift => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && !state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveDddZhuansheCardUsed => {
            let Some(kind) = ctx.card_kind.and_then(CardKind::from_tag) else {
                return false;
            };
            let def = if kind == CardKind::Other {
                ctx.card
                    .and_then(|card| state.cards.get(card))
                    .map(|card| card.def.raw())
                    .unwrap_or(0)
            } else {
                0
            };
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && (kind.is_basic() || kind.is_trick() && !kind.is_delayed_trick())
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.ddd_zhuanshe_state().is_some_and(|(actor, names)| {
                        actor == ctx.player
                            && names.iter().any(|(stored_kind, stored_def, _)| {
                                *stored_kind == kind.tag() && *stored_def == def
                            })
                    })
                })
        }
        TriggerFilter::OwnerAliveDddZhuanshePhaseEnd => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill
                        .ddd_zhuanshe_state()
                        .is_some_and(|(actor, _)| actor == ctx.player)
                })
        }
        TriggerFilter::OwnerAliveDddWeiqiu => {
            owner_alive
                && ctx.kind == EventKind::RecoverBefore
                && state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveDddLianerAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source == active.owner
                && ctx.card.is_some_and(|card| {
                    crate::query::effective_card_color_mask(state, active.owner, card) == 0b01
                })
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
        }
        TriggerFilter::OwnerAliveDddLianerCeiling => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark > 0)
        }
        TriggerFilter::OwnerAliveDddAnzhiAvailable => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.ddd_anzhi_state().0)
        }
        TriggerFilter::OwnerAliveDddAnzhiTracking => {
            owner_alive && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
        }
        TriggerFilter::OwnerAliveDddAnzhiTargetBlack => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.card.is_some_and(|card| {
                    crate::query::effective_card_color_mask(state, ctx.source, card) == 0b10
                })
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    let (_, actor, target) = skill.ddd_anzhi_state();
                    actor == state.turn.current && target == ctx.player
                })
        }
        TriggerFilter::OwnerAliveDddAnzhiTurnEnd => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    let (_, actor, target) = skill.ddd_anzhi_state();
                    actor == ctx.player && target.is_some()
                })
        }
        TriggerFilter::OwnerAliveDddShixingTurnRefresh => {
            let desired = u32::from(ddd_shixing_female_available(state)) | 2;
            owner_alive
                && matches!(ctx.kind, EventKind::GameStarted | EventKind::PhaseBegin)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark != desired)
        }
        TriggerFilter::OwnerAliveDddShixingFemaleChanged => {
            let event_is_female = state.players.get(ctx.player).is_some_and(|participant| {
                state
                    .content
                    .character(participant.character_def)
                    .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
            });
            owner_alive
                && event_is_female
                && matches!(
                    ctx.kind,
                    EventKind::Damage
                        | EventKind::HpLost
                        | EventKind::HpRecovered
                        | EventKind::MaxHpChanged
                )
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    let desired_female = u32::from(ddd_shixing_female_available(state));
                    skill.mark & 1 != desired_female
                })
        }
        TriggerFilter::OwnerAliveDddShixingFirstTrick => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx
                    .card_kind
                    .and_then(CardKind::from_tag)
                    .is_some_and(CardKind::is_trick)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark & 2 != 0)
        }
        TriggerFilter::OwnerAliveDddDangguAvailable => {
            let alive = state.players.alive_by_seat();
            let linked = alive
                .iter()
                .filter(|player| ddd_player_linked(state, **player))
                .count();
            let qun = alive
                .iter()
                .filter(|player| state.player_group(**player) == Some("qun"))
                .count();
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player == active.owner
                && linked < qun
        }
        TriggerFilter::OwnerAliveDddFuzongDeath => {
            owner_alive
                && ctx.kind == EventKind::Death
                && state.player_group(ctx.player) == Some("qun")
                && (!state.hand_cards(active.owner).is_empty()
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()))
        }
        TriggerFilter::OwnerIsTargetFaceDown => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.turned_over)
        }
        TriggerFilter::OwnerAliveDddXuanlunPending => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark & 1 != 0)
        }
        TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::RecoverBefore
                && ctx.player != active.owner
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp < owner.max_hp as i32)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }
        TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.player != active.owner
                && ctx
                    .card_kind
                    .and_then(CardKind::from_tag)
                    .is_some_and(CardKind::is_equip)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
                && ctx.cards.iter().copied().any(|card| {
                    ddd_zhangkai_can_take_equipment(state, active.owner, ctx.player, card)
                })
        }
        TriggerFilter::OwnerAliveDddJiexingRecoveryReset => {
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && owner_is_player
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count > 0)
        }
        TriggerFilter::OwnerAliveDddJiexingEquipReset => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source == active.owner
                && ctx
                    .card_kind
                    .and_then(CardKind::from_tag)
                    .is_some_and(CardKind::is_equip)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count > 0)
        }
        TriggerFilter::OwnerAliveDddBaileiAvailable => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && owner_is_player
                && (state.players.alive_by_seat().into_iter().any(|target| {
                    state
                        .players
                        .get(target)
                        .is_some_and(|target| target.hp == 1)
                }) || ddd_zhangkai_unique_max_equip(state).is_some_and(|target| {
                    !crate::query::gainable_target_cards(
                        state,
                        active.owner,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                }))
        }
        TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::TrickWugu)
                && view_as_skill == active.skill_id.raw()
        }
        TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::TrickWugu)
                && view_as_skill == active.skill_id.raw()
                && event_player_alive
        }
        TriggerFilter::OwnerAliveDddTongyuRemained => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::WuguRemained
                && !ctx.cards.is_empty()
                && state
                    .resolution
                    .ddd_tongyu_rejectors(active.owner)
                    .next()
                    .is_some()
        }
        TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable => {
            const DDD_MIAOXING_PILE_1: u32 = 919_529_013;
            const DDD_MIAOXING_PILE_2: u32 = 2_949_101_455;
            owner_alive
                && owner_is_source
                && matches!(ctx.kind, EventKind::CardResolved | EventKind::CardResponded)
                && card_kind.is_some_and(CardKind::is_basic)
                && turn_fires == 0
                && (state.hand_count(active.owner) > 0
                    || !state
                        .skill_pile_cards(active.owner, DDD_MIAOXING_PILE_1)
                        .is_empty()
                    || !state
                        .skill_pile_cards(active.owner, DDD_MIAOXING_PILE_2)
                        .is_empty())
        }
        TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange => {
            if !owner_alive
                || !owner_is_source
                || !event_player_alive
                || ctx.kind != EventKind::CardTargeted
                || !card_is_sha
            {
                return false;
            }
            let hand = state.hand_count(ctx.player).min(i32::MAX as u32) as i32;
            let hp = state
                .players
                .get(active.owner)
                .map_or(i32::MAX, |owner| owner.hp);
            hand >= hp
                || matches!(
                    query(
                        state,
                        &QueryRequest::AttackRange {
                            player: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(range) if hand <= range
                )
        }
        TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && state
                    .players
                    .get(ctx.player)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(target, owner)| {
                        state.hand_count(ctx.player) <= state.hand_count(active.owner)
                            || target.hp >= owner.hp
                    })
        }
        TriggerFilter::OwnerIsEventPlayerJudgeNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .zones
                    .find(ZoneKind::Judge, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty())
        }
        TriggerFilter::OwnerIsEventPlayerDrawNotReplaced => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
        }
        TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
                && (hand_empty
                    || state
                        .players
                        .get(active.owner)
                        .is_some_and(|owner| owner.hp == 1))
        }
        TriggerFilter::OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::ShaDodged
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }
        TriggerFilter::OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::ShaDodged
                && ctx.card.is_some_and(|card| {
                    state.resolution.sha_skill_target_recorded(
                        active.owner,
                        card,
                        active.skill_id,
                        ctx.player,
                    )
                })
                && !crate::query::discardable_target_cards(
                    state,
                    ctx.player,
                    active.owner,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }
        TriggerFilter::OwnerSourceShaDodged => {
            owner_alive && owner_is_source && ctx.kind == EventKind::ShaDodged
        }
        TriggerFilter::OwnerSourceShaDamageRecordedBySkill => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.card.is_some_and(|card| {
                    state.resolution.sha_skill_target_recorded(
                        active.owner,
                        card,
                        active.skill_id,
                        ctx.player,
                    )
                })
        }
        TriggerFilter::OwnerAliveOtherDyingOncePerRound => {
            owner_alive && active.owner != ctx.player && ctx.kind == EventKind::Dying
        }
        TriggerFilter::OwnerIsTargetDiscardableHandEquipGteTwo => {
            owner_alive
                && owner_is_player
                && crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    active.owner,
                    rule_api::TargetCardScope::HandEquip,
                )
                .len()
                    >= 2
        }
        TriggerFilter::OwnerSourceTargetAliveHasHandOrEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && (state.hand_count(ctx.player) > 0 || equip_count(ctx.player) > 0)
        }
        TriggerFilter::OwnerSourceOtherTargetAliveHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }
        TriggerFilter::OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !crate::query::player_in_attack_range(state, ctx.player, active.owner)
        }
        TriggerFilter::OwnerAliveFirstTargetVirtualOrConvertedCard => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card| {
                        card.def.is_none()
                            || ctx.card_kind != Some(card.kind.tag())
                            || view_as_card == card.id.raw()
                    })
        }
        TriggerFilter::OwnerTargetedByOtherShaOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_sha
                && turn_fires == 0
        }
        TriggerFilter::OwnerTargetedByOtherCardOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && turn_fires == 0
        }
        TriggerFilter::OwnerTwfeifuSingleShaPartyTargetHasHandEquip => {
            let branch = state
                .skills
                .get(active.skill_id)
                .map_or(0, |skill| skill.view_as_branch.min(1));
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.target_count == 1
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive)
                && (state.hand_count(ctx.player) > 0 || equip_count(ctx.player) > 0)
                && match branch {
                    0 => owner_is_player && ctx.source != active.owner,
                    _ => owner_is_source,
                }
        }
        TriggerFilter::OwnerTargetedByOtherSha => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && !owner_is_source
                && card_is_sha
        }
        TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked => {
            owner_alive
                && ctx.kind == EventKind::CharacterStateChanged
                && matches!(
                    ctx.amount,
                    CHARACTER_STATE_TURNED_FACE_UP
                        | CHARACTER_STATE_TURNED_FACE_DOWN
                        | CHARACTER_STATE_BECAME_LINKED
                )
        }
        TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked => {
            owner_alive
                && ctx.kind == EventKind::CharacterStateChanged
                && matches!(
                    ctx.amount,
                    CHARACTER_STATE_TURNED_FACE_UP | CHARACTER_STATE_BECAME_LINKED
                )
        }
        TriggerFilter::OwnerCardActionCountEqualsAttackRange => {
            if !owner_alive
                || !owner_is_source
                || !matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
            {
                return false;
            }
            let next_count = history
                .map(|snapshot| snapshot.owner_card_action_count)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::card_action_count_by_player(active.owner))
                })
                .saturating_add(1);
            matches!(
                query(
                    state,
                    &QueryRequest::AttackRange {
                        player: active.owner,
                    },
                    &mut QueryContext::new(),
                ),
                QueryResponse::Int(range) if next_count == range
            )
        }
        TriggerFilter::OwnerLifetimeCardAction => {
            owner_alive
                && owner_is_source
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
        }
        TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(group) => {
            owner_alive
                && event_player_alive
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && ctx.player != active.owner
                && ctx.player != state.turn.current
                && ctx.card_kind == Some(CardKind::Shan.tag())
                && event_player_group.is_some_and(|event_group| event_group == group)
                && turn_fires == 0
        }
        TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurn => {
            owner_alive_other_discarded_basic_outside_turn
        }
        TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn => {
            owner_alive_other_discarded_basic_outside_turn && turn_fires == 0
        }
        TriggerFilter::OwnerFirstHandDiscardHasShaInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let prior_hand_discards = history
                .map(|snapshot| snapshot.event_player_hand_discard_events)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::hand_discard_event_by_player(active.owner))
                        .max(0) as u32
                });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && prior_hand_discards == 0
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_sha()
                    })
                })
        }
        TriggerFilter::OwnerAliveDealtOrReceivedDamageThisTurn => {
            owner_dealt_or_received_damage_this_turn
        }
        TriggerFilter::OwnerReceivedGiftFromOther => owner_received_gift_from_other,
        TriggerFilter::OwnerInitiatedDuelTargetAliveLoserHasHandOrEquip => {
            let original_target_alive = ctx
                .targets
                .first()
                .and_then(|target| state.players.get(*target))
                .is_some_and(|player| player.alive);
            let loser_has_card = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
                state
                    .zones
                    .find(kind, ctx.source)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty())
            });
            owner_alive
                && ctx.kind == EventKind::DuelResolved
                && ctx.respond_to == active.owner
                && original_target_alive
                && ctx.player != ctx.source
                && loser_has_card
        }
        TriggerFilter::OwnerHasEnabledEquipSlot => {
            owner_alive
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.disabled_equip_slots.iter().any(|count| *count == 0))
        }
        TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::EquipSlotDisabled
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.disabled_equip_slots.iter().all(|count| *count > 0))
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }
        TriggerFilter::OwnerIsEventPlayerHasMovableEquipment => {
            owner_alive
                && owner_is_player
                && !crate::query::movable_equipment_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.hp <= 0)
                && state.hand_count(active.owner) > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }
        TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp <= 0)
                && state.players.alive_by_seat().into_iter().any(|target| {
                    target != active.owner
                        && [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
                            state
                                .zones
                                .find(kind, target)
                                .and_then(|zone| state.zones.get(zone))
                                .is_some_and(|zone| !zone.cards.is_empty())
                        })
                })
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }
        TriggerFilter::OwnerAliveOtherDyingFirstPerTarget => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.hp <= 0)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| !skill.has_lifetime_target(ctx.player))
        }
        TriggerFilter::OwnerIsEventPlayerTrickUsedZero => {
            owner_alive
                && owner_is_player
                && state.turn.owner_card_category_mask(active.owner) & 0b010 == 0
        }
        TriggerFilter::OwnerIsEventPlayerHpLeOne => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp <= 1)
        }
        TriggerFilter::OwnerIsEventPlayerHpLeTwo => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp <= 2)
        }
        TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn => {
            owner_alive
                && (owner_is_player || owner_is_source)
                && ctx.kind == EventKind::Damage
                && state.turn.damage_taken_count(ctx.player) == 2
        }
        TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn => {
            owner_alive
                && (owner_is_player || owner_is_source)
                && ctx.kind == EventKind::Damage
                && state.turn.damage_taken_count(ctx.player) == 1
        }
        TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn => {
            owner_source_distance_le_one
                && ctx.kind == EventKind::Damage
                && state.turn.damage_pair_count(active.owner, ctx.player) == 1
        }
        TriggerFilter::OwnerFirstTransformedShaUsedThisTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && CardKind::from_tag(state.turn.flag(turn_flag::VIEW_AS_LOGICAL_KIND).max(0) as u8)
                    .is_some_and(CardKind::is_sha)
                && view_as_skill != 0
                && ctx.card.is_some_and(|card| {
                    state
                        .cards
                        .get(card)
                        .is_some_and(|card| card.def != engine_api::CardDefId::NONE)
                })
        }
        TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && hand_empty
                && (ctx.nature != 0 || card_is_ordinary_trick)
        }
        TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable => {
            let target_is_min_hp = state
                .players
                .get(ctx.player)
                .filter(|target| target.alive)
                .is_some_and(|target| {
                    state
                        .players
                        .iter_live()
                        .filter(|other| other.alive)
                        .all(|other| target.hp <= other.hp)
                });
            let drawn_mask = ctx.cards.iter().fold(0u8, |mask, card| {
                mask | state.cards.get(*card).map_or(0, |card| {
                    if card.kind.is_basic() {
                        0b001
                    } else if card.kind.is_trick() {
                        0b010
                    } else if card.kind.is_equip() {
                        0b100
                    } else {
                        0
                    }
                })
            });
            let gained_mask =
                state
                    .turn
                    .flag(crate::state::turn_flag::twjiaohua_category_by_player(
                        ctx.player,
                    )) as u8;

            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardsDrawn
                && (owner_is_player || target_is_min_hp)
                && (drawn_mask | gained_mask) & 0b111 != 0b111
        }
        TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && source_alive
                && event_player_alive
                && (card_is_sha || card_is_juedou)
                && (state.hand_count(ctx.player) > 0
                    || equip_count(ctx.player) > 0
                    || state.hand_count(ctx.source) > 0
                    || equip_count(ctx.source) > 0)
        }
        TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && card_is_sha
                && event_player_alive
                && ctx.source.is_some()
                && state.players.get(ctx.source).is_some()
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip => {
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::discardable_target_cards(
                        state,
                        target,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
        }
        TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip => {
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::discardable_target_cards(
                        state,
                        active.owner,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
        }
        TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand => {
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::discardable_target_cards(
                        state,
                        target,
                        target,
                        rule_api::TargetCardScope::Hand,
                    )
                    .is_empty()
                })
        }
        TriggerFilter::OwnerIsEventPlayerAnyAliveWounded => {
            owner_alive && owner_is_player && crate::query::alive_wounded_count(state) > 0
        }
        TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && (state.hand_count(active.owner) > 0 || equip_count(active.owner) > 0)
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }
        TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive => {
            owner_alive
                && state.turn.card_gain_count(active.owner) >= 3
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }
        TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.card_gain_count(active.owner) >= 2
                && state.hand_count(active.owner) > 0
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }
        TriggerFilter::OwnerAliveGaveCardsToOtherAlive => {
            owner_alive
                && ctx.kind == EventKind::CardsGiven
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && ctx.amount > 0
        }
        TriggerFilter::OwnerAliveTargetedByOtherHiddenSkillPile(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && !state
                    .hidden_skill_pile_cards(active.owner, *skill_def)
                    .is_empty()
        }
        TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile(skill_def) => {
            owner_alive
                && owner_is_player
                && state.hand_count(active.owner) > 0
                && !state
                    .hidden_skill_pile_cards(active.owner, *skill_def)
                    .is_empty()
        }
        TriggerFilter::OwnerLostMatchingRevealedSkillPileSuit(skill_def) => {
            if !owner_alive
                || ctx.kind != EventKind::CardsMoved
                || !owner_is_player
                || !matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
            {
                false
            } else {
                let suits: Vec<_> = state
                    .revealed_skill_pile_cards(active.owner, *skill_def)
                    .into_iter()
                    .filter_map(|card| state.cards.get(card).map(|card| card.suit))
                    .collect();
                !suits.is_empty()
                    && ctx.cards.iter().any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| suits.contains(&card.suit))
                    })
            }
        }
        TriggerFilter::OwnerAliveGroupCountAtMostTwo => {
            owner_alive && crate::query::alive_group_count(state) <= 2
        }
        TriggerFilter::OwnerAliveCurrentTurn => owner_alive && state.turn.current == active.owner,
        TriggerFilter::OwnerIsEventPlayerCurrentTurn => {
            owner_alive && owner_is_player && state.turn.current == active.owner
        }
        TriggerFilter::DebateOwnerIsInitiator => {
            owner_alive && ctx.kind == EventKind::DebateResolved && owner_is_player
        }
        TriggerFilter::DebateOwnerParticipated => {
            owner_alive
                && ctx.kind == EventKind::DebateResolved
                && (ctx.debate_red.contains(&active.owner)
                    || ctx.debate_black.contains(&active.owner))
        }
        TriggerFilter::OwnerDebateParticipant => {
            owner_alive
                && ctx.kind == EventKind::DebateStarted
                && ctx.targets.contains(&active.owner)
        }
        TriggerFilter::OwnerIsLord => {
            owner_alive
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
        }
        TriggerFilter::OwnerAliveEventPlayerLord => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
        }
        TriggerFilter::OwnerAliveEventPlayerLordNoDamage => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
                && state.turn.flag(turn_flag::DAMAGE_DEALT) == 0
        }
        TriggerFilter::OwnerAliveEventPlayerLordDamaged => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
                && state.turn.flag(turn_flag::DAMAGE_DEALT) > 0
        }
        TriggerFilter::OwnerAliveTurnUsedAllSuits => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::TurnEnded
                && state.turn.distinct_used_card_suits(active.owner) >= 4
        }
        TriggerFilter::OwnerAliveEventTargetMarkPositiveNoDamage(mark) => {
            let marked_no_damage = state
                .players
                .get(ctx.player)
                .is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
                && state.turn.flag(turn_flag::DAMAGE_DEALT) == 0;
            owner_alive && ctx.kind == EventKind::TurnEnded && marked_no_damage
        }
        TriggerFilter::OwnerAliveEventTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && owner_is_player
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| {
                        target.alive
                            && target
                                .marks
                                .iter()
                                .find(|(id, _)| id == mark)
                                .is_some_and(|(_, count)| *count > 0)
                    })
        }
        TriggerFilter::OwnerAliveDamageBeforeEventTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| {
                        target.alive
                            && target
                                .marks
                                .iter()
                                .find(|(id, _)| id == mark)
                                .is_some_and(|(_, count)| *count > 0)
                    })
        }
        TriggerFilter::OwnerAlivePhaseBeginEventTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| {
                        target.alive
                            && target
                                .marks
                                .iter()
                                .find(|(id, _)| id == mark)
                                .is_some_and(|(_, count)| *count > 0)
                    })
        }
        TriggerFilter::OwnerAlivePlayEndUsedCategoriesPositive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && state
                    .turn
                    .flag(turn_flag::play_card_category_history_by_player(
                        active.owner,
                    ))
                    .count_ones()
                    > 0
        }
        TriggerFilter::OwnerGainedCardsDuringPlayPhaseWounded => {
            let own_play_phase =
                state.turn.current == active.owner && state.turn.phase == Phase::Play;
            let gained = match ctx.kind {
                EventKind::CardsDrawn => ctx.player == active.owner,
                EventKind::CardsMoved => {
                    ctx.player == active.owner
                        && ctx
                            .to_zone
                            .and_then(|zone| state.zones.get(zone))
                            .is_some_and(|zone| {
                                zone.owner == active.owner && zone.kind == ZoneKind::Hand
                            })
                }
                _ => false,
            };
            owner_alive
                && own_play_phase
                && gained
                && ctx.amount > 0
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp < player.max_hp as i32)
        }
        TriggerFilter::OwnerDiscardedOutsidePlayPhase => {
            let own_play_phase =
                state.turn.current == active.owner && state.turn.phase == Phase::Play;
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && !own_play_phase
                && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                && !ctx.cards.is_empty()
        }
        TriggerFilter::OwnerInMultitargetShaOrTrickTargets => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.target_count > 1
                && (card_is_sha || card_is_ordinary_trick)
        }
        TriggerFilter::OwnerTargetedByNanman => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && card_kind == Some(CardKind::TrickNanman)
        }
        TriggerFilter::OwnerBlackCardMovedEquipJudge => {
            let black = ctx
                .cards
                .iter()
                .any(|c| {
                    state.cards.get(*c).is_some_and(|card| {
                        matches!(card.suit, engine_api::Suit::Spade | engine_api::Suit::Club)
                    })
                });
            let leaves_owner_zone = ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Equip) | Some(ZoneKind::Judge)
                );
            let enters_owner_zone = ctx
                .to_zone
                .and_then(|z| state.zones.get(z))
                .is_some_and(|z| {
                    z.owner == active.owner
                        && matches!(z.kind, ZoneKind::Equip | ZoneKind::Judge)
                });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && black
                && (leaves_owner_zone || enters_owner_zone)
        }
        TriggerFilter::OwnerAliveDamageCardNanman => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && card_kind == Some(CardKind::TrickNanman)
        }
        TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(suit_tag, mark_def) => {
            let suit = match *suit_tag {
                1 => engine_api::Suit::Spade,
                2 => engine_api::Suit::Heart,
                3 => engine_api::Suit::Club,
                4 => engine_api::Suit::Diamond,
                _ => engine_api::Suit::None,
            };
            let owner_has_mark = state
                .players
                .get(active.owner)
                .is_some_and(|p| {
                    p.marks
                        .iter()
                        .any(|(found, count)| *found == *mark_def && *count > 0)
                });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && ctx
                    .cards
                    .iter()
                    .any(|c| state.cards.get(*c).is_some_and(|card| card.suit == suit))
                && ctx
                    .to_zone
                    .and_then(|z| state.zones.get(z))
                    .is_some_and(|z| z.kind == ZoneKind::DiscardPile)
                && !owner_has_mark
        }
        TriggerFilter::OwnerAliveOtherDyingHasEquipCard => {
            let has_equip_card = state
                .hand_cards(active.owner)
                .iter()
                .any(|card| state.cards.get(*card).is_some_and(|c| c.kind.is_equip()))
                || state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty());
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.hp <= 0)
                && has_equip_card
        }
        TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && card_is_ordinary_trick
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && turn_fires == 0
        }
        TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_player
                && source_alive
                && state
                    .players
                    .get(ctx.source)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(source, target)| {
                        source.hp != target.hp
                            && state.hand_count(ctx.source) != state.hand_count(active.owner)
                    })
                && turn_fires == 0
        }
        TriggerFilter::OwnerSourceHandGaintagAbsent(tag) => {
            // natuxing 兴门: owner uses a card while holding no card carrying
            // the given gaintag.
            let has_tag_card = state
                .hand_cards(active.owner)
                .iter()
                .any(|card| {
                    state
                        .content_card_tags
                        .get(card)
                        .is_some_and(|tags| tags.contains(tag))
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && !has_tag_card
        }
        TriggerFilter::CardsMovedDiscardAtLeastThree => {
            ctx.kind == EventKind::CardsMoved
                && ctx.to_zone
                    .and_then(|zone| state.zones.get(zone).map(|z| z.kind))
                    == Some(ZoneKind::DiscardPile)
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.cards.len() >= 3
        }
        TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(mark) => {
            let carries_mark = |player: PlayerId| {
                state
                    .players
                    .get(player)
                    .is_some_and(|target| {
                        target.alive
                            && target
                                .marks
                                .iter()
                                .find(|(id, _)| id == mark)
                                .is_some_and(|(_, count)| *count > 0)
                    })
            };
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && turn_fires == 0
                && carries_mark(ctx.player)
        }
        TriggerFilter::OwnerAliveMarkedAliveAboveTwo(mark) => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|player| {
                        state
                            .players
                            .get(*player)
                            .is_some_and(|holder| holder.alive)
                            && state
                                .players
                                .get(*player)
                                .and_then(|holder| {
                                    holder.marks.iter().find(|(id, _)| id == mark).map(
                                        |(_, count)| *count,
                                    )
                                })
                                .unwrap_or(0)
                                > 0
                    })
                    .count()
                    > 2
        }
        TriggerFilter::CardsMovedIntoExpansion => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .is_some_and(|zone| {
                        state
                            .zones
                            .get(zone)
                            .is_some_and(|z| z.kind == ZoneKind::Expansion)
                    })
        }
        TriggerFilter::CardsMovedOutOfExpansion => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Expansion)
        }
        TriggerFilter::OwnerAliveDiscardMatchesOwnPileCard(pile_def) => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.to_zone
                    .and_then(|zone| state.zones.get(zone).map(|z| z.kind))
                    == Some(ZoneKind::DiscardPile)
                && {
                    let pile = state
                        .skill_pile_cards(active.owner, *pile_def);
                    !pile.is_empty()
                }
                && {
                    let pile_card = state
                        .skill_pile_cards(active.owner, *pile_def)[0];
                    let Some(pile_state) = state.cards.get(pile_card) else {
                        return false;
                    };
                    ctx.cards.iter().any(|card| {
                        state.cards.get(*card).is_some_and(|c| {
                            c.suit == pile_state.suit && c.rank == pile_state.rank
                        })
                    })
                }
        }
        TriggerFilter::OwnerAliveSomeoneMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.players.alive_by_seat().into_iter().any(|player| {
                    state
                        .players
                        .get(player)
                        .and_then(|holder| {
                            holder.marks.iter().find(|(id, _)| id == mark).map(|(_, count)| *count)
                        })
                        .unwrap_or(0)
                        > 0
                })
        }
        TriggerFilter::OwnerAliveShaResolvedCity(skill_def) => {
            let has_city = state.skill_piles.iter().any(|pile| {
                pile.owner == active.owner
                    && pile.skill_def == *skill_def
                    && state
                        .zones
                        .get(pile.zone)
                        .is_some_and(|zone| !zone.cards.is_empty())
            });
            let any_lost = state.players.alive_by_seat().into_iter().any(|player| {
                player != active.owner
                    && state
                        .turn
                        .flag(crate::state::turn_flag::lost_card_count_by_player(player))
                        > 0
            });
            owner_alive
                && ctx.kind == EventKind::ShaResolved
                && has_city
                && any_lost
        }
        TriggerFilter::OwnerAliveOtherTargetedBySha => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.player != active.owner
        }
        TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou => {
            let has_xumou = state
                .zones
                .find(ZoneKind::Judge, active.owner)
                .map(|judge| {
                    state
                        .zones
                        .get(judge)
                        .map(|zone| {
                            zone.cards.iter().rev().any(|card| {
                                state
                                    .content_card_tags
                                    .get(card)
                                    .is_some_and(|tags| tags.contains(&crate::content::XUMOU_TAG))
                            })
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && !owner_is_source
                && card_is_sha
                && has_xumou
        }
        TriggerFilter::OwnerCardsMovedContainYing => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.to_zone.map(|z| state.zones.get(z).map(|z| z.kind)) == Some(Some(ZoneKind::DiscardPile))
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == crate::state::CardKind::Ying)
                })
        }
        TriggerFilter::OwnerAliveOtherSourceDamage => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && ctx.source != PlayerId::NONE
                && ctx.source != active.owner
        }
        TriggerFilter::DebateOwnerInitiatorResultRed => {
            owner_alive
                && ctx.kind == EventKind::DebateResolved
                && owner_is_player
                && ctx.amount == 1
        }
        TriggerFilter::DebateOwnerInitiatorResultBlack => {
            owner_alive
                && ctx.kind == EventKind::DebateResolved
                && owner_is_player
                && ctx.amount == 2
        }
        TriggerFilter::DebateOwnerOpinionDiffersInitiator => {
            owner_alive && ctx.kind == EventKind::DebateResolved && {
                let owner_red = ctx.debate_red.contains(&active.owner);
                let owner_black = ctx.debate_black.contains(&active.owner);
                let init_red = ctx.debate_red.contains(&ctx.player);
                let init_black = ctx.debate_black.contains(&ctx.player);
                (owner_red || owner_black)
                    && (init_red || init_black)
                    && owner_red != init_red
            }
        }
        TriggerFilter::OwnerLostPackageCardFromEquip => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Equip)
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .and_then(|card| state.content.card_metadata(card.def))
                        .is_some_and(|card| card.on_leave_equip_skill_def == active.skill_def.raw())
                })
        }
        TriggerFilter::OwnerAliveSourceAllyShaDamageBefore => {
            let allied = matches!(
                query(
                    state,
                    &QueryRequest::Relation {
                        a: active.owner,
                        b: ctx.source,
                    },
                    &mut QueryContext::default(),
                ),
                QueryResponse::Relation(RelationTag::Ally | RelationTag::Self_)
            );
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && ctx.source.is_some()
                && allied
        }
        TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp < player.max_hp as i32)
                && state.players.iter_live().any(|player| {
                    !player.alive
                        && matches!(
                            query(
                                state,
                                &QueryRequest::Relation {
                                    a: active.owner,
                                    b: player.id,
                                },
                                &mut QueryContext::default(),
                            ),
                            QueryResponse::Relation(RelationTag::Ally)
                        )
                })
        }
        TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && (state.hand_count(active.owner) > 0 || equip_count(active.owner) > 0)
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }
        TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && card_is_sha
                && !crate::query::discardable_equipment_cards(state, active.owner, ctx.player)
                    .is_empty()
        }
        TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !crate::query::discardable_equipment_cards(state, active.owner, ctx.player)
                    .is_empty()
        }
        TriggerFilter::OwnerSourceCardUsedTargetsSelf => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && ctx.targets.contains(&active.owner)
        }
        TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && active.owner == state.turn.current
                && ctx.player != active.owner
                && event_player_alive
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.hp.max(0) as u32 == target.max_hp)
        }
        TriggerFilter::OwnerSkillAddedSelf(skill_def) => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::SkillAdded
                && ctx.added_skill_def == *skill_def
        }
        TriggerFilter::OwnerSourceFirstCardSuitThisTurn => {
            let matched = owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_suit.is_some_and(|suit| {
                    let bit = match suit {
                        engine_api::Suit::Spade => 1,
                        engine_api::Suit::Heart => 1 << 1,
                        engine_api::Suit::Club => 1 << 2,
                        engine_api::Suit::Diamond => 1 << 3,
                        engine_api::Suit::None => return false,
                    };
                    // 使用事件时快照（remember_card_action 在计划后执行，快照不含当前牌）
                    let history_flag = history
                        .map(|snapshot| snapshot.owner_used_card_suit_history)
                        .unwrap_or_else(|| {
                            state
                                .turn
                                .flag(turn_flag::used_card_suit_history_by_player(active.owner))
                                .max(0) as u32
                        });
                    history_flag & bit == 0
                });
            matched
        }
        TriggerFilter::OwnerLostHandNowEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && ctx.move_reason == CardMoveReason::Discard
                && !ctx.cards.is_empty()
                && state.hand_cards(active.owner).is_empty()
        }
        TriggerFilter::OwnerLostSpadeTwoToNine => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        card_state.suit == engine_api::Suit::Spade
                            && (2..=9).contains(&card_state.rank)
                    })
                })
        }
        TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && ctx.card.is_some()
                && !crate::query::ddd_duanbing_transfer_cards(state, active.owner, ctx.player)
                    .is_empty()
        }
        TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_equip)
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map_or(0, |(_, count)| *count)
                        < 7
                })
        }
        TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded => {
            owner_alive
                && owner_is_source
                && active.owner != state.turn.current
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && ctx.card.is_some()
                && card_kind.is_some()
        }
        TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::Shan)
                && (owner_is_source || state.turn.current == active.owner)
        }
        TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && !state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        | TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged => {
            let damage_card = card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick());
            let damaged = ctx
                .targets
                .iter()
                .position(|target| *target == active.owner)
                .and_then(|index| 1u32.checked_shl(index as u32))
                .is_some_and(|bit| ctx.amount & bit != 0);
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && !owner_is_player
                && active.owner != ctx.source
                && damage_card
                && ctx.targets.contains(&active.owner)
                && (damaged
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
                    ))
        }
        TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && !owner_is_player
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|participant| participant.alive && participant.dying)
                && [active.owner, ctx.player, ctx.source]
                    .into_iter()
                    .filter(|target| {
                        target.is_some()
                            && state
                                .players
                                .get(*target)
                                .is_some_and(|participant| participant.alive)
                    })
                    .any(|target| {
                        if target == active.owner {
                            !crate::query::discardable_hand_equip_cards(state, target).is_empty()
                        } else {
                            !crate::query::discardable_target_cards(
                                state,
                                active.owner,
                                target,
                                rule_api::TargetCardScope::HandEquipJudge,
                            )
                            .is_empty()
                        }
                    })
        }
        TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }
        TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable => {
            let used = state
                .skills
                .get(active.skill_id)
                .and_then(|instance| {
                    (instance.state_blob.len() == 12 && &instance.state_blob[..4] == b"RNX1")
                        .then(|| {
                            let round =
                                u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap());
                            (round == state.turn.round).then(|| {
                                u32::from_le_bytes(instance.state_blob[8..12].try_into().unwrap())
                            })
                        })
                        .flatten()
                })
                .unwrap_or(0);
            let has_discard_target = state.players.alive_by_seat().into_iter().any(|target| {
                state.turn.flag(turn_flag::sha_used_by_player(target)) == 0
                    && !crate::query::discardable_target_cards(
                        state,
                        active.owner,
                        target,
                        rule_api::TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
            });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && state.turn.phase != Phase::Discard
                && state.turn.flag(turn_flag::NON_DISCARD_PHASE_DISCARD_SEEN) == 0
                && used < 2
                && (state
                    .players
                    .get(state.turn.current)
                    .is_some_and(|participant| participant.alive)
                    || has_discard_target)
        }
        TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable => {
            let current_use = ctx.card.and_then(|card| {
                state
                    .resolution
                    .pending_card_resolutions
                    .iter()
                    .rev()
                    .find(|record| record.card == card)
            });
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.targets.first() == Some(&ctx.player)
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick())
                && current_use
                    .is_some_and(|record| record.source == active.owner && record.source_was_hand)
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    active.owner,
                    rule_api::TargetCardScope::Hand,
                )
                .is_empty()
        }
        TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && !matches!(
                    ctx.move_reason,
                    CardMoveReason::UseOrRespond | CardMoveReason::Use
                )
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.owner.is_none() && card.kind.is_sha())
                })
        }
        TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseAnyEnd
                && !state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable => {
            let tracked = state.skills.get(active.skill_id).and_then(|instance| {
                (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"DYG1").then(
                    || {
                        CardId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        ))
                    },
                )
            });
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && event_player_alive
                && ctx.card.is_some_and(|card| Some(card) == tracked)
                && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .any(|zone| {
                        state
                            .zones
                            .find(zone, ctx.player)
                            .and_then(|id| state.zones.get(id))
                            .is_some_and(|zone| !zone.cards.is_empty())
                    })
        }
        TriggerFilter::OwnerMbrunweiLossThreshold => {
            let armed = state.skills.get(active.skill_id).is_some_and(|instance| {
                instance.state_blob.len() >= 16
                    && &instance.state_blob[..4] == b"RNW1"
                    && u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()) > 0
            });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && armed
                && !ctx.cards.is_empty()
                && !(ctx.move_reason == CardMoveReason::Use
                    && ctx.cards.iter().all(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.kind.is_equip())
                    }))
        }
        TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand => {
            let hand = state.zones.find(ZoneKind::Hand, active.owner);
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && state.skills.get(active.skill_id).is_some_and(|instance| {
                    instance.state_blob.len() >= 16
                        && &instance.state_blob[..4] == b"RNW1"
                        && instance.state_blob[16..].chunks_exact(4).any(|raw| {
                            state
                                .cards
                                .get(CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
                                .is_some_and(|card| Some(card.zone) == hand)
                        })
                })
        }
        TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && event_player_alive
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: ctx.player,
                            to: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        }
        TriggerFilter::OwnerSourceDamageMbxiongtuArmed => {
            let flags = state
                .skills
                .get(active.skill_id)
                .and_then(|instance| {
                    (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"XTG1")
                        .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
                })
                .unwrap_or(0);
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && flags & 0b001 != 0
                && flags & 0b100 == 0
        }
        TriggerFilter::OwnerPhasePlayEndMbxiongtuState => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && state.skills.get(active.skill_id).is_some_and(|instance| {
                    instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"XTG1"
                })
        }
        TriggerFilter::OwnerPhaseEndMbganggengTargetAlive => {
            let target = state.skills.get(active.skill_id).and_then(|instance| {
                (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"GGG1").then(
                    || {
                        PlayerId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        ))
                    },
                )
            });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && target.is_some_and(|target| {
                    state
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                })
        }
        TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn => {
            owner_alive
                && owner_is_player
                && turn_fires < 2
                && (ctx.kind == EventKind::Dying
                    || (ctx.kind == EventKind::CardsMoved
                        && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                        && hand_empty))
        }
        TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn => {
            owner_alive
                && owner_is_player
                && turn_fires < 2
                && matches!(ctx.kind, EventKind::PhasePrepare | EventKind::Damage)
                && state.players.iter_live().any(|source| {
                    source.alive
                        && !crate::query::movable_field_cards_from(state, source.id).is_empty()
                })
        }
        TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.target_count == 1
                && ctx.targets.first() == Some(&ctx.player)
                && turn_fires < 2
                && state.players.iter_live().filter(|role| role.alive).count() > 2
        }
        TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && ctx.card.is_some_and(|card| {
                    mbfeijing_record_matches(state, active.owner, card, Some(ctx.player))
                })
        }
        TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && card_is_sha
                && ctx.card.is_some_and(|card| {
                    ctx.target_count == 1
                        || mbfeijing_record_matches(state, active.owner, card, None)
                })
        }
        TriggerFilter::OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn => {
            let hand = state.hand_count(ctx.player);
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && turn_fires < 1
                && state
                    .players
                    .iter_live()
                    .all(|role| hand <= state.hand_count(role.id))
        }
        TriggerFilter::OwnerMbkuangxiangReceivedCardsExhausted => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && crate::effect::mbkuangxiang_exhausted_role(
                    state,
                    active.skill_id,
                    active.owner,
                    ctx.player,
                    &ctx.cards,
                )
        }
        TriggerFilter::OwnerPhasePlayMbkuangxiangActive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && state.skills.get(active.skill_id).is_some_and(|instance| {
                    instance.state_blob.len() >= 16 && &instance.state_blob[..4] == b"KXI1"
                })
        }
        TriggerFilter::OwnerGainedCardsPotduzuo => {
            let gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive && gained && ctx.amount > 0 && ctx.move_reason != CardMoveReason::Potduzuo
        }
        TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn => {
            let gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive && gained && ctx.amount > 0 && ctx.move_reason != CardMoveReason::Potduzuo
        }
        TriggerFilter::OwnerShaDamageBeforePotbihanAvailable => {
            let above_hp = |role| {
                state.players.get(role).is_some_and(|participant| {
                    participant.alive && state.hand_count(role) > participant.hp.max(0) as u32
                })
            };
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && (above_hp(active.owner) || above_hp(ctx.player))
        }
        TriggerFilter::OwnerAliveCardTargetedHealthy => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp.max(0) as u32 == player.max_hp)
        }
        TriggerFilter::OwnerAliveCardResolvedSourceCurrentPotsifengPile => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source == state.turn.current
                && !state
                    .skill_pile_cards(ctx.source, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerAlivePhaseEndPlayerPotsifengPile => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && !state
                    .skill_pile_cards(ctx.player, active.skill_def.raw())
                    .is_empty()
        }
        TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|player| state.turn.card_loss_count(player) > 0)
        }
        TriggerFilter::OwnerSourceCardResolvedPhasePolicy => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state
                        .skills
                        .get(active.skill_id)
                        .and_then(|instance| instance.phase_card_use_policy())
                        .is_some_and(|(armed, _, _)| armed == card)
                })
        }
        TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && (state.hand_count(active.owner) > 0
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()))
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }
        TriggerFilter::OwnerDiscardedShaToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_sha()
                    })
                })
        }
        TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit => {
            let recorded = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| {
                    (skill.state_blob.len() == 32 && &skill.state_blob[..4] == b"FUA1")
                        .then(|| u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap()))
                })
                .unwrap_or(0);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.current != active.owner
                && recorded & state.turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32 != 0
        }
        TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage => {
            let pending = state.skills.get(active.skill_id).and_then(|skill| {
                (skill.state_blob.len() == 32 && &skill.state_blob[..4] == b"FUA1").then(|| {
                    (
                        CardId::new(u32::from_le_bytes(
                            skill.state_blob[20..24].try_into().unwrap(),
                        )),
                        u32::from_le_bytes(skill.state_blob[28..32].try_into().unwrap()),
                    )
                })
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0
                && pending.is_some_and(|(pending, first)| ctx.card == Some(pending) && first != 0)
        }
        TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && event_player_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.pothuilv_state(state.turn.round).0 != 0b11)
        }
        TriggerFilter::OwnerIsTargetCanUseWuzhong => {
            owner_alive
                && owner_is_player
                && matches!(ctx.kind, EventKind::PhasePrepare | EventKind::Damage)
                && crate::command::play::source_free_wuzhong_has_legal_use(state, active.owner)
        }
        TriggerFilter::OwnerAliveDiscardBatch => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && event_player_alive
                && !ctx.cards.is_empty()
        }
        TriggerFilter::OwnerAliveOtherRespondedToOwner => {
            owner_alive
                && ctx.kind == EventKind::CardResponded
                && event_player_alive
                && ctx.player != active.owner
                && ctx.respond_to == active.owner
        }
        TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potjianzhuan_timing_available(state, active, 0b001)
        }
        TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potjianzhuan_timing_available(state, active, 0b010)
        }
        TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && potjianzhuan_timing_available(state, active, 0b100)
        }
        TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potcaoshuang_timing_ready(state, active, 0b001)
        }
        TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potcaoshuang_timing_ready(state, active, 0b010)
        }
        TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && potcaoshuang_timing_ready(state, active, 0b100)
        }
        TriggerFilter::OwnerUsedPotnizunPileCard => {
            owner_alive
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && (ctx.source == active.owner || ctx.player == active.owner)
                && ctx.card.is_some_and(|card| {
                    state.turn.used_playable_skill_pile_card(active.owner, card)
                        && state.card_has_content_tag(card, crate::state::SHEQUAN_CARD_TAG)
                })
        }
        TriggerFilter::OwnerTargetDamageBeforePotnizunUsed => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.potnizun_use_count(state.turn.round) > 0)
        }
        TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .turn
                    .flag(turn_flag::card_used_by_player(active.owner))
                    > 0
                && discard.is_some_and(|discard| {
                    state.zones.get(discard).is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state.card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                        })
                    })
                })
        }
        TriggerFilter::OwnerSourceCardName(selector) => {
            // Owner used/responded a card whose logical kind/suit matches the selector.
            event_card_matches_selector(state, active.owner, ctx, *selector)
                && owner_is_source
                && matches!(
                    ctx.kind,
                    EventKind::CardUsed | EventKind::CardResolved | EventKind::CardResponded
                )
        }
        TriggerFilter::OwnerIsTargetCardName(selector) => {
            // Owner is the target of a card whose logical kind/suit matches the selector.
            owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && event_card_matches_selector(state, active.owner, ctx, *selector)
        }
        TriggerFilter::OwnerIsTargetCardDamage(selector) => {
            // Owner took damage whose originating card matches the selector.
            owner_is_player
                && ctx.kind == EventKind::Damage
                && event_card_matches_selector(state, active.owner, ctx, *selector)
        }
        TriggerFilter::EventViewAsSkill(skill_def) => {
            // Event card originated from a view-as skill. `view_as_skill` is the skill INSTANCE id;
            // resolve to its definition id and compare. `_backup` view-as helpers (engine-internal)
            // also have resolvable instances, so this covers them too.
            view_as_skill != 0
                && state
                    .skills
                    .get(engine_api::SkillId::new(view_as_skill))
                    .is_some_and(|skill| skill.def.raw() == *skill_def)
        }
        TriggerFilter::EventViewAsSkillName(name) => {
            // Event card originated from a view-as skill whose content-metadata name matches.
            // Covers dynamic _backup helpers without a static def id (ADR 0639 architecture limit).
            view_as_skill != 0
                && state
                    .skills
                    .get(engine_api::SkillId::new(view_as_skill))
                    .and_then(|skill| state.content.skill_metadata(skill.def))
                    .is_some_and(|metadata| metadata.name == *name)
        }
    }
}

fn potcaoshuang_timing_ready(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    timing: u8,
) -> bool {
    state.skills.get(active.skill_id).is_some_and(|skill| {
        matches!(
            skill.def.raw(),
            crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
        ) && {
            let (assigned, triggered, _) = skill.potcaoshuang_state(state.turn.round);
            assigned & timing != 0 && triggered & timing == 0
        }
    })
}

fn potjianzhuan_timing_available(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    timing: u8,
) -> bool {
    let Some(skill) = state.skills.get(active.skill_id) else {
        return false;
    };
    if skill.def.raw() != crate::state::POTJIANZHUAN_DEF || skill.lifetime_fire_count != 0 {
        return false;
    }
    let assigned = state
        .players
        .get(active.owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|id| state.skills.get(*id))
        .filter(|instance| {
            matches!(
                instance.def.raw(),
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) && state.skill_effectively_enabled(instance)
        })
        .fold(0, |mask, instance| {
            mask | instance.potcaoshuang_state(state.turn.round).0
        });
    assigned != 0b111 && assigned & timing == 0 || assigned == 0b111
}

fn mbfeijing_record_matches(
    state: &crate::state::GameState,
    owner: PlayerId,
    card: CardId,
    target: Option<PlayerId>,
) -> bool {
    state
        .players
        .get(owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .any(|skill| {
            let blob = &skill.state_blob;
            blob.get(..4) == Some(b"FJG1")
                && blob.len() >= 12
                && (blob.len() - 12) % 8 == 0
                && u32::from_le_bytes(blob[4..8].try_into().unwrap()) == card.raw()
                && target.is_none_or(|target| {
                    blob[12..].chunks_exact(8).any(|pair| {
                        u32::from_le_bytes(pair[..4].try_into().unwrap()) == target.raw()
                    })
                })
        })
}

fn xicha_state_has_source(blob: &[u8], source: PlayerId) -> bool {
    if blob.get(..4) != Some(b"XCH1") {
        return false;
    }
    let mut cursor = 4;
    while cursor + 5 <= blob.len() {
        let saved = PlayerId::new(u32::from_le_bytes(
            blob[cursor..cursor + 4].try_into().unwrap(),
        ));
        let count = blob[cursor + 4] as usize;
        cursor += 5;
        let bytes = count.saturating_mul(4);
        if cursor + bytes > blob.len() {
            return false;
        }
        if saved == source {
            return true;
        }
        cursor += bytes;
    }
    false
}

fn same_sex(left: &str, right: &str) -> bool {
    !matches!(left, "" | "unknown")
        && !matches!(right, "" | "unknown")
        && (left == "double" || right == "double" || left == right)
}

fn same_logical_card_name(
    previous_kind: CardKind,
    previous_def: u32,
    current_kind: CardKind,
    current_def: u32,
) -> bool {
    let canonical = |kind: CardKind| {
        if kind.is_sha() {
            CardKind::Sha
        } else {
            kind
        }
    };
    let previous_kind = canonical(previous_kind);
    let current_kind = canonical(current_kind);
    previous_kind == current_kind
        && (!previous_kind.is_equip() && previous_kind != CardKind::Other
            || previous_def == current_def)
}

fn turn_limit(filter: &TriggerFilter) -> Option<u32> {
    match filter {
        TriggerFilter::OncePerTurn
        | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        | TriggerFilter::OwnerSourceOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherShaOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        | TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn
        | TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        | TriggerFilter::OwnerAliveFirstCardsMovedToHand
        | TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        | TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(_)
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        | TriggerFilter::OwnerSourceDamageCardOncePerTurn
        | TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        | TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        | TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        | TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(_)
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        | TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        | TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        | TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        | TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        | TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(_) => Some(1),
        TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable => Some(1),
        TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceWoundedExistsOncePerTurn => Some(1),
        TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn => Some(1),
        TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn => Some(1),
        TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        | TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn => Some(u32::MAX),
        TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceOtherTwicePerTurn
        | TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn
        | TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        | TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        | TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn => Some(2),
        _ => None,
    }
}

/// 整肃三技能中该角色未有效拥有的数量（ABI 432）。
fn zhengsu_missing_count(
    state: &crate::state::GameState,
    player: engine_api::PlayerId,
    defs: &[u32; 3],
) -> u32 {
    defs.iter()
        .filter(|def| {
            !state.skills.sorted_live().iter().any(|instance| {
                instance.owner == player
                    && instance.def.raw() == **def
                    && state.skill_effectively_enabled(instance)
            })
        })
        .count() as u32
}

fn play_phase_limit(filter: &TriggerFilter) -> Option<u32> {
    match filter {
        TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        | TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        | TriggerFilter::OwnerAliveLostHandThisTurn
        | TriggerFilter::OwnerSourceFirstShaThisPlayPhase => Some(1),
        TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo => Some(2),
        TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase => Some(3),
        _ => None,
    }
}

fn once_per_round(filter: &TriggerFilter) -> bool {
    matches!(
        filter,
        TriggerFilter::OwnerAliveOtherDyingOncePerRound
            | TriggerFilter::OwnerAliveSourceAliveOncePerRound
            | TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
            | TriggerFilter::OwnerAliveOncePerRound
            | TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
            | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
            | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
            | TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
            | TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
    )
}

fn lifetime_limit(filter: &TriggerFilter) -> Option<u32> {
    matches!(
        filter,
        TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
            | TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
            | TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
            | TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
            | TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
            | TriggerFilter::OwnerDyingZhupeilanOncePerGame
            | TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
            | TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
    )
    .then_some(1)
}

fn native_handler_effects(active: &ActiveTrigger, handler: &TriggerHandler) -> Vec<EffectRequest> {
    match handler {
        TriggerHandler::DrawSelf { count } => vec![EffectRequest::DrawCardsFromSkill {
            player: active.owner,
            count: *count,
            skill: active.skill_id,
        }],
        TriggerHandler::EmitNote { code } => vec![EffectRequest::EmitRuleEvent {
            code: *code,
            payload: vec![active.skill_id.raw()],
        }],
        TriggerHandler::RecoverSelf { amount } => vec![EffectRequest::RecoverHp {
            source: active.owner,
            player: active.owner,
            amount: *amount,
        }],
        TriggerHandler::RunVm { .. } => Vec::new(),
    }
}

fn timing_from_u16(v: u16) -> TimingBand {
    match v {
        0 => TimingBand::FirstDo,
        200 => TimingBand::LastDo,
        _ => TimingBand::Normal,
    }
}

/// Decrement trigger budget once per planned trigger batch item that is a "fire".
pub fn spend_trigger_budget(tx: &mut Transaction, n: u32) -> Result<(), EffectError> {
    let b = tx.state().resolution.trigger_budget;
    if b < n {
        return Err(EffectError::BudgetExceeded);
    }
    tx.state_mut().resolution.trigger_budget = b - n;
    Ok(())
}

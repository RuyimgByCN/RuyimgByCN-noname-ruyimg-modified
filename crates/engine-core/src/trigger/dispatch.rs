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

mod filters_always;
mod filters_dcyinjun_tracked_sha_resolved;
mod filters_owner_alive_ddd_fengzheng_round_pending;
mod filters_owner_alive_other_marked_xi;
mod filters_owner_alive_turn_ended_rehengjiang_pending;
mod filters_owner_black_card_moved_equip_judge;

mod ddd_shixing_female_available;
pub use ddd_shixing_female_available::*;
mod remember_card_action;
pub use remember_card_action::*;
mod filter_match;
pub use filter_match::*;

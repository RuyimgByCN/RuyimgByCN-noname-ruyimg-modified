//! Trigger index and stable total-order dispatch (Phase 3).
//!
//! Architecture §8:
//! EventKind → TriggerDefinition slice → Active instances → filter → stable sort → execute.
//!
//! Aligned with noname `event.trigger` / `arrangeTrigger` *ordering intent*
//! (firstDo / priority / seat / lastDo), not Promise chains.

mod catalog;
mod dispatch;
mod index;

pub use catalog::{
    bootstrap_trigger_defs, defs_for_skill, TriggerDefId, TriggerDefinition, TriggerFilter,
    TriggerHandler, BOOTSTRAP_JIANXIONG, BOOTSTRAP_MASHU, BOOTSTRAP_QICAO,
};
pub(crate) use dispatch::dispatch_for_events_with_damage_context;
pub(crate) use dispatch::plan_triggers;
pub use dispatch::{
    dispatch_for_events, run_pending_triggers, spend_trigger_budget, PendingTrigger,
    TriggerContext, TriggerFireRecord,
};
pub use index::{ActiveTrigger, ActiveTriggerIndex};

use serde::{Deserialize, Serialize};

/// Event kinds that can wake triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKind {
    /// After damage is dealt (DamageDealt domain event).
    Damage,
    /// HP was lost without damage.
    HpLost,
    /// After a card is used.
    CardUsed,
    /// Phase began (TurnStateChanged into a phase).
    PhaseBegin,
    /// Preparation phase began.
    PhasePrepare,
    /// Phase ended (conceptual; used when leaving a phase).
    PhaseEnd,
    /// The play phase ended.
    PhasePlayEnd,
    /// Player entered dying.
    Dying,
    /// Player left dying after being saved.
    DyingAfter,
    /// Player's maximum HP changed.
    MaxHpChanged,
    /// Player killed.
    Death,
    /// Cards drawn.
    CardsDrawn,
    /// Cards moved.
    CardsMoved,
    /// Custom content code.
    Custom(u32),
    /// Card played as a response.
    CardResponded,
    /// One declared target of a used card, emitted in target order.
    CardTargeted,
    /// A draw phase was skipped or cancelled.
    PhaseDrawSkipped,
    /// Match setup and opening hands completed, before the first turn phase begins.
    GameStarted,
    /// Judgment phase began.
    PhaseJudge,
    /// Draw phase began.
    PhaseDraw,
    /// Play phase began.
    PhasePlay,
    /// Discard phase began.
    PhaseDiscard,
    /// A player gained one skill instance.
    SkillAdded,
    /// A player's whole turn ended.
    TurnEnded,
    /// After HP is recovered.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    HpRecovered,
    /// Before prepared damage mutates HP; content may prevent the current damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DamageBefore,
    /// After a Duel resolves with a winner and loser.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DuelResolved,
    /// One equipment slot became disabled.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EquipSlotDisabled,
    /// A character-card state changed, such as turning over or linking/unlinking.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CharacterStateChanged,
    /// A logical Sha was fully dodged by its current target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDodged,
    /// The draw phase ended.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    PhaseDrawEnd,
    /// One logical Sha use fully resolved across every target and Shan response.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaResolved,
    /// A two-player hand comparison started before the second card is finalized.
    HandCompareStarted,
    /// A two-player hand comparison completed after both cards left hand.
    HandCompareFinished,
    /// A debate settled (ADR 0644 P2).
    DebateResolved,
    /// A debate opened, before opinions show (ADR 0644 P3).
    DebateStarted,
    /// A judgment completed after every rewrite opportunity settled.
    JudgmentResolved,
    /// Any concrete turn phase ended.
    ///
    /// Appended to preserve existing serialized enum ordinals.
    PhaseAnyEnd,
    /// One disabled copy of an equipment slot was restored.
    ///
    /// Appended to preserve existing serialized enum ordinals.
    EquipSlotEnabled,
    /// One player gave physical cards to another player's hand.
    ///
    /// Appended to preserve existing serialized enum ordinals.
    CardsGiven,
    /// The discard phase ended.
    ///
    /// Appended to preserve existing serialized enum ordinals.
    PhaseDiscardEnd,
    /// Positive damage was cancelled or reduced to zero before HP changed.
    ///
    /// Appended to preserve existing serialized enum ordinals.
    DamagePrevented,
    /// One complete logical card use ended after nested responses and choices. ABI 503.
    CardResolved,
    /// Any phase was skipped or cancelled. ABI 622.
    PhaseAnySkipped,
    /// Before recovery mutates HP; content may replace/cancel it. ABI 635.
    RecoverBefore,
    /// A new round began (round number incremented at seat-0 turn rollover).
    /// Appended for ordinal stability. Foundation for upstream roundStart triggers.
    RoundStart,
    /// The previous round fully ended at the seat-0 turn rollover, before `RoundStart`.
    ///
    /// Appended to preserve existing serialized enum ordinals. Foundation for upstream
    /// roundEnd triggers.
    RoundEnd,
}

impl EventKind {
    /// Stable tag for maps / hashing of catalogs.
    pub fn tag(self) -> u32 {
        match self {
            EventKind::Damage => 1,
            EventKind::CardUsed => 2,
            EventKind::PhaseBegin => 3,
            EventKind::PhaseEnd => 4,
            EventKind::Dying => 5,
            EventKind::Death => 6,
            EventKind::CardsDrawn => 7,
            EventKind::CardsMoved => 8,
            EventKind::Custom(c) => 0x8000_0000 | c,
            EventKind::CardResponded => 9,
            EventKind::PhasePrepare => 10,
            EventKind::HpLost => 11,
            EventKind::DyingAfter => 12,
            EventKind::MaxHpChanged => 13,
            EventKind::PhasePlayEnd => 14,
            EventKind::CardTargeted => 15,
            EventKind::PhaseDrawSkipped => 16,
            EventKind::GameStarted => 17,
            EventKind::PhaseJudge => 18,
            EventKind::PhaseDraw => 19,
            EventKind::PhasePlay => 20,
            EventKind::PhaseDiscard => 21,
            EventKind::SkillAdded => 22,
            EventKind::TurnEnded => 23,
            EventKind::HpRecovered => 24,
            EventKind::DamageBefore => 25,
            EventKind::DuelResolved => 26,
            EventKind::EquipSlotDisabled => 27,
            EventKind::CharacterStateChanged => 28,
            EventKind::ShaDodged => 29,
            EventKind::PhaseDrawEnd => 30,
            EventKind::ShaResolved => 31,
            EventKind::HandCompareStarted => 32,
            EventKind::DebateResolved => 45,
            EventKind::DebateStarted => 46,
            EventKind::HandCompareFinished => 33,
            EventKind::JudgmentResolved => 34,
            EventKind::PhaseAnyEnd => 35,
            EventKind::EquipSlotEnabled => 36,
            EventKind::CardsGiven => 37,
            EventKind::PhaseDiscardEnd => 38,
            EventKind::DamagePrevented => 39,
            EventKind::CardResolved => 40,
            EventKind::PhaseAnySkipped => 41,
            EventKind::RecoverBefore => 42,
            EventKind::RoundStart => 43,
            EventKind::RoundEnd => 44,
        }
    }
}

/// Timing band — mirrors noname firstDo / normal / lastDo queues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u16)]
pub enum TimingBand {
    /// Execute before normal seat-ordered triggers.
    FirstDo = 0,
    /// Default band.
    Normal = 100,
    /// Execute after all seats.
    LastDo = 200,
}

/// Sort key for simultaneous triggers (architecture §8.2).
///
/// Total order (ascending):
/// `Timing → Priority → Optional → SeatRel → SkillInstance → TriggerDef`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TriggerOrderKey {
    /// Timing band ordinal.
    pub timing: u16,
    /// Priority (lower runs first; ruleset-fixed).
    pub priority: i32,
    /// 0 = mandatory, 1 = optional.
    pub optional: u8,
    /// Seat order relative to current turn (0 = current, then clockwise).
    pub seat_rel: u8,
    /// Skill instance id.
    pub skill_instance: u32,
    /// Trigger definition id.
    pub trigger_def: u32,
}

impl TriggerOrderKey {
    /// Build key from components.
    pub fn new(
        timing: TimingBand,
        priority: i32,
        optional: bool,
        seat_rel: u8,
        skill_instance: u32,
        trigger_def: u32,
    ) -> Self {
        Self {
            timing: timing as u16,
            priority,
            optional: u8::from(optional),
            seat_rel,
            skill_instance,
            trigger_def,
        }
    }
}

/// Compute seat-relative order from current player (0 = current, then clockwise by seat).
pub fn seat_rel_to_current(
    alive_by_seat: &[engine_api::PlayerId],
    current: engine_api::PlayerId,
    owner: engine_api::PlayerId,
) -> u8 {
    if alive_by_seat.is_empty() {
        return 0;
    }
    let cur_idx = alive_by_seat
        .iter()
        .position(|p| *p == current)
        .unwrap_or(0);
    let own_idx = alive_by_seat.iter().position(|p| *p == owner).unwrap_or(0);
    let n = alive_by_seat.len();
    ((own_idx + n - cur_idx) % n) as u8
}

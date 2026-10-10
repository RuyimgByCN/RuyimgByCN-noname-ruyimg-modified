//! ChoiceRequest: rule-state input barrier (not a UI Promise).

use crate::ids::{ChoiceId, PlayerId};
use serde::{Deserialize, Serialize};

/// A pending choice the engine cannot continue past until resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceRequest {
    /// Unique choice id for this match.
    pub choice_id: ChoiceId,
    /// Observer who must answer (usually a player).
    pub observer: PlayerId,
    /// Options the player may select (or empty if constraint-only).
    pub options: Vec<ChoiceOption>,
    /// Minimum number of selections.
    pub min_count: u8,
    /// Maximum number of selections.
    pub max_count: u8,
    /// Whether cancel is allowed.
    pub allow_cancel: bool,
    /// Timeout policy identifier (ruleset-defined).
    pub timeout_policy: TimeoutPolicy,
    /// Associated state revision when the choice was published.
    pub revision: u64,
    /// VM / effect continuation handle (opaque to clients).
    pub continuation_id: u64,
    /// Human-oriented choice kind for UI (not hashed into rules).
    pub kind: ChoiceKind,
}

/// Single selectable option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption {
    /// Option index / token returned in ResolveChoice.
    pub token: u32,
    /// Optional display key (localization id); empty if pure entity target.
    pub label_key: String,
}

/// Timeout policy tag (concrete durations live outside the core rules clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeoutPolicy {
    /// No automatic timeout (offline / debug).
    #[default]
    None,
    /// Standard multiplayer response window.
    Standard,
    /// Short response (confirm / yes-no).
    Short,
    /// Tournament policy.
    Tournament,
}

/// Semantic kind of choice (for projection / UI routing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceKind {
    /// Select targets for a card or skill.
    SelectTargets,
    /// Select cards from a known set.
    SelectCards,
    /// Yes / no confirm.
    Confirm,
    /// Order simultaneous triggers.
    OrderTriggers,
    /// Discard down to hand limit.
    DiscardToLimit,
    /// Generic / content-defined.
    Custom,
}

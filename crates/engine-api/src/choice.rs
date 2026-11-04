//! ChoiceRequest: rule-state input barrier (not a UI Promise).

use crate::ids::{ChoiceId, PlayerId};
use serde::{Deserialize, Serialize};

/// A pending choice the engine cannot continue past until resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChoiceRequest {
    /// Unique choice id for this match.
    pub choice_id: ChoiceId,
    /// Observer who must answer (usually a player).
    pub observer: PlayerId,
    /// Options the player may select (or empty if constraint-only).
    pub options: Vec<ChoiceOption>,
    /// Minimum number of selections.
    pub min_count: u32,
    /// Maximum number of selections.
    pub max_count: u32,
    /// Whether cancel is allowed.
    pub allow_cancel: bool,
    /// Timeout policy identifier (ruleset-defined).
    pub timeout_policy: TimeoutPolicy,
    /// Server Unix deadline; absent for offline and legacy rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_unix_ms: Option<u64>,
    /// Associated state revision when the choice was published.
    pub revision: u64,
    /// VM / effect continuation handle (opaque to clients).
    pub continuation_id: u64,
    /// Human-oriented choice kind for UI (not hashed into rules).
    pub kind: ChoiceKind,
    /// Static prompt key for presentation; absent in older payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Precise native choice kind, restricted to the five canonical names.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_native_kind")]
    pub native_kind: Option<String>,
    /// Known native skill producing this owner-visible ask; never command authority.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_source_skill")]
    pub source_skill: Option<String>,
    /// Owner/referee-visible structured choice context; omitted when absent or hidden.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_choice_context")]
    pub context: Option<ChoiceContext>,
}

/// Public context attached to an owner/referee-visible choice request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChoiceContext {
    /// Tiqi sign choice adjustment context (`target` is 1-based API `PlayerId`, `delta = |n - 2|`).
    TiqiAdjustment {
        /// Logical skill identity producing this Tiqi adjustment.
        skill: crate::view::QualifiedSkillRef,
        /// Target seat receiving the hand-limit adjustment.
        target: PlayerId,
        /// Unsigned adjustment magnitude `|n - 2|`.
        delta: u32,
    },
}

impl<'de> Deserialize<'de> for ChoiceRequest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let choice = ChoiceRequestFields::deserialize(deserializer)?;
        if choice.min_count > choice.max_count {
            return Err(serde::de::Error::custom("choice min_count exceeds max_count"));
        }
        Ok(choice)
    }
}
#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(remote = "ChoiceRequest", rename = "ChoiceRequest")]
struct ChoiceRequestFields {
    choice_id: ChoiceId,
    observer: PlayerId,
    options: Vec<ChoiceOption>,
    min_count: u32,
    max_count: u32,
    allow_cancel: bool,
    timeout_policy: TimeoutPolicy,
    #[serde(default)]
    deadline_unix_ms: Option<u64>,
    revision: u64,
    continuation_id: u64,
    kind: ChoiceKind,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default, deserialize_with = "deserialize_native_kind")]
    native_kind: Option<String>,
    #[serde(default, deserialize_with = "deserialize_source_skill")]
    source_skill: Option<String>,
    #[serde(default, deserialize_with = "deserialize_choice_context")]
    context: Option<ChoiceContext>,
}

/// Preserve the existing wire enum while retaining precise native choice metadata.
pub fn choice_kind_from_native(kind: &str) -> Option<ChoiceKind> {
    Some(match kind {
        "confirm" => ChoiceKind::Confirm,
        "select_cards" => ChoiceKind::SelectCards,
        "select_players" => ChoiceKind::SelectTargets,
        "respond_card" | "pick_tag" => ChoiceKind::Custom,
        _ => return None,
    })
}

fn deserialize_native_kind<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let value = Option::<String>::deserialize(deserializer)?;
    if value.as_deref().is_some_and(|kind| choice_kind_from_native(kind).is_none()) {
        return Err(serde::de::Error::custom("invalid native_kind"));
    }
    Ok(value)
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

fn deserialize_source_skill<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let value = Option::<String>::deserialize(deserializer)?;
    if value.as_deref().is_some_and(str::is_empty) {
        return Err(serde::de::Error::custom("empty source_skill"));
    }
    Ok(value)
}

fn deserialize_choice_context<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ChoiceContext>, D::Error> {
    let value = Option::<ChoiceContext>::deserialize(deserializer)?;
    if let Some(ChoiceContext::TiqiAdjustment {
        skill,
        target,
        delta,
    }) = &value
    {
        if skill.namespace.is_empty()
            || skill.package_id.is_empty()
            || *target == PlayerId::NONE
            || *delta == 0
        {
            return Err(serde::de::Error::custom("invalid choice context"));
        }
    }
    Ok(value)
}

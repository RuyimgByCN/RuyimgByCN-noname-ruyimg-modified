//! Frozen WS5 reader fields from HEAD 100489f5, before approved B.
use engine_api::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OldSeatView {
    /// Player id.
    pub player: PlayerId,
    /// Stable character definition id.
    #[serde(default)]
    pub character_def: u32,
    /// Seat index.
    pub seat: SeatIndex,
    /// Current HP (public in standard rules).
    pub hp: i32,
    /// Max HP.
    pub max_hp: u32,
    /// Alive flag.
    pub alive: bool,
    /// Whether the character card is currently face-down.
    #[serde(default)]
    pub turned_over: bool,
    /// Hand count (not hand contents unless authorized).
    pub hand_count: u32,
    /// Known identity if revealed / authorized; otherwise unknown.
    pub identity: Option<IdentityTag>,
    /// Public skill instances owned by this player.
    pub skills: Vec<SkillView>,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OldChoiceRequest {
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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OldObserverSnapshot {
    /// Revision of the authority state this view is derived from.
    pub revision: u64,
    /// Canonical hash of authority state (not of the view).
    pub state_hash: [u8; 32],
    /// Observer role.
    pub observer: ObserverKind,
    /// Current round number (public turn state).
    pub round: u32,
    /// Current turn player, or `PlayerId::NONE` between turns.
    pub current_player: PlayerId,
    /// Current public phase.
    pub phase: engine_api::TurnPhaseTag,
    /// Seat order (public).
    pub seats: Vec<OldSeatView>,
    /// Public / authorized cards known to this observer.
    pub known_cards: Vec<CardView>,
    /// Match outcome if finished.
    pub outcome: MatchOutcome,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum OldServerMessage {
    Snapshot {
        header: engine_protocol::MessageHeader,
        view: OldObserverSnapshot,
        legal_actions: Vec<LegalAction>,
        choice: Option<OldChoiceRequest>,
    },
}

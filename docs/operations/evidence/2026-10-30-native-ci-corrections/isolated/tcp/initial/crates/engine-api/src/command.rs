//! Normalized player / system commands.

use crate::ids::{CardId, ChoiceId, PlayerId, SkillId};
use serde::{Deserialize, Serialize};

/// Idempotency key for network retries. Zero means "not set".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct IdempotencyKey(pub u64);

/// Expected state revision when submitting a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ExpectedRevision(pub u64);

/// Fully normalized command after auth and room sequencing.
///
/// Only accepted commands enter the deterministic command log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    /// Submitting player (or system actor).
    pub actor: PlayerId,
    /// Expected revision for optimistic concurrency; ignored in headless single-thread.
    pub expected_revision: ExpectedRevision,
    /// Network idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Payload.
    pub kind: CommandKind,
}

/// Command variants that advance (or attempt to advance) rule state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CommandKind {
    /// Create and start a match from configuration.
    StartGame {
        /// Player seat count (2–8 for standard identity).
        player_count: u8,
        /// Deterministic RNG seed.
        rng_seed: u64,
        /// Optional ruleset package id string (logical).
        ruleset: String,
    },
    /// Play a card from hand / legal zone.
    PlayCard {
        /// Card instance.
        card: CardId,
        /// Declared targets, stable order.
        targets: Vec<PlayerId>,
        /// Optional extra card targets (e.g. discard for cost).
        card_targets: Vec<CardId>,
    },
    /// Activate an active skill.
    ActivateSkill {
        /// Skill instance.
        skill: SkillId,
        /// Player targets.
        targets: Vec<PlayerId>,
        /// Card targets / cost cards.
        card_targets: Vec<CardId>,
    },
    /// Use a package-declared source-free virtual card conversion.
    UseSkillCard {
        /// Skill instance providing the conversion.
        skill: SkillId,
        /// Stable logical card-kind tag.
        card_kind: u8,
        /// Declared targets, stable order.
        targets: Vec<PlayerId>,
        /// Equipment slot selected for the conversion cost; zero when unused.
        equip_slot: u8,
    },
    /// Resolve a pending ChoiceRequest.
    ResolveChoice {
        /// Choice id from the request.
        choice_id: ChoiceId,
        /// Selected option indices or entity ids encoded as u32.
        selections: Vec<u32>,
        /// Whether the player cancelled (if allowed).
        cancelled: bool,
    },
    /// End current phase (e.g. discard phase done / skip play).
    EndPhase,
    /// Guozhan: reveal (明置) one of the actor's face-down generals.
    /// `slot` is 0 for the main general (主将) or 1 for the lieutenant (副将).
    /// See ADR 0598 phase C follow-up.
    RevealGeneral {
        /// Which general slot to reveal (0 = main, 1 = lieutenant).
        slot: u8,
    },
    /// Voluntary surrender.
    Surrender,
    /// System timeout → auto action.
    TimeoutAction {
        /// Related choice, if any.
        choice_id: Option<ChoiceId>,
    },
    /// Admin abort.
    AdminAbortMatch {
        /// Reason code for audit (not part of rule semantics beyond abort).
        reason_code: u32,
    },
}

/// A concrete action that the authority has already validated as legal for one player.
///
/// Hosts may expose this personalized list to UI or AI callers. It contains only the
/// acting player's authorized entity ids and never represents authority state itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegalAction {
    /// End the current phase.
    EndPhase,
    /// Play a card with an exact target vector.
    PlayCard {
        /// Card instance.
        card: CardId,
        /// Exact ordered targets.
        targets: Vec<PlayerId>,
    },
    /// Activate an owned skill with exact inputs.
    ActivateSkill {
        /// Runtime skill instance.
        skill: SkillId,
        /// Exact ordered player targets.
        targets: Vec<PlayerId>,
        /// Exact ordered card inputs.
        card_targets: Vec<CardId>,
    },
    /// Use a package-declared source-free virtual card conversion.
    UseSkillCard {
        /// Runtime skill instance.
        skill: SkillId,
        /// Stable logical card-kind tag.
        card_kind: u8,
        /// Exact ordered targets.
        targets: Vec<PlayerId>,
        /// Equipment slot selected for the conversion cost; zero when unused.
        equip_slot: u8,
    },
    /// Resolve the current choice with an exact selection vector.
    ResolveChoice {
        /// Choice id.
        choice_id: ChoiceId,
        /// Exact selections.
        selections: Vec<u32>,
        /// Whether this action cancels the choice.
        cancelled: bool,
    },
}

impl LegalAction {
    /// Convert this authority-generated action into a revision-bound command.
    pub fn to_command(&self, actor: PlayerId, expected_revision: u64) -> Command {
        let kind = match self {
            Self::EndPhase => CommandKind::EndPhase,
            Self::PlayCard { card, targets } => CommandKind::PlayCard {
                card: *card,
                targets: targets.clone(),
                card_targets: vec![],
            },
            Self::ActivateSkill {
                skill,
                targets,
                card_targets,
            } => CommandKind::ActivateSkill {
                skill: *skill,
                targets: targets.clone(),
                card_targets: card_targets.clone(),
            },
            Self::UseSkillCard {
                skill,
                card_kind,
                targets,
                equip_slot,
            } => CommandKind::UseSkillCard {
                skill: *skill,
                card_kind: *card_kind,
                targets: targets.clone(),
                equip_slot: *equip_slot,
            },
            Self::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            } => CommandKind::ResolveChoice {
                choice_id: *choice_id,
                selections: selections.clone(),
                cancelled: *cancelled,
            },
        };
        Command {
            actor,
            expected_revision: ExpectedRevision(expected_revision),
            idempotency_key: IdempotencyKey(0),
            kind,
        }
    }
}

/// Result of submitting a command to the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandResult {
    /// Accepted; state advanced to `new_revision`.
    Accepted {
        /// New state revision after commit.
        new_revision: u64,
        /// Canonical state hash after commit.
        state_hash: [u8; 32],
    },
    /// Rejected without state mutation.
    Rejected {
        /// Machine-readable reason.
        reason: RejectReason,
    },
}

/// Why a command was rejected (no state change).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectReason {
    /// Unknown or illegal actor.
    InvalidActor,
    /// Revision mismatch.
    StaleRevision {
        /// Current revision.
        current: u64,
        /// Expected by client.
        expected: u64,
    },
    /// Duplicate idempotency key already applied.
    DuplicateIdempotency,
    /// Command not legal in current state.
    IllegalAction {
        /// Short code.
        code: String,
    },
    /// Choice id missing / expired / wrong revision.
    InvalidChoice,
    /// Match already finished.
    MatchOver,
    /// Engine not started.
    NotStarted,
    /// Budget or internal content fault surfaced as rejection at boundary.
    Fault {
        /// Fault class.
        kind: FaultKind,
        /// Message (must not leak hidden info on wire).
        message: String,
    },
}

/// Fault classification from the architecture plan §6.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultKind {
    /// Content / VM error.
    ContentFault,
    /// Core invariant failure.
    EngineFault,
    /// Effect / trigger / VM budget exceeded.
    BudgetExceeded,
}

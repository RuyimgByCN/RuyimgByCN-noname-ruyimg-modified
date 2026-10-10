//! Guarded-choice openers: the shared skeleton behind "check preconditions, build a
//! candidate set, open one choice".
//!
//! Most skill entry points that begin with a player decision have the same three-part
//! shape (ADR 0595):
//!
//! 1. a guard — the originating skill is enabled, the participants are alive, and the
//!    turn/phase context is right; if it fails, the skill silently does nothing;
//! 2. a candidate set — the cards or players the chooser may pick from; if it comes out
//!    empty the skill likewise does nothing;
//! 3. a single [`open_choice`] that parks a continuation and hands control to the chooser.
//!
//! Only step 2 genuinely differs between skills. Steps 1 and 3 were duplicated across
//! dozens of `resolve_*` functions, so they live here instead. The candidate set stays
//! with the caller: it is the part that actually encodes the skill.
//!
//! This is deliberately *not* a new `EffectRequest` variant. Adding one costs eight
//! files of wiring (`rule-api` variant, bytecode, VM, `vm_host` context fill, `hash`
//! exhaustive match, the primitive, `resolve_choice`, and the compiler's
//! typecheck/codegen). These helpers are plain functions, so folding a skill onto them
//! removes wiring rather than adding it.

use super::*;

/// Who must be alive for a guarded choice to proceed.
///
/// Checked in the listed order, so callers get the same short-circuit behaviour as the
/// hand-written `||` chains these replace.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Participants {
    /// The skill owner.
    pub(crate) owner: PlayerId,
    /// An optional second participant (target, source, actor, …), required to be alive.
    pub(crate) other: Option<PlayerId>,
    /// When true, `other` must be a different seat from `owner`.
    pub(crate) distinct: bool,
    /// Whether `owner` must be alive. A few skills resolve from a dead owner and only
    /// check that the skill itself is enabled, so this is not unconditional.
    pub(crate) owner_must_be_alive: bool,
}

impl Participants {
    /// Only the skill owner participates, and must be alive.
    pub(crate) fn owner(owner: PlayerId) -> Self {
        Self {
            owner,
            other: None,
            distinct: false,
            owner_must_be_alive: true,
        }
    }

    /// Only the skill owner participates, without an aliveness check.
    ///
    /// For skills whose original guard was `originating_skill_is_enabled` alone — folding
    /// them onto [`Participants::owner`] would silently add a precondition.
    pub(crate) fn owner_any_state(owner: PlayerId) -> Self {
        Self {
            owner,
            other: None,
            distinct: false,
            owner_must_be_alive: false,
        }
    }

    /// The owner plus another seat, both alive; the two may be the same seat.
    pub(crate) fn pair(owner: PlayerId, other: PlayerId) -> Self {
        Self {
            owner,
            other: Some(other),
            distinct: false,
            owner_must_be_alive: true,
        }
    }

    /// Another seat must be alive and distinct from the owner, but the owner's own state
    /// is not checked — the pattern for skills that fire from a dying or dead owner.
    pub(crate) fn distinct_other_only(owner: PlayerId, other: PlayerId) -> Self {
        Self {
            owner,
            other: Some(other),
            distinct: true,
            owner_must_be_alive: false,
        }
    }

    /// The owner plus another seat, both alive and required to be different seats.
    pub(crate) fn distinct_pair(owner: PlayerId, other: PlayerId) -> Self {
        Self {
            owner,
            other: Some(other),
            distinct: true,
            owner_must_be_alive: true,
        }
    }

    fn satisfied(&self, tx: &Transaction) -> bool {
        let alive = |p: PlayerId| tx.state().players.get(p).is_some_and(|seat| seat.alive);
        if self.owner_must_be_alive && !alive(self.owner) {
            return false;
        }
        match self.other {
            None => true,
            Some(other) => alive(other) && !(self.distinct && other == self.owner),
        }
    }
}

/// The turn context a guarded choice requires.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum TurnContext {
    /// No turn or phase requirement.
    #[default]
    Any,
    /// The owner must be the current turn player.
    OwnersTurn,
    /// The owner must be the current turn player *and* in the given phase.
    OwnersPhase(Phase),
}

impl TurnContext {
    fn satisfied(&self, tx: &Transaction, owner: PlayerId) -> bool {
        match self {
            Self::Any => true,
            Self::OwnersTurn => tx.state().turn.current == owner,
            Self::OwnersPhase(phase) => {
                tx.state().turn.current == owner && tx.state().turn.phase == *phase
            }
        }
    }
}

/// How many options the chooser must pick, and whether they may decline.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ChoiceShape {
    /// Minimum number of options to select.
    pub(crate) min: u8,
    /// Maximum number of options to select.
    pub(crate) max: u8,
    /// Whether the chooser may cancel out of the choice.
    pub(crate) optional: bool,
}

impl ChoiceShape {
    /// Select exactly one option, no cancelling — by far the most common shape.
    pub(crate) fn one() -> Self {
        Self {
            min: 1,
            max: 1,
            optional: false,
        }
    }

    /// Select exactly one option, cancellable.
    pub(crate) fn one_optional() -> Self {
        Self {
            min: 1,
            max: 1,
            optional: true,
        }
    }

    /// Select exactly `n` options, no cancelling.
    pub(crate) fn exactly(n: u8) -> Self {
        Self {
            min: n,
            max: n,
            optional: false,
        }
    }

    /// Select between `min` and `max` options.
    pub(crate) fn range(min: u8, max: u8, optional: bool) -> Self {
        Self { min, max, optional }
    }
}

/// A guarded choice: the guard, the chooser, the choice shape, and the continuation.
///
/// Build one with [`GuardedChoice::new`], adjust it with the builder methods, then hand
/// the candidate set to [`GuardedChoice::open_cards`] or
/// [`GuardedChoice::open_targets`].
pub(crate) struct GuardedChoice {
    skill: engine_api::SkillId,
    participants: Participants,
    turn: TurnContext,
    chooser: Option<PlayerId>,
    shape: ChoiceShape,
    cont: ContinuationKind,
    payload: Vec<u32>,
}

impl GuardedChoice {
    /// Start a guarded choice for `skill`, owned by `participants.owner`.
    ///
    /// By default the owner is also the chooser, no turn context is required, and the
    /// chooser picks exactly one option without the option to cancel.
    pub(crate) fn new(
        skill: engine_api::SkillId,
        participants: Participants,
        cont: ContinuationKind,
        payload: Vec<u32>,
    ) -> Self {
        Self {
            skill,
            participants,
            turn: TurnContext::Any,
            chooser: None,
            shape: ChoiceShape::one(),
            cont,
            payload,
        }
    }

    /// Require a turn or phase context.
    pub(crate) fn during(mut self, turn: TurnContext) -> Self {
        self.turn = turn;
        self
    }

    /// Let someone other than the owner make the choice.
    pub(crate) fn chosen_by(mut self, chooser: PlayerId) -> Self {
        self.chooser = Some(chooser);
        self
    }

    /// Override the selection count / cancellability.
    pub(crate) fn shape(mut self, shape: ChoiceShape) -> Self {
        self.shape = shape;
        self
    }

    /// Whether the guard passes. Callers that need to compute an expensive candidate set
    /// can check this first, though [`open_cards`](Self::open_cards) and
    /// [`open_targets`](Self::open_targets) check it themselves.
    pub(crate) fn guard_passes(&self, tx: &Transaction) -> bool {
        originating_skill_is_enabled(tx, self.participants.owner, self.skill)
            && self.participants.satisfied(tx)
            && self.turn.satisfied(tx, self.participants.owner)
    }

    /// Open a card choice over `cards`.
    ///
    /// Returns no events when the guard fails or `cards` is empty — the same silent
    /// no-op the hand-written openers produced.
    pub(crate) fn open_cards(
        self,
        tx: &mut Transaction,
        cards: Vec<CardId>,
    ) -> Result<Vec<DomainEvent>, EffectError> {
        if !self.guard_passes(tx) || cards.len() < usize::from(self.shape.min.max(1)) {
            return Ok(Vec::new());
        }
        let tokens = cards.into_iter().map(CardId::raw).collect();
        self.open(tx, tokens, ChoiceKind::SelectCards)
    }

    /// Open a target choice over `targets`.
    ///
    /// Returns no events when the guard fails or `targets` is empty.
    pub(crate) fn open_targets(
        self,
        tx: &mut Transaction,
        targets: Vec<PlayerId>,
    ) -> Result<Vec<DomainEvent>, EffectError> {
        if !self.guard_passes(tx) || targets.len() < usize::from(self.shape.min.max(1)) {
            return Ok(Vec::new());
        }
        let tokens = targets.into_iter().map(PlayerId::raw).collect();
        self.open(tx, tokens, ChoiceKind::SelectTargets)
    }

    /// Open a choice whose options are already encoded as raw tokens.
    ///
    /// Used by `ChoiceKind::Custom` openers, where the tokens are option indices rather
    /// than card or player ids.
    pub(crate) fn open_tokens(
        self,
        tx: &mut Transaction,
        tokens: Vec<u32>,
        kind: ChoiceKind,
    ) -> Result<Vec<DomainEvent>, EffectError> {
        if !self.guard_passes(tx) || tokens.len() < usize::from(self.shape.min.max(1)) {
            return Ok(Vec::new());
        }
        self.open(tx, tokens, kind)
    }

    fn open(
        self,
        tx: &mut Transaction,
        tokens: Vec<u32>,
        kind: ChoiceKind,
    ) -> Result<Vec<DomainEvent>, EffectError> {
        let observer = self.chooser.unwrap_or(self.participants.owner);
        let max = self.shape.max.max(self.shape.min);
        open_choice(
            tx,
            observer,
            self.shape.min,
            max,
            self.shape.optional,
            tokens,
            kind,
            self.cont,
            self.payload,
        )
    }
}

/// The error a `resolve_*` function returns when it is re-entered at a stage that only
/// its choice continuation should reach.
///
/// The message names the skill so content faults stay diagnosable.
pub(crate) fn stage_must_resume_via_continuation(skill: &str) -> EffectError {
    EffectError::ContentFault(format!(
        "{skill} stage must resolve through its choice continuation"
    ))
}

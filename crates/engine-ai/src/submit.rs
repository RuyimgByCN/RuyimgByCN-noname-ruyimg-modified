//! Apply AI decisions only when authority revision still matches.

use crate::fork::StateFork;
use crate::search::{SearchBudget, SearchPolicy, SearchResult};
use engine_api::{Command, CommandResult, PlayerId};
use engine_core::Engine;
use thiserror::Error;

/// AI decision package for hosts (server thread pool / browser worker).
#[derive(Debug, Clone)]
pub struct AiDecision {
    /// Actor.
    pub actor: PlayerId,
    /// Base authority revision.
    pub base_revision: u64,
    /// Command to submit (expected_revision set to base).
    pub command: Command,
    /// Nodes used in search.
    pub nodes_used: u64,
}

/// AI errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AiError {
    /// No legal moves.
    #[error("no legal action")]
    NoAction,
    /// Authority moved — discard result.
    #[error("stale revision: base={base} current={current}")]
    Stale {
        /// Base.
        base: u64,
        /// Current.
        current: u64,
    },
    /// Command rejected by engine.
    #[error("command rejected")]
    Rejected,
}

/// Runtime that owns a search policy.
pub struct AiRuntime {
    /// Search seed base.
    pub search_seed: u64,
}

impl Default for AiRuntime {
    fn default() -> Self {
        Self {
            search_seed: 0xA11CE,
        }
    }
}

impl AiRuntime {
    /// Create.
    pub fn new(search_seed: u64) -> Self {
        Self { search_seed }
    }

    /// Run search on a fork; does not mutate authority.
    pub fn decide(
        &self,
        authority: &Engine,
        actor: PlayerId,
        policy: &mut dyn SearchPolicy,
        budget: SearchBudget,
    ) -> Result<AiDecision, AiError> {
        let seed = self
            .search_seed
            .wrapping_add(authority.state().revision)
            .wrapping_add(actor.raw() as u64);
        let mut fork = StateFork::from_authority(authority, seed);
        // Isolation: remember authority hash
        let auth_hash = authority.state_hash();
        let result: SearchResult = policy.select(&mut fork, actor, budget);
        // Authority must be untouched
        debug_assert_eq!(authority.state_hash(), auth_hash);

        let command = result.command.ok_or(AiError::NoAction)?;
        Ok(AiDecision {
            actor,
            base_revision: result.base_revision,
            command,
            nodes_used: result.nodes_used,
        })
    }
}

/// Submit decision only if authority revision still equals base.
pub fn apply_if_fresh(
    authority: &mut Engine,
    decision: &AiDecision,
) -> Result<CommandResult, AiError> {
    let current = authority.state().revision;
    if current != decision.base_revision {
        return Err(AiError::Stale {
            base: decision.base_revision,
            current,
        });
    }
    let mut cmd = decision.command.clone();
    cmd.expected_revision = engine_api::ExpectedRevision(decision.base_revision);
    // Allow 0 for headless flexibility: re-bind
    if cmd.expected_revision.0 == 0 {
        cmd.expected_revision = engine_api::ExpectedRevision(current);
    }
    let r = authority.submit(cmd);
    match &r {
        CommandResult::Rejected { .. } => Err(AiError::Rejected),
        CommandResult::Accepted { .. } => Ok(r),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::HeuristicSearch;
    use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey};

    fn started() -> Engine {
        let mut eng = Engine::new();
        eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 21,
                ruleset: "identity-standard".into(),
            },
        });
        eng
    }

    #[test]
    fn stale_decision_discarded() {
        let mut eng = started();
        let actor = eng.state().turn.current;
        let rt = AiRuntime::new(1);
        let mut policy = HeuristicSearch;
        let decision = rt
            .decide(&eng, actor, &mut policy, SearchBudget::default())
            .unwrap();

        // Advance authority
        eng.submit(Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });

        let err = apply_if_fresh(&mut eng, &decision).unwrap_err();
        assert!(matches!(err, AiError::Stale { .. }));
    }

    #[test]
    fn stale_decision_never_commits() {
        // M6 gate 2: 过期搜索结果不提交——权威推进后，基于旧 base_revision 的决策
        // 必须被丢弃，且不得改变权威状态。
        let mut eng = started();
        let actor = eng.state().turn.current;
        let rt = AiRuntime::new(4);
        let mut policy = HeuristicSearch;
        let decision = rt
            .decide(&eng, actor, &mut policy, SearchBudget::default())
            .unwrap();
        let rev_after_search = eng.state().revision;

        eng.submit(Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        let rev_after_advance = eng.state().revision;
        assert_ne!(rev_after_search, rev_after_advance);

        let err = apply_if_fresh(&mut eng, &decision).unwrap_err();
        assert!(matches!(
            err,
            AiError::Stale {
                base,
                current
            } if base == rev_after_search && current == rev_after_advance
        ));
        assert_eq!(
            eng.state().revision,
            rev_after_advance,
            "stale decision must not change authority"
        );
    }

    #[test]
    fn non_current_actor_decision_is_rejected_without_state_change() {
        // M6 gate 2: 无越权——非当前回合角色（或未获授权的观察者）提交 AI 决策必须被拒绝，
        // 且权威状态不变。
        let mut eng = started();
        let current = eng.state().turn.current;
        let other = eng
            .state()
            .players
            .iter_live()
            .find(|player| player.id != current)
            .unwrap()
            .id;
        let rev_before = eng.state().revision;
        let forged = AiDecision {
            actor: other,
            base_revision: rev_before,
            command: Command {
                actor: other,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(99),
                kind: CommandKind::EndPhase,
            },
            nodes_used: 0,
        };
        let err = apply_if_fresh(&mut eng, &forged).unwrap_err();
        assert!(matches!(err, AiError::Rejected));
        assert_eq!(
            eng.state().revision,
            rev_before,
            "overreach must not change authority"
        );
        assert_eq!(eng.state().turn.current, current, "turn actor unchanged");
    }

    #[test]
    fn search_does_not_mutate_authority() {
        let eng = started();
        let h0 = eng.state_hash();
        let rev = eng.state().revision;
        let rt = AiRuntime::new(2);
        let mut policy = HeuristicSearch;
        let _ = rt.decide(
            &eng,
            eng.state().turn.current,
            &mut policy,
            SearchBudget {
                max_nodes: 32,
                max_depth: 4,
            },
        );
        assert_eq!(eng.state_hash(), h0);
        assert_eq!(eng.state().revision, rev);
    }
}

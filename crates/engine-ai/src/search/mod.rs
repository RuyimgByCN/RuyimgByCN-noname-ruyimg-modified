//! Search policies: heuristic, beam, MCTS (fixed node budgets — no wall clock).

mod beam;
mod heuristic;
mod mcts;

pub use beam::BeamSearch;
pub use heuristic::HeuristicSearch;
pub use mcts::MctsSearch;

use crate::fork::StateFork;
use crate::legal::legal_actions;
use engine_api::LegalAction;
use engine_api::{Command, PlayerId};
use serde::{Deserialize, Serialize};

/// Fixed search budget (architecture: no wall-clock stop for deterministic AI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchBudget {
    /// Max fork apply / expand operations.
    pub max_nodes: u64,
    /// Max playout depth for MCTS/beam.
    pub max_depth: u32,
}

impl Default for SearchBudget {
    fn default() -> Self {
        Self {
            max_nodes: 64,
            max_depth: 8,
        }
    }
}

/// Result of a search call.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Chosen action (if any).
    pub action: Option<LegalAction>,
    /// Command bound to fork base revision.
    pub command: Option<Command>,
    /// Nodes used.
    pub nodes_used: u64,
    /// Base revision.
    pub base_revision: u64,
}

/// Pluggable search policy.
pub trait SearchPolicy: Send {
    /// Select an action for `actor` on the given fork within budget.
    fn select(
        &mut self,
        fork: &mut StateFork,
        actor: PlayerId,
        budget: SearchBudget,
    ) -> SearchResult;
}

/// Shared helper: score a position for `actor` (higher better).
pub(crate) fn evaluate(fork: &StateFork, actor: PlayerId) -> i32 {
    use engine_core::state::Identity;
    let st = fork.engine().state();
    if st.outcome.is_finished() {
        return terminal_score(st, actor);
    }
    let Some(me) = st.players.get(actor) else {
        return -10_000;
    };
    let mut score = me.hp * 100;
    score += st.hand_count(actor) as i32 * 10;
    // Prefer enemies low HP
    for p in st.players.iter_live() {
        if !p.alive || p.id == actor {
            continue;
        }
        let enemy = match me.identity {
            Identity::Lord | Identity::Loyalist => {
                matches!(p.identity, Identity::Rebel | Identity::Renegade)
            }
            Identity::Rebel => matches!(p.identity, Identity::Lord | Identity::Loyalist),
            Identity::Renegade => true,
        };
        if enemy {
            score += (4 - p.hp.max(0)) * 30;
        } else {
            score += p.hp * 5;
        }
    }
    score
}

fn terminal_score(st: &engine_core::GameState, actor: PlayerId) -> i32 {
    use engine_api::MatchOutcome;
    use engine_core::state::Identity;
    let Some(me) = st.players.get(actor) else {
        return -10_000;
    };
    match &st.outcome {
        MatchOutcome::LoyalistWin => match me.identity {
            Identity::Lord | Identity::Loyalist => 10_000,
            _ => -10_000,
        },
        MatchOutcome::RebelWin => match me.identity {
            Identity::Rebel => 10_000,
            _ => -10_000,
        },
        MatchOutcome::RenegadeWin { player } => {
            if *player == actor {
                10_000
            } else {
                -10_000
            }
        }
        MatchOutcome::Draw | MatchOutcome::Aborted { .. } => 0,
        // Guozhan faction victory (ADR 0598 §D4): identity-mode AI never sees this;
        // return neutral until guozhan AI scoring lands in phase B.
        MatchOutcome::GroupWin { .. } => 0,
        // M4 单挑：AI 评分与单挑模式联动待 phase B；先返回中性。
        MatchOutcome::SingleWin { player } => {
            if *player == me.id {
                1
            } else {
                -1
            }
        }
        MatchOutcome::TeamWin { .. } => 0,
        MatchOutcome::InProgress => 0,
    }
}

/// Convert action to command with fork base revision.
pub(crate) fn action_to_cmd(action: &LegalAction, actor: PlayerId, base_rev: u64) -> Command {
    action.to_command(actor, base_rev)
}

/// List actions without consuming budget.
pub(crate) fn actions(fork: &StateFork, actor: PlayerId) -> Vec<LegalAction> {
    legal_actions(fork.engine(), actor)
}

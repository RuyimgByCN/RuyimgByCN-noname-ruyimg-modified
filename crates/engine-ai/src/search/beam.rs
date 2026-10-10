//! Fixed-width beam search with node budget.

use super::{action_to_cmd, actions, evaluate, SearchBudget, SearchPolicy, SearchResult};
use crate::fork::StateFork;
use engine_api::LegalAction;
use engine_api::PlayerId;

/// Beam search.
#[derive(Debug, Clone)]
pub struct BeamSearch {
    /// Beam width.
    pub width: usize,
}

impl Default for BeamSearch {
    fn default() -> Self {
        Self { width: 4 }
    }
}

impl SearchPolicy for BeamSearch {
    fn select(
        &mut self,
        fork: &mut StateFork,
        actor: PlayerId,
        budget: SearchBudget,
    ) -> SearchResult {
        let base_revision = fork.base_revision;
        let root_actions = actions(fork, actor);
        if root_actions.is_empty() {
            return SearchResult {
                action: None,
                command: None,
                nodes_used: 0,
                base_revision,
            };
        }

        // beam entries: (score, nodes_path, first_action, state)
        let mut beam: Vec<(i32, LegalAction, StateFork)> = Vec::new();
        let mut nodes = 0u64;

        for act in &root_actions {
            if nodes >= budget.max_nodes {
                break;
            }
            let mut child = fork.clone();
            let cmd = action_to_cmd(act, actor, base_revision);
            let _ = child.apply(cmd);
            nodes = nodes.saturating_add(1);
            let score = evaluate(&child, actor);
            beam.push((score, act.clone(), child));
        }
        beam.sort_by(|a, b| b.0.cmp(&a.0));
        beam.truncate(self.width.max(1));

        for _depth in 1..budget.max_depth {
            if nodes >= budget.max_nodes || beam.is_empty() {
                break;
            }
            let mut next = Vec::new();
            for (_, first, state) in &beam {
                let acts = actions(state, actor);
                if acts.is_empty() {
                    next.push((evaluate(state, actor), first.clone(), state.clone()));
                    continue;
                }
                for act in acts {
                    if nodes >= budget.max_nodes {
                        break;
                    }
                    let mut child = state.clone();
                    let cmd = action_to_cmd(&act, actor, child.base_revision);
                    // Use 0 expected for deep playouts on forked revisions
                    let mut cmd = cmd;
                    cmd.expected_revision = engine_api::ExpectedRevision(0);
                    let _ = child.apply(cmd);
                    nodes = nodes.saturating_add(1);
                    let score = evaluate(&child, actor);
                    next.push((score, first.clone(), child));
                }
            }
            if next.is_empty() {
                break;
            }
            next.sort_by(|a, b| b.0.cmp(&a.0));
            next.truncate(self.width.max(1));
            beam = next;
        }

        let best = beam
            .into_iter()
            .max_by_key(|(s, _, _)| *s)
            .map(|(_, a, _)| a);
        let command = best
            .as_ref()
            .map(|a| action_to_cmd(a, actor, base_revision));
        SearchResult {
            action: best,
            command,
            nodes_used: nodes,
            base_revision,
        }
    }
}

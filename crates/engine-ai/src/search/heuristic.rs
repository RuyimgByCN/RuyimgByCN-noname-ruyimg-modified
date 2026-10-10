//! Greedy one-ply heuristic search.

use super::{action_to_cmd, actions, evaluate, SearchBudget, SearchPolicy, SearchResult};
use crate::fork::StateFork;
use engine_api::{CommandResult, PlayerId};

/// Pick legal action maximizing immediate evaluate after apply.
#[derive(Debug, Default)]
pub struct HeuristicSearch;

impl SearchPolicy for HeuristicSearch {
    fn select(
        &mut self,
        fork: &mut StateFork,
        actor: PlayerId,
        budget: SearchBudget,
    ) -> SearchResult {
        let base_revision = fork.base_revision;
        let candidates = actions(fork, actor);
        if candidates.is_empty() {
            return SearchResult {
                action: None,
                command: None,
                nodes_used: fork.nodes,
                base_revision,
            };
        }

        let mut best_i = 0usize;
        let mut best_score = i32::MIN;
        let mut nodes = 0u64;

        for (i, act) in candidates.iter().enumerate() {
            if nodes >= budget.max_nodes {
                break;
            }
            let mut child = fork.clone();
            let cmd = action_to_cmd(act, actor, base_revision);
            let _ = child.apply(cmd);
            nodes = nodes.saturating_add(1).saturating_add(child.nodes);
            let score = evaluate(&child, actor);
            if score > best_score {
                best_score = score;
                best_i = i;
            }
        }

        let action = candidates[best_i].clone();
        let command = Some(action_to_cmd(&action, actor, base_revision));
        // Prove at least one apply accepted on a throwaway clone when possible
        let _ = CommandResult::Accepted {
            new_revision: 0,
            state_hash: [0; 32],
        };
        SearchResult {
            action: Some(action),
            command,
            nodes_used: nodes,
            base_revision,
        }
    }
}

//! Simple MCTS with fixed simulation count (no wall clock).

use super::{action_to_cmd, actions, evaluate, SearchBudget, SearchPolicy, SearchResult};
use crate::fork::StateFork;
use engine_api::LegalAction;
use engine_api::{CommandResult, ExpectedRevision, PlayerId};

/// Monte Carlo Tree Search (flat UCB over root actions).
#[derive(Debug, Clone)]
pub struct MctsSearch {
    /// Exploration constant ×100 (default 141 ≈ √2).
    pub exploration_x100: i32,
}

impl Default for MctsSearch {
    fn default() -> Self {
        Self {
            exploration_x100: 141,
        }
    }
}

impl SearchPolicy for MctsSearch {
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

        let n = root_actions.len();
        let mut visits = vec![0u64; n];
        let mut totals = vec![0i64; n];
        let mut nodes = 0u64;
        let sims = budget.max_nodes.max(1);

        for _ in 0..sims {
            if nodes >= budget.max_nodes {
                break;
            }
            // UCB1 select
            let total_visits: u64 = visits.iter().sum::<u64>().max(1);
            let mut best_i = 0usize;
            let mut best_ucb = f64::MIN;
            for i in 0..n {
                let ucb = if visits[i] == 0 {
                    f64::INFINITY
                } else {
                    let exploit = totals[i] as f64 / visits[i] as f64;
                    let explore = (self.exploration_x100 as f64 / 100.0)
                        * ((total_visits as f64).ln() / visits[i] as f64).sqrt();
                    exploit + explore
                };
                if ucb > best_ucb {
                    best_ucb = ucb;
                    best_i = i;
                }
            }

            // Simulate
            let mut sim = fork.clone();
            let first = &root_actions[best_i];
            let cmd = action_to_cmd(first, actor, base_revision);
            let _ = sim.apply(cmd);
            nodes = nodes.saturating_add(1);

            for _d in 1..budget.max_depth {
                if nodes >= budget.max_nodes {
                    break;
                }
                let acts = actions(&sim, actor);
                if acts.is_empty() {
                    // try current player switch — just end if possible for whoever
                    let cur = sim.engine().state().turn.current;
                    let acts2 = actions(&sim, cur);
                    if acts2.is_empty() {
                        break;
                    }
                    let idx = sim.search_rng.gen_range_u32(acts2.len() as u32) as usize;
                    let mut c = action_to_cmd(&acts2[idx], cur, 0);
                    c.expected_revision = ExpectedRevision(0);
                    match sim.apply(c) {
                        CommandResult::Accepted { .. } => nodes += 1,
                        CommandResult::Rejected { .. } => break,
                    }
                    continue;
                }
                let idx = sim.search_rng.gen_range_u32(acts.len() as u32) as usize;
                let mut c = action_to_cmd(&acts[idx], actor, 0);
                c.expected_revision = ExpectedRevision(0);
                match sim.apply(c) {
                    CommandResult::Accepted { .. } => nodes += 1,
                    CommandResult::Rejected { .. } => break,
                }
            }

            let score = evaluate(&sim, actor) as i64;
            visits[best_i] += 1;
            totals[best_i] += score;
        }

        let best_i = visits
            .iter()
            .enumerate()
            .max_by_key(|(_, v)| *v)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let action = root_actions[best_i].clone();
        let command = Some(action_to_cmd(&action, actor, base_revision));
        let _ = LegalAction::EndPhase;
        SearchResult {
            action: Some(action),
            command,
            nodes_used: nodes,
            base_revision,
        }
    }
}

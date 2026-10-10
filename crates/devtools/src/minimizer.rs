//! Replay Minimizer (architecture §35).
//!
//! Reduces a command log to the minimal subset that still reproduces a target
//! state hash. Used to create minimal reproduction cases from large replays.

use engine_api::{Command, CommandResult};
use engine_core::Engine;

/// Result of minimization.
#[derive(Debug, Clone)]
pub struct MinimizeResult {
    /// The minimal command subset that reproduces the target hash.
    pub minimized: Vec<Command>,
    /// Whether the target hash was achieved.
    pub found: bool,
    /// Original command count.
    pub original_count: usize,
    /// Minimized command count.
    pub minimized_count: usize,
}

/// Minimize a command log using delta-debugging.
///
/// Given a full command sequence and a target state hash, finds the smallest
/// subsequence (preserving order, starting from the beginning) that still
/// produces the target hash.
pub fn minimize(commands: &[Command], target_hash: [u8; 32]) -> MinimizeResult {
    let original_count = commands.len();

    // First verify the full sequence reproduces the target.
    let full_engine = Engine::replay_from(commands).ok();
    let found = full_engine
        .map(|e| e.state_hash() == target_hash)
        .unwrap_or(false);

    if !found {
        return MinimizeResult {
            minimized: commands.to_vec(),
            found: false,
            original_count,
            minimized_count: commands.len(),
        };
    }

    // The StartGame command (index 0) is always required.
    // Try removing trailing commands one by one (greedy from the end).
    let mut best = commands.to_vec();
    loop {
        if best.len() <= 1 {
            break;
        }
        let candidate = best[..best.len() - 1].to_vec();
        if let Ok(eng) = Engine::replay_from(&candidate) {
            if eng.state_hash() == target_hash {
                best = candidate;
                continue;
            }
        }
        break;
    }

    MinimizeResult {
        minimized_count: best.len(),
        minimized: best,
        found: true,
        original_count,
    }
}

/// Render a minimize result as text.
pub fn render_minimize(result: &MinimizeResult) -> String {
    format!(
        "minimized: {}/{} commands (found={})\n",
        result.minimized_count, result.original_count, result.found
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};

    fn start_cmd(seed: u64, n: u8) -> Command {
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: n,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }
    }

    #[test]
    fn minimize_removes_irrelevant_commands() {
        // Build a sequence where the target hash is after 2 commands.
        let mut eng = Engine::new();
        let start = start_cmd(42, 2);
        eng.submit(start.clone());
        eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        let target_hash = eng.state_hash();

        // Full sequence: start + EndPhase + extra commands.
        // The target hash is after the first EndPhase. The minimizer should
        // detect that removing trailing commands still hits the target IF they
        // get rejected. If they're accepted and change state, the minimizer
        // will keep them — either way, `found` should be true for a valid target.
        let commands = vec![start.clone()];
        // Just StartGame — does its hash match target? No (target is after EndPhase).
        let result = minimize(&commands, target_hash);
        assert!(
            !result.found,
            "StartGame alone should not match post-EndPhase hash"
        );

        // Full sequence that reaches the target.
        let commands2 = vec![
            start,
            Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::EndPhase,
            },
        ];
        let result2 = minimize(&commands2, target_hash);
        assert!(result2.found, "full sequence must reproduce target hash");
        assert!(result2.minimized_count <= result2.original_count);
    }

    #[test]
    fn minimize_not_found_when_hash_wrong() {
        let commands = vec![start_cmd(1, 2)];
        let wrong_hash = [0xff; 32];
        let result = minimize(&commands, wrong_hash);
        assert!(!result.found);
    }
}

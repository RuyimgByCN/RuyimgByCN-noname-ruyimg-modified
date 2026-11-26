//! Replay Minimizer (architecture §35).
//!
//! Reduces a command log to the minimal prefix that still reproduces a target
//! canonical state hash on `engine_native::Engine`.

use engine_native::{Engine, GameCommand};

/// Result of minimization.
#[derive(Debug, Clone)]
pub struct MinimizeResult {
    /// The minimal command prefix that reproduces the target hash.
    pub minimized: Vec<GameCommand>,
    /// Whether the target hash was achieved.
    pub found: bool,
    /// Original command count.
    pub original_count: usize,
    /// Minimized command count.
    pub minimized_count: usize,
}

fn replay(seed: u64, seats: usize, commands: &[GameCommand]) -> Engine {
    let mut eng = Engine::new(seed, seats);
    for cmd in commands {
        let _ = eng.submit(cmd.clone());
    }
    eng
}

/// Minimize a command sequence from `(seed, seats)` using trailing prefix pruning.
pub fn minimize(
    seed: u64,
    seats: usize,
    commands: &[GameCommand],
    target_hash: [u8; 32],
) -> MinimizeResult {
    let original_count = commands.len();
    let full = replay(seed, seats, commands);
    if full.state_hash() != target_hash {
        return MinimizeResult {
            minimized: commands.to_vec(),
            found: false,
            original_count,
            minimized_count: original_count,
        };
    }

    let mut best = commands.to_vec();
    while !best.is_empty() {
        let candidate = best[..best.len() - 1].to_vec();
        if replay(seed, seats, &candidate).state_hash() == target_hash {
            best = candidate;
        } else {
            break;
        }
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
    use engine_native::GameAction;

    #[test]
    fn minimize_removes_irrelevant_commands() {
        let mut eng = Engine::new(42, 2);
        let cmd1 = GameCommand {
            actor: eng.turn().current,
            expected_revision: eng.revision(),
            dedup_id: 1,
            action: GameAction::NextPhase,
        };
        eng.submit(cmd1.clone());
        let target_hash = eng.state_hash();

        let bad_cmd = GameCommand {
            actor: 99,
            expected_revision: eng.revision(),
            dedup_id: 2,
            action: GameAction::NextPhase,
        };

        let result_empty = minimize(42, 2, &[], target_hash);
        assert!(
            !result_empty.found,
            "initial state alone should not match post-NextPhase hash"
        );

        let result2 = minimize(42, 2, &[cmd1, bad_cmd], target_hash);
        assert!(result2.found, "sequence with rejected trailing cmd must reproduce hash");
        assert_eq!(result2.minimized_count, 1);
    }

    #[test]
    fn minimize_not_found_when_hash_wrong() {
        let wrong_hash = [0xff; 32];
        let result = minimize(1, 2, &[], wrong_hash);
        assert!(!result.found);
    }
}

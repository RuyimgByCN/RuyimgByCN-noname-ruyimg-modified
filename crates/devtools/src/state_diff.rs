//! State Diff Viewer (architecture §35).
//!
//! Diffs two `GameState` snapshots field-by-field and reports what changed.

use engine_core::GameState;

/// One field difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiff {
    /// Field path (e.g. "revision", "turn.current", "players[1].hp").
    pub field: String,
    /// Old value.
    pub old: String,
    /// New value.
    pub new: String,
}

/// Compute the diff between two game states.
pub fn diff_states(old: &GameState, new: &GameState) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    if old.revision != new.revision {
        diffs.push(FieldDiff {
            field: "revision".into(),
            old: old.revision.to_string(),
            new: new.revision.to_string(),
        });
    }

    if old.turn.current != new.turn.current {
        diffs.push(FieldDiff {
            field: "turn.current".into(),
            old: old.turn.current.raw().to_string(),
            new: new.turn.current.raw().to_string(),
        });
    }

    if old.turn.round != new.turn.round {
        diffs.push(FieldDiff {
            field: "turn.round".into(),
            old: old.turn.round.to_string(),
            new: new.turn.round.to_string(),
        });
    }

    // Player diffs.
    let old_players: std::collections::HashMap<_, _> = old
        .players
        .sorted_live()
        .into_iter()
        .map(|p| (p.id, p))
        .collect();
    let new_players: std::collections::HashMap<_, _> = new
        .players
        .sorted_live()
        .into_iter()
        .map(|p| (p.id, p))
        .collect();

    for (pid, np) in &new_players {
        if let Some(op) = old_players.get(pid) {
            if op.hp != np.hp {
                diffs.push(FieldDiff {
                    field: format!("players[{}].hp", pid.raw()),
                    old: op.hp.to_string(),
                    new: np.hp.to_string(),
                });
            }
            if op.alive != np.alive {
                diffs.push(FieldDiff {
                    field: format!("players[{}].alive", pid.raw()),
                    old: op.alive.to_string(),
                    new: np.alive.to_string(),
                });
            }
            if op.dying != np.dying {
                diffs.push(FieldDiff {
                    field: format!("players[{}].dying", pid.raw()),
                    old: op.dying.to_string(),
                    new: np.dying.to_string(),
                });
            }
            if old.hand_count(*pid) != new.hand_count(*pid) {
                diffs.push(FieldDiff {
                    field: format!("players[{}].hand_count", pid.raw()),
                    old: old.hand_count(*pid).to_string(),
                    new: new.hand_count(*pid).to_string(),
                });
            }
        }
    }

    // Card count.
    if old.cards.live_count() != new.cards.live_count() {
        diffs.push(FieldDiff {
            field: "cards.live_count".into(),
            old: old.cards.live_count().to_string(),
            new: new.cards.live_count().to_string(),
        });
    }

    // Outcome.
    if old.outcome != new.outcome {
        diffs.push(FieldDiff {
            field: "outcome".into(),
            old: format!("{:?}", old.outcome),
            new: format!("{:?}", new.outcome),
        });
    }

    diffs
}

/// Render diffs as text.
pub fn render_diffs(diffs: &[FieldDiff]) -> String {
    if diffs.is_empty() {
        return "(no differences)\n".into();
    }
    let mut out = String::new();
    out.push_str("field                        old → new\n");
    out.push_str("---------------------------  ----------------\n");
    for d in diffs {
        out.push_str(&format!("{:<28} {} → {}\n", d.field, d.old, d.new));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
    use engine_core::Engine;

    #[test]
    fn diff_detects_revision_change() {
        let mut eng = Engine::new();
        eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 1,
                ruleset: "identity-standard".into(),
            },
        });
        let before = eng.state().clone();
        eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        let after = eng.state();
        let diffs = diff_states(&before, after);
        assert!(!diffs.is_empty());
        assert!(diffs.iter().any(|d| d.field == "revision"));
    }
}

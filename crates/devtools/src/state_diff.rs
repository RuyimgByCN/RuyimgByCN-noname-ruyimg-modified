//! State Diff Viewer (architecture §35).
//!
//! Diffs two `GameStateCore` snapshots field-by-field and reports what changed.

use engine_native::state::GameStateCore;

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

fn live_card_total(state: &GameStateCore) -> usize {
    let hand_total: usize = state.players.iter().map(|p| p.hand.len()).sum();
    let equip_total: usize = state.players.iter().map(|p| p.equips.len()).sum();
    let judge_total: usize = state.players.iter().map(|p| p.judge.len()).sum();
    state.deck.len() + state.discard.len() + state.table.len() + hand_total + equip_total + judge_total
}

/// Compute the diff between two native game states.
pub fn diff_states(old: &GameStateCore, new: &GameStateCore) -> Vec<FieldDiff> {
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
            old: old.turn.current.to_string(),
            new: new.turn.current.to_string(),
        });
    }

    if old.turn.round != new.turn.round {
        diffs.push(FieldDiff {
            field: "turn.round".into(),
            old: old.turn.round.to_string(),
            new: new.turn.round.to_string(),
        });
    }

    if old.turn.phase != new.turn.phase {
        diffs.push(FieldDiff {
            field: "turn.phase".into(),
            old: format!("{:?}", old.turn.phase),
            new: format!("{:?}", new.turn.phase),
        });
    }

    let n = old.players.len().min(new.players.len());
    for idx in 0..n {
        let op = &old.players[idx];
        let np = &new.players[idx];
        if op.hp != np.hp {
            diffs.push(FieldDiff {
                field: format!("players[{idx}].hp"),
                old: op.hp.to_string(),
                new: np.hp.to_string(),
            });
        }
        if op.alive != np.alive {
            diffs.push(FieldDiff {
                field: format!("players[{idx}].alive"),
                old: op.alive.to_string(),
                new: np.alive.to_string(),
            });
        }
        if op.dying != np.dying {
            diffs.push(FieldDiff {
                field: format!("players[{idx}].dying"),
                old: op.dying.to_string(),
                new: np.dying.to_string(),
            });
        }
        if op.hand.len() != np.hand.len() {
            diffs.push(FieldDiff {
                field: format!("players[{idx}].hand_count"),
                old: op.hand.len().to_string(),
                new: np.hand.len().to_string(),
            });
        }
    }

    let old_cards = live_card_total(old);
    let new_cards = live_card_total(new);
    if old_cards != new_cards {
        diffs.push(FieldDiff {
            field: "cards.live_count".into(),
            old: old_cards.to_string(),
            new: new_cards.to_string(),
        });
    }

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
    use engine_native::{Engine, GameAction, GameCommand};

    #[test]
    fn diff_detects_revision_change() {
        let mut eng = Engine::new(1, 2);
        let before = eng.state_core().clone();
        eng.submit(GameCommand {
            actor: eng.turn().current,
            expected_revision: eng.revision(),
            dedup_id: 1,
            action: GameAction::NextPhase,
        });
        let after = eng.state_core();
        let diffs = diff_states(&before, after);
        assert!(!diffs.is_empty());
        assert!(diffs.iter().any(|d| d.field == "revision"));
    }
}

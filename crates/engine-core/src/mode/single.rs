//! Single mode (单挑) rules.
//!
//! Classic 1v1: two players face off; the first to eliminate the other wins
//! (`MatchOutcome::SingleWin`). If both die simultaneously (e.g. mutual kill
//! resolution), the match is a draw. Identity values are ignored: both players
//! are treated as neutral for victory purposes.
//!
//! Upstream reference: `mode/single.js`. M4 milestone: headless 终局 + golden.

use crate::state::GameState;
use engine_api::{MatchOutcome, PlayerId};

/// Result of evaluating single-mode victory after a state change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SingleVictory {
    /// Still in progress.
    None,
    /// Terminal outcome.
    Finished(MatchOutcome),
}

/// Evaluate single-mode victory from the current alive players.
pub fn check_single_victory(state: &GameState) -> SingleVictory {
    if state.outcome.is_finished() {
        return SingleVictory::Finished(state.outcome.clone());
    }
    let alive: Vec<PlayerId> = state
        .players
        .iter_live()
        .filter(|player| player.alive)
        .map(|player| player.id)
        .collect();
    match alive.len() {
        0 => SingleVictory::Finished(MatchOutcome::Draw),
        1 => SingleVictory::Finished(MatchOutcome::SingleWin { player: alive[0] }),
        _ => SingleVictory::None,
    }
}

/// Apply single-mode victory into working state if finished.
pub fn apply_single_victory_if_any(state: &mut GameState) -> Option<engine_api::DomainEvent> {
    match check_single_victory(state) {
        SingleVictory::Finished(outcome) => {
            state.outcome = outcome.clone();
            Some(engine_api::DomainEvent::MatchEnded { outcome })
        }
        SingleVictory::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;
    use crate::transaction::Transaction;

    #[test]
    fn one_alive_wins() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        let loser = PlayerId::new(2);
        tx.state_mut().players.get_mut(loser).unwrap().alive = false;
        assert_eq!(
            check_single_victory(tx.state()),
            SingleVictory::Finished(MatchOutcome::SingleWin {
                player: PlayerId::new(1)
            })
        );
    }

    #[test]
    fn zero_alive_is_draw() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        for id in [PlayerId::new(1), PlayerId::new(2)] {
            tx.state_mut().players.get_mut(id).unwrap().alive = false;
        }
        assert_eq!(
            check_single_victory(tx.state()),
            SingleVictory::Finished(MatchOutcome::Draw)
        );
    }

    #[test]
    fn both_alive_is_in_progress() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        assert_eq!(check_single_victory(tx.state()), SingleVictory::None);
    }
}

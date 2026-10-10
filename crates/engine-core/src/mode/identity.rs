//! Identity mode (身份局) rules.
//!
//! Win conditions (classic):
//! - Lord dies: if only renegade(s) remain alive among non-lord camps → RenegadeWin
//!   (sole living renegade when lord dies with no other alive enemies of renegade);
//!   otherwise RebelWin.
//! - All Rebels and Renegades eliminated while Lord (or Loyalist) side stands → LoyalistWin.
//!
//! Simplified sole-renegade rule used by most digital implementations:
//! - Lord dead → if exactly one alive player and that player is Renegade → RenegadeWin
//! - Lord dead → else RebelWin
//! - No Rebel and no Renegade alive → LoyalistWin

use crate::state::{GameState, Identity};
use engine_api::{MatchOutcome, PlayerId};

/// Standard identity counts by player count (index = player_count).
/// Each list: seat-order assignment before shuffle of non-lord seats.
/// Seat 0 is always Lord.
pub const STANDARD_IDENTITY_TABLE: &[&[Identity]] = &[
    &[], // 0
    &[], // 1
    &[Identity::Lord, Identity::Renegade],
    &[Identity::Lord, Identity::Loyalist, Identity::Rebel],
    &[
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Renegade,
    ],
    &[
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Renegade,
    ],
    &[
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Renegade,
    ],
    &[
        Identity::Lord,
        Identity::Loyalist,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Renegade,
    ],
    &[
        Identity::Lord,
        Identity::Loyalist,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Rebel,
        Identity::Renegade,
    ],
];

/// Build identity list for `n` players (2..=8). Lord fixed at index 0; others as table.
pub fn distribute_identities(n: u8) -> Vec<Identity> {
    let n = n as usize;
    if n < 2 || n > 8 {
        return vec![Identity::Lord; n.max(1)];
    }
    STANDARD_IDENTITY_TABLE[n].to_vec()
}

/// Result of evaluating victory after a state change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityVictory {
    /// Still in progress.
    None,
    /// Terminal outcome.
    Finished(MatchOutcome),
}

/// Evaluate identity-mode victory from current alive players.
pub fn check_identity_victory(state: &GameState) -> IdentityVictory {
    if state.outcome.is_finished() {
        return IdentityVictory::Finished(state.outcome.clone());
    }

    let alive: Vec<_> = state.players.iter_live().filter(|p| p.alive).collect();

    if alive.is_empty() {
        return IdentityVictory::Finished(MatchOutcome::Draw);
    }

    let lord_alive = alive.iter().any(|p| p.identity == Identity::Lord);
    let rebel_alive = alive.iter().any(|p| p.identity == Identity::Rebel);
    let renegade_alive: Vec<PlayerId> = alive
        .iter()
        .filter(|p| p.identity == Identity::Renegade)
        .map(|p| p.id)
        .collect();
    let loyalist_alive = alive.iter().any(|p| p.identity == Identity::Loyalist);

    if !lord_alive {
        // Lord is dead
        if alive.len() == 1 && renegade_alive.len() == 1 {
            return IdentityVictory::Finished(MatchOutcome::RenegadeWin {
                player: renegade_alive[0],
            });
        }
        // Multiple alive after lord death, or rebels still up → rebel win
        // (classic: 反贼胜利 unless sole 内奸)
        return IdentityVictory::Finished(MatchOutcome::RebelWin);
    }

    // Lord alive: if no rebel and no renegade left → lord/loyalist win
    if !rebel_alive && renegade_alive.is_empty() {
        let _ = loyalist_alive;
        return IdentityVictory::Finished(MatchOutcome::LoyalistWin);
    }

    IdentityVictory::None
}

/// Apply victory into working state if finished; returns domain event if any.
pub fn apply_victory_if_any(state: &mut GameState) -> Option<engine_api::DomainEvent> {
    match check_identity_victory(state) {
        IdentityVictory::Finished(outcome) => {
            state.outcome = outcome.clone();
            Some(engine_api::DomainEvent::MatchEnded { outcome })
        }
        IdentityVictory::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;
    use crate::transaction::Transaction;

    #[test]
    fn table_lengths() {
        for n in 2..=8 {
            assert_eq!(distribute_identities(n).len(), n as usize);
            assert_eq!(distribute_identities(n)[0], Identity::Lord);
        }
    }

    #[test]
    fn loyalist_win_when_enemies_dead() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        // Kill renegade
        let p2 = PlayerId::new(2);
        tx.state_mut().players.get_mut(p2).unwrap().alive = false;
        assert_eq!(
            check_identity_victory(tx.state()),
            IdentityVictory::Finished(MatchOutcome::LoyalistWin)
        );
    }

    #[test]
    fn rebel_win_when_lord_dies_with_others() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 4, 1, "bootstrap").unwrap();
        let lord = PlayerId::new(1);
        tx.state_mut().players.get_mut(lord).unwrap().alive = false;
        // rebels/others still alive
        assert_eq!(
            check_identity_victory(tx.state()),
            IdentityVictory::Finished(MatchOutcome::RebelWin)
        );
    }
}

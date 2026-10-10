//! Match outcome.

use crate::ids::PlayerId;
use serde::{Deserialize, Serialize};

/// Terminal or in-progress match outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MatchOutcome {
    /// Still playing.
    #[default]
    InProgress,
    /// Identity mode: rebel victory.
    RebelWin,
    /// Identity mode: loyalist / lord victory.
    LoyalistWin,
    /// Identity mode: renegade sole survivor.
    RenegadeWin {
        /// Winning renegade.
        player: PlayerId,
    },
    /// Abort / admin / fault termination.
    Aborted {
        /// Reason code.
        reason_code: u32,
    },
    /// Draw / no winner.
    Draw,
    /// Guozhan mode: a faction (势力) wins by eliminating all other factions.
    /// `group` is the winning faction key (e.g. `"wei"`, `"shu"`, `"wu"`, `"qun"`,
    /// `"jin"`, or `"ye"` for 野心家). See ADR 0598 §D4.
    GroupWin {
        /// Winning faction key.
        group: String,
    },
    /// Single mode (单挑): the last living player wins.
    SingleWin {
        /// Sole surviving player.
        player: PlayerId,
    },
}

impl MatchOutcome {
    /// Whether the match has ended.
    pub fn is_finished(&self) -> bool {
        !matches!(self, MatchOutcome::InProgress)
    }
}

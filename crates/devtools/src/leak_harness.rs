//! Observer Leak Test Harness (architecture §35 / §37.9).
//!
//! Systematically scans observer views for hidden information leaks.
//! Verifies that no unauthorized card faces, identities, or private data
//! are visible to the wrong observer.

use engine_api::{ObserverKind, ObserverSnapshot, PlayerId};
use engine_core::GameState;

/// One detected leak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leak {
    /// Type of leaked info.
    pub kind: LeakKind,
    /// Description.
    pub description: String,
}

/// Kind of information leak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeakKind {
    /// Foreign hand card face is visible.
    ForeignHandFace,
    /// Private identity revealed to unauthorized observer.
    PrivateIdentity,
    /// Spectator sees card faces they shouldn't.
    SpectatorCardFace,
}

/// Scan a single observer snapshot for leaks against the authority state.
pub fn scan_view(state: &GameState, view: &ObserverSnapshot) -> Vec<Leak> {
    let mut leaks = Vec::new();
    let self_id = match view.observer {
        ObserverKind::Player(id) => Some(id),
        ObserverKind::ReplayExporter { privacy } => match privacy {
            engine_api::ReplayPrivacy::Player(id) => Some(id),
            _ => None,
        },
        _ => None,
    };

    // Check: foreign hand card faces must not be visible.
    for p in state.players.sorted_live() {
        if Some(p.id) == self_id {
            continue;
        }
        let foreign_hand = state.hand_cards(p.id);
        for cid in &foreign_hand {
            let leaked = view
                .known_cards
                .iter()
                .any(|c| c.id == *cid && (c.def.is_some() || c.suit.is_some()));
            if leaked {
                leaks.push(Leak {
                    kind: LeakKind::ForeignHandFace,
                    description: format!(
                        "observer {:?} sees foreign hand card {} face of player {}",
                        view.observer,
                        cid.raw(),
                        p.id.raw()
                    ),
                });
            }
        }
    }

    // Check: spectator must not see any hand card face.
    if matches!(view.observer, ObserverKind::Spectator) {
        for cv in &view.known_cards {
            if let Some(owner) = cv.owner {
                if cv.def.is_some() {
                    let hand = state.hand_cards(owner);
                    if hand.iter().any(|h| h == &cv.id) {
                        leaks.push(Leak {
                            kind: LeakKind::SpectatorCardFace,
                            description: format!(
                                "spectator sees hand card {} face of player {}",
                                cv.id.raw(),
                                owner.raw()
                            ),
                        });
                    }
                }
            }
        }
    }

    // Check: private identities hidden from non-lord, non-dead, non-self.
    let is_full = matches!(view.observer, ObserverKind::Referee)
        || matches!(
            view.observer,
            ObserverKind::ReplayExporter {
                privacy: engine_api::ReplayPrivacy::Full
            }
        );
    if !is_full {
        for seat in &view.seats {
            if seat.identity.is_some() {
                // Identity visible: is it lord, dead, or self?
                let is_self = self_id == Some(seat.player);
                let is_lord = seat.identity == Some(engine_api::IdentityTag::Lord);
                let is_dead = !seat.alive;
                if !is_self && !is_lord && !is_dead {
                    leaks.push(Leak {
                        kind: LeakKind::PrivateIdentity,
                        description: format!(
                            "observer {:?} sees private identity {:?} of player {}",
                            view.observer,
                            seat.identity,
                            seat.player.raw()
                        ),
                    });
                }
            }
        }
    }

    leaks
}

/// Scan all possible observers for a state (comprehensive harness).
pub fn scan_all_observers(state: &GameState) -> Vec<Leak> {
    let mut all_leaks = Vec::new();

    // Check each player's view.
    for p in state.players.sorted_live() {
        let view = engine_core::project_observer(state, ObserverKind::Player(p.id));
        all_leaks.extend(scan_view(state, &view));
    }

    // Check spectator view.
    let spec_view = engine_core::project_observer(state, ObserverKind::Spectator);
    all_leaks.extend(scan_view(state, &spec_view));

    all_leaks
}

/// Render leaks as text.
pub fn render_leaks(leaks: &[Leak]) -> String {
    if leaks.is_empty() {
        return "no leaks detected ✓\n".into();
    }
    let mut out = format!("{} leak(s) detected:\n", leaks.len());
    for (i, l) in leaks.iter().enumerate() {
        out.push_str(&format!("  {}. [{:?}] {}\n", i + 1, l.kind, l.description));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey};
    use engine_core::Engine;

    #[test]
    fn no_leaks_in_clean_match() {
        let mut eng = Engine::new();
        eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 42,
                ruleset: "identity-standard".into(),
            },
        });
        let leaks = scan_all_observers(eng.state());
        assert!(
            leaks.is_empty(),
            "clean match must have no leaks: {:?}",
            leaks
        );
    }
}

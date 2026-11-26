//! Observer Leak Test Harness (architecture §35 / §37.9).
//!
//! Systematically scans `PlayerView` observer projections for hidden information
//! leaks against the authoritative `GameStateCore` and `Engine`.

use engine_native::state::{GameStateCore, Role};
use engine_native::view::{HandView, PlayerView};
use engine_native::Engine;

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
    /// Private identity/role revealed to unauthorized observer.
    PrivateIdentity,
    /// Spectator sees hand card faces they shouldn't.
    SpectatorCardFace,
}

/// Scan a non-referee observer view (player seat or spectator) for hidden-information leaks.
pub fn scan_view(state: &GameStateCore, view: &PlayerView) -> Vec<Leak> {
    let mut leaks = Vec::new();
    let is_spectator = view.seat == u8::MAX;

    for cell in &view.players {
        let is_self = !is_spectator && cell.seat == view.seat;
        if !is_self {
            if let HandView::Own(cards) = &cell.hand {
                leaks.push(Leak {
                    kind: if is_spectator {
                        LeakKind::SpectatorCardFace
                    } else {
                        LeakKind::ForeignHandFace
                    },
                    description: format!(
                        "observer seat={} sees hand faces ({} cards) of player {}",
                        view.seat,
                        cards.len(),
                        cell.seat
                    ),
                });
            }
            if let Some(p) = state.players.get(cell.seat as usize) {
                for &cid in &p.hand {
                    if view.cards.iter().any(|c| c.id == cid) {
                        leaks.push(Leak {
                            kind: if is_spectator {
                                LeakKind::SpectatorCardFace
                            } else {
                                LeakKind::ForeignHandFace
                            },
                            description: format!(
                                "observer seat={} sees foreign hand card {} in view.cards of player {}",
                                view.seat, cid, cell.seat
                            ),
                        });
                    }
                }
            }
        }

        if let Some(role) = cell.role {
            let is_lord = role == Role::Lord;
            let is_dead = !cell.alive;
            if !is_self && !is_lord && !is_dead {
                leaks.push(Leak {
                    kind: LeakKind::PrivateIdentity,
                    description: format!(
                        "observer seat={} sees private role {:?} of player {}",
                        view.seat, role, cell.seat
                    ),
                });
            }
        }
    }

    leaks
}

/// Scan all seat views and spectator view on an authoritative `Engine`.
pub fn scan_all_observers(engine: &Engine) -> Vec<Leak> {
    let state = engine.state_core();
    let mut all_leaks = Vec::new();

    for seat in 0..state.players.len() as u8 {
        let view = engine.view_for(seat);
        all_leaks.extend(scan_view(state, &view));
    }

    let spec_view = engine.view_spectator();
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

    #[test]
    fn no_leaks_in_clean_match() {
        let eng = Engine::new(42, 4);
        let leaks = scan_all_observers(&eng);
        assert!(
            leaks.is_empty(),
            "clean match must have no leaks: {:?}",
            leaks
        );
    }
}

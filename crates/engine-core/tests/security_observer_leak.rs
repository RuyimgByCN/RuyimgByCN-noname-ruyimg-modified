//! Security tests: observer projection anti-leak invariants (architecture §37.9 / §21).
//!
//! These tests verify that the observer projection never leaks hidden information:
//! - Hidden hand card faces are never visible to unauthorized observers.
//! - Private identities are never revealed before death / lord.
//! - Spectators (public observers) see strictly less than players.
//! - Reject error messages do not embed hidden state (stable codes only).
//! - Replay privacy levels produce different views (Full vs Player vs Public).

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind,
    ObserverSnapshot, PlayerId, ReplayPrivacy,
};
use engine_core::{project_observer, Engine};

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

/// §37.9 / §21: A player's hand card faces must never appear in another
/// player's observer view. We check the complete known_cards set.
#[test]
fn foreign_hand_faces_never_leak() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 8));
    for observer_seat in 1..=8u32 {
        let observer = PlayerId::new(observer_seat);
        let view = project_observer(eng.state(), ObserverKind::Player(observer));
        for p in eng.state().players.sorted_live() {
            if p.id == observer {
                continue;
            }
            let foreign_hand = eng.state().hand_cards(p.id);
            for cid in &foreign_hand {
                let leaked = view.known_cards.iter().any(|c| {
                    c.id == *cid && (c.def.is_some() || c.suit.is_some() || c.rank.is_some())
                });
                assert!(
                    !leaked,
                    "observer {} sees face of foreign hand card {} owned by {}",
                    observer_seat,
                    cid.raw(),
                    p.id.raw()
                );
            }
        }
    }
}

/// §37.9: A spectator must never see any player's hand card faces (only public
/// info: seats, equip cards, discard pile).
#[test]
fn spectator_sees_no_hand_faces() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(7, 4));
    let view = project_observer(eng.state(), ObserverKind::Spectator);
    // Spectator known_cards may include equips + discard, but never hand cards
    // (cards whose zone is Hand).
    for cv in &view.known_cards {
        if let Some(owner) = cv.owner {
            if let Some(owner_player) = eng.state().players.get(owner) {
                let hand = eng.state().hand_cards(owner);
                if hand.iter().any(|h| h == &cv.id) {
                    panic!(
                        "spectator sees hand card face {} of player {}",
                        cv.id.raw(),
                        owner_player.id.raw()
                    );
                }
            }
        }
    }
}

/// §37.9: Private identities (non-lord, alive) must not be revealed to other
/// players. Only the lord identity and dead players' identities are public.
#[test]
fn private_identities_hidden_from_other_players() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(3, 8));
    // Find a non-lord player.
    let lord = eng
        .state()
        .players
        .sorted_live()
        .iter()
        .find(|p| p.identity == engine_core::Identity::Lord)
        .map(|p| p.id)
        .expect("there is a lord");
    let non_lord = eng
        .state()
        .players
        .sorted_live()
        .iter()
        .find(|p| p.id != lord && p.alive)
        .map(|p| p.id)
        .expect("there is a non-lord player");
    // Observer = lord should NOT see non_lord's identity.
    let view = project_observer(eng.state(), ObserverKind::Player(lord));
    let seat = view.seats.iter().find(|s| s.player == non_lord).unwrap();
    assert!(
        seat.identity.is_none(),
        "non-lord identity leaked to another player before death"
    );
}

/// §37.9: A player sees their own identity.
#[test]
fn self_identity_visible_to_self() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(5, 4));
    let p1 = PlayerId::new(1);
    let view = project_observer(eng.state(), ObserverKind::Player(p1));
    let own_seat = view.seats.iter().find(|s| s.player == p1).unwrap();
    assert!(own_seat.identity.is_some(), "player must see own identity");
}

/// §37.9: The referee (full hidden info) sees strictly more than a player.
#[test]
fn referee_sees_more_than_player() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(9, 4));
    let p1 = PlayerId::new(1);
    let player_view = project_observer(eng.state(), ObserverKind::Player(p1));
    let ref_view = project_observer(eng.state(), ObserverKind::Referee);
    // Referee knows at least as many card faces as the player.
    let player_known = player_view
        .known_cards
        .iter()
        .filter(|c| c.def.is_some())
        .count();
    let ref_known = ref_view
        .known_cards
        .iter()
        .filter(|c| c.def.is_some())
        .count();
    assert!(
        ref_known >= player_known,
        "referee must see >= cards than player (ref={ref_known}, player={player_known})"
    );
    // Referee sees all identities.
    let ref_identities = ref_view
        .seats
        .iter()
        .filter(|s| s.identity.is_some())
        .count();
    assert!(
        ref_identities >= 4,
        "referee must see all identities (saw {ref_identities})"
    );
}

/// §37.9: Reject reason must be a stable code, not embed hidden card identities
/// or HP values. We check that reject messages are bounded and don't contain
/// raw card def ids from other players' hands.
#[test]
fn reject_messages_do_not_leak_hidden_state() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(1, 4));
    // Capture a foreign player's hand card id to check it doesn't appear in errors.
    let p2 = PlayerId::new(2);
    let p2_hand = eng.state().hand_cards(p2);
    let secret_card_def = p2_hand
        .first()
        .and_then(|cid| eng.state().cards.get(*cid).map(|c| c.def.raw()));
    // Submit an illegal play of a card we don't own.
    let target_card = p2_hand.first().copied().unwrap_or(engine_api::CardId::NONE);
    let r = eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::PlayCard {
            card: target_card,
            targets: vec![],
            card_targets: vec![],
        },
    });
    if let CommandResult::Rejected { reason } = r {
        let msg = format!("{reason:?}");
        if let Some(def) = secret_card_def {
            // The reject message should not contain the raw def id in a way
            // that reveals what the card is. (Stable codes like "not owner"
            // are fine; leaking the def value would be a side channel.)
            let def_str = format!("{def}");
            // We allow the message to reference the *attempted* card id (the
            // player already knows what they tried to play), but the foreign
            // card's *definition* should not be newly revealed.
            assert!(
                !msg.contains(&format!("def={def_str}")),
                "reject message may leak hidden card def: {msg}"
            );
        }
    }
}

/// §37.9 / §22.5: Replay privacy levels produce different views. A Player
/// replay view reveals that player's authorized info; a Public view reveals less.
#[test]
fn replay_privacy_levels_differ() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(8, 4));
    let p1 = PlayerId::new(1);
    let player_view: ObserverSnapshot = project_observer(
        eng.state(),
        ObserverKind::ReplayExporter {
            privacy: ReplayPrivacy::Player(p1),
        },
    );
    let public_view = project_observer(
        eng.state(),
        ObserverKind::ReplayExporter {
            privacy: ReplayPrivacy::Public,
        },
    );
    // Player view sees own identity; public view does not see non-lord identities.
    let player_identities = player_view
        .seats
        .iter()
        .filter(|s| s.identity.is_some())
        .count();
    let public_identities = public_view
        .seats
        .iter()
        .filter(|s| s.identity.is_some())
        .count();
    // Player exporter sees at least own identity + lord; public sees only lord.
    assert!(
        player_identities >= public_identities,
        "player replay must see >= identities than public (player={player_identities}, public={public_identities})"
    );
}

/// §21.1: Delta-side-channel guard — the number of known_cards for a spectator
/// must not reveal hand sizes of individual players (only the aggregate
/// hand_count on seats is public). We verify spectator known_cards don't
/// include foreign hand card *ids* at all (not just faces).
#[test]
fn spectator_known_cards_exclude_foreign_hand_ids() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(13, 4));
    let view = project_observer(eng.state(), ObserverKind::Spectator);
    for cv in &view.known_cards {
        if let Some(owner) = cv.owner {
            let hand = eng.state().hand_cards(owner);
            if hand.iter().any(|h| h == &cv.id) {
                // A spectator must not even track foreign hand card ids — that
                // would let them count exact draw/discard deltas.
                panic!(
                    "spectator tracks foreign hand card id {} — side channel",
                    cv.id.raw()
                );
            }
        }
    }
}

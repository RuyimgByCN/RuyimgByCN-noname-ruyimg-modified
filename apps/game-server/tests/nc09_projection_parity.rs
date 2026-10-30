//! NC09 decision15: compare server snapshots with the actual local WASM projection.
#[path = "../src/native_view.rs"]
mod native_view;

use engine_api::{CardZoneTag, PlayerId};
use engine_native::{state::GameMode, Engine};
use native_view::{observer_snapshot_for, RoomRole};
use serde_json::Value;
use std::collections::BTreeSet;

#[path = "support/nc09_projection_fixtures.rs"]
mod fixtures;
use fixtures::{all_cards_fixture, fixture};

fn local(engine: &Engine, seat: u8) -> Value {
    let hex: String = engine_native::archive::encode_archive(&engine.save())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let mut wasm = engine_native_wasm::NativeEngine::new_identity(4, 1515).unwrap();
    wasm.restore_hex(&hex).unwrap();
    serde_json::from_str(&wasm.view_json(seat).unwrap()).unwrap()
}

fn assert_parity(engine: &Engine) {
    for seat in 0..4 {
        let local = local(engine, seat);
        let server = observer_snapshot_for(
            engine,
            RoomRole::Player,
            Some(PlayerId::new(seat as u32 + 1)),
        );
        let local_ids: BTreeSet<u32> = local["cards"]
            .as_object()
            .unwrap()
            .keys()
            .map(|id| id.parse().unwrap())
            .collect();
        assert_eq!(
            server
                .known_cards
                .iter()
                .map(|c| c.id.raw())
                .collect::<BTreeSet<_>>(),
            local_ids,
            "server must preserve every locally visible card, including public zones"
        );
        for card in &server.known_cards {
            let face = &local["cards"][card.id.raw().to_string()];
            assert_eq!(serde_json::to_value(card.def).unwrap(), face["def"]);
            assert_eq!(serde_json::to_value(card.suit).unwrap(), face["suit"]);
            assert_eq!(serde_json::to_value(card.rank).unwrap(), face["rank"]);
            assert_eq!(serde_json::to_value(card.kind).unwrap(), face["kind"]);
            match card.zone {
                CardZoneTag::Hand => {
                    assert_eq!(card.owner, Some(PlayerId::new(seat as u32 + 1)));
                    assert!(local["players"][seat as usize]["hand"]["own"]
                        .as_array()
                        .unwrap()
                        .contains(&Value::from(card.id.raw())));
                }
                CardZoneTag::Equip | CardZoneTag::Judge => {
                    let owner = card.owner.unwrap().raw() as usize - 1;
                    let zone = if card.zone == CardZoneTag::Equip {
                        "equips"
                    } else {
                        "judge"
                    };
                    assert!(local["players"][owner][zone]
                        .as_array()
                        .unwrap()
                        .contains(&Value::from(card.id.raw())));
                }
                CardZoneTag::DiscardPile | CardZoneTag::Processing => {
                    assert!(card.owner.is_none());
                    let zone = if card.zone == CardZoneTag::DiscardPile {
                        "discard"
                    } else {
                        "table"
                    };
                    assert!(local[zone]
                        .as_array()
                        .unwrap()
                        .contains(&Value::from(card.id.raw())));
                }
                other => panic!("unexpected visible zone {other:?}"),
            }
        }
        for (server, local) in server
            .seats
            .iter()
            .zip(local["players"].as_array().unwrap())
        {
            assert_eq!(
                serde_json::to_value(server.identity).unwrap(),
                local["identity"]
            );
            assert_eq!(Value::from(server.character_def), local["characterDef"]);
        }
        let hidden: BTreeSet<_> = engine
            .state_core()
            .deck
            .iter()
            .copied()
            .chain(
                engine
                    .state_core()
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != seat as usize)
                    .flat_map(|(_, p)| p.hand.iter().copied()),
            )
            .collect();
        assert!(server
            .known_cards
            .iter()
            .all(|c| !hidden.contains(&c.id.raw())));
    }
}

#[test]
fn nc09_server_cards_and_identity_match_local_before_and_after_restart() {
    for (_, engine) in fixtures::scenarios() {
        assert_parity(&engine);
        let bytes = engine_native::archive::encode_archive(&engine.save());
        let restored =
            Engine::restore(&engine_native::archive::decode_archive(&bytes).unwrap()).unwrap();
        assert_parity(&restored);
    }
    let restored = fixture();
    for seat in [None, Some(PlayerId::new(2))] {
        {
            let view = observer_snapshot_for(&restored, RoomRole::Spectator, seat);
            assert_eq!(
                view.known_cards.len(),
                4,
                "observer sees four public cards and no hands"
            );
            assert!(view.known_cards.iter().all(|c| c.zone != CardZoneTag::Hand));
            assert!(
                view.seats[1].identity.is_none(),
                "role must control identity visibility"
            );
        }
    }
}

#[test]
fn nc09_server_faces_have_real_suit_and_rank() {
    let engine = fixture();
    let view = observer_snapshot_for(&engine, RoomRole::Player, Some(PlayerId::new(1)));
    assert!(view
        .known_cards
        .iter()
        .all(|c| c.suit.is_some() && c.rank.is_some()));
}

#[test]
fn nc09_server_general_visibility_uses_native_projection() {
    let engine = fixture();
    let mut snapshot = engine.save();
    snapshot.core.mode = GameMode::Kingdoms;
    snapshot.core.player_mut(1).revealed_generals = [false, false];
    let engine = Engine::restore(&snapshot).unwrap();
    let general = engine.state_core().player(1).general_slots[0].unwrap() as u32;
    assert_ne!(general, 0);
    let own = observer_snapshot_for(&engine, RoomRole::Player, Some(PlayerId::new(2)));
    assert_eq!(own.seats[1].character_def, general);
    for role in [RoomRole::Spectator, RoomRole::Player] {
        let view = observer_snapshot_for(&engine, role, Some(PlayerId::new(1)));
        assert_eq!(
            view.seats[1].character_def, 0,
            "unrevealed opponent general must stay hidden"
        );
    }
    assert_parity(&engine);
    let mut snapshot = engine.save();
    snapshot.core.player_mut(1).revealed_generals[0] = true;
    let revealed = Engine::restore(&snapshot).unwrap();
    assert_eq!(
        observer_snapshot_for(&revealed, RoomRole::Spectator, None).seats[1].character_def,
        general
    );
    assert_parity(&revealed);
}

#[test]
fn nc09_dead_identity_is_public_after_restart() {
    let engine = fixture();
    let mut snapshot = engine.save();
    snapshot.core.player_mut(2).alive = false;
    snapshot.core.player_mut(2).hp = 0;
    let engine = Engine::restore(&snapshot).unwrap();
    assert_parity(&engine);
    let view = observer_snapshot_for(&engine, RoomRole::Spectator, None);
    assert!(view.seats[0].identity.is_some(), "lord remains public");
    assert!(view.seats[2].identity.is_some(), "dead identity is public");
    assert!(
        view.seats[1].identity.is_none(),
        "living opponent stays hidden"
    );
}

#[test]
fn nc09_invalid_seat_cannot_wrap_to_a_private_projection() {
    let engine = fixture();
    for seat in [PlayerId::NONE, PlayerId::new(257), PlayerId::new(u32::MAX)] {
        let view = observer_snapshot_for(&engine, RoomRole::Player, Some(seat));
        assert_eq!(view.known_cards.len(), 4);
        assert!(view.known_cards.iter().all(|c| c.zone != CardZoneTag::Hand));
        assert!(view.seats[1].identity.is_none());
    }
}

#[test]
fn nc09_projection_uses_existing_v5_snapshot_and_card_fields() {
    let engine = fixture();
    let view = observer_snapshot_for(&engine, RoomRole::Player, Some(PlayerId::new(1)));
    let message = engine_protocol::ServerMessage::Snapshot {
        header: engine_protocol::MessageHeader::new(15, 1, engine.revision()),
        view: view.clone(),
        choice: native_view::choice_for(&engine, RoomRole::Player, Some(PlayerId::new(1))),
        legal_actions: native_view::legal_actions_for(&engine, PlayerId::new(1)),
    };
    assert_eq!(engine_protocol::NETWORK_PROTOCOL_VERSION, 5);
    let wire = engine_protocol::encode_s2c(&message).unwrap();
    let engine_protocol::ServerMessage::Snapshot {
        header,
        view: decoded,
        ..
    } = engine_protocol::decode_s2c(&wire).unwrap()
    else {
        panic!("snapshot");
    };
    assert_eq!(header.protocol_version, 5);
    assert_eq!(decoded, view);
}

#[test]
fn nc09_every_standard_card_kind_preserves_local_granularity() {
    let engine = all_cards_fixture();
    assert_parity(&engine);
    let public = observer_snapshot_for(&engine, RoomRole::Spectator, None);
    assert_eq!(public.known_cards.len(), engine.catalog().cards.len());
    let kinds: BTreeSet<_> = public
        .known_cards
        .iter()
        .map(|c| format!("{:?}", c.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            "Sha",
            "Shan",
            "Tao",
            "Jiu",
            "ShaFire",
            "ShaThunder",
            "Other"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );
}

#[test]
fn nc09_referee_retains_authorized_unrevealed_general_definition() {
    let mut snapshot = fixture().save();
    snapshot.core.mode = GameMode::Kingdoms;
    snapshot.core.player_mut(1).revealed_generals = [false, false];
    let engine = Engine::restore(&snapshot).unwrap();
    let general = engine.state_core().player(1).general_slots[0].unwrap() as u32;
    assert_ne!(general, 0);
    for seat in [
        None,
        Some(PlayerId::new(1)),
        Some(PlayerId::NONE),
        Some(PlayerId::new(257)),
    ] {
        let view = observer_snapshot_for(&engine, RoomRole::Referee, seat);
        assert_eq!(view.observer, engine_api::ObserverKind::Referee);
        assert_eq!(
            view.seats[1].character_def, general,
            "Referee is full-info authority, not a seated Player or Spectator"
        );
    }
}

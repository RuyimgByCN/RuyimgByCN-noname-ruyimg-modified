//! Referee/native agreement at real XunXun, JiJie and judgment choice windows.
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../../../crates/engine-native/tests/support/nc09_machine_windows.rs"]
mod windows;

use engine_api::{CardZoneTag, PlayerId};
use native_view::{observer_snapshot_for, RoomRole};
use windows::{fixture, Window};

fn projection(window: Window) {
    let fixture = fixture(window);
    let engine = &fixture.engine;
    let before = engine.save();
    let native = engine.view_referee();
    let server = observer_snapshot_for(engine, RoomRole::Referee, None);
    let mut ids: Vec<_> = server
        .known_cards
        .iter()
        .map(|card| card.id.raw())
        .collect();
    ids.sort_unstable();
    assert_eq!(
        ids, fixture.physical_ids,
        "{window:?}: exact physical multiset"
    );
    assert_eq!(
        server
            .known_cards
            .iter()
            .map(|card| card.id.raw())
            .collect::<Vec<_>>(),
        native.visible_card_ids()
    );
    for card in &server.known_cards {
        let expected = native
            .cards
            .iter()
            .find(|location| location.id == card.id.raw())
            .unwrap();
        assert_eq!(
            card.owner,
            expected.owner.map(|seat| PlayerId::new(seat as u32 + 1))
        );
        assert_eq!(format!("{:?}", card.zone), format!("{:?}", expected.zone));
        let face = &engine.catalog().cards[card.id.raw() as usize];
        assert_eq!(card.def, Some(face.kind as u16 as u32));
        assert_eq!(card.rank, Some(face.rank));
        assert!(card.suit.is_some());
    }
    for id in &fixture.held {
        let card = server
            .known_cards
            .iter()
            .find(|card| card.id.raw() == *id)
            .unwrap();
        assert_eq!((card.zone, card.owner), (CardZoneTag::Processing, None));
    }
    for seat in 0..engine.state_core().players.len() as u32 {
        let player = PlayerId::new(seat + 1);
        let ordinary = observer_snapshot_for(engine, RoomRole::Player, Some(player));
        assert!(ordinary
            .known_cards
            .iter()
            .all(|card| !fixture.held.contains(&card.id.raw())));
        assert_eq!(
            ordinary
                .known_cards
                .iter()
                .map(|card| card.id.raw())
                .collect::<Vec<_>>(),
            engine.view_for(seat as u8).visible_card_ids()
        );
        let spectator = observer_snapshot_for(engine, RoomRole::Spectator, Some(player));
        assert!(spectator
            .known_cards
            .iter()
            .all(|card| !fixture.held.contains(&card.id.raw())));
        assert!(native_view::choice_for(engine, RoomRole::Spectator, Some(player)).is_none());
    }
    let bytes = engine_native::archive::encode_archive(&before);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(decoded, before);
    let restored = fixture.restore(&decoded).unwrap();
    assert_eq!(
        observer_snapshot_for(&restored, RoomRole::Referee, None),
        server
    );
    assert_eq!(
        native_view::choice_for(&restored, RoomRole::Referee, None),
        native_view::choice_for(engine, RoomRole::Referee, None)
    );
    assert_eq!(engine.save(), before);
}

#[test]
fn xunxun_native_server_referee_and_privacy() {
    projection(Window::XunXun);
}
#[test]
fn jijie_native_server_referee_and_privacy() {
    projection(Window::JiJie);
}
#[test]
fn gaiming_native_server_referee_and_privacy() {
    projection(Window::GaiMing);
}
#[test]
fn haotianta_native_server_referee_and_privacy() {
    projection(Window::HaoTianTa);
}

#[test]
fn liaoyuan_native_server_referee_and_privacy() {
    projection(Window::Liaoyuan);
}

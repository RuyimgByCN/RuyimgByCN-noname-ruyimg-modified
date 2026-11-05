//! NC09: authorized cards come from physical ownership, never catalog membership.
#[path = "../src/native_view.rs"]
mod native_view;

use engine_api::{CardZoneTag, PlayerId};
use engine_native::Engine;
use native_view::{observer_snapshot_for, RoomRole};
use std::collections::BTreeSet;

#[test]
fn referee_sees_all_physical_cards_faces_owners_and_deck_order_after_archive() {
    let engine = Engine::new_identity(1515, 4);
    let mut snapshot = engine.save();
    for index in 0..6 {
        let card = snapshot.core.deck.pop().unwrap();
        let p = snapshot.core.player_mut(1);
        match index {
            0 => {
                p.off_court.push(card);
                p.yishan_lost.push(card);
                p.jiguanyuan_cards.push(card);
            }
            1 => p.zhucheng.push(card),
            2 => p.lianyaohu_store.push(card),
            3 => p.sxianjing_traps.push(card),
            4 => p.hutian_hold.push(card),
            5 => {
                snapshot
                    .core
                    .vm_aux
                    .piles
                    .insert((2, "test.pile".into()), vec![card]);
            }
            _ => unreachable!(),
        }
    }
    let encoded = engine_native::archive::encode_archive(&snapshot);
    let decoded = engine_native::archive::decode_archive(&encoded).unwrap();
    assert_eq!(snapshot, decoded);
    let engine = Engine::restore(&decoded).unwrap();
    let before = engine.save();
    let authority_bytes = engine_native::archive::encode_archive(&before);
    let view = observer_snapshot_for(&engine, RoomRole::Referee, None);
    assert_eq!(view.known_cards.len(), engine.catalog().cards.len());
    let ids: BTreeSet<_> = view.known_cards.iter().map(|card| card.id.raw()).collect();
    assert_eq!(
        ids.len(),
        view.known_cards.len(),
        "references must not duplicate ownership"
    );
    assert_eq!(ids, (0..engine.catalog().cards.len() as u32).collect());
    assert!(view.seats.iter().all(|seat| seat.identity.is_some()));
    for card in &view.known_cards {
        let def = &engine.catalog().cards[card.id.raw() as usize];
        assert_eq!(card.def, Some(def.kind as u16 as u32));
        assert!(card.suit.is_some());
        assert_eq!(card.rank, Some(def.rank));
        if card.zone == CardZoneTag::Hand {
            let seat = card.owner.unwrap().raw() as usize - 1;
            assert!(engine.state_core().players[seat]
                .hand
                .contains(&card.id.raw()));
        }
    }
    assert_eq!(
        view.known_cards
            .iter()
            .filter(|c| c.zone == CardZoneTag::DrawPile)
            .map(|c| c.id.raw())
            .collect::<Vec<_>>(),
        snapshot.core.deck.iter().copied().collect::<Vec<_>>()
    );
    assert_eq!(
        view.known_cards
            .iter()
            .filter(|c| c.zone == CardZoneTag::Removed)
            .map(|c| (c.id.raw(), c.owner))
            .collect::<Vec<_>>(),
        vec![(snapshot.core.player(1).off_court[0], Some(PlayerId::new(2)))]
    );
    for id in snapshot
        .core
        .player(1)
        .zhucheng
        .iter()
        .chain(&snapshot.core.player(1).lianyaohu_store)
        .chain(&snapshot.core.player(1).sxianjing_traps)
        .chain(&snapshot.core.player(1).hutian_hold)
    {
        let card = view.known_cards.iter().find(|c| c.id.raw() == *id).unwrap();
        assert_eq!(
            (card.zone, card.owner),
            (CardZoneTag::Expansion, Some(PlayerId::new(2)))
        );
    }
    let vm = view
        .known_cards
        .iter()
        .find(|c| c.id.raw() == snapshot.core.vm_aux.piles[&(2, "test.pile".into())][0])
        .unwrap();
    assert_eq!(
        (vm.zone, vm.owner),
        (CardZoneTag::Expansion, Some(PlayerId::new(3)))
    );
    for role in [RoomRole::Spectator, RoomRole::Player] {
        let ordinary = observer_snapshot_for(&engine, role, Some(PlayerId::new(1)));
        assert!(ordinary
            .known_cards
            .iter()
            .all(|c| c.zone == CardZoneTag::Hand && c.owner == Some(PlayerId::new(1))));
        if role == RoomRole::Spectator {
            assert!(ordinary.known_cards.is_empty());
        }
    }
    assert_eq!(engine.save(), before);
    assert_eq!(
        engine_native::archive::encode_archive(&engine.save()),
        authority_bytes
    );
}

fn submit(engine: &mut Engine, action: engine_native::GameAction) {
    let actor = engine
        .open_choice()
        .map_or(engine.turn().current, |c| c.observer);
    let result = engine.submit(engine_native::GameCommand {
        actor,
        expected_revision: engine.revision(),
        dedup_id: engine.revision() + 1,
        action,
    });
    assert!(
        !matches!(result, engine_native::CommandOutcome::Rejected(_)),
        "{result:?}"
    );
}

fn assert_choice_kind(engine: &Engine, kind: engine_api::ChoiceKind) {
    let owner = PlayerId::new(engine.open_choice().unwrap().observer as u32 + 1);
    let choice = native_view::choice_for(engine, RoomRole::Player, Some(owner)).unwrap();
    assert_eq!(choice.kind, kind);
    assert_eq!(
        Some(choice.clone()),
        native_view::choice_for(engine, RoomRole::Referee, None)
    );
    assert!(native_view::choice_for(engine, RoomRole::Spectator, Some(owner)).is_none());
    let message = engine_protocol::ServerMessage::Snapshot {
        header: engine_protocol::MessageHeader::new(909, 1, engine.revision()),
        view: observer_snapshot_for(engine, RoomRole::Referee, None),
        legal_actions: vec![],
        choice: Some(choice.clone()),
    };
    let wire = engine_protocol::encode_s2c(&message).unwrap();
    let engine_protocol::ServerMessage::Snapshot {
        header,
        choice: decoded,
        ..
    } = engine_protocol::decode_s2c(&wire).unwrap()
    else {
        panic!("snapshot")
    };
    assert_eq!(header.protocol_version, 8);
    assert_eq!(decoded, Some(choice));
}

#[test]
fn current_choices_preserve_all_expressible_existing_wire_kinds() {
    let mut engine = Engine::new(804, 3);
    engine.debug_grant(0, "突袭");
    engine.state_core_mut().turn.phase = engine_native::Phase::Judge;
    submit(&mut engine, engine_native::GameAction::NextPhase);
    assert_eq!(
        engine.open_choice().unwrap().spec.kind,
        engine_native::ChoiceKind::Confirm
    );
    assert_choice_kind(&engine, engine_api::ChoiceKind::Confirm);
    let id = engine.open_choice().unwrap().id;
    submit(
        &mut engine,
        engine_native::GameAction::Answer {
            choice_id: id,
            selection: vec![engine_native::ChoiceOption::Confirm],
        },
    );
    assert_eq!(
        engine.open_choice().unwrap().spec.kind,
        engine_native::ChoiceKind::SelectPlayers
    );
    assert_choice_kind(&engine, engine_api::ChoiceKind::SelectTargets);
    let mut discard = Engine::new(1515, 4);
    for _ in 0..3 {
        submit(&mut discard, engine_native::GameAction::NextPhase);
    }
    assert_eq!(
        discard.open_choice().unwrap().spec.kind,
        engine_native::ChoiceKind::SelectCards
    );
    assert_choice_kind(&discard, engine_api::ChoiceKind::SelectCards);
}

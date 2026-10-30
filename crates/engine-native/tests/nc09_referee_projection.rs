//! Trusted full information projection must remain read-only and isolated from AI seats.
use engine_native::{
    ChoiceOption, ChoiceView, CommandOutcome, Engine, GameAction, GameCommand, HandView,
};

fn submit(engine: &mut Engine, action: GameAction) {
    let out = engine.submit(GameCommand {
        actor: engine
            .open_choice()
            .map_or(engine.turn().current, |c| c.observer),
        expected_revision: engine.revision(),
        dedup_id: engine.revision() + 1,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}

fn restart(engine: &Engine) -> Engine {
    let before = engine.save();
    let bytes = engine_native::archive::encode_archive(&before);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(before, decoded);
    let restored = Engine::restore(&decoded).unwrap();
    assert_eq!(engine.state_hash(), restored.state_hash());
    restored
}

#[test]
fn referee_reveals_both_dark_generals_and_identities_without_mutating_authority() {
    let mut engine = Engine::new_identity(1515, 4);
    engine.assign_general_named(0, "曹操").unwrap();
    engine.assign_general_named(1, "刘备").unwrap();
    let identity_view = engine.view_referee();
    for (seat, player) in identity_view.players.iter().enumerate() {
        assert_eq!(
            player.role,
            Some(engine.state_core().identity_roles().unwrap()[seat])
        );
        assert_eq!(
            player.hand,
            HandView::Own(engine.state_core().players[seat].hand.clone())
        );
    }
    let mut snapshot = engine.save();
    snapshot.core.mode = engine_native::GameMode::Kingdoms;
    snapshot.core.players[1].general_slots[1] = snapshot.core.players[0].general_slots[0];
    snapshot.core.players[1].revealed_generals = [false, false];
    let engine = Engine::restore(&snapshot).unwrap();
    let before = engine.save();
    let view = engine.view_referee();
    for slot in 0..2 {
        let def = snapshot.core.players[1].general_slots[slot].unwrap();
        let meta = engine.catalog().general_meta(def).unwrap();
        let general = view.players[1].generals[slot].as_ref().unwrap();
        assert_eq!(
            (general.def, general.name, general.faction),
            (def, meta.0, meta.1)
        );
    }
    assert_eq!(engine.view_for(0).players[1].generals, [None, None]);
    assert_eq!(engine.view_spectator().players[1].generals, [None, None]);
    assert_eq!(restart(&engine).view_referee(), view);
    assert_eq!(engine.save(), before);
}

#[test]
fn full_info_entry_cannot_escape_to_ordinary_views_or_ai_sessions() {
    let engine = Engine::new_identity(1515, 4);
    let mut other = engine.save();
    std::mem::swap(&mut other.core.players[1].hand[0], &mut other.core.deck[0]);
    let other = Engine::restore(&other).unwrap();
    assert_ne!(engine.view_referee(), other.view_referee());
    assert_eq!(engine.view_for(0), other.view_for(0));
    assert_eq!(engine.view_spectator(), other.view_spectator());
    let a = engine.ai_session(0).input();
    let b = other.ai_session(0).input();
    assert_eq!(a.view, b.view);
    assert_eq!(a.choice, b.choice);
    assert_eq!(a.own_hand_kinds, b.own_hand_kinds);
    assert_eq!(engine.view_for(0), a.view);
    for view in [a.view, engine.view_spectator()] {
        assert!(view.cards.iter().all(|card| {
            use engine_native::snap::PhysicalCardZone::*;
            matches!(card.zone, Hand | Equip | Judge | DiscardPile | Processing)
        }));
    }
}

#[test]
fn referee_includes_machine_held_cards_and_private_choice_after_archive() {
    let mut engine = Engine::new(901, 3);
    engine.debug_grant(0, "观星");
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    for _ in 0..128 {
        if engine.open_choice().is_some() {
            break;
        }
        submit(&mut engine, GameAction::NextPhase);
    }
    assert_eq!(engine.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let choice_id = engine.open_choice().unwrap().id;
    submit(
        &mut engine,
        GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let open = engine.open_choice().unwrap().clone();
    assert_eq!(open.spec.prompt, "guanxing_arrange");
    let held: Vec<_> = open
        .spec
        .options
        .iter()
        .filter_map(|option| {
            if let ChoiceOption::Card(id) = option {
                Some(*id)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(held.len(), 3);
    let view = engine.view_referee();
    let ids = view.visible_card_ids();
    let unique: std::collections::BTreeSet<_> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());
    assert_eq!(ids.len(), engine.catalog().cards.len());
    for id in held {
        let card = view.cards.iter().find(|card| card.id == id).unwrap();
        assert_eq!(card.zone, engine_native::snap::PhysicalCardZone::Processing);
        assert_eq!(card.owner, None);
        assert!(!engine.view_spectator().visible_card_ids().contains(&id));
    }
    assert_eq!(
        engine.choice_view_referee(),
        engine.choice_view_for(Some(open.observer))
    );
    assert!(matches!(
        engine.choice_view_for(None),
        Some(ChoiceView::Other { .. })
    ));
    assert!(
        matches!(engine.choice_view_referee(), Some(ChoiceView::Own { kind, prompt, options, min, max, allow_cancel, .. })
        if kind == open.spec.kind && prompt == open.spec.prompt && options == open.spec.options && min == open.spec.min && max == open.spec.max && allow_cancel == open.spec.allow_cancel)
    );
    let mut restored = restart(&engine);
    assert_eq!(restored.view_referee(), view);
    assert_eq!(restored.choice_view_referee(), engine.choice_view_referee());
    submit(
        &mut engine,
        GameAction::Answer {
            choice_id: open.id,
            selection: vec![],
        },
    );
    submit(
        &mut restored,
        GameAction::Answer {
            choice_id: open.id,
            selection: vec![],
        },
    );
    assert_eq!(restored.state_hash(), engine.state_hash());
    assert_eq!(restored.save().core, engine.save().core);
    assert_eq!(restored.save().resolution, engine.save().resolution);
}

//! Shared authoritative states for native-facade and generated-WASM projection comparisons.
use engine_native::{catalog::CardKind, state::GameMode, Engine};

pub fn fixture() -> Engine {
    let mut engine = Engine::new_identity(1515, 4);
    engine.assign_general_named(0, "曹操").unwrap();
    engine.assign_general_named(1, "刘备").unwrap();
    let mut snapshot = engine.save();
    let equipment = snapshot
        .core
        .deck
        .iter()
        .position(|id| engine.catalog().card_kind(*id) == CardKind::Weapon)
        .unwrap();
    let equipment = snapshot.core.deck.remove(equipment);
    snapshot.core.player_mut(1).equips.push(equipment);
    let judge = snapshot
        .core
        .deck
        .iter()
        .position(|id| engine.catalog().card_kind(*id) == CardKind::Le)
        .unwrap();
    let judge = snapshot.core.deck.remove(judge);
    snapshot.core.player_mut(2).judge.push(judge);
    let discard = snapshot.core.deck.pop().unwrap();
    snapshot.core.discard.push(discard);
    let processing = snapshot.core.deck.pop().unwrap();
    snapshot.core.table.push(processing);
    Engine::restore(&snapshot).unwrap()
}

pub fn all_cards_fixture() -> Engine {
    let engine = fixture();
    let mut snapshot = engine.save();
    snapshot.core.discard.append(&mut snapshot.core.deck);
    for player in &mut snapshot.core.players {
        snapshot.core.discard.append(&mut player.hand);
        snapshot.core.discard.append(&mut player.equips);
        snapshot.core.discard.append(&mut player.judge);
    }
    snapshot.core.discard.append(&mut snapshot.core.table);
    Engine::restore(&snapshot).unwrap()
}

pub fn scenarios() -> Vec<(&'static str, Engine)> {
    let engine = fixture();
    let mut dead = engine.save();
    dead.core.player_mut(2).alive = false;
    dead.core.player_mut(2).hp = 0;
    let mut hidden = engine.save();
    hidden.core.mode = GameMode::Kingdoms;
    hidden.core.player_mut(1).revealed_generals = [false, false];
    let mut revealed = hidden.clone();
    revealed.core.player_mut(1).revealed_generals[0] = true;
    vec![
        ("identity-public-zones", engine),
        ("dead-identity", Engine::restore(&dead).unwrap()),
        ("kingdoms-hidden", Engine::restore(&hidden).unwrap()),
        ("kingdoms-revealed", Engine::restore(&revealed).unwrap()),
        ("all-kinds", all_cards_fixture()),
    ]
}

//! Identity mode: four/eight seats, private views, pending restore, and native/WASM trajectories.
use super::{packs, run};
use engine_native::state::{GameMode, Role};
use engine_native::view::HandView;
use engine_native::Engine;
use engine_native_wasm::test_support::{
    assign_general_for, engine_from_archive, engine_identity_from_packs, hex_encode,
};
use serde_json::{json, Value};

const SEED: u64 = 20261028;
const STEPS: usize = 80;

fn start(players: u8, packs: &str) -> Engine {
    let mut engine = engine_identity_from_packs(players, SEED, packs).expect("identity with packs");
    assert!(matches!(
        engine.state_core().mode,
        GameMode::Identity { .. }
    ));
    assign_general_for(&mut engine, 0, "vm_fixture").expect("assign VM general");
    assert!(engine.view_for(0).players[0]
        .skills
        .contains(&"vm_fixture_phase_end_phase_end"));
    engine
}

fn check_native_identity_privacy(engine: &Engine) {
    let roles = engine
        .state_core()
        .identity_roles()
        .expect("identity roles");
    assert_eq!(roles[0], Role::Lord);
    assert_eq!(roles.iter().filter(|r| **r == Role::Lord).count(), 1);
    assert_eq!(roles.iter().filter(|r| **r == Role::Traitor).count(), 1);
    assert_eq!(
        roles.iter().filter(|r| **r == Role::Loyalist).count(),
        if roles.len() == 4 { 1 } else { 2 }
    );
    for observer in 0..roles.len() {
        let view = engine.view_for(observer as u8);
        for (seat, cell) in view.players.iter().enumerate() {
            let expected = (seat == observer || roles[seat] == Role::Lord || !cell.alive)
                .then_some(roles[seat]);
            assert_eq!(cell.role, expected, "observer={observer} seat={seat}");
            assert_eq!(matches!(cell.hand, HandView::Own(_)), seat == observer);
        }
    }
    for (seat, cell) in engine.view_spectator().players.iter().enumerate() {
        let expected = (roles[seat] == Role::Lord || !cell.alive).then_some(roles[seat]);
        assert_eq!(cell.role, expected, "spectator seat={seat}");
        assert!(matches!(cell.hand, HandView::Hidden));
    }
}

// shortcut: WASM omits identity, compare it when the view API exposes filtered identities.
fn wasm_compatible_views(engine: &Engine) -> Vec<Value> {
    (0..engine.state_core().players.len())
        .map(|observer| {
            let view = engine.view_for(observer as u8);
            json!({
                "seat": view.seat,
                "players": view.players.iter().map(|p| json!({
                    "seat": p.seat, "alive": p.alive, "hp": p.hp, "maxHp": p.max_hp,
                    "hand": match &p.hand {
                        HandView::Own(cards) => json!({"own": cards}),
                        HandView::Hidden => json!({"hidden": true}),
                    },
                    "handCount": p.hand_count, "equips": p.equips, "judge": p.judge,
                    "turnedOver": p.turned_over, "skills": p.skills,
                    "revealedGenerals": p.revealed_generals,
                })).collect::<Vec<_>>(),
                "deckCount": view.deck_count, "discard": view.discard, "table": view.table,
                "turn": view.turn, "phase": format!("{:?}", view.phase).to_lowercase(),
                "round": view.round, "outcome": null, "rulesetVersion": view.ruleset_version,
            })
        })
        .collect()
}

#[test]
fn identity_multiseat_vm_and_package_match_restore_trajectories() {
    let packs = packs();
    let mut scenarios = Vec::new();
    for players in [4, 8] {
        let mut direct_engine = start(players, &packs);
        check_native_identity_privacy(&direct_engine);
        let initial_views = wasm_compatible_views(&direct_engine);
        let direct = run(&mut direct_engine, STEPS);
        check_native_identity_privacy(&direct_engine);
        let final_views = wasm_compatible_views(&direct_engine);
        let checkpoint = direct
            .iter()
            .position(|record| record["choice"]["hidden"] == true)
            .expect("another seat has a private choice");
        assert!(direct.iter().any(|r| r["before_turn"] == 0
            && r["before_phase"] == "discard"
            && r["choice"].is_null()
            && r["turn"] == 0
            && r["phase"] == "end"
            && r["hand0"].as_u64().unwrap() == r["before_hand0"].as_u64().unwrap() + 1));
        let mut first = start(players, &packs);
        let before = run(&mut first, checkpoint);
        assert!(first.open_choice().is_some_and(|c| c.observer != 0));
        check_native_identity_privacy(&first);
        let checkpoint_views = wasm_compatible_views(&first);
        let saved = hex_encode(&engine_native::archive::encode_archive(&first.save()));
        let mut restored =
            engine_from_archive(&saved, &packs).expect("restore pending identity choice");
        check_native_identity_privacy(&restored);
        assert_eq!(wasm_compatible_views(&restored), checkpoint_views);
        let after = run(&mut restored, STEPS - checkpoint);
        check_native_identity_privacy(&restored);
        assert_eq!(wasm_compatible_views(&restored), final_views);
        assert_eq!(before, direct[..checkpoint]);
        assert_eq!(after, direct[checkpoint..]);
        scenarios.push(json!({"players": players, "seed": SEED, "direct": direct,
            "initial_views": initial_views, "checkpoint_views": checkpoint_views, "final_views": final_views,
            "restore": {"midpoint": checkpoint, "before": before, "after": after}}));
    }
    if let Some(path) = std::env::var_os("NC08_IDENTITY_OUTPUT") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({"packs": packs, "scenarios": scenarios})).unwrap(),
        )
        .unwrap();
    }
}

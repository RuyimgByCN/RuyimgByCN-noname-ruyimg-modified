//! NC08: multi-seat native/WASM trajectories with a real installed VM fixture.
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_wasm::test_support::{
    assign_general_for, engine_from_archive, engine_from_packs, hex_encode,
};
use serde_json::{json, Value};

const SEED: u64 = 20261027;
const STEPS: usize = 80;

fn packs() -> String {
    let mut vm: Value = serde_json::from_str(include_str!(
        "../../../apps/web-client/scripts/fixtures/vm-trigger/package.npack.json"
    ))
    .unwrap();
    vm["characters"] = serde_json::from_str(include_str!(
        "../../../apps/web-client/scripts/fixtures/vm-trigger/character.json"
    ))
    .unwrap();
    let huanle: Value = serde_json::from_str(include_str!(
        "../../../packages/extensions/huanle-upstream-noname/package.npack.json"
    ))
    .unwrap();
    let cards: Value = serde_json::from_str(include_str!(
        "../../../packages/extensions/huanle-upstream-noname/assets/cards.json"
    ))
    .unwrap();
    json!([vm, {"manifest": huanle["manifest"], "cards": cards}]).to_string()
}

fn start(players: u8, packs: &str) -> Engine {
    let mut engine = engine_from_packs(players, SEED, packs).expect("load packs");
    assign_general_for(&mut engine, 0, "vm_fixture").expect("assign VM general");
    assert!(engine.view_for(0).players[0]
        .skills
        .contains(&"vm_fixture_phase_end_phase_end"));
    engine
}

fn run(engine: &mut Engine, count: usize) -> Vec<Value> {
    let mut records = Vec::new();
    let mut cursor = engine.log().len();
    for _ in 0..count {
        let before = engine.view_for(0);
        let choice = engine.open_choice().cloned();
        let choice_info = choice.as_ref().map(|c| {
            let hidden = c.observer != 0;
            if hidden {
                assert!(matches!(engine.choice_view_for(Some(0)), Some(engine_native::view::ChoiceView::Other { .. })));
            }
            json!({"id": c.id, "observer": c.observer, "min": c.spec.min, "count": c.spec.options.len(), "hidden": hidden})
        });
        let (actor, action) = if let Some(choice) = choice {
            (
                choice.observer,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: choice
                        .spec
                        .options
                        .iter()
                        .take(choice.spec.min as usize)
                        .cloned()
                        .collect(),
                },
            )
        } else {
            (engine.turn().current, GameAction::NextPhase)
        };
        let out = engine.submit(GameCommand {
            actor,
            expected_revision: engine.revision(),
            dedup_id: engine.revision().wrapping_mul(2).max(1),
            action,
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
        let codes = engine.public_event_codes_from(cursor);
        cursor = engine.log().len();
        let after = engine.view_for(0);
        records.push(json!({
            "choice": choice_info, "actor": actor,
            "before_turn": before.turn, "before_phase": format!("{:?}", before.phase).to_lowercase(),
            "before_hand0": before.players[0].hand_count,
            "revision": engine.revision(), "hash": hex_encode(&engine.state_hash()), "codes": codes,
            "turn": after.turn, "phase": format!("{:?}", after.phase).to_lowercase(),
            "hand0": after.players[0].hand_count,
        }));
    }
    records
}

#[test]
fn vm_and_installed_package_match_multiseat_restore_trajectories() {
    let packs = packs();
    let mut scenarios = Vec::new();
    for players in [4, 8] {
        let mut direct_engine = start(players, &packs);
        let direct = run(&mut direct_engine, STEPS);
        let checkpoint = direct
            .iter()
            .position(|record| record["choice"]["hidden"] == true)
            .expect("another seat has a private choice");
        assert!(
            direct.iter().any(|r| r["before_turn"] == 0
                && r["before_phase"] == "discard"
                && r["choice"].is_null()
                && r["turn"] == 0
                && r["phase"] == "end"
                && r["hand0"].as_u64().unwrap() == r["before_hand0"].as_u64().unwrap() + 1),
            "fixture must draw at its end phase"
        );
        let mut first = start(players, &packs);
        let before = run(&mut first, checkpoint);
        assert!(first.open_choice().is_some_and(|c| c.observer != 0));
        let saved = hex_encode(&engine_native::archive::encode_archive(&first.save()));
        let mut restored = engine_from_archive(&saved, &packs).expect("restore pending choice");
        let after = run(&mut restored, STEPS - checkpoint);
        assert_eq!(before, direct[..checkpoint]);
        assert_eq!(after, direct[checkpoint..]);
        scenarios.push(json!({"players": players, "seed": SEED, "direct": direct,
            "restore": {"midpoint": checkpoint, "before": before, "after": after}}));
    }
    if let Some(path) = std::env::var_os("NC08_MULTISEAT_OUTPUT") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({"packs": packs, "scenarios": scenarios})).unwrap(),
        )
        .unwrap();
    }
}

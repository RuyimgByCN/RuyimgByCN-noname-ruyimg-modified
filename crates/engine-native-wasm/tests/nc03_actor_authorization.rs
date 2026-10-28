//! Invalid seat IDs must not become another seat at the WASM command boundary.

use engine_native_wasm::NativeEngine;
use serde_json::{json, Value};

fn parse(raw: String) -> Value {
    serde_json::from_str(&raw).unwrap()
}

fn engine() -> NativeEngine {
    NativeEngine::new(4, 20261028).unwrap()
}

fn assert_rejected_unchanged(engine: &NativeEngine, before: &str, result: &Value, reason: &str) {
    assert_eq!(result["outcome"], "rejected", "{result}");
    assert_eq!(result["reason"], reason, "{result}");
    assert_eq!(
        engine.state_hash_hex(),
        before,
        "rejected command changed state"
    );
}

#[test]
fn out_of_range_actor_cannot_wrap_to_a_real_seat() {
    for kind in ["phase", "card", "skill"] {
        let mut e = engine();
        let before = e.state_hash_hex();
        let out = match kind {
            "phase" => e.next_phase(256),
            "card" => e.play_card(256, 0, vec![]),
            _ => e.activate_skill(256, 0),
        }
        .unwrap();
        assert_rejected_unchanged(&e, &before, &parse(out), "UnknownActor");
    }
}

#[test]
fn another_seat_cannot_answer_a_private_choice() {
    let mut e = engine();
    let choice = (0..100)
        .find_map(|_| {
            let value = parse(e.open_choice_json(0).unwrap());
            if value.is_null() {
                e.auto_step().unwrap();
                None
            } else {
                let owner = value["observer"].as_u64().unwrap() as u8;
                Some(parse(e.open_choice_json(owner).unwrap()))
            }
        })
        .expect("game must reach a choice");
    let owner = choice["observer"].as_u64().unwrap() as u32;
    let choice_id = choice["id"].as_u64().unwrap();
    let count = choice["min"].as_u64().unwrap() as u32;
    let selection: Vec<u32> = (0..count).collect();
    let before = e.state_hash_hex();
    let other = (owner + 1) % 4;
    let out = parse(e.answer(other, choice_id, selection.clone()).unwrap());
    assert_rejected_unchanged(&e, &before, &out, "WrongPlayer");
    let out = parse(e.answer(owner + 256, choice_id, selection.clone()).unwrap());
    assert_rejected_unchanged(&e, &before, &out, "UnknownActor");
    let out = parse(e.answer(owner, choice_id, selection).unwrap());
    assert!(
        out["outcome"] == "accepted" || out["outcome"] == "pending",
        "{out}"
    );
}

#[test]
fn out_of_range_target_cannot_wrap_to_a_real_seat() {
    let mut e = engine();
    let (actor, card, targets) = (0..100)
        .find_map(|_| {
            let view = parse(e.view_json(0).unwrap());
            let actor = view["turn"].as_u64().unwrap() as u32;
            let actions = parse(e.legal_actions_json(actor as u8).unwrap());
            if let Some(card) = actions["actions"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|action| action.get("PlayCard"))
                .find(|play| {
                    play["targets"]
                        .as_array()
                        .is_some_and(|targets| !targets.is_empty())
                })
            {
                let targets = card["targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u32)
                    .collect::<Vec<_>>();
                Some((actor, card["card"].as_u64().unwrap() as u32, targets))
            } else {
                e.auto_step().unwrap();
                None
            }
        })
        .expect("game must reach a targeted legal card");
    let before = e.state_hash_hex();
    let mut forged = targets.clone();
    forged[0] += 256;
    let out = parse(e.play_card(actor, card, forged).unwrap());
    assert_rejected_unchanged(&e, &before, &out, "IllegalTargets");
    let out = parse(e.play_card(actor, card, targets).unwrap());
    assert!(
        out["outcome"] == "accepted" || out["outcome"] == "pending",
        "{}",
        json!(out)
    );
}

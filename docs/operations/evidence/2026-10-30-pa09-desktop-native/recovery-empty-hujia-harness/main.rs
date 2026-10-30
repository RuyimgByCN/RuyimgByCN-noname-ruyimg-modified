use engine_native::{ai::BoundedPlanner, CommandOutcome, Engine};
use engine_native_wasm::{hex_encode, NativeSession};
use serde_json::{json, Value};

fn choices(session: &NativeSession) -> Vec<Value> {
    (0..8).map(|seat| serde_json::from_str(&session.open_choice_json(seat).unwrap()).unwrap()).collect()
}

fn main() {
    let output = std::env::args().nth(1).expect("output vector path");
    let mut engine = Engine::new(45000, 8);
    for seat in 0..8 {
        engine.assign_general_named(seat, ["曹操", "刘备", "孙权", "赵云"][seat as usize % 4]).unwrap();
    }
    let mut pending = None;
    for step in 0..100 {
        if let Some(choice) = engine.open_choice() {
            if choice.spec.options.is_empty() && choice.spec.min == 0 && !choice.spec.allow_cancel {
                assert_eq!(choice.spec.prompt, "hujia_respond");
                pending = Some((engine.save(), choice.observer, choice.id, step));
                break;
            }
        }
        let actor = engine.open_choice().map(|choice| choice.observer).unwrap_or(engine.turn().current);
        let plan = engine.plan_action(actor, &BoundedPlanner::new(8));
        assert!(!matches!(engine.apply_plan(&plan).unwrap(), CommandOutcome::Rejected(_)));
    }
    let (snapshot, observer, choice_id, fixture_steps) = pending.expect("real empty hujia window");
    let archive = hex_encode(&engine_native::archive::encode_archive(&snapshot));
    let mut manual = NativeSession::new(8, 45000).unwrap();
    manual.restore_hex_checked(&archive).unwrap();
    let restored_hex = manual.save_hex();
    let initial_choices = choices(&manual);
    let log_start = manual.log_len();
    let manual_reply = manual.answer(observer as u32, choice_id, vec![]).unwrap();
    let mut automatic = NativeSession::new(8, 45000).unwrap();
    automatic.restore_hex_checked(&archive).unwrap();
    assert_eq!(automatic.save_hex(), restored_hex);
    assert_eq!(automatic.log_len(), log_start);
    let reply = automatic.auto_step().unwrap();
    let parsed: Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(parsed["kind"], "answer");
    assert_eq!(parsed["outcome"], manual_reply);
    assert_eq!(automatic.save_hex(), manual.save_hex());
    assert_eq!(automatic.state_hash_hex(), manual.state_hash_hex());
    assert_eq!(choices(&automatic), choices(&manual));
    assert_eq!(automatic.public_event_codes_json(log_start), manual.public_event_codes_json(log_start));
    let vector = json!({
        "players": 8, "seed": 45000, "fixtureSteps": fixture_steps,
        "observer": observer, "choiceId": choice_id.to_string(), "archiveHex": archive,
        "expectedRestoredHex": restored_hex, "initialChoices": initial_choices,
        "logStart": log_start, "expectedManualReply": manual_reply,
        "expectedReplyJSON": parsed, "expectedReplyString": reply,
        "expectedEmptyAnswerSavedHex": manual.save_hex(), "expectedHash": manual.state_hash_hex(),
        "expectedRevision": manual.revision().to_string(), "expectedChoices": choices(&manual),
        "expectedEventCodes": serde_json::from_str::<Value>(&manual.public_event_codes_json(log_start)).unwrap(),
    });
    std::fs::write(output, serde_json::to_vec_pretty(&vector).unwrap()).unwrap();
    println!("{}", json!({"passed":true,"players":8,"seed":45000,"fixtureSteps":fixture_steps,"observer":observer,"choiceId":choice_id.to_string(),"prompt":"hujia_respond","options":[],"min":0,"allowCancel":false,"nativeManualEqualsAutomatic":true}));
}

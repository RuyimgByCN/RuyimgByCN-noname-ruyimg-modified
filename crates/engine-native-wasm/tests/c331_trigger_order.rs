//! C331 author package: formal general assignment, 257 choices, full native/WASM restart vector.
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use engine_native_wasm::{engine_from_packs, hex_encode, NativeSession};
use serde_json::{json, Value};
fn submit(e: &mut Engine, action: GameAction) {
    let actor = e
        .open_choice()
        .map(|c| c.observer)
        .unwrap_or(e.turn().current);
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision().wrapping_mul(2).max(1),
        action,
    });
    assert!(
        !matches!(out, engine_native::CommandOutcome::Rejected(_)),
        "{out:?}"
    );
}
fn snapshot(s: &NativeSession) -> Value {
    json!({"archive":s.save_hex(),"hash":s.state_hash_hex(),"views":(0..4).map(|seat| serde_json::from_str::<Value>(&s.view_json(seat).unwrap()).unwrap()).collect::<Vec<_>>(),
        "choices":(0..4).map(|seat| serde_json::from_str::<Value>(&s.open_choice_json(seat).unwrap()).unwrap()).collect::<Vec<_>>(),"events":serde_json::from_str::<Value>(&s.public_event_codes_json(0)).unwrap()})
}
#[test]
fn author_schema4_general_assignment_and_257th_skill_restart() {
    let names: Vec<_> = (0..257).map(|i| format!("c331_author_{i:03}")).collect();
    let source = names.iter().enumerate().map(|(i,name)| format!("skill {name}({}) locked {{ trigger phase_play_end owner_is_target {{ effect draw_cards(self, 1); }} }}", 4200001000u32 + i as u32)).collect::<Vec<_>>().join("\n");
    let mut package = content_compiler::compile_source_named("c331-author-257", &source).unwrap();
    package.manifest.version = "0.2.0".into();
    for t in &mut package.trigger_index {
        t.priority = 17;
    }
    package.manifest.content_schema_version = package.required_content_schema();
    package.translations.insert(
        "zh_CN".into(),
        names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), format!("排序技能{}", i + 1)))
            .collect(),
    );
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package.validate().unwrap();
    let mut item = serde_json::to_value(&package).unwrap();
    item["characters"] = json!({"generals":[{"name":"C331 author","hp":4,"faction":"qun","gender":"male","skills":names.iter().map(|name|json!({"name":name})).collect::<Vec<_>>() }]});
    let packs_json = json!([item]).to_string();
    for owner in [0u8, 1] {
        let mut e = engine_from_packs(4, 331257, &packs_json).unwrap();
        e.assign_general_named(owner, "C331 author").unwrap();
        for _ in 0..30 {
            if e.open_choice()
                .is_some_and(|c| c.spec.prompt == "trigger_order")
            {
                break;
            }
            let action = if let Some(c) = e.open_choice() {
                GameAction::Answer {
                    choice_id: c.id,
                    selection: c.spec.options.iter().take(c.spec.min).copied().collect(),
                }
            } else {
                GameAction::NextPhase
            };
            submit(&mut e, action);
        }
        let c = e.open_choice().unwrap();
        assert_eq!(c.spec.prompt, "trigger_order");
        assert_eq!(c.spec.options.len(), 257);
        assert!(matches!(c.spec.options[256], ChoiceOption::Skill(_)));
        assert_eq!(
            e.legal_actions(owner).len(),
            256,
            "legacy action query cap is independent of full ChoiceRequest"
        );
        let initial = hex_encode(&engine_native::archive::encode_archive(&e.save()));
        let mut session = NativeSession::new(4, 0).unwrap();
        session
            .restore_hex_with_packs_json(&initial, &packs_json)
            .unwrap();
        let before = snapshot(&session);
        assert_eq!(before["choices"][owner as usize]["kind"], "OrderTriggers");
        assert_eq!(
            before["choices"][owner as usize]["options"][256],
            json!({"skill":"c331_author_256"})
        );
        for seat in (0..4).filter(|s| *s != owner as usize) {
            assert!(before["choices"][seat]["options"].is_null());
        }
        let id = before["choices"][owner as usize]["id"].as_u64().unwrap();
        let bad = session
            .answer(u32::from((owner + 1) % 4), id, vec![256])
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&bad).unwrap()["outcome"],
            "rejected"
        );
        assert_eq!(snapshot(&session), before);
        let outcome: Value =
            serde_json::from_str(&session.answer(u32::from(owner), id, vec![256]).unwrap())
                .unwrap();
        assert_eq!(outcome["outcome"], "pending");
        let after = snapshot(&session);
        assert_eq!(
            after["choices"][owner as usize]["options"]
                .as_array()
                .unwrap()
                .len(),
            256
        );
        let mut resumed = NativeSession::new(4, 0).unwrap();
        resumed
            .restore_hex_with_packs_json(&initial, &packs_json)
            .unwrap();
        resumed.answer(u32::from(owner), id, vec![256]).unwrap();
        assert_eq!(snapshot(&resumed), after);
        if let Ok(root) = std::env::var("C331_VECTOR_EVIDENCE") {
            std::fs::write(std::path::Path::new(&root).join(if owner==0 {"author257-vector.json"}else{"author257-vector-seat1.json"}),serde_json::to_vec_pretty(&json!({"owner":owner,"initial":initial,"packs_json":packs_json,"before":before,"after":after,"choice_id":id,"answer":[256],"outcome":outcome})).unwrap()).unwrap();
        }
        if owner == 1 {
            if let Ok(root) = std::env::var("C331_PHASE3_VECTOR_EVIDENCE") {
                let mut tail = NativeSession::new(4, 0).unwrap();
                tail.restore_hex_with_packs_json(after["archive"].as_str().unwrap(), &packs_json)
                    .unwrap();
                let restart_before = snapshot(&tail);
                let mut continuation = Vec::new();
                for _ in 0..300 {
                    let c: Value =
                        serde_json::from_str(&tail.open_choice_json(owner).unwrap()).unwrap();
                    if c["kind"] != "OrderTriggers" {
                        break;
                    }
                    let choice_id = c["id"].as_u64().unwrap();
                    let result: Value = serde_json::from_str(
                        &tail.answer(u32::from(owner), choice_id, vec![0]).unwrap(),
                    )
                    .unwrap();
                    continuation.push(json!({"choice_id":choice_id,"answer":[0],"outcome":result,"after":snapshot(&tail)}));
                }
                assert_eq!(continuation.len(), 255);
                std::fs::write(std::path::Path::new(&root).join("author257-full-vector.json"),serde_json::to_vec(&json!({"owner":owner,"initial":initial,"packs_json":packs_json,"before":before,"after":after,"choice_id":id,"answer":[256],"outcome":outcome,"restart_before":restart_before,"continuation":continuation,"terminal":snapshot(&tail)})).unwrap()).unwrap();
            }
        }
        assert_eq!(e.turn().phase, Phase::Discard);
    }
}

use engine_native::event::DamageNature;
use engine_native::Engine;
use engine_native_wasm::{hex_encode, NativeSession};
use serde_json::Value;
#[test]
fn actual_control_uses_existing_string_custom_and_index_answer() {
    let mut e = Engine::new(15101, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "远策");
    e.debug_set_deck_top_by(|c| c.suit == engine_native::catalog::Suit::Heart);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let bytes = engine_native::archive::encode_archive(&e.save());
    let mut session = NativeSession::new(3, 15101).unwrap();
    session.restore_hex_checked(&hex_encode(&bytes)).unwrap();
    let raw = session.open_choice_json(2).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["kind"], "Custom");
    assert!(v["native_kind"].is_null());
    assert_eq!(
        v["options"],
        serde_json::json!(["yuance_lose_hp", "yuance_recover"])
    );
    let id = v["id"].as_u64().unwrap();
    let out = session.answer(2, id, vec![1]).unwrap();
    assert!(!out.contains("Rejected"), "{out}");
    assert_eq!(
        serde_json::from_str::<Value>(&session.open_choice_json(2).unwrap()).unwrap(),
        Value::Null
    );
}
#[test]
fn actual_hidden_card_uses_existing_generic_labels_and_index_answer() {
    let raw = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&raw).unwrap().pack;
    let mut e = Engine::new_with_extensions(15102, 4, &[&pack]).unwrap();
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    let id = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::CardKind::Qinglonglingzhu)
        .unwrap() as u32;
    let name = e.catalog().cards[id as usize].name;
    let equipped = e.debug_place(0, name);
    e.state_core_mut()
        .player_mut(0)
        .hand
        .retain(|c| *c != equipped);
    e.state_core_mut().player_mut(0).equips.push(equipped);
    e.debug_place(1, "tao");
    e.debug_place(1, "sha");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "c331_qinglong_card");
    let packs = serde_json::json!([serde_json::from_str::<Value>(&raw).unwrap()]).to_string();
    let mut session = NativeSession::new_with_packs_json(4, 15102, &packs).unwrap();
    session
        .restore_hex_with_packs_json(
            &hex_encode(&engine_native::archive::encode_archive(&e.save())),
            &packs,
        )
        .unwrap();
    let choice: Value = serde_json::from_str(&session.open_choice_json(0).unwrap()).unwrap();
    assert_eq!(choice["kind"], "SelectCards");
    assert_eq!(choice["native_kind"], "select_cards");
    assert_eq!(choice["options"], serde_json::json!(["hand", "hand"]));
    assert!(choice.get("source_skill").is_none());
    for seat in 1..4 {
        let other: Value = serde_json::from_str(&session.open_choice_json(seat).unwrap()).unwrap();
        assert!(other.get("options").is_none());
    }
    let out = session
        .answer(0, choice["id"].as_u64().unwrap(), vec![1])
        .unwrap();
    assert!(!out.contains("Rejected"), "{out}");
    assert_eq!(
        serde_json::from_str::<Value>(&session.open_choice_json(0).unwrap()).unwrap(),
        Value::Null
    );
}

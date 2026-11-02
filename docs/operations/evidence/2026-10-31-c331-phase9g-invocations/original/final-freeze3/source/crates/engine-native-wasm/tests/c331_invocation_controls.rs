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

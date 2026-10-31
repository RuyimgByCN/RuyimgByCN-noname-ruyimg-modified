//! Frozen 5fc source readers, not rewritten compatibility approximations.
#[path = "fixtures/i02-main5fc/messages.rs"]
mod messages;
#[path = "fixtures/i02-main5fc/wire.rs"]
mod wire;
pub use messages::*;
pub const NETWORK_PROTOCOL_VERSION: u32 = 6;
pub const MAX_MESSAGE_BYTES: usize = 256 * 1024;

#[test]
fn actual_old_reader_ignores_optional_new_response_field() {
    let input = serde_json::json!({"type":"CommandOutcome","payload":{
        "header":{"protocol_version":6,"room_id":1,"conn_seq":2,"revision":2},"idempotency_key":2,
        "result":{"Accepted":{"new_revision":2,"state_hash":vec![0;32]}},
        "view":{"revision":2,"state_hash":vec![0;32],"observer":"Spectator","round":1,"current_player":0,"phase":"Idle","seats":[],"known_cards":[],"outcome":"InProgress"},
        "legal_actions":[],"choice":null,"public_event_codes":[],
        "presentation":{"revision":2,"status":"complete","unavailable":[],"occurrences":[]}
    }});
    let old = wire::decode_s2c(&input.to_string()).unwrap();
    assert!(!wire::encode_s2c(&old).unwrap().contains("presentation"));
    assert!(engine_protocol::decode_s2c(&input.to_string()).is_ok());
}

#[test]
fn old_and_new_client_request_contract_remains_strict_and_unchanged() {
    let input = serde_json::json!({"type":"SnapshotRequest","payload":{"header":{"protocol_version":6,"room_id":1,"conn_seq":2,"revision":2},"last_revision":2}});
    assert!(wire::decode_c2s(&input.to_string()).is_ok());
    assert!(engine_protocol::decode_c2s(&input.to_string()).is_ok());
    let mut with_presentation=input.clone();
    with_presentation["payload"]["presentation"]=serde_json::json!({});
    assert!(wire::decode_c2s(&with_presentation.to_string()).is_err());
    assert!(engine_protocol::decode_c2s(&with_presentation.to_string()).is_err());
}

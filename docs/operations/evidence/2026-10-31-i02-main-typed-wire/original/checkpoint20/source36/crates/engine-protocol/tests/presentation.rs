use engine_api::PublicPresentationBatch;
use engine_protocol::{decode_s2c, encode_s2c, ServerMessage};
use serde_json::{json, Value};

fn batch() -> Value {
    serde_json::from_str(include_str!("fixtures/i02-presentation.json")).unwrap()
}
fn outcome(presentation: Option<Value>) -> Value {
    let seat = |player| json!({"player":player,"seat":player-1,"hp":4,"max_hp":4,"alive":true,"hand_count":0,"identity":null,"skills":[]});
    let mut v = json!({"type":"CommandOutcome","payload":{
        "header":{"protocol_version":6,"room_id":7,"conn_seq":9,"revision":2},"idempotency_key":9,
        "result":{"Accepted":{"new_revision":2,"state_hash":vec![0;32]}},
        "view":{"revision":2,"state_hash":vec![0;32],"observer":"Spectator","round":1,"current_player":1,"phase":"Play","seats":[seat(1),seat(2)],"known_cards":[],"outcome":"InProgress"},
        "legal_actions":[],"choice":null,"public_event_codes":[3,5,7]
    }});
    if let Some(p) = presentation {
        v["payload"]["presentation"] = p;
    }
    v
}

#[test]
fn old_absence_and_seven_typed_events_roundtrip() {
    let absent = decode_s2c(&outcome(None).to_string()).unwrap();
    assert!(!encode_s2c(&absent).unwrap().contains("presentation"));
    let message = decode_s2c(&outcome(Some(batch())).to_string()).unwrap();
    let roundtrip = decode_s2c(&encode_s2c(&message).unwrap()).unwrap();
    assert_eq!(message, roundtrip);
    let ServerMessage::CommandOutcome {
        presentation: Some(p),
        ..
    } = message
    else {
        panic!()
    };
    assert_eq!(p.occurrences.len(), 7);
}

#[test]
fn invalid_present_payloads_are_rejected_at_actual_wire_ingress() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/i02-presentation-invalid.json")).unwrap();
    for case in cases {
        let mut bad = batch();
        for change in case["changes"].as_array().unwrap() {
            let (parent, key) = change["pointer"]
                .as_str()
                .unwrap()
                .rsplit_once('/')
                .unwrap();
            let target = bad.pointer_mut(parent).unwrap();
            if change["remove"] == true {
                target.as_object_mut().unwrap().remove(key);
            } else if let Some(object) = target.as_object_mut() {
                object.insert(key.into(), change["value"].clone());
            } else {
                target[key.parse::<usize>().unwrap()] = change["value"].clone();
            }
        }
        assert!(
            decode_s2c(&outcome(Some(bad)).to_string()).is_err(),
            "{}",
            case["name"]
        );
    }
    assert!(decode_s2c(&outcome(Some(Value::Null)).to_string()).is_err());
    let mut too_many = batch();
    too_many["occurrences"] = Value::Array(
        (0..1025)
            .map(|i| json!({"ordinal":i,"event":{"kind":"hp_recovered","actor":1,"amount":1}}))
            .collect(),
    );
    assert!(serde_json::from_value::<PublicPresentationBatch>(too_many).is_err());
    let mut too_large = batch();
    let voice = too_large["occurrences"][0]["event"]["voice"].clone();
    too_large["occurrences"] = Value::Array((0..90).map(|i|json!({"ordinal":i,"event":{"kind":"player_died","actor":1,"killer":null,"voice":voice}})).collect());
    assert!(decode_s2c(&outcome(Some(too_large)).to_string()).is_err());
}

#[test]
fn package_and_original_identities_are_discriminated() {
    let mut b = batch();
    let owner = json!({"kind":"package","namespace":"author","package_id":"voices","version":"1.0.0","logic_hash":"1".repeat(64),"content_hash":"2".repeat(64)});
    let role = &mut b["occurrences"][0]["event"]["voice"]["role"];
    role["owner"] = owner.clone();
    role["local_id"] = json!(42);
    let asset = &mut b["occurrences"][0]["event"]["voice"]["assets"][0];
    asset["source"] = owner;
    asset["reference"] = json!({"kind":"package","namespace":"author","package_id":"voices","asset_id":"die_liubei"});
    asset["digest_algorithm"] = json!("blake3");
    assert!(decode_s2c(&outcome(Some(b.clone())).to_string()).is_ok());
    b["occurrences"][0]["event"]["voice"]["role"]["local_id"] =
        json!({"pack":"standard","raw_symbol":"liubei"});
    assert!(decode_s2c(&outcome(Some(b)).to_string()).is_err());
}

#[test]
fn raw_json_surrogates_reject_before_utf8_string_creation_and_valid_astral_pairs_pass() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/i02-presentation-surrogates.json")).unwrap();
    for case in cases {
        let mut bad = batch();
        *bad.pointer_mut(case["pointer"].as_str().unwrap()).unwrap() =
            json!("I02_RAW_UNICODE_SENTINEL");
        // Keep invalid escapes in raw JSON: parsing the injected value as Value first would lose this boundary.
        let raw = outcome(Some(bad)).to_string().replace(
            "\"I02_RAW_UNICODE_SENTINEL\"",
            case["raw_json_value"].as_str().unwrap(),
        );
        assert!(decode_s2c(&raw).is_err(), "{}", case["name"]);
    }
    let mut good = batch();
    good["occurrences"][0]["event"]["voice"]["role"]["owner"]["repository"] = json!("repo🐉");
    good["occurrences"][0]["event"]["effective"]["key"] = json!("sha🐉");
    good["occurrences"][0]["event"]["voice"]["assets"][0]["reference"]["path"] =
        json!("audio/die/role🐉.mp3");
    assert!(decode_s2c(&outcome(Some(good)).to_string()).is_ok());
}

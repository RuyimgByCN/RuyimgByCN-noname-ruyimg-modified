use super::*;
use engine_api::{ChoiceId, ChoiceRequest, PlayerId};

fn fixture() -> Engine {
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: engine_api::ExpectedRevision(0),
        idempotency_key: engine_api::IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 7,
            ruleset: "identity-standard".into(),
        },
    });
    engine.state.resolution.pending_choice_id = 7;
    engine.state.resolution.pending_observer = 2;
    engine.state.resolution.choice_revision = engine.state.revision;
    engine.state.resolution.continuation_id = 19;
    engine.last_choice=Some(serde_json::from_value(serde_json::json!({"choice_id":7,"observer":2,"options":[{"token":42,"label_key":"keep"}],"min_count":1,"max_count":1,"allow_cancel":true,"timeout_policy":"Standard","deadline_unix_ms":123456,"revision":engine.state.revision,"continuation_id":19,"kind":"Confirm","prompt":"trigger_confirm","native_kind":"confirm"})).unwrap());
    engine
}
fn stripped(c: &ChoiceRequest) -> ChoiceRequest {
    let mut c = c.clone();
    c.source_skill = None;
    c
}
#[test]
fn source_typed_capture_and_both_restores_without_json() {
    let mut engine = fixture();
    engine.last_choice.as_mut().unwrap().source_skill = Some("forged".into());
    let expected = stripped(engine.last_choice.as_ref().unwrap());
    let captured = engine.snapshot();
    assert_eq!(
        captured.pending_choice.as_ref(),
        Some(&expected),
        "typed capture must strip only source"
    );
    assert_eq!(
        engine.last_choice.as_ref().unwrap().source_skill.as_deref(),
        Some("forged")
    );
    for source in [Some("forged".to_owned()), Some(String::new()), None] {
        let mut snap = captured.clone();
        snap.pending_choice.as_mut().unwrap().source_skill = source;
        let original = snap.clone();
        let mut restored = Engine::new();
        restored.restore_snapshot(&snap).unwrap();
        assert_eq!(restored.last_choice(), Some(&expected));
        assert_eq!(snap, original);
        let mut restored = Engine::new();
        restored
            .restore_snapshot_with_manifest(&snap, engine.manifest().unwrap().clone())
            .unwrap();
        assert_eq!(restored.last_choice(), Some(&expected));
        assert_eq!(snap, original);
        assert_eq!(restored.manifest_hash(), engine.manifest_hash());
        assert_eq!(restored.state_hash(), engine.state_hash());
    }
}
#[test]
fn source_json_ignores_before_wire_validation_and_preserves_duplicates() {
    let engine = fixture();
    let snap = engine.snapshot();
    let expected = snap.pending_choice.clone();
    let clean = serde_json::to_string(&snap).unwrap();
    for value in ["null", "\"forged\"", "\"\"", "123", "{}", "[]", "true"] {
        let raw = clean.replacen(
            "\"pending_choice\":{",
            &format!("\"pending_choice\":{{\"source_skill\":{value},\"source_skill\":{{}},"),
            1,
        );
        let decoded: Snapshot = serde_json::from_str(&raw)
            .expect("legacy must ignore source before strict String validation");
        assert_eq!(decoded.pending_choice, expected);
        assert_eq!(
            Snapshot::from_bytes(raw.as_bytes()).unwrap().pending_choice,
            expected
        );
        let duplicate = raw.replacen("\"choice_id\":7", "\"choice_id\":7,\"choice_id\":7", 1);
        assert!(serde_json::from_str::<Snapshot>(&duplicate).is_err());
    }
    for source in ["forged", ""] {
        let mut poisoned = snap.clone();
        poisoned.pending_choice.as_mut().unwrap().source_skill = Some(source.into());
        assert_eq!(poisoned.to_bytes().unwrap(), snap.to_bytes().unwrap());
        assert_eq!(
            serde_json::to_vec(&poisoned).unwrap(),
            snap.to_bytes().unwrap()
        );
    }
}
#[test]
fn source_wire_remains_strict_and_legacy_errors_atomic() {
    let engine = fixture();
    let snap = engine.snapshot();
    let choice = serde_json::to_value(snap.pending_choice.as_ref().unwrap()).unwrap();
    for value in [
        serde_json::json!(""),
        serde_json::json!(123),
        serde_json::json!({}),
        serde_json::json!([]),
        serde_json::json!(true),
    ] {
        let mut wire = choice.clone();
        wire["source_skill"] = value;
        assert!(serde_json::from_value::<ChoiceRequest>(wire).is_err());
    }
    for value in [serde_json::Value::Null, serde_json::json!("遗计")] {
        let mut wire = choice.clone();
        wire["source_skill"] = value.clone();
        let decoded: ChoiceRequest = serde_json::from_value(wire).unwrap();
        assert_eq!(decoded.source_skill, value.as_str().map(str::to_owned));
    }
    let mut bad = snap.clone();
    bad.pending_choice.as_mut().unwrap().source_skill = Some("forged".into());
    bad.pending_choice.as_mut().unwrap().choice_id = ChoiceId::new(99);
    let mut restored = fixture();
    let before = restored.snapshot();
    assert!(restored.restore_snapshot(&bad).is_err());
    assert_eq!(restored.snapshot(), before);
    let mut wrong = snap.clone();
    wrong.manifest_hash = engine_api::MatchManifestHash([99; 32]);
    assert!(restored
        .restore_snapshot_with_manifest(&wrong, engine.manifest().unwrap().clone())
        .is_err());
    assert_eq!(restored.snapshot(), before);
}

#[test]
fn source_json_bad_types_are_ignored_before_string_validation() {
    let clean = serde_json::to_string(&fixture().snapshot()).unwrap();
    for value in ["123", "{}", "[]", "true", "\"\""] {
        let raw = clean.replacen(
            "\"pending_choice\":{",
            &format!("\"pending_choice\":{{\"source_skill\":{value},"),
            1,
        );
        let decoded: Snapshot =
            serde_json::from_str(&raw).expect("old archive reader ignores arbitrary source value");
        assert_eq!(decoded.pending_choice.unwrap().source_skill, None);
    }
}
#[test]
fn source_typed_restore_fake_without_json() {
    let engine = fixture();
    let mut snap = engine.snapshot();
    snap.pending_choice.as_mut().unwrap().source_skill = Some("forged".into());
    let original = snap.clone();
    let mut restored = Engine::new();
    restored.restore_snapshot(&snap).unwrap();
    assert_eq!(restored.last_choice().unwrap().source_skill, None);
    assert_eq!(snap, original);
}
#[test]
fn source_typed_manifest_restore_fake_without_json() {
    let engine = fixture();
    let mut snap = engine.snapshot();
    snap.pending_choice.as_mut().unwrap().source_skill = Some("forged".into());
    let original = snap.clone();
    let mut restored = Engine::new();
    restored
        .restore_snapshot_with_manifest(&snap, engine.manifest().unwrap().clone())
        .unwrap();
    assert_eq!(restored.last_choice().unwrap().source_skill, None);
    assert_eq!(snap, original);
}

#[test]
fn source_legacy_positional_choice_keeps_old_length_and_fields() {
    let snap = fixture().snapshot();
    let mut json = serde_json::to_value(&snap).unwrap();
    json["pending_choice"] = serde_json::json!([7,2,[{"token":42,"label_key":"keep"}],1,1,true,"Standard",123456,snap.revision,19,"Confirm","trigger_confirm","confirm"]);
    let raw = serde_json::to_vec(&json).unwrap();
    assert_eq!(
        Snapshot::from_bytes(&raw).unwrap().pending_choice,
        snap.pending_choice
    );
    json["pending_choice"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!("forged"));
    assert!(
        Snapshot::from_bytes(&serde_json::to_vec(&json).unwrap()).is_err(),
        "old positional archives reject excess elements"
    );
}

#[test]
fn source_state_only_and_original_restore_errors_are_unchanged() {
    let engine = fixture();
    let snap = engine.snapshot();
    let expected = snap.restore().unwrap();
    for source in [None, Some("forged".to_owned()), Some(String::new())] {
        let mut candidate = snap.clone();
        candidate.pending_choice.as_mut().unwrap().source_skill = source;
        assert_eq!(candidate.restore().unwrap(), expected);
        assert_eq!(
            candidate
                .restore_with_manifest(engine.manifest_hash())
                .unwrap(),
            expected
        );
    }
    let mut schema = snap.clone();
    schema.schema_version += 1;
    let mut hash = snap.clone();
    hash.state_hash[0] ^= 1;
    let mut choice = snap.clone();
    choice.pending_choice.as_mut().unwrap().observer = PlayerId::new(99);
    for bad in [schema, hash, choice] {
        let expected = bad.restore().unwrap_err();
        let mut poisoned = bad.clone();
        poisoned.pending_choice.as_mut().unwrap().source_skill = Some("forged".into());
        assert_eq!(poisoned.restore().unwrap_err(), expected);
        let raw = serde_json::to_string(&bad).unwrap().replacen(
            "\"pending_choice\":{",
            "\"pending_choice\":{\"source_skill\":{},",
            1,
        );
        assert_eq!(
            Snapshot::from_bytes(raw.as_bytes())
                .unwrap()
                .restore()
                .unwrap_err(),
            expected
        );
    }
    let mut json = serde_json::to_value(&snap).unwrap();
    json["pending_choice"]["min_count"] = serde_json::json!("bad");
    let raw = serde_json::to_string(&json).unwrap().replacen(
        "\"pending_choice\":{",
        "\"pending_choice\":{\"source_skill\":{},",
        1,
    );
    assert!(Snapshot::from_bytes(raw.as_bytes()).is_err());
}

#[test]
fn source_does_not_bypass_legacy_content_fingerprint_errors() {
    let mut engine = fixture();
    let package=content_compiler::compile_source_named("source-legacy","skill legacy_source(99107928) { trigger phase_end owner_is_target { effect draw_cards(self,1); } }").unwrap();
    engine.load_package(&package).unwrap();
    let snapshot = engine.snapshot();
    let mut value = serde_json::to_value(&snapshot).unwrap();
    let module = value["state"]["content"]["modules"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap();
    module["logic_hash"] = serde_json::to_value([0u8; 32]).unwrap();
    let mut expected: Snapshot = serde_json::from_value(value.clone()).unwrap();
    expected.state_hash = crate::hash::canonical_state_hash(&expected.state);
    let error = expected.restore().unwrap_err();
    assert!(matches!(
        error,
        crate::SnapshotError::ContentCompatibility(_)
    ));
    let raw = serde_json::to_string(&expected).unwrap().replacen(
        "\"pending_choice\":{",
        "\"pending_choice\":{\"source_skill\":{},",
        1,
    );
    assert_eq!(
        Snapshot::from_bytes(raw.as_bytes())
            .unwrap()
            .restore()
            .unwrap_err(),
        error
    );
}

#[path = "source_tests/legacy_choice_errors.rs"]
mod legacy_choice_errors;

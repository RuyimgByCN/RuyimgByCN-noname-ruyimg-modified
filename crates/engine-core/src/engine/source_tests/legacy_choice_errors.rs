//! Exact old archive diagnostics, including serde_json's original location.
use crate::{Snapshot, SnapshotError};
use serde::Deserialize;

#[path = "choice_request_4dd.rs"]
mod old;

// Same envelope field names, order and defaults as 4dd; only the choice uses its frozen DTO.
#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename = "Snapshot")]
struct OldSnapshot {
    schema_version: u32,
    versions: engine_api::VersionBundle,
    revision: u64,
    state_hash: [u8; 32],
    #[serde(default)]
    manifest_hash: engine_api::MatchManifestHash,
    #[serde(default)]
    pending_choice: Option<old::ChoiceRequest>,
    state: crate::state::GameState,
}

fn fields() -> Vec<(&'static str, String)> {
    [
        ("choice_id", "7"),
        ("observer", "2"),
        ("options", r#"[{"token":42,"label_key":"keep"}]"#),
        ("min_count", "1"),
        ("max_count", "1"),
        ("allow_cancel", "true"),
        ("timeout_policy", r#""Standard""#),
        ("deadline_unix_ms", "123456"),
        ("revision", "1"),
        ("continuation_id", "19"),
        ("kind", r#""Confirm""#),
        ("prompt", r#""trigger_confirm""#),
        ("native_kind", r#""confirm""#),
    ]
    .map(|(name, value)| (name, value.to_owned()))
    .to_vec()
}

fn object(fields: &[(&str, String)]) -> String {
    format!(
        "{{{}}}",
        fields
            .iter()
            .map(|(name, value)| format!("\"{name}\":{value}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn envelope(pending: &str) -> String {
    let plain =
        serde_json::to_string(&Snapshot::capture(&crate::state::GameState::empty())).unwrap();
    // Multiline input makes loss of source positions observable as well.
    plain.replacen(
        "\"pending_choice\":null",
        &format!("\"pending_choice\":\n  {pending}"),
        1,
    )
}

#[test]
fn legacy_choice_errors_match_4dd_oracle() {
    let fields = fields();
    let valid = object(&fields);
    let mut cases: Vec<(String, String)> = Vec::new();
    for pending in [
        "null", "0", "-1", "1.5", "true", "false", "\"text\"", "\"\"", "{}",
    ] {
        cases.push((format!("pending/{pending}"), envelope(pending)));
    }
    for length in 0..=14 {
        let mut values = fields
            .iter()
            .map(|(_, value)| value.as_str())
            .take(length)
            .collect::<Vec<_>>();
        if length == 14 {
            values.push("null");
        }
        cases.push((
            format!("sequence/{length}"),
            envelope(&format!("[{}]", values.join(","))),
        ));
    }
    let source_values = [
        "null",
        "\"遗计\"",
        "\"\"",
        "123",
        "{}",
        "[]",
        "true",
        r#"{"source_skill":[{},null,{"choice_id":9}],"min_count":"untouched"}"#,
    ];
    for (index, value) in source_values.iter().enumerate() {
        for key in ["source_skill", r"source_\u0073kill"] {
            let with_source = format!(r#"{{"{key}":{value},{}"#, &valid[1..]);
            cases.push((format!("source/{index}/{key}"), envelope(&with_source)));
            let duplicate = format!(r#"{{"{key}":{value},"source_skill":{{}},{}"#, &valid[1..]);
            cases.push((
                format!("source-duplicate/{index}/{key}"),
                envelope(&duplicate),
            ));
        }
    }
    for index in 0..fields.len() {
        let (name, value) = &fields[index];
        let mut missing = fields.clone();
        missing.remove(index);
        cases.push((format!("missing/{name}"), envelope(&object(&missing))));
        for duplicate_value in [value.as_str(), "null", "{}"] {
            let raw = format!(r#"{{"{name}":{duplicate_value},{}"#, &valid[1..]);
            cases.push((
                format!("duplicate/{name}/{duplicate_value}"),
                envelope(&raw),
            ));
            cases.push((
                format!("duplicate-with-source/{name}/{duplicate_value}"),
                envelope(&format!(r#"{{"source_skill":{{}},{}"#, &raw[1..])),
            ));
        }
        for bad in [
            "null",
            "false",
            "0",
            "-1",
            "1.25",
            "18446744073709551616",
            "\"\"",
            "\"bad\"",
            "[]",
            "{}",
            "[null]",
        ] {
            let mut changed = fields.clone();
            changed[index].1 = bad.into();
            let raw = object(&changed);
            cases.push((format!("type/{name}/{bad}"), envelope(&raw)));
            cases.push((
                format!("type-with-source/{name}/{bad}"),
                envelope(&format!(r#"{{"source_skill":[{{}}],{}"#, &raw[1..])),
            ));
        }
    }
    for pending in [
        r#"{"source_skill":,"choice_id":7}"#,
        r#"{"source_skill":[1,],"choice_id":7}"#,
        r#"{"source_skill":{"x":},"choice_id":7}"#,
        r#"{"source_skill":"\uZZZZ"}"#,
        r#"{"source_skill":tru}"#,
        r#"{"source_skill":{]"#,
        r#"{"choice_id":7,"source_skill":["#,
    ] {
        cases.push((format!("malformed/{pending}"), envelope(pending)));
    }
    cases.push((
        "trailing/full-snapshot".into(),
        format!("{} true", envelope(&valid)),
    ));
    cases.push((
        "trailing/pending".into(),
        envelope(&format!("{valid} false")),
    ));
    cases.push((
        "nested-option-unknown".into(),
        envelope(&valid.replace(
            r#""label_key":"keep""#,
            r#""label_key":"keep","source_skill":{"token":"untouched"}"#,
        )),
    ));
    cases.push((
        "nested-option-duplicate".into(),
        envelope(&valid.replace(
            r#""token":42"#,
            r#""token":42,"token":42,"source_skill":{}"#,
        )),
    ));
    // The original reader also ignored other unknown fields and repeated unknown keys.
    cases.push((
        "unknown-duplicate".into(),
        envelope(&format!(r#"{{"future":{{}},"future":[],{}"#, &valid[1..])),
    ));

    let mut results = Vec::new();
    let mut differences = Vec::new();
    for (label, raw) in cases {
        let expected = serde_json::from_str::<OldSnapshot>(&raw)
            .map(|_| ())
            .map_err(|error| SnapshotError::Deserialize(error.to_string()));
        let actual = Snapshot::from_bytes(raw.as_bytes()).map(|_| ());
        let direct = serde_json::from_str::<Snapshot>(&raw)
            .map(|_| ())
            .map_err(|error| SnapshotError::Deserialize(error.to_string()));
        if actual != expected || direct != expected {
            differences.push(format!(
                "{label}: old={expected:?}; bytes={actual:?}; direct={direct:?}"
            ));
        }
        results.push(serde_json::json!({"case":label,"raw":raw,"old":format!("{expected:?}"),"bytes":format!("{actual:?}"),"direct":format!("{direct:?}"),"equal":actual==expected&&direct==expected}));
    }
    eprintln!(
        "LEGACY_ERROR_ORACLE cases={} differences={}",
        results.len(),
        differences.len()
    );
    for difference in &differences {
        eprintln!("{difference}");
    }
    if let Ok(path) = std::env::var("NC27_ERROR_MATRIX_OUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
    assert!(
        differences.is_empty(),
        "full old/new error differences: {}",
        differences.join("\n")
    );
}
